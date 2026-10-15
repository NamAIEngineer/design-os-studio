//! Native Embedded PTY Agent Terminal Runtime (`claude`, `codex`, `grok`, `$SHELL`)
//! and Bidirectional Canvas <-> Terminal Spec Bridge (`RailItem::Terminal` + Bottom Dock).
//!
//! Crafted to `es:designer` (`craft-floor.md` + `tokens.css`) and **EaseUI BrainWave 2.0**:
//! - Zero emoji icons (`§2 Refuse by default`): uses crisp Lucide vector glyphs (`crate::icons::Icon`)
//! - Surface steps: `#0E0E12` viewport -> `#16171B` base -> `#1F2024` raised -> `#26272C` hover
//! - 1px hairlines (`#2A2B31`) instead of nested cards-in-cards
//! - Concentric radii (`12px` outer surface, `6px` inner controls)
//! - Semantic status pills (`OK #4CC58B`, `WARN #E0A53C`, `BAD #FF7A8C`, `INFO #8FB0FF`)

use crate::icons::{self, Icon};
use crate::panels::{clipped, scrollbar, wheel_scroll};
use crate::theme;
use crate::widgets;
use designos_doc::{Document, NodeId, NodeSpec};
use egui::text::LayoutJob;
use egui::{Align2, Color32, CornerRadius, Event, FontFamily, FontId, Id, Key, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, TextFormat, Ui, Vec2};
use portable_pty::{native_pty_system, Child, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

// EaseUI BrainWave 2.0 + es:designer Craft Floor Surface & Hairline Tokens
pub const TERM_BG_VIEWPORT: Color32 = Color32::from_rgb(0x0E, 0x0E, 0x12);
pub const TERM_BG_BAR: Color32 = Color32::from_rgb(0x16, 0x17, 0x1B);
pub const TERM_BG_ELEVATED: Color32 = Color32::from_rgb(0x1F, 0x20, 0x24);
pub const TERM_BG_HOVER: Color32 = Color32::from_rgb(0x26, 0x27, 0x2C);
pub const TERM_BORDER: Color32 = Color32::from_rgb(0x2A, 0x2B, 0x31);
pub const TERM_ORANGE: Color32 = Color32::from_rgb(0xFF, 0x6A, 0x3D);
pub const TERM_ORANGE_BG: Color32 = Color32::from_rgb(0x36, 0x1A, 0x12);
pub const TERM_FG_DEFAULT: Color32 = Color32::from_rgb(0xE4, 0xE4, 0xE7);

// Semantic Status Pill Tokens (`es:designer` tokens.css)
pub const STATUS_OK_TEXT: Color32 = Color32::from_rgb(0x4C, 0xC5, 0x8B);
pub const STATUS_OK_BG: Color32 = Color32::from_rgb(0x17, 0x30, 0x2A);
pub const STATUS_OK_BORDER: Color32 = Color32::from_rgb(0x2C, 0x5F, 0x4A);

pub const STATUS_WARN_TEXT: Color32 = Color32::from_rgb(0xE0, 0xA5, 0x3C);
pub const STATUS_WARN_BG: Color32 = Color32::from_rgb(0x35, 0x29, 0x1A);
pub const STATUS_WARN_BORDER: Color32 = Color32::from_rgb(0x5C, 0x47, 0x20);

pub const STATUS_BAD_TEXT: Color32 = Color32::from_rgb(0xFF, 0x7A, 0x8C);
pub const STATUS_BAD_BG: Color32 = Color32::from_rgb(0x3B, 0x1D, 0x23);
pub const STATUS_BAD_BORDER: Color32 = Color32::from_rgb(0x6A, 0x2E, 0x39);

pub const STATUS_INFO_TEXT: Color32 = Color32::from_rgb(0x8F, 0xB0, 0xFF);
pub const STATUS_INFO_BG: Color32 = Color32::from_rgb(0x1B, 0x26, 0x40);
pub const STATUS_INFO_BORDER: Color32 = Color32::from_rgb(0x2F, 0x45, 0x77);

const CHAR_W: f32 = 7.8;
const ROW_H: f32 = 16.0;
const MAX_RAW_LOG_BYTES: usize = 128 * 1024;

/// Presets for launching an interactive PTY session inside DESIGN:OS Studio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentCliPreset {
    Shell,
    Claude,
    Codex,
    Grok,
    DesignOsHelp,
}

impl AgentCliPreset {
    pub const QUICK_LAUNCH: [AgentCliPreset; 4] = [AgentCliPreset::Shell, AgentCliPreset::Claude, AgentCliPreset::Codex, AgentCliPreset::Grok];

    pub fn tab_title(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "Shell ($SHELL)",
            AgentCliPreset::Claude => "Claude Code",
            AgentCliPreset::Codex => "Codex CLI",
            AgentCliPreset::Grok => "Grok Build",
            AgentCliPreset::DesignOsHelp => "DESIGN:OS Agent CLI",
        }
    }

    pub fn button_label(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "Shell",
            AgentCliPreset::Claude => "Claude CLI",
            AgentCliPreset::Codex => "Codex CLI",
            AgentCliPreset::Grok => "Grok CLI",
            AgentCliPreset::DesignOsHelp => "Agent Guide",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "ZSH/SH",
            AgentCliPreset::Claude => "CLAUDE",
            AgentCliPreset::Codex => "CODEX",
            AgentCliPreset::Grok => "GROK",
            AgentCliPreset::DesignOsHelp => "DESIGNOS",
        }
    }
}

/// Actions emitted by the Embedded Agent Terminal for the host `App` / `Shell` to execute.
#[derive(Clone, Debug, PartialEq)]
pub enum TerminalAction {
    /// Parses a `NodeSpec` JSON string and applies it directly to the active `Document` page.
    ApplySpecJson(String),
    /// Loads and applies a `.spec.json` file from disk.
    ApplySpecFile(PathBuf),
    /// Triggers immediate hot-reload of `save_path` into the Canvas.
    ReloadFromDisk,
    /// Serializes the currently selected Canvas node (or first top-level frame) to
    /// `/tmp/designos-live-selection.spec.json` and pipes its path + node summary +
    /// unresolved Canvas Pin-Comments directly into the active PTY stdin.
    ExportSelectionToTerminal,
}

/// One live interactive PTY tab backed by `portable-pty` and a `vt100::Parser`.
pub struct TerminalTab {
    pub id: usize,
    pub title: String,
    #[allow(dead_code)]
    pub preset: AgentCliPreset,
    pub parser: Arc<Mutex<vt100::Parser>>,
    pub writer: Box<dyn Write + Send>,
    pub master: Box<dyn MasterPty + Send>,
    pub child: Box<dyn Child + Send + Sync>,
    pub rows: u16,
    pub cols: u16,
    pub last_spec_json: Arc<Mutex<Option<String>>>,
    pub raw_log: Arc<Mutex<String>>,
}

impl TerminalTab {
    /// Spawns a real interactive PTY session for `preset` with auto-injected `DESIGNOS_*` environment.
    pub fn spawn(id: usize, preset: AgentCliPreset, rows: u16, cols: u16, file_path: Option<&Path>) -> Result<Self, String> {
        let rows = rows.max(6);
        let cols = cols.max(20);
        let pty_system = native_pty_system();
        let pair = pty_system.openpty(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 }).map_err(|e| format!("openpty failed: {e}"))?;

        let shell_bin = default_shell_program();
        let mut cmd = match preset {
            AgentCliPreset::Shell | AgentCliPreset::DesignOsHelp => {
                let mut c = CommandBuilder::new(&shell_bin);
                if !cfg!(windows) {
                    c.arg("-l");
                }
                c
            }
            AgentCliPreset::Claude => build_agent_shell_cmd(&shell_bin, "claude"),
            AgentCliPreset::Codex => build_agent_shell_cmd(&shell_bin, "codex"),
            AgentCliPreset::Grok => build_agent_shell_cmd(&shell_bin, "grok"),
        };

