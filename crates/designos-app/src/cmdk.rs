//! In-Canvas `Cmd+K` Agent Command Bar, Prompt-to-`NodeSpec` Synthesizer,
//! and Live Design Gate (`vsf-reviewer`) Audit Engine.
//!
//! Crafted to `es:designer` (`craft-floor.md` + `tokens.css`) and **EaseUI BrainWave 2.0**:
//! - Zero emoji icons (`§2 Refuse by default`): uses crisp Lucide vector glyphs (`crate::icons::Icon`)
//! - No cards-in-cards (`§2`): uses 1px hairlines (`#2A2B31`), spacing steps (`4/8/12/16/24px`),
//!   and surface steps (`#16171B` -> `#1F2024` -> `#26272C`)
//! - Concentric radii (`12px` outer dialog, `6px` inner rows and controls)
//! - Semantic status pills (`OK #4CC58B`, `WARN #E0A53C`, `BAD #FF7A8C`, `INFO #8FB0FF`)

use crate::icons::{self, Icon};
use crate::terminal::{
    AgentCliPreset, STATUS_BAD_BG, STATUS_BAD_BORDER, STATUS_BAD_TEXT, STATUS_INFO_BG, STATUS_INFO_BORDER, STATUS_INFO_TEXT, STATUS_OK_BG, STATUS_OK_BORDER,
    STATUS_OK_TEXT, STATUS_WARN_BG, STATUS_WARN_BORDER, STATUS_WARN_TEXT, TERM_BG_ELEVATED, TERM_BG_HOVER, TERM_BG_VIEWPORT, TERM_BORDER, TERM_ORANGE,
};
use crate::theme;
use crate::widgets;
use designos_doc::{Document, LayoutSpec, NodeId, NodeKind, NodeSpec, PaintKind};
use egui::{Align2, Area, Color32, Context, CornerRadius, Id, Key, Order, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Vec2};

/// Persistent UI state for the floating `Cmd+K` Agent & Design Gate Bar.
#[derive(Clone, Debug, Default)]
pub struct CmdKState {
    pub open: bool,
    pub query: String,
    pub selected_idx: usize,
    pub active_tab: usize,
    pub last_audit: Option<DesignGateReport>,
    pub status: Option<String>,
}

/// Summary report produced by `run_design_gate_audit`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignGateReport {
    pub score: u32,
    pub passed: usize,
    pub warnings: Vec<DesignGateIssue>,
    pub node_count: usize,
}

/// A single Design Gate violation tied to a concrete `NodeId` on the canvas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignGateIssue {
    pub node_id: NodeId,
    pub node_name: String,
    pub rule: &'static str,
    pub detail: String,
}

/// Actions returned by `cmdk::show` for the host `App` / `Shell` to execute.
#[derive(Clone, Debug, PartialEq)]
pub enum CmdKAction {
    ApplySpec(NodeSpec),
    OpenSpecReader,
    OpenTerminal(AgentCliPreset),
    SendSelectionToTerminal,
    ExportFig,
    SelectNode(NodeId),
    AutoFixDesignGate,
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
struct QuickCommandItem {
    id: &'static str,
    icon: Icon,
    title: &'static str,
    subtitle: &'static str,
    badge: &'static str,
    shortcut: &'static str,
    tab_mask: u8, // bit 0: All, bit 1: Assets, bit 2: Plugins & widgets, bit 3: AI & Gate
    keywords: &'static str,
}

const QUICK_COMMANDS: [QuickCommandItem; 6] = [
    QuickCommandItem {
        id: "cmdk-synth-spec",
        icon: Icon::Sparkle,
        title: "Generate EaseUI Screen / Card from Prompt…",
        subtitle: "Synthesizes an EaseUI BrainWave 2.0 NodeSpec subtree with Auto-Layout & $easeui.* tokens",
        badge: "SYNTH",
        shortcut: "↵",
        tab_mask: 0b1011,
        keywords: "generate create prompt card modal table pricing login hero sidebar console spec assets",
    },
    QuickCommandItem {
        id: "cmdk-design-gate",
        icon: Icon::Check,
        title: "Run Design Gate Audit (VSF / EaseUI Rules)",
        subtitle: "Audits 4px spacing rhythm, >=32px tap targets, token compliance, and >=11px typography",
        badge: "AUDIT",
        shortcut: "⌥⌘G",
        tab_mask: 0b1101,
        keywords: "audit gate review vsf lint spacing tap target token readability plugin widget",
    },
    QuickCommandItem {
        id: "cmdk-open-terminal",
        icon: Icon::Dev,
        title: "Open Agent Terminal (Claude / Codex / Shell)",
        subtitle: "Launches an embedded interactive PTY session with DESIGNOS_BIN & DESIGNOS_FILE injected",
        badge: "PTY",
        shortcut: "⌃`",
        tab_mask: 0b1101,
        keywords: "terminal pty claude codex grok shell cli agent plugin",
    },
    QuickCommandItem {
        id: "cmdk-send-selection",
        icon: Icon::Comment,
        title: "Send Selection & Pin-Comments to Agent Terminal",
        subtitle: "Exports selected frame to /tmp/designos-live-selection.spec.json and pipes into PTY stdin",
        badge: "BRIDGE",
        shortcut: "⇧⌘E",
        tab_mask: 0b1101,
        keywords: "send selection comments pin export terminal bridge",
    },
    QuickCommandItem {
        id: "cmdk-open-specs",
        icon: Icon::Specs,
        title: "Open Native Spec Reader (plans/specs/)",
        subtitle: "Inspects User Stories, Gherkin acceptance criteria, and NodeSpec blueprints",
        badge: "SPECS",
        shortcut: "⇧⌘S",
        tab_mask: 0b1011,
        keywords: "spec reader plans brainstorm usecases gherkin prd assets",
    },
    QuickCommandItem {
        id: "cmdk-export-fig",
        icon: Icon::Export,
        title: "Save Local Figma .fig Binary Copy",
        subtitle: "Writes a 1:1 round-trip Figma kiwi binary snapshot of the current document",
        badge: ".FIG",
        shortcut: "⇧⌘K",
        tab_mask: 0b0011,
        keywords: "save export fig figma binary file assets",
    },
];

const CMDK_BG_SURFACE: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
const CMDK_BG_ROW_HOVER: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const CMDK_BORDER: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const CMDK_TAB_ACTIVE: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);

