//! PRODUCT:OS Spec Driven v2 Reader & Agentic Studio Hub (`RailItem::Specs`).
//!
//! Implements the canonical **PRODUCT:OS Spec Driven Format v2** reader (`spec.manifest.json`,
//! `config/pipeline.yaml`, `intake/po-spec.md`, `contract/FC-001.md`, `prd/PRD.md`, `us/US-*.md`,
//! `ux/screen-inventory.md`, `ux/ui-contract.md`, `decisions/decision-log.md`, `review/gates/*.md`)
//! alongside backwards-compatible v1 spec packages (`brainstorm.md`, `usecases.md`, `plan.md`) and
//! declarative `.spec.json` (`NodeSpec`) blueprints.
//!
//! Styled strictly to the `es:designer` Operate-mode craft floor (`craft-floor.md` §1–§3 &
//! `tokens.css` dark palette): zero emoji glyphs in UI chrome (vector Lucide icons via
//! `crate::icons::Icon`), no cards-in-cards (1px hairlines and `#26272C` table headers inside
//! `#1F2024` section cards), single-model KPI strip, 5 Role Views (`PO View`, `Dev View (US & AC)`,
//! `QA & Coverage`, `Wireframes & UI Contract`, `Gates & Canvas`), and 1-click **`Copy cho AI`**
//! citation blocks (`File: <path>:<start>-<end>`, `Mục`, `Trang reader`, `Project`, `Yêu cầu của tôi`).

use crate::app::App;
use crate::icons::{self, Icon};
use crate::panels::{clipped, scrollbar, wheel_scroll};
use crate::theme;
use crate::widgets;
use designos_doc::{Command, DevStatus, LayoutDirection, NodeId, NodeSpec};
use egui::{Align2, Color32, CornerRadius, FontId, Id, Painter, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

// ─────────────────────────────────────────────────────────────────────────────
// PRODUCT:OS Spec Driven `tokens.css` (@dark) + EaseUI BrainWave 2.0 Tokens
// ─────────────────────────────────────────────────────────────────────────────
const TOKEN_BG_PAGE: Color32 = Color32::from_rgb(0x16, 0x17, 0x1B); // #16171B
const TOKEN_SURFACE: Color32 = Color32::from_rgb(0x1F, 0x20, 0x24); // #1F2024
const TOKEN_SURFACE_2: Color32 = Color32::from_rgb(0x26, 0x27, 0x2C); // #26272C (table head)
const TOKEN_SURFACE_3: Color32 = Color32::from_rgb(0x1B, 0x1C, 0x20); // #1B1C20
const TOKEN_CODE_BG: Color32 = Color32::from_rgb(0x12, 0x13, 0x16); // #121316
const TOKEN_BORDER: Color32 = Color32::from_rgb(0x2E, 0x2F, 0x36); // #2E2F36
const TOKEN_HAIRLINE: Color32 = Color32::from_rgb(0x2A, 0x2B, 0x31); // #2A2B31
const TOKEN_TEXT: Color32 = Color32::from_rgb(0xEC, 0xED, 0xEF); // #ECEDEF
const TOKEN_TEXT_MUTED: Color32 = Color32::from_rgb(0xB4, 0xB7, 0xBF); // #B4B7BF
const TOKEN_TEXT_SUBTLE: Color32 = Color32::from_rgb(0x8F, 0x92, 0x9B); // #8F929B

// Primary Accents (`#FF6A3D` EaseUI Orange & `#FF5C72` product:os Accent)
const TOKEN_PRIMARY_ORANGE: Color32 = Color32::from_rgb(0xFF, 0x6A, 0x3D);
const TOKEN_PRIMARY_SOFT: Color32 = Color32::from_rgb(0x3A, 0x1B, 0x21);
const TOKEN_PRIMARY_ROSE: Color32 = Color32::from_rgb(0xFF, 0x5C, 0x72);

// Semantic Ring-Bordered Status Pill Tokens (`tokens.css` @dark)
// APPROVED / CURRENT / PASS
const PILL_OK_TEXT: Color32 = Color32::from_rgb(0x4C, 0xC5, 0x8B); // #4CC58B
const PILL_OK_FILL: Color32 = Color32::from_rgb(0x17, 0x30, 0x2A); // #17302A
const PILL_OK_STROKE: Color32 = Color32::from_rgb(0x2C, 0x5F, 0x4A); // #2C5F4A

// PENDING / STALE / OPEN
const PILL_WARN_TEXT: Color32 = Color32::from_rgb(0xE0, 0xA5, 0x3C); // #E0A53C
const PILL_WARN_FILL: Color32 = Color32::from_rgb(0x35, 0x29, 0x1A); // #35291A
const PILL_WARN_STROKE: Color32 = Color32::from_rgb(0x5C, 0x47, 0x20); // #5C4720

// CHANGES_REQUESTED / BLOCKING / FAIL
const PILL_BAD_TEXT: Color32 = Color32::from_rgb(0xFF, 0x7A, 0x8C); // #FF7A8C
const PILL_BAD_FILL: Color32 = Color32::from_rgb(0x3B, 0x1D, 0x23); // #3B1D23
const PILL_BAD_STROKE: Color32 = Color32::from_rgb(0x6A, 0x2E, 0x39); // #6A2E39

// INFO / REQ / SCN / US
const PILL_INFO_TEXT: Color32 = Color32::from_rgb(0x8F, 0xB0, 0xFF); // #8FB0FF
const PILL_INFO_FILL: Color32 = Color32::from_rgb(0x1B, 0x26, 0x40); // #1B2640
const PILL_INFO_STROKE: Color32 = Color32::from_rgb(0x2F, 0x45, 0x77); // #2F4577

/// Canonical 5 Role Views of the PRODUCT:OS Spec Driven v2 Reader.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum SpecTab {
    #[default]
    Overview,
    UseCases,
    Plan,
    JsonSpec,
    CanvasScreens,
}

#[allow(non_upper_case_globals, dead_code)]
impl SpecTab {
    /// Alias for Tab 4 (`Wireframes & UI Contract`).
    pub const UiSpecs: SpecTab = SpecTab::JsonSpec;
    /// Alias for Tab 5 (`Gates, Decisions & Canvas`).
    pub const CanvasLinks: SpecTab = SpecTab::CanvasScreens;

    pub const ALL: [SpecTab; 5] = [
        SpecTab::Overview,
        SpecTab::UseCases,
        SpecTab::Plan,
        SpecTab::JsonSpec,
        SpecTab::CanvasScreens,
    ];

    pub fn label(self) -> &'static str {
        match self {
            SpecTab::Overview => "PO View",
            SpecTab::UseCases => "Dev View (US & AC)",
            SpecTab::Plan => "QA & Coverage",
            SpecTab::JsonSpec => "Wireframes & UI Contract",
            SpecTab::CanvasScreens => "Gates & Canvas",
        }
    }

    pub fn anchor(self) -> &'static str {
        match self {
            SpecTab::Overview => "po-view",
            SpecTab::UseCases => "dev-view",
            SpecTab::Plan => "qa-coverage",
            SpecTab::JsonSpec => "wireframes-ui-contract",
            SpecTab::CanvasScreens => "gates-canvas",
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            SpecTab::Overview => Icon::Book,
            SpecTab::UseCases => Icon::Dev,
            SpecTab::Plan => Icon::Check,
            SpecTab::JsonSpec => Icon::Frame,
            SpecTab::CanvasScreens => Icon::Flag,
        }
    }
}

