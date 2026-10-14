//! Native EaseUI-Styled Spec Reader & Agentic Product Hub (`RailItem::Specs`).
//!
//! Discovers workspace spec packages under `plans/specs/*/` (`brainstorm.md`, `usecases.md`,
//! `plan.md`, `*.spec.json`) and standalone declarative UI blueprints under `examples/specs/*.spec.json`.
//! Renders both the Left Sidebar Spec Explorer (`show_left`) and the central EaseUI BrainWave 2.0
//! Spec Reader View (`show_reader`) with structured User Story cards, Gherkin syntax highlighting,
//! NodeSpec token/preset analytics, and 1-click Canvas Frame focus / Spec apply / `.fig` export.

use crate::app::App;
use crate::panels::{clipped, scrollbar, wheel_scroll};
use crate::theme;
use crate::widgets;
use designos_doc::{Command, DevStatus, LayoutDirection, NodeId, NodeSpec};
use egui::{Align2, Color32, CornerRadius, FontId, Id, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// EaseUI BrainWave 2.0 Dark Slate Palette
const EASE_BG_CANVAS: Color32 = Color32::from_rgb(14, 14, 18);
const EASE_BG_CARD: Color32 = Color32::from_rgb(24, 24, 28);
const EASE_BG_CARD_ALT: Color32 = Color32::from_rgb(20, 20, 24);
const EASE_BG_CODE: Color32 = Color32::from_rgb(17, 17, 21);
const EASE_BORDER: Color32 = Color32::from_rgb(39, 39, 45);
const EASE_BORDER_SUBTLE: Color32 = Color32::from_rgb(32, 32, 38);
const EASE_ORANGE: Color32 = Color32::from_rgb(255, 106, 61);
const EASE_ORANGE_BG: Color32 = Color32::from_rgba_premultiplied(52, 24, 16, 255);
const EASE_BLUE: Color32 = Color32::from_rgb(59, 130, 246);
const EASE_BLUE_LIGHT: Color32 = Color32::from_rgb(96, 165, 250);
const EASE_BLUE_BG: Color32 = Color32::from_rgba_premultiplied(18, 34, 62, 255);
const EASE_VIOLET: Color32 = Color32::from_rgb(139, 92, 246);
const EASE_VIOLET_LIGHT: Color32 = Color32::from_rgb(167, 139, 250);
const EASE_VIOLET_BG: Color32 = Color32::from_rgba_premultiplied(34, 22, 60, 255);
const EASE_EMERALD: Color32 = Color32::from_rgb(16, 185, 129);
const EASE_EMERALD_LIGHT: Color32 = Color32::from_rgb(52, 211, 153);
const EASE_EMERALD_BG: Color32 = Color32::from_rgba_premultiplied(12, 44, 34, 255);
const EASE_AMBER: Color32 = Color32::from_rgb(245, 158, 11);
const EASE_AMBER_BG: Color32 = Color32::from_rgba_premultiplied(48, 34, 12, 255);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SpecTab {
    #[default]
    Overview,
    UseCases,
    Plan,
    JsonSpec,
    CanvasScreens,
}

impl SpecTab {
    pub const ALL: [SpecTab; 5] = [
        SpecTab::Overview,
        SpecTab::UseCases,
        SpecTab::Plan,
        SpecTab::JsonSpec,
        SpecTab::CanvasScreens,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SpecTab::Overview => "Overview & PRD",
            SpecTab::UseCases => "Use Cases & Gherkin",
            SpecTab::Plan => "Architecture Plan",
            SpecTab::JsonSpec => "NodeSpec Blueprint (.spec.json)",
            SpecTab::CanvasScreens => "Linked Canvas Screens & Feedback",
        }
    }
}

#[derive(Clone, Debug)]
pub struct SpecPackage {
    pub slug: String,
    pub title: String,
    pub brainstorm_md: String,
    pub usecases_md: String,
    pub plan_md: String,
    pub json_specs: Vec<(String, PathBuf, String)>,
}