/// Renders the bottom-docked Figma UI3 `Actions (Cmd+K)` modal when `state.open` is true.
pub fn show(ctx: &Context, state: &mut CmdKState, doc: &Document, page: NodeId, selection: &[NodeId]) -> Option<CmdKAction> {
    if !state.open {
        return None;
    }

    let mut out_action: Option<CmdKAction> = None;
    let screen = ctx.content_rect();
    let bar_w = 520.0f32.min((screen.width() - 32.0).max(320.0));

    let q_lower = state.query.trim().to_ascii_lowercase();
    let tab_bit = 1u8 << (state.active_tab.min(3));
    let filtered_cmds: Vec<(usize, QuickCommandItem)> = QUICK_COMMANDS
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, item)| {
            let tab_ok = (item.tab_mask & tab_bit) != 0;
            let query_ok =
                q_lower.is_empty() || item.title.to_ascii_lowercase().contains(&q_lower) || item.keywords.contains(&q_lower) || item.id == "cmdk-synth-spec";
            tab_ok && query_ok
        })
        .collect();

    if state.selected_idx >= filtered_cmds.len() && !filtered_cmds.is_empty() {
        state.selected_idx = 0;
    }

    let mut trigger_selected = false;
    let mut close_requested = false;
    ctx.input(|i| {
        if i.key_pressed(Key::Escape) {
            close_requested = true;
        }
        if i.key_pressed(Key::ArrowDown) && !filtered_cmds.is_empty() {
            state.selected_idx = (state.selected_idx + 1) % filtered_cmds.len();
        }
        if i.key_pressed(Key::ArrowUp) && !filtered_cmds.is_empty() {
            state.selected_idx = if state.selected_idx == 0 { filtered_cmds.len() - 1 } else { state.selected_idx - 1 };
        }
        if i.key_pressed(Key::Enter) {
            trigger_selected = true;
        }
    });

    if close_requested {
        state.open = false;
        return None;
    }

    let audit_rows = state.last_audit.as_ref().map(|a| a.warnings.len().min(4)).unwrap_or(0);
    let audit_h = if state.last_audit.is_some() { 48.0 + audit_rows as f32 * 26.0 } else { 0.0 };
    // Search row (44) + Tab bar (36) + Quick pills row (32) + Section header (24) + rows (36 each) + audit + footer (28)
    let bar_h = 44.0 + 36.0 + 32.0 + 24.0 + (filtered_cmds.len() as f32 * 36.0) + audit_h + 28.0;
    // Bottom-docked right above the bottom floating toolbar (`screen.bottom() - 68.0`)
    let bar_pos = Pos2::new((screen.center().x - bar_w / 2.0).max(16.0), (screen.bottom() - 68.0 - bar_h).max(20.0));

    Area::new(Id::new("designos-cmdk-agent-bar")).order(Order::Foreground).fixed_pos(bar_pos).show(ctx, |ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::new(bar_w, bar_h), Sense::click());
        let p = ui.painter().clone();

        // Outer modal: 12px radius + 1px `#383838` border + deep elevation shadow
        widgets::elevation_400(&p, rect, 12);
        p.rect_filled(rect, CornerRadius::same(12), CMDK_BG_SURFACE);
        p.rect_stroke(rect, CornerRadius::same(12), Stroke::new(1.0, CMDK_BORDER), StrokeKind::Inside);

        // 1. Top Search Row (44px) — matches `figma_ui3_actions_cmdk.png`
        let input_row = Rect::from_min_size(rect.min, Vec2::new(bar_w, 44.0));
        widgets::hline(&p, input_row.left(), input_row.right(), input_row.bottom() - 1.0, CMDK_BORDER);

        icons::paint(&p, Icon::Search, Pos2::new(input_row.left() + 20.0, input_row.center().y), 16.0, theme::ICON_SECONDARY);

        // Right-side terminal / synthesize controls
        let term_btn_r = Rect::from_center_size(Pos2::new(input_row.right() - 22.0, input_row.center().y), Vec2::splat(24.0));
        let term_resp = ui.interact(term_btn_r, Id::new("cmdk-term-icon-btn"), Sense::click());
        if term_resp.hovered() {
            p.rect_filled(term_btn_r, CornerRadius::same(6), CMDK_BG_ROW_HOVER);
        }
        icons::paint(&p, Icon::Dev, term_btn_r.center(), 14.0, theme::ICON_SECONDARY);
        if term_resp.clicked() {
            state.open = false;
            out_action = Some(CmdKAction::OpenTerminal(AgentCliPreset::Claude));
        }

        let gen_w = 88.0;
        let gen_rect = Rect::from_min_size(Pos2::new(term_btn_r.left() - gen_w - 6.0, input_row.center().y - 12.0), Vec2::new(gen_w, 24.0));
        let gen_resp = ui.interact(gen_rect, Id::new("cmdk-generate-btn"), Sense::click());
        p.rect_filled(gen_rect, CornerRadius::same(6), if gen_resp.hovered() { theme::BG_BRAND_HOVER } else { TERM_ORANGE });
        p.text(gen_rect.center(), Align2::CENTER_CENTER, "Synthesize", theme::font_small_strong(), Color32::WHITE);
        if gen_resp.clicked() {
            let prompt = if state.query.trim().is_empty() { "pricing card" } else { state.query.trim() };
            let spec = synthesize_spec_from_prompt(prompt);
            state.status = Some(format!("Synthesized '{}' on Canvas", spec.name));
            state.open = false;
            out_action = Some(CmdKAction::ApplySpec(spec));
        }

        let edit_rect =
            Rect::from_min_max(Pos2::new(input_row.left() + 36.0, input_row.top() + 8.0), Pos2::new(gen_rect.left() - 8.0, input_row.bottom() - 8.0));
        ui.scope_builder(egui::UiBuilder::new().max_rect(edit_rect), |ui| {
            let te = egui::TextEdit::singleline(&mut state.query)
                .frame(egui::Frame::NONE)
                .font(theme::font_body())
                .text_color(theme::TEXT)
                .hint_text("Search...")
                .desired_width(edit_rect.width());
            let resp = ui.add(te);
            if state.open && !resp.has_focus() {
                resp.request_focus();
            }
        });

        // 2. 4-Tab Bar (`All`, `Assets`, `Plugins & widgets`, `AI & Gate`) — 36px height
        let tab_row = Rect::from_min_size(Pos2::new(rect.left(), input_row.bottom()), Vec2::new(bar_w, 36.0));
        widgets::hline(&p, tab_row.left(), tab_row.right(), tab_row.bottom() - 1.0, CMDK_BORDER);
        let mut tx = tab_row.left() + 12.0;
        for (t_idx, t_label) in ["All", "Assets", "Plugins & widgets", "AI & Gate"].iter().enumerate() {
            let tw = widgets::text_width(&p, t_label, &theme::font_body()) + 20.0;
            let tr = Rect::from_min_size(Pos2::new(tx, tab_row.center().y - 12.0), Vec2::new(tw, 24.0));
            let tresp = ui.interact(tr, Id::new(("cmdk-tab", t_idx)), Sense::click());
            let is_active = state.active_tab == t_idx;
            if is_active {
                p.rect_filled(tr, CornerRadius::same(6), CMDK_TAB_ACTIVE);
            } else if tresp.hovered() {
                p.rect_filled(tr, CornerRadius::same(6), TERM_BG_ELEVATED);
            }
            p.text(
                tr.center(),
                Align2::CENTER_CENTER,
                *t_label,
                if is_active { theme::font_body_semibold() } else { theme::font_body() },
                if is_active { Color32::WHITE } else { theme::TEXT_SECONDARY },
            );
            if tresp.clicked() {
                state.active_tab = t_idx;
                state.selected_idx = 0;
            }
            tx = tr.right() + 6.0;
        }

        // 3. Quick Prompt Template Pills + Agent CLI Launchers Row (32px)
        let pills_row = Rect::from_min_size(Pos2::new(rect.left(), tab_row.bottom()), Vec2::new(bar_w, 32.0));
        widgets::hline(&p, pills_row.left(), pills_row.right(), pills_row.bottom() - 1.0, CMDK_BORDER);
        let mut px = rect.left() + 12.0;
        for (pill_id, label, prompt_preset) in [
            ("cmdk-pill-pricing", "Pricing Card", Some("pricing card")),
            ("cmdk-pill-login", "Login Modal", Some("login modal")),
            ("cmdk-pill-table", "Audit Table", Some("audit log table")),
            ("cmdk-pill-console", "Agent Console", Some("agent console")),
            ("cmdk-pill-claude", "Claude CLI", None),
            ("cmdk-pill-codex", "Codex CLI", None),
        ] {
            let pw = widgets::text_width(&p, label, &theme::font_small_medium()) + 28.0;
            if px + pw > rect.right() - 12.0 {
                break;
            }
            let pr = Rect::from_min_size(Pos2::new(px, pills_row.center().y - 10.0), Vec2::new(pw, 20.0));
            let presp = ui.interact(pr, Id::new(pill_id), Sense::click());
            let is_cli = prompt_preset.is_none();
            let bg = if presp.hovered() {
                CMDK_BG_ROW_HOVER
            } else if is_cli {
                STATUS_INFO_BG
            } else {
                TERM_BG_ELEVATED
            };
            let border = if is_cli { STATUS_INFO_BORDER } else { CMDK_BORDER };
            let fg = if is_cli { STATUS_INFO_TEXT } else { theme::TEXT_SECONDARY };
            p.rect_filled(pr, CornerRadius::same(6), bg);
            p.rect_stroke(pr, CornerRadius::same(6), Stroke::new(1.0, border), StrokeKind::Inside);
            icons::paint(&p, if is_cli { Icon::Dev } else { Icon::Plus }, Pos2::new(pr.left() + 10.0, pr.center().y), 10.0, fg);
            p.text(Pos2::new(pr.left() + 18.0, pr.center().y), Align2::LEFT_CENTER, label, theme::font_small_medium(), fg);
            if presp.clicked() {
                if let Some(pr_text) = prompt_preset {
                    let spec = synthesize_spec_from_prompt(pr_text);
                    state.status = Some(format!("Generated '{}'", spec.name));
                    state.open = false;
                    out_action = Some(CmdKAction::ApplySpec(spec));
                } else if pill_id == "cmdk-pill-claude" {
                    state.open = false;
                    out_action = Some(CmdKAction::OpenTerminal(AgentCliPreset::Claude));
                } else if pill_id == "cmdk-pill-codex" {
                    state.open = false;
                    out_action = Some(CmdKAction::OpenTerminal(AgentCliPreset::Codex));
                }
            }
            px = pr.right() + 6.0;
        }

        // 4. Section Header ("Recents" when query is empty, matching `figma_ui3_actions_cmdk.png`)
        let sec_hdr_y = pills_row.bottom() + 12.0;
        let sec_title = if q_lower.is_empty() { "Recents" } else { "Matching actions" };
        p.text(Pos2::new(rect.left() + 16.0, sec_hdr_y), Align2::LEFT_CENTER, sec_title, theme::font_small_medium(), theme::TEXT_SECONDARY);

        // 5. Command List Rows (36px height, surface-step selection)
        let mut row_y = pills_row.bottom() + 24.0;
        for (list_idx, (_, item)) in filtered_cmds.iter().enumerate() {
            let row_rect = Rect::from_min_size(Pos2::new(rect.left() + 8.0, row_y), Vec2::new(bar_w - 16.0, 34.0));
            let resp = ui.interact(row_rect, Id::new(item.id), Sense::click());
            if resp.hovered() {
                state.selected_idx = list_idx;
            }
            let is_sel = list_idx == state.selected_idx;
            if is_sel {
                p.rect_filled(row_rect, CornerRadius::same(6), CMDK_BG_ROW_HOVER);
            }

            icons::paint(
                &p,
                item.icon,
                Pos2::new(row_rect.left() + 14.0, row_rect.center().y),
                14.0,
                if is_sel { Color32::WHITE } else { theme::ICON_SECONDARY },
            );

            let title_str = widgets::elide(&p, item.title, &theme::font_body(), (row_rect.width() - 120.0).max(60.0));
            p.text(Pos2::new(row_rect.left() + 32.0, row_rect.center().y), Align2::LEFT_CENTER, title_str, theme::font_body(), Color32::WHITE);

            // Right-aligned badge + shortcut symbol
            p.text(Pos2::new(row_rect.right() - 12.0, row_rect.center().y), Align2::RIGHT_CENTER, item.shortcut, theme::font_body(), theme::TEXT_SECONDARY);

            let activated = resp.clicked() || (trigger_selected && is_sel);
            if activated {
                trigger_selected = false;
                match item.id {
                    "cmdk-synth-spec" => {
                        let prompt = if state.query.trim().is_empty() { "pricing card" } else { state.query.trim() };
                        let spec = synthesize_spec_from_prompt(prompt);
                        state.status = Some(format!("Synthesized '{}'", spec.name));
                        state.open = false;
                        out_action = Some(CmdKAction::ApplySpec(spec));
                    }
                    "cmdk-design-gate" => {
                        let report = run_design_gate_audit(doc, page, selection);
                        state.status =
                            Some(format!("Design Gate Score: {}/100 ({} warnings across {} nodes)", report.score, report.warnings.len(), report.node_count));
                        state.last_audit = Some(report);
                    }
                    "cmdk-open-terminal" => {
                        state.open = false;
                        out_action = Some(CmdKAction::OpenTerminal(AgentCliPreset::Claude));
                    }
                    "cmdk-send-selection" => {
                        state.open = false;
                        out_action = Some(CmdKAction::SendSelectionToTerminal);
                    }
                    "cmdk-open-specs" => {
                        state.open = false;
                        out_action = Some(CmdKAction::OpenSpecReader);
                    }
                    "cmdk-export-fig" => {
                        state.open = false;
                        out_action = Some(CmdKAction::ExportFig);
                    }
                    _ => {}
                }
            }
            row_y += 36.0;
        }

        // 6. Live Design Gate Audit Section (separated by 1px hairline — no cards-in-cards)
        if let Some(audit) = state.last_audit.clone() {
            render_audit_section(ui, &p, rect, &mut row_y, &audit, state, &mut out_action);
        }

        // 7. Bottom Hint Footer
        widgets::hline(&p, rect.left(), rect.right(), rect.bottom() - 26.0, CMDK_BORDER);
        let footer_y = rect.bottom() - 13.0;
        let footer_msg = state.status.as_deref().unwrap_or("Up/Down Navigate • Enter Run • Esc Close • Ctrl+` Toggle PTY Dock");
        let elided_footer = widgets::elide(&p, footer_msg, &theme::font_small(), (bar_w - 28.0).max(80.0));
        p.text(Pos2::new(rect.left() + 14.0, footer_y), Align2::LEFT_CENTER, elided_footer, theme::font_small(), theme::TEXT_TERTIARY);
    });

    out_action
}

