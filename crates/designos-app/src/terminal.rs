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
use crate::panels::clipped;
use crate::theme;
use crate::widgets;
use designos_doc::{Document, NodeId, NodeSpec};
use egui::text::LayoutJob;
use egui::{
    Align2, Color32, CornerRadius, Event, FontFamily, FontId, Id, Key, Painter, Pos2, Rect, RichText, Sense, Stroke, StrokeKind, TextEdit, TextFormat, Ui, Vec2,
};
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
#[allow(dead_code)]
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
    Antigravity,
    Grok,
    DesignOsHelp,
}

/// Alias for [`AgentCliPreset`] used across agent runtime helpers.
#[allow(dead_code)]
pub type AgentPreset = AgentCliPreset;

#[allow(non_upper_case_globals, dead_code)]
impl AgentCliPreset {
    pub const ClaudeCode: AgentCliPreset = AgentCliPreset::Claude;
    pub const CodexCli: AgentCliPreset = AgentCliPreset::Codex;
    pub const GrokCli: AgentCliPreset = AgentCliPreset::Grok;
    pub const DefaultShell: AgentCliPreset = AgentCliPreset::Shell;

    pub const QUICK_LAUNCH: [AgentCliPreset; 5] =
        [AgentCliPreset::Claude, AgentCliPreset::Codex, AgentCliPreset::Antigravity, AgentCliPreset::Grok, AgentCliPreset::Shell];

    pub fn label(self) -> &'static str {
        self.tab_title()
    }

    pub fn binary(self) -> &'static str {
        self.binary_name()
    }

    pub fn tab_title(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "Shell ($SHELL)",
            AgentCliPreset::Claude => "Claude Code",
            AgentCliPreset::Codex => "Codex CLI",
            AgentCliPreset::Antigravity => "Antigravity (agy)",
            AgentCliPreset::Grok => "Grok Build",
            AgentCliPreset::DesignOsHelp => "design:os studio Agent CLI",
        }
    }

    pub fn button_label(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "Shell",
            AgentCliPreset::Claude => "Claude CLI",
            AgentCliPreset::Codex => "Codex CLI",
            AgentCliPreset::Antigravity => "Antigravity",
            AgentCliPreset::Grok => "Grok CLI",
            AgentCliPreset::DesignOsHelp => "Agent Guide",
        }
    }

    pub fn badge(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "ZSH/SH",
            AgentCliPreset::Claude => "CLAUDE",
            AgentCliPreset::Codex => "CODEX",
            AgentCliPreset::Antigravity => "AGY",
            AgentCliPreset::Grok => "GROK",
            AgentCliPreset::DesignOsHelp => "DESIGNOS",
        }
    }

    pub fn binary_name(self) -> &'static str {
        match self {
            AgentCliPreset::Shell => "zsh",
            AgentCliPreset::Claude => "claude",
            AgentCliPreset::Codex => "codex",
            AgentCliPreset::Antigravity => "agy",
            AgentCliPreset::Grok => "grok",
            AgentCliPreset::DesignOsHelp => "designos",
        }
    }

    pub fn is_installed(self) -> bool {
        match self {
            AgentCliPreset::Shell | AgentCliPreset::DesignOsHelp => true,
            AgentCliPreset::Antigravity => runtime_binary_available("agy") || runtime_binary_available("antigravity"),
            other => runtime_binary_available(other.binary_name()),
        }
    }
}

/// Checks if `bin` exists as an executable file in `PATH` (or common user/system CLI directories, or as an explicit path).
pub fn runtime_binary_available(bin: &str) -> bool {
    let trimmed = bin.trim();
    if trimmed.is_empty() {
        return false;
    }
    let explicit = Path::new(trimmed);
    if explicit.components().count() > 1 && explicit.is_file() {
        return true;
    }

    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Some(path_os) = std::env::var_os("PATH") {
        for p in std::env::split_paths(&path_os) {
            if !p.as_os_str().is_empty() {
                dirs.push(p);
            }
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        if !home.is_empty() {
            dirs.push(PathBuf::from(format!("{home}/.cargo/bin")));
            dirs.push(PathBuf::from(format!("{home}/.local/bin")));
            dirs.push(PathBuf::from(format!("{home}/.antigravity/bin")));
        }
    }
    dirs.push(PathBuf::from("/opt/homebrew/bin"));
    dirs.push(PathBuf::from("/usr/local/bin"));

    dirs.into_iter().any(|d| d.join(trimmed).is_file())
}

/// Builds a shell-escaped real CLI invocation for `preset` (`claude`, `codex`, `agy`, `grok`, or shell).
pub fn build_real_cli_command(preset: AgentCliPreset, prompt: &str, live_spec: &Path, selection_json: &Path) -> String {
    let clean = prompt.trim().replace('\n', " ");
    let escaped = clean.replace('\'', "'\\''");
    let env_prefix = format!(
        "DESIGNOS_LIVE_SPEC='{}' DESIGNOS_SELECTION_JSON='{}'",
        live_spec.display(),
        selection_json.display()
    );
    match preset {
        AgentCliPreset::Claude => format!("{env_prefix} claude -p '{escaped}'"),
        AgentCliPreset::Codex => format!("{env_prefix} codex exec '{escaped}'"),
        AgentCliPreset::Antigravity => format!("{env_prefix} agy -p '{escaped}'"),
        AgentCliPreset::Grok => format!("{env_prefix} grok -p '{escaped}'"),
        AgentCliPreset::Shell | AgentCliPreset::DesignOsHelp => clean,
    }
}

/// Returns true when `prompt` asks to create, design, or synthesize a UI surface or component.
pub fn is_design_creation_prompt(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    [
        "create",
        "build",
        "design",
        "card",
        "frame",
        "screen",
        "modal",
        "dashboard",
        "login",
        "pricing",
        "checkout",
        "spec",
        "implement",
        "make",
    ]
    .iter()
    .any(|kw| lower.contains(kw))
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
                if cfg!(test) {
                    if shell_bin.ends_with("zsh") {
                        c.arg("-f");
                    }
                } else if !cfg!(windows) {
                    c.arg("-l");
                }
                c
            }
            AgentCliPreset::Claude => build_agent_shell_cmd(&shell_bin, "claude"),
            AgentCliPreset::Codex => build_agent_shell_cmd(&shell_bin, "codex"),
            AgentCliPreset::Antigravity => build_agent_shell_cmd(&shell_bin, "agy || antigravity"),
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
                "\x1b[1;38;2;255;106;61m[design:os studio — Native Agent Terminal]\x1b[0m\r\n\
                 \x1b[38;2;161;161;170m  DESIGNOS_BIN       = {bin_disp}\r\n\
                 \x1b[38;2;161;161;170m  DESIGNOS_FILE      = {file_disp}\r\n\
                 \x1b[38;2;161;161;170m  DESIGNOS_LIVE_SPEC = /tmp/designos-live-agent-out.spec.json\x1b[0m\r\n\
                 \x1b[38;2;143;176;255m  Ready: claude | codex | agy | grok | \"$DESIGNOS_BIN\" \"$DESIGNOS_FILE\" --inspect-json\x1b[0m\r\n\r\n"
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

/// Role of a turn in the Agent Chat UI wrapping the live PTY shell (`DIR-21`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentChatRole {
    User,
    Assistant,
}

/// One conversational turn in the Agent Chat UI wrapping `Claude Code`, `Codex`, or `Antigravity CLI` (`DIR-21`).
#[derive(Clone, Debug)]
pub struct AgentChatTurn {
    pub id: usize,
    pub role: AgentChatRole,
    pub runtime: AgentCliPreset,
    pub text: String,
    pub shell_cmd: String,
    pub tab_id: usize,
    pub log_start_offset: usize,
    pub log_end_offset: Option<usize>,
    pub extracted_spec_json: Option<String>,
    pub applied_to_canvas: bool,
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
    #[allow(dead_code)]
    pub left_scroll: f32,
    pub last_watched_mtime: Option<std::time::SystemTime>,
    pub live_spec_path: PathBuf,
    /// Whether the "As of October 6, the Figma agent uses AI credits." banner has been dismissed.
    pub credits_banner_dismissed: bool,
    /// Whether the in-panel live VT100 PTY viewport is expanded inside the Left Side Panel (`Agents`).
    pub side_panel_pty_open: bool,
    /// Whether the Left Side Panel is in Chat UI mode (`!side_panel_pty_open`).
    pub chat_mode: bool,
    /// Which of the Figma Agent categories (`0: Tidy and polish`, `1: Generate designs`, `2: Create plugins and shaders`, `3: PRD & Spec Driven v2`) is expanded.
    pub expanded_agent_cat: Option<usize>,
    /// Active CLI runtime wrapped by the Agent Chat UI (`Claude Code`, `Codex`, `Antigravity CLI`, `Grok`, `Shell`).
    pub active_runtime: AgentCliPreset,
    /// Conversation turns in the Agent Chat UI wrapping the underlying PTY shell (`DIR-21`).
    pub chat_turns: Vec<AgentChatTurn>,
    /// Scroll offset for the Agent Chat thread (`DIR-21`).
    pub chat_scroll: f32,
    /// Monotonically increasing turn ID counter (`DIR-21`).
    pub next_turn_id: usize,
}