#[derive(Default)]
pub struct SpecsState {
    pub open: bool,
    pub loaded: bool,
    pub packages: Vec<SpecPackage>,
    pub selected_pkg: usize,
    pub active_tab: SpecTab,
    pub selected_json_spec: usize,
    pub scroll: f32,
    pub left_scroll: f32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GherkinKeyword {
    Scenario,
    Given,
    When,
    Then,
    And,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GherkinLine {
    pub keyword: GherkinKeyword,
    pub keyword_text: String,
    pub rest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MdBlock {
    Heading {
        level: u8,
        text: String,
    },
    UserStory {
        id_and_title: String,
        as_a: String,
        i_want: String,
        so_that: String,
    },
    Gherkin {
        lines: Vec<GherkinLine>,
    },
    CodeBlock {
        lang: String,
        lines: Vec<String>,
    },
    Bullet {
        badge: Option<String>,
        bold_key: Option<String>,
        text: String,
    },
    Paragraph(String),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SpecJsonSummary {
    pub root_name: String,
    pub total_nodes: usize,
    pub presets: Vec<String>,
    pub easeui_tokens: Vec<String>,
}

impl SpecsState {
    pub fn ensure_loaded(&mut self) {
        if !self.loaded || self.packages.is_empty() {
            self.refresh();
        }
    }

    pub fn refresh(&mut self) {
        self.loaded = true;
        let root = find_workspace_root();
        let mut standalone_specs = Vec::new();
        if let Some(r) = &root {
            let ex_dir = r.join("examples").join("specs");
            collect_json_specs_in(&ex_dir, &mut standalone_specs);
        }
        if standalone_specs.is_empty() {
            standalone_specs.push((
                "easeui-agent-console.spec.json".to_string(),
                PathBuf::from("examples/specs/easeui-agent-console.spec.json"),
                FALLBACK_SPEC_JSON.to_string(),
            ));
        }

        let mut pkgs = Vec::new();
        if let Some(r) = &root {
            let specs_root = r.join("plans").join("specs");
            if let Ok(entries) = std::fs::read_dir(&specs_root) {
                let mut dirs: Vec<PathBuf> = entries
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.is_dir())
                    .collect();
                dirs.sort();
                for dir in dirs {
                    if let Some(pkg) = load_package_dir(&dir, &standalone_specs) {
                        pkgs.push(pkg);
                    }
                }
            }
        }

        if pkgs.is_empty() {
            pkgs.push(fallback_package(standalone_specs));
        } else {
            // Prioritize packages that have full PRD + UseCases + Plan (putting figma-parity-and-agentic-spec-reader first).
            pkgs.sort_by(|a, b| {
                let rank = |p: &SpecPackage| -> (u8, u8) {
                    let featured = if p.slug == "figma-parity-and-agentic-spec-reader" { 0 } else { 1 };
                    let completeness = (!p.brainstorm_md.is_empty() as u8)
                        + (!p.usecases_md.is_empty() as u8)
                        + (!p.plan_md.is_empty() as u8);
                    (featured, 3u8.saturating_sub(completeness))
                };
                rank(a).cmp(&rank(b)).then_with(|| a.slug.cmp(&b.slug))
            });
        }

        self.packages = pkgs;
        if self.selected_pkg >= self.packages.len() {
            self.selected_pkg = 0;
        }
    }

    pub fn current_package(&self) -> Option<&SpecPackage> {
        self.packages.get(self.selected_pkg).or_else(|| self.packages.first())
    }
}

fn find_workspace_root() -> Option<PathBuf> {
    let mut candidates = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        let mut cur = Some(cwd.as_path());
        while let Some(p) = cur {
            candidates.push(p.to_path_buf());
            cur = p.parent();
        }
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut cur = Some(manifest_dir.as_path());
    while let Some(p) = cur {
        candidates.push(p.to_path_buf());
        cur = p.parent();
    }
    for cand in candidates {
        if cand.join("plans").join("specs").is_dir() || cand.join("examples").join("specs").is_dir() {
            return Some(cand);
        }
    }
    None
}

fn collect_json_specs_in(dir: &Path, out: &mut Vec<(String, PathBuf, String)>) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let mut paths: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect();
    paths.sort();
    for p in paths {
        let Some(name) = p.file_name().and_then(|n| n.to_str()).map(|s| s.to_string()) else {
            continue;
        };
        if name.ends_with(".spec.json") {
            if let Ok(raw) = std::fs::read_to_string(&p) {
                out.push((name, p, raw));
            }
        }
    }
}

fn load_package_dir(dir: &Path, standalone_specs: &[(String, PathBuf, String)]) -> Option<SpecPackage> {
    let slug = dir.file_name()?.to_str()?.to_string();
    let brainstorm_md = std::fs::read_to_string(dir.join("brainstorm.md")).unwrap_or_default();
    let usecases_md = std::fs::read_to_string(dir.join("usecases.md")).unwrap_or_default();
    let plan_md = std::fs::read_to_string(dir.join("plan.md"))
        .or_else(|_| std::fs::read_to_string(dir.join("roadmap.md")))
        .unwrap_or_default();

    let mut json_specs = Vec::new();
    collect_json_specs_in(dir, &mut json_specs);
    for s in standalone_specs {
        if !json_specs.iter().any(|(n, _, _)| n == &s.0) {
            json_specs.push(s.clone());
        }
    }

    if brainstorm_md.is_empty() && usecases_md.is_empty() && plan_md.is_empty() && json_specs.is_empty() {
        return None;
    }

    let title = extract_first_heading(&brainstorm_md)
        .or_else(|| extract_first_heading(&plan_md))
        .or_else(|| extract_first_heading(&usecases_md))
        .unwrap_or_else(|| humanize_slug(&slug));

    Some(SpecPackage {
        slug,
        title,
        brainstorm_md,
        usecases_md,
        plan_md,
        json_specs,
    })
}

fn extract_first_heading(md: &str) -> Option<String> {
    for line in md.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("# ") {
            let clean = rest
                .trim_start_matches("Brainstorm:")
                .trim_start_matches("Implementation Plan:")
                .trim_start_matches("Use Cases & Acceptance Criteria:")
                .trim();
            if !clean.is_empty() {
                return Some(clean.to_string());
            }
        }
    }
    None
}

fn humanize_slug(slug: &str) -> String {
    slug.split('-')
        .filter(|w| !w.is_empty())
        .map(|w| {
            let mut chars = w.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

const FALLBACK_BRAINSTORM: &str = include_str!("../../../plans/specs/figma-parity-and-agentic-spec-reader/brainstorm.md");
const FALLBACK_USECASES: &str = include_str!("../../../plans/specs/figma-parity-and-agentic-spec-reader/usecases.md");
const FALLBACK_PLAN: &str = include_str!("../../../plans/specs/figma-parity-and-agentic-spec-reader/plan.md");
const FALLBACK_SPEC_JSON: &str = include_str!("../../../examples/specs/easeui-agent-console.spec.json");

fn fallback_package(standalone_specs: Vec<(String, PathBuf, String)>) -> SpecPackage {
    let title = extract_first_heading(FALLBACK_BRAINSTORM)
        .unwrap_or_else(|| "DESIGN:OS Studio — 1:1 Figma Binary Parity & Agentic Spec Reader".to_string());
    SpecPackage {
        slug: "figma-parity-and-agentic-spec-reader".to_string(),
        title,
        brainstorm_md: FALLBACK_BRAINSTORM.to_string(),
        usecases_md: FALLBACK_USECASES.to_string(),
        plan_md: FALLBACK_PLAN.to_string(),
        json_specs: standalone_specs,
    }
}

/// Parses Markdown and Gherkin blocks into structured visual blocks for the EaseUI Spec Reader.
pub fn parse_markdown_blocks(md: &str) -> Vec<MdBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();

        if trimmed.is_empty() || trimmed == "---" {
            i += 1;
            continue;
        }

        // Fenced code block (```gherkin or ```rust / ```text / etc.)
        if let Some(fence_lang) = trimmed.strip_prefix("```") {
            let lang = fence_lang.trim().to_ascii_lowercase();
            i += 1;
            let mut code_lines = Vec::new();
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1; // skip closing ```
            }
            if lang == "gherkin" {
                let parsed = code_lines.into_iter().filter(|l| !l.trim().is_empty()).map(parse_gherkin_line).collect();
                blocks.push(MdBlock::Gherkin { lines: parsed });
            } else {
                blocks.push(MdBlock::CodeBlock {
                    lang,
                    lines: code_lines.into_iter().map(|s| s.to_string()).collect(),
                });
            }
            continue;
        }

        // User Story Heading: `### US-...`
        if let Some(us_rest) = trimmed.strip_prefix("### ") {
            if us_rest.starts_with("US-") {
                let id_and_title = us_rest.trim().to_string();
                i += 1;
                let mut as_a = String::new();
                let mut i_want = String::new();
                let mut so_that = String::new();
                while i < lines.len() {
                    let l = lines[i].trim();
                    if l.is_empty() {
                        i += 1;
                        if !as_a.is_empty() || !i_want.is_empty() || !so_that.is_empty() {
                            break;
                        }
                        continue;
                    }
                    if l.starts_with("```") || l.starts_with('#') {
                        break;
                    }
                    let clean = strip_inline_md(l);
                    if let Some(r) = clean.strip_prefix("As a ").or_else(|| clean.strip_prefix("As an ")) {
                        as_a = r.trim().trim_end_matches(',').to_string();
                    } else if let Some(r) = clean.strip_prefix("I want ") {
                        i_want = r.trim().trim_end_matches(',').to_string();
                    } else if let Some(r) = clean.strip_prefix("So that ") {
                        so_that = r.trim().trim_end_matches('.').to_string();
                    } else if !so_that.is_empty() {
                        so_that.push(' ');
                        so_that.push_str(&clean);
                    } else if !i_want.is_empty() {
                        i_want.push(' ');
                        i_want.push_str(&clean);
                    } else {
                        as_a.push_str(&clean);
                    }
                    i += 1;
                }
                blocks.push(MdBlock::UserStory {
                    id_and_title,
                    as_a,
                    i_want,
                    so_that,
                });
                continue;
            }
        }

        // Standard Headings: `# `, `## `, `### `
        if let Some(h3) = trimmed.strip_prefix("### ") {
            blocks.push(MdBlock::Heading {
                level: 3,
                text: strip_inline_md(h3),
            });
            i += 1;
            continue;
        }
        if let Some(h2) = trimmed.strip_prefix("## ") {
            blocks.push(MdBlock::Heading {
                level: 2,
                text: strip_inline_md(h2),
            });
            i += 1;
            continue;
        }
        if let Some(h1) = trimmed.strip_prefix("# ") {
            blocks.push(MdBlock::Heading {
                level: 1,
                text: strip_inline_md(h1),
            });
            i += 1;
            continue;
        }

        // Bullet or numbered list item
        if let Some(bullet_body) = trimmed
            .strip_prefix("- ")
            .or_else(|| trimmed.strip_prefix("* "))
            .or_else(|| strip_numbered_prefix(trimmed))
        {
            let (badge, bold_key, text) = parse_bullet_parts(bullet_body);
            blocks.push(MdBlock::Bullet { badge, bold_key, text });
            i += 1;
            continue;
        }

        // Paragraph
        blocks.push(MdBlock::Paragraph(strip_inline_md(trimmed)));
        i += 1;
    }

    blocks
}

fn strip_numbered_prefix(s: &str) -> Option<&str> {
    let dot = s.find(". ")?;
    if dot > 0 && dot <= 3 && s[..dot].chars().all(|c| c.is_ascii_digit()) {
        Some(&s[dot + 2..])
    } else {
        None
    }
}

fn parse_gherkin_line(line: &str) -> GherkinLine {
    let t = line.trim();
    if let Some(rest) = t.strip_prefix("Scenario:") {
        GherkinLine {
            keyword: GherkinKeyword::Scenario,
            keyword_text: "Scenario:".to_string(),
            rest: rest.trim().to_string(),
        }
    } else if let Some(rest) = t.strip_prefix("Given ") {
        GherkinLine {
            keyword: GherkinKeyword::Given,
            keyword_text: "Given".to_string(),
            rest: rest.trim().to_string(),
        }
    } else if let Some(rest) = t.strip_prefix("When ") {
        GherkinLine {
            keyword: GherkinKeyword::When,
            keyword_text: "When".to_string(),
            rest: rest.trim().to_string(),
        }
    } else if let Some(rest) = t.strip_prefix("Then ") {
        GherkinLine {
            keyword: GherkinKeyword::Then,
            keyword_text: "Then".to_string(),
            rest: rest.trim().to_string(),
        }
    } else if let Some(rest) = t.strip_prefix("And ") {
        GherkinLine {
            keyword: GherkinKeyword::And,
            keyword_text: "And".to_string(),
            rest: rest.trim().to_string(),
        }
    } else {
        GherkinLine {
            keyword: GherkinKeyword::Other,
            keyword_text: String::new(),
            rest: t.to_string(),
        }
    }
}

fn parse_bullet_parts(body: &str) -> (Option<String>, Option<String>, String) {
    let trimmed = body.trim();
    if let Some(after_open) = trimmed.strip_prefix("**") {
        if let Some(end_bold) = after_open.find("**") {
            let key = after_open[..end_bold].trim().trim_end_matches(':').to_string();
            let remainder = after_open[end_bold + 2..]
                .trim()
                .trim_start_matches(':')
                .trim_start_matches('—')
                .trim_start_matches('-')
                .trim();
            let badge = detect_moscow_or_uc_badge(&key);
            return (badge, Some(key), strip_inline_md(remainder));
        }
    }
    (None, None, strip_inline_md(trimmed))
}

fn detect_moscow_or_uc_badge(key: &str) -> Option<String> {
    let upper = key.to_ascii_uppercase();
    if upper.starts_with("MUST") {
        Some("MUST".to_string())
    } else if upper.starts_with("SHOULD") {
        Some("SHOULD".to_string())
    } else if upper.starts_with("COULD") {
        Some("COULD".to_string())
    } else if upper.starts_with("UC-") {
        key.split([':', ' ', '—']).next().map(|s| s.to_string())
    } else {
        None
    }
}

fn strip_inline_md(s: &str) -> String {
    s.replace("**", "").replace('`', "")
}

/// Summarizes a declarative `.spec.json` payload (total node count, presets used, `$easeui.*` tokens).
pub fn summarize_json_spec(raw: &str) -> SpecJsonSummary {
    let Ok(spec) = serde_json::from_str::<NodeSpec>(raw) else {
        return SpecJsonSummary::default();
    };
    let mut total_nodes = 0usize;
    let mut presets = BTreeSet::new();
    let mut tokens = BTreeSet::new();
    fn walk(n: &NodeSpec, total: &mut usize, presets: &mut BTreeSet<String>, tokens: &mut BTreeSet<String>) {
        *total += 1;
        if let Some(p) = &n.preset {
            if !p.is_empty() {
                presets.insert(p.clone());
            }
        }
        for col in [&n.fill, &n.stroke, &n.text_color].into_iter().flatten() {
            if col.trim().starts_with("$easeui.") {
                tokens.insert(col.trim().to_string());
            }
        }
        for c in &n.children {
            walk(c, total, presets, tokens);
        }
    }
    walk(&spec, &mut total_nodes, &mut presets, &mut tokens);
    SpecJsonSummary {
        root_name: if spec.name.is_empty() { "Root Spec".to_string() } else { spec.name },
        total_nodes,
        presets: presets.into_iter().collect(),
        easeui_tokens: tokens.into_iter().collect(),
    }
}

impl App {
    /// Applies a declarative [`NodeSpec`] to the active page in one undoable command and fits the canvas.
    pub fn apply_node_spec(&mut self, spec: &NodeSpec, label: &str) {
        let parent = self.page;
        let mut trial = self.doc.clone();
        match trial.apply_spec(parent, spec) {
            Ok(root_id) => {
                let mut nodes = Vec::new();
                trial.walk(root_id, |n, _| nodes.push(n.clone()));
                let index = self.doc.get(parent).map(|p| p.children.len()).unwrap_or(0);
                if self.hist.exec(&mut self.doc, Command::Insert { parent, index, nodes }).is_ok() {
                    self.doc.relayout_subtree(parent);
                    self.selection = vec![root_id];
                    self.changed();
                    self.fit_all();
                    self.show_toast(format!("Applied UI spec '{label}' to canvas"), false);
                }
            }
            Err(e) => self.show_toast(format!("Spec error: {e}"), true),
        }
    }