fn render_audit_section(
    ui: &mut egui::Ui,
    p: &Painter,
    bar_rect: Rect,
    row_y: &mut f32,
    audit: &DesignGateReport,
    state: &mut CmdKState,
    out_action: &mut Option<CmdKAction>,
) {
    let shown_issues = audit.warnings.len().min(4);
    let sec_h = 40.0 + shown_issues as f32 * 26.0;
    let sec_rect = Rect::from_min_size(Pos2::new(bar_rect.left(), *row_y + 2.0), Vec2::new(bar_rect.width(), sec_h));
    p.rect_filled(sec_rect, 0.0, TERM_BG_VIEWPORT);
    widgets::hline(p, sec_rect.left(), sec_rect.right(), sec_rect.top(), TERM_BORDER);

    // Semantic status pill (`tokens.css`: OK / WARN / BAD)
    let (score_fg, score_bg, score_border) = if audit.score >= 90 {
        (STATUS_OK_TEXT, STATUS_OK_BG, STATUS_OK_BORDER)
    } else if audit.score >= 70 {
        (STATUS_WARN_TEXT, STATUS_WARN_BG, STATUS_WARN_BORDER)
    } else {
        (STATUS_BAD_TEXT, STATUS_BAD_BG, STATUS_BAD_BORDER)
    };
    let score_badge = Rect::from_min_size(Pos2::new(sec_rect.left() + 12.0, sec_rect.top() + 8.0), Vec2::new(86.0, 22.0));
    p.rect_filled(score_badge, CornerRadius::same(6), score_bg);
    p.rect_stroke(score_badge, CornerRadius::same(6), Stroke::new(1.0, score_border), StrokeKind::Inside);
    p.text(score_badge.center(), Align2::CENTER_CENTER, format!("SCORE {}/100", audit.score), theme::font_small_strong(), score_fg);

    let summary_txt = format!("{} passed • {} warnings ({} nodes audited)", audit.passed, audit.warnings.len(), audit.node_count);
    let elided_sum = widgets::elide(p, &summary_txt, &theme::font_small_medium(), (sec_rect.width() - 320.0).max(60.0));
    p.text(Pos2::new(score_badge.right() + 10.0, score_badge.center().y), Align2::LEFT_CENTER, elided_sum, theme::font_small_medium(), theme::TEXT_SECONDARY);

    // 1-Click "Auto-Fix Spacing & Tap Targets" button with Sparkle vector icon
    let fix_w = 198.0;
    let fix_rect = Rect::from_min_size(Pos2::new(sec_rect.right() - fix_w - 12.0, sec_rect.top() + 8.0), Vec2::new(fix_w, 22.0));
    let fix_resp = ui.interact(fix_rect, Id::new("cmdk-autofix-gate-btn"), Sense::click());
    p.rect_filled(fix_rect, CornerRadius::same(6), if fix_resp.hovered() { TERM_BG_HOVER } else { STATUS_WARN_BG });
    p.rect_stroke(fix_rect, CornerRadius::same(6), Stroke::new(1.0, STATUS_WARN_BORDER), StrokeKind::Inside);
    icons::paint(p, Icon::Sparkle, Pos2::new(fix_rect.left() + 11.0, fix_rect.center().y), 11.0, STATUS_WARN_TEXT);
    p.text(
        Pos2::new(fix_rect.left() + 20.0, fix_rect.center().y),
        Align2::LEFT_CENTER,
        "Auto-Fix Spacing & Tap Targets",
        theme::font_small_strong(),
        STATUS_WARN_TEXT,
    );
    if fix_resp.clicked() {
        state.status = Some("Applied 4px grid & >=32px tap target auto-fixes".into());
        *out_action = Some(CmdKAction::AutoFixDesignGate);
    }

    let mut iy = sec_rect.top() + 36.0;
    for (idx, issue) in audit.warnings.iter().take(4).enumerate() {
        let focus_r = Rect::from_min_size(Pos2::new(sec_rect.right() - 58.0, iy), Vec2::new(46.0, 20.0));
        let focus_resp = ui.interact(focus_r, Id::new(("cmdk-issue-focus", idx, issue.node_id.0)), Sense::click());
        p.rect_filled(focus_r, CornerRadius::same(4), if focus_resp.hovered() { TERM_BG_HOVER } else { STATUS_INFO_BG });
        p.rect_stroke(focus_r, CornerRadius::same(4), Stroke::new(1.0, STATUS_INFO_BORDER), StrokeKind::Inside);
        p.text(focus_r.center(), Align2::CENTER_CENTER, "Focus", theme::font_small_medium(), STATUS_INFO_TEXT);
        if focus_resp.clicked() {
            state.open = false;
            *out_action = Some(CmdKAction::SelectNode(issue.node_id));
        }

        let line = format!("[{}] {}: {}", issue.rule, issue.node_name, issue.detail);
        let elided_line = widgets::elide(p, &line, &theme::font_small(), (sec_rect.width() - 80.0).max(60.0));
        p.text(Pos2::new(sec_rect.left() + 12.0, iy + 10.0), Align2::LEFT_CENTER, elided_line, theme::font_small(), theme::TEXT);
        iy += 26.0;
    }

    *row_y += sec_h + 4.0;
}

