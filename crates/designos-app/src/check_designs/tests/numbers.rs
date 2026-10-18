//! Literal fixtures for hard-coded number variables in Check designs (#154): gap, padding, corner
//! radius, stroke weight, font size and line height, plus scope filtering, ambiguity and undo.

use super::*;
use designos_doc::{AutoLayout, LayoutDirection, LineHeight, TextRun};

impl Fx {
    /// A number variable with one value per mode of the shared "Theme" collection (light default, dark)
    /// and the given Figma `scopes` (empty = all scopes).
    fn num_var(&mut self, name: &str, light: f32, dark: f32, scopes: &[&str]) -> String {
        let coll = self.doc.variable_collections.entry("theme".into()).or_insert_with(|| VariableCollection {
            id: "theme".into(),
            name: "Theme".into(),
            modes: vec![VariableMode { id: "light".into(), name: "Light".into() }, VariableMode { id: "dark".into(), name: "Dark".into() }],
            default_mode: "light".into(),
            variables: vec![],
            key: None,
            source_library_key: None,
        });
        let id = format!("var:{name}");
        if !coll.variables.contains(&id) {
            coll.variables.push(id.clone());
        }
        let values = [("light".to_string(), VariableValue::Number(light)), ("dark".to_string(), VariableValue::Number(dark))].into_iter().collect();
        self.doc.variables.insert(
            id.clone(),
            Variable {
                id: id.clone(),
                name: name.into(),
                collection: "theme".into(),
                kind: VariableKind::Number,
                values,
                description: String::new(),
                scopes: scopes.iter().map(|s| (*s).to_string()).collect(),
                hidden_from_publishing: false,
                key: None,
                code_syntax: Default::default(),
            },
        );
        id
    }
}

#[test]
fn a_sixteen_px_gap_matching_a_number_variable_suggests_binding_it() {
    let mut f = fx();
    let row = f.frame(f.page, "Row", None);
    f.doc.get_mut(row).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { item_spacing: 16.0, ..AutoLayout::v1() }), ..FrameProps::default() });
    let var = f.num_var("space/4", 16.0, 16.0, &["GAP"]);
    let found = only(&f.report(), Rule::HardCodedValue);
    assert_eq!(found.node, row);
    assert_eq!(found.fix, Some(Fix::BindVariable { key: "itemSpacing".into(), variable: var.clone() }));
    assert_eq!(found.message, "Gap 16 px matches variable \"space/4\"");
    assert_eq!(found.fix_label, "Bind to variable space/4");

    let mut app = f.app();
    let before = snapshot(&app, &[row]);
    assert!(app.check_designs_fix(&found));
    assert_eq!(app.doc.get(row).unwrap().variables.bindings.get("itemSpacing"), Some(&var));
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty(), "the finding is gone after the fix");
    app.undo();
    assert_eq!(snapshot(&app, &[row]), before);
    assert!(!app.hist.can_undo(), "exactly one undo step");
}

#[test]
fn counter_axis_gap_is_checked_on_wrapped_and_grid_frames_only() {
    let mut f = fx();
    let plain = f.frame(f.page, "Plain row", None);
    f.doc.get_mut(plain).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { counter_spacing: 12.0, wrap: false, ..AutoLayout::v1() }), ..FrameProps::default() });
    let wrapped = f.frame(f.page, "Wrapped row", None);
    f.doc.get_mut(wrapped).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { counter_spacing: 12.0, wrap: true, ..AutoLayout::v1() }), ..FrameProps::default() });
    let grid = f.frame(f.page, "Grid", None);
    f.doc.get_mut(grid).unwrap().kind = NodeKind::Frame(FrameProps {
        auto_layout: Some(AutoLayout { direction: LayoutDirection::Grid, counter_spacing: 12.0, grid_columns: 2, ..AutoLayout::v1() }),
        ..FrameProps::default()
    });
    let var = f.num_var("space/3", 12.0, 12.0, &["GAP"]);
    let report = f.report();
    let nodes: Vec<NodeId> = report.of(Rule::HardCodedValue).map(|x| x.node).collect();
    assert_eq!(nodes, vec![wrapped, grid], "counter_spacing is inactive on a non-wrapping flow frame");
    for found in report.of(Rule::HardCodedValue) {
        assert_eq!(found.fix, Some(Fix::BindVariable { key: "counterAxisSpacing".into(), variable: var.clone() }));
    }
}

