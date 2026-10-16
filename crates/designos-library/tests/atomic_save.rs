//! `write_atomic` must behave like a plain `fs::write` for links, modes and read-only files,
//! and must stay safe when many saves race for one path.

use designos_library::write_atomic;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("atomic-save-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn leftover_tmp_files(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).filter(|n| n.ends_with(".tmp")).collect()
}

#[cfg(unix)]
#[test]
fn saving_through_a_symlink_updates_the_real_file_and_keeps_the_link() {
    let dir = scratch("symlink");
    let real = dir.join("real.designos");
    let link = dir.join("link.designos");
    std::fs::write(&real, b"old").unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();

    write_atomic(&link, b"new").unwrap();

    assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink(), "the link must stay a link");
    assert_eq!(std::fs::read(&real).unwrap(), b"new", "the real file receives the bytes");
    assert_eq!(std::fs::read(&link).unwrap(), b"new");
    assert!(leftover_tmp_files(&dir).is_empty());
}

#[cfg(unix)]
#[test]
fn saving_through_a_symlink_into_another_directory_writes_next_to_the_real_file() {
    let dir = scratch("symlink-dir");
    let elsewhere = dir.join("elsewhere");
    std::fs::create_dir_all(&elsewhere).unwrap();
    let real = elsewhere.join("real.designos");
    let link = dir.join("link.designos");
    std::fs::write(&real, b"old").unwrap();
    std::os::unix::fs::symlink(&real, &link).unwrap();

    write_atomic(&link, b"new").unwrap();

    assert_eq!(std::fs::read(&real).unwrap(), b"new");
    assert!(std::fs::symlink_metadata(&link).unwrap().file_type().is_symlink());
    assert!(leftover_tmp_files(&dir).is_empty() && leftover_tmp_files(&elsewhere).is_empty());
}

#[cfg(unix)]
#[test]
fn saving_keeps_the_existing_permission_mode() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("mode");
    let file = dir.join("private.designos");
    std::fs::write(&file, b"old").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o600)).unwrap();

    write_atomic(&file, b"new").unwrap();

    assert_eq!(std::fs::read(&file).unwrap(), b"new");
    assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
}

#[cfg(unix)]
#[test]
fn read_only_target_is_refused_and_left_untouched() {
    use std::os::unix::fs::PermissionsExt;
    let dir = scratch("readonly");
    let file = dir.join("locked.designos");
    std::fs::write(&file, b"old").unwrap();
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o444)).unwrap();

    let err = write_atomic(&file, b"new").expect_err("a read-only file must not be replaced");

    assert!(err.to_string().contains("read-only"), "error names the cause: {err}");
    assert_eq!(std::fs::read(&file).unwrap(), b"old");
    assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o444);
    assert!(leftover_tmp_files(&dir).is_empty());
}

#[test]
fn directory_target_is_refused() {
    let dir = scratch("dir-target");
    let sub = dir.join("sub.designos");
    std::fs::create_dir(&sub).unwrap();
    assert!(write_atomic(&sub, b"x").is_err());
    assert!(sub.is_dir());
}

#[test]
fn missing_file_and_parent_directories_are_created() {
    let dir = scratch("create");
    let file = dir.join("a").join("b").join("new.designos");
    write_atomic(&file, b"fresh").unwrap();
    assert_eq!(std::fs::read(&file).unwrap(), b"fresh");
    assert!(leftover_tmp_files(file.parent().unwrap()).is_empty());
}

#[test]
fn forty_concurrent_saves_of_one_file_all_succeed_and_none_is_torn() {
    const SAVERS: usize = 40;
    const LEN: usize = 64 * 1024;
    let dir = scratch("concurrent");
    let file = dir.join("doc.designos");
    std::fs::write(&file, b"seed").unwrap();
    let start = Arc::new(Barrier::new(SAVERS));

    let handles: Vec<_> = (0..SAVERS)
        .map(|i| {
            let (file, start) = (file.clone(), start.clone());
            std::thread::spawn(move || {
                let bytes = vec![b'a' + (i % 26) as u8; LEN];
                start.wait();
                write_atomic(&file, &bytes).map_err(|e| e.to_string())
            })
        })
        .collect();
    let failures: Vec<String> = handles.into_iter().filter_map(|h| h.join().unwrap().err()).collect();

    assert!(failures.is_empty(), "{} of {SAVERS} saves failed: {failures:?}", failures.len());
    let content = std::fs::read(&file).unwrap();
    assert_eq!(content.len(), LEN, "the file holds one complete save");
    assert!(content.iter().all(|b| *b == content[0]), "the file mixes bytes of two saves");
    assert!(leftover_tmp_files(&dir).is_empty(), "temporary files left behind");
}
