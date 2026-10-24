//! Safe `.fig` export targets for the workspace session and direct engine calls.
//!
//! Paths supplied by agents are untrusted: inspect the final directory entry without
//! following links, create new files exclusively, and replace an existing regular file
//! only after an explicit opt-in.

use crate::tools::errors::{ErrorCode, ToolError};
use serde_json::json;
use std::{
    fs::{self, File, Metadata, OpenOptions},
    io::ErrorKind,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

/// Resolve a workspace-relative export path and validate its final directory entry.
pub fn resolve_workspace_path(root: &Path, requested: &str, overwrite: bool) -> Result<PathBuf, ToolError> {
    let joined = root.join(requested);
    require_fig_extension(&joined, requested)?;
    let parent = joined
        .parent()
        .ok_or_else(|| ToolError::new(ErrorCode::InvalidArguments, "/output_path", "Export path needs a parent directory"))?
        .canonicalize()
        .map_err(|e| {
            if requested.contains("..") || Path::new(requested).is_absolute() {
                ToolError::new(ErrorCode::PathOutsideWorkspace, "/output_path", format!("Path must stay inside the configured workspace: {e}"))
                    .with_received(json!(requested))
            } else {
                ToolError::new(ErrorCode::InvalidArguments, "/output_path", format!("Export directory must already exist: {e}")).with_received(json!(requested))
            }
        })?;
    if !parent.starts_with(root) {
        return Err(
            ToolError::new(ErrorCode::PathOutsideWorkspace, "/output_path", "Path must stay inside the configured workspace").with_received(json!(requested))
        );
    }
    let path = parent.join(joined.file_name().ok_or_else(|| ToolError::new(ErrorCode::InvalidArguments, "/output_path", "Export path needs a filename"))?);
    inspect_target(&path, requested, overwrite)?;
    Ok(path)
}

/// Write bytes while preserving the no-follow and no-implicit-overwrite contract.
pub fn write_export(path: &Path, display_path: &str, bytes: &[u8], overwrite: bool) -> Result<(), ToolError> {
    require_fig_extension(path, display_path)?;
    let existing = inspect_target(path, display_path, overwrite)?;
    match existing {
        None => create_new(path, display_path, bytes),
        Some(old) => replace(path, display_path, bytes, &old),
    }
}

fn require_fig_extension(path: &Path, display_path: &str) -> Result<(), ToolError> {
    if path.extension().and_then(|e| e.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("fig")) {
        return Ok(());
    }
    Err(ToolError::new(ErrorCode::InvalidArguments, "/output_path", "Export path must end in .fig").with_received(json!(display_path)).with_expected("*.fig"))
}

fn inspect_target(path: &Path, display_path: &str, overwrite: bool) -> Result<Option<Metadata>, ToolError> {
    let meta = match fs::symlink_metadata(path) {
        Ok(meta) => meta,
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(ToolError::new(ErrorCode::InternalError, "/output_path", format!("Cannot inspect export target: {e}"))),
    };
    if meta.file_type().is_symlink() {
        return Err(ToolError::new(
            ErrorCode::InvalidArguments,
            "/output_path",
            "Cannot write a .fig export through a symbolic link; choose a regular file path",
        )
        .with_received(json!(display_path)));
    }
    if !meta.is_file() {
        return Err(
            ToolError::new(ErrorCode::InvalidArguments, "/output_path", "Cannot overwrite a non-regular export target").with_received(json!(display_path))
        );
    }
    if !overwrite {
        return Err(ToolError::new(
            ErrorCode::FileExists,
            "/output_path",
            "Cannot replace an existing .fig file without overwrite=true; choose a new path or opt in",
        )
        .with_received(json!(display_path))
        .with_corrected_example(json!({"output_path": display_path, "overwrite": true})));
    }
    Ok(Some(meta))
}

fn create_new(path: &Path, display_path: &str, bytes: &[u8]) -> Result<(), ToolError> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(|e| {
        if e.kind() == ErrorKind::AlreadyExists {
            ToolError::new(ErrorCode::FileExists, "/output_path", "Cannot replace an existing .fig file without overwrite=true; choose a new path or opt in")
                .with_received(json!(display_path))
                .with_corrected_example(json!({"output_path": display_path, "overwrite": true}))
        } else {
            ToolError::new(ErrorCode::InternalError, "/output_path", format!("Cannot create .fig file: {e}"))
        }
    })?;
    if let Err(e) = file.write_all(bytes).and_then(|_| file.sync_all()) {
        drop(file);
        let _ = fs::remove_file(path);
        return Err(ToolError::new(ErrorCode::InternalError, "/output_path", format!("Cannot write .fig file: {e}")));
    }
    Ok(())
}

fn replace(path: &Path, display_path: &str, bytes: &[u8], old: &Metadata) -> Result<(), ToolError> {
    let parent = path.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let (tmp, mut file) =
        create_temp_beside(parent).map_err(|e| ToolError::new(ErrorCode::InternalError, "/output_path", format!("Cannot prepare .fig file: {e}")))?;
    let result = (|| {
        file.write_all(bytes)?;
        file.sync_all()?;
        file.set_permissions(old.permissions())?;
        drop(file);
        fs::rename(&tmp, path)
    })();
    if let Err(e) = result {
        let _ = fs::remove_file(&tmp);
        return Err(ToolError::new(ErrorCode::InternalError, "/output_path", format!("Cannot replace .fig file {display_path}: {e}")));
    }
    Ok(())
}

fn create_temp_beside(dir: &Path) -> std::io::Result<(PathBuf, File)> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    for _ in 0..16 {
        let path = dir.join(format!(".designos-mcp-export-{}-{}.tmp", std::process::id(), TMP_SEQ.fetch_add(1, Ordering::Relaxed)));
        match options.open(&path) {
            Ok(file) => return Ok((path, file)),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(ErrorKind::AlreadyExists, "no free temporary file name"))
}