/// Synthesizes a multi-node EaseUI BrainWave 2.0 `NodeSpec` subtree from a natural-language prompt.
pub fn synthesize_spec_from_prompt(prompt: &str) -> NodeSpec {
    let raw = prompt.trim();
    let lower = raw.to_ascii_lowercase();

    if lower.contains("pricing") || lower.contains("plan") || lower.contains("billing") {
        synthesize_pricing_card(raw)
    } else if lower.contains("login") || lower.contains("auth") || lower.contains("sign in") || lower.contains("modal") {
        synthesize_login_modal(raw)
    } else if lower.contains("table") || lower.contains("audit") || lower.contains("log") || lower.contains("grid") {
        synthesize_audit_table(raw)
    } else if lower.contains("console") || lower.contains("terminal") || lower.contains("agent") || lower.contains("telemetry") {
        synthesize_agent_console(raw)
    } else if lower.contains("sidebar") || lower.contains("nav") || lower.contains("menu") {
        synthesize_sidebar_nav(raw)
    } else if lower.contains("hero") || lower.contains("banner") || lower.contains("landing") {
        synthesize_hero_banner(raw)
    } else {
        synthesize_custom_easeui_card(raw)
    }
}

fn text_node(name: &str, text: &str, size: f32, weight: u16, color: &str) -> NodeSpec {
    NodeSpec {
        kind: "text".into(),
        name: name.into(),
        text: Some(text.into()),
        font_size: Some(size),
        font_weight: Some(weight),
        text_color: Some(color.into()),
        ..Default::default()
    }
}

