//! Check designs: scans layers for design-system drift and offers a one-click fix per finding.
//!
//! Three rules, as in Figma's "Check designs":
//! - **Hard-coded values**: a fill, stroke, text style or effect list holds a raw value that equals an
//!   existing variable or shared style but is not linked to it. The suggested fix links it.
//! - **Contrast**: text fails WCAG 2.x AA against the backdrop its ancestors paint.
//! - **Detached instances**: a frame that was detached from a main component present in this file;
//!   the reconnect fix is offered only while the frame still equals the component.
//!
//! The audit is a pure function of the [`Document`]. Fixes run through [`crate::edit::Tx`], so each
//! fix (and "Fix all") is exactly one undo step. Layers inside an instance mirror their main
//! component and are skipped: the fix belongs on the main component, and the instance follows.

mod contrast;
mod detached;
mod fix;
pub mod fixture;
mod hardcoded;
mod library_assets;
mod state;
#[cfg(test)]
mod tests;
mod text_styles;

pub use fixture::FIXTURE;
pub use state::CheckDesignsState;

use designos_doc::{Color, Document, IgnoreKey, LibraryRef, LibrarySnapshot, NodeId, NodeKind};
use std::collections::HashMap;
use std::sync::Arc;

/// Layers scanned per audit. A page of several hundred thousand layers must never stall the UI
/// thread, so the audit stops here and the panel says so.
pub const MAX_SCANNED_LAYERS: usize = 20_000;

/// The kind of problem a [`Finding`] reports; also the group it is listed under.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Rule {
    HardCodedValue,
    Contrast,
    DetachedInstance,
    LibraryAssets,
    TextStyles,
    CornerRadius,
    Padding,
    HorizontalSpacing,
    VerticalSpacing,
    TypographyAndStroke,
}

impl Rule {
    pub const ALL: [Rule; 10] = [
        Rule::HardCodedValue,
        Rule::Contrast,
        Rule::DetachedInstance,
        Rule::LibraryAssets,
        Rule::TextStyles,
        Rule::CornerRadius,
        Rule::Padding,
        Rule::HorizontalSpacing,
        Rule::VerticalSpacing,
        Rule::TypographyAndStroke,
    ];

    /// Group heading in the panel.
    pub fn title(self) -> &'static str {
        match self {
            Rule::HardCodedValue => "Colours",
            Rule::Contrast => "Contrast",
            Rule::DetachedInstance => "Detached instances",
            Rule::LibraryAssets => "Library assets",
            Rule::TextStyles => "Text styles",
            Rule::CornerRadius => "Corner radius",
            Rule::Padding => "Padding",
            Rule::HorizontalSpacing => "Horizontal spacing",
            Rule::VerticalSpacing => "Vertical spacing",
            Rule::TypographyAndStroke => "Typography and stroke",
        }
    }

    /// Stable Figma design-linter rule identifier used in `CheckDesignsSettings::disabled_rules`
    /// and `IgnoreKey::rule`.
    pub fn figma_id(self) -> &'static str {
        match self {
            Rule::HardCodedValue | Rule::Contrast => "RECOMMEND_VARIABLES",
            Rule::DetachedInstance => "REQUIRE_NO_DETACHED_COMPONENTS",
            Rule::LibraryAssets => "REQUIRE_ASSETS_FROM_SELECTED_LIBRARIES",
            Rule::TextStyles => "REQUIRE_TEXT_STYLES",
            Rule::CornerRadius => "REQUIRE_CORNER_RADIUS_VARIABLES",
            Rule::Padding => "REQUIRE_PADDING_VARIABLES",
            Rule::HorizontalSpacing => "REQUIRE_HORIZONTAL_SPACING_VARIABLES",
            Rule::VerticalSpacing => "REQUIRE_VERTICAL_SPACING_VARIABLES",
            Rule::TypographyAndStroke => "REQUIRE_TYPOGRAPHY_AND_STROKE_VARIABLES",
        }
    }
}

/// The edit that resolves a finding.
#[derive(Clone, Debug, PartialEq)]
pub enum Fix {
    /// Bind the property `key` (`fills/0`, `strokes/1`, `itemSpacing`, …) to a variable.
    BindVariable { key: String, variable: String },
    /// Bind several properties at once (uniform corner radius or uniform padding) to a variable.
    BindVariables { keys: Vec<String>, variable: String },
    /// Link the layer to a shared style (fill, stroke, text or effect: the style knows which).
    ApplyStyle { style: String },
    /// Replace the text colour with `color` (alpha is kept).
    SetTextColor { color: Color },
    /// Replace the detached frame with an instance of `component`, keeping its name and size.
    Reconnect { component: NodeId },
}