    /// Exports the selected frame (or first screen on the active page) to `examples/specs/<slug>.spec.json`.
    pub fn export_active_frame_to_spec(&mut self) {
        let target_id = self
            .selection
            .first()
            .copied()
            .or_else(|| self.doc.get(self.page).and_then(|p| p.children.first().copied()))
            .unwrap_or(self.page);
        let node_name = self.doc.get(target_id).map(|n| n.name.clone()).unwrap_or_else(|| "screen".to_string());
        let slug: String = node_name
            .to_ascii_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-");
        let filename = format!("{}.spec.json", if slug.is_empty() { "canvas-screen" } else { &slug });
        let out_dir = find_workspace_root()
            .map(|r| r.join("examples").join("specs"))
            .unwrap_or_else(|| std::env::temp_dir().join("designos_specs"));
        let _ = std::fs::create_dir_all(&out_dir);
        let out_path = out_dir.join(&filename);
        match self.doc.export_spec(target_id) {
            Ok(spec) => match serde_json::to_string_pretty(&spec) {
                Ok(json) => match std::fs::write(&out_path, json) {
                    Ok(()) => {
                        self.specs.refresh();
                        self.show_toast(format!("Exported {}", out_path.display()), false);
                    }
                    Err(e) => self.show_toast(format!("Cannot write spec: {e}"), true),
                },
                Err(e) => self.show_toast(format!("Serialize error: {e}"), true),
            },
            Err(e) => self.show_toast(format!("Export spec error: {e}"), true),
        }
    }

    /// Saves a local `.fig` binary copy of the current document.
    pub fn export_fig_copy(&mut self) {
        let stem = if self.doc.name.trim().is_empty() { "Untitled" } else { self.doc.name.trim() };
        let default_path = self
            .file_path
            .as_ref()
            .and_then(|p| p.parent().map(|d| d.join(format!("{stem}.fig"))))
            .unwrap_or_else(|| std::env::temp_dir().join(format!("{stem}.fig")));
        match crate::io::save_path(&self.doc, &default_path) {
            Ok(saved) => self.show_toast(format!("Saved .fig binary to {}", saved.display()), false),
            Err(e) => self.show_toast(e, true),
        }
    }
}

/// Left Sidebar Spec Explorer rendered when `app.ui.rail == Some(RailItem::Specs)`.
pub fn show_left(app: &mut App, ui: &mut Ui, rect: Rect) {
    app.specs.ensure_loaded();
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, theme::BG);