fn badge_node(name: &str, label: &str, fill: &str, text_color: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: name.into(),
        preset: Some("easeui-badge".into()),
        fill: Some(fill.into()),
        text: Some(label.into()),
        text_color: Some(text_color.into()),
        ..Default::default()
    }
}

fn button_node(name: &str, label: &str, primary: bool) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: name.into(),
        preset: Some(if primary { "easeui-button-primary" } else { "easeui-button-secondary" }.into()),
        min_height: Some(36.0),
        text: Some(label.into()),
        ..Default::default()
    }
}

fn synthesize_pricing_card(_prompt: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: "EaseUI Pro Pricing Card".into(),
        preset: Some("easeui-card".into()),
        width: Some(380.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.accent.orange".into()),
        stroke_weight: Some(1.5),
        corner_radius: Some(16.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 16.0, padding: [24.0, 24.0, 24.0, 24.0], ..Default::default() }),
        children: vec![
            badge_node("Tier Badge", "MOST POPULAR • BRAINWAVE 2.0", "$easeui.accent.orange", "$easeui.text.primary"),
            text_node("Plan Title", "Studio Autonomous Pro", 22.0, 700, "$easeui.text.primary"),
            text_node("Plan Price", "$49 / month • Unlimited Agent Specs", 15.0, 600, "$easeui.accent.orange"),
            text_node("Feature 1", "Native Embedded PTY (Claude Code, Codex, Grok)", 13.0, 500, "$easeui.text.secondary"),
            text_node("Feature 2", "2D CSS Grid + Auto-Layout 5.0 Subtree Solver", 13.0, 500, "$easeui.text.secondary"),
            text_node("Feature 3", "1:1 Figma .fig Binary Round-Trip + React/Tailwind Export", 13.0, 500, "$easeui.text.secondary"),
            button_node("Upgrade CTA Button", "Upgrade to Studio Pro ->", true),
        ],
        ..Default::default()
    }
}

