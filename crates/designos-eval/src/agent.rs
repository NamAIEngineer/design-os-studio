//! Agent CLI runner, token usage parsers, scripted `stub-agent`, and in-process/stdio MCP session (E15-25 §3.5, §3.6, §3.10).

use crate::config::{AgentProfile, CommandVars, UsageFormat};
use crate::jsonl::RunExit;
use crate::sandbox::RunSandbox;
use designos_doc::{Command as DocCommand, Document, History, Node, NodeId, NodeKind, NodeSpec, Paint, Rect};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Result of invoking an agent CLI process for one run.
#[derive(Clone, Debug)]
pub struct AgentRunOutcome {
    pub exit: RunExit,
    pub wall_ms: u64,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
    pub stderr_note: Option<String>,
}

/// Checks whether a binary name or path exists and is executable on `PATH`.
pub fn binary_exists_on_path(bin: &str) -> bool {
    let p = Path::new(bin);
    if p.components().count() > 1 {
        return p.is_file();
    }
    if let Ok(exe) = std::env::current_exe() {
        if exe.file_stem().and_then(|s| s.to_str()) == Some(bin) {
            return true;
        }
        if let Some(dir) = exe.parent() {
            if dir.join(bin).is_file() {
                return true;
            }
        }
    }
    if let Some(path_os) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_os) {
            if dir.join(bin).is_file() {
                return true;
            }
            #[cfg(windows)]
            if dir.join(format!("{bin}.exe")).is_file() {
                return true;
            }
        }
    }
    false
}

/// Resolves a binary name to its full path when possible (including sibling target binary `designos-eval`).
pub fn resolve_binary_path(bin: &str) -> Option<PathBuf> {
    let p = Path::new(bin);
    if p.components().count() > 1 {
        return p.is_file().then(|| p.to_path_buf());
    }
    if let Ok(exe) = std::env::current_exe() {
        if exe.file_stem().and_then(|s| s.to_str()) == Some(bin) {
            return Some(exe);
        }
        if let Some(dir) = exe.parent() {
            let sibling = dir.join(bin);
            if sibling.is_file() {
                return Some(sibling);
            }
            if let Some(parent) = dir.parent() {
                let parent_sibling = parent.join(bin);
                if parent_sibling.is_file() {
                    return Some(parent_sibling);
                }
            }
        }
    }
    if let Some(path_os) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path_os) {
            let cand = dir.join(bin);
            if cand.is_file() {
                return Some(cand);
            }
        }
    }
    None
}