        configure_pty_env(&mut cmd, file_path);

        let child = pair.slave.spawn_command(cmd).map_err(|e| format!("spawn_command failed: {e}"))?;
        drop(pair.slave);

        let mut reader = pair.master.try_clone_reader().map_err(|e| format!("try_clone_reader failed: {e}"))?;
        let writer = pair.master.take_writer().map_err(|e| format!("take_writer failed: {e}"))?;

        let mut init_parser = vt100::Parser::new(rows, cols, 1000);
        if matches!(preset, AgentCliPreset::DesignOsHelp | AgentCliPreset::Shell) {
            let bin_disp = std::env::current_exe().ok().map(|p| p.display().to_string()).unwrap_or_else(|| "designos".into());
            let file_disp = file_path.map(|p| p.display().to_string()).unwrap_or_else(|| "examples/specs/easeui-agent-console.spec.json".into());
            let banner = format!(
                "\x1b[1;38;2;255;106;61m[DESIGN:OS Studio — Native Agent Terminal]\x1b[0m\r\n\
                 \x1b[38;2;161;161;170m  DESIGNOS_BIN  = {bin_disp}\r\n\
                 \x1b[38;2;161;161;170m  DESIGNOS_FILE = {file_disp}\x1b[0m\r\n\
                 \x1b[38;2;143;176;255m  Ready: claude | codex | grok | \"$DESIGNOS_BIN\" \"$DESIGNOS_FILE\" --inspect-json\x1b[0m\r\n\r\n"
            );
            init_parser.process(banner.as_bytes());
        }

        let parser = Arc::new(Mutex::new(init_parser));
        let last_spec_json = Arc::new(Mutex::new(None));
        let raw_log = Arc::new(Mutex::new(String::new()));

        let parser_bg = Arc::clone(&parser);
        let spec_bg = Arc::clone(&last_spec_json);
        let log_bg = Arc::clone(&raw_log);

        std::thread::Builder::new()
            .name(format!("designos-pty-reader-{id}"))
            .spawn(move || {
                let mut buf = [0u8; 4096];
                loop {
                    match reader.read(&mut buf) {
                        Ok(0) => break,
                        Ok(n) => {
                            if let Ok(mut p) = parser_bg.lock() {
                                p.process(&buf[..n]);
                            }
                            let chunk = String::from_utf8_lossy(&buf[..n]);
                            if let Ok(mut log) = log_bg.lock() {
                                log.push_str(&chunk);
                                if log.len() > MAX_RAW_LOG_BYTES {
                                    let drain = log.len() - (MAX_RAW_LOG_BYTES / 2);
                                    let boundary = log.char_indices().map(|(i, _)| i).find(|&i| i >= drain).unwrap_or(log.len());
                                    log.drain(..boundary);
                                }
                                if let Some(spec) = extract_spec_json_from_log(&log) {
                                    if let Ok(mut last) = spec_bg.lock() {
                                        *last = Some(spec);
                                    }
                                }
                            }
                        }
                        Err(_) => break,
                    }
                }
            })
            .map_err(|e| format!("failed to spawn PTY reader thread: {e}"))?;

        Ok(Self {
            id,
            title: format!("{} #{}", preset.tab_title(), id),
            preset,
            parser,
            writer,
            master: pair.master,
            child,
            rows,
            cols,
            last_spec_json,
            raw_log,
        })
    }

    /// Resizes both the underlying OS PTY and the `vt100::Parser` when the viewport changes.
    pub fn resize(&mut self, rows: u16, cols: u16) {
        let rows = rows.max(4);
        let cols = cols.max(16);
        if self.rows == rows && self.cols == cols {
            return;
        }
        self.rows = rows;
        self.cols = cols;
        let _ = self.master.resize(PtySize { rows, cols, pixel_width: 0, pixel_height: 0 });
        if let Ok(mut p) = self.parser.lock() {
            p.set_size(rows, cols);
        }
    }

    /// Writes raw bytes directly to the PTY master stdin.
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    /// Writes a single line followed by `\r` to execute in the PTY.
    pub fn send_line(&mut self, line: &str) {
        let _ = self.writer.write_all(line.as_bytes());
        let _ = self.writer.write_all(b"\r");
        let _ = self.writer.flush();
    }
}

/// Persistent state for the embedded multi-tab PTY terminal.
pub struct TerminalState {
    pub tabs: Vec<TerminalTab>,
    pub active_tab: usize,
    pub dock_open: bool,
    pub dock_height: f32,
    pub input_line: String,
    pub status_msg: Option<String>,
    pub next_tab_id: usize,
    pub viewport_focused: bool,
    pub left_scroll: f32,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab: 0,
            dock_open: false,
            dock_height: 280.0,
            input_line: String::new(),
            status_msg: None,
            next_tab_id: 1,
            viewport_focused: true,
            left_scroll: 0.0,
        }
    }
}

impl TerminalState {
    /// Ensures at least one PTY tab is running when the terminal UI is first shown.
    pub fn ensure_default_tab(&mut self, file_path: Option<&Path>) {
        if self.tabs.is_empty() {
            let _ = self.spawn_tab(AgentCliPreset::DesignOsHelp, file_path);
        }
    }

    /// Spawns a new tab with `preset` and switches `active_tab` to it.
    pub fn spawn_tab(&mut self, preset: AgentCliPreset, file_path: Option<&Path>) -> Result<usize, String> {
        let id = self.next_tab_id.max(1);
        self.next_tab_id = id + 1;
        match TerminalTab::spawn(id, preset, 28, 100, file_path) {
            Ok(tab) => {
                self.tabs.push(tab);
                self.active_tab = self.tabs.len() - 1;
                self.viewport_focused = true;
                self.status_msg = Some(format!("Spawned {} (session #{id})", preset.tab_title()));
                Ok(self.active_tab)
            }
            Err(e) => {
                self.status_msg = Some(format!("PTY error: {e}"));
                Err(e)
            }
        }
    }

    /// Closes tab at `idx`, killing its child process if still running.
    pub fn close_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            let mut removed = self.tabs.remove(idx);
            let _ = removed.child.kill();
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            } else if self.tabs.is_empty() {
                self.active_tab = 0;
            }
        }
    }

    /// Returns a mutable reference to the currently active `TerminalTab`.
    pub fn active_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        let idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        self.tabs.get_mut(idx)
    }

    /// Returns a shared reference to the currently active `TerminalTab`.
    pub fn active_tab(&self) -> Option<&TerminalTab> {
        let idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        self.tabs.get(idx)
    }

    /// Copies the `product-os-spec-driven` block citation (`File: spec/...`, `Mục:`, `Trang reader:`,
    /// `Project:`, `Yêu cầu của tôi:`) to the system clipboard AND pipes it directly into the
    /// active `claude` / `codex` / shell PTY tab.
    pub fn send_copy_for_ai(&mut self, block_markdown: &str) {
        let payload = format_copy_for_ai_payload(block_markdown);

        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(payload.clone());
        }

        self.ensure_default_tab(None);
        self.dock_open = true;
        if let Some(tab) = self.active_tab_mut() {
            tab.write_bytes(payload.as_bytes());
            tab.write_bytes(b"\r");
        }
        self.status_msg = Some("Copied citation block & piped to active Agent PTY".into());
    }
}

/// Free-function wrapper for [`TerminalState::send_copy_for_ai`].
#[allow(dead_code)]
pub fn send_copy_for_ai(state: &mut TerminalState, block_markdown: &str) {
    state.send_copy_for_ai(block_markdown);
}