fn synthesize_login_modal(_prompt: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: "EaseUI Auth Sign-In Modal".into(),
        preset: Some("easeui-card".into()),
        width: Some(420.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.border.subtle".into()),
        corner_radius: Some(16.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 16.0, padding: [28.0, 28.0, 28.0, 28.0], ..Default::default() }),
        children: vec![
            badge_node("Auth Badge", "SSO • ZERO-TRUST WORKSPACE", "$easeui.accent.blue", "$easeui.text.primary"),
            text_node("Modal Heading", "Sign in to design:os studio", 20.0, 700, "$easeui.text.primary"),
            text_node("Modal Subtitle", "Connect your Claude Code, Codex, and Figma workspace credentials.", 13.0, 400, "$easeui.text.secondary"),
            NodeSpec {
                kind: "frame".into(),
                name: "Email Input Field".into(),
                preset: Some("easeui-input".into()),
                width_sizing: Some("fill".into()),
                min_height: Some(40.0),
                text: Some("jang@easestart.dev".into()),
                ..Default::default()
            },
            NodeSpec {
                kind: "frame".into(),
                name: "API Token Input Field".into(),
                preset: Some("easeui-input".into()),
                width_sizing: Some("fill".into()),
                min_height: Some(40.0),
                text: Some("••••••••••••••••••••••••".into()),
                ..Default::default()
            },
            button_node("Sign In Primary Button", "Continue with Agent Key ->", true),
        ],
        ..Default::default()
    }
}

fn synthesize_audit_table(_prompt: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: "EaseUI Design Gate Audit Table".into(),
        preset: Some("easeui-panel".into()),
        width: Some(680.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.border".into()),
        corner_radius: Some(12.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 12.0, padding: [20.0, 20.0, 20.0, 20.0], ..Default::default() }),
        children: vec![
            badge_node("Gate Status Badge", "VSF GATE G4 • 98/100 PASS", "$easeui.status.success", "$easeui.text.primary"),
            text_node("Table Title", "Live Agent & Design Gate Telemetry Log", 18.0, 700, "$easeui.text.primary"),
            NodeSpec {
                kind: "frame".into(),
                name: "Audit Row 1".into(),
                preset: Some("easeui-row".into()),
                width_sizing: Some("fill".into()),
                fill: Some("$easeui.bg.surface".into()),
                corner_radius: Some(8.0),
                layout: Some(LayoutSpec { direction: "row".into(), gap: 16.0, padding: [12.0, 16.0, 12.0, 16.0], ..Default::default() }),
                children: vec![
                    badge_node("Pass Pill 1", "PASS", "$easeui.status.success", "$easeui.text.primary"),
                    text_node("Rule 1", "4px Spacing Rhythm & Auto-Layout 5.0 Grid", 13.0, 600, "$easeui.text.primary"),
                ],
                ..Default::default()
            },
            NodeSpec {
                kind: "frame".into(),
                name: "Audit Row 2".into(),
                preset: Some("easeui-row".into()),
                width_sizing: Some("fill".into()),
                fill: Some("$easeui.bg.surface".into()),
                corner_radius: Some(8.0),
                layout: Some(LayoutSpec { direction: "row".into(), gap: 16.0, padding: [12.0, 16.0, 12.0, 16.0], ..Default::default() }),
                children: vec![
                    badge_node("Pass Pill 2", "PASS", "$easeui.accent.blue", "$easeui.text.primary"),
                    text_node("Rule 2", "100% $easeui.* Semantic Token Binding Coverage", 13.0, 600, "$easeui.text.primary"),
                ],
                ..Default::default()
            },
        ],
        ..Default::default()
    }
}

fn synthesize_agent_console(_prompt: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: "EaseUI Agent Console Panel".into(),
        preset: Some("easeui-panel".into()),
        width: Some(640.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.accent.violet".into()),
        corner_radius: Some(12.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 12.0, padding: [20.0, 20.0, 20.0, 20.0], ..Default::default() }),
        children: vec![
            badge_node("Agent Runtime Badge", "CLAUDE + CODEX + GROK PTY", "$easeui.accent.violet", "$easeui.text.primary"),
            text_node("Console Header", "Autonomous Design-to-Code Agent Bridge", 18.0, 700, "$easeui.text.primary"),
            text_node(
                "Console Stream",
                "$ designos --apply-spec examples/specs/easeui-agent-console.spec.json --export-code react",
                13.0,
                500,
                "$easeui.accent.emerald",
            ),
            button_node("Run Agent Task Button", "Sync Selection to Active PTY", true),
        ],
        ..Default::default()
    }
}

fn synthesize_sidebar_nav(_prompt: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: "EaseUI Sidebar Navigation".into(),
        preset: Some("easeui-panel".into()),
        width: Some(260.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.border".into()),
        corner_radius: Some(12.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 8.0, padding: [16.0, 16.0, 16.0, 16.0], ..Default::default() }),
        children: vec![
            badge_node("Workspace Pill", "design:os studio", "$easeui.accent.orange", "$easeui.text.primary"),
            button_node("Nav Item Canvas", "Canvas & Layers", true),
            button_node("Nav Item Specs", "Specs & Gherkin Hub", false),
            button_node("Nav Item Terminal", "Embedded Agent PTY", false),
            button_node("Nav Item Tokens", "EaseUI Token Variables", false),
        ],
        ..Default::default()
    }
}

fn synthesize_hero_banner(_prompt: &str) -> NodeSpec {
    NodeSpec {
        kind: "frame".into(),
        name: "EaseUI BrainWave 2.0 Hero Banner".into(),
        preset: Some("easeui-card".into()),
        width: Some(840.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.accent.orange".into()),
        corner_radius: Some(16.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 16.0, padding: [32.0, 32.0, 32.0, 32.0], ..Default::default() }),
        children: vec![
            badge_node("Hero Tag", "AGENT-NATIVE VECTOR & PRODUCT STUDIO", "$easeui.accent.orange", "$easeui.text.primary"),
            text_node("Hero Headline", "Design, Audit, and Ship Production UI with Live Agent PTYs", 28.0, 700, "$easeui.text.primary"),
            text_node(
                "Hero Description",
                "Bidirectional NodeSpec blueprints, 2D CSS Grid layout, and 1:1 Figma .fig binary compatibility.",
                15.0,
                400,
                "$easeui.text.secondary",
            ),
            button_node("Hero Primary CTA", "Launch Agent Workspace ->", true),
        ],
        ..Default::default()
    }
}