    // Top Header Banner: "AGENTIC PRODUCT HUB" + Reader toggle button
    let header_h = 44.0;
    let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_h));
    p.rect_filled(header_rect, 0.0, EASE_BG_CARD_ALT);
    widgets::hline(&p, header_rect.left(), header_rect.right(), header_rect.bottom() - 1.0, EASE_BORDER);

    // Orange status dot + title
    p.circle_filled(Pos2::new(header_rect.left() + 14.0, header_rect.center().y), 3.5, EASE_ORANGE);
    let hub_label = widgets::elide(&p, "AGENTIC HUB", &theme::font_small_strong(), (header_rect.width() - 96.0).max(40.0));
    p.text(
        Pos2::new(header_rect.left() + 24.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        hub_label,
        theme::font_small_strong(),
        EASE_ORANGE,
    );

    // "Reader" toggle pill button
    let toggle_w = 58.0;
    let toggle_rect = Rect::from_min_size(
        Pos2::new(header_rect.right() - toggle_w - 10.0, header_rect.center().y - 11.0),
        Vec2::new(toggle_w, 22.0),
    );
    let toggle_resp = ui.interact(toggle_rect, Id::new("specs-reader-toggle"), Sense::click());
    let toggle_bg = if app.specs.open {
        EASE_BLUE
    } else if toggle_resp.hovered() {
        theme::BG_HOVER
    } else {
        EASE_BG_CARD
    };
    p.rect_filled(toggle_rect, CornerRadius::same(6), toggle_bg);
    p.rect_stroke(
        toggle_rect,
        CornerRadius::same(6),
        Stroke::new(1.0, if app.specs.open { EASE_BLUE_LIGHT } else { EASE_BORDER }),
        StrokeKind::Inside,
    );
    p.text(
        toggle_rect.center(),
        Align2::CENTER_CENTER,
        "Reader",
        theme::font_small_medium(),
        theme::TEXT,
    );
    widgets::record(ui, "specs-reader-toggle", toggle_rect);
    if toggle_resp.clicked() {
        app.specs.open = !app.specs.open;
    }

    // Bottom Action Bar (2 rows of quick actions)
    let bottom_h = 74.0;
    let body_rect = Rect::from_min_max(
        Pos2::new(rect.left(), header_rect.bottom()),
        Pos2::new(rect.right(), (rect.bottom() - bottom_h).max(header_rect.bottom() + 40.0)),
    );
    let bottom_rect = Rect::from_min_max(Pos2::new(rect.left(), body_rect.bottom()), rect.max);

    // Scrollable middle area
    let pkg_count = app.specs.packages.len();
    let json_count = app.specs.current_package().map(|p| p.json_specs.len()).unwrap_or(0);
    let page_frames: Vec<NodeId> = app
        .doc
        .get(app.page)
        .map(|pg| pg.children.clone())
        .unwrap_or_default();
    let est_h = 34.0
        + pkg_count as f32 * 56.0
        + 38.0
        + json_count as f32 * 44.0
        + 38.0
        + page_frames.len().max(1) as f32 * 44.0
        + 24.0;

    let mut lscroll = app.specs.left_scroll;
    wheel_scroll(ui, body_rect, &mut lscroll, est_h);
    app.specs.left_scroll = lscroll;

    let mut apply_json_action: Option<(String, String)> = None;
    let mut focus_frame_action: Option<NodeId> = None;

    let resolved_comments = app.doc.resolve_comment_targets();

    clipped(ui, body_rect, |ui| {
        let p = ui.painter().clone();
        let mut y = body_rect.top() + 8.0 - lscroll;
        let pad_x = 10.0;
        let row_w = (body_rect.width() - pad_x * 2.0).max(60.0);

        // SECTION 1: Spec Packages (plans/specs/)
        draw_left_section_header(&p, Pos2::new(body_rect.left() + pad_x, y), row_w, "SPEC PACKAGES", EASE_BLUE_LIGHT);
        y += 24.0;

        let pkgs_snapshot = app.specs.packages.clone();
        for (idx, pkg) in pkgs_snapshot.iter().enumerate() {
            let card_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 50.0));
            let selected = idx == app.specs.selected_pkg;
            let resp = ui.interact(card_r, Id::new(("spec-pkg-row", idx)), Sense::click());
            let bg = if selected {
                EASE_BLUE_BG
            } else if resp.hovered() {
                theme::BG_HOVER
            } else {
                EASE_BG_CARD
            };
            p.rect_filled(card_r, CornerRadius::same(8), bg);
            p.rect_stroke(
                card_r,
                CornerRadius::same(8),
                Stroke::new(1.0, if selected { EASE_BLUE } else { EASE_BORDER }),
                StrokeKind::Inside,
            );
            let title_str = widgets::elide(&p, &pkg.title, &theme::font_body_strong(), (card_r.width() - 20.0).max(20.0));
            p.text(
                Pos2::new(card_r.left() + 10.0, card_r.top() + 15.0),
                Align2::LEFT_CENTER,
                title_str,
                theme::font_body_strong(),
                theme::TEXT,
            );
            let slug_str = widgets::elide(&p, &pkg.slug, &theme::font_small(), (card_r.width() - 84.0).max(20.0));
            p.text(
                Pos2::new(card_r.left() + 10.0, card_r.top() + 34.0),
                Align2::LEFT_CENTER,
                slug_str,
                theme::font_small(),
                theme::TEXT_SECONDARY,
            );
            // Status badge on bottom-right of card
            let badge_txt = if !pkg.usecases_md.is_empty() { "Gherkin" } else { "Plan" };
            draw_mini_pill(
                &p,
                Pos2::new(card_r.right() - 8.0, card_r.top() + 34.0),
                Align2::RIGHT_CENTER,
                badge_txt,
                EASE_EMERALD_BG,
                EASE_EMERALD_LIGHT,
            );
            widgets::record(ui, &format!("spec-pkg:{idx}"), card_r);
            if resp.clicked() {
                app.specs.selected_pkg = idx;
                app.specs.open = true;
                app.specs.scroll = 0.0;
            }
            y += 56.0;
        }

        y += 6.0;
        // SECTION 2: Declarative UI Blueprints (*.spec.json)
        draw_left_section_header(&p, Pos2::new(body_rect.left() + pad_x, y), row_w, "UI BLUEPRINTS (.SPEC.JSON)", EASE_VIOLET_LIGHT);
        y += 24.0;

        let json_list = app
            .specs
            .current_package()
            .map(|pkg| pkg.json_specs.clone())
            .unwrap_or_default();
        for (j_idx, (j_name, _j_path, j_raw)) in json_list.iter().enumerate() {
            let row_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 38.0));
            let apply_w = 48.0;
            let apply_r = Rect::from_min_size(
                Pos2::new(row_r.right() - apply_w - 6.0, row_r.center().y - 11.0),
                Vec2::new(apply_w, 22.0),
            );
            let main_click_r = Rect::from_min_max(row_r.min, Pos2::new(apply_r.left() - 4.0, row_r.bottom()));
            let row_resp = ui.interact(main_click_r, Id::new(("spec-json-row", j_idx)), Sense::click());
            let apply_resp = ui.interact(apply_r, Id::new(("spec-json-apply", j_idx)), Sense::click());

            let active = app.specs.active_tab == SpecTab::JsonSpec && app.specs.selected_json_spec == j_idx;
            p.rect_filled(
                row_r,
                CornerRadius::same(7),
                if active {
                    EASE_VIOLET_BG
                } else if row_resp.hovered() {
                    theme::BG_HOVER
                } else {
                    EASE_BG_CARD
                },
            );
            p.rect_stroke(
                row_r,
                CornerRadius::same(7),
                Stroke::new(1.0, if active { EASE_VIOLET } else { EASE_BORDER }),
                StrokeKind::Inside,
            );
            let clean_name = widgets::elide(&p, j_name, &theme::font_body(), (main_click_r.width() - 14.0).max(20.0));
            p.text(
                Pos2::new(row_r.left() + 10.0, row_r.center().y),
                Align2::LEFT_CENTER,
                clean_name,
                theme::font_body(),
                theme::TEXT,
            );

            // 1-click Apply button
            p.rect_filled(
                apply_r,
                CornerRadius::same(5),
                if apply_resp.hovered() { EASE_EMERALD_LIGHT } else { EASE_EMERALD },
            );
            p.text(
                apply_r.center(),
                Align2::CENTER_CENTER,
                "Apply",
                theme::font_small_strong(),
                Color32::from_rgb(10, 14, 12),
            );
            widgets::record(ui, &format!("spec-json-apply:{j_idx}"), apply_r);
            if apply_resp.clicked() {
                apply_json_action = Some((j_name.clone(), j_raw.clone()));
            } else if row_resp.clicked() {
                app.specs.selected_json_spec = j_idx;
                app.specs.active_tab = SpecTab::JsonSpec;
                app.specs.open = true;
            }
            y += 44.0;
        }

        y += 6.0;
        // SECTION 3: Canvas Frames & Pin Feedback
        draw_left_section_header(&p, Pos2::new(body_rect.left() + pad_x, y), row_w, "CANVAS FRAMES & PINS", EASE_EMERALD_LIGHT);
        y += 24.0;

        for fid in &page_frames {
            let Some(fnode) = app.doc.get(*fid) else { continue };
            let row_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 38.0));
            let resp = ui.interact(row_r, Id::new(("spec-frame-row", fid.0)), Sense::click());
            let is_sel = app.selection.contains(fid);
            p.rect_filled(
                row_r,
                CornerRadius::same(7),
                if is_sel {
                    theme::BG_SELECTED
                } else if resp.hovered() {
                    theme::BG_HOVER
                } else {
                    EASE_BG_CARD
                },
            );
            p.rect_stroke(
                row_r,
                CornerRadius::same(7),
                Stroke::new(1.0, if is_sel { EASE_ORANGE } else { EASE_BORDER }),
                StrokeKind::Inside,
            );

            let open_pins = resolved_comments
                .iter()
                .filter(|c| !c.resolved && (c.target_frame_id == Some(fid.0) || c.target_node_id == Some(fid.0)))
                .count();
            let ready = fnode.dev.status == Some(DevStatus::ReadyForDev);

            let right_reserve = if open_pins > 0 || ready { 62.0 } else { 12.0 };
            let fname = widgets::elide(&p, &fnode.name, &theme::font_body(), (row_r.width() - right_reserve - 12.0).max(20.0));
            p.text(
                Pos2::new(row_r.left() + 10.0, row_r.center().y),
                Align2::LEFT_CENTER,
                fname,
                theme::font_body(),
                theme::TEXT,
            );

            if open_pins > 0 {
                let badge = format!("{open_pins} pin{}", if open_pins == 1 { "" } else { "s" });
                draw_mini_pill(
                    &p,
                    Pos2::new(row_r.right() - 8.0, row_r.center().y),
                    Align2::RIGHT_CENTER,
                    &badge,
                    EASE_ORANGE_BG,
                    EASE_ORANGE,
                );
            } else if ready {
                draw_mini_pill(
                    &p,
                    Pos2::new(row_r.right() - 8.0, row_r.center().y),
                    Align2::RIGHT_CENTER,
                    "Ready",
                    EASE_EMERALD_BG,
                    EASE_EMERALD_LIGHT,
                );
            }

            if resp.clicked() {
                focus_frame_action = Some(*fid);
            }
            y += 44.0;
        }
    });
    scrollbar(ui, body_rect, app.specs.left_scroll, est_h);

    if let Some((j_name, j_raw)) = apply_json_action {
        match serde_json::from_str::<NodeSpec>(&j_raw) {
            Ok(spec) => app.apply_node_spec(&spec, &j_name),
            Err(e) => app.show_toast(format!("Invalid .spec.json: {e}"), true),
        }
    }
    if let Some(fid) = focus_frame_action {
        app.select_only(fid);
        app.specs.open = false;
        app.fit_selection();
    }

    // Bottom Action Bar
    p.rect_filled(bottom_rect, 0.0, EASE_BG_CARD_ALT);
    widgets::hline(&p, bottom_rect.left(), bottom_rect.right(), bottom_rect.top(), EASE_BORDER);
    let btn_w = (bottom_rect.width() - 20.0).max(60.0);
    let btn1 = Rect::from_min_size(Pos2::new(bottom_rect.left() + 10.0, bottom_rect.top() + 8.0), Vec2::new(btn_w, 26.0));
    let btn2 = Rect::from_min_size(Pos2::new(bottom_rect.left() + 10.0, bottom_rect.top() + 40.0), Vec2::new(btn_w, 26.0));

    let r1 = ui.interact(btn1, Id::new("specs-export-json-btn"), Sense::click());
    p.rect_filled(btn1, CornerRadius::same(6), if r1.hovered() { theme::BG_HOVER } else { EASE_BG_CARD });
    p.rect_stroke(btn1, CornerRadius::same(6), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);
    let b1_txt = widgets::elide(&p, "Export Frame -> .spec.json", &theme::font_small_medium(), (btn1.width() - 16.0).max(20.0));
    p.text(btn1.center(), Align2::CENTER_CENTER, b1_txt, theme::font_small_medium(), theme::TEXT);
    if r1.clicked() {
        app.export_active_frame_to_spec();
    }

    let r2 = ui.interact(btn2, Id::new("specs-save-fig-btn"), Sense::click());
    p.rect_filled(btn2, CornerRadius::same(6), if r2.hovered() { EASE_VIOLET } else { EASE_VIOLET_BG });
    p.rect_stroke(btn2, CornerRadius::same(6), Stroke::new(1.0, EASE_VIOLET), StrokeKind::Inside);
    let b2_txt = widgets::elide(&p, "Save .fig Copy", &theme::font_small_strong(), (btn2.width() - 16.0).max(20.0));
    p.text(btn2.center(), Align2::CENTER_CENTER, b2_txt, theme::font_small_strong(), theme::TEXT);
    if r2.clicked() {
        app.export_fig_copy();
    }
}