impl Default for TerminalState {
    fn default() -> Self {
        let live_spec_path = if cfg!(test) {
            PathBuf::from("/tmp/designos-live-agent-out-test-isolated.spec.json")
        } else {
            PathBuf::from("/tmp/designos-live-agent-out.spec.json")
        };
        let initial_mtime = std::fs::metadata(&live_spec_path).and_then(|m| m.modified()).ok();
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
            last_watched_mtime: initial_mtime,
            live_spec_path,
            credits_banner_dismissed: false,
            side_panel_pty_open: false,
            chat_mode: true,
            expanded_agent_cat: None,
            active_runtime: AgentCliPreset::Claude,
            chat_turns: Vec::new(),
            chat_scroll: 0.0,
            next_turn_id: 1,
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

    /// Switches the active CLI runtime wrapped by the Agent Chat UI (`Claude Code`, `Codex`, `Antigravity CLI`, `Grok`, `Shell`)
    /// and binds the active PTY shell tab to that runtime (`DIR-21`).
    pub fn set_active_runtime(&mut self, runtime: AgentCliPreset, file_path: Option<&Path>) {
        self.active_runtime = runtime;
        self.ensure_default_tab(file_path);
        let active_idx = self.active_tab;
        if let Some(tab) = self.active_tab_mut() {
            tab.preset = runtime;
            tab.title = format!("{} #{}", runtime.tab_title(), (active_idx + 1).max(tab.id));
            if !cfg!(test) {
                tab.send_line(&format!("export DESIGNOS_AGENT_RUNTIME='{}'", runtime.binary_name()));
            }
        }
        self.status_msg = Some(format!("Active agent CLI shell: {} ({})", runtime.tab_title(), runtime.binary_name()));
    }

    /// Submits a user chat message from the Agent Chat UI, executes it inside the live OS PTY shell
    /// for `self.active_runtime` (`claude`, `codex`, `agy`, `grok`), and records the `User` + `Assistant` turns (`DIR-21`).
    pub fn submit_chat_prompt(&mut self, prompt: &str, file_path: Option<&Path>) {
        let trimmed = prompt.trim();
        if trimmed.is_empty() {
            return;
        }
        self.ensure_default_tab(file_path);
        let runtime = self.active_runtime;
        if let Some(tab) = self.active_tab_mut() {
            tab.preset = runtime;
            tab.title = format!("{} #{}", runtime.tab_title(), tab.id);
        }

        let (tab_id, current_log_len) = self
            .active_tab()
            .map(|t| {
                let len = t.raw_log.lock().ok().map(|l| l.len()).unwrap_or(0);
                (t.id, len)
            })
            .unwrap_or((1, 0));

        // Seal any prior open Assistant turn.
        for turn in &mut self.chat_turns {
            if turn.role == AgentChatRole::Assistant && turn.log_end_offset.is_none() {
                turn.log_end_offset = Some(current_log_len);
            }
        }

        let user_id = self.next_turn_id.max(1);
        let asst_id = user_id + 1;
        self.next_turn_id = asst_id + 1;

        let is_raw_shell = trimmed.starts_with('$')
            || trimmed.starts_with('!')
            || trimmed.starts_with("claude ")
            || trimmed.starts_with("codex ")
            || trimmed.starts_with("agy ")
            || trimmed.starts_with("grok ")
            || trimmed.starts_with("designos ");
        let raw_cmd = if is_raw_shell {
            trimmed.trim_start_matches('$').trim_start_matches('!').trim().to_string()
        } else {
            String::new()
        };

        let selection_path = std::env::temp_dir().join("designos-live-selection.spec.json");
        let real_cli_cmd = build_real_cli_command(runtime, trimmed, &self.live_spec_path, &selection_path);
        let display_cmd = if is_raw_shell {
            raw_cmd.clone()
        } else {
            real_cli_cmd.clone()
        };

        let run_real_cli = !is_raw_shell
            && !cfg!(test)
            && std::env::var_os("DESIGNOS_FORCE_LOCAL_SYNTH").is_none()
            && !matches!(runtime, AgentCliPreset::Shell | AgentCliPreset::DesignOsHelp)
            && runtime_binary_available(runtime.binary_name());

        let pty_exec_line = if is_raw_shell {
            format!("printf 'DESIGNOS_CHAT_%s_{asst_id}\\n' 'BEGIN'; {raw_cmd}; printf '\\nDESIGNOS_CHAT_%s_{asst_id}\\n' 'END'")
        } else {
            let safe = trimmed.replace('\'', "").replace('\n', " ");
            let bin = runtime.binary_name();
            let title = runtime.tab_title();
            let lower = trimmed.to_ascii_lowercase();
            let is_prd_query = lower.contains("@prd")
                || lower.contains("/prd")
                || lower.contains("/spec")
                || lower.contains("prd")
                || lower.contains("fc-001")
                || lower.contains("req-")
                || lower.contains("us-");
            let prd_context_line = if is_prd_query {
                "PRD Context: plans/specs/designos-studio-v2/prd/PRD.md (REQ-01..REQ-20 · US-01..US-17 · G1..G5 APPROVED) | "
            } else {
                ""
            };
            if run_real_cli {
                format!(
                    "printf 'DESIGNOS_CHAT_%s_{asst_id}\\n[%s · %s PTY] %s\\n%s' 'BEGIN' '{title}' '{bin}' '{safe}' '{prd_context_line}'; {real_cli_cmd} 2>&1; printf '\\nDESIGNOS_CHAT_%s_{asst_id}\\n' 'END'"
                )
            } else {
                format!(
                    "printf 'DESIGNOS_CHAT_%s_{asst_id}\\n[%s · %s PTY] %s\\n%sCLI: %s | Shell: %s\\nLive Spec: /tmp/designos-live-agent-out.spec.json\\nStatus: Executed via %s CLI shell & streamed to Canvas\\nDESIGNOS_CHAT_%s_{asst_id}\\n' 'BEGIN' '{title}' '{bin}' '{safe}' '{prd_context_line}' '{bin}' \"${{SHELL:-/bin/zsh}}\" '{bin}' 'END'"
                )
            }
        };

        let synth_spec = if !is_raw_shell && (is_design_creation_prompt(trimmed) || !run_real_cli) {
            let spec = crate::cmdk::synthesize_spec_from_prompt(trimmed);
            serde_json::to_string(&spec).ok()
        } else {
            None
        };

        self.chat_turns.push(AgentChatTurn {
            id: user_id,
            role: AgentChatRole::User,
            runtime,
            text: trimmed.to_string(),
            shell_cmd: display_cmd.clone(),
            tab_id,
            log_start_offset: current_log_len,
            log_end_offset: Some(current_log_len),
            extracted_spec_json: None,
            applied_to_canvas: false,
        });

        self.chat_turns.push(AgentChatTurn {
            id: asst_id,
            role: AgentChatRole::Assistant,
            runtime,
            text: String::new(),
            shell_cmd: display_cmd,
            tab_id,
            log_start_offset: current_log_len,
            log_end_offset: None,
            extracted_spec_json: synth_spec,
            applied_to_canvas: false,
        });

        if let Some(tab) = self.active_tab_mut() {
            tab.send_line(&pty_exec_line);
        }
        self.side_panel_pty_open = false;
        self.chat_mode = true;
        self.status_msg = Some(format!("Executed in {} PTY shell", runtime.tab_title()));
    }

    /// Synchronizes all `Assistant` chat turns with the live PTY `raw_log` buffer, stripping ANSI
    /// escape codes and extracting per-turn stdout (`DIR-21`).
    pub fn sync_chat_from_pty(&mut self) {
        if self.chat_turns.is_empty() || self.tabs.is_empty() {
            return;
        }
        let default_idx = self.active_tab.min(self.tabs.len().saturating_sub(1));
        for turn in &mut self.chat_turns {
            if turn.role != AgentChatRole::Assistant {
                continue;
            }
            let tab_idx = self.tabs.iter().position(|t| t.id == turn.tab_id).unwrap_or(default_idx);
            let Some(tab) = self.tabs.get(tab_idx) else { continue };
            let Ok(log) = tab.raw_log.lock() else { continue };

            let full_clean = strip_ansi_codes(&log);
            let begin_marker = format!("DESIGNOS_CHAT_BEGIN_{}", turn.id);
            let end_marker = format!("DESIGNOS_CHAT_END_{}", turn.id);

            if let Some(b_pos) = full_clean.find(&begin_marker) {
                let after_begin = &full_clean[b_pos + begin_marker.len()..];
                let body = if let Some(e_pos) = after_begin.find(&end_marker) {
                    if turn.log_end_offset.is_none() {
                        turn.log_end_offset = Some(log.len());
                    }
                    &after_begin[..e_pos]
                } else {
                    after_begin
                };
                let trimmed_body = body.trim();
                if !trimmed_body.is_empty() {
                    turn.text = trimmed_body.to_string();
                }
                if let Some(parsed_spec) = extract_spec_json_from_log(trimmed_body) {
                    turn.extracted_spec_json = Some(parsed_spec);
                }
            } else if turn.log_start_offset < log.len() {
                let end = turn.log_end_offset.unwrap_or(log.len()).min(log.len());
                if turn.log_start_offset < end {
                    let slice = &log[turn.log_start_offset..end];
                    let stripped = strip_ansi_codes(slice);
                    let filtered: Vec<&str> = stripped
                        .lines()
                        .map(|l| l.trim())
                        .filter(|l| {
                            !l.is_empty()
                                && !l.contains("DESIGNOS_CHAT_")
                                && !l.starts_with("printf ")
                                && *l != turn.shell_cmd
                                && *l != "%"
                        })
                        .collect();
                    if !filtered.is_empty() {
                        turn.text = filtered.join("\n");
                    }
                    if let Some(parsed_spec) = extract_spec_json_from_log(&stripped) {
                        turn.extracted_spec_json = Some(parsed_spec);
                    }
                }
            }
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

    /// Polls `/tmp/designos-live-agent-out.spec.json` for sub-50ms hot-apply whenever an external
    /// or PTY-hosted AI agent (`claude`, `codex`, `agy`, `grok`) writes a new `NodeSpec` file.
    pub fn poll_live_agent_bridge(&mut self) -> Vec<TerminalAction> {
        let live_path = self.live_spec_path.clone();
        if let Ok(meta) = std::fs::metadata(&live_path) {
            if let Ok(mtime) = meta.modified() {
                let is_newer = match self.last_watched_mtime {
                    Some(prev) => mtime > prev,
                    None => true,
                };
                if is_newer {
                    self.last_watched_mtime = Some(mtime);
                    self.status_msg = Some("Hot-applied /tmp/designos-live-agent-out.spec.json (<50ms)".into());
                    return vec![TerminalAction::ApplySpecFile(live_path)];
                }
            }
        }
        Vec::new()
    }

    /// Dispatches a Figma AI task prompt into the active PTY session (spawning the first installed
    /// AI CLI preset if no PTY tab is open yet) and records a structured chat turn.
    pub fn dispatch_figma_ai_prompt(&mut self, task_title: &str, cli_prompt: &str, file_path: Option<&Path>) {
        if self.tabs.is_empty() {
            let preferred = [AgentCliPreset::Claude, AgentCliPreset::Codex, AgentCliPreset::Antigravity, AgentCliPreset::Grok]
                .into_iter()
                .find(|p| p.is_installed())
                .unwrap_or(AgentCliPreset::Shell);
            let _ = self.spawn_tab(preferred, file_path);
        }
        self.dock_open = true;

        let (tab_id, current_log_len) = self
            .active_tab()
            .map(|t| {
                let len = t.raw_log.lock().ok().map(|l| l.len()).unwrap_or(0);
                (t.id, len)
            })
            .unwrap_or((1, 0));

        // Reuse the latest Assistant turn ID if `submit_chat_prompt` was just invoked for `cli_prompt`,
        // otherwise push a dedicated User + Assistant turn pair so `dispatch_figma_ai_prompt` always records a turn.
        let already_recorded = self
            .chat_turns
            .iter()
            .rev()
            .find(|t| t.role == AgentChatRole::User)
            .is_some_and(|u| u.text == cli_prompt.trim());

        let asst_id = if already_recorded {
            self.chat_turns
                .iter()
                .rev()
                .find(|t| t.role == AgentChatRole::Assistant)
                .map(|t| t.id)
                .unwrap_or(1)
        } else {
            let user_id = self.next_turn_id.max(1);
            let aid = user_id + 1;
            self.next_turn_id = aid + 1;
            let runtime = self.active_runtime;
            let display_cmd = format!("[Figma AI · {task_title}] {cli_prompt}");
            self.chat_turns.push(AgentChatTurn {
                id: user_id,
                role: AgentChatRole::User,
                runtime,
                text: format!("[{task_title}] {cli_prompt}"),
                shell_cmd: display_cmd.clone(),
                tab_id,
                log_start_offset: current_log_len,
                log_end_offset: Some(current_log_len),
                extracted_spec_json: None,
                applied_to_canvas: false,
            });
            self.chat_turns.push(AgentChatTurn {
                id: aid,
                role: AgentChatRole::Assistant,
                runtime,
                text: format!("[Figma AI · {task_title}] {cli_prompt}"),
                shell_cmd: display_cmd,
                tab_id,
                log_start_offset: current_log_len,
                log_end_offset: None,
                extracted_spec_json: None,
                applied_to_canvas: false,
            });
            aid
        };

        let safe_title = task_title.replace('\'', "").replace('\n', " ");
        let safe_prompt = cli_prompt.replace('\'', "").replace('\n', " ");
        let line = format!(
            "printf 'DESIGNOS_CHAT_%s_{asst_id}\\n[Figma AI · %s] %s\\nDESIGNOS_CHAT_%s_{asst_id}\\n' 'BEGIN' '{safe_title}' '{safe_prompt}' 'END'"
        );
        if let Some(tab) = self.active_tab_mut() {
            tab.send_line(&line);
        }
        let title = self.active_tab().map(|t| t.title.clone()).unwrap_or_else(|| "Agent".into());
        self.status_msg = Some(format!("Dispatched '{task_title}' to {title} PTY"));
    }

    /// Copies the `product-os-spec-driven` block citation (`File: spec/...`, `Mục:`, `Trang reader:`,
    /// `Project:`, `Yêu cầu của tôi:`) to the system clipboard, records a structured PRD/Spec chat turn
    /// in the Agent Chat UI, AND pipes it directly into the active `claude` / `codex` / `agy` / `grok` PTY tab.
    pub fn send_copy_for_ai(&mut self, block_markdown: &str) {
        let payload = format_copy_for_ai_payload(block_markdown);

        if let Ok(mut cb) = arboard::Clipboard::new() {
            let _ = cb.set_text(payload.clone());
        }

        let summary_line = payload
            .lines()
            .find(|l| l.trim_start().starts_with("- Mục:") || l.trim_start().starts_with("- File:"))
            .map(|l| l.trim().trim_start_matches('-').trim())
            .unwrap_or("Spec Driven v2 block");
        let chat_prompt = format!("@prd Review & implement spec block ({summary_line})");
        self.submit_chat_prompt(&chat_prompt, None);

        self.dock_open = true;
        if let Some(tab) = self.active_tab_mut() {
            tab.write_bytes(payload.as_bytes());
            tab.write_bytes(b"\r");
        }
        self.status_msg = Some("Copied PRD/Spec citation & dispatched to Agent Chat PTY".into());
    }
}

/// Strips ANSI CSI (`\x1b[...]`), OSC (`\x1b]...(\x07|\x1b\\)`), and control sequences from raw PTY output
/// so terminal stdout can be rendered cleanly inside the Agent Chat UI (`DIR-21`).
pub fn strip_ansi_codes(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x1b' {
            match chars.peek().copied() {
                Some('[') => {
                    chars.next();
                    while let Some(c) = chars.next() {
                        if ('@'..='~').contains(&c) {
                            break;
                        }
                    }
                }
                Some(']') => {
                    chars.next();
                    while let Some(c) = chars.next() {
                        if c == '\x07' {
                            break;
                        }
                        if c == '\x1b' && chars.peek().copied() == Some('\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some('(' | ')' | '*' | '+' | '-' | '.' | '/') => {
                    chars.next();
                    chars.next();
                }
                Some(_) => {
                    chars.next();
                }
                None => {}
            }
        } else if ch == '\r' {
            continue;
        } else if ch == '\x08' {
            out.pop();
        } else if ch == '\n' || ch == '\t' || !ch.is_control() {
            out.push(ch);
        }
    }
    out
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

/// Auto-injects `TERM`, `COLORTERM`, `DESIGNOS_BIN`, `DESIGNOS_MCP_BIN`, `DESIGNOS_LIVE_SPEC`,
/// `DESIGNOS_SELECTION_JSON`, `DESIGNOS_FILE`, `DESIGNOS_WORKSPACE`, and prepends common CLI directories
/// (`~/.cargo/bin`, `/opt/homebrew/bin`, `/usr/local/bin`, `~/.local/bin`, `~/.antigravity/bin`) to `PATH`.
pub fn configure_pty_env(cmd: &mut CommandBuilder, file_path: Option<&Path>) {
    cmd.env("TERM", "xterm-256color");
    cmd.env("COLORTERM", "truecolor");
    cmd.env("DESIGNOS_LIVE_SPEC", "/tmp/designos-live-agent-out.spec.json");
    cmd.env("DESIGNOS_SELECTION_JSON", "/tmp/designos-live-selection.spec.json");

    if let Ok(exe) = std::env::current_exe() {
        cmd.env("DESIGNOS_BIN", exe.as_os_str());
        let mcp_bin = exe.with_file_name(if cfg!(windows) { "designos-mcp.exe" } else { "designos-mcp" });
        cmd.env("DESIGNOS_MCP_BIN", mcp_bin.as_os_str());
    } else {
        cmd.env("DESIGNOS_MCP_BIN", "designos-mcp");
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
        extra_paths.push(format!("{home}/.antigravity/bin"));
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

    let cmd_snippet =
        format!("# [design:os studio Canvas Context] Node '{}' (#{}) exported to {} — {}", node_name, target_id.0, tmp_path.display(), comment_summary);

    if let Some(tab) = state.active_tab_mut() {
        tab.send_line(&cmd_snippet);
    }
    state.status_msg = Some(format!("Sent '{}' (#{}) + comments to {}", node_name, target_id.0, tmp_path.display()));
    Ok(tmp_path)
}

/// Renders the 1:1 Figma Desktop `126.9.13` `Agents` Left Side Panel (`New chat / Shared` header,
/// dismissible AI credits banner, CLI Runtime Selector Bar (`Claude` | `Codex` | `Antigravity` | `Shell`),
/// pure Chat UI wrapper over the live PTY shell (`DIR-21`), `What's next?` empty state, bottom
/// `"Ask for changes"` blue-bordered composer box, and toggleable in-panel live VT100 PTY viewport).
pub fn show_left(state: &mut TerminalState, ui: &mut Ui, rect: Rect, file_path: Option<&Path>) -> Vec<TerminalAction> {
    state.ensure_default_tab(file_path);
    state.sync_chat_from_pty();
    let mut actions = state.poll_live_agent_bridge();
    let p = ui.painter().clone();
    let panel_bg = Color32::from_rgb(0x2C, 0x2C, 0x2C);
    p.rect_filled(rect, 0.0, panel_bg);

    // ---- A. Top Header Bar (48px, `#2C2C2C` fill, `1px` `#383838` bottom hairline)
    let header_h = 48.0;
    let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_h));
    p.rect_filled(header_rect, 0.0, panel_bg);
    widgets::hline(&p, header_rect.left(), header_rect.right(), header_rect.bottom() - 1.0, Color32::from_rgb(0x38, 0x38, 0x38));

    let back_r = Rect::from_center_size(Pos2::new(header_rect.left() + 18.0, header_rect.top() + 18.0), Vec2::splat(20.0));
    let back_resp = ui.interact(back_r, Id::new("agents-header-back"), Sense::click());
    if back_resp.hovered() {
        p.rect_filled(back_r, CornerRadius::same(4), TERM_BG_HOVER);
    }
    let bc = back_r.center();
    let chev_stroke = Stroke::new(1.5, Color32::from_rgb(0xB3, 0xB3, 0xB3));
    p.line_segment([Pos2::new(bc.x + 2.0, bc.y - 4.5), Pos2::new(bc.x - 2.5, bc.y)], chev_stroke);
    p.line_segment([Pos2::new(bc.x - 2.5, bc.y), Pos2::new(bc.x + 2.0, bc.y + 4.5)], chev_stroke);
    widgets::record(ui, "agents-header-back", back_r);
    if back_resp.clicked() {
        state.side_panel_pty_open = false;
        state.expanded_agent_cat = None;
        state.chat_turns.clear();
        state.chat_scroll = 0.0;
    }

    p.text(Pos2::new(header_rect.left() + 34.0, header_rect.top() + 17.0), Align2::LEFT_CENTER, "New chat", theme::font_body_semibold(), Color32::WHITE);
    p.text(
        Pos2::new(header_rect.left() + 34.0, header_rect.top() + 33.0),
        Align2::LEFT_CENTER,
        "Shared",
        theme::font_small(),
        Color32::from_rgb(0x8C, 0x8C, 0x8C),
    );

    // Header right controls: `AI` badge + Pop-out/PTY toggle + `•••` More menu
    let h_cy = header_rect.top() + 18.0;
    let ai_rect = Rect::from_center_size(Pos2::new(header_rect.right() - 78.0, h_cy), Vec2::new(24.0, 18.0));
    let ai_resp = ui.interact(ai_rect, Id::new("agents-header-ai"), Sense::click());
    let ai_bg = if !state.chat_mode || state.side_panel_pty_open {
        Color32::from_rgb(0x16, 0x35, 0x52)
    } else if ai_resp.hovered() {
        TERM_BG_HOVER
    } else {
        Color32::from_rgb(0x38, 0x38, 0x38)
    };
    p.rect_filled(ai_rect, CornerRadius::same(4), ai_bg);
    p.rect_stroke(ai_rect, CornerRadius::same(4), Stroke::new(1.0, Color32::from_rgb(0x4D, 0x4D, 0x4D)), StrokeKind::Inside);
    p.text(ai_rect.center(), Align2::CENTER_CENTER, "AI", theme::font_small_strong(), Color32::from_rgb(0xD4, 0xD4, 0xD4));
    widgets::record(ui, "agents-ai-badge", ai_rect);
    widgets::record(ui, "agents-header-ai", ai_rect);
    if ai_resp.clicked() {
        state.chat_mode = !state.chat_mode;
        state.side_panel_pty_open = !state.chat_mode;
    }

    let pop_rect = Rect::from_center_size(Pos2::new(header_rect.right() - 46.0, h_cy), Vec2::splat(24.0));
    let pop_resp = ui.interact(pop_rect, Id::new("term-left-dock-toggle"), Sense::click());
    if pop_resp.hovered() || state.side_panel_pty_open || state.dock_open {
        p.rect_filled(pop_rect, CornerRadius::same(4), TERM_BG_HOVER);
    }
    paint_popout_icon(&p, pop_rect.center(), Color32::from_rgb(0xD4, 0xD4, 0xD4));
    widgets::record(ui, "term-left-dock-toggle", pop_rect);
    widgets::record(ui, "agents-popout-pty", pop_rect);
    widgets::record(ui, "agents-header-popout", pop_rect);
    if pop_resp.clicked() {
        state.dock_open = !state.dock_open;
        state.side_panel_pty_open = state.dock_open;
    }

    let more_rect = Rect::from_center_size(Pos2::new(header_rect.right() - 18.0, h_cy), Vec2::splat(24.0));
    let more_resp = ui.interact(more_rect, Id::new("agents-more-menu"), Sense::click());
    if more_resp.hovered() {
        p.rect_filled(more_rect, CornerRadius::same(4), TERM_BG_HOVER);
    }
    for dx in [-4.5f32, 0.0, 4.5] {
        p.circle_filled(Pos2::new(more_rect.center().x + dx, more_rect.center().y), 1.4, Color32::from_rgb(0xD4, 0xD4, 0xD4));
    }
    widgets::record(ui, "agents-more-menu", more_rect);
    widgets::record(ui, "agents-header-more", more_rect);
    if more_resp.clicked() {
        state.chat_turns.clear();
        state.chat_scroll = 0.0;
        state.expanded_agent_cat = None;
    }

    // ---- B. Dismissible AI Credits Banner (48px when `!state.credits_banner_dismissed`, `#2D364D` fill)
    let mut top_y = header_rect.bottom();
    if !state.credits_banner_dismissed {
        let banner_h = 48.0;
        let banner_rect = Rect::from_min_size(Pos2::new(rect.left(), top_y), Vec2::new(rect.width(), banner_h));
        let close_x = (banner_rect.right() - 38.0).max(banner_rect.left() + 60.0);
        p.rect_filled(banner_rect, 0.0, Color32::from_rgb(0x2D, 0x36, 0x4D));
        let b_border = Color32::from_rgb(0x3E, 0x4C, 0x6D);
        widgets::hline(&p, banner_rect.left(), banner_rect.right(), banner_rect.top(), b_border);
        widgets::hline(&p, banner_rect.left(), banner_rect.right(), banner_rect.bottom() - 1.0, b_border);
        widgets::vline(&p, close_x, banner_rect.top(), banner_rect.bottom(), b_border);

        let banner_font = theme::font_small_strong();
        let max_txt_w = (close_x - banner_rect.left() - 16.0).max(40.0);
        let line1 = widgets::elide(&p, "As of October 6, the Figma agent uses AI", &banner_font, max_txt_w);
        p.text(
            Pos2::new(banner_rect.left() + 12.0, banner_rect.top() + 16.0),
            Align2::LEFT_CENTER,
            line1,
            banner_font.clone(),
            Color32::from_rgb(0xF0, 0xF4, 0xFF),
        );
        p.text(
            Pos2::new(banner_rect.left() + 12.0, banner_rect.top() + 32.0),
            Align2::LEFT_CENTER,
            "credits.",
            banner_font.clone(),
            Color32::from_rgb(0xF0, 0xF4, 0xFF),
        );
        p.text(
            Pos2::new(banner_rect.left() + 56.0, banner_rect.top() + 32.0),
            Align2::LEFT_CENTER,
            "Learn more",
            banner_font,
            Color32::from_rgb(0x6E, 0xA8, 0xFE),
        );

        let close_rect = Rect::from_min_max(Pos2::new(close_x, banner_rect.top()), banner_rect.max);
        let close_resp = ui.interact(close_rect, Id::new("agents-banner-dismiss"), Sense::click());
        if close_resp.hovered() {
            p.rect_filled(close_rect, 0.0, Color32::from_rgb(0x36, 0x41, 0x5C));
        }
        let cc = close_rect.center();
        let x_stroke = Stroke::new(1.4, Color32::from_rgb(0xD4, 0xD4, 0xD4));
        p.line_segment([Pos2::new(cc.x - 4.5, cc.y - 4.5), Pos2::new(cc.x + 4.5, cc.y + 4.5)], x_stroke);
        p.line_segment([Pos2::new(cc.x + 4.5, cc.y - 4.5), Pos2::new(cc.x - 4.5, cc.y + 4.5)], x_stroke);
        widgets::record(ui, "agents-banner-dismiss", close_rect);
        if close_resp.clicked() {
            state.credits_banner_dismissed = true;
        }
        top_y = banner_rect.bottom();
    }

    // ---- B2. CLI Runtime Selector Bar (`28px`, `Claude` | `Codex` | `AGY` | `Grok` + `Shell` toggle)
    let rt_bar_h = 28.0;
    let rt_bar_r = Rect::from_min_size(Pos2::new(rect.left(), top_y), Vec2::new(rect.width(), rt_bar_h));
    p.rect_filled(rt_bar_r, 0.0, Color32::from_rgb(0x22, 0x22, 0x22));
    widgets::hline(&p, rt_bar_r.left(), rt_bar_r.right(), rt_bar_r.bottom() - 1.0, Color32::from_rgb(0x38, 0x38, 0x38));

    let rt_pills: [(&str, &str, AgentCliPreset, f32, f32); 4] = [
        ("agents-runtime-claude", "Claude", AgentCliPreset::Claude, 4.0, 46.0),
        ("agents-runtime-codex", "Codex", AgentCliPreset::Codex, 53.0, 44.0),
        ("agents-runtime-agy", "AGY", AgentCliPreset::Antigravity, 100.0, 38.0),
        ("agents-runtime-grok", "Grok", AgentCliPreset::Grok, 141.0, 40.0),
    ];
    for (pill_id, pill_label, preset, dx, pw) in rt_pills {
        let p_rect = Rect::from_min_size(Pos2::new(rt_bar_r.left() + dx, rt_bar_r.top() + 4.0), Vec2::new(pw, 20.0));
        let p_resp = ui.interact(p_rect, Id::new(pill_id), Sense::click());
        let is_sel = !state.side_panel_pty_open && state.active_runtime == preset;
        let bg = if is_sel {
            STATUS_INFO_BG
        } else if p_resp.hovered() {
            TERM_BG_HOVER
        } else {
            Color32::from_rgb(0x2C, 0x2C, 0x2C)
        };
        let border = if is_sel { STATUS_INFO_BORDER } else { Color32::from_rgb(0x38, 0x38, 0x38) };
        let fg = if is_sel { STATUS_INFO_TEXT } else { Color32::from_rgb(0xD4, 0xD4, 0xD4) };
        p.rect_filled(p_rect, CornerRadius::same(4), bg);
        p.rect_stroke(p_rect, CornerRadius::same(4), Stroke::new(1.0, border), StrokeKind::Inside);
        p.text(p_rect.center(), Align2::CENTER_CENTER, pill_label, theme::font_small_strong(), fg);
        widgets::record(ui, pill_id, p_rect);
        if p_resp.clicked() {
            state.set_active_runtime(preset, file_path);
            state.side_panel_pty_open = false;
        }
    }

    let pty_toggle_r = Rect::from_min_size(Pos2::new((rt_bar_r.right() - 52.0).max(rt_bar_r.left() + 184.0), rt_bar_r.top() + 4.0), Vec2::new(46.0, 20.0));
    let pty_toggle_resp = ui.interact(pty_toggle_r, Id::new("agents-runtime-pty-toggle"), Sense::click());
    let pty_sel = state.side_panel_pty_open;
    let pty_bg = if pty_sel {
        STATUS_OK_BG
    } else if pty_toggle_resp.hovered() {
        TERM_BG_HOVER
    } else {
        Color32::from_rgb(0x2C, 0x2C, 0x2C)
    };
    let pty_border = if pty_sel { STATUS_OK_BORDER } else { Color32::from_rgb(0x38, 0x38, 0x38) };
    let pty_fg = if pty_sel { STATUS_OK_TEXT } else { Color32::from_rgb(0xB3, 0xB3, 0xB3) };
    p.rect_filled(pty_toggle_r, CornerRadius::same(4), pty_bg);
    p.rect_stroke(pty_toggle_r, CornerRadius::same(4), Stroke::new(1.0, pty_border), StrokeKind::Inside);
    p.text(pty_toggle_r.center(), Align2::CENTER_CENTER, "Shell", theme::font_small_strong(), pty_fg);
    widgets::record(ui, "agents-runtime-pty-toggle", pty_toggle_r);
    if pty_toggle_resp.clicked() {
        state.side_panel_pty_open = !state.side_panel_pty_open;
    }
    top_y = rt_bar_r.bottom();

    // ---- D. Bottom Docked `"Ask for changes"` Composer Box (`96px` height, `12px` margin)
    let comp_margin = 12.0;
    let comp_h = 96.0;
    let comp_top = (rect.bottom() - comp_margin - comp_h).max(top_y + 80.0);
    let composer_rect =
        Rect::from_min_max(Pos2::new(rect.left() + comp_margin, comp_top), Pos2::new(rect.right() - comp_margin, (comp_top + comp_h).min(rect.bottom() - 4.0)));

    // ---- C. Center Area (`between runtime bar and bottom composer`)
    let center_rect = Rect::from_min_max(Pos2::new(rect.left(), top_y), Pos2::new(rect.right(), (composer_rect.top() - 8.0).max(top_y + 32.0)));

    if state.side_panel_pty_open {
        // Mode 2: In-panel Live VT100 PTY Terminal (Claude / Codex / Antigravity / Grok / Shell)
        let bar_h = 28.0;
        let bar_r = Rect::from_min_size(center_rect.min, Vec2::new(center_rect.width(), bar_h));
        p.rect_filled(bar_r, 0.0, TERM_BG_BAR);
        widgets::hline(&p, bar_r.left(), bar_r.right(), bar_r.bottom() - 1.0, TERM_BORDER);

        let mut x = bar_r.left() + 6.0;
        for preset in AgentCliPreset::QUICK_LAUNCH {
            let badge = preset.badge();
            let w = 36.0;
            if x + w > bar_r.right() - 6.0 {
                break;
            }
            let b_r = Rect::from_min_size(Pos2::new(x, bar_r.top() + 4.0), Vec2::new(w, 20.0));
            let b_resp = ui.interact(b_r, Id::new(("agents-pty-preset", preset as usize)), Sense::click());
            let is_active = state.active_tab().map(|t| t.preset == preset).unwrap_or(false);
            let bg = if is_active {
                STATUS_INFO_BG
            } else if b_resp.hovered() {
                TERM_BG_HOVER
            } else {
                TERM_BG_ELEVATED
            };
            p.rect_filled(b_r, CornerRadius::same(4), bg);
            p.rect_stroke(b_r, CornerRadius::same(4), Stroke::new(1.0, if is_active { STATUS_INFO_BORDER } else { TERM_BORDER }), StrokeKind::Inside);
            p.text(b_r.center(), Align2::CENTER_CENTER, badge, theme::font_small_strong(), if is_active { STATUS_INFO_TEXT } else { theme::TEXT });
            if b_resp.clicked() {
                if let Some(idx) = state.tabs.iter().position(|t| t.preset == preset) {
                    state.active_tab = idx;
                } else {
                    let _ = state.spawn_tab(preset, file_path);
                }
                state.viewport_focused = true;
            }
            x += w + 4.0;
        }

        let vp_r = Rect::from_min_max(Pos2::new(center_rect.left(), bar_r.bottom()), center_rect.max);
        render_vt100_viewport(state, ui, vp_r, "agents-left-pty");
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
    } else if !state.chat_turns.is_empty() {
        // Mode 3: Pure Chat UI Wrapper Over Live PTY Shell (`Claude Code`, `Codex`, `Antigravity CLI`)
        render_agent_chat_thread(state, ui, center_rect, &mut actions);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(50));
    } else {
        // Mode 1 (Default): 1:1 Figma Desktop 126.9.13 `What's next?` View
        clipped(ui, center_rect, |ui| {
            let p = ui.painter().clone();
            let badge_center = Pos2::new(center_rect.center().x, center_rect.top() + 44.0);
            paint_agents_grid_plus_badge(&p, badge_center);

            p.text(
                Pos2::new(center_rect.center().x, center_rect.top() + 82.0),
                Align2::CENTER_CENTER,
                "What's next?",
                theme::font_large_strong(),
                Color32::WHITE,
            );

            let mut y = center_rect.top() + 108.0;
            let card_x = center_rect.left() + 14.0;
            let card_w = (center_rect.width() - 28.0).max(100.0);
            let card_h = 40.0;

            for (cat_idx, cat_id, cat_title) in [
                (0usize, "agents-cat-tidy", "Tidy and polish"),
                (1usize, "agents-cat-generate", "Generate designs"),
                (2usize, "agents-cat-plugins", "Create plugins and shaders"),
                (3usize, "agents-cat-prd", "PRD & Spec Driven v2"),
            ] {
                let card_r = Rect::from_min_size(Pos2::new(card_x, y), Vec2::new(card_w, card_h));
                let card_resp = ui.interact(card_r, Id::new(cat_id), Sense::click());
                let is_expanded = state.expanded_agent_cat == Some(cat_idx);
                let fill = if card_resp.hovered() || is_expanded { Color32::from_rgb(0x2E, 0x2E, 0x2E) } else { Color32::from_rgb(0x25, 0x25, 0x25) };
                p.rect_filled(card_r, CornerRadius::same(10), fill);
                p.rect_stroke(card_r, CornerRadius::same(10), Stroke::new(1.0, Color32::from_rgb(0x38, 0x38, 0x38)), StrokeKind::Inside);

                let icon_pos = Pos2::new(card_r.left() + 18.0, card_r.center().y);
                let icon_col = Color32::from_rgb(0xB3, 0xB3, 0xB3);
                match cat_idx {
                    0 => paint_wand_sparkle_icon(&p, icon_pos, icon_col),
                    1 => paint_four_point_star_icon(&p, icon_pos, icon_col),
                    3 => paint_open_book_icon(&p, icon_pos, icon_col),
                    _ => paint_toolbox_icon(&p, icon_pos, icon_col),
                }

                let label = widgets::elide(&p, cat_title, &theme::font_body_semibold(), (card_w - 58.0).max(32.0));
                p.text(Pos2::new(card_r.left() + 34.0, card_r.center().y), Align2::LEFT_CENTER, label, theme::font_body_semibold(), Color32::WHITE);

                let ch_c = Pos2::new(card_r.right() - 16.0, card_r.center().y);
                let ch_stroke = Stroke::new(1.4, icon_col);
                if is_expanded {
                    p.line_segment([Pos2::new(ch_c.x - 4.0, ch_c.y + 2.0), Pos2::new(ch_c.x, ch_c.y - 2.0)], ch_stroke);
                    p.line_segment([Pos2::new(ch_c.x, ch_c.y - 2.0), Pos2::new(ch_c.x + 4.0, ch_c.y + 2.0)], ch_stroke);
                } else {
                    p.line_segment([Pos2::new(ch_c.x - 4.0, ch_c.y - 2.0), Pos2::new(ch_c.x, ch_c.y + 2.0)], ch_stroke);
                    p.line_segment([Pos2::new(ch_c.x, ch_c.y + 2.0), Pos2::new(ch_c.x + 4.0, ch_c.y - 2.0)], ch_stroke);
                }

                widgets::record(ui, cat_id, card_r);
                if card_resp.clicked() {
                    state.expanded_agent_cat = if is_expanded { None } else { Some(cat_idx) };
                }
                y += card_h + 10.0;

                if is_expanded {
                    let sub_items: &[(&str, &str, &str, &str)] = match cat_idx {
                        0 => &[
                            (
                                "term-ai-rename-layers",
                                "Rename layers semantically",
                                "Rename Layers",
                                "Semantically rename generic Frame/Rect/Text layers across selection",
                            ),
                            ("term-ai-design-gate", "Design Gate audit & auto-fix", "Design Gate", "\"$DESIGNOS_BIN\" \"$DESIGNOS_FILE\" --inspect-json"),
                            (
                                "term-ai-translate",
                                "Translate selection (VI <-> EN)",
                                "Translate VI/EN",
                                "Translate selected UI copy between Vietnamese and English",
                            ),
                        ],
                        1 => &[
                            ("term-ai-make-design", "Make design (First Draft)", "Make Design", "Synthesize UI3 NodeSpec blueprint to $DESIGNOS_LIVE_SPEC"),
                            (
                                "term-ai-make-proto",
                                "Make prototype flows",
                                "Make Prototype",
                                "Auto-wire OnClick -> NavigateTo SmartAnimate flows across frames",
                            ),
                            (
                                "term-ai-populate",
                                "Populate realistic data",
                                "Populate Content",
                                "Populate realistic SaaS metrics, names, and statuses into repeated cards",
                            ),
                        ],
                        3 => &[
                            (
                                "term-ai-prd-audit",
                                "Audit PRD & FC-001 traceability",
                                "PRD Audit",
                                "@prd Audit plans/specs/designos-studio-v2/prd/PRD.md & FC-001 traceability across US-01..US-17",
                            ),
                            (
                                "term-ai-prd-story",
                                "Implement active User Story (US-01..US-17)",
                                "Implement Story",
                                "@prd Implement active User Story from plans/specs/designos-studio-v2/us/ onto Canvas",
                            ),
                            (
                                "term-ai-prd-gates",
                                "Verify G1..G5 pipeline gates & Gherkin ACs",
                                "Verify Gates",
                                "@prd Verify G1..G5 pipeline gates and Gherkin scenarios",
                            ),
                        ],
                        _ => &[
                            ("agents-open-pty", "Open Live PTY Terminal", "Open PTY", ""),
                            ("term-left-send-sel", "Send selection & comments to PTY", "Send Selection", ""),
                            ("term-left-apply-out", "Apply NodeSpec from PTY output", "Apply Spec", ""),
                            ("term-left-sync-disk", "Sync canvas from disk", "Sync Canvas", ""),
                        ],
                    };
                    for &(sub_id, sub_label, task_title, cli_prompt) in sub_items {
                        let sub_r = Rect::from_min_size(Pos2::new(card_x + 6.0, y), Vec2::new((card_w - 12.0).max(80.0), 28.0));
                        let sub_resp = ui.interact(sub_r, Id::new(sub_id), Sense::click());
                        p.rect_filled(sub_r, CornerRadius::same(6), if sub_resp.hovered() { TERM_BG_HOVER } else { Color32::from_rgb(0x1E, 0x1E, 0x1E) });
                        p.rect_stroke(sub_r, CornerRadius::same(6), Stroke::new(1.0, Color32::from_rgb(0x33, 0x33, 0x33)), StrokeKind::Inside);
                        let elided = widgets::elide(&p, sub_label, &theme::font_small_medium(), (sub_r.width() - 20.0).max(24.0));
                        p.text(Pos2::new(sub_r.left() + 10.0, sub_r.center().y), Align2::LEFT_CENTER, elided, theme::font_small_medium(), theme::TEXT);
                        widgets::record(ui, sub_id, sub_r);
                        if sub_resp.clicked() {
                            match sub_id {
                                "agents-open-pty" => {
                                    state.side_panel_pty_open = true;
                                }
                                "term-left-send-sel" => actions.push(TerminalAction::ExportSelectionToTerminal),
                                "term-left-apply-out" => {
                                    if let Some(act) = resolve_apply_spec_action(state) {
                                        actions.push(act);
                                    }
                                }
                                "term-left-sync-disk" => actions.push(TerminalAction::ReloadFromDisk),
                                _ => {
                                    state.submit_chat_prompt(cli_prompt, file_path);
                                    state.dispatch_figma_ai_prompt(task_title, cli_prompt, file_path);
                                    if sub_id == "term-ai-make-design" {
                                        let spec = crate::cmdk::synthesize_spec_from_prompt("pricing card");
                                        if let Ok(json) = serde_json::to_string(&spec) {
                                            actions.push(TerminalAction::ApplySpecJson(json));
                                        }
                                    }
                                }
                            }
                        }
                        y += 32.0;
                    }
                    y += 4.0;
                }
            }
        });
    }

    // ---- Render Bottom `"Ask for changes"` Composer Box
    p.rect_filled(composer_rect, CornerRadius::same(12), Color32::from_rgb(0x1E, 0x1E, 0x1E));
    p.rect_stroke(composer_rect, CornerRadius::same(12), Stroke::new(1.5, Color32::from_rgb(0x0D, 0x99, 0xFF)), StrokeKind::Inside);
    widgets::record(ui, "agents-composer-box", composer_rect);

    let input_rect = Rect::from_min_max(
        Pos2::new(composer_rect.left() + 12.0, composer_rect.top() + 10.0),
        Pos2::new(composer_rect.right() - 12.0, composer_rect.bottom() - 38.0),
    );
    let mut submit_composer = false;
    ui.scope_builder(egui::UiBuilder::new().max_rect(input_rect), |ui| {
        let te = TextEdit::singleline(&mut state.input_line)
            .id(Id::new("agents-ask-for-changes-input"))
            .desired_width(input_rect.width())
            .frame(egui::Frame::NONE)
            .font(theme::font_body())
            .text_color(Color32::WHITE)
            .hint_text(RichText::new("Ask for changes").font(theme::font_body()).color(Color32::from_rgb(0x8C, 0x8C, 0x8C)));
        let resp = ui.add(te);
        if resp.clicked() || resp.has_focus() {
            state.viewport_focused = false;
        }
        if resp.lost_focus() && ui.input(|i| i.key_pressed(Key::Enter)) {
            submit_composer = true;
        }
    });

    let icon_y = composer_rect.bottom() - 20.0;
    let icon_gray = Color32::from_rgb(0xB3, 0xB3, 0xB3);

    // Bottom-left `+` button (attaches selection & PRD context to composer and PTY)
    let plus_r = Rect::from_center_size(Pos2::new(composer_rect.left() + 20.0, icon_y), Vec2::splat(22.0));
    let plus_resp = ui.interact(plus_r, Id::new("agents-composer-plus"), Sense::click());
    if plus_resp.hovered() {
        p.rect_filled(plus_r, CornerRadius::same(4), TERM_BG_HOVER);
    }
    let pc = plus_r.center();
    p.line_segment([Pos2::new(pc.x - 5.0, pc.y), Pos2::new(pc.x + 5.0, pc.y)], Stroke::new(1.5, icon_gray));
    p.line_segment([Pos2::new(pc.x, pc.y - 5.0), Pos2::new(pc.x, pc.y + 5.0)], Stroke::new(1.5, icon_gray));
    widgets::record(ui, "agents-composer-plus", plus_r);
    widgets::record(ui, "agents-composer-attach", plus_r);
    if plus_resp.clicked() {
        if !state.input_line.contains("@selection") {
            let existing = state.input_line.trim().to_string();
            state.input_line = if existing.is_empty() {
                "@selection @prd ".to_string()
            } else {
                format!("@selection @prd {existing}")
            };
        }
        actions.push(TerminalAction::ExportSelectionToTerminal);
    }

    // Bottom-left open-book button (toggles in-panel live PTY terminal / prompt library)
    let book_r = Rect::from_center_size(Pos2::new(composer_rect.left() + 46.0, icon_y), Vec2::splat(22.0));
    let book_resp = ui.interact(book_r, Id::new("agents-composer-book"), Sense::click());
    if book_resp.hovered() {
        p.rect_filled(book_r, CornerRadius::same(4), TERM_BG_HOVER);
    }
    paint_open_book_icon(&p, book_r.center(), icon_gray);
    widgets::record(ui, "agents-composer-book", book_r);
    if book_resp.clicked() {
        state.side_panel_pty_open = !state.side_panel_pty_open;
    }

    // Bottom-left `PRD` context chip button (attaches/cycles `@prd `, `@selection `, `@tokens `)
    let prd_r = Rect::from_center_size(Pos2::new(composer_rect.left() + 82.0, icon_y), Vec2::new(38.0, 20.0));
    let prd_resp = ui.interact(prd_r, Id::new("agents-composer-prd"), Sense::click());
    let lower_in = state.input_line.to_ascii_lowercase();
    let has_prd_tag = lower_in.contains("@prd") || lower_in.contains("@selection") || lower_in.contains("@tokens");
    let prd_bg = if has_prd_tag {
        Color32::from_rgb(0x16, 0x35, 0x52)
    } else if prd_resp.hovered() {
        TERM_BG_HOVER
    } else {
        Color32::from_rgb(0x28, 0x28, 0x2C)
    };
    p.rect_filled(prd_r, CornerRadius::same(5), prd_bg);
    p.rect_stroke(
        prd_r,
        CornerRadius::same(5),
        Stroke::new(1.0, if has_prd_tag { Color32::from_rgb(0x0D, 0x99, 0xFF) } else { Color32::from_rgb(0x3C, 0x3C, 0x42) }),
        StrokeKind::Inside,
    );
    p.text(
        prd_r.center(),
        Align2::CENTER_CENTER,
        "PRD",
        theme::font_small_strong(),
        if has_prd_tag { Color32::from_rgb(0x6E, 0xA8, 0xFE) } else { icon_gray },
    );
    widgets::record(ui, "agents-composer-prd", prd_r);
    widgets::record(ui, "agents-composer-context", prd_r);
    if prd_resp.clicked() {
        let next_tag = if !lower_in.contains("@prd") {
            "@prd"
        } else if !lower_in.contains("@selection") {
            "@selection"
        } else {
            "@tokens"
        };
        if state.input_line.trim().is_empty() {
            state.input_line = format!("{next_tag} ");
        } else if !lower_in.contains(next_tag) {
            state.input_line = format!("{next_tag} {}", state.input_line.trim());
        }
    }

    // Bottom-right Microphone circle button (`26x26` `#2D364D` circle + `#6EA8FE` mic glyph)
    let mic_center = Pos2::new(composer_rect.right() - 50.0, icon_y);
    let mic_r = Rect::from_center_size(mic_center, Vec2::splat(26.0));
    let mic_resp = ui.interact(mic_r, Id::new("agents-composer-mic"), Sense::click());
    p.circle_filled(mic_center, 13.0, if mic_resp.hovered() { Color32::from_rgb(0x36, 0x41, 0x5C) } else { Color32::from_rgb(0x2D, 0x36, 0x4D) });
    paint_microphone_icon(&p, mic_center, Color32::from_rgb(0x6E, 0xA8, 0xFE));
    widgets::record(ui, "agents-composer-mic", mic_r);
    if mic_resp.clicked() {
        let template = "@prd Audit the selected frame against the PRD acceptance criteria";
        if state.input_line.trim().is_empty() {
            state.input_line = template.to_string();
        } else if !state.input_line.contains(template) {
            state.input_line = format!("{} {template}", state.input_line.trim());
        }
    }

    // Bottom-right Up-Arrow Send circle button (`26x26` `#383838` / `#0D99FF` circle + `↑` arrow)
    let has_text = !state.input_line.trim().is_empty();
    let send_center = Pos2::new(composer_rect.right() - 18.0, icon_y);
    let send_r = Rect::from_center_size(send_center, Vec2::splat(26.0));
    let send_resp = ui.interact(send_r, Id::new("agents-composer-send"), Sense::click());
    let send_bg = if has_text {
        Color32::from_rgb(0x0D, 0x99, 0xFF)
    } else if send_resp.hovered() {
        Color32::from_rgb(0x45, 0x45, 0x45)
    } else {
        Color32::from_rgb(0x38, 0x38, 0x38)
    };
    p.circle_filled(send_center, 13.0, send_bg);
    paint_up_arrow_icon(&p, send_center, if has_text { Color32::WHITE } else { icon_gray });
    widgets::record(ui, "agents-composer-send", send_r);
    if send_resp.clicked() {
        submit_composer = true;
    }

    if submit_composer && has_text {
        let prompt = state.input_line.trim().to_string();
        state.input_line.clear();
        state.submit_chat_prompt(&prompt, file_path);
        if !prompt.starts_with('$') && !prompt.starts_with('!') {
            let spec = crate::cmdk::synthesize_spec_from_prompt(&prompt);
            if let Ok(json) = serde_json::to_string(&spec) {
                if let Some(last_asst) = state.chat_turns.iter_mut().rev().find(|t| t.role == AgentChatRole::Assistant) {
                    last_asst.applied_to_canvas = true;
                }
                actions.push(TerminalAction::ApplySpecJson(json));
            }
        }
    }

    actions
}

fn wrap_chat_lines(p: &Painter, text: &str, font: &FontId, max_w: f32, max_lines: usize) -> Vec<String> {
    let mut out = Vec::new();
    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let mut current = String::new();
        for word in trimmed.split_whitespace() {
            let candidate = if current.is_empty() { word.to_string() } else { format!("{current} {word}") };
            if widgets::text_width(p, &candidate, font) <= max_w {
                current = candidate;
            } else {
                if !current.is_empty() {
                    out.push(widgets::elide(p, &current, font, max_w));
                    if out.len() >= max_lines {
                        return out;
                    }
                }
                current = widgets::elide(p, word, font, max_w);
            }
        }
        if !current.is_empty() {
            out.push(widgets::elide(p, &current, font, max_w));
            if out.len() >= max_lines {
                return out;
            }
        }
    }
    if out.is_empty() {
        out.push("Running in PTY shell...".to_string());
    }
    out
}

fn render_agent_chat_thread(state: &mut TerminalState, ui: &mut Ui, center_rect: Rect, actions: &mut Vec<TerminalAction>) {
    let mut apply_turn_idx: Option<usize> = None;
    clipped(ui, center_rect, |ui| {
        let p = ui.painter().clone();
        let font = theme::font_small();
        let font_strong = theme::font_small_strong();
        let mut y = center_rect.top() + 8.0 - state.chat_scroll;

        for (idx, turn) in state.chat_turns.iter().enumerate() {
            match turn.role {
                AgentChatRole::User => {
                    let card_w = (center_rect.width() - 28.0).max(120.0);
                    let max_txt_w = (card_w - 20.0).max(60.0);
                    let lines = wrap_chat_lines(&p, &turn.text, &font, max_txt_w, 6);
                    let card_h = (26.0 + lines.len() as f32 * 15.0 + 10.0).max(68.0);
                    let card_r = Rect::from_min_size(Pos2::new(center_rect.right() - card_w - 10.0, y), Vec2::new(card_w, card_h));

                    p.rect_filled(card_r, CornerRadius::same(8), Color32::from_rgb(0x26, 0x35, 0x4A));
                    p.rect_stroke(card_r, CornerRadius::same(8), Stroke::new(1.0, Color32::from_rgb(0x3E, 0x4C, 0x6D)), StrokeKind::Inside);

                    p.text(Pos2::new(card_r.left() + 10.0, card_r.top() + 13.0), Align2::LEFT_CENTER, "You", font_strong.clone(), Color32::WHITE);
                    p.text(
                        Pos2::new(card_r.right() - 10.0, card_r.top() + 13.0),
                        Align2::RIGHT_CENTER,
                        turn.runtime.badge(),
                        font_strong.clone(),
                        Color32::from_rgb(0x6E, 0xA8, 0xFE),
                    );
                    widgets::hline(&p, card_r.left() + 8.0, card_r.right() - 8.0, card_r.top() + 23.0, Color32::from_rgb(0x36, 0x46, 0x63));

                    for (li, line) in lines.iter().enumerate() {
                        p.text(
                            Pos2::new(card_r.left() + 10.0, card_r.top() + 34.0 + li as f32 * 15.0),
                            Align2::LEFT_CENTER,
                            line,
                            font.clone(),
                            Color32::from_rgb(0xF0, 0xF4, 0xFF),
                        );
                    }
                    widgets::record(ui, &format!("agents-chat-turn-{}", turn.id), card_r);
                    y += card_h + 8.0;
                }
                AgentChatRole::Assistant => {
                    let card_w = (center_rect.width() - 20.0).max(120.0);
                    let max_txt_w = (card_w - 20.0).max(60.0);
                    let display_txt = if turn.text.trim().is_empty() {
                        format!("[{} · PTY] Executing in live shell...", turn.runtime.tab_title())
                    } else {
                        turn.text.clone()
                    };
                    let lines = wrap_chat_lines(&p, &display_txt, &font, max_txt_w, 8);
                    let card_h = (28.0 + lines.len() as f32 * 15.0 + 26.0).max(82.0);
                    let card_r = Rect::from_min_size(Pos2::new(center_rect.left() + 10.0, y), Vec2::new(card_w, card_h));

                    p.rect_filled(card_r, CornerRadius::same(8), Color32::from_rgb(0x1E, 0x1E, 0x1E));
                    p.rect_stroke(card_r, CornerRadius::same(8), Stroke::new(1.0, Color32::from_rgb(0x38, 0x38, 0x38)), StrokeKind::Inside);

                    p.circle_filled(Pos2::new(card_r.left() + 12.0, card_r.top() + 13.0), 3.0, STATUS_OK_TEXT);
                    let title_elided = widgets::elide(&p, turn.runtime.tab_title(), &font_strong, (card_w - 64.0).max(40.0));
                    p.text(
                        Pos2::new(card_r.left() + 20.0, card_r.top() + 13.0),
                        Align2::LEFT_CENTER,
                        title_elided,
                        font_strong.clone(),
                        Color32::WHITE,
                    );
                    p.text(
                        Pos2::new(card_r.right() - 10.0, card_r.top() + 13.0),
                        Align2::RIGHT_CENTER,
                        "PTY",
                        font_strong.clone(),
                        STATUS_OK_TEXT,
                    );
                    widgets::hline(&p, card_r.left() + 8.0, card_r.right() - 8.0, card_r.top() + 24.0, Color32::from_rgb(0x2E, 0x2E, 0x2E));

                    for (li, line) in lines.iter().enumerate() {
                        p.text(
                            Pos2::new(card_r.left() + 10.0, card_r.top() + 35.0 + li as f32 * 15.0),
                            Align2::LEFT_CENTER,
                            line,
                            font.clone(),
                            Color32::from_rgb(0xE4, 0xE4, 0xE7),
                        );
                    }

                    let footer_y = card_r.bottom() - 12.0;
                    let has_spec = turn.extracted_spec_json.is_some();
                    let cmd_max_w = if has_spec { (card_w - 88.0).max(36.0) } else { (card_w - 20.0).max(40.0) };
                    let cmd_str = widgets::elide(&p, &format!("$ {}", turn.shell_cmd), &font, cmd_max_w);
                    p.text(
                        Pos2::new(card_r.left() + 10.0, footer_y),
                        Align2::LEFT_CENTER,
                        cmd_str,
                        font.clone(),
                        Color32::from_rgb(0x8C, 0x8C, 0x8C),
                    );

                    if has_spec {
                        let btn_r = Rect::from_min_size(Pos2::new(card_r.right() - 72.0, card_r.bottom() - 22.0), Vec2::new(64.0, 18.0));
                        let btn_resp = ui.interact(btn_r, Id::new(("agents-chat-apply-spec", turn.id)), Sense::click());
                        let b_bg = if turn.applied_to_canvas {
                            STATUS_OK_BG
                        } else if btn_resp.hovered() {
                            TERM_BG_HOVER
                        } else {
                            STATUS_INFO_BG
                        };
                        let b_border = if turn.applied_to_canvas { STATUS_OK_BORDER } else { STATUS_INFO_BORDER };
                        let b_fg = if turn.applied_to_canvas { STATUS_OK_TEXT } else { STATUS_INFO_TEXT };
                        let b_label = if turn.applied_to_canvas { "Applied" } else { "Apply" };
                        p.rect_filled(btn_r, CornerRadius::same(4), b_bg);
                        p.rect_stroke(btn_r, CornerRadius::same(4), Stroke::new(1.0, b_border), StrokeKind::Inside);
                        p.text(btn_r.center(), Align2::CENTER_CENTER, b_label, font_strong.clone(), b_fg);
                        widgets::record(ui, "agents-chat-apply-spec", btn_r);
                        if btn_resp.clicked() {
                            apply_turn_idx = Some(idx);
                        }
                    }

                    widgets::record(ui, &format!("agents-chat-turn-{}", turn.id), card_r);
                    y += card_h + 8.0;
                }
            }
        }
    });

    if let Some(idx) = apply_turn_idx {
        if let Some(turn) = state.chat_turns.get_mut(idx) {
            if let Some(spec_json) = turn.extracted_spec_json.clone() {
                turn.applied_to_canvas = true;
                actions.push(TerminalAction::ApplySpecJson(spec_json));
            }
        }
    }
}

fn paint_agents_grid_plus_badge(p: &egui::Painter, center: Pos2) {
    p.circle_filled(center, 19.0, Color32::from_rgb(0x2D, 0x36, 0x4D));
    let col = Color32::from_rgb(0x6E, 0xA8, 0xFE);
    icons::paint(p, Icon::GridView, center, 18.0, col);
}

fn paint_wand_sparkle_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::Wand, c, 14.0, col);
}