#[test]
fn uniform_and_per_side_padding_match_number_variables_and_bind_cleanly() {
    let mut f = fx();
    let card = f.frame(f.page, "Card", None);
    f.doc.get_mut(card).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { padding: [24.0; 4], ..AutoLayout::v1() }), ..FrameProps::default() });
    let var24 = f.num_var("space/6", 24.0, 24.0, &["GAP"]);
    let found = only(&f.report(), Rule::HardCodedValue);
    assert_eq!(found.node, card);
    assert_eq!(found.message, "Padding 24 px matches variable \"space/6\"");

    let mut app = f.app();
    let before = snapshot(&app, &[card]);
    assert!(app.check_designs_fix(&found));
    let n = app.doc.get(card).unwrap();
    for k in ["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"] {
        assert_eq!(n.variables.bindings.get(k), Some(&var24), "{k} should be bound");
    }
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());
    app.undo();
    assert_eq!(snapshot(&app, &[card]), before);

    // Per-side padding when sides differ.
    let mut f = fx();
    let bar = f.frame(f.page, "Bar", None);
    f.doc.get_mut(bar).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { padding: [8.0, 16.0, 0.0, 0.0], ..AutoLayout::v1() }), ..FrameProps::default() });
    let var8 = f.num_var("space/2", 8.0, 8.0, &["GAP"]);
    let var16 = f.num_var("space/4", 16.0, 16.0, &["GAP"]);
    let report = f.report();
    assert_eq!(report.findings.len(), 2, "{:#?}", report.findings);
    assert_eq!(report.findings[0].fix, Some(Fix::BindVariable { key: "paddingTop".into(), variable: var8.clone() }));
    assert_eq!(report.findings[1].fix, Some(Fix::BindVariable { key: "paddingRight".into(), variable: var16.clone() }));
    let mut app = f.app();
    assert_eq!(app.check_designs_fix_all(&report.findings), 2);
    let n = app.doc.get(bar).unwrap();
    assert_eq!(n.variables.bindings.get("paddingTop"), Some(&var8));
    assert_eq!(n.variables.bindings.get("paddingRight"), Some(&var16));
    assert!(!n.variables.bindings.contains_key("paddingBottom"));
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());
}

#[test]
fn uniform_and_per_corner_radius_match_number_variables_and_bind_cleanly() {
    let mut f = fx();
    let r = f.rect(f.page, "Chip", BLUE);
    f.doc.get_mut(r).unwrap().corner_radii = [12.0; 4];
    let var12 = f.num_var("radius/lg", 12.0, 12.0, &["CORNER_RADIUS"]);
    let found = only(&f.report(), Rule::HardCodedValue);
    assert_eq!(found.node, r);
    assert_eq!(found.message, "Corner radius 12 px matches variable \"radius/lg\"");

    let mut app = f.app();
    let before = snapshot(&app, &[r]);
    assert!(app.check_designs_fix(&found));
    let n = app.doc.get(r).unwrap();
    for k in ["topLeftRadius", "topRightRadius", "bottomRightRadius", "bottomLeftRadius"] {
        assert_eq!(n.variables.bindings.get(k), Some(&var12), "{k} should be bound");
    }
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());
    app.undo();
    assert_eq!(snapshot(&app, &[r]), before);

    // Per-corner radius when corners differ.
    let mut f = fx();
    let tab = f.rect(f.page, "Tab", BLUE);
    f.doc.get_mut(tab).unwrap().corner_radii = [8.0, 8.0, 0.0, 0.0];
    let var8 = f.num_var("radius/md", 8.0, 8.0, &["CORNER_RADIUS"]);
    let report = f.report();
    assert_eq!(report.findings.len(), 2, "{:#?}", report.findings);
    assert_eq!(report.findings[0].fix, Some(Fix::BindVariable { key: "topLeftRadius".into(), variable: var8.clone() }));
    assert_eq!(report.findings[1].fix, Some(Fix::BindVariable { key: "topRightRadius".into(), variable: var8.clone() }));
    let mut app = f.app();
    assert_eq!(app.check_designs_fix_all(&report.findings), 2);
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());
}