fn format_copy_for_ai_payload(block_markdown: &str) -> String {
    let trimmed = block_markdown.trim();
    if trimmed.contains("File:") && trimmed.contains("Yêu cầu của tôi:") {
        trimmed.to_string()
    } else {
        format!(
            "Hãy đọc và xử lý đoạn tài liệu này:\n\
             - File: spec/00-overview.md\n\
             - Mục: Product OS Spec Block\n\
             - Trang reader: #overview\n\
             - Project: design-os-studio\n\n\
             ```markdown\n\
             {trimmed}\n\
             ```\n\n\
             Yêu cầu của tôi: "
        )
    }
}

fn default_shell_program() -> String {
    if cfg!(windows) {
        std::env::var("COMSPEC").unwrap_or_else(|_| "powershell.exe".into())
    } else {
        std::env::var("SHELL").ok().filter(|s| !s.trim().is_empty()).unwrap_or_else(|| {
            if Path::new("/bin/zsh").exists() {
                "/bin/zsh".into()
            } else {
                "/bin/sh".into()
            }
        })
    }
}

fn build_agent_shell_cmd(shell_bin: &str, agent_cli: &str) -> CommandBuilder {
    let mut c = CommandBuilder::new(shell_bin);
    if cfg!(windows) {
        c.arg("-Command");
        c.arg(agent_cli);
    } else {
        c.arg("-l");
        c.arg("-i");
        c.arg("-c");
        c.arg(format!("{agent_cli} || exec \"{shell_bin}\" -l"));
    }
    c
}

/// Auto-injects `TERM`, `COLORTERM`, `DESIGNOS_BIN`, `DESIGNOS_FILE`, `DESIGNOS_WORKSPACE`,
/// and prepends common CLI directories (`~/.cargo/bin`, `/opt/homebrew/bin`, `/usr/local/bin`,
/// `~/.local/bin`) to `PATH`.
pub fn configure_pty_env(cmd: &mut CommandBuilder, file_path: Option<&Path>) {
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");

    if let Ok(exe) = std::env::current_exe() {
        cmd.env("DESIGNOS_BIN", exe.as_os_str());
    }
    if let Some(fp) = file_path {
        cmd.env("DESIGNOS_FILE", fp.as_os_str());
    }
    if let Ok(cwd) = std::env::current_dir() {
        cmd.env("DESIGNOS_WORKSPACE", cwd.as_os_str());
        cmd.cwd(cwd);
    }

    let home = std::env::var("HOME").unwrap_or_default();
    let mut extra_paths: Vec<String> = Vec::new();
    if !home.is_empty() {
        extra_paths.push(format!("{home}/.cargo/bin"));
        extra_paths.push(format!("{home}/.local/bin"));
    }
    extra_paths.push("/opt/homebrew/bin".into());
    extra_paths.push("/usr/local/bin".into());

    let current_path = std::env::var("PATH").unwrap_or_default();
    let sep = if cfg!(windows) { ";" } else { ":" };
    let combined = if current_path.is_empty() { extra_paths.join(sep) } else { format!("{}{sep}{current_path}", extra_paths.join(sep)) };
    cmd.env("PATH", combined);
}

/// Scans terminal output for a ```json ... ``` fence or JSON object representing a `NodeSpec`.
/// Returns the latest valid `NodeSpec` JSON string if found.
pub fn extract_spec_json_from_log(log: &str) -> Option<String> {
    let mut search_end = log.len();
    while let Some(fence_start) = log[..search_end].rfind("```json") {
        let after_tag = &log[fence_start + 7..search_end];
        if let Some(fence_end) = after_tag.find("```") {
            let candidate = after_tag[..fence_end].trim();
            if is_valid_node_spec_json(candidate) {
                return Some(candidate.to_string());
            }
        }
        search_end = fence_start;
    }

    let bytes = log.as_bytes();
    let mut i = bytes.len();
    let mut checked = 0usize;
    while i > 0 && checked < 12 {
        i -= 1;
        if bytes[i] == b'}' {
            if let Some(open_idx) = find_matching_open_brace(&log[..=i]) {
                let candidate = log[open_idx..=i].trim();
                if (candidate.contains("\"name\"") || candidate.contains("\"preset\"")) && is_valid_node_spec_json(candidate) {
                    return Some(candidate.to_string());
                }
                checked += 1;
            }
        }
    }
    None
}

fn find_matching_open_brace(slice_ending_with_close: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_str = false;
    let mut prev_backslash = false;
    let chars: Vec<(usize, char)> = slice_ending_with_close.char_indices().collect();
    for &(idx, ch) in chars.iter().rev() {
        if in_str {
            if ch == '"' && !prev_backslash {
                in_str = false;
            }
            prev_backslash = ch == '\\';
            continue;
        }
        match ch {
            '"' => in_str = true,
            '}' => depth += 1,
            '{' => {
                depth -= 1;
                if depth == 0 {
                    return Some(idx);
                }
            }
            _ => {}
        }
    }
    None
}

fn is_valid_node_spec_json(candidate: &str) -> bool {
    if !candidate.starts_with('{') || !candidate.ends_with('}') {
        return false;
    }
    if let Ok(spec) = serde_json::from_str::<NodeSpec>(candidate) {
        !spec.name.trim().is_empty() || spec.preset.is_some() || !spec.children.is_empty() || !spec.kind.trim().is_empty()
    } else {
        false
    }
}

/// Serializes the active selection (or first top-level frame on `page`) to
/// `/tmp/designos-live-selection.spec.json` and pipes its path, dimensions, and
/// unresolved Canvas Pin-Comments into the active PTY tab's stdin.
pub fn export_selection_and_comments_to_pty(
    state: &mut TerminalState,
    doc: &Document,
    page: NodeId,
    selection: &[NodeId],
    file_path: Option<&Path>,
) -> Result<PathBuf, String> {
    state.ensure_default_tab(file_path);

    let target_id = selection
        .first()
        .copied()
        .or_else(|| doc.get(page).and_then(|pg| pg.children.first().copied()))
        .ok_or_else(|| "No frame or node on current page to export".to_string())?;

    let spec = doc.export_spec(target_id)?;
    let json = serde_json::to_string_pretty(&spec).map_err(|e| e.to_string())?;
    let tmp_path = std::env::temp_dir().join("designos-live-selection.spec.json");
    std::fs::write(&tmp_path, &json).map_err(|e| format!("write {}: {e}", tmp_path.display()))?;

    let unresolved_comments: Vec<String> = doc
        .resolve_comment_targets()
        .into_iter()
        .filter(|c| !c.resolved)
        .map(|c| {
            let msg = c.messages.first().map(|m| m.text.as_str()).unwrap_or("(empty comment)");
            let target = c.target_node_name.unwrap_or_else(|| "Canvas".to_string());
            format!("[Pin on {target}]: {msg}")
        })
        .collect();

    let node_name = doc.get(target_id).map(|n| n.name.clone()).unwrap_or_else(|| "Selection".into());
    let comment_summary = if unresolved_comments.is_empty() {
        "0 open pin-comments".to_string()
    } else {
        format!("{} open pin-comments ({})", unresolved_comments.len(), unresolved_comments.join(" | "))
    };

    let cmd_snippet = format!("# [DESIGN:OS Canvas Context] Node '{}' (#{}) exported to {} — {}", node_name, target_id.0, tmp_path.display(), comment_summary);

    if let Some(tab) = state.active_tab_mut() {
        tab.send_line(&cmd_snippet);
    }
    state.status_msg = Some(format!("Sent '{}' (#{}) + comments to {}", node_name, target_id.0, tmp_path.display()));
    Ok(tmp_path)
}