/// Central EaseUI BrainWave 2.0 Spec Reader View rendered over the canvas area when `app.specs.open` is true.
pub fn show_reader(app: &mut App, ui: &mut Ui, rect: Rect) {
    app.specs.ensure_loaded();
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, EASE_BG_CANVAS);

    let Some(pkg) = app.specs.current_package().cloned() else {
        return;
    };

    let pad = 20.0;
    let inner_w = (rect.width() - pad * 2.0).max(200.0);

    // 1. Top Hero Banner (96px tall)
    let hero_h = 92.0;
    let hero_rect = Rect::from_min_size(Pos2::new(rect.left() + pad, rect.top() + 14.0), Vec2::new(inner_w, hero_h));
    p.rect_filled(hero_rect, CornerRadius::same(12), EASE_BG_CARD);
    p.rect_stroke(hero_rect, CornerRadius::same(12), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);

    // Top row inside hero: status pills + action buttons on the right
    let mut pill_x = hero_rect.left() + 16.0;
    let pill_y = hero_rect.top() + 22.0;
    for (txt, bg, fg) in [
        ("AGENT-FIRST", EASE_ORANGE_BG, EASE_ORANGE),
        ("EASEUI TOKENS", EASE_BLUE_BG, EASE_BLUE_LIGHT),
        ("1:1 FIGMA", EASE_VIOLET_BG, EASE_VIOLET_LIGHT),
    ] {
        pill_x = draw_mini_pill(&p, Pos2::new(pill_x, pill_y), Align2::LEFT_CENTER, txt, bg, fg) + 8.0;
    }
    let slug_badge = format!("plans/specs/{}", pkg.slug);
    let _ = draw_mini_pill(
        &p,
        Pos2::new(pill_x, pill_y),
        Align2::LEFT_CENTER,
        &slug_badge,
        EASE_BG_CODE,
        theme::TEXT_SECONDARY,
    );

    // Hero Action Buttons on the top-right
    let mut btn_right = hero_rect.right() - 16.0;
    let btn_y = hero_rect.top() + 22.0;

    // Button 4: "View on Canvas"
    let view_canvas_r = draw_action_button(
        ui,
        &p,
        "hero-view-canvas",
        Pos2::new(btn_right, btn_y),
        "View on Canvas",
        EASE_BG_CODE,
        EASE_BORDER,
        theme::TEXT,
    );
    btn_right = view_canvas_r.left() - 8.0;

    // Button 3: "Export .fig Binary"
    let export_fig_r = draw_action_button(
        ui,
        &p,
        "hero-export-fig",
        Pos2::new(btn_right, btn_y),
        "Export .fig Binary",
        EASE_VIOLET_BG,
        EASE_VIOLET,
        EASE_VIOLET_LIGHT,
    );
    btn_right = export_fig_r.left() - 8.0;

    // Button 2: "Export Canvas to .spec.json"
    let export_spec_r = draw_action_button(
        ui,
        &p,
        "hero-export-spec",
        Pos2::new(btn_right, btn_y),
        "Export Canvas to .spec.json",
        EASE_BLUE_BG,
        EASE_BLUE,
        EASE_BLUE_LIGHT,
    );
    btn_right = export_spec_r.left() - 8.0;

    // Button 1: "Apply UI Spec to Canvas"
    let apply_spec_r = draw_action_button(
        ui,
        &p,
        "hero-apply-spec",
        Pos2::new(btn_right, btn_y),
        "Apply UI Spec to Canvas",
        EASE_EMERALD_BG,
        EASE_EMERALD,
        EASE_EMERALD_LIGHT,
    );

    // Hero Title row
    let title_max_w = (hero_rect.width() - 32.0).max(100.0);
    let hero_title = widgets::elide(&p, &pkg.title, &theme::font_large_strong(), title_max_w);
    p.text(
        Pos2::new(hero_rect.left() + 16.0, hero_rect.top() + 62.0),
        Align2::LEFT_CENTER,
        hero_title,
        theme::font_large_strong(),
        theme::TEXT,
    );

    // Handle Hero Button Clicks
    if ui.interact(view_canvas_r, Id::new("hero-view-canvas-click"), Sense::click()).clicked() {
        app.specs.open = false;
    }
    if ui.interact(export_fig_r, Id::new("hero-export-fig-click"), Sense::click()).clicked() {
        app.export_fig_copy();
    }
    if ui.interact(export_spec_r, Id::new("hero-export-spec-click"), Sense::click()).clicked() {
        app.export_active_frame_to_spec();
    }
    if ui.interact(apply_spec_r, Id::new("hero-apply-spec-click"), Sense::click()).clicked() {
        if let Some((j_name, _, j_raw)) = pkg.json_specs.get(app.specs.selected_json_spec).or_else(|| pkg.json_specs.first()) {
            if let Ok(spec) = serde_json::from_str::<NodeSpec>(j_raw) {
                app.apply_node_spec(&spec, j_name);
            }
        }
    }

    // 2. Segmented Tab Bar
    let tab_bar_y = hero_rect.bottom() + 12.0;
    let tab_bar_h = 36.0;
    let tab_bar_rect = Rect::from_min_size(Pos2::new(rect.left() + pad, tab_bar_y), Vec2::new(inner_w, tab_bar_h));
    p.rect_filled(tab_bar_rect, CornerRadius::same(9), EASE_BG_CARD);
    p.rect_stroke(tab_bar_rect, CornerRadius::same(9), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);

    let tab_w = (inner_w - 8.0) / SpecTab::ALL.len() as f32;
    for (ti, tab) in SpecTab::ALL.iter().enumerate() {
        let tr = Rect::from_min_size(
            Pos2::new(tab_bar_rect.left() + 4.0 + ti as f32 * tab_w, tab_bar_rect.top() + 4.0),
            Vec2::new(tab_w - 2.0, tab_bar_h - 8.0),
        );
        let resp = ui.interact(tr, Id::new(("spec-reader-tab", ti)), Sense::click());
        let active = app.specs.active_tab == *tab;
        if active {
            p.rect_filled(tr, CornerRadius::same(6), EASE_BLUE_BG);
            p.rect_stroke(tr, CornerRadius::same(6), Stroke::new(1.0, EASE_BLUE), StrokeKind::Inside);
        } else if resp.hovered() {
            p.rect_filled(tr, CornerRadius::same(6), theme::BG_HOVER);
        }
        let lbl = widgets::elide(&p, tab.label(), &theme::font_body_strong(), (tr.width() - 14.0).max(20.0));
        p.text(
            tr.center(),
            Align2::CENTER_CENTER,
            lbl,
            if active { theme::font_body_strong() } else { theme::font_body() },
            if active { EASE_BLUE_LIGHT } else { theme::TEXT_SECONDARY },
        );
        widgets::record(ui, &format!("spec-tab:{ti}"), tr);
        if resp.clicked() {
            app.specs.active_tab = *tab;
            app.specs.scroll = 0.0;
        }
    }

    // 3. Content Area below Tab Bar
    let content_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, tab_bar_rect.bottom() + 12.0),
        Pos2::new(rect.right() - pad, rect.bottom() - 14.0),
    );

    match app.specs.active_tab {
        SpecTab::Overview => {
            render_markdown_tab(app, ui, content_rect, &pkg.brainstorm_md);
        }
        SpecTab::UseCases => {
            render_markdown_tab(app, ui, content_rect, &pkg.usecases_md);
        }
        SpecTab::Plan => {
            render_markdown_tab(app, ui, content_rect, &pkg.plan_md);
        }
        SpecTab::JsonSpec => {
            render_json_spec_tab(app, ui, content_rect, &pkg);
        }
        SpecTab::CanvasScreens => {
            render_canvas_screens_tab(app, ui, content_rect);
        }
    }
}