#[test]
fn uniform_and_per_side_stroke_weights_match_number_variables() {
    let mut f = fx();
    let a = f.rect(f.page, "Box", BLUE);
    f.doc.get_mut(a).unwrap().stroke.weight = 2.0;
    let b = f.rect(f.page, "Divider", BLUE);
    f.doc.get_mut(b).unwrap().stroke.side_weights = Some([0.0, 0.0, 2.0, 0.0]);
    let var = f.num_var("border/thick", 2.0, 2.0, &["STROKE_FLOAT"]);
    let report = f.report();
    assert_eq!(report.findings.len(), 2, "{:#?}", report.findings);
    let fa = report.findings.iter().find(|x| x.node == a).unwrap();
    assert_eq!(fa.fix, Some(Fix::BindVariable { key: "strokeWeight".into(), variable: var.clone() }));
    assert_eq!(fa.message, "Stroke weight 2 px matches variable \"border/thick\"");
    let fb = report.findings.iter().find(|x| x.node == b).unwrap();
    assert_eq!(fb.fix, Some(Fix::BindVariable { key: "strokeBottomWeight".into(), variable: var.clone() }));
    assert_eq!(fb.message, "Stroke bottom weight 2 px matches variable \"border/thick\"");

    let mut app = f.app();
    assert_eq!(app.check_designs_fix_all(&report.findings), 2);
    assert_eq!(app.doc.get(a).unwrap().variables.bindings.get("strokeWeight"), Some(&var));
    assert_eq!(app.doc.get(b).unwrap().variables.bindings.get("strokeBottomWeight"), Some(&var));
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());
}

#[test]
fn font_size_and_pixel_line_height_match_number_variables() {
    let mut f = fx();
    let white = f.frame(f.page, "Surface", Some(Color::WHITE));
    let t = f.text(white, "Title", 20.0, 600, gray(0x11));
    if let NodeKind::Text(tp) = &mut f.doc.get_mut(t).unwrap().kind {
        tp.style.line_height = LineHeight::Pixels(28.0);
    }
    let fs_var = f.num_var("font/size/lg", 20.0, 20.0, &["FONT_SIZE"]);
    let lh_var = f.num_var("font/line/lg", 28.0, 28.0, &["LINE_HEIGHT"]);
    let report = f.report();
    assert_eq!(report.findings.len(), 2, "{:#?}", report.findings);
    assert_eq!(report.findings[0].fix, Some(Fix::BindVariable { key: "fontSize".into(), variable: fs_var.clone() }));
    assert_eq!(report.findings[0].message, "Font size 20 px matches variable \"font/size/lg\"");
    assert_eq!(report.findings[1].fix, Some(Fix::BindVariable { key: "lineHeight".into(), variable: lh_var.clone() }));
    assert_eq!(report.findings[1].message, "Line height 28 px matches variable \"font/line/lg\"");

    let mut app = f.app();
    let before = snapshot(&app, &[t]);
    assert_eq!(app.check_designs_fix_all(&report.findings), 2);
    let bound = &app.doc.get(t).unwrap().variables.bindings;
    assert_eq!(bound.get("fontSize"), Some(&fs_var));
    assert_eq!(bound.get("lineHeight"), Some(&lh_var));
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());
    app.undo();
    assert_eq!(snapshot(&app, &[t]), before);
}

#[test]
fn text_linked_to_a_style_or_with_mixed_run_sizes_is_not_flagged_for_number_variables() {
    let mut f = fx();
    let white = f.frame(f.page, "Surface", Some(Color::WHITE));
    let linked = f.text(white, "Linked", 20.0, 600, gray(0x11));
    f.style("H3", "Heading/H3", Style::Text(TextStyle { font_size: 20.0, font_weight: 600, ..TextStyle::default() }), StyleProperty::Text);
    f.doc.get_mut(linked).unwrap().style_refs.insert("text".into(), "H3".into());

    let mixed = f.text(white, "Mixed", 20.0, 600, gray(0x11));
    if let NodeKind::Text(tp) = &mut f.doc.get_mut(mixed).unwrap().kind {
        tp.runs = vec![TextRun { start: 0, end: 2, style: TextStyle { font_size: 14.0, ..tp.style.clone() } }];
    }
    f.num_var("font/size/lg", 20.0, 20.0, &["FONT_SIZE"]);
    assert!(f.report().findings.is_empty(), "{:#?}", f.report().findings);
}