/// Renders the Left Sidebar Agent Terminal Explorer (`es:designer` craft floor: hairline dividers,
/// surface steps, semantic status pills, and zero emoji icons).
pub fn show_left(state: &mut TerminalState, ui: &mut Ui, rect: Rect, file_path: Option<&Path>) -> Vec<TerminalAction> {
    state.ensure_default_tab(file_path);
    let mut actions = Vec::new();
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, theme::BG);

    // Header banner (44px)
    let header_h = 44.0;
    let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_h));
    p.rect_filled(header_rect, 0.0, TERM_BG_BAR);
    widgets::hline(&p, header_rect.left(), header_rect.right(), header_rect.bottom() - 1.0, TERM_BORDER);

    p.circle_filled(Pos2::new(header_rect.left() + 14.0, header_rect.center().y), 3.5, STATUS_OK_TEXT);
    let title = widgets::elide(&p, "AGENT PTY RUNTIME", &theme::font_small_strong(), (header_rect.width() - 96.0).max(40.0));
    p.text(Pos2::new(header_rect.left() + 24.0, header_rect.center().y), Align2::LEFT_CENTER, title, theme::font_small_strong(), TERM_ORANGE);

    // Dock toggle pill button (concentric 6px radius)
    let dock_w = 58.0;
    let dock_rect = Rect::from_min_size(Pos2::new(header_rect.right() - dock_w - 10.0, header_rect.center().y - 11.0), Vec2::new(dock_w, 22.0));
    let dock_resp = ui.interact(dock_rect, Id::new("term-left-dock-toggle"), Sense::click());
    let dock_bg = if state.dock_open {
        STATUS_INFO_BG
    } else if dock_resp.hovered() {
        TERM_BG_HOVER
    } else {
        TERM_BG_ELEVATED
    };
    p.rect_filled(dock_rect, CornerRadius::same(6), dock_bg);
    p.rect_stroke(dock_rect, CornerRadius::same(6), Stroke::new(1.0, if state.dock_open { STATUS_INFO_BORDER } else { TERM_BORDER }), StrokeKind::Inside);
    p.text(dock_rect.center(), Align2::CENTER_CENTER, "Dock", theme::font_small_medium(), if state.dock_open { STATUS_INFO_TEXT } else { theme::TEXT });
    widgets::record(ui, "term-left-dock-toggle", dock_rect);
    if dock_resp.clicked() {
        state.dock_open = !state.dock_open;
    }

    let body_rect = Rect::from_min_max(Pos2::new(rect.left(), header_rect.bottom()), rect.max);
    let est_h = 340.0 + state.tabs.len() as f32 * 38.0;
    wheel_scroll(ui, body_rect, &mut state.left_scroll, est_h);

    clipped(ui, body_rect, |ui| {
        let p = ui.painter().clone();
        let mut y = body_rect.top() - state.left_scroll + 12.0;
        let pad_x = 12.0;
        let row_w = (body_rect.width() - pad_x * 2.0).max(80.0);

        // Section 1: Launch Agent CLI Session (surface-step rows, no nested card borders)
        p.text(Pos2::new(body_rect.left() + pad_x, y + 8.0), Align2::LEFT_CENTER, "LAUNCH AGENT CLI", theme::font_small_strong(), theme::TEXT_SECONDARY);
        y += 22.0;

        for preset in [AgentCliPreset::Claude, AgentCliPreset::Codex, AgentCliPreset::Grok, AgentCliPreset::Shell] {
            let btn_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 28.0));
            let resp = ui.interact(btn_r, Id::new(("term-left-spawn", preset as usize)), Sense::click());
            let bg = if resp.hovered() { TERM_BG_HOVER } else { TERM_BG_ELEVATED };
            p.rect_filled(btn_r, CornerRadius::same(6), bg);
            icons::paint(&p, Icon::Plus, Pos2::new(btn_r.left() + 13.0, btn_r.center().y), 12.0, theme::ICON_SECONDARY);
            let label = widgets::elide(&p, preset.button_label(), &theme::font_body_semibold(), (row_w - 86.0).max(24.0));
            p.text(Pos2::new(btn_r.left() + 25.0, btn_r.center().y), Align2::LEFT_CENTER, label, theme::font_body_semibold(), theme::TEXT);
            // Semantic Info Pill on right
            let badge_w = 52.0;
            let badge_r = Rect::from_min_size(Pos2::new(btn_r.right() - badge_w - 6.0, btn_r.center().y - 8.0), Vec2::new(badge_w, 16.0));
            p.rect_filled(badge_r, CornerRadius::same(4), STATUS_INFO_BG);
            p.rect_stroke(badge_r, CornerRadius::same(4), Stroke::new(1.0, STATUS_INFO_BORDER), StrokeKind::Inside);
            p.text(badge_r.center(), Align2::CENTER_CENTER, preset.badge(), theme::font_small_strong(), STATUS_INFO_TEXT);
            if resp.clicked() {
                let _ = state.spawn_tab(preset, file_path);
            }
            y += 32.0;
        }

        y += 6.0;
        widgets::hline(&p, body_rect.left() + pad_x, body_rect.right() - pad_x, y, TERM_BORDER);
        y += 10.0;

        // Section 2: Active PTY Sessions
        p.text(Pos2::new(body_rect.left() + pad_x, y + 8.0), Align2::LEFT_CENTER, "ACTIVE SESSIONS", theme::font_small_strong(), theme::TEXT_SECONDARY);
        y += 22.0;

        let mut select_idx = None;
        for (idx, tab) in state.tabs.iter().enumerate() {
            let active = idx == state.active_tab;
            let row_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 30.0));
            let resp = ui.interact(row_r, Id::new(("term-left-session", tab.id)), Sense::click());
            let bg = if active {
                STATUS_OK_BG
            } else if resp.hovered() {
                TERM_BG_HOVER
            } else {
                TERM_BG_ELEVATED
            };
            p.rect_filled(row_r, CornerRadius::same(6), bg);
            if active {
                p.rect_stroke(row_r, CornerRadius::same(6), Stroke::new(1.0, STATUS_OK_BORDER), StrokeKind::Inside);
            }
            p.circle_filled(Pos2::new(row_r.left() + 11.0, row_r.center().y), 3.5, STATUS_OK_TEXT);
            let t_label = widgets::elide(&p, &tab.title, &theme::font_body_semibold(), (row_w - 28.0).max(24.0));
            p.text(
                Pos2::new(row_r.left() + 22.0, row_r.center().y),
                Align2::LEFT_CENTER,
                t_label,
                theme::font_body_semibold(),
                if active { STATUS_OK_TEXT } else { theme::TEXT },
            );
            if resp.clicked() {
                select_idx = Some(idx);
            }
            y += 34.0;
        }
        if let Some(i) = select_idx {
            state.active_tab = i;
        }

        y += 6.0;
        widgets::hline(&p, body_rect.left() + pad_x, body_rect.right() - pad_x, y, TERM_BORDER);
        y += 10.0;

        // Section 3: Canvas <-> Terminal Bridge Actions (vector icons, zero emojis)
        p.text(Pos2::new(body_rect.left() + pad_x, y + 8.0), Align2::LEFT_CENTER, "CANVAS <-> PTY BRIDGE", theme::font_small_strong(), theme::TEXT_SECONDARY);
        y += 22.0;

        for (id_key, icon, label_txt, fg, bg_col, border_col) in [
            ("term-left-send-sel", Icon::Comment, "Send Selection & Comments", STATUS_INFO_TEXT, STATUS_INFO_BG, STATUS_INFO_BORDER),
            ("term-left-apply-out", Icon::Sparkle, "Apply Spec from Output", STATUS_WARN_TEXT, STATUS_WARN_BG, STATUS_WARN_BORDER),
            ("term-left-sync-disk", Icon::Rotation, "Sync Canvas from Disk", STATUS_OK_TEXT, STATUS_OK_BG, STATUS_OK_BORDER),
        ] {
            let btn_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 28.0));
            let resp = ui.interact(btn_r, Id::new(id_key), Sense::click());
            p.rect_filled(btn_r, CornerRadius::same(6), if resp.hovered() { TERM_BG_HOVER } else { bg_col });
            p.rect_stroke(btn_r, CornerRadius::same(6), Stroke::new(1.0, border_col), StrokeKind::Inside);
            icons::paint(&p, icon, Pos2::new(btn_r.left() + 14.0, btn_r.center().y), 13.0, fg);
            let elided = widgets::elide(&p, label_txt, &theme::font_body_semibold(), (row_w - 34.0).max(24.0));
            p.text(Pos2::new(btn_r.left() + 26.0, btn_r.center().y), Align2::LEFT_CENTER, elided, theme::font_body_semibold(), fg);
            if resp.clicked() {
                match id_key {
                    "term-left-send-sel" => actions.push(TerminalAction::ExportSelectionToTerminal),
                    "term-left-apply-out" => {
                        if let Some(act) = resolve_apply_spec_action(state) {
                            actions.push(act);
                        }
                    }
                    "term-left-sync-disk" => actions.push(TerminalAction::ReloadFromDisk),
                    _ => {}
                }
            }
            y += 34.0;
        }
    });

    scrollbar(ui, body_rect, state.left_scroll, est_h);
    actions
}