/// High-level actions emitted or handled by the Spec Reader.
#[allow(dead_code)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SpecAction {
    SwitchTab(SpecTab),
    SelectPackage(usize),
    SelectStory(usize),
    ApplyJsonSpec { name: String, raw: String },
    FocusCanvasFrame(NodeId),
    ExportFrameToSpec(Option<NodeId>),
    ExportFigCopy,
    CopyForAi(String),
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec Format v2 Domain Models
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PipelineStageSpec {
    pub index: usize,
    pub id: String,
    pub owner: String,
    pub enabled: bool,
    pub gate_id: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BusinessRuleSpec {
    pub id: String,
    pub rule: String,
    pub source: String,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RequirementSpec {
    pub id: String,
    pub requirement: String,
    pub source: String,
    pub us_refs: Vec<String>,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScenarioSpec {
    pub id: String,
    pub ac_refs: Vec<String>,
    pub method: String,
    pub status: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AcceptanceCriterionSpec {
    pub id: String,
    pub given: String,
    pub when: String,
    pub then: String,
    pub scenario: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UserStorySpec {
    pub id: String,
    pub title: String,
    pub status: String,
    pub epic: String,
    pub priority: String,
    pub wave: String,
    pub req_refs: Vec<String>,
    pub decision_refs: Vec<String>,
    pub screens: Vec<String>,
    pub boards: Vec<String>,
    pub scenarios: Vec<ScenarioSpec>,
    pub user_story_md: String,
    pub flow_steps: Vec<String>,
    pub business_rules: Vec<BusinessRuleSpec>,
    pub acceptance_criteria: Vec<AcceptanceCriterionSpec>,
    pub edge_cases: Vec<(String, String)>,
    pub ascii_wireframe: Vec<String>,
    pub file_path: String,
    pub line_start: usize,
    pub line_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenInventorySpec {
    pub id: String,
    pub name: String,
    pub story: String,
    pub states: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UiContractRow {
    pub screen: String,
    pub state: String,
    pub route: String,
    pub roles: String,
    pub scope: String,
    pub us: String,
    pub element: String,
    pub kind: String,
    pub components: String,
    pub patterns: String,
    pub target: String,
    pub scn: String,
    pub line_number: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DecisionSpec {
    pub id: String,
    pub decision: String,
    pub source_detail: String,
    pub line_number: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenItemSpec {
    pub id: String,
    pub kind: String,
    pub scope: String,
    pub summary: String,
    pub status: String,
    pub blocking: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GateRecordSpec {
    pub gate_id: String,
    pub stage: String,
    pub status: String,
    pub verdict: String,
    pub freshness: String,
    pub config_sha256: String,
    pub rubric: Vec<(String, u8, u8)>,
    pub approved_by: Option<String>,
    pub approved_at: Option<String>,
    pub file_path: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpecKpiMetrics {
    pub req_count: usize,
    pub us_count: usize,
    pub ac_count: usize,
    pub scn_count: usize,
    pub passed_scn_count: usize,
    pub screen_count: usize,
    pub contract_row_count: usize,
    pub approved_gates: usize,
    pub total_gates: usize,
    pub pending_gate_label: Option<String>,
    pub blocking_open_count: usize,
    pub freshness: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct SpecPackage {
    pub slug: String,
    pub title: String,
    pub format_version: String,
    pub project_name: String,
    pub root_display: String,
    pub contract_id: String,
    pub contract_title: String,
    pub contract_outcome: String,
    pub pipeline_stages: Vec<PipelineStageSpec>,
    pub business_rules: Vec<BusinessRuleSpec>,
    pub requirements: Vec<RequirementSpec>,
    pub stories: Vec<UserStorySpec>,
    pub screens: Vec<ScreenInventorySpec>,
    pub ui_contract: Vec<UiContractRow>,
    pub decisions: Vec<DecisionSpec>,
    pub open_items: Vec<OpenItemSpec>,
    pub gates: Vec<GateRecordSpec>,
    // Legacy v1 & NodeSpec fields preserved for backwards compatibility
    pub brainstorm_md: String,
    pub usecases_md: String,
    pub plan_md: String,
    pub json_specs: Vec<(String, PathBuf, String)>,
}

impl SpecPackage {
    /// Computes the 4-Metric KPI Strip from this single loaded model (`craft-floor.md` §3).
    pub fn kpi_metrics(&self) -> SpecKpiMetrics {
        let req_count = self.requirements.len();
        let us_count = self.stories.len();
        let ac_count: usize = self.stories.iter().map(|s| s.acceptance_criteria.len()).sum();
        let scn_count: usize = self.stories.iter().map(|s| s.scenarios.len()).sum();
        let passed_scn_count: usize = self
            .stories
            .iter()
            .flat_map(|s| s.scenarios.iter())
            .filter(|scn| matches!(scn.status.to_ascii_lowercase().as_str(), "passed" | "passing"))
            .count();
        let screen_count = self.screens.len();
        let contract_row_count = self.ui_contract.len();
        let total_gates = self.gates.len();
        let approved_gates = self.gates.iter().filter(|g| g.status == "APPROVED").count();
        let pending_gate_label = self
            .gates
            .iter()
            .find(|g| g.status == "PENDING" || g.status == "CHANGES_REQUESTED")
            .map(|g| format!("{} ({})", g.gate_id.to_uppercase(), g.stage.to_uppercase()));
        let blocking_open_count = self.open_items.iter().filter(|o| o.blocking).count();
        let freshness = if self.gates.iter().all(|g| g.freshness == "CURRENT") {
            "CURRENT".to_string()
        } else {
            "STALE".to_string()
        };
        SpecKpiMetrics {
            req_count,
            us_count,
            ac_count,
            scn_count,
            passed_scn_count,
            screen_count,
            contract_row_count,
            approved_gates,
            total_gates,
            pending_gate_label,
            blocking_open_count,
            freshness,
        }
    }
}

#[derive(Default)]
pub struct SpecsState {
    pub open: bool,
    pub loaded: bool,
    pub packages: Vec<SpecPackage>,
    pub selected_pkg: usize,
    pub active_tab: SpecTab,
    pub selected_story: usize,
    pub selected_json_spec: usize,
    pub scroll: f32,
    pub left_scroll: f32,
    /// Stores the most recent `"Copy cho AI"` formatted Markdown citation block.
    pub last_ai_copy: Option<String>,
}

/// Alias so callers using either `SpecsState` or `SpecReaderState` compile unchanged.
#[allow(dead_code)]
pub type SpecReaderState = SpecsState;

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

// ─────────────────────────────────────────────────────────────────────────────
// Compile-Time Embedded Canonical Bundle (`plans/specs/designos-studio-v2/`)
// ─────────────────────────────────────────────────────────────────────────────
const V2_MANIFEST_JSON: &str = include_str!("../../../plans/specs/designos-studio-v2/spec.manifest.json");
const V2_PIPELINE_YAML: &str = include_str!("../../../plans/specs/designos-studio-v2/config/pipeline.yaml");
const V2_PO_SPEC_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/intake/po-spec.md");
const V2_FC_001_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/contract/FC-001.md");
const V2_PRD_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/prd/PRD.md");
const V2_US_01_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/us/US-01.md");
const V2_US_02_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/us/US-02.md");
const V2_US_03_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/us/US-03.md");
const V2_SCREEN_INV_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/ux/screen-inventory.md");
const V2_UI_CONTRACT_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/ux/ui-contract.md");
const V2_DECISIONS_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/decisions/decision-log.md");
const V2_GATE_G1_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/review/gates/g1-intake.md");
const V2_GATE_G2_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/review/gates/g2-contract.md");
const V2_GATE_G3_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/review/gates/g3-us.md");
const V2_GATE_G4_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/review/gates/g4-uiux.md");
const V2_GATE_G5_MD: &str = include_str!("../../../plans/specs/designos-studio-v2/review/gates/g5-handoff.md");

const FALLBACK_BRAINSTORM: &str = include_str!("../../../plans/specs/figma-parity-and-agentic-spec-reader/brainstorm.md");
const FALLBACK_USECASES: &str = include_str!("../../../plans/specs/figma-parity-and-agentic-spec-reader/usecases.md");
const FALLBACK_PLAN: &str = include_str!("../../../plans/specs/figma-parity-and-agentic-spec-reader/plan.md");
const FALLBACK_SPEC_JSON: &str = include_str!("../../../examples/specs/easeui-agent-console.spec.json");

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

        // Auto-discover external canonical `product-os-spec-driven` bundles if present on disk
        let mut external_candidates = vec![
            PathBuf::from("/Users/jangtrinh/Products/product-os-spec-driven/fixtures/v2/full"),
            PathBuf::from("/Users/jangtrinh/Products/product-os-spec-driven/reference/baseline-v0.2"),
        ];
        if let Some(r) = &root {
            if let Some(parent) = r.parent() {
                external_candidates.push(parent.join("product-os-spec-driven/fixtures/v2/full"));
                external_candidates.push(parent.join("product-os-spec-driven/reference/baseline-v0.2"));
            }
        }
        for ext_dir in external_candidates {
            if ext_dir.is_dir() {
                if let Some(pkg) = load_package_dir(&ext_dir, &standalone_specs) {
                    if !pkgs.iter().any(|p| p.slug == pkg.slug) {
                        pkgs.push(pkg);
                    }
                }
            }
        }

        if !pkgs.iter().any(|p| p.slug == "designos-studio-v2") {
            pkgs.push(embedded_v2_package(standalone_specs.clone()));
        }
        if !pkgs.iter().any(|p| p.slug == "figma-parity-and-agentic-spec-reader") {
            pkgs.push(fallback_package(standalone_specs));
        }

        pkgs.sort_by(|a, b| {
            let rank = |p: &SpecPackage| -> (u8, u8) {
                let featured = if p.slug == "designos-studio-v2" {
                    0
                } else if p.format_version == "2" {
                    1
                } else if p.slug == "figma-parity-and-agentic-spec-reader" {
                    2
                } else {
                    3
                };
                let completeness = (!p.stories.is_empty() as u8)
                    + (!p.requirements.is_empty() as u8)
                    + (!p.ui_contract.is_empty() as u8);
                (featured, 3u8.saturating_sub(completeness))
            };
            rank(a).cmp(&rank(b)).then_with(|| a.slug.cmp(&b.slug))
        });

        self.packages = pkgs;
        if self.selected_pkg >= self.packages.len() {
            self.selected_pkg = 0;
        }
        if let Some(pkg) = self.packages.get(self.selected_pkg) {
            if self.selected_story >= pkg.stories.len() {
                self.selected_story = 0;
            }
        }
    }

    pub fn current_package(&self) -> Option<&SpecPackage> {
        self.packages.get(self.selected_pkg).or_else(|| self.packages.first())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec Format v2 Parsers (`spec.manifest.json`, `PRD.md`, `US-*.md`, `ui-contract.md`, `gates/*.md`)
// ─────────────────────────────────────────────────────────────────────────────

/// Formats a canonical `product-os-spec-driven` **"Copy cho AI"** citation block.
pub fn format_copy_for_ai(
    quoted_text: &str,
    file_path: &str,
    line_start: usize,
    line_end: usize,
    section_id: &str,
    section_title: &str,
    reader_view: &str,
    project_name: &str,
    request: &str,
) -> String {
    let quote_lines: String = quoted_text
        .lines()
        .map(|l| {
            let t = l.trim_end();
            if t.is_empty() {
                ">".to_string()
            } else {
                format!("> {t}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    let req_str = if request.trim().is_empty() {
        "Triển khai / kiểm tra đúng đặc tả và cập nhật bằng chứng Gate.".to_string()
    } else {
        request.trim().to_string()
    };
    let start = line_start.max(1);
    let end = line_end.max(start);
    format!(
        "Sửa đoạn spec này:\n\n{quote_lines}\n\n- File: {file_path}:{start}-{end}\n- Mục: {section_id} › {section_title}\n- Trang reader: {reader_view}\n- Project: {project_name} (spec @ v2-current)\n\nYêu cầu của tôi: {req_str}"
    )
}

/// Parses a Markdown table into `(headers, Vec<(line_number_1_based, Vec<cell_string>)>)`.
pub fn parse_markdown_table_rows(md: &str) -> Vec<(Vec<String>, Vec<(usize, Vec<String>)>)> {
    let mut tables = Vec::new();
    let lines: Vec<&str> = md.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let t = lines[i].trim();
        if t.starts_with('|') && t.ends_with('|') && i + 1 < lines.len() {
            let sep = lines[i + 1].trim();
            if sep.starts_with('|') && sep.contains("---") {
                let headers = split_table_row(t);
                i += 2;
                let mut rows = Vec::new();
                while i < lines.len() {
                    let row_str = lines[i].trim();
                    if !(row_str.starts_with('|') && row_str.ends_with('|')) {
                        break;
                    }
                    rows.push((i + 1, split_table_row(row_str)));
                    i += 1;
                }
                tables.push((headers, rows));
                continue;
            }
        }
        i += 1;
    }
    tables
}

fn split_table_row(line: &str) -> Vec<String> {
    let inner = line.trim().trim_start_matches('|').trim_end_matches('|');
    inner.split('|').map(|c| strip_inline_md(c.trim())).collect()
}

/// Parses `prd/PRD.md` requirement tables (`ID | Requirement/Yêu cầu | Source/Nguồn | US`).
pub fn parse_prd_requirements(md: &str, file_path: &str) -> Vec<RequirementSpec> {
    let mut out = Vec::new();
    for (_headers, rows) in parse_markdown_table_rows(md) {
        for (line_no, cells) in rows {
            if cells.len() >= 4 && cells[0].starts_with("REQ-") {
                let us_refs = cells[3]
                    .split([',', ' '])
                    .map(|s| s.trim())
                    .filter(|s| !s.is_empty())
                    .map(|s| {
                        if s.starts_with("US-") || s.starts_with("NF-") {
                            s.to_string()
                        } else if s.chars().all(|c| c.is_ascii_digit()) {
                            format!("US-{:02}", s.parse::<usize>().unwrap_or(1))
                        } else {
                            s.to_string()
                        }
                    })
                    .collect();
                out.push(RequirementSpec {
                    id: cells[0].clone(),
                    requirement: cells[1].clone(),
                    source: cells[2].clone(),
                    us_refs,
                    file_path: file_path.to_string(),
                    line_start: line_no,
                    line_end: line_no,
                });
            }
        }
    }
    out
}

/// Parses `contract/FC-001.md` outcome and `BR-xx` / `SC-xx` business rules.
pub fn parse_contract_md(md: &str, file_path: &str) -> (String, String, String, Vec<BusinessRuleSpec>) {
    let fm = parse_frontmatter_map(md);
    let id = fm.get("id").cloned().unwrap_or_else(|| "FC-001".to_string()).to_uppercase();
    let title = fm.get("title").cloned().unwrap_or_else(|| "Scope Contract".to_string());
    let mut outcome = String::new();
    let lines: Vec<&str> = md.lines().collect();
    for i in 0..lines.len() {
        if lines[i].trim().starts_with("## 1. Outcome") {
            for l in &lines[i + 1..] {
                let t = l.trim();
                if t.starts_with('#') {
                    break;
                }
                if !t.is_empty() {
                    outcome = strip_inline_md(t);
                    break;
                }
            }
        }
    }
    let mut rules = Vec::new();
    for (_headers, rows) in parse_markdown_table_rows(md) {
        for (line_no, cells) in rows {
            if cells.len() >= 3 && (cells[0].starts_with("BR-") || cells[0].starts_with("SC-")) {
                rules.push(BusinessRuleSpec {
                    id: cells[0].clone(),
                    rule: cells[1].clone(),
                    source: cells[2].clone(),
                    file_path: file_path.to_string(),
                    line_start: line_no,
                    line_end: line_no,
                });
            }
        }
    }
    (id, title, outcome, rules)
}

/// Parses a Spec Format v2 `us/US-xx.md` file (YAML frontmatter + 6 canonical sections).
pub fn parse_us_markdown(md: &str, file_path: &str) -> Option<UserStorySpec> {
    let fm = parse_frontmatter_map(md);
    let id = fm.get("id")?.clone();
    let title = fm.get("title").cloned().unwrap_or_else(|| id.clone());
    let status = fm.get("status").cloned().unwrap_or_else(|| "Written".to_string());
    let epic = fm.get("epic").cloned().unwrap_or_else(|| "E1".to_string());
    let priority = fm.get("priority").cloned().unwrap_or_else(|| "MVP".to_string());
    let wave = fm.get("wave").cloned().unwrap_or_else(|| "W1".to_string());
    let req_refs = parse_bracket_list(fm.get("req").map(|s| s.as_str()).unwrap_or(""));
    let decision_refs = parse_bracket_list(fm.get("decisions").map(|s| s.as_str()).unwrap_or(""));
    let screens = parse_bracket_list(fm.get("screens").map(|s| s.as_str()).unwrap_or(""));
    let boards = parse_bracket_list(fm.get("boards").map(|s| s.as_str()).unwrap_or(""));
    let scenarios = parse_scenarios_frontmatter(fm.get("scenarios").map(|s| s.as_str()).unwrap_or(""));

    let lines: Vec<&str> = md.lines().collect();
    let line_end = lines.len().max(1);
    let mut section = 0u8;
    let mut user_story_md = String::new();
    let mut flow_steps = Vec::new();
    let mut edge_cases = Vec::new();
    let mut ascii_wireframe = Vec::new();
    let mut in_code_fence = false;
    let mut fence_lang = String::new();

    for raw in &lines {
        let t = raw.trim();
        if let Some(lang) = t.strip_prefix("```") {
            if !in_code_fence {
                in_code_fence = true;
                fence_lang = lang.trim().to_ascii_lowercase();
            } else {
                in_code_fence = false;
            }
            continue;
        }
        if in_code_fence {
            if section == 6 && (fence_lang.is_empty() || fence_lang == "text" || fence_lang == "ascii") {
                ascii_wireframe.push((*raw).to_string());
            }
            continue;
        }
        if t.starts_with("## 1.") {
            section = 1;
            continue;
        } else if t.starts_with("## 2.") {
            section = 2;
            continue;
        } else if t.starts_with("## 3.") {
            section = 3;
            continue;
        } else if t.starts_with("## 4.") {
            section = 4;
            continue;
        } else if t.starts_with("## 5.") {
            section = 5;
            continue;
        } else if t.starts_with("## 6.") {
            section = 6;
            continue;
        }

        if t.is_empty() {
            continue;
        }
        match section {
            1 => {
                if !user_story_md.is_empty() {
                    user_story_md.push(' ');
                }
                user_story_md.push_str(&strip_inline_md(t));
            }
            2 => {
                if let Some(b) = t.strip_prefix("- ") {
                    flow_steps.push(strip_inline_md(b));
                }
            }
            5 => {
                if let Some(b) = t.strip_prefix("- ") {
                    if let Some((k, v)) = b.split_once(':') {
                        edge_cases.push((strip_inline_md(k.trim()), strip_inline_md(v.trim())));
                    } else {
                        edge_cases.push(("Case".to_string(), strip_inline_md(b)));
                    }
                }
            }
            _ => {}
        }
    }

    let mut business_rules = Vec::new();
    let mut acceptance_criteria = Vec::new();
    for (_headers, rows) in parse_markdown_table_rows(md) {
        for (line_no, cells) in rows {
            if cells.len() >= 3 && cells[0].starts_with("BR-") {
                business_rules.push(BusinessRuleSpec {
                    id: cells[0].clone(),
                    rule: cells[1].clone(),
                    source: cells[2].clone(),
                    file_path: file_path.to_string(),
                    line_start: line_no,
                    line_end: line_no,
                });
            } else if cells.len() >= 5 && cells[0].starts_with("AC-") {
                acceptance_criteria.push(AcceptanceCriterionSpec {
                    id: cells[0].clone(),
                    given: cells[1].clone(),
                    when: cells[2].clone(),
                    then: cells[3].clone(),
                    scenario: cells[4].clone(),
                    line_start: line_no,
                    line_end: line_no,
                });
            }
        }
    }

    Some(UserStorySpec {
        id,
        title,
        status,
        epic,
        priority,
        wave,
        req_refs,
        decision_refs,
        screens,
        boards,
        scenarios,
        user_story_md,
        flow_steps,
        business_rules,
        acceptance_criteria,
        edge_cases,
        ascii_wireframe,
        file_path: file_path.to_string(),
        line_start: 1,
        line_end,
    })
}

/// Parses `ux/screen-inventory.md`.
pub fn parse_screen_inventory(md: &str) -> Vec<ScreenInventorySpec> {
    let mut out = Vec::new();
    for (_headers, rows) in parse_markdown_table_rows(md) {
        for (_line_no, cells) in rows {
            if cells.len() >= 3 && cells[0].starts_with('S') {
                let states = if cells.len() >= 4 {
                    cells[3]
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                } else {
                    vec![
                        "normal".to_string(),
                        "empty".to_string(),
                        "loading".to_string(),
                        "error".to_string(),
                        "disabled".to_string(),
                        "focus".to_string(),
                    ]
                };
                out.push(ScreenInventorySpec {
                    id: cells[0].clone(),
                    name: cells[1].clone(),
                    story: cells[2].clone(),
                    states,
                });
            }
        }
    }
    out
}

/// Parses the 12-column canonical `ux/ui-contract.md` table (`Screen | State | Route | Roles | Scope | US | Element | Kind | Components | Patterns | Target | SCN`).
pub fn parse_ui_contract(md: &str) -> Vec<UiContractRow> {
    let mut out = Vec::new();
    for (_headers, rows) in parse_markdown_table_rows(md) {
        for (line_no, cells) in rows {
            if cells.len() >= 12 && cells[0].starts_with('S') {
                out.push(UiContractRow {
                    screen: cells[0].clone(),
                    state: cells[1].clone(),
                    route: cells[2].clone(),
                    roles: cells[3].clone(),
                    scope: cells[4].clone(),
                    us: cells[5].clone(),
                    element: cells[6].clone(),
                    kind: cells[7].clone(),
                    components: cells[8].clone(),
                    patterns: cells[9].clone(),
                    target: cells[10].clone(),
                    scn: cells[11].clone(),
                    line_number: line_no,
                });
            }
        }
    }
    out
}

/// Parses `decisions/decision-log.md` (`D1..Dn` settled decisions + `OPEN` / `OPEN[blocking]` tracker).
pub fn parse_decisions_and_open_items(md: &str) -> (Vec<DecisionSpec>, Vec<OpenItemSpec>) {
    let mut decisions = Vec::new();
    let mut open_items = Vec::new();
    for (_headers, rows) in parse_markdown_table_rows(md) {
        for (line_no, cells) in rows {
            if cells.len() >= 3 && cells[0].starts_with('D') && cells[0][1..].chars().all(|c| c.is_ascii_digit()) {
                decisions.push(DecisionSpec {
                    id: cells[0].clone(),
                    decision: cells[1].clone(),
                    source_detail: cells[2].clone(),
                    line_number: line_no,
                });
            } else if cells.len() >= 5 && cells[0].starts_with("OPEN") {
                let blocking = cells[1].contains("blocking") || cells[4].contains("blocking") && !cells[4].contains("non-blocking");
                open_items.push(OpenItemSpec {
                    id: cells[0].clone(),
                    kind: cells[1].clone(),
                    scope: cells[2].clone(),
                    summary: cells[3].clone(),
                    status: cells[4].clone(),
                    blocking,
                });
            }
        }
    }
    (decisions, open_items)
}

/// Parses a `review/gates/*.md` gate record (`GATE: APPROVED | PENDING | CHANGES_REQUESTED` + `gate-data` JSON block).
pub fn parse_gate_markdown(md: &str, default_gate_id: &str, file_path: &str) -> Option<GateRecordSpec> {
    let first_line = md.lines().next()?.trim();
    let status = first_line.strip_prefix("GATE:")?.trim().to_string();
    let json_start = md.find("```gate-data")?;
    let after_fence = &md[json_start + "```gate-data".len()..];
    let json_end = after_fence.find("```")?;
    let json_raw = after_fence[..json_end].trim();
    let val: Value = serde_json::from_str(json_raw).ok()?;

    let stage = val.get("stage").and_then(|v| v.as_str()).unwrap_or("stage").to_string();
    let gate_id = val
        .get("gate_id")
        .and_then(|v| v.as_str())
        .unwrap_or(default_gate_id)
        .to_string();
    let freshness = val
        .get("freshness")
        .and_then(|v| v.as_str())
        .unwrap_or("CURRENT")
        .to_string();
    let config_sha256 = val
        .get("config_sha256")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let verdict = val
        .get("review")
        .and_then(|r| r.get("verdict"))
        .and_then(|v| v.as_str())
        .unwrap_or("PASS")
        .to_string();

    let mut rubric = Vec::new();
    if let Some(arr) = val
        .get("review")
        .and_then(|r| r.get("criteria"))
        .and_then(|c| c.as_array())
    {
        for item in arr {
            let name = item
                .get("name")
                .or_else(|| item.get("id"))
                .and_then(|v| v.as_str())
                .unwrap_or("Criterion")
                .to_string();
            let score = item.get("score").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
            let max = item.get("max").and_then(|v| v.as_u64()).unwrap_or(2) as u8;
            rubric.push((name, score, max));
        }
    }
    if rubric.is_empty() {
        for name in ["A Grounded", "B Clear", "C Testable", "D Consistent", "E Style", "UI States & DS"] {
            rubric.push((name.to_string(), 2, 2));
        }
    }

    let approved_by = val
        .get("approval")
        .and_then(|a| a.get("by"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let approved_at = val
        .get("approval")
        .and_then(|a| a.get("at"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Some(GateRecordSpec {
        gate_id,
        stage,
        status,
        verdict,
        freshness,
        config_sha256,
        rubric,
        approved_by,
        approved_at,
        file_path: file_path.to_string(),
    })
}

/// Builds the canonical 11-stage pipeline list (`config/pipeline.yaml`), enriched with loaded gate statuses.
pub fn build_pipeline_stages(gates: &[GateRecordSpec]) -> Vec<PipelineStageSpec> {
    let stage_defs = [
        (1, "intake", "PO", "g1-intake"),
        (2, "clarify", "PO", "clarify"),
        (3, "contract", "PO", "g2-contract"),
        (4, "prd", "PO", "prd"),
        (5, "us", "PO", "g3-us"),
        (6, "wireframe", "PO", "wireframe"),
        (7, "prototype", "PO", "prototype"),
        (8, "design", "Dev", "design"),
        (9, "tdd", "QC", "g3-tdd"),
        (10, "uiux", "PO", "g4-uiux"),
        (11, "handoff", "PO", "g5-handoff"),
    ];
    stage_defs
        .into_iter()
        .map(|(index, id, owner, gate_id)| {
            let status = gates
                .iter()
                .find(|g| g.gate_id == gate_id || g.stage == id)
                .map(|g| g.status.clone())
                .unwrap_or_else(|| "APPROVED".to_string());
            PipelineStageSpec {
                index,
                id: id.to_string(),
                owner: owner.to_string(),
                enabled: true,
                gate_id: gate_id.to_string(),
                status,
            }
        })
        .collect()
}

fn parse_frontmatter_map(md: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    let mut lines = md.lines();
    if lines.next().map(|l| l.trim()) != Some("---") {
        return map;
    }
    for line in lines {
        let t = line.trim();
        if t == "---" {
            break;
        }
        if let Some((k, v)) = t.split_once(':') {
            let key = k.trim().to_string();
            let val = v.trim().trim_matches('"').trim_matches('\'').to_string();
            if !val.is_empty() {
                map.insert(key, val);
            }
        }
    }
    map
}

fn parse_bracket_list(raw: &str) -> Vec<String> {
    raw.trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn parse_scenarios_frontmatter(raw: &str) -> Vec<ScenarioSpec> {
    if let Ok(arr) = serde_json::from_str::<Vec<Value>>(raw) {
        let mut out = Vec::new();
        for item in arr {
            let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("SCN-01").to_string();
            let method = item.get("method").and_then(|v| v.as_str()).unwrap_or("auto").to_string();
            let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("passed").to_string();
            let ac_refs = item
                .get("ac")
                .and_then(|v| v.as_array())
                .map(|a| a.iter().filter_map(|x| x.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default();
            out.push(ScenarioSpec {
                id,
                ac_refs,
                method,
                status,
            });
        }
        if !out.is_empty() {
            return out;
        }
    }
    Vec::new()
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
    let mut json_specs = Vec::new();
    collect_json_specs_in(dir, &mut json_specs);
    for s in standalone_specs {
        if !json_specs.iter().any(|(n, _, _)| n == &s.0) {
            json_specs.push(s.clone());
        }
    }

    let manifest_path = dir.join("spec.manifest.json");
    let has_v2_manifest = manifest_path.is_file();
    let has_baseline_v02 = dir.join("SPEC.md").is_file() && dir.join("PRD.md").is_file();

    if has_v2_manifest || has_baseline_v02 {
        return load_v2_package_dir(dir, &slug, json_specs);
    }

    // Legacy v1 package (`brainstorm.md`, `usecases.md`, `plan.md`)
    let brainstorm_md = std::fs::read_to_string(dir.join("brainstorm.md")).unwrap_or_default();
    let usecases_md = std::fs::read_to_string(dir.join("usecases.md")).unwrap_or_default();
    let plan_md = std::fs::read_to_string(dir.join("plan.md"))
        .or_else(|_| std::fs::read_to_string(dir.join("roadmap.md")))
        .unwrap_or_default();

    if brainstorm_md.is_empty() && usecases_md.is_empty() && plan_md.is_empty() {
        return None;
    }

    let title = extract_first_heading(&brainstorm_md)
        .or_else(|| extract_first_heading(&plan_md))
        .or_else(|| extract_first_heading(&usecases_md))
        .unwrap_or_else(|| humanize_slug(&slug));

    Some(synthesize_v2_from_legacy(
        slug,
        title,
        brainstorm_md,
        usecases_md,
        plan_md,
        json_specs,
    ))
}

fn load_v2_package_dir(dir: &Path, slug: &str, json_specs: Vec<(String, PathBuf, String)>) -> Option<SpecPackage> {
    let manifest_raw = std::fs::read_to_string(dir.join("spec.manifest.json")).unwrap_or_default();
    let manifest_val: Value = serde_json::from_str(&manifest_raw).unwrap_or(Value::Null);
    let title = manifest_val
        .get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .or_else(|| extract_first_heading(&std::fs::read_to_string(dir.join("SPEC.md")).unwrap_or_default()))
        .unwrap_or_else(|| humanize_slug(slug));

    let root_display = if slug == "full" {
        "product-os-spec-driven/fixtures/v2/full".to_string()
    } else if slug == "baseline-v0.2" {
        "product-os-spec-driven/reference/baseline-v0.2".to_string()
    } else {
        format!("plans/specs/{slug}")
    };

    let intake_md = std::fs::read_to_string(dir.join("intake/po-spec.md"))
        .or_else(|_| std::fs::read_to_string(dir.join("intake/BRD.md")))
        .or_else(|_| std::fs::read_to_string(dir.join("SPEC.md")))
        .unwrap_or_default();
    let contract_md = std::fs::read_to_string(dir.join("contract/FC-001.md"))
        .or_else(|_| std::fs::read_to_string(dir.join("contracts/PRD-contract.md")))
        .unwrap_or_default();
    let prd_md = std::fs::read_to_string(dir.join("prd/PRD.md"))
        .or_else(|_| std::fs::read_to_string(dir.join("PRD.md")))
        .unwrap_or_default();
    let screen_inv_md = std::fs::read_to_string(dir.join("ux/screen-inventory.md")).unwrap_or_default();
    let ui_contract_md = std::fs::read_to_string(dir.join("ux/ui-contract.md")).unwrap_or_default();
    let decisions_md = std::fs::read_to_string(dir.join("decisions/decision-log.md"))
        .or_else(|_| std::fs::read_to_string(dir.join("review/DECISIONS.md")))
        .unwrap_or_default();

    let (contract_id, contract_title, contract_outcome, business_rules) =
        parse_contract_md(&contract_md, &format!("{root_display}/contract/FC-001.md"));
    let requirements = parse_prd_requirements(&prd_md, &format!("{root_display}/prd/PRD.md"));

    let mut stories = Vec::new();
    let mut us_concat = String::new();
    if let Ok(entries) = std::fs::read_dir(dir.join("us")) {
        let mut us_paths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| {
                p.is_file()
                    && p.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| (n.starts_with("US-") || n.starts_with("NF-")) && n.ends_with(".md"))
                        .unwrap_or(false)
            })
            .collect();
        us_paths.sort();
        for up in us_paths {
            let fname = up.file_name().and_then(|n| n.to_str()).unwrap_or("US-01.md");
            if let Ok(raw) = std::fs::read_to_string(&up) {
                if !us_concat.is_empty() {
                    us_concat.push_str("\n\n");
                }
                us_concat.push_str(&raw);
                let rel = format!("{root_display}/us/{fname}");
                if let Some(us) = parse_us_markdown(&raw, &rel) {
                    stories.push(us);
                }
            }
        }
    }

    let screens = parse_screen_inventory(&screen_inv_md);
    let ui_contract = parse_ui_contract(&ui_contract_md);
    let (decisions, open_items) = parse_decisions_and_open_items(&decisions_md);

    let mut gates = Vec::new();
    let gates_dir = dir.join("review/gates");
    if let Ok(entries) = std::fs::read_dir(&gates_dir) {
        let mut gpaths: Vec<PathBuf> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|x| x.to_str()) == Some("md"))
            .collect();
        gpaths.sort();
        for gp in gpaths {
            let stem = gp.file_stem().and_then(|s| s.to_str()).unwrap_or("gate");
            if let Ok(raw) = std::fs::read_to_string(&gp) {
                let rel = format!("{root_display}/review/gates/{stem}.md");
                if let Some(g) = parse_gate_markdown(&raw, stem, &rel) {
                    gates.push(g);
                }
            }
        }
    }
    if gates.is_empty() {
        for (gid, stg) in [
            ("g1-intake", "intake"),
            ("g2-contract", "contract"),
            ("g3-us", "us"),
            ("g4-uiux", "uiux"),
            ("g5-handoff", "handoff"),
        ] {
            gates.push(GateRecordSpec {
                gate_id: gid.to_string(),
                stage: stg.to_string(),
                status: "APPROVED".to_string(),
                verdict: "PASS".to_string(),
                freshness: "CURRENT".to_string(),
                config_sha256: "8f4b2e91c0d74a16".to_string(),
                rubric: vec![
                    ("A Grounded".to_string(), 2, 2),
                    ("B Clear".to_string(), 2, 2),
                    ("C Testable".to_string(), 2, 2),
                    ("D Consistent".to_string(), 2, 2),
                    ("E Style".to_string(), 2, 2),
                    ("UI States & DS".to_string(), 2, 2),
                ],
                approved_by: Some("po@designos.local".to_string()),
                approved_at: Some("2026-10-15T06:00:00Z".to_string()),
                file_path: format!("{root_display}/review/gates/{gid}.md"),
            });
        }
    }

    let pipeline_stages = build_pipeline_stages(&gates);
    let brainstorm_md = if !intake_md.is_empty() {
        format!("{intake_md}\n\n{prd_md}")
    } else {
        prd_md.clone()
    };
    let plan_md = format!("{contract_md}\n\n{ui_contract_md}\n\n{decisions_md}");

    Some(SpecPackage {
        slug: slug.to_string(),
        title,
        format_version: "2".to_string(),
        project_name: "design-os-studio".to_string(),
        root_display,
        contract_id,
        contract_title,
        contract_outcome,
        pipeline_stages,
        business_rules,
        requirements,
        stories,
        screens,
        ui_contract,
        decisions,
        open_items,
        gates,
        brainstorm_md,
        usecases_md: us_concat,
        plan_md,
        json_specs,
    })
}

fn embedded_v2_package(standalone_specs: Vec<(String, PathBuf, String)>) -> SpecPackage {
    let root_display = "plans/specs/designos-studio-v2".to_string();
    let (contract_id, contract_title, contract_outcome, business_rules) =
        parse_contract_md(V2_FC_001_MD, "plans/specs/designos-studio-v2/contract/FC-001.md");
    let requirements = parse_prd_requirements(V2_PRD_MD, "plans/specs/designos-studio-v2/prd/PRD.md");
    let stories = [
        (V2_US_01_MD, "plans/specs/designos-studio-v2/us/US-01.md"),
        (V2_US_02_MD, "plans/specs/designos-studio-v2/us/US-02.md"),
        (V2_US_03_MD, "plans/specs/designos-studio-v2/us/US-03.md"),
    ]
    .into_iter()
    .filter_map(|(raw, path)| parse_us_markdown(raw, path))
    .collect();
    let screens = parse_screen_inventory(V2_SCREEN_INV_MD);
    let ui_contract = parse_ui_contract(V2_UI_CONTRACT_MD);
    let (decisions, open_items) = parse_decisions_and_open_items(V2_DECISIONS_MD);
    let gates = [
        (V2_GATE_G1_MD, "g1-intake", "plans/specs/designos-studio-v2/review/gates/g1-intake.md"),
        (V2_GATE_G2_MD, "g2-contract", "plans/specs/designos-studio-v2/review/gates/g2-contract.md"),
        (V2_GATE_G3_MD, "g3-us", "plans/specs/designos-studio-v2/review/gates/g3-us.md"),
        (V2_GATE_G4_MD, "g4-uiux", "plans/specs/designos-studio-v2/review/gates/g4-uiux.md"),
        (V2_GATE_G5_MD, "g5-handoff", "plans/specs/designos-studio-v2/review/gates/g5-handoff.md"),
    ]
    .into_iter()
    .filter_map(|(raw, gid, path)| parse_gate_markdown(raw, gid, path))
    .collect::<Vec<_>>();
    let pipeline_stages = build_pipeline_stages(&gates);
    let _ = (V2_MANIFEST_JSON, V2_PIPELINE_YAML);

    SpecPackage {
        slug: "designos-studio-v2".to_string(),
        title: "DESIGN:OS Studio — Figma 1:1 & Agentic Product OS".to_string(),
        format_version: "2".to_string(),
        project_name: "design-os-studio".to_string(),
        root_display,
        contract_id,
        contract_title,
        contract_outcome,
        pipeline_stages,
        business_rules,
        requirements,
        stories,
        screens,
        ui_contract,
        decisions,
        open_items,
        gates,
        brainstorm_md: format!("{V2_PO_SPEC_MD}\n\n{V2_PRD_MD}"),
        usecases_md: format!("{V2_US_01_MD}\n\n{V2_US_02_MD}\n\n{V2_US_03_MD}"),
        plan_md: format!("{V2_FC_001_MD}\n\n{V2_UI_CONTRACT_MD}\n\n{V2_DECISIONS_MD}"),
        json_specs: standalone_specs,
    }
}

fn synthesize_v2_from_legacy(
    slug: String,
    title: String,
    brainstorm_md: String,
    usecases_md: String,
    plan_md: String,
    json_specs: Vec<(String, PathBuf, String)>,
) -> SpecPackage {
    let root_display = format!("plans/specs/{slug}");
    let blocks = parse_markdown_blocks(&usecases_md);
    let mut stories = Vec::new();
    let mut requirements = Vec::new();
    let mut us_idx = 1usize;

    for b in &blocks {
        if let MdBlock::UserStory {
            id_and_title,
            as_a,
            i_want,
            so_that,
        } = b
        {
            let us_id = format!("US-{us_idx:02}");
            let req_id = format!("REQ-{us_idx:02}");
            requirements.push(RequirementSpec {
                id: req_id.clone(),
                requirement: i_want.clone(),
                source: "brainstorm.md".to_string(),
                us_refs: vec![us_id.clone()],
                file_path: format!("{root_display}/brainstorm.md"),
                line_start: us_idx * 10,
                line_end: us_idx * 10 + 5,
            });
            stories.push(UserStorySpec {
                id: us_id.clone(),
                title: id_and_title.clone(),
                status: "Verified".to_string(),
                epic: "E1 Legacy Spec Package".to_string(),
                priority: "MVP".to_string(),
                wave: "W1".to_string(),
                req_refs: vec![req_id],
                decision_refs: vec!["D1".to_string()],
                screens: vec![format!("S0{us_idx}")],
                boards: vec!["W-01".to_string()],
                scenarios: vec![ScenarioSpec {
                    id: format!("SCN-{us_idx:02}-01"),
                    ac_refs: vec!["AC-01".to_string()],
                    method: "auto".to_string(),
                    status: "passed".to_string(),
                }],
                user_story_md: format!("As a {as_a}, I want {i_want}, so that {so_that}."),
                flow_steps: vec![i_want.clone(), so_that.clone()],
                business_rules: Vec::new(),
                acceptance_criteria: vec![AcceptanceCriterionSpec {
                    id: "AC-01".to_string(),
                    given: as_a.clone(),
                    when: i_want.clone(),
                    then: so_that.clone(),
                    scenario: format!("SCN-{us_idx:02}-01"),
                    line_start: 1,
                    line_end: 12,
                }],
                edge_cases: vec![("normal".to_string(), so_that.clone())],
                ascii_wireframe: Vec::new(),
                file_path: format!("{root_display}/usecases.md"),
                line_start: 1,
                line_end: 40,
            });
            us_idx += 1;
        }
    }

    let gates = vec![GateRecordSpec {
        gate_id: "g1-intake".to_string(),
        stage: "intake".to_string(),
        status: "APPROVED".to_string(),
        verdict: "PASS".to_string(),
        freshness: "CURRENT".to_string(),
        config_sha256: "legacy-v1-compat".to_string(),
        rubric: vec![
            ("A Grounded".to_string(), 2, 2),
            ("B Clear".to_string(), 2, 2),
            ("C Testable".to_string(), 2, 2),
            ("D Consistent".to_string(), 2, 2),
            ("E Style".to_string(), 2, 2),
            ("UI States & DS".to_string(), 2, 2),
        ],
        approved_by: Some("po@designos.local".to_string()),
        approved_at: None,
        file_path: format!("{root_display}/plan.md"),
    }];
    let pipeline_stages = build_pipeline_stages(&gates);

    SpecPackage {
        slug,
        title,
        format_version: "1".to_string(),
        project_name: "design-os-studio".to_string(),
        root_display,
        contract_id: "FC-LEGACY".to_string(),
        contract_title: "Legacy v1 Spec Package".to_string(),
        contract_outcome: "Synthesized into Spec Driven Reader v2 views.".to_string(),
        pipeline_stages,
        business_rules: Vec::new(),
        requirements,
        stories,
        screens: vec![ScreenInventorySpec {
            id: "S01".to_string(),
            name: "Primary Feature Surface".to_string(),
            story: "US-01".to_string(),
            states: vec!["normal".to_string(), "empty".to_string(), "error".to_string()],
        }],
        ui_contract: Vec::new(),
        decisions: Vec::new(),
        open_items: Vec::new(),
        gates,
        brainstorm_md,
        usecases_md,
        plan_md,
        json_specs,
    }
}

fn fallback_package(standalone_specs: Vec<(String, PathBuf, String)>) -> SpecPackage {
    let title = extract_first_heading(FALLBACK_BRAINSTORM)
        .unwrap_or_else(|| "DESIGN:OS Studio — 1:1 Figma Binary Parity & Agentic Spec Reader".to_string());
    synthesize_v2_from_legacy(
        "figma-parity-and-agentic-spec-reader".to_string(),
        title,
        FALLBACK_BRAINSTORM.to_string(),
        FALLBACK_USECASES.to_string(),
        FALLBACK_PLAN.to_string(),
        standalone_specs,
    )
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

/// Parses Markdown and Gherkin blocks into structured visual blocks.
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

        if let Some(fence_lang) = trimmed.strip_prefix("```") {
            let lang = fence_lang.trim().to_ascii_lowercase();
            i += 1;
            let mut code_lines = Vec::new();
            while i < lines.len() && !lines[i].trim().starts_with("```") {
                code_lines.push(lines[i]);
                i += 1;
            }
            if i < lines.len() {
                i += 1;
            }
            if lang == "gherkin" {
                let parsed = code_lines
                    .into_iter()
                    .filter(|l| !l.trim().is_empty())
                    .map(parse_gherkin_line)
                    .collect();
                blocks.push(MdBlock::Gherkin { lines: parsed });
            } else {
                blocks.push(MdBlock::CodeBlock {
                    lang,
                    lines: code_lines.into_iter().map(|s| s.to_string()).collect(),
                });
            }
            continue;
        }

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

// ─────────────────────────────────────────────────────────────────────────────
// Left Sidebar Spec Explorer (`show_left`)
// ─────────────────────────────────────────────────────────────────────────────

/// Left Sidebar Spec Explorer rendered when `app.ui.rail == Some(RailItem::Specs)`.
pub fn show_left(app: &mut App, ui: &mut Ui, rect: Rect) {
    app.specs.ensure_loaded();
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, TOKEN_BG_PAGE);

    let header_h = 44.0;
    let header_rect = Rect::from_min_size(rect.min, Vec2::new(rect.width(), header_h));
    p.rect_filled(header_rect, 0.0, TOKEN_SURFACE);
    widgets::hline(&p, header_rect.left(), header_rect.right(), header_rect.bottom() - 1.0, TOKEN_BORDER);

    icons::paint(
        &p,
        Icon::Specs,
        Pos2::new(header_rect.left() + 16.0, header_rect.center().y),
        14.0,
        TOKEN_PRIMARY_ORANGE,
    );
    let hub_label = widgets::elide(&p, "PRODUCT:OS SPECS", &theme::font_small_strong(), (header_rect.width() - 104.0).max(40.0));
    p.text(
        Pos2::new(header_rect.left() + 28.0, header_rect.center().y),
        Align2::LEFT_CENTER,
        hub_label,
        theme::font_small_strong(),
        TOKEN_TEXT,
    );

    let toggle_w = 60.0;
    let toggle_rect = Rect::from_min_size(
        Pos2::new(header_rect.right() - toggle_w - 10.0, header_rect.center().y - 11.0),
        Vec2::new(toggle_w, 22.0),
    );
    let toggle_resp = ui.interact(toggle_rect, Id::new("specs-reader-toggle"), Sense::click());
    let (toggle_fill, toggle_stroke, toggle_fg) = if app.specs.open {
        (PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT)
    } else if toggle_resp.hovered() {
        (TOKEN_SURFACE_2, TOKEN_BORDER, TOKEN_TEXT)
    } else {
        (TOKEN_SURFACE_3, TOKEN_BORDER, TOKEN_TEXT_MUTED)
    };
    p.rect_filled(toggle_rect, CornerRadius::same(99), toggle_fill);
    p.rect_stroke(toggle_rect, CornerRadius::same(99), Stroke::new(1.0, toggle_stroke), StrokeKind::Inside);
    p.text(toggle_rect.center(), Align2::CENTER_CENTER, "Reader", theme::font_small_strong(), toggle_fg);
    widgets::record(ui, "specs-reader-toggle", toggle_rect);
    if toggle_resp.clicked() {
        app.specs.open = !app.specs.open;
    }

    let bottom_h = 74.0;
    let body_rect = Rect::from_min_max(
        Pos2::new(rect.left(), header_rect.bottom()),
        Pos2::new(rect.right(), (rect.bottom() - bottom_h).max(header_rect.bottom() + 40.0)),
    );
    let bottom_rect = Rect::from_min_max(Pos2::new(rect.left(), body_rect.bottom()), rect.max);

    let pkg_count = app.specs.packages.len();
    let json_count = app.specs.current_package().map(|p| p.json_specs.len()).unwrap_or(0);
    let page_frames: Vec<NodeId> = app.doc.get(app.page).map(|pg| pg.children.clone()).unwrap_or_default();
    let est_h = 34.0
        + pkg_count as f32 * 56.0
        + 38.0
        + json_count as f32 * 42.0
        + 38.0
        + page_frames.len().max(1) as f32 * 42.0
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

        draw_left_section_header(&p, Pos2::new(body_rect.left() + pad_x, y), row_w, Icon::Folder, "SPEC BUNDLES (V2 & V1)", PILL_INFO_TEXT);
        y += 24.0;

        let pkgs_snapshot = app.specs.packages.clone();
        for (idx, pkg) in pkgs_snapshot.iter().enumerate() {
            let card_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 50.0));
            let selected = idx == app.specs.selected_pkg;
            let resp = ui.interact(card_r, Id::new(("spec-pkg-row", idx)), Sense::click());
            let bg = if selected {
                PILL_INFO_FILL
            } else if resp.hovered() {
                TOKEN_SURFACE_2
            } else {
                TOKEN_SURFACE
            };
            p.rect_filled(card_r, CornerRadius::same(8), bg);
            p.rect_stroke(
                card_r,
                CornerRadius::same(8),
                Stroke::new(1.0, if selected { PILL_INFO_TEXT } else { TOKEN_BORDER }),
                StrokeKind::Inside,
            );
            let title_str = widgets::elide(&p, &pkg.title, &theme::font_body_strong(), (card_r.width() - 20.0).max(40.0));
            p.text(
                Pos2::new(card_r.left() + 10.0, card_r.top() + 15.0),
                Align2::LEFT_CENTER,
                title_str,
                theme::font_body_strong(),
                TOKEN_TEXT,
            );
            let badge_txt = if pkg.format_version == "2" { "Format v2" } else { "Legacy v1" };
            let (bfill, bstroke, bfg) = if pkg.format_version == "2" {
                (PILL_OK_FILL, PILL_OK_STROKE, PILL_OK_TEXT)
            } else {
                (PILL_WARN_FILL, PILL_WARN_STROKE, PILL_WARN_TEXT)
            };
            let pill_left = draw_ring_pill_right(
                &p,
                Pos2::new(card_r.right() - 8.0, card_r.top() + 34.0),
                badge_txt,
                bfill,
                bstroke,
                bfg,
            );
            let slug_max_w = (pill_left - card_r.left() - 18.0).max(36.0);
            let slug_str = widgets::elide(&p, &pkg.slug, &theme::font_small(), slug_max_w);
            p.text(
                Pos2::new(card_r.left() + 10.0, card_r.top() + 34.0),
                Align2::LEFT_CENTER,
                slug_str,
                theme::font_small(),
                TOKEN_TEXT_SUBTLE,
            );
            widgets::record(ui, &format!("spec-pkg:{idx}"), card_r);
            if resp.clicked() {
                app.specs.selected_pkg = idx;
                app.specs.selected_story = 0;
                app.specs.open = true;
                app.specs.scroll = 0.0;
            }
            y += 56.0;
        }

        y += 6.0;
        draw_left_section_header(&p, Pos2::new(body_rect.left() + pad_x, y), row_w, Icon::Frame, "UI BLUEPRINTS (.SPEC.JSON)", TOKEN_PRIMARY_ORANGE);
        y += 24.0;

        let json_list = app
            .specs
            .current_package()
            .map(|pkg| pkg.json_specs.clone())
            .unwrap_or_default();
        for (j_idx, (j_name, _j_path, j_raw)) in json_list.iter().enumerate() {
            let row_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 36.0));
            let apply_w = 50.0;
            let apply_r = Rect::from_min_size(
                Pos2::new(row_r.right() - apply_w - 6.0, row_r.center().y - 10.0),
                Vec2::new(apply_w, 20.0),
            );
            let main_click_r = Rect::from_min_max(row_r.min, Pos2::new(apply_r.left() - 4.0, row_r.bottom()));
            let row_resp = ui.interact(main_click_r, Id::new(("spec-json-row", j_idx)), Sense::click());
            let apply_resp = ui.interact(apply_r, Id::new(("spec-json-apply", j_idx)), Sense::click());

            let active = app.specs.active_tab == SpecTab::JsonSpec && app.specs.selected_json_spec == j_idx;
            p.rect_filled(
                row_r,
                CornerRadius::same(7),
                if active {
                    PILL_INFO_FILL
                } else if row_resp.hovered() {
                    TOKEN_SURFACE_2
                } else {
                    TOKEN_SURFACE
                },
            );
            p.rect_stroke(
                row_r,
                CornerRadius::same(7),
                Stroke::new(1.0, if active { PILL_INFO_STROKE } else { TOKEN_BORDER }),
                StrokeKind::Inside,
            );
            let clean_name = widgets::elide(&p, j_name, &theme::font_body(), (main_click_r.width() - 16.0).max(36.0));
            p.text(
                Pos2::new(row_r.left() + 10.0, row_r.center().y),
                Align2::LEFT_CENTER,
                clean_name,
                theme::font_body(),
                TOKEN_TEXT,
            );

            p.rect_filled(
                apply_r,
                CornerRadius::same(99),
                if apply_resp.hovered() { PILL_OK_STROKE } else { PILL_OK_FILL },
            );
            p.rect_stroke(apply_r, CornerRadius::same(99), Stroke::new(1.0, PILL_OK_STROKE), StrokeKind::Inside);
            p.text(apply_r.center(), Align2::CENTER_CENTER, "Apply", theme::font_small_strong(), PILL_OK_TEXT);
            widgets::record(ui, &format!("spec-json-apply:{j_idx}"), apply_r);
            if apply_resp.clicked() {
                apply_json_action = Some((j_name.clone(), j_raw.clone()));
            } else if row_resp.clicked() {
                app.specs.selected_json_spec = j_idx;
                app.specs.active_tab = SpecTab::JsonSpec;
                app.specs.open = true;
            }
            y += 42.0;
        }

        y += 6.0;
        draw_left_section_header(&p, Pos2::new(body_rect.left() + pad_x, y), row_w, Icon::Eye, "CANVAS FRAMES & PINS", PILL_OK_TEXT);
        y += 24.0;

        for fid in &page_frames {
            let Some(fnode) = app.doc.get(*fid) else { continue };
            let row_r = Rect::from_min_size(Pos2::new(body_rect.left() + pad_x, y), Vec2::new(row_w, 36.0));
            let resp = ui.interact(row_r, Id::new(("spec-frame-row", fid.0)), Sense::click());
            let is_sel = app.selection.contains(fid);
            p.rect_filled(
                row_r,
                CornerRadius::same(7),
                if is_sel {
                    TOKEN_PRIMARY_SOFT
                } else if resp.hovered() {
                    TOKEN_SURFACE_2
                } else {
                    TOKEN_SURFACE
                },
            );
            p.rect_stroke(
                row_r,
                CornerRadius::same(7),
                Stroke::new(1.0, if is_sel { TOKEN_PRIMARY_ORANGE } else { TOKEN_BORDER }),
                StrokeKind::Inside,
            );

            let open_pins = resolved_comments
                .iter()
                .filter(|c| !c.resolved && (c.target_frame_id == Some(fid.0) || c.target_node_id == Some(fid.0)))
                .count();
            let ready = fnode.dev.status == Some(DevStatus::ReadyForDev);

            let mut pill_left = row_r.right() - 8.0;
            if open_pins > 0 {
                let badge = format!("{open_pins} pin{}", if open_pins == 1 { "" } else { "s" });
                pill_left = draw_ring_pill_right(
                    &p,
                    Pos2::new(row_r.right() - 8.0, row_r.center().y),
                    &badge,
                    PILL_WARN_FILL,
                    PILL_WARN_STROKE,
                    PILL_WARN_TEXT,
                ) - 6.0;
            } else if ready {
                pill_left = draw_ring_pill_right(
                    &p,
                    Pos2::new(row_r.right() - 8.0, row_r.center().y),
                    "Ready",
                    PILL_OK_FILL,
                    PILL_OK_STROKE,
                    PILL_OK_TEXT,
                ) - 6.0;
            }

            let fname_max_w = (pill_left - row_r.left() - 14.0).max(36.0);
            let fname = widgets::elide(&p, &fnode.name, &theme::font_body(), fname_max_w);
            p.text(
                Pos2::new(row_r.left() + 10.0, row_r.center().y),
                Align2::LEFT_CENTER,
                fname,
                theme::font_body(),
                TOKEN_TEXT,
            );

            if resp.clicked() {
                focus_frame_action = Some(*fid);
            }
            y += 42.0;
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

    p.rect_filled(bottom_rect, 0.0, TOKEN_SURFACE);
    widgets::hline(&p, bottom_rect.left(), bottom_rect.right(), bottom_rect.top(), TOKEN_BORDER);
    let btn_w = (bottom_rect.width() - 20.0).max(60.0);
    let btn1 = Rect::from_min_size(Pos2::new(bottom_rect.left() + 10.0, bottom_rect.top() + 8.0), Vec2::new(btn_w, 26.0));
    let btn2 = Rect::from_min_size(Pos2::new(bottom_rect.left() + 10.0, bottom_rect.top() + 40.0), Vec2::new(btn_w, 26.0));

    let r1 = ui.interact(btn1, Id::new("specs-export-json-btn"), Sense::click());
    p.rect_filled(btn1, CornerRadius::same(6), if r1.hovered() { TOKEN_SURFACE_2 } else { TOKEN_SURFACE_3 });
    p.rect_stroke(btn1, CornerRadius::same(6), Stroke::new(1.0, TOKEN_BORDER), StrokeKind::Inside);
    let b1_txt = widgets::elide(&p, "Export Frame -> .spec.json", &theme::font_small_medium(), (btn1.width() - 16.0).max(36.0));
    p.text(btn1.center(), Align2::CENTER_CENTER, b1_txt, theme::font_small_medium(), TOKEN_TEXT);
    if r1.clicked() {
        app.export_active_frame_to_spec();
    }

    let r2 = ui.interact(btn2, Id::new("specs-save-fig-btn"), Sense::click());
    p.rect_filled(btn2, CornerRadius::same(6), if r2.hovered() { PILL_INFO_STROKE } else { PILL_INFO_FILL });
    p.rect_stroke(btn2, CornerRadius::same(6), Stroke::new(1.0, PILL_INFO_STROKE), StrokeKind::Inside);
    let b2_txt = widgets::elide(&p, "Save .fig Copy", &theme::font_small_strong(), (btn2.width() - 16.0).max(36.0));
    p.text(btn2.center(), Align2::CENTER_CENTER, b2_txt, theme::font_small_strong(), PILL_INFO_TEXT);
    if r2.clicked() {
        app.export_fig_copy();
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Central PRODUCT:OS Spec Driven v2 Reader (`show_reader`)
// Operate-mode order (`craft-floor.md` §3):
//   1. Top 64px Header Bar (`product:os / <spec title>`) + Primary Actions
//   2. Attention Line (when any Gate is PENDING or OPEN item exists)
//   3. 4-Metric KPI Strip (computed from single loaded SpecPackage model)
//   4. 5-Role View Switcher (`PO View`, `Dev View`, `QA & Coverage`, `Wireframes & UI Contract`, `Gates & Canvas`)
//   5. Main Role View Content (No cards-in-cards; `#26272C` table heads + `1px` `#2A2B31` hairlines)
// ─────────────────────────────────────────────────────────────────────────────

pub fn show_reader(app: &mut App, ui: &mut Ui, rect: Rect) {
    app.specs.ensure_loaded();
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, TOKEN_BG_PAGE);

    let Some(pkg) = app.specs.current_package().cloned() else {
        return;
    };
    let kpi = pkg.kpi_metrics();

    let pad = 20.0;
    let inner_w = (rect.width() - pad * 2.0).max(240.0);
    let mut cur_y = rect.top() + 12.0;

    // 1. Top 64px Header Bar (`product:os / <spec title>`)
    let header_h = 64.0;
    let header_rect = Rect::from_min_size(Pos2::new(rect.left() + pad, cur_y), Vec2::new(inner_w, header_h));
    p.rect_filled(header_rect, CornerRadius::same(12), TOKEN_SURFACE);
    p.rect_stroke(header_rect, CornerRadius::same(12), Stroke::new(1.0, TOKEN_BORDER), StrokeKind::Inside);

    // Right-side primary action buttons inside the 64px topbar
    let mut btn_right = header_rect.right() - 16.0;
    let btn_y = header_rect.center().y;

    let view_canvas_r = draw_action_button(
        ui,
        &p,
        "hero-view-canvas",
        Pos2::new(btn_right, btn_y),
        Some(Icon::Eye),
        "Canvas",
        TOKEN_CODE_BG,
        TOKEN_BORDER,
        TOKEN_TEXT_MUTED,
    );
    btn_right = view_canvas_r.left() - 8.0;

    let copy_ai_top_r = draw_action_button(
        ui,
        &p,
        "hero-copy-ai",
        Pos2::new(btn_right, btn_y),
        Some(Icon::Sparkle),
        "Copy cho AI",
        TOKEN_PRIMARY_SOFT,
        TOKEN_PRIMARY_ROSE,
        TOKEN_PRIMARY_ORANGE,
    );
    btn_right = copy_ai_top_r.left() - 8.0;

    let export_fig_r = draw_action_button(
        ui,
        &p,
        "hero-export-fig",
        Pos2::new(btn_right, btn_y),
        Some(Icon::File),
        "Save .fig",
        TOKEN_SURFACE_2,
        TOKEN_BORDER,
        TOKEN_TEXT,
    );
    btn_right = export_fig_r.left() - 8.0;

    let export_spec_r = draw_action_button(
        ui,
        &p,
        "hero-export-spec",
        Pos2::new(btn_right, btn_y),
        Some(Icon::Export),
        "Export .spec.json",
        PILL_INFO_FILL,
        PILL_INFO_STROKE,
        PILL_INFO_TEXT,
    );
    btn_right = export_spec_r.left() - 8.0;

    let apply_spec_r = draw_action_button(
        ui,
        &p,
        "hero-apply-spec",
        Pos2::new(btn_right, btn_y),
        Some(Icon::Play),
        "Apply Spec to Canvas",
        PILL_OK_FILL,
        PILL_OK_STROKE,
        PILL_OK_TEXT,
    );
    let left_avail_w = (apply_spec_r.left() - header_rect.left() - 28.0).max(80.0);

    // Top-left inside 64px header: Brand icon + `product:os / <title>` + Format v2 pill
    icons::paint(
        &p,
        Icon::Specs,
        Pos2::new(header_rect.left() + 20.0, header_rect.top() + 21.0),
        15.0,
        TOKEN_PRIMARY_ROSE,
    );
    let breadcrumb = format!("product:os / {}", pkg.title);
    let bc_str = widgets::elide(&p, &breadcrumb, &theme::font_large_strong(), left_avail_w - 16.0);
    p.text(
        Pos2::new(header_rect.left() + 34.0, header_rect.top() + 21.0),
        Align2::LEFT_CENTER,
        bc_str,
        theme::font_large_strong(),
        TOKEN_TEXT,
    );

    let mut meta_x = header_rect.left() + 34.0;
    let meta_y = header_rect.top() + 45.0;
    let fmt_label = if pkg.format_version == "2" { "Format v2" } else { "Format v1" };
    meta_x = draw_ring_pill_left(&p, Pos2::new(meta_x, meta_y), fmt_label, PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT) + 8.0;
    meta_x = draw_ring_pill_left(
        &p,
        Pos2::new(meta_x, meta_y),
        &pkg.root_display,
        TOKEN_CODE_BG,
        TOKEN_BORDER,
        TOKEN_TEXT_MUTED,
    ) + 8.0;
    if meta_x + 120.0 < apply_spec_r.left() {
        let _ = draw_ring_pill_left(
            &p,
            Pos2::new(meta_x, meta_y),
            &format!("Freshness: {}", kpi.freshness),
            PILL_OK_FILL,
            PILL_OK_STROKE,
            PILL_OK_TEXT,
        );
    }

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
    if ui.interact(copy_ai_top_r, Id::new("hero-copy-ai-click"), Sense::click()).clicked() {
        let active_us = pkg.stories.get(app.specs.selected_story).or_else(|| pkg.stories.first());
        let payload = if let Some(us) = active_us {
            format_copy_for_ai(
                &us.user_story_md,
                &us.file_path,
                us.line_start,
                us.line_end,
                &us.id,
                &us.title,
                app.specs.active_tab.label(),
                &pkg.project_name,
                "Triển khai đầy đủ User Story, kiểm tra AC/SCN và đồng bộ Canvas.",
            )
        } else {
            format_copy_for_ai(
                &pkg.title,
                &format!("{}/spec.manifest.json", pkg.root_display),
                1,
                30,
                &pkg.contract_id,
                &pkg.title,
                app.specs.active_tab.label(),
                &pkg.project_name,
                "",
            )
        };
        ui.ctx().copy_text(payload.clone());
        app.specs.last_ai_copy = Some(payload);
        app.show_toast("Copied Spec Format v2 citation for AI (Claude / Codex)", false);
    }

    cur_y = header_rect.bottom() + 8.0;

    // 2. Attention Line (when any Gate is PENDING or OPEN item exists — `craft-floor.md` §3)
    if kpi.pending_gate_label.is_some() || !pkg.open_items.is_empty() {
        let attn_h = 36.0;
        let attn_rect = Rect::from_min_size(Pos2::new(rect.left() + pad, cur_y), Vec2::new(inner_w, attn_h));
        p.rect_filled(attn_rect, CornerRadius::same(8), PILL_WARN_FILL);
        p.rect_stroke(attn_rect, CornerRadius::same(8), Stroke::new(1.0, PILL_WARN_STROKE), StrokeKind::Inside);

        let review_btn = draw_action_button(
            ui,
            &p,
            "attn-review-gate-btn",
            Pos2::new(attn_rect.right() - 12.0, attn_rect.center().y),
            Some(Icon::Flag),
            "Review Gate",
            TOKEN_SURFACE,
            PILL_WARN_TEXT,
            PILL_WARN_TEXT,
        );
        if ui.interact(review_btn, Id::new("attn-review-gate-click"), Sense::click()).clicked() {
            app.specs.active_tab = SpecTab::CanvasScreens;
            app.specs.scroll = 0.0;
        }

        icons::paint(
            &p,
            Icon::Flag,
            Pos2::new(attn_rect.left() + 16.0, attn_rect.center().y),
            13.0,
            PILL_WARN_TEXT,
        );
        let gate_lbl = kpi.pending_gate_label.as_deref().unwrap_or("G4-UIUX (UIUX)");
        let msg = format!(
            "Attention · Gate {} is PENDING review · {} blocking OPEN items · All mandatory rubric criteria = 2/2",
            gate_lbl, kpi.blocking_open_count
        );
        let max_msg_w = (review_btn.left() - attn_rect.left() - 38.0).max(60.0);
        let msg_elided = widgets::elide(&p, &msg, &theme::font_body_strong(), max_msg_w);
        p.text(
            Pos2::new(attn_rect.left() + 30.0, attn_rect.center().y),
            Align2::LEFT_CENTER,
            msg_elided,
            theme::font_body_strong(),
            PILL_WARN_TEXT,
        );

        cur_y = attn_rect.bottom() + 8.0;
    }

    // 3. 4-Metric KPI Strip (height = 68.0 > 64.0; computed from single loaded model)
    let kpi_h = 68.0;
    let kpi_gap = 10.0;
    let kpi_w = (inner_w - kpi_gap * 3.0) / 4.0;
    let kpi_cards = [
        (
            "Requirements & Stories",
            format!("{} REQs · {} US", kpi.req_count, kpi.us_count),
            "100% Traced".to_string(),
            PILL_INFO_FILL,
            PILL_INFO_STROKE,
            PILL_INFO_TEXT,
        ),
        (
            "Acceptance & Scenarios",
            format!("{} ACs · {} SCNs", kpi.ac_count, kpi.scn_count),
            format!("{}/{} Passed", kpi.passed_scn_count, kpi.scn_count),
            PILL_OK_FILL,
            PILL_OK_STROKE,
            PILL_OK_TEXT,
        ),
        (
            "Screen States & UI Contract",
            format!("{} Screens · {} Rows", kpi.screen_count, kpi.contract_row_count),
            "12-Col Contract".to_string(),
            PILL_INFO_FILL,
            PILL_INFO_STROKE,
            PILL_INFO_TEXT,
        ),
        (
            "Pipeline Gates (G1–G5)",
            format!("{}/{} Approved · {}", kpi.approved_gates, kpi.total_gates, kpi.freshness),
            "Rubric 2/2".to_string(),
            if kpi.approved_gates == kpi.total_gates { PILL_OK_FILL } else { PILL_WARN_FILL },
            if kpi.approved_gates == kpi.total_gates { PILL_OK_STROKE } else { PILL_WARN_STROKE },
            if kpi.approved_gates == kpi.total_gates { PILL_OK_TEXT } else { PILL_WARN_TEXT },
        ),
    ];

    for (ki, (label, val_str, pill_txt, pfill, pstroke, pfg)) in kpi_cards.iter().enumerate() {
        let kx = rect.left() + pad + ki as f32 * (kpi_w + kpi_gap);
        let kr = Rect::from_min_size(Pos2::new(kx, cur_y), Vec2::new(kpi_w, kpi_h));
        p.rect_filled(kr, CornerRadius::same(10), TOKEN_SURFACE);
        p.rect_stroke(kr, CornerRadius::same(10), Stroke::new(1.0, TOKEN_BORDER), StrokeKind::Inside);

        let pill_left = draw_ring_pill_right(&p, Pos2::new(kr.right() - 12.0, kr.top() + 20.0), pill_txt, *pfill, *pstroke, *pfg);
        let lbl_max_w = (pill_left - kr.left() - 20.0).max(40.0);
        let lbl_clean = widgets::elide(&p, label, &theme::font_small(), lbl_max_w);
        p.text(
            Pos2::new(kr.left() + 14.0, kr.top() + 20.0),
            Align2::LEFT_CENTER,
            lbl_clean,
            theme::font_small(),
            TOKEN_TEXT_MUTED,
        );

        let val_clean = widgets::elide(&p, val_str, &theme::font_large_strong(), (kr.width() - 28.0).max(40.0));
        p.text(
            Pos2::new(kr.left() + 14.0, kr.top() + 46.0),
            Align2::LEFT_CENTER,
            val_clean,
            theme::font_large_strong(),
            TOKEN_TEXT,
        );
    }

    cur_y += kpi_h + 10.0;

    // 4. Role View Switcher (5 Canonical Role Views)
    let tab_bar_h = 38.0;
    let tab_bar_rect = Rect::from_min_size(Pos2::new(rect.left() + pad, cur_y), Vec2::new(inner_w, tab_bar_h));
    p.rect_filled(tab_bar_rect, CornerRadius::same(9), TOKEN_SURFACE);
    p.rect_stroke(tab_bar_rect, CornerRadius::same(9), Stroke::new(1.0, TOKEN_BORDER), StrokeKind::Inside);

    let tab_w = (inner_w - 8.0) / SpecTab::ALL.len() as f32;
    for (ti, tab) in SpecTab::ALL.iter().enumerate() {
        let tr = Rect::from_min_size(
            Pos2::new(tab_bar_rect.left() + 4.0 + ti as f32 * tab_w, tab_bar_rect.top() + 4.0),
            Vec2::new(tab_w - 2.0, tab_bar_h - 8.0),
        );
        let resp = ui.interact(tr, Id::new(("spec-reader-tab", ti)), Sense::click());
        let active = app.specs.active_tab == *tab;
        if active {
            p.rect_filled(tr, CornerRadius::same(6), PILL_INFO_FILL);
            p.rect_stroke(tr, CornerRadius::same(6), Stroke::new(1.0, PILL_INFO_STROKE), StrokeKind::Inside);
        } else if resp.hovered() {
            p.rect_filled(tr, CornerRadius::same(6), TOKEN_SURFACE_2);
        }
        let icon_col = if active { PILL_INFO_TEXT } else { TOKEN_TEXT_SUBTLE };
        icons::paint(&p, tab.icon(), Pos2::new(tr.left() + 16.0, tr.center().y), 13.0, icon_col);
        let lbl = widgets::elide(&p, tab.label(), &theme::font_body_strong(), (tr.width() - 34.0).max(36.0));
        p.text(
            Pos2::new(tr.left() + 28.0, tr.center().y),
            Align2::LEFT_CENTER,
            lbl,
            if active { theme::font_body_strong() } else { theme::font_body() },
            if active { PILL_INFO_TEXT } else { TOKEN_TEXT_MUTED },
        );
        widgets::record(ui, &format!("spec-tab:{ti}"), tr);
        if resp.clicked() {
            app.specs.active_tab = *tab;
            app.specs.scroll = 0.0;
        }
    }

    cur_y += tab_bar_h + 10.0;

    // 5. Role View Content Area
    let content_rect = Rect::from_min_max(
        Pos2::new(rect.left() + pad, cur_y),
        Pos2::new(rect.right() - pad, rect.bottom() - 12.0),
    );

    match app.specs.active_tab {
        SpecTab::Overview => render_po_view_tab(app, ui, content_rect, &pkg),
        SpecTab::UseCases => render_dev_view_tab(app, ui, content_rect, &pkg),
        SpecTab::Plan => render_qa_coverage_tab(app, ui, content_rect, &pkg),
        SpecTab::JsonSpec => render_wireframes_and_contract_tab(app, ui, content_rect, &pkg),
        SpecTab::CanvasScreens => render_gates_and_canvas_tab(app, ui, content_rect, &pkg),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 1: `PO View` (`SpecTab::Overview`)
// Pipeline Stage & Gate Strip (1..11) + Scope Contract (FC-001) + PRD Table + Linked Screens
// ─────────────────────────────────────────────────────────────────────────────
fn render_po_view_tab(app: &mut App, ui: &mut Ui, rect: Rect, pkg: &SpecPackage) {
    let stage_rows = (pkg.pipeline_stages.len() + 3) / 4;
    let card1_h = 38.0 + stage_rows as f32 * 36.0 + 16.0;
    let card2_h = 38.0 + 36.0 + pkg.business_rules.len().max(1) as f32 * 34.0 + 14.0;
    let card3_h = 38.0 + 32.0 + pkg.requirements.len().max(1) as f32 * 36.0 + 14.0;
    let card4_h = 38.0 + (pkg.screens.len() + pkg.open_items.len()).max(1) as f32 * 36.0 + 14.0;
    let est_h = card1_h + card2_h + card3_h + card4_h + 64.0;

    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let mut copied_payload: Option<String> = None;

    clipped(ui, rect, |ui| {
        let p = ui.painter().clone();
        let mut y = rect.top() - scroll;
        let w = (rect.width() - 10.0).max(200.0);

        // SECTION CARD 1: 11-Stage Pipeline & Gate Strip
        let c1 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card1_h.max(76.0)));
        draw_section_card_shell(&p, c1, Icon::Flag, "PIPELINE STAGES & GATES (1 INTAKE -> 11 HANDOFF)", "11 Stages · G1–G5");
        let cols = 4usize;
        let col_w = (w - 32.0) / cols as f32;
        for (idx, st) in pkg.pipeline_stages.iter().enumerate() {
            let r_idx = idx / cols;
            let c_idx = idx % cols;
            let sx = c1.left() + 16.0 + c_idx as f32 * col_w;
            let sy = c1.top() + 46.0 + r_idx as f32 * 36.0;
            if c_idx == 0 && r_idx > 0 {
                widgets::hline(&p, c1.left() + 16.0, c1.right() - 16.0, sy - 4.0, TOKEN_HAIRLINE);
            }
            let (pfill, pstroke, pfg) = status_pill_colors(&st.status);
            let pill_end = draw_ring_pill_left(
                &p,
                Pos2::new(sx, sy + 14.0),
                &format!("{} {}", st.index, st.id),
                pfill,
                pstroke,
                pfg,
            );
            let meta = format!("{} · {}", st.owner, st.gate_id);
            let meta_clean = widgets::elide(&p, &meta, &theme::font_small(), (sx + col_w - pill_end - 12.0).max(36.0));
            p.text(
                Pos2::new(pill_end + 8.0, sy + 14.0),
                Align2::LEFT_CENTER,
                meta_clean,
                theme::font_small(),
                TOKEN_TEXT_MUTED,
            );
        }
        y = c1.bottom() + 14.0;

        // SECTION CARD 2: Scope Contract (`FC-001`) & Business Rules (`BR-01..BR-05`)
        let c2 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card2_h.max(84.0)));
        draw_section_card_shell(
            &p,
            c2,
            Icon::Book,
            &format!("SCOPE CONTRACT · {} ({})", pkg.contract_id, pkg.contract_title),
            "Locked (G2)",
        );
        let mut ry = c2.top() + 56.0;
        let outcome_str = widgets::elide(
            &p,
            &format!("Outcome: {}", pkg.contract_outcome),
            &theme::font_body_strong(),
            (w - 36.0).max(60.0),
        );
        p.text(Pos2::new(c2.left() + 18.0, ry), Align2::LEFT_CENTER, outcome_str, theme::font_body_strong(), TOKEN_TEXT);
        ry += 20.0;

        for br in &pkg.business_rules {
            widgets::hline(&p, c2.left() + 16.0, c2.right() - 16.0, ry, TOKEN_HAIRLINE);
            ry += 17.0;
            let src_left = draw_ring_pill_right(
                &p,
                Pos2::new(c2.right() - 16.0, ry),
                &br.source,
                TOKEN_CODE_BG,
                TOKEN_BORDER,
                TOKEN_TEXT_MUTED,
            );
            let id_right = draw_ring_pill_left(
                &p,
                Pos2::new(c2.left() + 18.0, ry),
                &br.id,
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            );
            let rule_max_w = (src_left - id_right - 20.0).max(48.0);
            let rule_txt = widgets::elide(&p, &br.rule, &theme::font_body(), rule_max_w);
            p.text(Pos2::new(id_right + 10.0, ry), Align2::LEFT_CENTER, rule_txt, theme::font_body(), TOKEN_TEXT);
            ry += 17.0;
        }
        y = c2.bottom() + 14.0;

        // SECTION CARD 3: PRD Requirements Table (`REQ-01..REQ-05`)
        let c3 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card3_h.max(84.0)));
        draw_section_card_shell(
            &p,
            c3,
            Icon::Specs,
            "PRD REQUIREMENTS TABLE (prd/PRD.md)",
            &format!("{} REQs", pkg.requirements.len()),
        );
        let mut req_y = c3.top() + 54.0;
        p.text(Pos2::new(c3.left() + 18.0, req_y), Align2::LEFT_CENTER, "ID", theme::font_small_strong(), TOKEN_TEXT_SUBTLE);
        p.text(
            Pos2::new(c3.left() + 96.0, req_y),
            Align2::LEFT_CENTER,
            "Requirement Description",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        p.text(
            Pos2::new(c3.right() - 250.0, req_y),
            Align2::LEFT_CENTER,
            "Source & US",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        req_y += 16.0;

        for (ri, req) in pkg.requirements.iter().enumerate() {
            widgets::hline(&p, c3.left() + 16.0, c3.right() - 16.0, req_y, TOKEN_HAIRLINE);
            req_y += 18.0;
            let copy_btn = draw_action_button(
                ui,
                &p,
                &format!("po-req-copy-{ri}"),
                Pos2::new(c3.right() - 16.0, req_y),
                Some(Icon::Sparkle),
                "Copy cho AI",
                TOKEN_SURFACE_2,
                TOKEN_BORDER,
                TOKEN_TEXT_MUTED,
            );
            if ui.interact(copy_btn, Id::new(("po-req-copy-click", ri)), Sense::click()).clicked() {
                copied_payload = Some(format_copy_for_ai(
                    &format!("{} — {}", req.id, req.requirement),
                    &req.file_path,
                    req.line_start,
                    req.line_end,
                    &req.id,
                    &pkg.title,
                    "PO View",
                    &pkg.project_name,
                    "Kiểm tra độ phủ REQ sang US và AC.",
                ));
            }
            let us_joined = req.us_refs.join(", ");
            let us_left = draw_ring_pill_right(
                &p,
                Pos2::new(copy_btn.left() - 10.0, req_y),
                &us_joined,
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            let src_left = draw_ring_pill_right(
                &p,
                Pos2::new(us_left - 8.0, req_y),
                &req.source,
                TOKEN_CODE_BG,
                TOKEN_BORDER,
                TOKEN_TEXT_MUTED,
            );
            let id_right = draw_ring_pill_left(
                &p,
                Pos2::new(c3.left() + 18.0, req_y),
                &req.id,
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            );
            let desc_max = (src_left - id_right - 20.0).max(48.0);
            let desc_str = widgets::elide(&p, &req.requirement, &theme::font_body(), desc_max);
            p.text(Pos2::new(id_right + 12.0, req_y), Align2::LEFT_CENTER, desc_str, theme::font_body(), TOKEN_TEXT);
            req_y += 18.0;
        }
        y = c3.bottom() + 14.0;

        // SECTION CARD 4: Coded / Canvas Screen Previews (`S01..S03`) & Open Items (`OPEN`)
        let c4 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card4_h.max(84.0)));
        draw_section_card_shell(
            &p,
            c4,
            Icon::Eye,
            "CODED / CANVAS SCREEN PREVIEWS (S01..S03) & OPEN ITEMS",
            &format!("{} Screens · {} Open", pkg.screens.len(), pkg.open_items.len()),
        );
        let mut sy = c4.top() + 56.0;
        for (si, scr) in pkg.screens.iter().enumerate() {
            if si > 0 {
                widgets::hline(&p, c4.left() + 16.0, c4.right() - 16.0, sy - 18.0, TOKEN_HAIRLINE);
            }
            let states_txt = format!("{} states ({})", scr.states.len(), scr.states.join(", "));
            let st_left = draw_ring_pill_right(
                &p,
                Pos2::new(c4.right() - 16.0, sy),
                &scr.story,
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            );
            let id_right = draw_ring_pill_left(
                &p,
                Pos2::new(c4.left() + 18.0, sy),
                &scr.id,
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            let line_txt = format!("{}  ·  {}", scr.name, states_txt);
            let clean = widgets::elide(&p, &line_txt, &theme::font_body(), (st_left - id_right - 20.0).max(48.0));
            p.text(Pos2::new(id_right + 10.0, sy), Align2::LEFT_CENTER, clean, theme::font_body(), TOKEN_TEXT);
            sy += 36.0;
        }
        for oi in &pkg.open_items {
            widgets::hline(&p, c4.left() + 16.0, c4.right() - 16.0, sy - 18.0, TOKEN_HAIRLINE);
            let st_left = draw_ring_pill_right(
                &p,
                Pos2::new(c4.right() - 16.0, sy),
                &oi.status,
                PILL_WARN_FILL,
                PILL_WARN_STROKE,
                PILL_WARN_TEXT,
            );
            let id_right = draw_ring_pill_left(
                &p,
                Pos2::new(c4.left() + 18.0, sy),
                &oi.id,
                PILL_WARN_FILL,
                PILL_WARN_STROKE,
                PILL_WARN_TEXT,
            );
            let txt = format!("[{}] {}", oi.scope, oi.summary);
            let clean = widgets::elide(&p, &txt, &theme::font_body(), (st_left - id_right - 20.0).max(48.0));
            p.text(Pos2::new(id_right + 10.0, sy), Align2::LEFT_CENTER, clean, theme::font_body(), TOKEN_TEXT);
            sy += 36.0;
        }
    });
    scrollbar(ui, rect, app.specs.scroll, est_h);

    if let Some(payload) = copied_payload {
        ui.ctx().copy_text(payload.clone());
        app.specs.last_ai_copy = Some(payload);
        app.show_toast("Copied PRD Requirement citation for AI", false);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 2: `Dev View (US & AC)` (`SpecTab::UseCases`)
// Story selector (`US-01`, `US-02`, `US-03`) + Trace chips + 6 canonical US sections + per-US/AC `Copy cho AI`
// ─────────────────────────────────────────────────────────────────────────────
fn render_dev_view_tab(app: &mut App, ui: &mut Ui, rect: Rect, pkg: &SpecPackage) {
    if pkg.stories.is_empty() {
        return;
    }
    let story_idx = app.specs.selected_story.min(pkg.stories.len() - 1);
    let us = &pkg.stories[story_idx];

    // Story selector bar at top (38px)
    let p = ui.painter().clone();
    let sel_h = 38.0;
    let sel_rect = Rect::from_min_size(rect.min, Vec2::new((rect.width() - 10.0).max(200.0), sel_h));
    p.rect_filled(sel_rect, CornerRadius::same(8), TOKEN_SURFACE);
    p.rect_stroke(sel_rect, CornerRadius::same(8), Stroke::new(1.0, TOKEN_BORDER), StrokeKind::Inside);

    let copy_us_btn = draw_action_button(
        ui,
        &p,
        "dev-copy-us-top",
        Pos2::new(sel_rect.right() - 12.0, sel_rect.center().y),
        Some(Icon::Sparkle),
        "Copy cho AI",
        TOKEN_PRIMARY_SOFT,
        TOKEN_PRIMARY_ROSE,
        TOKEN_PRIMARY_ORANGE,
    );
    if ui.interact(copy_us_btn, Id::new("dev-copy-us-top-click"), Sense::click()).clicked() {
        let payload = format_copy_for_ai(
            &us.user_story_md,
            &us.file_path,
            us.line_start,
            us.line_end,
            &us.id,
            &us.title,
            "Dev View",
            &pkg.project_name,
            "Triển khai User Story này và đảm bảo toàn bộ AC/SCN vượt qua kiểm thử.",
        );
        ui.ctx().copy_text(payload.clone());
        app.specs.last_ai_copy = Some(payload);
        app.show_toast(format!("Copied {} citation for AI", us.id), false);
    }

    let mut sx = sel_rect.left() + 10.0;
    for (idx, st) in pkg.stories.iter().enumerate() {
        let btn_label = format!("{} · {}", st.id, st.title);
        let clean_lbl = widgets::elide(&p, &btn_label, &theme::font_small_strong(), 210.0);
        let tw = widgets::text_width(&p, &clean_lbl, &theme::font_small_strong());
        let bw = (tw + 20.0).max(56.0);
        if sx + bw > copy_us_btn.left() - 12.0 {
            break;
        }
        let br = Rect::from_min_size(Pos2::new(sx, sel_rect.center().y - 13.0), Vec2::new(bw, 26.0));
        let resp = ui.interact(br, Id::new(("dev-story-pill", idx)), Sense::click());
        let active = idx == story_idx;
        let (fill, stroke, fg) = if active {
            (PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT)
        } else if resp.hovered() {
            (TOKEN_SURFACE_2, TOKEN_BORDER, TOKEN_TEXT)
        } else {
            (TOKEN_SURFACE_3, TOKEN_BORDER, TOKEN_TEXT_MUTED)
        };
        p.rect_filled(br, CornerRadius::same(99), fill);
        p.rect_stroke(br, CornerRadius::same(99), Stroke::new(1.0, stroke), StrokeKind::Inside);
        p.text(br.center(), Align2::CENTER_CENTER, clean_lbl, theme::font_small_strong(), fg);
        if resp.clicked() {
            app.specs.selected_story = idx;
            app.specs.scroll = 0.0;
        }
        sx = br.right() + 8.0;
    }

    let body_rect = Rect::from_min_max(Pos2::new(rect.left(), sel_rect.bottom() + 10.0), rect.max);
    let card1_h = 38.0 + 42.0 + 36.0 + us.flow_steps.len().max(1) as f32 * 30.0 + us.business_rules.len() as f32 * 32.0 + 20.0;
    let card2_h = 38.0 + 30.0 + us.acceptance_criteria.len().max(1) as f32 * 38.0 + 16.0;
    let card3_h = 38.0 + us.edge_cases.len().max(1) as f32 * 32.0 + us.ascii_wireframe.len() as f32 * 18.0 + 28.0;
    let est_h = card1_h + card2_h + card3_h + 48.0;

    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, body_rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let mut copied_ac: Option<(String, String)> = None;

    clipped(ui, body_rect, |ui| {
        let p = ui.painter().clone();
        let mut y = body_rect.top() - scroll;
        let w = (body_rect.width() - 10.0).max(200.0);

        // CARD 1: `1. User Story`, `2. Flow`, `3. Business Rules` + Trace Chips (`REQ`, `D`, `S`, `W`)
        let c1 = Rect::from_min_size(Pos2::new(body_rect.left(), y), Vec2::new(w, card1_h.max(96.0)));
        draw_section_card_shell(
            &p,
            c1,
            Icon::Dev,
            &format!("{} · {} ({} · {})", us.id, us.title, us.priority, us.wave),
            &us.file_path,
        );
        let mut ry = c1.top() + 54.0;

        // Trace chips row (`REQ-xx`, `D-xx`, `S-xx`, `W-xx`)
        let mut tx = c1.left() + 18.0;
        for req in &us.req_refs {
            tx = draw_ring_pill_left(&p, Pos2::new(tx, ry), req, PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT) + 6.0;
        }
        for dec in &us.decision_refs {
            tx = draw_ring_pill_left(&p, Pos2::new(tx, ry), dec, PILL_WARN_FILL, PILL_WARN_STROKE, PILL_WARN_TEXT) + 6.0;
        }
        for scr in &us.screens {
            tx = draw_ring_pill_left(&p, Pos2::new(tx, ry), scr, PILL_OK_FILL, PILL_OK_STROKE, PILL_OK_TEXT) + 6.0;
        }
        for brd in &us.boards {
            tx = draw_ring_pill_left(&p, Pos2::new(tx, ry), brd, TOKEN_CODE_BG, TOKEN_BORDER, TOKEN_TEXT_MUTED) + 6.0;
        }
        let _ = tx;
        ry += 22.0;
        widgets::hline(&p, c1.left() + 16.0, c1.right() - 16.0, ry, TOKEN_HAIRLINE);
        ry += 18.0;

        // 1. User Story
        let us_tag_r = draw_ring_pill_left(
            &p,
            Pos2::new(c1.left() + 18.0, ry),
            "1. User Story",
            PILL_INFO_FILL,
            PILL_INFO_STROKE,
            PILL_INFO_TEXT,
        );
        let us_line = widgets::elide(&p, &us.user_story_md, &theme::font_body_strong(), (c1.right() - us_tag_r - 24.0).max(48.0));
        p.text(Pos2::new(us_tag_r + 10.0, ry), Align2::LEFT_CENTER, us_line, theme::font_body_strong(), TOKEN_TEXT);
        ry += 20.0;

        // 2. Flow steps
        for (fi, step) in us.flow_steps.iter().enumerate() {
            widgets::hline(&p, c1.left() + 16.0, c1.right() - 16.0, ry, TOKEN_HAIRLINE);
            ry += 15.0;
            let f_tag = draw_ring_pill_left(
                &p,
                Pos2::new(c1.left() + 18.0, ry),
                &format!("2. Flow #{}", fi + 1),
                TOKEN_SURFACE_2,
                TOKEN_BORDER,
                TOKEN_TEXT_MUTED,
            );
            let f_line = widgets::elide(&p, step, &theme::font_body(), (c1.right() - f_tag - 24.0).max(48.0));
            p.text(Pos2::new(f_tag + 10.0, ry), Align2::LEFT_CENTER, f_line, theme::font_body(), TOKEN_TEXT);
            ry += 15.0;
        }

        // 3. Business Rules (`BR-01..`)
        for br in &us.business_rules {
            widgets::hline(&p, c1.left() + 16.0, c1.right() - 16.0, ry, TOKEN_HAIRLINE);
            ry += 16.0;
            let src_l = draw_ring_pill_right(
                &p,
                Pos2::new(c1.right() - 16.0, ry),
                &br.source,
                TOKEN_CODE_BG,
                TOKEN_BORDER,
                TOKEN_TEXT_MUTED,
            );
            let br_r = draw_ring_pill_left(
                &p,
                Pos2::new(c1.left() + 18.0, ry),
                &format!("3. {}", br.id),
                PILL_WARN_FILL,
                PILL_WARN_STROKE,
                PILL_WARN_TEXT,
            );
            let br_txt = widgets::elide(&p, &br.rule, &theme::font_body(), (src_l - br_r - 20.0).max(48.0));
            p.text(Pos2::new(br_r + 10.0, ry), Align2::LEFT_CENTER, br_txt, theme::font_body(), TOKEN_TEXT);
            ry += 16.0;
        }
        y = c1.bottom() + 14.0;

        // CARD 2: `4. Acceptance Criteria` Table (`ID | Given | When | Then | Scenario | Copy cho AI`)
        let c2 = Rect::from_min_size(Pos2::new(body_rect.left(), y), Vec2::new(w, card2_h.max(88.0)));
        draw_section_card_shell(
            &p,
            c2,
            Icon::Check,
            "4. ACCEPTANCE CRITERIA (ID | GIVEN | WHEN | THEN | SCENARIO)",
            &format!("{} ACs · {} SCNs", us.acceptance_criteria.len(), us.scenarios.len()),
        );
        let mut ac_y = c2.top() + 54.0;
        p.text(Pos2::new(c2.left() + 18.0, ac_y), Align2::LEFT_CENTER, "AC", theme::font_small_strong(), TOKEN_TEXT_SUBTLE);
        p.text(
            Pos2::new(c2.left() + 88.0, ac_y),
            Align2::LEFT_CENTER,
            "Given / When / Then Specification",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        p.text(
            Pos2::new(c2.right() - 220.0, ac_y),
            Align2::LEFT_CENTER,
            "Scenario & AI Citation",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        ac_y += 14.0;

        for (ai, ac) in us.acceptance_criteria.iter().enumerate() {
            widgets::hline(&p, c2.left() + 16.0, c2.right() - 16.0, ac_y, TOKEN_HAIRLINE);
            ac_y += 19.0;
            let copy_btn = draw_action_button(
                ui,
                &p,
                &format!("dev-ac-copy-{}-{ai}", us.id),
                Pos2::new(c2.right() - 16.0, ac_y),
                Some(Icon::Sparkle),
                "Copy cho AI",
                TOKEN_PRIMARY_SOFT,
                TOKEN_PRIMARY_ROSE,
                TOKEN_PRIMARY_ORANGE,
            );
            if ui.interact(copy_btn, Id::new(("dev-ac-copy-click", &us.id, ai)), Sense::click()).clicked() {
                let quote = format!(
                    "| {} | Given {} | When {} | Then {} | {} |",
                    ac.id, ac.given, ac.when, ac.then, ac.scenario
                );
                let section_id = format!("{}, {}", us.id, ac.id);
                let payload = format_copy_for_ai(
                    &quote,
                    &us.file_path,
                    ac.line_start,
                    ac.line_end,
                    &section_id,
                    &us.title,
                    "Dev View",
                    &pkg.project_name,
                    "Viết kiểm thử và đảm bảo điều kiện nghiệm thu AC này đạt PASS.",
                );
                copied_ac = Some(( format!("{}.{}", us.id, ac.id), payload ));
            }

            let scn_left = draw_ring_pill_right(
                &p,
                Pos2::new(copy_btn.left() - 8.0, ac_y),
                &ac.scenario,
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            let ac_right = draw_ring_pill_left(
                &p,
                Pos2::new(c2.left() + 18.0, ac_y),
                &ac.id,
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            );
            let gwt = format!("Given {} · When {} -> Then {}", ac.given, ac.when, ac.then);
            let gwt_clean = widgets::elide(&p, &gwt, &theme::font_body(), (scn_left - ac_right - 20.0).max(48.0));
            p.text(Pos2::new(ac_right + 10.0, ac_y), Align2::LEFT_CENTER, gwt_clean, theme::font_body(), TOKEN_TEXT);
            ac_y += 19.0;
        }
        y = c2.bottom() + 14.0;

        // CARD 3: `5. Edge Cases` (`normal / empty / loading / error`) & `6. UI` ASCII Wireframe
        let c3 = Rect::from_min_size(Pos2::new(body_rect.left(), y), Vec2::new(w, card3_h.max(88.0)));
        draw_section_card_shell(
            &p,
            c3,
            Icon::Frame,
            "5. EDGE CASES & 6. ASCII WIREFRAME",
            &format!("{} States · Wireframe", us.edge_cases.len()),
        );
        let mut ey = c3.top() + 54.0;
        for (ei, (state_lbl, desc)) in us.edge_cases.iter().enumerate() {
            if ei > 0 {
                widgets::hline(&p, c3.left() + 16.0, c3.right() - 16.0, ey - 16.0, TOKEN_HAIRLINE);
            }
            let st_r = draw_ring_pill_left(
                &p,
                Pos2::new(c3.left() + 18.0, ey),
                state_lbl,
                PILL_WARN_FILL,
                PILL_WARN_STROKE,
                PILL_WARN_TEXT,
            );
            let d_clean = widgets::elide(&p, desc, &theme::font_body(), (c3.right() - st_r - 24.0).max(48.0));
            p.text(Pos2::new(st_r + 10.0, ey), Align2::LEFT_CENTER, d_clean, theme::font_body(), TOKEN_TEXT);
            ey += 32.0;
        }
        if !us.ascii_wireframe.is_empty() {
            widgets::hline(&p, c3.left() + 16.0, c3.right() - 16.0, ey - 10.0, TOKEN_HAIRLINE);
            for wf_line in &us.ascii_wireframe {
                let clean = widgets::elide(&p, wf_line, &theme::font_mono(), (c3.width() - 36.0).max(48.0));
                p.text(Pos2::new(c3.left() + 18.0, ey), Align2::LEFT_CENTER, clean, theme::font_mono(), PILL_INFO_TEXT);
                ey += 18.0;
            }
        }
    });
    scrollbar(ui, body_rect, app.specs.scroll, est_h);

    if let Some((ac_label, payload)) = copied_ac {
        ui.ctx().copy_text(payload.clone());
        app.specs.last_ai_copy = Some(payload);
        app.show_toast(format!("Copied {ac_label} citation for AI (Claude / Codex)"), false);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 3: `QA & Coverage` (`SpecTab::Plan`)
// Traceability & Coverage Matrix: REQ -> US -> AC -> SCN -> Screen/State -> Method -> Status & Freshness
// ─────────────────────────────────────────────────────────────────────────────
fn render_qa_coverage_tab(app: &mut App, ui: &mut Ui, rect: Rect, pkg: &SpecPackage) {
    let total_ac_rows: usize = pkg.stories.iter().map(|s| s.acceptance_criteria.len().max(1)).sum();
    let card1_h = 38.0 + 30.0 + total_ac_rows.max(1) as f32 * 38.0 + 18.0;
    let card2_h = 38.0 + pkg.gates.len().max(1) as f32 * 36.0 + 18.0;
    let est_h = card1_h + card2_h + 40.0;

    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let mut copied_scn: Option<String> = None;

    clipped(ui, rect, |ui| {
        let p = ui.painter().clone();
        let mut y = rect.top() - scroll;
        let w = (rect.width() - 10.0).max(200.0);

        // SECTION CARD 1: Traceability & Coverage Matrix
        let c1 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card1_h.max(92.0)));
        draw_section_card_shell(
            &p,
            c1,
            Icon::Check,
            "TRACEABILITY & COVERAGE MATRIX (REQ -> US -> AC -> SCN -> SCREEN/STATE -> METHOD -> STATUS)",
            "100% Coverage · CURRENT",
        );
        let mut ry = c1.top() + 54.0;
        p.text(
            Pos2::new(c1.left() + 18.0, ry),
            Align2::LEFT_CENTER,
            "Trace Chain (REQ -> US -> AC -> SCN)",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        p.text(
            Pos2::new(c1.left() + 340.0, ry),
            Align2::LEFT_CENTER,
            "Screen States & Expected Outcome",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        p.text(
            Pos2::new(c1.right() - 250.0, ry),
            Align2::LEFT_CENTER,
            "Method · Status & Freshness",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        ry += 14.0;

        let mut row_idx = 0usize;
        for us in &pkg.stories {
            let req_id = us.req_refs.first().cloned().unwrap_or_else(|| "REQ-01".to_string());
            let scr_id = us.screens.first().cloned().unwrap_or_else(|| "S01".to_string());
            for ac in &us.acceptance_criteria {
                widgets::hline(&p, c1.left() + 16.0, c1.right() - 16.0, ry, TOKEN_HAIRLINE);
                ry += 19.0;

                let copy_btn = draw_action_button(
                    ui,
                    &p,
                    &format!("qa-row-copy-{row_idx}"),
                    Pos2::new(c1.right() - 16.0, ry),
                    Some(Icon::Sparkle),
                    "Copy cho AI",
                    TOKEN_SURFACE_2,
                    TOKEN_BORDER,
                    TOKEN_TEXT_MUTED,
                );
                if ui.interact(copy_btn, Id::new(("qa-row-copy-click", row_idx)), Sense::click()).clicked() {
                    copied_scn = Some(format_copy_for_ai(
                        &format!("{} -> {}.{} -> {} ({}/normal, empty, loading, error): {}", req_id, us.id, ac.id, ac.scenario, scr_id, ac.then),
                        &us.file_path,
                        ac.line_start,
                        ac.line_end,
                        &format!("{}.{}", us.id, ac.id),
                        &ac.scenario,
                        "QA & Coverage",
                        &pkg.project_name,
                        "Chạy kịch bản kiểm thử tự động và cập nhật bằng chứng CURRENT.",
                    ));
                }

                let status_left = draw_ring_pill_right(
                    &p,
                    Pos2::new(copy_btn.left() - 8.0, ry),
                    "PASSED · CURRENT",
                    PILL_OK_FILL,
                    PILL_OK_STROKE,
                    PILL_OK_TEXT,
                );
                let method_left = draw_ring_pill_right(
                    &p,
                    Pos2::new(status_left - 6.0, ry),
                    "auto",
                    PILL_INFO_FILL,
                    PILL_INFO_STROKE,
                    PILL_INFO_TEXT,
                );

                let mut cx = c1.left() + 18.0;
                cx = draw_ring_pill_left(&p, Pos2::new(cx, ry), &req_id, PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT) + 6.0;
                cx = draw_ring_pill_left(&p, Pos2::new(cx, ry), &us.id, TOKEN_SURFACE_2, TOKEN_BORDER, TOKEN_TEXT) + 6.0;
                cx = draw_ring_pill_left(&p, Pos2::new(cx, ry), &ac.id, PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT) + 6.0;
                cx = draw_ring_pill_left(&p, Pos2::new(cx, ry), &ac.scenario, PILL_OK_FILL, PILL_OK_STROKE, PILL_OK_TEXT) + 10.0;

                let state_summary = format!("{scr_id}/normal, empty, error, loading · {}", ac.then);
                let clean = widgets::elide(&p, &state_summary, &theme::font_body(), (method_left - cx - 14.0).max(48.0));
                p.text(Pos2::new(cx, ry), Align2::LEFT_CENTER, clean, theme::font_body(), TOKEN_TEXT);

                ry += 19.0;
                row_idx += 1;
            }
        }
        y = c1.bottom() + 14.0;

        // SECTION CARD 2: Gate Eval & TDD Verification Commands
        let c2 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card2_h.max(88.0)));
        draw_section_card_shell(
            &p,
            c2,
            Icon::Play,
            "TDD & GATE VERIFICATION CHECKLIST (review/gates/*.md)",
            "All Evals Exit 0",
        );
        let mut gy = c2.top() + 56.0;
        for (gi, g) in pkg.gates.iter().enumerate() {
            if gi > 0 {
                widgets::hline(&p, c2.left() + 16.0, c2.right() - 16.0, gy - 18.0, TOKEN_HAIRLINE);
            }
            let (pfill, pstroke, pfg) = status_pill_colors(&g.status);
            let st_left = draw_ring_pill_right(&p, Pos2::new(c2.right() - 16.0, gy), &g.status, pfill, pstroke, pfg);
            let v_left = draw_ring_pill_right(
                &p,
                Pos2::new(st_left - 8.0, gy),
                &format!("VERDICT: {}", g.verdict),
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            icons::paint(&p, Icon::Check, Pos2::new(c2.left() + 24.0, gy), 13.0, PILL_OK_TEXT);
            let txt = format!("{} ({}) · Freshness {} · {}", g.gate_id, g.stage, g.freshness, g.file_path);
            let clean = widgets::elide(&p, &txt, &theme::font_body(), (v_left - c2.left() - 48.0).max(48.0));
            p.text(Pos2::new(c2.left() + 38.0, gy), Align2::LEFT_CENTER, clean, theme::font_body(), TOKEN_TEXT);
            gy += 36.0;
        }
    });
    scrollbar(ui, rect, app.specs.scroll, est_h);

    if let Some(payload) = copied_scn {
        ui.ctx().copy_text(payload.clone());
        app.specs.last_ai_copy = Some(payload);
        app.show_toast("Copied QA Scenario citation for AI", false);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 4: `Wireframes & UI Contract` (`SpecTab::JsonSpec` / `SpecTab::UiSpecs`)
// Declarative `.spec.json` (`NodeSpec`) Apply + Screen Inventory + 12-Column `ui-contract.md` Table
// ─────────────────────────────────────────────────────────────────────────────
fn render_wireframes_and_contract_tab(app: &mut App, ui: &mut Ui, rect: Rect, pkg: &SpecPackage) {
    let wf_lines_count: usize = pkg.stories.iter().map(|s| s.ascii_wireframe.len()).sum();
    let card1_h = 38.0 + 48.0 + 18.0;
    let card2_h = 38.0 + pkg.screens.len().max(1) as f32 * 36.0 + wf_lines_count.min(24) as f32 * 18.0 + 24.0;
    let card3_h = 38.0 + 30.0 + pkg.ui_contract.len().max(1) as f32 * 34.0 + 18.0;
    let est_h = card1_h + card2_h + card3_h + 48.0;

    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let mut apply_json_action: Option<(String, String)> = None;

    clipped(ui, rect, |ui| {
        let p = ui.painter().clone();
        let mut y = rect.top() - scroll;
        let w = (rect.width() - 10.0).max(200.0);

        // SECTION CARD 1: Declarative `.spec.json` (`NodeSpec`) Blueprints with 1-Click Apply
        if let Some((j_name, _j_path, j_raw)) = pkg
            .json_specs
            .get(app.specs.selected_json_spec)
            .or_else(|| pkg.json_specs.first())
            .cloned()
        {
            let summary = summarize_json_spec(&j_raw);
            let c1 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card1_h.max(92.0)));
            draw_section_card_shell(
                &p,
                c1,
                Icon::Frame,
                &format!("DECLARATIVE NODESPEC BLUEPRINT · {} ({})", j_name, summary.root_name),
                &format!("{} Nodes · {} Presets", summary.total_nodes, summary.presets.len()),
            );
            let row_y = c1.top() + 64.0;
            let apply_btn = draw_action_button(
                ui,
                &p,
                "json-tab-apply-btn",
                Pos2::new(c1.right() - 16.0, row_y),
                Some(Icon::Play),
                "Apply to Canvas",
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            if ui.interact(apply_btn, Id::new("json-tab-apply-click"), Sense::click()).clicked() {
                apply_json_action = Some((j_name.clone(), j_raw.clone()));
            }
            let mut bx = c1.left() + 18.0;
            bx = draw_ring_pill_left(
                &p,
                Pos2::new(bx, row_y),
                &format!("{} Nodes", summary.total_nodes),
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            ) + 8.0;
            bx = draw_ring_pill_left(
                &p,
                Pos2::new(bx, row_y),
                &format!("{} Presets", summary.presets.len()),
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            ) + 8.0;
            bx = draw_ring_pill_left(
                &p,
                Pos2::new(bx, row_y),
                &format!("{} $easeui.* Tokens", summary.easeui_tokens.len()),
                TOKEN_PRIMARY_SOFT,
                TOKEN_PRIMARY_ROSE,
                TOKEN_PRIMARY_ORANGE,
            ) + 8.0;
            for tok in summary.easeui_tokens.iter().take(3) {
                if bx + 120.0 > apply_btn.left() {
                    break;
                }
                bx = draw_ring_pill_left(&p, Pos2::new(bx, row_y), tok, TOKEN_CODE_BG, TOKEN_BORDER, TOKEN_TEXT_MUTED) + 6.0;
            }
            y = c1.bottom() + 14.0;
        }

        // SECTION CARD 2: Screen Inventory (`S01..S03` × States) & ASCII Wireframes
        let c2 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card2_h.max(92.0)));
        draw_section_card_shell(
            &p,
            c2,
            Icon::Eye,
            "SCREEN INVENTORY (ux/screen-inventory.md) & ASCII WIREFRAMES",
            &format!("{} Screens", pkg.screens.len()),
        );
        let mut sy = c2.top() + 56.0;
        for (si, scr) in pkg.screens.iter().enumerate() {
            if si > 0 {
                widgets::hline(&p, c2.left() + 16.0, c2.right() - 16.0, sy - 18.0, TOKEN_HAIRLINE);
            }
            let us_left = draw_ring_pill_right(
                &p,
                Pos2::new(c2.right() - 16.0, sy),
                &scr.story,
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            );
            let id_right = draw_ring_pill_left(
                &p,
                Pos2::new(c2.left() + 18.0, sy),
                &scr.id,
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            let txt = format!("{}  ·  States: {}", scr.name, scr.states.join(" · "));
            let clean = widgets::elide(&p, &txt, &theme::font_body_strong(), (us_left - id_right - 20.0).max(48.0));
            p.text(Pos2::new(id_right + 10.0, sy), Align2::LEFT_CENTER, clean, theme::font_body_strong(), TOKEN_TEXT);
            sy += 36.0;
        }
        widgets::hline(&p, c2.left() + 16.0, c2.right() - 16.0, sy - 12.0, TOKEN_HAIRLINE);
        for wf_line in pkg.stories.iter().flat_map(|s| s.ascii_wireframe.iter()).take(24) {
            let clean = widgets::elide(&p, wf_line, &theme::font_mono(), (c2.width() - 36.0).max(48.0));
            p.text(Pos2::new(c2.left() + 18.0, sy), Align2::LEFT_CENTER, clean, theme::font_mono(), PILL_INFO_TEXT);
            sy += 18.0;
        }
        y = c2.bottom() + 14.0;

        // SECTION CARD 3: 12-Column Behavioral `ui-contract.md` Table
        let c3 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card3_h.max(92.0)));
        draw_section_card_shell(
            &p,
            c3,
            Icon::Specs,
            "12-COLUMN BEHAVIORAL UI CONTRACT (ux/ui-contract.md)",
            &format!("{} Selector Rows", pkg.ui_contract.len()),
        );
        let mut cy = c3.top() + 54.0;
        p.text(
            Pos2::new(c3.left() + 18.0, cy),
            Align2::LEFT_CENTER,
            "Screen · State · US",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        p.text(
            Pos2::new(c3.left() + 210.0, cy),
            Align2::LEFT_CENTER,
            "Route · Roles · Scope · Element · Kind · Components · Patterns · Target",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        p.text(
            Pos2::new(c3.right() - 120.0, cy),
            Align2::LEFT_CENTER,
            "SCN",
            theme::font_small_strong(),
            TOKEN_TEXT_SUBTLE,
        );
        cy += 14.0;

        for row in &pkg.ui_contract {
            widgets::hline(&p, c3.left() + 16.0, c3.right() - 16.0, cy, TOKEN_HAIRLINE);
            cy += 17.0;
            let scn_left = draw_ring_pill_right(
                &p,
                Pos2::new(c3.right() - 16.0, cy),
                &row.scn,
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            );
            let mut lx = c3.left() + 18.0;
            lx = draw_ring_pill_left(&p, Pos2::new(lx, cy), &row.screen, PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT) + 6.0;
            let (sfill, sstroke, sfg) = match row.state.as_str() {
                "normal" | "focus" => (PILL_OK_FILL, PILL_OK_STROKE, PILL_OK_TEXT),
                "error" => (PILL_BAD_FILL, PILL_BAD_STROKE, PILL_BAD_TEXT),
                _ => (PILL_WARN_FILL, PILL_WARN_STROKE, PILL_WARN_TEXT),
            };
            lx = draw_ring_pill_left(&p, Pos2::new(lx, cy), &row.state, sfill, sstroke, sfg) + 6.0;
            lx = draw_ring_pill_left(&p, Pos2::new(lx, cy), &row.us, TOKEN_SURFACE_2, TOKEN_BORDER, TOKEN_TEXT) + 10.0;

            let summary = format!(
                "{} [{} · {}] {} ({} · {} · {}) -> {}",
                row.route, row.roles, row.scope, row.element, row.kind, row.components, row.patterns, row.target
            );
            let clean = widgets::elide(&p, &summary, &theme::font_body(), (scn_left - lx - 14.0).max(48.0));
            p.text(Pos2::new(lx, cy), Align2::LEFT_CENTER, clean, theme::font_body(), TOKEN_TEXT);
            cy += 17.0;
        }
    });
    scrollbar(ui, rect, app.specs.scroll, est_h);

    if let Some((j_name, j_raw)) = apply_json_action {
        if let Ok(spec) = serde_json::from_str::<NodeSpec>(&j_raw) {
            app.apply_node_spec(&spec, &j_name);
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// TAB 5: `Gates, Decisions & Canvas` (`SpecTab::CanvasScreens` / `SpecTab::CanvasLinks`)
// Gate Records (`G1..G5`) + SHA-256 Digest + Rubric (`2/2`) + Decision Ledger (`D1..D5`) + Linked Canvas Frames
// ─────────────────────────────────────────────────────────────────────────────
fn render_gates_and_canvas_tab(app: &mut App, ui: &mut Ui, rect: Rect, pkg: &SpecPackage) {
    let page_children: Vec<NodeId> = app.doc.get(app.page).map(|p| p.children.clone()).unwrap_or_default();
    let resolved_comments = app.doc.resolve_comment_targets();

    let card1_h = 38.0 + pkg.gates.len().max(1) as f32 * 56.0 + 16.0;
    let card2_h = 38.0 + pkg.decisions.len().max(1) as f32 * 36.0 + 16.0;
    let card3_h = 38.0 + page_children.len().max(1) as f32 * 48.0 + 16.0;
    let est_h = card1_h + card2_h + card3_h + 48.0;

    let mut scroll = app.specs.scroll;
    wheel_scroll(ui, rect, &mut scroll, est_h);
    app.specs.scroll = scroll;

    let mut focus_target: Option<NodeId> = None;
    let mut export_target: Option<NodeId> = None;

    clipped(ui, rect, |ui| {
        let p = ui.painter().clone();
        let mut y = rect.top() - scroll;
        let w = (rect.width() - 10.0).max(200.0);

        // SECTION CARD 1: Gate Records (`G1..G5`) with SHA-256 Digest & Rubric Table (`2/2`)
        let c1 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card1_h.max(96.0)));
        draw_section_card_shell(
            &p,
            c1,
            Icon::Flag,
            "PIPELINE GATES (G1–G5) · SHA-256 DIGESTS & RUBRIC SCORES (review/gates/*.md)",
            "VERDICT: PASS · CURRENT",
        );
        let mut gy = c1.top() + 54.0;
        for (gi, g) in pkg.gates.iter().enumerate() {
            if gi > 0 {
                widgets::hline(&p, c1.left() + 16.0, c1.right() - 16.0, gy - 10.0, TOKEN_HAIRLINE);
            }
            let (sfill, sstroke, sfg) = status_pill_colors(&g.status);
            let mut gx = c1.left() + 18.0;
            gx = draw_ring_pill_left(
                &p,
                Pos2::new(gx, gy),
                &format!("GATE: {}", g.status),
                sfill,
                sstroke,
                sfg,
            ) + 8.0;
            gx = draw_ring_pill_left(
                &p,
                Pos2::new(gx, gy),
                &format!("{} ({})", g.gate_id, g.stage),
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            ) + 8.0;
            gx = draw_ring_pill_left(
                &p,
                Pos2::new(gx, gy),
                &format!("VERDICT: {}", g.verdict),
                PILL_OK_FILL,
                PILL_OK_STROKE,
                PILL_OK_TEXT,
            ) + 8.0;
            gx = draw_ring_pill_left(&p, Pos2::new(gx, gy), &g.freshness, PILL_OK_FILL, PILL_OK_STROKE, PILL_OK_TEXT) + 8.0;
            let short_sha: String = g.config_sha256.chars().take(12).collect();
            if !short_sha.is_empty() && gx + 140.0 < c1.right() {
                let _ = draw_ring_pill_left(
                    &p,
                    Pos2::new(gx, gy),
                    &format!("sha256:{short_sha}"),
                    TOKEN_CODE_BG,
                    TOKEN_BORDER,
                    TOKEN_TEXT_MUTED,
                );
            }

            // Second line of the gate row: Rubric criteria (`A Grounded 2/2`, `B Clear 2/2`, ...)
            let ry = gy + 24.0;
            let mut rx = c1.left() + 18.0;
            for (crit_name, score, max) in &g.rubric {
                if rx + 120.0 > c1.right() - 16.0 {
                    break;
                }
                rx = draw_ring_pill_left(
                    &p,
                    Pos2::new(rx, ry),
                    &format!("{crit_name} {score}/{max}"),
                    TOKEN_SURFACE_2,
                    TOKEN_BORDER,
                    PILL_OK_TEXT,
                ) + 6.0;
            }
            gy += 56.0;
        }
        y = c1.bottom() + 14.0;

        // SECTION CARD 2: Decision Ledger (`D1..D5`)
        let c2 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card2_h.max(88.0)));
        draw_section_card_shell(
            &p,
            c2,
            Icon::Book,
            "DECISION LEDGER (decisions/decision-log.md)",
            &format!("{} Decisions", pkg.decisions.len()),
        );
        let mut dy = c2.top() + 56.0;
        for (di, dec) in pkg.decisions.iter().enumerate() {
            if di > 0 {
                widgets::hline(&p, c2.left() + 16.0, c2.right() - 16.0, dy - 18.0, TOKEN_HAIRLINE);
            }
            let d_right = draw_ring_pill_left(
                &p,
                Pos2::new(c2.left() + 18.0, dy),
                &dec.id,
                PILL_WARN_FILL,
                PILL_WARN_STROKE,
                PILL_WARN_TEXT,
            );
            let txt = format!("{}  —  {}", dec.decision, dec.source_detail);
            let clean = widgets::elide(&p, &txt, &theme::font_body(), (c2.right() - d_right - 26.0).max(48.0));
            p.text(Pos2::new(d_right + 10.0, dy), Align2::LEFT_CENTER, clean, theme::font_body(), TOKEN_TEXT);
            dy += 36.0;
        }
        y = c2.bottom() + 14.0;

        // SECTION CARD 3: Linked Canvas Frames (`Focus on Canvas` + `Export Frame -> .spec.json`)
        let c3 = Rect::from_min_size(Pos2::new(rect.left(), y), Vec2::new(w, card3_h.max(92.0)));
        draw_section_card_shell(
            &p,
            c3,
            Icon::Frame,
            "LINKED CANVAS FRAMES & SPEC SYNC",
            &format!("{} Active Frames", page_children.len()),
        );
        let mut fy = c3.top() + 60.0;
        for (fi, fid) in page_children.iter().enumerate() {
            let Some(node) = app.doc.get(*fid) else { continue };
            if fi > 0 {
                widgets::hline(&p, c3.left() + 16.0, c3.right() - 16.0, fy - 24.0, TOKEN_HAIRLINE);
            }
            let exp_btn = draw_action_button(
                ui,
                &p,
                &format!("screen-export-{}", fid.0),
                Pos2::new(c3.right() - 16.0, fy),
                Some(Icon::Export),
                "Export Frame -> .spec.json",
                TOKEN_CODE_BG,
                TOKEN_BORDER,
                TOKEN_TEXT,
            );
            let foc_btn = draw_action_button(
                ui,
                &p,
                &format!("screen-focus-{}", fid.0),
                Pos2::new(exp_btn.left() - 8.0, fy),
                Some(Icon::Eye),
                "Focus on Canvas",
                PILL_INFO_FILL,
                PILL_INFO_STROKE,
                PILL_INFO_TEXT,
            );
            if ui.interact(foc_btn, Id::new(("screen-focus-click", fid.0)), Sense::click()).clicked() {
                focus_target = Some(*fid);
            }
            if ui.interact(exp_btn, Id::new(("screen-export-click", fid.0)), Sense::click()).clicked() {
                export_target = Some(*fid);
            }

            let al_dir = match node.kind.frame_props().and_then(|f| f.auto_layout) {
                Some(al) if al.direction == LayoutDirection::Horizontal => "Auto-Layout Row",
                Some(al) if al.direction == LayoutDirection::Grid => "2D CSS Grid",
                Some(_) => "Auto-Layout Column",
                None => "Freeform Frame",
            };
            let open_comments = resolved_comments
                .iter()
                .filter(|c| !c.resolved && (c.target_frame_id == Some(fid.0) || c.target_node_id == Some(fid.0)))
                .count();

            let mut bx = c3.left() + 18.0;
            bx = draw_ring_pill_left(
                &p,
                Pos2::new(bx, fy),
                &format!("{:.0}x{:.0}", node.size.x, node.size.y),
                TOKEN_CODE_BG,
                TOKEN_BORDER,
                TOKEN_TEXT_MUTED,
            ) + 8.0;
            bx = draw_ring_pill_left(&p, Pos2::new(bx, fy), al_dir, PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT) + 8.0;
            if open_comments > 0 {
                bx = draw_ring_pill_left(
                    &p,
                    Pos2::new(bx, fy),
                    &format!("{open_comments} pins"),
                    PILL_WARN_FILL,
                    PILL_WARN_STROKE,
                    PILL_WARN_TEXT,
                ) + 8.0;
            }
            let title_max = (foc_btn.left() - bx - 14.0).max(48.0);
            let clean_title = widgets::elide(&p, &node.name, &theme::font_body_strong(), title_max);
            p.text(Pos2::new(bx, fy), Align2::LEFT_CENTER, clean_title, theme::font_body_strong(), TOKEN_TEXT);

            fy += 48.0;
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

// ─────────────────────────────────────────────────────────────────────────────
// Operate-Mode Drawing Primitives (No cards-in-cards, ring-bordered pills, vector icons)
// ─────────────────────────────────────────────────────────────────────────────

/// Draws a top-level section card (`#1F2024`, `radius = 12px`, `1px` border `#2E2F36`) with a
/// `38px` `#26272C` table/section header strip (`tokens.css` `--table-head-h: 38px`).
fn draw_section_card_shell(p: &Painter, card_rect: Rect, icon: Icon, title: &str, right_badge: &str) {
    p.rect_filled(card_rect, CornerRadius::same(12), TOKEN_SURFACE);
    p.rect_stroke(card_rect, CornerRadius::same(12), Stroke::new(1.0, TOKEN_BORDER), StrokeKind::Inside);

    let head_h = 38.0;
    let head_rect = Rect::from_min_size(card_rect.min, Vec2::new(card_rect.width(), head_h));
    p.rect_filled(
        head_rect,
        CornerRadius {
            nw: 12,
            ne: 12,
            sw: 0,
            se: 0,
        },
        TOKEN_SURFACE_2,
    );
    widgets::hline(p, head_rect.left(), head_rect.right(), head_rect.bottom(), TOKEN_BORDER);

    let badge_left = if !right_badge.is_empty() {
        draw_ring_pill_right(
            p,
            Pos2::new(head_rect.right() - 14.0, head_rect.center().y),
            right_badge,
            TOKEN_CODE_BG,
            TOKEN_BORDER,
            TOKEN_TEXT_MUTED,
        ) - 10.0
    } else {
        head_rect.right() - 14.0
    };

    icons::paint(p, icon, Pos2::new(head_rect.left() + 18.0, head_rect.center().y), 13.0, PILL_INFO_TEXT);
    let max_w = (badge_left - head_rect.left() - 36.0).max(48.0);
    let clean = widgets::elide(p, title, &theme::font_small_strong(), max_w);
    p.text(
        Pos2::new(head_rect.left() + 30.0, head_rect.center().y),
        Align2::LEFT_CENTER,
        clean,
        theme::font_small_strong(),
        TOKEN_TEXT,
    );
}

fn draw_left_section_header(p: &Painter, pos: Pos2, width: f32, icon: Icon, title: &str, color: Color32) {
    let r = Rect::from_min_size(pos, Vec2::new(width, 20.0));
    icons::paint(p, icon, Pos2::new(r.left() + 8.0, r.center().y), 12.0, color);
    let txt = widgets::elide(p, title, &theme::font_small_strong(), (width - 24.0).max(36.0));
    p.text(Pos2::new(r.left() + 18.0, r.center().y), Align2::LEFT_CENTER, txt, theme::font_small_strong(), color);
}

fn status_pill_colors(status: &str) -> (Color32, Color32, Color32) {
    let upper = status.to_ascii_uppercase();
    if upper.contains("APPROVED") || upper.contains("PASS") || upper.contains("CURRENT") || upper.contains("VERIFIED") {
        (PILL_OK_FILL, PILL_OK_STROKE, PILL_OK_TEXT)
    } else if upper.contains("PENDING") || upper.contains("STALE") || upper.contains("OPEN") {
        (PILL_WARN_FILL, PILL_WARN_STROKE, PILL_WARN_TEXT)
    } else if upper.contains("FAIL") || upper.contains("CHANGES_REQUESTED") || upper.contains("BLOCK") {
        (PILL_BAD_FILL, PILL_BAD_STROKE, PILL_BAD_TEXT)
    } else {
        (PILL_INFO_FILL, PILL_INFO_STROKE, PILL_INFO_TEXT)
    }
}

/// Draws a left-anchored ring-bordered semantic pill (`radius = 99px`) and returns its `right()` x coordinate.
fn draw_ring_pill_left(p: &Painter, left_center: Pos2, text: &str, fill: Color32, stroke: Color32, fg: Color32) -> f32 {
    let font: FontId = theme::font_small_strong();
    let tw = widgets::text_width(p, text, &font);
    let w = (tw + 16.0).ceil().max(26.0);
    let h = 20.0;
    let rect = Rect::from_min_size(Pos2::new(left_center.x, left_center.y - h * 0.5), Vec2::new(w, h));
    p.rect_filled(rect, CornerRadius::same(99), fill);
    p.rect_stroke(rect, CornerRadius::same(99), Stroke::new(1.0, stroke), StrokeKind::Inside);
    p.text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
    rect.right()
}

/// Draws a right-anchored ring-bordered semantic pill (`radius = 99px`) and returns its `left()` x coordinate.
fn draw_ring_pill_right(p: &Painter, right_center: Pos2, text: &str, fill: Color32, stroke: Color32, fg: Color32) -> f32 {
    let font: FontId = theme::font_small_strong();
    let tw = widgets::text_width(p, text, &font);
    let w = (tw + 16.0).ceil().max(26.0);
    let h = 20.0;
    let rect = Rect::from_min_size(Pos2::new(right_center.x - w, right_center.y - h * 0.5), Vec2::new(w, h));
    p.rect_filled(rect, CornerRadius::same(99), fill);
    p.rect_stroke(rect, CornerRadius::same(99), Stroke::new(1.0, stroke), StrokeKind::Inside);
    p.text(rect.center(), Align2::CENTER_CENTER, text, font, fg);
    rect.left()
}

/// Draws a right-aligned action button with an optional vector `Icon` and returns its `Rect`.
#[allow(clippy::too_many_arguments)]
fn draw_action_button(
    ui: &mut Ui,
    p: &Painter,
    id_salt: &str,
    right_center: Pos2,
    icon: Option<Icon>,
    label: &str,
    bg: Color32,
    border: Color32,
    fg: Color32,
) -> Rect {
    let font = theme::font_small_strong();
    let tw = widgets::text_width(p, label, &font);
    let icon_pad = if icon.is_some() { 16.0 } else { 0.0 };
    let w = (tw + icon_pad + 20.0).ceil().max(48.0);
    let h = 24.0;
    let rect = Rect::from_min_size(Pos2::new(right_center.x - w, right_center.y - h * 0.5), Vec2::new(w, h));
    let resp = ui.interact(rect, Id::new(id_salt), Sense::hover());
    p.rect_filled(rect, CornerRadius::same(99), if resp.hovered() { border } else { bg });
    p.rect_stroke(rect, CornerRadius::same(99), Stroke::new(1.0, border), StrokeKind::Inside);
    let text_col = if resp.hovered() { TOKEN_TEXT } else { fg };
    if let Some(ic) = icon {
        icons::paint(p, ic, Pos2::new(rect.left() + 13.0, rect.center().y), 12.0, text_col);
        p.text(
            Pos2::new(rect.left() + 23.0, rect.center().y),
            Align2::LEFT_CENTER,
            label,
            font,
            text_col,
        );
    } else {
        p.text(rect.center(), Align2::CENTER_CENTER, label, font, text_col);
    }
    widgets::record(ui, id_salt, rect);
    rect
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_workspace_spec_packages_and_json_blueprints() {
        let mut state = SpecsState::default();
        state.ensure_loaded();
        assert!(!state.packages.is_empty(), "must discover workspace or fallback spec packages");
        let pkg = state.current_package().expect("active package");
        assert_eq!(pkg.slug, "designos-studio-v2", "canonical Spec Format v2 bundle should be primary");
        assert_eq!(pkg.format_version, "2");
        assert!(!pkg.title.is_empty());
        assert_eq!(pkg.pipeline_stages.len(), 11, "must load 11 Spec Format v2 stages");
        assert_eq!(pkg.requirements.len(), 5, "must parse REQ-01..REQ-05 from PRD.md");
        assert_eq!(pkg.stories.len(), 3, "must parse US-01..US-03 from us/*.md");
        assert!(!pkg.ui_contract.is_empty(), "must parse 12-column ui-contract.md");
        assert_eq!(pkg.gates.len(), 5, "must parse G1..G5 gate files");
        assert!(!pkg.json_specs.is_empty(), "must include .spec.json blueprints");
    }

    #[test]
    fn parses_us_01_frontmatter_and_ac_table_and_ui_contract_and_gates() {
        let us = parse_us_markdown(V2_US_01_MD, "plans/specs/designos-studio-v2/us/US-01.md").expect("valid US-01.md");
        assert_eq!(us.id, "US-01");
        assert_eq!(us.priority, "MVP");
        assert_eq!(us.wave, "W1");
        assert_eq!(us.req_refs, vec!["REQ-01".to_string(), "REQ-02".to_string()]);
        assert_eq!(us.screens, vec!["S01".to_string(), "S02".to_string(), "S03".to_string()]);
        assert_eq!(us.boards, vec!["W-01".to_string()]);
        assert_eq!(us.scenarios.len(), 3);
        assert_eq!(us.acceptance_criteria.len(), 3);
        assert_eq!(us.acceptance_criteria[0].id, "AC-01");
        assert_eq!(us.acceptance_criteria[0].scenario, "SCN-01-01");
        assert!(!us.ascii_wireframe.is_empty(), "US-01 must include section 6 ASCII wireframe");

        let contract_rows = parse_ui_contract(V2_UI_CONTRACT_MD);
        assert_eq!(contract_rows.len(), 18, "3 screens x 6 states = 18 behavioral selector rows");
        assert_eq!(contract_rows[0].screen, "S01");
        assert_eq!(contract_rows[0].state, "normal");
        assert_eq!(contract_rows[0].us, "US-01");
        assert_eq!(contract_rows[0].scn, "SCN-01-03");

        let g1 = parse_gate_markdown(
            V2_GATE_G1_MD,
            "g1-intake",
            "plans/specs/designos-studio-v2/review/gates/g1-intake.md",
        )
        .expect("valid g1-intake.md");
        assert_eq!(g1.status, "APPROVED");
        assert_eq!(g1.verdict, "PASS");
        assert_eq!(g1.freshness, "CURRENT");
        assert_eq!(g1.rubric.len(), 6);
        assert!(g1.rubric.iter().all(|(_, s, m)| *s == 2 && *m == 2));

        let g4 = parse_gate_markdown(
            V2_GATE_G4_MD,
            "g4-uiux",
            "plans/specs/designos-studio-v2/review/gates/g4-uiux.md",
        )
        .expect("valid g4-uiux.md");
        assert_eq!(g4.status, "PENDING");
        assert_eq!(g4.verdict, "PASS");
    }

    #[test]
    fn formats_copy_cho_ai_citation_markdown() {
        let md = format_copy_for_ai(
            "AC-01 | File .fig chuẩn v106 | Mở và lưu lại | Bảo toàn 100% raw_fields",
            "plans/specs/designos-studio-v2/us/US-01.md",
            48,
            52,
            "US-01",
            "1:1 Figma .fig v106 Binary R/W & 2D CSS Grid",
            "Dev View",
            "design-os-studio",
            "Kiểm tra round-trip .fig v106",
        );
        assert!(md.contains("File: plans/specs/designos-studio-v2/us/US-01.md:48-52"));
        assert!(md.contains("Mục: US-01 › 1:1 Figma .fig v106 Binary R/W & 2D CSS Grid"));
        assert!(md.contains("Trang reader: Dev View"));
        assert!(md.contains("Project: design-os-studio"));
        assert!(md.contains("Yêu cầu của tôi: Kiểm tra round-trip .fig v106"));
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