fn synthesize_custom_easeui_card(prompt: &str) -> NodeSpec {
    let title = if prompt.is_empty() {
        "EaseUI Generated Component".to_string()
    } else {
        let mut chars = prompt.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => "EaseUI Generated Component".to_string(),
        }
    };
    NodeSpec {
        kind: "frame".into(),
        name: format!("EaseUI — {title}"),
        preset: Some("easeui-card".into()),
        width: Some(420.0),
        fill: Some("$easeui.bg.panel".into()),
        stroke: Some("$easeui.border.subtle".into()),
        corner_radius: Some(12.0),
        dev_status: Some("ready".into()),
        layout: Some(LayoutSpec { direction: "column".into(), gap: 12.0, padding: [20.0, 20.0, 20.0, 20.0], ..Default::default() }),
        children: vec![
            badge_node("Spec Badge", "CMD+K SYNTHESIZED • EASEUI 2.0", "$easeui.accent.orange", "$easeui.text.primary"),
            text_node("Card Title", &title, 18.0, 700, "$easeui.text.primary"),
            text_node("Card Subtitle", "Declarative Auto-Layout 5.0 frame bound to $easeui.* semantic tokens.", 13.0, 400, "$easeui.text.secondary"),
            button_node("Primary Action Button", "Inspect & Export Code", true),
        ],
        ..Default::default()
    }
}

/// Audits the active selection (or the entire `page` subtree when `selection` is empty)
/// against VSF / EaseUI BrainWave 2.0 design rules:
/// 1. **Spacing Rhythm (4px grid)**: `AutoLayout::item_spacing` and `padding` multiples of 4px.
/// 2. **Tap Target (`>= 32px`)**: interactive frames/buttons/instances at least `32px` wide & tall.
/// 3. **Token Compliance**: solid fills/strokes bound to variables or standard EaseUI palette colors.
/// 4. **Text Readability (`>= 11px`)**: `TextStyle::font_size >= 11.0`.
pub fn run_design_gate_audit(doc: &Document, page: NodeId, selection: &[NodeId]) -> DesignGateReport {
    let roots: Vec<NodeId> = if !selection.is_empty() { selection.to_vec() } else { doc.get(page).map(|pg| pg.children.clone()).unwrap_or_default() };

    let mut visited = Vec::new();
    for r in roots {
        collect_subtree(doc, r, &mut visited);
    }

    let mut passed = 0usize;
    let mut warnings = Vec::new();

    for &nid in &visited {
        let Some(node) = doc.get(nid) else { continue };
        let mut node_ok = true;

        // 1. Spacing Rhythm (4px grid)
        if let Some(fp) = node.kind.frame_props() {
            if let Some(al) = &fp.auto_layout {
                if !is_multiple_of_4(al.item_spacing) {
                    node_ok = false;
                    warnings.push(DesignGateIssue {
                        node_id: nid,
                        node_name: node.name.clone(),
                        rule: "Spacing Rhythm (4px)",
                        detail: format!("item_spacing {:.1}px is off the 4px grid", al.item_spacing),
                    });
                }
                for (side_name, pad_val) in [("top", al.padding[0]), ("right", al.padding[1]), ("bottom", al.padding[2]), ("left", al.padding[3])] {
                    if !is_multiple_of_4(pad_val) {
                        node_ok = false;
                        warnings.push(DesignGateIssue {
                            node_id: nid,
                            node_name: node.name.clone(),
                            rule: "Spacing Rhythm (4px)",
                            detail: format!("padding-{side_name} {pad_val:.1}px is off the 4px grid"),
                        });
                        break;
                    }
                }
            }
        }

        // 2. Tap Target (>= 32px) for interactive frames/buttons/instances
        if is_interactive_candidate(node) && (node.size.x < 32.0 || node.size.y < 32.0) {
            node_ok = false;
            warnings.push(DesignGateIssue {
                node_id: nid,
                node_name: node.name.clone(),
                rule: "Tap Target (>=32px)",
                detail: format!("interactive size {:.0}x{:.0}px < 32px minimum", node.size.x, node.size.y),
            });
        }

        // 3. Token Compliance
        if !matches!(node.kind, NodeKind::Page { .. } | NodeKind::Text(_)) {
            let has_solid_fill = node.fills.iter().any(|p| p.visible && matches!(p.kind, PaintKind::Solid(_)));
            if has_solid_fill && node.variables.bindings.is_empty() && !uses_easeui_standard_palette(node) {
                node_ok = false;
                warnings.push(DesignGateIssue {
                    node_id: nid,
                    node_name: node.name.clone(),
                    rule: "Token Compliance",
                    detail: "unbound raw hex fill (bind a $easeui.* variable token)".into(),
                });
            }
        }

        // 4. Text Readability (>= 11px)
        if let Some(tp) = node.kind.any_text() {
            if tp.style.font_size < 11.0 {
                node_ok = false;
                warnings.push(DesignGateIssue {
                    node_id: nid,
                    node_name: node.name.clone(),
                    rule: "Text Readability (>=11px)",
                    detail: format!("font_size {:.1}px is below 11px floor", tp.style.font_size),
                });
            }
        }

        if node_ok {
            passed += 1;
        }
    }

    let node_count = visited.len();
    let penalty = (warnings.len() as u32).saturating_mul(7);
    let score = if node_count == 0 { 100 } else { 100u32.saturating_sub(penalty).max(15) };

    DesignGateReport { score: if warnings.is_empty() { 100 } else { score }, passed, warnings, node_count }
}