/// Alias for `show_full` used when rendering the full-workspace Agent Terminal view.
#[allow(dead_code)]
pub fn show_workspace(state: &mut TerminalState, ui: &mut Ui, rect: Rect, file_path: Option<&Path>) -> Vec<TerminalAction> {
    show_full(state, ui, rect, file_path)
}

/// Renders the full-canvas Agent Terminal view (`RailItem::Terminal`).
#[allow(dead_code)]
pub fn show_full(state: &mut TerminalState, ui: &mut Ui, rect: Rect, file_path: Option<&Path>) -> Vec<TerminalAction> {
    render_terminal_surface(state, ui, rect, file_path, "full")
}

/// Renders the docked bottom split-view Agent Terminal (`Ctrl+\`` / `dock_open`).
pub fn show_dock(state: &mut TerminalState, ui: &mut Ui, rect: Rect, file_path: Option<&Path>) -> Vec<TerminalAction> {
    render_terminal_surface(state, ui, rect, file_path, "dock")
}

fn resolve_apply_spec_action(state: &mut TerminalState) -> Option<TerminalAction> {
    let mut detected_from_guard = None;
    let mut detected_from_log = None;
    if let Some(tab) = state.active_tab() {
        if let Ok(guard) = tab.last_spec_json.lock() {
            detected_from_guard = guard.clone();
        }
        if detected_from_guard.is_none() {
            if let Ok(log) = tab.raw_log.lock() {
                detected_from_log = extract_spec_json_from_log(&log);
            }
        }
    }
    if let Some(json) = detected_from_guard {
        state.status_msg = Some("Applied NodeSpec JSON detected from PTY output".into());
        return Some(TerminalAction::ApplySpecJson(json));
    }
    if let Some(json) = detected_from_log {
        state.status_msg = Some("Applied NodeSpec JSON parsed from PTY log".into());
        return Some(TerminalAction::ApplySpecJson(json));
    }
    let fallback = PathBuf::from("examples/specs/easeui-agent-console.spec.json");
    state.status_msg = Some(format!("Applied {}", fallback.display()));
    Some(TerminalAction::ApplySpecFile(fallback))
}

fn render_terminal_surface(state: &mut TerminalState, ui: &mut Ui, rect: Rect, file_path: Option<&Path>, surface_id: &'static str) -> Vec<TerminalAction> {
    state.ensure_default_tab(file_path);
    let mut actions = Vec::new();
    let p = ui.painter().clone();

    p.rect_filled(rect, 0.0, TERM_BG_VIEWPORT);
    widgets::hline(&p, rect.left(), rect.right(), rect.top(), TERM_BORDER);

    // 1. Top Tab & Preset Bar (36px)
    let tab_bar_h = 36.0;
    let tab_bar_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), tab_bar_h));
    p.rect_filled(tab_bar_rect, 0.0, TERM_BG_BAR);
    widgets::hline(&p, tab_bar_rect.left(), tab_bar_rect.right(), tab_bar_rect.bottom() - 1.0, TERM_BORDER);

    // Bridge action buttons on the right of the tab bar (with vector icons, zero emojis)
    let mut right_x = tab_bar_rect.right() - 8.0;
    let btn_cy = tab_bar_rect.center().y;

    let sync_r = draw_bar_icon_pill(
        ui,
        &p,
        (surface_id, "term-bar-sync"),
        Pos2::new(right_x, btn_cy),
        Icon::Rotation,
        "Sync Canvas",
        TERM_BG_ELEVATED,
        TERM_BORDER,
        theme::TEXT,
    );
    right_x = sync_r.left() - 6.0;
    if ui.interact(sync_r, Id::new((surface_id, "term-bar-sync")), Sense::click()).clicked() {
        actions.push(TerminalAction::ReloadFromDisk);
    }

    let apply_r = draw_bar_icon_pill(
        ui,
        &p,
        (surface_id, "term-bar-apply-spec"),
        Pos2::new(right_x, btn_cy),
        Icon::Sparkle,
        "Apply Spec from Output",
        STATUS_WARN_BG,
        STATUS_WARN_BORDER,
        STATUS_WARN_TEXT,
    );
    right_x = apply_r.left() - 6.0;
    if ui.interact(apply_r, Id::new((surface_id, "term-bar-apply-spec")), Sense::click()).clicked() {
        if let Some(act) = resolve_apply_spec_action(state) {
            actions.push(act);
        }
    }

    let send_r = draw_bar_icon_pill(
        ui,
        &p,
        (surface_id, "term-bar-send-sel"),
        Pos2::new(right_x, btn_cy),
        Icon::Comment,
        "Send Selection & Comments",
        STATUS_INFO_BG,
        STATUS_INFO_BORDER,
        STATUS_INFO_TEXT,
    );
    right_x = send_r.left() - 8.0;
    if ui.interact(send_r, Id::new((surface_id, "term-bar-send-sel")), Sense::click()).clicked() {
        actions.push(TerminalAction::ExportSelectionToTerminal);
    }

    // Left side of Tab Bar: Session Tabs + Quick-Launch Preset Buttons
    let mut left_x = tab_bar_rect.left() + 8.0;
    let mut tab_to_select: Option<usize> = None;
    let mut tab_to_close: Option<usize> = None;

    for (idx, tab) in state.tabs.iter().enumerate() {
        if left_x + 120.0 > right_x {
            break;
        }
        let is_active = idx == state.active_tab;
        let tab_w = 136.0f32.min((right_x - left_x - 4.0).max(76.0));
        let t_rect = Rect::from_min_size(Pos2::new(left_x, tab_bar_rect.center().y - 12.0), Vec2::new(tab_w, 24.0));
        let close_rect = Rect::from_min_size(Pos2::new(t_rect.right() - 18.0, t_rect.center().y - 7.0), Vec2::new(14.0, 14.0));
        let close_resp = ui.interact(close_rect, Id::new((surface_id, "term-tab-close", tab.id)), Sense::click());
        let tab_resp = ui.interact(t_rect, Id::new((surface_id, "term-tab-select", tab.id)), Sense::click());

        let bg = if is_active {
            TERM_BG_ELEVATED
        } else if tab_resp.hovered() {
            TERM_BG_HOVER
        } else {
            TERM_BG_BAR
        };
        p.rect_filled(t_rect, CornerRadius::same(6), bg);
        if is_active {
            p.rect_stroke(t_rect, CornerRadius::same(6), Stroke::new(1.0, TERM_BORDER), StrokeKind::Inside);
            widgets::hline(&p, t_rect.left() + 6.0, t_rect.right() - 6.0, t_rect.bottom() - 1.0, TERM_ORANGE);
        }

        // Semantic OK live status dot
        p.circle_filled(Pos2::new(t_rect.left() + 9.0, t_rect.center().y), 3.0, STATUS_OK_TEXT);
        let title_str = widgets::elide(&p, &tab.title, &theme::font_small_medium(), (tab_w - 38.0).max(20.0));
        p.text(
            Pos2::new(t_rect.left() + 17.0, t_rect.center().y),
            Align2::LEFT_CENTER,
            title_str,
            theme::font_small_medium(),
            if is_active { theme::TEXT } else { theme::TEXT_SECONDARY },
        );
        icons::paint(&p, Icon::Close, close_rect.center(), 10.0, if close_resp.hovered() { STATUS_BAD_TEXT } else { theme::ICON_TERTIARY });

        if close_resp.clicked() {
            tab_to_close = Some(idx);
        } else if tab_resp.clicked() {
            tab_to_select = Some(idx);
        }
        left_x = t_rect.right() + 4.0;
    }

    if let Some(idx) = tab_to_select {
        state.active_tab = idx;
        state.viewport_focused = true;
    }
    if let Some(idx) = tab_to_close {
        state.close_tab(idx);
        state.ensure_default_tab(file_path);
    }

    // Quick-launch preset buttons with crisp Plus icon
    for preset in AgentCliPreset::QUICK_LAUNCH {
        let label = preset.button_label();
        let btn_w = (label.len() as f32 * 6.0 + 28.0).clamp(64.0, 98.0);
        if left_x + btn_w > right_x {
            break;
        }
        let b_rect = Rect::from_min_size(Pos2::new(left_x, tab_bar_rect.center().y - 11.0), Vec2::new(btn_w, 22.0));
        let b_resp = ui.interact(b_rect, Id::new((surface_id, "term-preset-spawn", preset as usize)), Sense::click());
        let is_agent = preset != AgentCliPreset::Shell;
        let bg = if b_resp.hovered() {
            TERM_BG_HOVER
        } else if is_agent {
            STATUS_INFO_BG
        } else {
            TERM_BG_ELEVATED
        };
        let border = if is_agent { STATUS_INFO_BORDER } else { TERM_BORDER };
        let fg = if is_agent { STATUS_INFO_TEXT } else { theme::TEXT_SECONDARY };
        p.rect_filled(b_rect, CornerRadius::same(6), bg);
        p.rect_stroke(b_rect, CornerRadius::same(6), Stroke::new(1.0, border), StrokeKind::Inside);
        icons::paint(&p, Icon::Plus, Pos2::new(b_rect.left() + 10.0, b_rect.center().y), 10.0, fg);
        let elided = widgets::elide(&p, label, &theme::font_small_medium(), (btn_w - 22.0).max(16.0));
        p.text(Pos2::new(b_rect.left() + 18.0, b_rect.center().y), Align2::LEFT_CENTER, elided, theme::font_small_medium(), fg);
        if b_resp.clicked() {
            let _ = state.spawn_tab(preset, file_path);
        }
        left_x = b_rect.right() + 4.0;
    }

    // 2. Bottom Quick Prompt / Command Bar (58px: chips row + singleline composer row)
    let bottom_h = 58.0;
    let viewport_rect = Rect::from_min_max(
        Pos2::new(rect.left(), tab_bar_rect.bottom()),
        Pos2::new(rect.right(), (rect.bottom() - bottom_h).max(tab_bar_rect.bottom() + 48.0)),
    );
    let prompt_bar_rect = Rect::from_min_max(Pos2::new(rect.left(), viewport_rect.bottom()), rect.max);

    // 3. VT100 Screen Grid Renderer + Raw Keyboard Capture
    render_vt100_viewport(state, ui, viewport_rect, surface_id);

    // 4. Render Bottom Prompt Composer Bar + Quick Chips
    render_prompt_composer(state, ui, prompt_bar_rect, surface_id, &mut actions);

    ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));

    actions
}