fn render_markdown_tab(app: &mut App, ui: &mut Ui, rect: Rect, md: &str) {
    let blocks = parse_markdown_blocks(md);
    let est_h = blocks
        .iter()
        .map(|b| match b {
            MdBlock::Heading { level: 1, .. } => 44.0,
            MdBlock::Heading { .. } => 38.0,
            MdBlock::UserStory { .. } => 114.0,
            MdBlock::Gherkin { lines } => 44.0 + lines.len() as f32 * 24.0,
            MdBlock::CodeBlock { lines, .. } => 36.0 + lines.len() as f32 * 20.0,
            MdBlock::Bullet { .. } => 34.0,
            MdBlock::Paragraph(_) => 36.0,
        })
        .sum::<f32>()
        + 40.0;

    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    clipped(ui, rect, |ui| {
        let p = ui.painter().clone();
        let mut y = rect.top() - scroll;
        let w = (rect.width() - 10.0).max(120.0);

        for block in &blocks {
            match block {
                MdBlock::Heading { level, text } => {
                    let row_h = if *level == 1 { 36.0 } else { 30.0 };
                    let r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, row_h));
                    let accent_col = match level {
                        1 => EASE_ORANGE,
                        2 => EASE_BLUE,
                        _ => EASE_VIOLET,
                    };
                    // 3px accent bar (< 8px width so layout lint never treats it as a text container)
                    p.rect_filled(
                        Rect::from_min_size(Pos2::new(r.left(), r.top() + 5.0), Vec2::new(3.0, row_h - 10.0)),
                        CornerRadius::same(2),
                        accent_col,
                    );
                    let font = if *level == 1 { theme::font_large_strong() } else { theme::font_body_strong() };
                    let clean = widgets::elide(&p, text, &font, (w - 20.0).max(40.0));
                    p.text(Pos2::new(r.left() + 12.0, r.center().y), Align2::LEFT_CENTER, clean, font, theme::TEXT);
                    y += row_h + 6.0;
                }
                MdBlock::UserStory {
                    id_and_title,
                    as_a,
                    i_want,
                    so_that,
                } => {
                    let card_h = 104.0;
                    let card_r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card_h));
                    p.rect_filled(card_r, CornerRadius::same(10), EASE_BG_CARD);
                    p.rect_stroke(card_r, CornerRadius::same(10), Stroke::new(1.0, EASE_BLUE), StrokeKind::Inside);

                    let title_end = draw_mini_pill(
                        &p,
                        Pos2::new(card_r.left() + 14.0, card_r.top() + 18.0),
                        Align2::LEFT_CENTER,
                        "USER STORY",
                        EASE_BLUE_BG,
                        EASE_BLUE_LIGHT,
                    );
                    let us_title = widgets::elide(&p, id_and_title, &theme::font_body_strong(), (card_r.right() - title_end - 24.0).max(40.0));
                    p.text(
                        Pos2::new(title_end + 10.0, card_r.top() + 18.0),
                        Align2::LEFT_CENTER,
                        us_title,
                        theme::font_body_strong(),
                        theme::TEXT,
                    );

                    for (row_idx, (tag, val, bg, fg)) in [
                        ("As a", as_a.as_str(), EASE_BLUE_BG, EASE_BLUE_LIGHT),
                        ("I want", i_want.as_str(), EASE_VIOLET_BG, EASE_VIOLET_LIGHT),
                        ("So that", so_that.as_str(), EASE_EMERALD_BG, EASE_EMERALD_LIGHT),
                    ]
                    .into_iter()
                    .enumerate()
                    {
                        let ry = card_r.top() + 42.0 + row_idx as f32 * 22.0;
                        let pill_right = draw_mini_pill(&p, Pos2::new(card_r.left() + 14.0, ry), Align2::LEFT_CENTER, tag, bg, fg);
                        let line_txt = widgets::elide(&p, val, &theme::font_body(), (card_r.right() - pill_right - 20.0).max(40.0));
                        p.text(
                            Pos2::new(pill_right + 10.0, ry),
                            Align2::LEFT_CENTER,
                            line_txt,
                            theme::font_body(),
                            theme::TEXT,
                        );
                    }
                    y += card_h + 10.0;
                }
                MdBlock::Gherkin { lines } => {
                    let card_h = 36.0 + lines.len() as f32 * 24.0;
                    let card_r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card_h));
                    p.rect_filled(card_r, CornerRadius::same(10), EASE_BG_CODE);
                    p.rect_stroke(card_r, CornerRadius::same(10), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);

                    let _ = draw_mini_pill(
                        &p,
                        Pos2::new(card_r.left() + 14.0, card_r.top() + 16.0),
                        Align2::LEFT_CENTER,
                        "GHERKIN ACCEPTANCE CRITERIA",
                        EASE_EMERALD_BG,
                        EASE_EMERALD_LIGHT,
                    );

                    for (li, gl) in lines.iter().enumerate() {
                        let ly = card_r.top() + 38.0 + li as f32 * 24.0;
                        let (kw_bg, kw_fg) = match gl.keyword {
                            GherkinKeyword::Scenario => (EASE_ORANGE_BG, EASE_ORANGE),
                            GherkinKeyword::Given => (EASE_BLUE_BG, EASE_BLUE_LIGHT),
                            GherkinKeyword::When => (EASE_VIOLET_BG, EASE_VIOLET_LIGHT),
                            GherkinKeyword::Then | GherkinKeyword::And => (EASE_EMERALD_BG, EASE_EMERALD_LIGHT),
                            GherkinKeyword::Other => (EASE_BG_CARD, theme::TEXT_SECONDARY),
                        };
                        let mut text_x = card_r.left() + 14.0;
                        if !gl.keyword_text.is_empty() {
                            text_x = draw_mini_pill(&p, Pos2::new(text_x, ly), Align2::LEFT_CENTER, &gl.keyword_text, kw_bg, kw_fg) + 8.0;
                        }
                        let rest_str = widgets::elide(&p, &gl.rest, &theme::font_mono(), (card_r.right() - text_x - 16.0).max(40.0));
                        p.text(
                            Pos2::new(text_x, ly),
                            Align2::LEFT_CENTER,
                            rest_str,
                            theme::font_mono(),
                            theme::TEXT,
                        );
                    }
                    y += card_h + 10.0;
                }
                MdBlock::CodeBlock { lang, lines } => {
                    let card_h = 32.0 + lines.len() as f32 * 20.0;
                    let card_r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card_h));
                    p.rect_filled(card_r, CornerRadius::same(8), EASE_BG_CODE);
                    p.rect_stroke(card_r, CornerRadius::same(8), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);
                    let tag = if lang.is_empty() { "CODE" } else { lang.as_str() };
                    let _ = draw_mini_pill(
                        &p,
                        Pos2::new(card_r.left() + 12.0, card_r.top() + 14.0),
                        Align2::LEFT_CENTER,
                        tag,
                        EASE_BG_CARD,
                        theme::TEXT_SECONDARY,
                    );
                    for (li, cl) in lines.iter().enumerate() {
                        let ly = card_r.top() + 32.0 + li as f32 * 20.0;
                        let s = widgets::elide(&p, cl, &theme::font_mono(), (card_r.width() - 28.0).max(40.0));
                        p.text(Pos2::new(card_r.left() + 14.0, ly), Align2::LEFT_CENTER, s, theme::font_mono(), theme::TEXT_SECONDARY);
                    }
                    y += card_h + 8.0;
                }
                MdBlock::Bullet { badge, bold_key, text } => {
                    let row_h = 28.0;
                    let row_r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, row_h));
                    p.rect_filled(row_r, CornerRadius::same(6), EASE_BG_CARD_ALT);
                    p.rect_stroke(row_r, CornerRadius::same(6), Stroke::new(1.0, EASE_BORDER_SUBTLE), StrokeKind::Inside);

                    let mut cur_x = row_r.left() + 12.0;
                    p.circle_filled(Pos2::new(cur_x, row_r.center().y), 2.5, EASE_BLUE_LIGHT);
                    cur_x += 10.0;

                    if let Some(b) = badge {
                        let (bg, fg) = if b == "MUST" {
                            (EASE_ORANGE_BG, EASE_ORANGE)
                        } else if b == "SHOULD" {
                            (EASE_BLUE_BG, EASE_BLUE_LIGHT)
                        } else {
                            (EASE_VIOLET_BG, EASE_VIOLET_LIGHT)
                        };
                        cur_x = draw_mini_pill(&p, Pos2::new(cur_x, row_r.center().y), Align2::LEFT_CENTER, b, bg, fg) + 8.0;
                    }
                    if let Some(k) = bold_key {
                        let key_str = widgets::elide(&p, k, &theme::font_body_strong(), (row_r.right() - cur_x - 40.0).max(20.0));
                        let kw = widgets::text_width(&p, &key_str, &theme::font_body_strong());
                        p.text(
                            Pos2::new(cur_x, row_r.center().y),
                            Align2::LEFT_CENTER,
                            key_str,
                            theme::font_body_strong(),
                            theme::TEXT,
                        );
                        cur_x += kw + 8.0;
                    }
                    if !text.is_empty() && cur_x < row_r.right() - 24.0 {
                        let body_str = widgets::elide(&p, text, &theme::font_body(), (row_r.right() - cur_x - 12.0).max(20.0));
                        p.text(
                            Pos2::new(cur_x, row_r.center().y),
                            Align2::LEFT_CENTER,
                            body_str,
                            theme::font_body(),
                            theme::TEXT_SECONDARY,
                        );
                    }
                    y += row_h + 6.0;
                }
                MdBlock::Paragraph(txt) => {
                    let row_h = 30.0;
                    let row_r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, row_h));
                    p.rect_filled(row_r, CornerRadius::same(6), EASE_BG_CARD);
                    let line = widgets::elide(&p, txt, &theme::font_body(), (w - 28.0).max(40.0));
                    p.text(
                        Pos2::new(row_r.left() + 14.0, row_r.center().y),
                        Align2::LEFT_CENTER,
                        line,
                        theme::font_body(),
                        theme::TEXT_SECONDARY,
                    );
                    y += row_h + 6.0;
                }
            }
        }
    });
    scrollbar(ui, rect, app.specs.scroll, est_h);
}