/// Auto-fixes spacing rhythm (snaps `item_spacing` and `padding` to nearest 4px multiple),
/// enlarges interactive buttons/inputs `<32px` up to `32px`, and raises `<11px` text to `11px`.
/// Returns the number of modified nodes.
pub fn auto_fix_design_gate(doc: &mut Document, page: NodeId, selection: &[NodeId]) -> usize {
    let roots: Vec<NodeId> = if !selection.is_empty() { selection.to_vec() } else { doc.get(page).map(|pg| pg.children.clone()).unwrap_or_default() };

    let mut visited = Vec::new();
    for r in roots {
        collect_subtree(doc, r, &mut visited);
    }

    let mut fixed_count = 0usize;
    let mut relayout_roots = Vec::new();

    for &nid in &visited {
        let Some(node) = doc.nodes.get_mut(&nid) else {
            continue;
        };
        let mut changed = false;

        if let Some(fp) = node.kind.frame_props_mut() {
            if let Some(al) = &mut fp.auto_layout {
                if !is_multiple_of_4(al.item_spacing) {
                    al.item_spacing = snap_to_4px(al.item_spacing);
                    changed = true;
                }
                for p in &mut al.padding {
                    if !is_multiple_of_4(*p) {
                        *p = snap_to_4px(*p);
                        changed = true;
                    }
                }
            }
        }

        if is_interactive_candidate(node) {
            if node.size.x < 32.0 {
                node.size.x = 32.0;
                changed = true;
            }
            if node.size.y < 32.0 {
                node.size.y = 32.0;
                changed = true;
            }
        }

        if let Some(tp) = node.kind.any_text_mut() {
            if tp.style.font_size < 11.0 {
                tp.style.font_size = 11.0;
                changed = true;
            }
        }

        if changed {
            fixed_count += 1;
            relayout_roots.push(nid);
        }
    }

    for r in relayout_roots {
        doc.relayout_subtree(r);
    }
    fixed_count
}

fn collect_subtree(doc: &Document, root: NodeId, out: &mut Vec<NodeId>) {
    let Some(node) = doc.get(root) else { return };
    if !matches!(node.kind, NodeKind::Page { .. }) {
        out.push(root);
    }
    let children = node.children.clone();
    for child in children {
        collect_subtree(doc, child, out);
    }
}

fn is_multiple_of_4(v: f32) -> bool {
    if v <= 0.0 {
        return true;
    }
    let rounded = v.round();
    (v - rounded).abs() < 0.05 && (rounded as i32) % 4 == 0
}

fn snap_to_4px(v: f32) -> f32 {
    if v <= 0.0 {
        0.0
    } else {
        ((v / 4.0).round() * 4.0).max(4.0)
    }
}

fn is_interactive_candidate(node: &designos_doc::Node) -> bool {
    if matches!(node.kind, NodeKind::Instance { .. }) {
        return true;
    }
    let lower = node.name.to_ascii_lowercase();
    lower.contains("button")
        || lower.contains("btn")
        || lower.contains("cta")
        || lower.contains("input")
        || lower.contains("chip")
        || lower.contains("toggle")
        || lower.contains("tab")
}

fn uses_easeui_standard_palette(node: &designos_doc::Node) -> bool {
    const EASEUI_HEXES: &[&str] = &[
        "121215", "0E0E12", "16171B", "18181B", "18181C", "1F2024", "202024", "26272C", "27272A", "27272D", "2A2B31", "303036", "3F3F46", "F4F4F5", "A1A1AA",
        "71717A", "E36323", "FF6A3D", "3582FF", "2563EB", "8755E9", "A855F7", "18B368", "10B981", "4CC58B", "F59E0B", "E0A53C", "EF4444", "FF7A8C", "8FB0FF",
        "FFFFFF", "FAFAFA", "E4E4E7",
    ];
    node.fills.iter().all(|p| match &p.kind {
        PaintKind::Solid(c) => {
            let hex = c.to_hex();
            EASEUI_HEXES.iter().any(|&h| h.eq_ignore_ascii_case(&hex))
        }
        _ => true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use designos_doc::{AutoLayout, Color, Node, Paint, Rect, TextStyle};

    #[test]
    fn synthesize_spec_from_prompt_produces_rich_easeui_subtrees() {
        for prompt in ["pricing card", "login modal", "audit log table", "agent console", "sidebar nav", "hero banner", "custom telemetry widget"] {
            let spec = synthesize_spec_from_prompt(prompt);
            assert!(!spec.name.is_empty());
            assert!(spec.preset.is_some());
            assert!(!spec.children.is_empty());

            let mut doc = Document::new("Test");
            let page = doc.add_page("Page 1");
            let root_id = doc.apply_spec(page, &spec).expect("synthesized NodeSpec should apply cleanly");
            let root = doc.get(root_id).unwrap();
            assert!(!root.children.is_empty());
        }
    }

    #[test]
    fn design_gate_audit_detects_and_autofixes_spacing_tap_and_font_issues() {
        let mut doc = Document::new("Audit Test");
        let page = doc.add_page("Page 1");

        let btn_id = doc.alloc_id();
        let mut btn = Node::frame(btn_id, Rect::new(0.0, 0.0, 120.0, 24.0));
        btn.name = "Primary Action Button".into();
        btn.fills = vec![Paint::solid(Color::rgb(0x12, 0x34, 0x56))];
        if let Some(fp) = btn.kind.frame_props_mut() {
            fp.auto_layout = Some(AutoLayout { item_spacing: 7.0, padding: [5.0, 12.0, 5.0, 12.0], ..Default::default() });
        }
        doc.insert(page, btn).unwrap();

        let txt_id = doc.alloc_id();
        let mut style = TextStyle::default();
        style.font_size = 9.0;
        let txt = Node::text(txt_id, Rect::new(0.0, 0.0, 60.0, 12.0), "Click", style);
        doc.insert(btn_id, txt).unwrap();

        let report = run_design_gate_audit(&doc, page, &[]);
        assert!(report.score < 100);
        assert!(report.warnings.iter().any(|w| w.rule == "Spacing Rhythm (4px)"));
        assert!(report.warnings.iter().any(|w| w.rule == "Tap Target (>=32px)"));
        assert!(report.warnings.iter().any(|w| w.rule == "Token Compliance"));
        assert!(report.warnings.iter().any(|w| w.rule == "Text Readability (>=11px)"));

        let fixed = auto_fix_design_gate(&mut doc, page, &[]);
        assert!(fixed >= 2);

        let after = run_design_gate_audit(&doc, page, &[]);
        assert!(!after.warnings.iter().any(|w| w.rule == "Spacing Rhythm (4px)"));
        assert!(!after.warnings.iter().any(|w| w.rule == "Tap Target (>=32px)"));
        assert!(!after.warnings.iter().any(|w| w.rule == "Text Readability (>=11px)"));
    }
}