/// One problem on one layer.
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    pub node: NodeId,
    pub node_name: String,
    pub rule: Rule,
    /// Property key within the rule (`"fills/0"`, `"paddingTop"`, `""` for whole-node rules).
    pub property: String,
    /// What is wrong, as shown in the row.
    pub message: String,
    /// The one-click fix, when there is one.
    pub fix: Option<Fix>,
    /// What the fix does, for its tooltip (empty without a fix).
    pub fix_label: String,
}

impl Finding {
    /// Whether this finding suggests an asset from an enabled library.
    pub fn is_from_library(&self) -> bool {
        self.message.contains(" from library \"")
    }

    /// Persistent ignore key for this finding.
    pub fn ignore_key(&self) -> IgnoreKey {
        IgnoreKey { node: self.node, rule: self.rule.figma_id().to_string(), property: self.property.clone() }
    }
}

/// What an audit looks at.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Every top-level layer of a page.
    Page(NodeId),
    /// These layers and everything below them.
    Layers(Vec<NodeId>),
}

/// Enabled library snapshots collected from the app cache before an audit.
#[derive(Clone, Debug, Default)]
pub struct LibrarySources {
    pub(super) libs: Vec<(LibraryRef, Arc<LibrarySnapshot>)>,
    pub(super) known_names: HashMap<String, String>,
}

/// Result of one audit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Report {
    pub findings: Vec<Finding>,
    /// Findings in the audited scope that were suppressed by `doc.check_designs.ignored`.
    pub ignored: Vec<Finding>,
    /// Layers examined.
    pub scanned: usize,
    /// The scan stopped at [`MAX_SCANNED_LAYERS`].
    pub truncated: bool,
    /// Display names of enabled libraries whose pinned snapshot could not be read.
    pub unreadable_libraries: Vec<String>,
}

impl Report {
    /// Findings of one rule, in document order.
    pub fn of(&self, rule: Rule) -> impl Iterator<Item = &Finding> {
        self.findings.iter().filter(move |f| f.rule == rule)
    }

    /// Findings that carry a fix.
    pub fn fixable(&self) -> Vec<&Finding> {
        self.findings.iter().filter(|f| f.fix.is_some()).collect()
    }
}

/// Scans `scope` with the document's own variables and styles only (the app always audits with
/// its enabled libraries through [`audit_with_libraries`]). Hidden layers are skipped; layers inside
/// an instance are audited only where their effective properties or backdrop diverge from their
/// main component.
#[cfg(test)]
pub fn audit(doc: &Document, scope: &Scope) -> Report {
    audit_with_libraries(doc, scope, &LibrarySources::default())
}

/// Scans `scope` and includes variables and styles from enabled `libs` snapshots.
pub fn audit_with_libraries(doc: &Document, scope: &Scope, libs: &LibrarySources) -> Report {
    let roots: Vec<NodeId> = match scope {
        Scope::Page(page) => doc.get(*page).map(|p| p.children.clone()).unwrap_or_default(),
        Scope::Layers(ids) => ids.clone(),
    };
    let index = hardcoded::Index::with_libraries(doc, libs);
    let components = detached::Components::new(doc);
    let bounds = contrast::SubtreeBounds::default();
    let contrast_on = !doc.check_designs.disabled_rules.contains(Rule::Contrast.figma_id());
    let detached_on = !doc.check_designs.disabled_rules.contains(Rule::DetachedInstance.figma_id());
    let lib_assets_on = !doc.check_designs.disabled_rules.contains(Rule::LibraryAssets.figma_id());
    let mut all_findings = Vec::new();
    let mut report = Report::default();
    let mut visited = 0usize;
    let mut stack: Vec<NodeId> = roots.into_iter().rev().collect();
    while let Some(id) = stack.pop() {
        let Some(n) = doc.get(id) else { continue };
        if !n.visible || matches!(n.kind, NodeKind::Page { .. }) {
            continue;
        }
        if visited >= MAX_SCANNED_LAYERS {
            report.scanned = visited;
            report.truncated = true;
            break;
        }
        visited += 1;
        let before = all_findings.len();
        hardcoded::check(doc, &index, n, &mut all_findings);
        if contrast_on {
            contrast::check(doc, &bounds, n, &mut all_findings);
        }
        if detached_on {
            detached::check(doc, &components, n, &mut all_findings);
        }
        if lib_assets_on {
            library_assets::check(doc, libs, n, n.component.source.and_then(|s| doc.get(s)), &mut all_findings);
        }
        if n.component.source.is_none() || !n.component.overrides.is_empty() || all_findings.len() > before {
            report.scanned += 1;
        }
        stack.extend(n.children.iter().rev().copied());
    }
    for f in all_findings {
        if doc.check_designs.ignored.contains(&f.ignore_key()) {
            report.ignored.push(f);
        } else {
            report.findings.push(f);
        }
    }
    report
}