fn render_json_spec_tab(app: &mut App, ui: &mut Ui, rect: Rect, pkg: &SpecPackage) {
    let p = ui.painter().clone();
    let Some((j_name, _j_path, j_raw)) = pkg.json_specs.get(app.specs.selected_json_spec).or_else(|| pkg.json_specs.first()).cloned() else {
        return;
    };
    let summary = summarize_json_spec(&j_raw);

    // Top summary metrics card (84px)
    let metrics_h = 80.0;
    let metrics_r = Rect::from_min_size(rect.min, Vec2::new(rect.width(), metrics_h));
    p.rect_filled(metrics_r, CornerRadius::same(10), EASE_BG_CARD);
    p.rect_stroke(metrics_r, CornerRadius::same(10), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);

    let title_str = widgets::elide(
        &p,
        &format!("{}  ·  {}", j_name, summary.root_name),
        &theme::font_body_strong(),
        (metrics_r.width() - 180.0).max(60.0),
    );
    p.text(
        Pos2::new(metrics_r.left() + 16.0, metrics_r.top() + 22.0),
        Align2::LEFT_CENTER,
        title_str,
        theme::font_body_strong(),
        theme::TEXT,
    );

    // Apply to Active Page button inside summary card
    let apply_btn = draw_action_button(
        ui,
        &p,
        "json-tab-apply-btn",
        Pos2::new(metrics_r.right() - 16.0, metrics_r.top() + 22.0),
        "Apply to Active Page",
        EASE_EMERALD_BG,
        EASE_EMERALD,
        EASE_EMERALD_LIGHT,
    );
    if ui.interact(apply_btn, Id::new("json-tab-apply-click"), Sense::click()).clicked() {
        if let Ok(spec) = serde_json::from_str::<NodeSpec>(&j_raw) {
            app.apply_node_spec(&spec, &j_name);
        }
    }

    // Metrics badges row
    let mut bx = metrics_r.left() + 16.0;
    let by = metrics_r.top() + 54.0;
    bx = draw_mini_pill(
        &p,
        Pos2::new(bx, by),
        Align2::LEFT_CENTER,
        &format!("{} Nodes", summary.total_nodes),
        EASE_BLUE_BG,
        EASE_BLUE_LIGHT,
    ) + 8.0;
    bx = draw_mini_pill(
        &p,
        Pos2::new(bx, by),
        Align2::LEFT_CENTER,
        &format!("{} Presets", summary.presets.len()),
        EASE_VIOLET_BG,
        EASE_VIOLET_LIGHT,
    ) + 8.0;
    bx = draw_mini_pill(
        &p,
        Pos2::new(bx, by),
        Align2::LEFT_CENTER,
        &format!("{} $easeui.* Tokens", summary.easeui_tokens.len()),
        EASE_ORANGE_BG,
        EASE_ORANGE,
    ) + 8.0;
    for token in summary.easeui_tokens.iter().take(4) {
        if bx + 110.0 > metrics_r.right() {
            break;
        }
        bx = draw_mini_pill(&p, Pos2::new(bx, by), Align2::LEFT_CENTER, token, EASE_BG_CODE, theme::TEXT_SECONDARY) + 6.0;
    }

    // JSON Preview Card below
    let code_rect = Rect::from_min_max(Pos2::new(rect.left(), metrics_r.bottom() + 10.0), rect.max);
    p.rect_filled(code_rect, CornerRadius::same(10), EASE_BG_CODE);
    p.rect_stroke(code_rect, CornerRadius::same(10), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);

    let json_lines: Vec<&str> = j_raw.lines().collect();
    let est_h = json_lines.len() as f32 * 18.0 + 24.0;
    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, code_rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let inner_code = code_rect.shrink(8.0);
    clipped(ui, inner_code, |ui| {
        let p = ui.painter().clone();
        for (li, line) in json_lines.iter().enumerate() {
            let ly = inner_code.top() + li as f32 * 18.0 - scroll + 9.0;
            if ly < inner_code.top() - 18.0 || ly > inner_code.bottom() + 18.0 {
                continue;
            }
            let col = if line.contains("\"$easeui.") {
                EASE_ORANGE
            } else if line.contains("\"preset\"") {
                EASE_VIOLET_LIGHT
            } else if line.contains("\"name\"") {
                EASE_BLUE_LIGHT
            } else {
                theme::TEXT_SECONDARY
            };
            let s = widgets::elide(&p, line, &theme::font_mono(), (inner_code.width() - 16.0).max(40.0));
            p.text(Pos2::new(inner_code.left() + 8.0, ly), Align2::LEFT_CENTER, s, theme::font_mono(), col);
        }
    });
    scrollbar(ui, code_rect, app.specs.scroll, est_h);
}