#[test]
fn number_variable_scopes_and_exact_value_matching_are_honoured() {
    let mut f = fx();
    let row = f.frame(f.page, "Row", None);
    f.doc.get_mut(row).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { item_spacing: 16.0, ..AutoLayout::v1() }), ..FrameProps::default() });
    f.doc.get_mut(row).unwrap().corner_radii = [15.0; 4];
    // 16.0 variable scoped to CORNER_RADIUS must not suggest on itemSpacing, and 15.0 radius does not match 16.0.
    f.num_var("radius/xl", 16.0, 16.0, &["CORNER_RADIUS"]);
    assert!(f.report().findings.is_empty(), "{:#?}", f.report().findings);
}

#[test]
fn ambiguous_number_variables_report_without_a_fix_and_constant_mode_wins() {
    let mut f = fx();
    let row = f.frame(f.page, "Row", None);
    f.doc.get_mut(row).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { item_spacing: 16.0, ..AutoLayout::v1() }), ..FrameProps::default() });
    f.num_var("space/4", 16.0, 16.0, &["GAP"]);
    f.num_var("space/md", 16.0, 16.0, &["GAP"]);
    let found = only(&f.report(), Rule::HardCodedValue);
    assert_eq!(found.fix, None);
    assert!(found.fix_label.is_empty());
    assert_eq!(found.message, "Gap 16 px matches 2 variables: \"space/4\", \"space/md\"");

    // When one variable holds 16 in every mode and the other changes in dark mode, the constant one wins.
    let mut f = fx();
    let row = f.frame(f.page, "Row", None);
    f.doc.get_mut(row).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { item_spacing: 16.0, ..AutoLayout::v1() }), ..FrameProps::default() });
    f.num_var("space/responsive", 16.0, 24.0, &["GAP"]);
    let steady = f.num_var("space/4", 16.0, 16.0, &["GAP"]);
    let found = only(&f.report(), Rule::HardCodedValue);
    assert_eq!(found.fix, Some(Fix::BindVariable { key: "itemSpacing".into(), variable: steady }));
}

#[test]
fn a_number_fix_for_a_property_edited_since_the_audit_is_refused() {
    let mut f = fx();
    let card = f.frame(f.page, "Card", None);
    f.doc.get_mut(card).unwrap().kind =
        NodeKind::Frame(FrameProps { auto_layout: Some(AutoLayout { item_spacing: 16.0, padding: [24.0; 4], ..AutoLayout::v1() }), ..FrameProps::default() });
    f.doc.get_mut(card).unwrap().corner_radii = [8.0; 4];
    f.num_var("space/4", 16.0, 16.0, &["GAP"]);
    f.num_var("space/6", 24.0, 24.0, &["GAP"]);
    f.num_var("radius/md", 8.0, 8.0, &["CORNER_RADIUS"]);
    let mut app = f.app();
    let findings = audit(&app.doc, &Scope::Page(app.page)).findings;
    assert_eq!(findings.len(), 3, "{findings:#?}");

    // Edit gap, one padding side, and one corner radius after the audit: all three fixes must refuse.
    let n = app.doc.get_mut(card).unwrap();
    n.corner_radii[2] = 4.0;
    if let Some(al) = n.kind.frame_props_mut().and_then(|fp| fp.auto_layout.as_mut()) {
        al.item_spacing = 20.0;
        al.padding[1] = 12.0;
    }
    for finding in &findings {
        assert!(!app.check_designs_fix(finding), "stale finding must be refused: {finding:?}");
    }
    assert!(app.doc.get(card).unwrap().variables.bindings.is_empty());
    assert!(!app.hist.can_undo());
}