/// Runs the configured agent CLI command in `sandbox` with `timeout_s`, returning [`AgentRunOutcome`].
pub fn execute_agent_profile(
    profile: &AgentProfile,
    vars: &CommandVars<'_>,
    sandbox: &RunSandbox,
    timeout_s: u64,
    stub_script_override: Option<&Path>,
) -> Result<AgentRunOutcome, String> {
    let expanded = profile.expand_command(vars);
    let Some(bin_name) = expanded.first() else {
        return Err("invalid_config: empty command after template expansion".into());
    };

    // Fast path for `designos-eval stub-agent --script ...` so unit/integration tests don't even
    // require a separate binary spawn if `bin_name == "stub-agent"` or points to `designos-eval` with `stub-agent`.
    if bin_name == "stub-agent" || (expanded.get(1).map(String::as_str) == Some("stub-agent")) {
        let t0 = Instant::now();
        let script_path = if let Some(s) = stub_script_override {
            s.to_path_buf()
        } else {
            let idx = expanded.iter().position(|a| a == "--script").ok_or_else(|| "stub_agent_error: missing --script <file.jsonl>".to_string())?;
            PathBuf::from(expanded.get(idx + 1).ok_or_else(|| "stub_agent_error: missing path after --script".to_string())?)
        };
        let res = run_stub_agent(&script_path, vars.mcp_config);
        let wall_ms = t0.elapsed().as_millis() as u64;
        return match res {
            Ok(()) => Ok(AgentRunOutcome { exit: RunExit::Ok, wall_ms, tokens_in: None, tokens_out: None, stderr_note: None }),
            Err(e) => Ok(AgentRunOutcome { exit: RunExit::Crashed, wall_ms, tokens_in: None, tokens_out: None, stderr_note: Some(e) }),
        };
    }

    let Some(resolved_bin) = resolve_binary_path(bin_name) else {
        return Err(format!("agent_not_found: binary '{bin_name}' is not on PATH; install it or update designos-eval.toml"));
    };

    let t0 = Instant::now();
    let mut cmd = Command::new(&resolved_bin);
    cmd.args(&expanded[1..]).current_dir(&sandbox.workspace_dir).stdin(Stdio::null()).stdout(Stdio::piped()).stderr(Stdio::piped());
    sandbox.configure_child_env(&mut cmd);

    let mut child = cmd.spawn().map_err(|e| format!("agent_not_found: failed to spawn '{}': {e}", resolved_bin.display()))?;

    let deadline = Duration::from_secs(timeout_s.max(1));
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let wall_ms = t0.elapsed().as_millis() as u64;
                let output = child.wait_with_output().ok();
                let stdout_str = output.as_ref().map(|o| String::from_utf8_lossy(&o.stdout).to_string()).unwrap_or_default();
                let stderr_str = output.as_ref().map(|o| String::from_utf8_lossy(&o.stderr).trim().to_string()).filter(|s| !s.is_empty());
                let (tokens_in, tokens_out) = parse_token_usage(profile.usage, &stdout_str);
                let exit = if status.success() { RunExit::Ok } else { RunExit::Crashed };
                return Ok(AgentRunOutcome { exit, wall_ms, tokens_in, tokens_out, stderr_note: stderr_str });
            }
            Ok(None) => {
                if t0.elapsed() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    let wall_ms = t0.elapsed().as_millis() as u64;
                    return Ok(AgentRunOutcome {
                        exit: RunExit::Timeout,
                        wall_ms,
                        tokens_in: None,
                        tokens_out: None,
                        stderr_note: Some(format!("Timed out after {timeout_s}s")),
                    });
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => {
                let wall_ms = t0.elapsed().as_millis() as u64;
                return Ok(AgentRunOutcome {
                    exit: RunExit::Crashed,
                    wall_ms,
                    tokens_in: None,
                    tokens_out: None,
                    stderr_note: Some(format!("Process wait failed: {e}")),
                });
            }
        }
    }
}

/// Extracts `(tokens_in, tokens_out)` from CLI stdout according to `format`.
pub fn parse_token_usage(format: UsageFormat, stdout: &str) -> (Option<u64>, Option<u64>) {
    match format {
        UsageFormat::None => (None, None),
        UsageFormat::ClaudeStreamJson | UsageFormat::CodexJson => {
            let mut total_in = 0u64;
            let mut total_out = 0u64;
            let mut found = false;
            for line in stdout.lines() {
                let Ok(val) = serde_json::from_str::<Value>(line.trim()) else {
                    continue;
                };
                if let Some(usage) = val.get("usage").or_else(|| val.pointer("/result/usage")) {
                    if let Some(inp) = usage.get("input_tokens").or_else(|| usage.get("prompt_tokens")).and_then(Value::as_u64) {
                        total_in += inp;
                        found = true;
                    }
                    if let Some(outp) = usage.get("output_tokens").or_else(|| usage.get("completion_tokens")).and_then(Value::as_u64) {
                        total_out += outp;
                        found = true;
                    }
                }
            }
            if found {
                (Some(total_in), Some(total_out))
            } else {
                (None, None)
            }
        }
    }
}

/// One scripted tool call line in a `stub-agent` JSONL script (`{ "tool": "...", "args": { ... } }`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StubCall {
    pub tool: String,
    #[serde(default)]
    pub args: Value,
}