fn render_canvas_screens_tab(app: &mut App, ui: &mut Ui, rect: Rect) {
    let page_children: Vec<NodeId> = app.doc.get(app.page).map(|p| p.children.clone()).unwrap_or_default();
    let resolved_comments = app.doc.resolve_comment_targets();

    let est_h = page_children.len().max(1) as f32 * 76.0 + 20.0;
    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let mut focus_target: Option<NodeId> = None;
    let mut export_target: Option<NodeId> = None;

    clipped(ui, rect, |ui| {
        let p = ui.painter().clone();
        let mut y = rect.top() - scroll;
        let w = (rect.width() - 10.0).max(160.0);

        for fid in &page_children {
            let Some(node) = app.doc.get(*fid) else { continue };
            let card_h = 68.0;
            let card_r = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card_h));
            p.rect_filled(card_r, CornerRadius::same(10), EASE_BG_CARD);
            p.rect_stroke(card_r, CornerRadius::same(10), Stroke::new(1.0, EASE_BORDER), StrokeKind::Inside);

            let title = widgets::elide(&p, &node.name, &theme::font_body_strong(), (w - 300.0).max(60.0));
            p.text(
                Pos2::new(card_r.left() + 16.0, card_r.top() + 20.0),
                Align2::LEFT_CENTER,
                title,
                theme::font_body_strong(),
                theme::TEXT,
            );

            // Subtree stats: auto-layout direction, bound variables count, open comment threads
            let mut bound_vars = 0usize;
            app.doc.walk(*fid, |n, _| bound_vars += n.variables.bindings.len());
            let al_dir = match node.kind.frame_props().and_then(|f| f.auto_layout) {
                Some(al) if al.direction == LayoutDirection::Horizontal => "Auto-Layout Row",
                Some(_) => "Auto-Layout Column",
                None => "Freeform Frame",
            };
            let open_comments = resolved_comments
                .iter()
                .filter(|c| !c.resolved && (c.target_frame_id == Some(fid.0) || c.target_node_id == Some(fid.0)))
                .count();

            let mut bx = card_r.left() + 16.0;
            let by = card_r.top() + 46.0;
            bx = draw_mini_pill(
                &p,
                Pos2::new(bx, by),
                Align2::LEFT_CENTER,
                &format!("{:.0}x{:.0}", node.size.x, node.size.y),
                EASE_BG_CODE,
                theme::TEXT_SECONDARY,
            ) + 8.0;
            bx = draw_mini_pill(
                &p,
                Pos2::new(bx, by),
                Align2::LEFT_CENTER,
                &format!("{} children", node.children.len()),
                EASE_BLUE_BG,
                EASE_BLUE_LIGHT,
            ) + 8.0;
            bx = draw_mini_pill(&p, Pos2::new(bx, by), Align2::LEFT_CENTER, al_dir, EASE_VIOLET_BG, EASE_VIOLET_LIGHT) + 8.0;
            if bound_vars > 0 {
                bx = draw_mini_pill(
                    &p,
                    Pos2::new(bx, by),
                    Align2::LEFT_CENTER,
                    &format!("{bound_vars} bound vars"),
                    EASE_EMERALD_BG,
                    EASE_EMERALD_LIGHT,
                ) + 8.0;
            }
            if open_comments > 0 {
                let _ = draw_mini_pill(
                    &p,
                    Pos2::new(bx, by),
                    Align2::LEFT_CENTER,
                    &format!("{open_comments} open comments"),
                    EASE_ORANGE_BG,
                    EASE_ORANGE,
                );
            } else {
                let _ = draw_mini_pill(&p, Pos2::new(bx, by), Align2::LEFT_CENTER, "0 open comments", EASE_AMBER_BG, EASE_AMBER);
            }

            // Right action buttons: Focus on Canvas + Export Frame to .spec.json
            let exp_btn = draw_action_button(
                ui,
                &p,
                &format!("screen-export-{}", fid.0),
                Pos2::new(card_r.right() - 16.0, card_r.center().y),
                "Export Frame to .spec.json",
                EASE_BG_CODE,
                EASE_BORDER,
                theme::TEXT,
            );
            let foc_btn = draw_action_button(
                ui,
                &p,
                &format!("screen-focus-{}", fid.0),
                Pos2::new(exp_btn.left() - 8.0, card_r.center().y),
                "Focus on Canvas",
                EASE_BLUE_BG,
                EASE_BLUE,
                EASE_BLUE_LIGHT,
            );
            if ui.interact(foc_btn, Id::new(("screen-focus-click", fid.0)), Sense::click()).clicked() {
                focus_target = Some(*fid);
            }
            if ui.interact(exp_btn, Id::new(("screen-export-click", fid.0)), Sense::click()).clicked() {
                export_target = Some(*fid);
            }

            y += card_h + 8.0;
        }
    });
    scrollbar(ui, rect, app.specs.scroll, est_h);

    if let Some(fid) = focus_target {
        app.select_only(fid);
        app.specs.open = false;
        app.fit_selection();
    }
    if let Some(fid) = export_target {
        app.select_only(fid);
        app.export_active_frame_to_spec();
    }
}

fn draw_left_section_header(p: &Painter, pos: Pos2, width: f32, title: &str, color: Color32) {
    let r = Rect::from_min_size(pos, Vec2::new(width, 20.0));
    let txt = widgets::elide(p, title, &theme::font_small_strong(), (width - 8.0).max(20.0));
    p.text(Pos2::new(r.left() + 2.0, r.center().y), Align2::LEFT_CENTER, txt, theme::font_small_strong(), color);
}

/// Draws a pill badge and returns the right-most x coordinate of the pill.
fn draw_mini_pill(p: &Painter, anchor: Pos2, align: Align2, text: &str, bg: Color32, fg: Color32) -> f32 {
    let font: FontId = theme::font_small_strong();
    let tw = widgets::text_width(p, text, &font);
    let w = (tw + 14.0).max(24.0);
    let h = 18.0;
    let rect = match align {
        Align2::RIGHT_CENTER => Rect::from_min_size(Pos2::new(anchor.x - w, anchor.y - h * 0.5), Vec2::new(w, h)),
        _ => Rect::from_min_size(Pos2::new(anchor.x, anchor.y - h * 0.5), Vec2::new(w, h)),
    };
    p.rect_filled(rect, CornerRadius::same(5), bg);
    p.text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
    rect.right()
}

/// Draws a right-aligned action button at `right_center` and returns its `Rect`.
fn draw_action_button(
    ui: &mut Ui,
    p: &Painter,
    id_salt: &str,
    right_center: Pos2,
    label: &str,
    bg: Color32,
    border: Color32,
    fg: Color32,
) -> Rect {
    let font = theme::font_small_strong();
    let tw = widgets::text_width(p, label, &font);
    let w = (tw + 20.0).max(48.0);
    let h = 24.0;
    let rect = Rect::from_min_size(Pos2::new(right_center.x - w, right_center.y - h * 0.5), Vec2::new(w, h));
    let resp = ui.interact(rect, Id::new(id_salt), Sense::hover());
    p.rect_filled(rect, CornerRadius::same(6), if resp.hovered() { border } else { bg });
    p.rect_stroke(rect, CornerRadius::same(6), Stroke::new(1.0, border), StrokeKind::Inside);
    p.text(
        rect.center(),
        Align2::CENTER_CENTER,
        label,
        font,
        if resp.hovered() { theme::TEXT } else { fg },
    );
    widgets::record(ui, id_salt, rect);
    rect
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_workspace_spec_packages_and_json_blueprints() {
        let mut state = SpecsState::default();
        state.ensure_loaded();
        assert!(!state.packages.is_empty(), "must discover workspace or fallback spec packages");
        let pkg = state.current_package().expect("active package");
        assert!(!pkg.title.is_empty());
        assert!(!pkg.brainstorm_md.is_empty());
        assert!(!pkg.usecases_md.is_empty());
        assert!(!pkg.plan_md.is_empty());
        assert!(!pkg.json_specs.is_empty(), "must include .spec.json blueprints");
    }

    #[test]
    fn parses_markdown_user_stories_and_gherkin_scenarios() {
        let blocks = parse_markdown_blocks(FALLBACK_USECASES);
        let has_us = blocks.iter().any(|b| {
            matches!(
                b,
                MdBlock::UserStory {
                    id_and_title,
                    as_a,
                    i_want,
                    so_that,
                } if id_and_title.starts_with("US-1") && !as_a.is_empty() && !i_want.is_empty() && !so_that.is_empty()
            )
        });
        assert!(has_us, "User stories must parse As a / I want / So that fields");

        let has_gherkin = blocks.iter().any(|b| match b {
            MdBlock::Gherkin { lines } => {
                lines.iter().any(|l| l.keyword == GherkinKeyword::Scenario)
                    && lines.iter().any(|l| l.keyword == GherkinKeyword::Given)
                    && lines.iter().any(|l| l.keyword == GherkinKeyword::When)
                    && lines.iter().any(|l| l.keyword == GherkinKeyword::Then)
            }
            _ => false,
        });
        assert!(has_gherkin, "Gherkin blocks must parse Scenario / Given / When / Then lines");
    }

    #[test]
    fn summarizes_nodespec_json_metrics() {
        let summary = summarize_json_spec(FALLBACK_SPEC_JSON);
        assert!(summary.total_nodes >= 10, "expected rich NodeSpec tree, got {}", summary.total_nodes);
        assert!(summary.presets.contains(&"easeui-screen".to_string()));
        assert!(!summary.easeui_tokens.is_empty());
    }

    #[test]
    fn renders_specs_reader_and_left_panel_without_lint_issues() {
        let ctx = egui::Context::default();
        theme::install_fonts(&ctx);
        theme::apply(&ctx);
        let mut app = App::new(crate::demo::demo_document());
        app.ui.rail = Some(crate::app::RailItem::Specs);
        app.specs.open = true;
        app.specs.ensure_loaded();

        let left_rect = Rect::from_min_size(Pos2::new(40.0, 0.0), Vec2::new(240.0, 960.0));
        let reader_rect = Rect::from_min_size(Pos2::new(280.0, 0.0), Vec2::new(1080.0, 960.0));

        for tab in SpecTab::ALL {
            app.specs.active_tab = tab;
            let raw = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(1600.0, 960.0))),
                ..Default::default()
            };
            let out = ctx.run_ui(raw, |ui| {
                show_left(&mut app, ui, left_rect);
                show_reader(&mut app, ui, reader_rect);
            });
            let (texts, boxes) = crate::lint::scan_shapes(&out.shapes);
            let issues = crate::lint::check_text(&texts, &boxes);
            assert!(issues.is_empty(), "lint issues on tab {:?}: {:#?}", tab, issues);
        }

        let before_nodes = app.doc.nodes.len();
        let spec: NodeSpec = serde_json::from_str(FALLBACK_SPEC_JSON).expect("valid fallback NodeSpec");
        app.apply_node_spec(&spec, "easeui-agent-console.spec.json");
        assert!(app.doc.nodes.len() > before_nodes, "applying .spec.json should add nodes to the canvas");
    }
}