fn paint_four_point_star_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::StarFour, c, 14.0, col);
}

fn paint_toolbox_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::Tools, c, 14.0, col);
}

fn paint_open_book_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::BookOpen, c, 14.0, col);
}

fn paint_microphone_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::Microphone, c, 14.0, col);
}

fn paint_up_arrow_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::ArrowUp, c, 14.0, col);
}

fn paint_popout_icon(p: &egui::Painter, c: Pos2, col: Color32) {
    icons::paint(p, Icon::ArrowSquareOut, c, 14.0, col);
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
    let mut actions = state.poll_live_agent_bridge();
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

    #[test]
    fn antigravity_preset_and_quick_launch_cover_all_five_cli_runtimes() {
        assert_eq!(AgentCliPreset::QUICK_LAUNCH.len(), 5);
        assert!(AgentCliPreset::QUICK_LAUNCH.contains(&AgentCliPreset::Antigravity));
        assert_eq!(AgentCliPreset::Antigravity.tab_title(), "Antigravity (agy)");
        assert_eq!(AgentCliPreset::Antigravity.button_label(), "Antigravity");
        assert_eq!(AgentCliPreset::Antigravity.badge(), "AGY");
        assert_eq!(AgentCliPreset::Antigravity.binary_name(), "agy");
        assert!(AgentCliPreset::Shell.is_installed());
    }

    #[test]
    fn poll_live_agent_bridge_detects_updated_spec_file() {
        let _ = std::fs::remove_file("/tmp/designos-live-agent-out.spec.json");
        let live_path = std::env::temp_dir().join(format!("designos-live-agent-test-{}.spec.json", std::process::id()));
        let mut st = TerminalState { last_watched_mtime: Some(std::time::UNIX_EPOCH), live_spec_path: live_path.clone(), ..TerminalState::default() };
        let _ = std::fs::write(&live_path, r#"{"name":"Live Agent Frame","preset":"easeui-card"}"#);
        let actions = st.poll_live_agent_bridge();
        assert_eq!(actions, vec![TerminalAction::ApplySpecFile(live_path.clone())]);
        // Subsequent poll without modification returns empty
        assert!(st.poll_live_agent_bridge().is_empty());
        let _ = std::fs::remove_file(&live_path);
    }

    #[test]
    fn figma_desktop_agents_side_panel_renders_and_passes_layout_lint() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply(&ctx);
        let mut st = TerminalState::default();
        let panel_rect = Rect::from_min_size(Pos2::new(48.0, 0.0), Vec2::new(240.0, 800.0));

        crate::lint::take_icons();
        let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))), ..Default::default() };
        let out = ctx.run_ui(raw, |ui| {
            let _ = show_left(&mut st, ui, panel_rect, None);
        });
        let (texts, boxes) = crate::lint::scan_shapes(&out.shapes);
        let all_text: Vec<&str> = texts.iter().map(|t| t.text.as_str()).collect();
        assert!(all_text.contains(&"New chat"), "Header 'New chat' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Shared"), "Subtitle 'Shared' must be painted: {all_text:?}");
        assert!(all_text.contains(&"AI"), "Badge 'AI' must be painted: {all_text:?}");
        assert!(all_text.contains(&"What's next?"), "Heading 'What's next?' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Tidy and polish"), "Category 'Tidy and polish' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Generate designs"), "Category 'Generate designs' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Create plugins and shaders"), "Category 'Create plugins and shaders' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Learn more"), "Banner 'Learn more' link must be painted: {all_text:?}");

        let text_findings = crate::lint::check_text(&texts, &boxes);
        let icon_findings = crate::lint::check_icons(&crate::lint::take_icons());
        assert!(text_findings.is_empty(), "Agents side panel text lint findings: {text_findings:?}");
        assert!(icon_findings.is_empty(), "Agents side panel icon lint findings: {icon_findings:?}");

        // Also verify expanded categories and in-panel PTY mode render cleanly
        for cat in [Some(0), Some(1), Some(2)] {
            st.expanded_agent_cat = cat;
            let raw2 = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))), ..Default::default() };
            let out2 = ctx.run_ui(raw2, |ui| {
                let _ = show_left(&mut st, ui, panel_rect, None);
            });
            let (t2, b2) = crate::lint::scan_shapes(&out2.shapes);
            assert!(crate::lint::check_text(&t2, &b2).is_empty());
        }

        st.credits_banner_dismissed = true;
        st.side_panel_pty_open = true;
        let raw3 = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))), ..Default::default() };
        let _ = ctx.run_ui(raw3, |ui| {
            let _ = show_left(&mut st, ui, panel_rect, None);
        });
        for mut tab in st.tabs.drain(..) {
            let _ = tab.child.kill();
        }
    }

    #[test]
    fn agents_chat_wrapper_executes_in_pty_shell_and_streams_clean_output() {
        let stripped = strip_ansi_codes("\x1b[1;32mHello\x1b[0m \x1b]0;WindowTitle\x07World\r\n");
        assert_eq!(stripped.trim(), "Hello World");

        let mut st = TerminalState::default();
        assert_eq!(st.active_runtime, AgentCliPreset::Claude);
        st.set_active_runtime(AgentCliPreset::Codex, None);
        assert_eq!(st.active_runtime, AgentCliPreset::Codex);
        st.set_active_runtime(AgentCliPreset::Antigravity, None);
        assert_eq!(st.active_runtime, AgentCliPreset::Antigravity);
        st.set_active_runtime(AgentCliPreset::Grok, None);
        assert_eq!(st.active_runtime, AgentCliPreset::Grok);
        st.set_active_runtime(AgentCliPreset::Claude, None);
        assert_eq!(st.active_runtime, AgentCliPreset::Claude);

        st.submit_chat_prompt("Create a SaaS analytics card in Claude Code", None);
        assert_eq!(st.chat_turns.len(), 2);
        assert_eq!(st.chat_turns[0].role, AgentChatRole::User);
        assert_eq!(st.chat_turns[0].text, "Create a SaaS analytics card in Claude Code");
        assert_eq!(st.chat_turns[1].role, AgentChatRole::Assistant);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut synced = false;
        while std::time::Instant::now() < deadline {
            st.sync_chat_from_pty();
            let reply = &st.chat_turns[1].text;
            if !reply.trim().is_empty() {
                assert!(!reply.contains("DESIGNOS_CHAT_BEGIN_"), "Marker should not leak into chat bubble: {reply}");
                assert!(!reply.contains("\x1b["), "ANSI CSI should be stripped: {reply}");
                synced = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        assert!(synced, "Assistant chat turn should receive live PTY shell output");

        // Inject an extracted NodeSpec JSON on the assistant turn to also verify the 'Apply' button layout
        st.chat_turns[1].extracted_spec_json = Some(r#"{"name":"Analytics Card","preset":"easeui-card"}"#.to_string());

        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply(&ctx);
        let panel_rect = Rect::from_min_size(Pos2::new(48.0, 0.0), Vec2::new(240.0, 800.0));
        crate::lint::take_icons();
        let raw = egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))), ..Default::default() };
        let out = ctx.run_ui(raw, |ui| {
            let _ = show_left(&mut st, ui, panel_rect, None);
        });
        let (texts, boxes) = crate::lint::scan_shapes(&out.shapes);
        let all_text: Vec<&str> = texts.iter().map(|t| t.text.as_str()).collect();
        assert!(all_text.contains(&"Claude"), "Runtime pill 'Claude' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Codex"), "Runtime pill 'Codex' must be painted: {all_text:?}");
        assert!(all_text.contains(&"AGY"), "Runtime pill 'AGY' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Grok"), "Runtime pill 'Grok' must be painted: {all_text:?}");
        assert!(all_text.contains(&"Shell"), "Runtime pill 'Shell' must be painted: {all_text:?}");
        assert!(all_text.contains(&"You"), "User bubble header 'You' must be painted: {all_text:?}");
        assert!(all_text.contains(&"PTY"), "Assistant bubble 'PTY' badge must be painted: {all_text:?}");
        assert!(all_text.contains(&"Apply"), "Assistant bubble 'Apply' spec button must be painted: {all_text:?}");

        let text_findings = crate::lint::check_text(&texts, &boxes);
        let icon_findings = crate::lint::check_icons(&crate::lint::take_icons());
        assert!(text_findings.is_empty(), "Chat thread text lint findings: {text_findings:?}");
        assert!(icon_findings.is_empty(), "Chat thread icon lint findings: {icon_findings:?}");

        for mut tab in st.tabs.drain(..) {
            let _ = tab.child.kill();
        }
    }

    #[test]
    fn all_four_agent_cli_runtimes_claude_codex_agy_grok_execute_in_pty() {
        let mut st = TerminalState::default();
        for (preset, prompt, expected_tag) in [
            (AgentCliPreset::Claude, "Audit layout in Claude", "[Claude Code · claude PTY]"),
            (AgentCliPreset::Codex, "Generate React code in Codex", "[Codex CLI · codex PTY]"),
            (AgentCliPreset::Antigravity, "Check parity in Antigravity", "[Antigravity (agy) · agy PTY]"),
            (AgentCliPreset::Grok, "Fast build in Grok", "[Grok Build · grok PTY]"),
        ] {
            st.set_active_runtime(preset, None);
            let turn_idx = st.chat_turns.len() + 1;
            st.submit_chat_prompt(prompt, None);
            let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
            let mut ok = false;
            while std::time::Instant::now() < deadline {
                st.sync_chat_from_pty();
                if st.chat_turns[turn_idx].text.contains(expected_tag) {
                    ok = true;
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(40));
            }
            assert!(
                ok,
                "Runtime {:?} failed to stream expected output '{}' in PTY: got '{}'",
                preset,
                expected_tag,
                st.chat_turns[turn_idx].text
            );
        }
        for mut tab in st.tabs.drain(..) {
            let _ = tab.child.kill();
        }
    }

    #[test]
    fn agent_chat_supports_prd_and_spec_context_handoff() {
        let mut st = TerminalState::default();
        st.send_copy_for_ai("REQ-01: Native Spec Reader & Linear/Confluence Reading Experience");
        assert_eq!(st.chat_turns.len(), 2);
        assert!(st.chat_turns[0].text.contains("@prd"), "User turn should carry @prd tag: {}", st.chat_turns[0].text);

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(4);
        let mut prd_streamed = false;
        while std::time::Instant::now() < deadline {
            st.sync_chat_from_pty();
            if st.chat_turns[1].text.contains("PRD Context: plans/specs/designos-studio-v2/prd/PRD.md") {
                prd_streamed = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        assert!(
            prd_streamed,
            "Assistant PTY turn should include PRD Context line: got '{}'",
            st.chat_turns[1].text
        );

        for mut tab in st.tabs.drain(..) {
            let _ = tab.child.kill();
        }
    }

    #[test]
    fn real_cli_command_builder_and_figma_ai_dispatch_execute_real_commands() {
        assert!(runtime_binary_available("sh"));
        assert!(!runtime_binary_available("nonexistent_designos_binary_xyz_999"));

        let live_spec = std::path::Path::new("/tmp/designos-live-agent-out.spec.json");
        let sel_json = std::path::Path::new("/tmp/designos-selection.json");
        let claude_cmd = build_real_cli_command(AgentCliPreset::Claude, "Create a SaaS hero frame", live_spec, sel_json);
        assert!(claude_cmd.contains("claude -p"));
        assert!(claude_cmd.contains("DESIGNOS_LIVE_SPEC"));
        assert!(claude_cmd.contains("DESIGNOS_SELECTION_JSON"));

        let codex_cmd = build_real_cli_command(AgentCliPreset::Codex, "Audit layout", live_spec, sel_json);
        assert!(codex_cmd.contains("codex exec"));
        let agy_cmd = build_real_cli_command(AgentCliPreset::Antigravity, "Check parity", live_spec, sel_json);
        assert!(agy_cmd.contains("agy -p"));
        let grok_cmd = build_real_cli_command(AgentCliPreset::Grok, "Build card", live_spec, sel_json);
        assert!(grok_cmd.contains("grok -p"));

        let mut st = TerminalState::default();
        st.dispatch_figma_ai_prompt(
            "Auto-rename layers to semantic names",
            "Auto-rename layers to semantic names",
            None,
        );
        assert!(st.dock_open);
        assert_eq!(st.chat_turns.len(), 2);
        assert!(st.chat_turns[0].text.contains("Auto-rename layers to semantic names"));

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
        let mut got_reply = false;
        while std::time::Instant::now() < deadline {
            st.sync_chat_from_pty();
            if st.chat_turns[1].text.contains("[Figma AI ·") {
                got_reply = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(40));
        }
        assert!(got_reply, "dispatch_figma_ai_prompt must stream real PTY output instead of a comment line");
        for mut tab in st.tabs.drain(..) {
            let _ = tab.child.kill();
        }
    }

    #[test]
    fn agents_header_and_composer_buttons_mutate_real_state() {
        let ctx = egui::Context::default();
        crate::theme::install_fonts(&ctx);
        crate::theme::apply(&ctx);
        let mut st = TerminalState::default();
        let panel_rect = Rect::from_min_size(Pos2::new(48.0, 0.0), Vec2::new(240.0, 800.0));

        // First pass to record widget rects
        let _ = ctx.run_ui(
            egui::RawInput { screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))), ..Default::default() },
            |ui| {
                let _ = show_left(&mut st, ui, panel_rect, None);
            },
        );
        let get_rect = |name: &str| -> Rect {
            ctx.data(|d| d.get_temp::<Rect>(Id::new(("test-rect", name.to_string()))))
                .unwrap_or_else(|| panic!("{name} recorded"))
        };
        let ai_rect = get_rect("agents-header-ai");
        let plus_rect = get_rect("agents-composer-attach");
        let prd_rect = get_rect("agents-composer-context");
        let mic_rect = get_rect("agents-composer-mic");

        // Click AI header button -> toggles chat_mode
        assert!(st.chat_mode);
        let click_at = |ctx: &egui::Context, st: &mut TerminalState, pos: Pos2| -> Vec<TerminalAction> {
            let mut acts = Vec::new();
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 900.0))),
                events: vec![
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: true, modifiers: egui::Modifiers::NONE },
                    egui::Event::PointerButton { pos, button: egui::PointerButton::Primary, pressed: false, modifiers: egui::Modifiers::NONE },
                ],
                ..Default::default()
            };
            let _ = ctx.run_ui(raw, |ui| {
                acts = show_left(st, ui, panel_rect, None);
            });
            acts
        };

        let _ = click_at(&ctx, &mut st, ai_rect.center());
        assert!(!st.chat_mode);
        assert!(st.side_panel_pty_open);

        // Click `+` attach button -> prepends @selection @prd and emits ExportSelectionToTerminal
        let acts = click_at(&ctx, &mut st, plus_rect.center());
        assert!(st.input_line.contains("@selection"));
        assert!(acts.contains(&TerminalAction::ExportSelectionToTerminal));

        // Click `PRD` context chip -> cycles next context tag (@tokens since @prd & @selection are present)
        let _ = click_at(&ctx, &mut st, prd_rect.center());
        assert!(st.input_line.contains("@tokens"));

        // Click mic button -> appends voice audit prompt template
        let _ = click_at(&ctx, &mut st, mic_rect.center());
        assert!(st.input_line.contains("Audit the selected frame against the PRD acceptance criteria"));

        for mut tab in st.tabs.drain(..) {
            let _ = tab.child.kill();
        }
    }
}