fn render_vt100_viewport(state: &mut TerminalState, ui: &mut Ui, viewport_rect: Rect, surface_id: &'static str) {
    let p = ui.painter().clone();
    p.rect_filled(viewport_rect, 0.0, TERM_BG_VIEWPORT);

    let pad_x = 12.0;
    let pad_y = 8.0;
    let grid_w = (viewport_rect.width() - pad_x * 2.0).max(120.0);
    let grid_h = (viewport_rect.height() - pad_y * 2.0).max(32.0);
    let cols = ((grid_w / CHAR_W).floor() as u16).clamp(20, 320);
    let rows = ((grid_h / ROW_H).floor() as u16).clamp(4, 120);

    let vp_resp = ui.interact(viewport_rect, Id::new((surface_id, "term-vt100-viewport")), Sense::click());
    if vp_resp.clicked() {
        state.viewport_focused = true;
    }
    let viewport_focused = state.viewport_focused;

    let Some(tab) = state.active_tab_mut() else {
        return;
    };
    tab.resize(rows, cols);

    // Direct raw keystroke & paste capture when the terminal viewport is focused
    if viewport_focused && !ui.ctx().egui_wants_keyboard_input() {
        let mut raw_bytes: Vec<u8> = Vec::new();
        ui.input(|i| {
            for ev in &i.events {
                match ev {
                    Event::Text(text) => {
                        raw_bytes.extend_from_slice(text.as_bytes());
                    }
                    Event::Paste(text) => {
                        raw_bytes.extend_from_slice(text.as_bytes());
                    }
                    Event::Key { key, pressed: true, modifiers, .. } => {
                        if modifiers.ctrl {
                            match key {
                                Key::C => raw_bytes.push(0x03),
                                Key::D => raw_bytes.push(0x04),
                                Key::L => raw_bytes.push(0x0c),
                                Key::U => raw_bytes.push(0x15),
                                Key::W => raw_bytes.push(0x17),
                                Key::A => raw_bytes.push(0x01),
                                Key::E => raw_bytes.push(0x05),
                                _ => {}
                            }
                        } else {
                            match key {
                                Key::Enter => raw_bytes.push(b'\r'),
                                Key::Backspace => raw_bytes.push(0x7f),
                                Key::Tab => raw_bytes.push(b'\t'),
                                Key::Escape => raw_bytes.push(0x1b),
                                Key::ArrowUp => raw_bytes.extend_from_slice(b"\x1b[A"),
                                Key::ArrowDown => raw_bytes.extend_from_slice(b"\x1b[B"),
                                Key::ArrowRight => raw_bytes.extend_from_slice(b"\x1b[C"),
                                Key::ArrowLeft => raw_bytes.extend_from_slice(b"\x1b[D"),
                                Key::Home => raw_bytes.extend_from_slice(b"\x1b[H"),
                                Key::End => raw_bytes.extend_from_slice(b"\x1b[F"),
                                Key::Delete => raw_bytes.extend_from_slice(b"\x1b[3~"),
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }
        });
        if !raw_bytes.is_empty() {
            tab.write_bytes(&raw_bytes);
        }
    }

    clipped(ui, viewport_rect, |ui| {
        let p = ui.painter().clone();
        let Ok(parser) = tab.parser.lock() else {
            return;
        };
        let screen = parser.screen();
        let (cur_row, cur_col) = screen.cursor_position();
        let font_id = FontId::new(11.5, FontFamily::Monospace);

        for r in 0..rows {
            let row_y = viewport_rect.top() + pad_y + r as f32 * ROW_H;
            if row_y + ROW_H < viewport_rect.top() || row_y > viewport_rect.bottom() {
                continue;
            }

            let mut job = LayoutJob::default();
            let mut run_text = String::new();
            let mut run_style: Option<(Color32, Color32, bool, bool, bool)> = None;

            for c in 0..cols {
                let Some(cell) = screen.cell(r, c) else {
                    continue;
                };
                if cell.is_wide_continuation() {
                    continue;
                }
                let mut fg = vt100_color_to_egui(cell.fgcolor(), true);
                let mut bg = vt100_color_to_egui(cell.bgcolor(), false);
                if cell.inverse() {
                    std::mem::swap(&mut fg, &mut bg);
                    if bg == Color32::TRANSPARENT {
                        bg = TERM_FG_DEFAULT;
                    }
                    if fg == Color32::TRANSPARENT {
                        fg = TERM_BG_VIEWPORT;
                    }
                }
                let style_key = (fg, bg, cell.bold(), cell.italic(), cell.underline());
                if run_style != Some(style_key) {
                    if let Some((p_fg, p_bg, _bold, italics, underline)) = run_style {
                        if !run_text.is_empty() {
                            job.append(
                                &run_text,
                                0.0,
                                TextFormat {
                                    font_id: font_id.clone(),
                                    color: p_fg,
                                    background: p_bg,
                                    italics,
                                    underline: if underline { Stroke::new(1.0, p_fg) } else { Stroke::NONE },
                                    ..Default::default()
                                },
                            );
                            run_text.clear();
                        }
                    }
                    run_style = Some(style_key);
                }
                let contents = cell.contents();
                if contents.is_empty() {
                    run_text.push(' ');
                } else {
                    run_text.push_str(&contents);
                }
            }

            let trimmed_len = run_text.trim_end_matches(' ').len();
            let has_bg = run_style.is_some_and(|(_, bg, _, _, _)| bg != Color32::TRANSPARENT);
            if !has_bg {
                run_text.truncate(trimmed_len);
            }
            if let Some((p_fg, p_bg, _bold, italics, underline)) = run_style {
                if !run_text.is_empty() {
                    job.append(
                        &run_text,
                        0.0,
                        TextFormat {
                            font_id: font_id.clone(),
                            color: p_fg,
                            background: p_bg,
                            italics,
                            underline: if underline { Stroke::new(1.0, p_fg) } else { Stroke::NONE },
                            ..Default::default()
                        },
                    );
                }
            }

            if !job.text.is_empty() {
                let galley = ui.fonts_mut(|f| f.layout_job(job));
                p.galley(Pos2::new(viewport_rect.left() + pad_x, row_y), galley, TERM_FG_DEFAULT);
            }
        }

        if !screen.hide_cursor() && cur_row < rows && cur_col < cols {
            let cx = viewport_rect.left() + pad_x + cur_col as f32 * CHAR_W;
            let cy = viewport_rect.top() + pad_y + cur_row as f32 * ROW_H;
            let cursor_rect = Rect::from_min_size(Pos2::new(cx, cy), Vec2::new(CHAR_W, ROW_H - 1.0));
            let cursor_col = if viewport_focused { TERM_ORANGE.gamma_multiply(0.75) } else { TERM_BORDER };
            p.rect_filled(cursor_rect, CornerRadius::same(1), cursor_col);
        }
    });
}

fn render_prompt_composer(state: &mut TerminalState, ui: &mut Ui, bar_rect: Rect, surface_id: &'static str, actions: &mut Vec<TerminalAction>) {
    let p = ui.painter().clone();
    p.rect_filled(bar_rect, 0.0, TERM_BG_BAR);
    widgets::hline(&p, bar_rect.left(), bar_rect.right(), bar_rect.top(), TERM_BORDER);

    // Row 1 (top 24px): Quick Action Chips + semantic status badge
    let chips_y = bar_rect.top() + 13.0;
    let mut chip_x = bar_rect.left() + 8.0;

    for (chip_id, chip_label, chip_cmd, is_apply_spec) in [
        ("chip-claude", "claude", "claude", false),
        ("chip-codex", "codex", "codex", false),
        ("chip-help", "designos --help", "\"$DESIGNOS_BIN\" --help", false),
        ("chip-console-spec", "Apply EaseUI Console Spec", "", true),
    ] {
        let cw = (chip_label.len() as f32 * 6.0 + 16.0).clamp(52.0, 172.0);
        if chip_x + cw > bar_rect.right() - 12.0 {
            break;
        }
        let c_rect = Rect::from_min_size(Pos2::new(chip_x, chips_y - 9.0), Vec2::new(cw, 18.0));
        let c_resp = ui.interact(c_rect, Id::new((surface_id, chip_id)), Sense::click());
        let bg = if c_resp.hovered() {
            TERM_BG_HOVER
        } else if is_apply_spec {
            STATUS_WARN_BG
        } else {
            TERM_BG_ELEVATED
        };
        let fg = if is_apply_spec { STATUS_WARN_TEXT } else { theme::TEXT_SECONDARY };
        let border = if is_apply_spec { STATUS_WARN_BORDER } else { TERM_BORDER };
        p.rect_filled(c_rect, CornerRadius::same(4), bg);
        p.rect_stroke(c_rect, CornerRadius::same(4), Stroke::new(1.0, border), StrokeKind::Inside);
        let elided = widgets::elide(&p, chip_label, &theme::font_small_medium(), (cw - 8.0).max(16.0));
        p.text(c_rect.center(), Align2::CENTER_CENTER, elided, theme::font_small_medium(), fg);
        if c_resp.clicked() {
            if is_apply_spec {
                actions.push(TerminalAction::ApplySpecFile(PathBuf::from("examples/specs/easeui-agent-console.spec.json")));
            } else if let Some(tab) = state.active_tab_mut() {
                tab.send_line(chip_cmd);
            }
        }
        chip_x = c_rect.right() + 6.0;
    }

    if let Some(status) = &state.status_msg {
        let avail_w = (bar_rect.right() - chip_x - 10.0).max(0.0);
        if avail_w > 60.0 {
            let msg = widgets::elide(&p, status, &theme::font_small(), avail_w);
            p.text(Pos2::new(bar_rect.right() - 10.0, chips_y), Align2::RIGHT_CENTER, msg, theme::font_small(), STATUS_OK_TEXT);
        }
    }

    // Row 2 (bottom 32px): Singleline TextEdit composer + Send + Ctrl+C + Clear
    let input_y = bar_rect.top() + 26.0;
    let btn_h = 24.0;
    let clear_w = 48.0;
    let ctrlc_w = 54.0;
    let send_w = 62.0;

    let clear_rect = Rect::from_min_size(Pos2::new(bar_rect.right() - clear_w - 8.0, input_y), Vec2::new(clear_w, btn_h));
    let ctrlc_rect = Rect::from_min_size(Pos2::new(clear_rect.left() - ctrlc_w - 6.0, input_y), Vec2::new(ctrlc_w, btn_h));
    let send_rect = Rect::from_min_size(Pos2::new(ctrlc_rect.left() - send_w - 6.0, input_y), Vec2::new(send_w, btn_h));
    let edit_rect =
        Rect::from_min_max(Pos2::new(bar_rect.left() + 8.0, input_y), Pos2::new((send_rect.left() - 8.0).max(bar_rect.left() + 80.0), input_y + btn_h));

    p.rect_filled(edit_rect, CornerRadius::same(6), TERM_BG_VIEWPORT);
    p.rect_stroke(edit_rect, CornerRadius::same(6), Stroke::new(1.0, TERM_BORDER), StrokeKind::Inside);

    let mut submit_input = false;
    ui.scope_builder(egui::UiBuilder::new().max_rect(edit_rect.shrink2(Vec2::new(8.0, 2.0))), |ui| {
        let te = egui::TextEdit::singleline(&mut state.input_line)
            .frame(egui::Frame::NONE)
            .font(theme::font_mono())
            .text_color(theme::TEXT)
            .hint_text("Type command or prompt for Claude/Codex (Enter to send to PTY, Ctrl+C to interrupt)…")
            .desired_width(edit_rect.width() - 16.0);
        let resp = ui.add(te);
        if resp.clicked() || resp.has_focus() {
            state.viewport_focused = false;
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            submit_input = true;
            resp.request_focus();
        }
    });

    let send_resp = ui.interact(send_rect, Id::new((surface_id, "term-send-btn")), Sense::click());
    p.rect_filled(send_rect, CornerRadius::same(6), if send_resp.hovered() { theme::BG_BRAND_HOVER } else { TERM_ORANGE });
    icons::paint(&p, Icon::Play, Pos2::new(send_rect.left() + 12.0, send_rect.center().y), 10.0, Color32::WHITE);
    p.text(Pos2::new(send_rect.left() + 21.0, send_rect.center().y), Align2::LEFT_CENTER, "Send", theme::font_small_strong(), Color32::WHITE);
    if send_resp.clicked() {
        submit_input = true;
    }

    if submit_input {
        let line = std::mem::take(&mut state.input_line);
        if let Some(tab) = state.active_tab_mut() {
            tab.send_line(&line);
        }
    }

    let ctrlc_resp = ui.interact(ctrlc_rect, Id::new((surface_id, "term-ctrlc-btn")), Sense::click());
    p.rect_filled(ctrlc_rect, CornerRadius::same(6), if ctrlc_resp.hovered() { TERM_BG_HOVER } else { TERM_BG_ELEVATED });
    p.rect_stroke(ctrlc_rect, CornerRadius::same(6), Stroke::new(1.0, TERM_BORDER), StrokeKind::Inside);
    p.text(ctrlc_rect.center(), Align2::CENTER_CENTER, "Ctrl+C", theme::font_small_medium(), theme::TEXT_SECONDARY);
    if ctrlc_resp.clicked() {
        if let Some(tab) = state.active_tab_mut() {
            tab.write_bytes(&[0x03]);
        }
    }

    let clear_resp = ui.interact(clear_rect, Id::new((surface_id, "term-clear-btn")), Sense::click());
    p.rect_filled(clear_rect, CornerRadius::same(6), if clear_resp.hovered() { TERM_BG_HOVER } else { TERM_BG_ELEVATED });
    p.rect_stroke(clear_rect, CornerRadius::same(6), Stroke::new(1.0, TERM_BORDER), StrokeKind::Inside);
    p.text(clear_rect.center(), Align2::CENTER_CENTER, "Clear", theme::font_small_medium(), theme::TEXT_SECONDARY);
    if clear_resp.clicked() {
        if let Some(tab) = state.active_tab_mut() {
            tab.write_bytes(&[0x0c]);
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_bar_icon_pill(
    ui: &mut Ui,
    p: &Painter,
    id: impl std::hash::Hash + std::fmt::Debug,
    right_center: Pos2,
    icon: Icon,
    label: &str,
    bg: Color32,
    border: Color32,
    fg: Color32,
) -> Rect {
    let w = (label.len() as f32 * 5.8 + 30.0).clamp(84.0, 184.0);
    let h = 22.0;
    let rect = Rect::from_min_size(Pos2::new(right_center.x - w, right_center.y - h / 2.0), Vec2::new(w, h));
    let resp = ui.interact(rect, Id::new(id), Sense::hover());
    let fill = if resp.hovered() { TERM_BG_HOVER } else { bg };
    p.rect_filled(rect, CornerRadius::same(6), fill);
    p.rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0, border), StrokeKind::Inside);
    icons::paint(p, icon, Pos2::new(rect.left() + 11.0, rect.center().y), 11.0, fg);
    let elided = widgets::elide(p, label, &theme::font_small_medium(), (w - 24.0).max(20.0));
    p.text(Pos2::new(rect.left() + 20.0, rect.center().y), Align2::LEFT_CENTER, elided, theme::font_small_medium(), fg);
    rect
}

/// Converts a `vt100::Color` (Default, 16-ANSI, 256-palette, or 24-bit TrueColor RGB) to `egui::Color32`.
pub fn vt100_color_to_egui(c: vt100::Color, is_fg: bool) -> Color32 {
    match c {
        vt100::Color::Default => {
            if is_fg {
                TERM_FG_DEFAULT
            } else {
                Color32::TRANSPARENT
            }
        }
        vt100::Color::Rgb(r, g, b) => Color32::from_rgb(r, g, b),
        vt100::Color::Idx(idx) => ansi_256_to_color32(idx),
    }
}

fn ansi_256_to_color32(idx: u8) -> Color32 {
    const ANSI_16: [(u8, u8, u8); 16] = [
        (0x18, 0x18, 0x1B), // 0 Black
        (0xEF, 0x44, 0x44), // 1 Red
        (0x10, 0xB9, 0x81), // 2 Green
        (0xF5, 0x9E, 0x0B), // 3 Yellow
        (0x3B, 0x82, 0xF6), // 4 Blue
        (0xA8, 0x55, 0xF7), // 5 Magenta
        (0x06, 0xB6, 0xD4), // 6 Cyan
        (0xE4, 0xE4, 0xE7), // 7 White
        (0x71, 0x71, 0x7A), // 8 Bright Black (Zinc-500)
        (0xF8, 0x71, 0x71), // 9 Bright Red
        (0x34, 0xD3, 0x99), // 10 Bright Green
        (0xFB, 0xBF, 0x24), // 11 Bright Yellow
        (0x60, 0xA5, 0xFA), // 12 Bright Blue
        (0xC0, 0x84, 0xFC), // 13 Bright Magenta
        (0x22, 0xD3, 0xEE), // 14 Bright Cyan
        (0xFA, 0xFA, 0xFA), // 15 Bright White
    ];
    match idx {
        0..=15 => {
            let (r, g, b) = ANSI_16[idx as usize];
            Color32::from_rgb(r, g, b)
        }
        16..=231 => {
            let n = idx - 16;
            let r_idx = n / 36;
            let g_idx = (n % 36) / 6;
            let b_idx = n % 6;
            let to_u8 = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            Color32::from_rgb(to_u8(r_idx), to_u8(g_idx), to_u8(b_idx))
        }
        232..=255 => {
            let gray = 8 + (idx - 232) * 10;
            Color32::from_rgb(gray, gray, gray)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_spec_json_from_fenced_and_raw_terminal_output() {
        let log = r#"
$ claude "design an EaseUI card"
Here is the NodeSpec blueprint:
```json
{
  "name": "Agent Card",
  "preset": "easeui-card",
  "children": [
    { "kind": "text", "name": "Title", "text": "Hello from Claude" }
  ]
}
```
Done!
"#;
        let extracted = extract_spec_json_from_log(log).expect("should extract fenced NodeSpec JSON");
        let parsed: NodeSpec = serde_json::from_str(&extracted).expect("valid NodeSpec");
        assert_eq!(parsed.name, "Agent Card");
        assert_eq!(parsed.preset.as_deref(), Some("easeui-card"));
        assert_eq!(parsed.children.len(), 1);
    }

    #[test]
    fn pty_spawn_and_vt100_screen_capture_work() {
        let mut tab = TerminalTab::spawn(1, AgentCliPreset::Shell, 24, 80, None).expect("should spawn shell PTY");
        tab.send_line("echo DESIGNOS_PTY_OK");

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut found = false;
        while std::time::Instant::now() < deadline {
            if let Ok(p) = tab.parser.lock() {
                if p.screen().contents().contains("DESIGNOS_PTY_OK") {
                    found = true;
                    break;
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        let _ = tab.child.kill();
        assert!(found, "VT100 screen should capture 'DESIGNOS_PTY_OK' from PTY stdout");
    }

    #[test]
    fn send_copy_for_ai_formats_product_os_citation_and_opens_dock() {
        let payload = format_copy_for_ai_payload("## US-01: Native PTY Agent Terminal");
        assert!(payload.contains("- File: spec/"));
        assert!(payload.contains("- Mục:"));
        assert!(payload.contains("- Trang reader:"));
        assert!(payload.contains("- Project:"));
        assert!(payload.contains("Yêu cầu của tôi:"));
    }
}