/// Connects to the MCP session described in `mcp_config_path` and replays every `{tool, args}` line
/// from `script_path`, writing `transcript.jsonl` and saving the resulting session state.
pub fn run_stub_agent(script_path: &Path, mcp_config_path: &Path) -> Result<(), String> {
    let raw_cfg =
        std::fs::read_to_string(mcp_config_path).map_err(|e| format!("stub_agent_error: cannot read mcp config '{}': {e}", mcp_config_path.display()))?;
    let cfg_val: Value = serde_json::from_str(&raw_cfg).map_err(|e| format!("stub_agent_error: invalid mcp config JSON: {e}"))?;

    let args_arr = cfg_val.pointer("/mcpServers/designos/args").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut workspace = mcp_config_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
    for (i, a) in args_arr.iter().enumerate() {
        if a.as_str() == Some("--workspace") {
            if let Some(w) = args_arr.get(i + 1).and_then(Value::as_str) {
                workspace = PathBuf::from(w);
            }
        }
    }
    let transcript_path = cfg_val
        .pointer("/mcpServers/designos/env/DESIGNOS_MCP_TRANSCRIPT")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("DESIGNOS_MCP_TRANSCRIPT").map(PathBuf::from).filter(|p| !p.as_os_str().is_empty()));

    let raw_script = std::fs::read_to_string(script_path).map_err(|e| format!("stub_agent_error: cannot read script '{}': {e}", script_path.display()))?;

    let mut session = EvalMcpSession::new(&workspace, transcript_path)?;
    for (idx, line) in raw_script.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let call: StubCall =
            serde_json::from_str(trimmed).map_err(|e| format!("stub_agent_error: invalid JSON on line {} of '{}': {e}", idx + 1, script_path.display()))?;
        // Execute the call; tool errors are logged to the transcript (just like a live MCP session)
        // unless transcript logging itself fails!
        if let Err(err) = session.call(&call.tool, call.args) {
            if err.starts_with("Transcript logging failed") {
                return Err(err);
            }
        }
    }
    session.persist_fallback_output()?;
    Ok(())
}

/// Deterministic in-process MCP session used by `stub-agent` and `designos-eval replay` so
/// `designos-eval` never depends on `designos-mcp` (avoiding circular workspace dependencies per E15-39).
pub struct EvalMcpSession {
    pub workspace: PathBuf,
    pub doc: Document,
    pub history: History,
    pub transcript_path: Option<PathBuf>,
    pub seq: u64,
}

impl EvalMcpSession {
    pub fn new(workspace: &Path, transcript_path: Option<PathBuf>) -> Result<Self, String> {
        std::fs::create_dir_all(workspace).map_err(|e| format!("cannot create workspace '{}': {e}", workspace.display()))?;
        let mut doc = Document::new("Untitled");
        let page = doc.add_page("Page 1");
        let _ = page;
        // If start.designos exists in workspace, pre-load it so agents that don't call designos_open_file still start from it
        let start_path = workspace.join("start.designos");
        if start_path.exists() {
            if let Ok(loaded) = designos_library::load_external(&start_path) {
                doc = loaded;
            }
        }
        Ok(Self { workspace: workspace.to_path_buf(), doc, history: History::default(), transcript_path, seq: 0 })
    }

    pub fn call(&mut self, tool: &str, args: Value) -> Result<Value, String> {
        let t0 = Instant::now();
        let args_copy = args.clone();
        let res = self.call_inner(tool, args);
        let t_ms = t0.elapsed().as_millis() as u64;
        if let Some(t_path) = &self.transcript_path {
            self.seq += 1;
            let undo_depth = self.history.undo_depth();
            let entry = match &res {
                Ok(val) => json!({
                    "seq": self.seq,
                    "t_ms": t_ms,
                    "tool": tool,
                    "args": args_copy,
                    "result": val,
                    "is_error": false,
                    "undo_depth": undo_depth,
                }),
                Err(err) => json!({
                    "seq": self.seq,
                    "t_ms": t_ms,
                    "tool": tool,
                    "args": args_copy,
                    "error": err,
                    "is_error": true,
                    "undo_depth": undo_depth,
                }),
            };
            let mut line = serde_json::to_string(&entry).map_err(|e| format!("Transcript serialization failed: {e}"))?;
            line.push('\n');
            if let Err(log_err) = append_line(t_path, &line) {
                return Err(format!("Transcript logging failed ({}): {log_err}", t_path.display()));
            }
        }
        res
    }

    fn call_inner(&mut self, tool: &str, args: Value) -> Result<Value, String> {
        match tool {
            "designos_get_document" => {
                let pages: Vec<Value> = self
                    .doc
                    .pages
                    .iter()
                    .filter_map(|&pid| {
                        let n = self.doc.get(pid)?;
                        Some(json!({
                            "id": pid.0,
                            "name": n.name,
                            "children": n.children.iter().map(|c| c.0).collect::<Vec<_>>(),
                        }))
                    })
                    .collect();
                Ok(json!({
                    "name": self.doc.name,
                    "pages": pages,
                    "node_count": self.doc.nodes.len(),
                    "can_undo": self.history.can_undo(),
                    "can_redo": self.history.can_redo(),
                }))
            }
            "designos_open_file" => {
                let rel = args.get("path").and_then(Value::as_str).ok_or_else(|| "Missing 'path'".to_string())?;
                let path = self.workspace.join(rel);
                let loaded = designos_library::load_external(&path).map_err(|e| format!("Cannot open '{}': {e}", path.display()))?;
                self.doc = loaded;
                self.history = History::default();
                self.call_inner("designos_get_document", json!({}))
            }
            "designos_new_document" => {
                let name = args.get("name").and_then(Value::as_str).unwrap_or("Untitled");
                let mut d = Document::new(name);
                d.add_page("Page 1");
                self.doc = d;
                self.history = History::default();
                self.call_inner("designos_get_document", json!({}))
            }
            "designos_save_file" => {
                let rel = args.get("path").and_then(Value::as_str).ok_or_else(|| "Missing 'path'".to_string())?;
                let path = self.workspace.join(rel);
                let bytes = self.doc.to_json().map_err(|e| e.to_string())?;
                if let Some(p) = path.parent() {
                    let _ = std::fs::create_dir_all(p);
                }
                std::fs::write(&path, bytes.as_bytes()).map_err(|e| format!("Cannot save '{}': {e}", path.display()))?;
                Ok(json!({
                    "path": path,
                    "bytes": bytes.len(),
                    "saved": true,
                }))
            }
            "designos_create_node" => {
                let parent_id = args
                    .get("parent_id")
                    .and_then(Value::as_u64)
                    .map(NodeId)
                    .or_else(|| self.doc.pages.first().copied())
                    .ok_or_else(|| "Document has no pages".to_string())?;
                let parent_node = self.doc.get(parent_id).ok_or_else(|| format!("Parent node {} not found", parent_id.0))?;
                if !parent_node.kind.is_container() {
                    return Err(format!("Node {} is not a container", parent_id.0));
                }
                let index = parent_node.children.len();
                let kind_str = args.get("kind").and_then(Value::as_str).unwrap_or("frame").to_ascii_lowercase();
                let x = args.get("x").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let y = args.get("y").and_then(Value::as_f64).unwrap_or(0.0) as f32;
                let w = args.get("width").and_then(Value::as_f64).unwrap_or(100.0) as f32;
                let h = args.get("height").and_then(Value::as_f64).unwrap_or(100.0) as f32;
                let id = self.doc.alloc_id();
                let rect = Rect::new(x, y, w, h);
                let mut node = match kind_str.as_str() {
                    "rectangle" | "rect" => Node::rectangle(id, rect),
                    "ellipse" => Node::ellipse(id, rect),
                    "text" => {
                        let text = args.get("text").and_then(Value::as_str).unwrap_or("Text");
                        Node::text(id, rect, text, designos_doc::TextStyle::default())
                    }
                    _ => Node::frame(id, rect),
                };
                if let Some(name) = args.get("name").and_then(Value::as_str) {
                    node.name = name.to_string();
                }
                if let Some(fill_hex) = args.get("fill").and_then(Value::as_str) {
                    if let Some(c) = designos_doc::Color::from_hex(fill_hex) {
                        node.fills = vec![Paint::solid(c)];
                    }
                }
                self.history.exec(&mut self.doc, DocCommand::Insert { parent: parent_id, index, nodes: vec![node] }).map_err(|e| e.to_string())?;
                Ok(json!({
                    "node_id": id.0,
                    "parent_id": parent_id.0,
                    "node": self.doc.get(id),
                    "changed": true,
                }))
            }
            "designos_apply_spec" => {
                let parent_id = args
                    .get("parent_id")
                    .and_then(Value::as_u64)
                    .filter(|&id| id != 0)
                    .map(NodeId)
                    .or_else(|| self.doc.pages.first().copied())
                    .ok_or_else(|| "Document has no pages".to_string())?;
                let spec_val = args.get("spec").cloned().ok_or_else(|| "Missing 'spec'".to_string())?;
                let spec: NodeSpec = serde_json::from_value(spec_val).map_err(|e| format!("Invalid NodeSpec: {e}"))?;
                let mut trial = self.doc.clone();
                let root_id = trial.apply_spec(parent_id, &spec)?;
                let mut nodes = Vec::new();
                trial.walk(root_id, |n, _| nodes.push(n.clone()));
                // Copy any newly materialized collections/variables (e.g. EaseUI deprecated or bound vars)
                self.doc.variables = trial.variables;
                self.doc.variable_collections = trial.variable_collections;
                let index = self.doc.get(parent_id).ok_or_else(|| format!("Parent {} not found", parent_id.0))?.children.len();
                self.history.exec(&mut self.doc, DocCommand::Insert { parent: parent_id, index, nodes }).map_err(|e| e.to_string())?;
                self.doc.relayout_subtree(parent_id);
                Ok(json!({
                    "root_node_id": root_id.0,
                    "parent_id": parent_id.0,
                    "node": self.doc.get(root_id),
                    "changed": true,
                }))
            }
            "designos_update_node" => {
                let node_id = NodeId(args.get("node_id").and_then(Value::as_u64).ok_or_else(|| "Missing 'node_id'".to_string())?);
                let mut updated = self.doc.get(node_id).ok_or_else(|| format!("Node {} not found", node_id.0))?.clone();
                if let Some(name) = args.get("name").and_then(Value::as_str) {
                    updated.name = name.to_string();
                }
                if let Some(w) = args.get("width").and_then(Value::as_f64) {
                    updated.size.x = w as f32;
                }
                if let Some(h) = args.get("height").and_then(Value::as_f64) {
                    updated.size.y = h as f32;
                }
                if let Some(fill_hex) = args.get("fill").and_then(Value::as_str) {
                    if let Some(c) = designos_doc::Color::from_hex(fill_hex) {
                        updated.fills = vec![Paint::solid(c)];
                    }
                }
                if let Some(text) = args.get("text").and_then(Value::as_str) {
                    if let NodeKind::Text(tp) = &mut updated.kind {
                        tp.content = text.to_string();
                        tp.runs.clear();
                    }
                }
                self.history.exec(&mut self.doc, DocCommand::Update { node: Box::new(updated) }).map_err(|e| e.to_string())?;
                Ok(json!({
                    "node_id": node_id.0,
                    "node": self.doc.get(node_id),
                    "changed": true,
                }))
            }
            "designos_delete_node" => {
                let node_id = NodeId(args.get("node_id").and_then(Value::as_u64).ok_or_else(|| "Missing 'node_id'".to_string())?);
                self.history.exec(&mut self.doc, DocCommand::Remove { id: node_id }).map_err(|e| e.to_string())?;
                Ok(json!({ "deleted": node_id.0, "changed": true }))
            }
            "designos_undo" => {
                let changed = self.history.undo(&mut self.doc).map_err(|e| e.to_string())?;
                Ok(json!({ "changed": changed }))
            }
            "designos_redo" => {
                let changed = self.history.redo(&mut self.doc).map_err(|e| e.to_string())?;
                Ok(json!({ "changed": changed }))
            }
            other => Err(format!("Unknown tool '{other}'")),
        }
    }

    /// Saves the current session document to `workspace/output.designos` if the agent didn't
    /// explicitly call `designos_save_file` with `output.designos` (E15-25 §3.7).
    pub fn persist_fallback_output(&self) -> Result<(), String> {
        let out_in_ws = self.workspace.join("output.designos");
        if !out_in_ws.exists() && self.seq > 0 {
            let bytes = self.doc.to_json().map_err(|e| e.to_string())?;
            std::fs::write(&out_in_ws, bytes.as_bytes()).map_err(|e| format!("cannot save fallback output.designos: {e}"))?;
        }
        Ok(())
    }
}

fn append_line(path: &Path, line: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create dir {}: {e}", parent.display()))?;
        }
    }
    let mut file = std::fs::OpenOptions::new().create(true).append(true).open(path).map_err(|e| format!("open {}: {e}", path.display()))?;
    file.write_all(line.as_bytes()).and_then(|_| file.flush()).map_err(|e| format!("write {}: {e}", path.display()))
}
