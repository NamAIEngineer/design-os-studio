//! Auditing inside component instances (#52): un-overridden layers that merely mirror their main
//! component are reported once on the main component, while layers whose properties or effective
//! backdrop diverge in an instance are reported and fixed via instance overrides without mutating
//! the main component.

use super::*;
use designos_doc::{create_instance, sync_instances, OverrideField, Vec2};

fn spawn_instance(f: &mut Fx, comp: NodeId, x: f32, y: f32) -> NodeId {
    let parent = f.page;
    let cmd = create_instance(&mut f.doc, comp, parent, Vec2::new(x, y));
    let root = match &cmd {
        designos_doc::Command::Insert { nodes, .. } => nodes[0].id,
        other => panic!("expected Insert from create_instance, got {other:?}"),
    };
    cmd.apply(&mut f.doc).unwrap();
    root
}

#[test]
fn unoverridden_instance_child_with_main_component_issue_is_reported_once_on_main_component() {
    let mut f = fx();
    white_page(&mut f);
    f.color_var("Color/Primary", BLUE, BLUE);
    let comp = component(&mut f, "10:1");
    f.doc.get_mut(comp).unwrap().size = Vec2::new(200.0, 100.0);
    f.doc.get_mut(comp).unwrap().fills = vec![Paint::solid(Color::WHITE)];
    let comp_swatch = f.rect(comp, "Swatch", BLUE);
    let comp_label = f.text(comp, "Caption", 14.0, 400, gray(0x77));
    place(&mut f, comp_label, 60.0, 12.0);

    let _inst_a = spawn_instance(&mut f, comp, 260.0, 0.0);
    let _inst_b = spawn_instance(&mut f, comp, 520.0, 0.0);

    let report = f.report();
    let nodes: Vec<(Rule, NodeId)> = report.findings.iter().map(|x| (x.rule, x.node)).collect();
    assert_eq!(
        nodes,
        vec![(Rule::HardCodedValue, comp_swatch), (Rule::Contrast, comp_label)],
        "main-component issues must not be duplicated per un-overridden instance: {:#?}",
        report.findings
    );
}

#[test]
fn overridden_instance_child_text_or_fill_is_reported_on_the_instance_child() {
    let mut f = fx();
    white_page(&mut f);
    f.color_var("Color/Primary", BLUE, BLUE);
    let comp = component(&mut f, "10:2");
    f.doc.get_mut(comp).unwrap().size = Vec2::new(200.0, 100.0);
    f.doc.get_mut(comp).unwrap().fills = vec![Paint::solid(Color::WHITE)];
    let _comp_swatch = f.rect(comp, "Swatch", gray(0x20));
    let comp_label = f.text(comp, "Caption", 14.0, 400, gray(0x11));
    place(&mut f, comp_label, 60.0, 12.0);

    let inst = spawn_instance(&mut f, comp, 260.0, 0.0);
    let kids = f.doc.get(inst).unwrap().children.clone();
    let (inst_swatch, inst_label) = (kids[0], kids[1]);

    // Override swatch fill to hard-coded BLUE and label colour to low-contrast #777777 on the instance.
    {
        let s = f.doc.get_mut(inst_swatch).unwrap();
        s.fills = vec![Paint::solid(BLUE)];
        s.component.overrides.insert(OverrideField::Fills);
    }
    {
        let l = f.doc.get_mut(inst_label).unwrap();
        if let NodeKind::Text(t) = &mut l.kind {
            t.style.fills = vec![Paint::solid(gray(0x77))];
        }
        l.component.overrides.insert(OverrideField::TextStyle);
    }

    let report = f.report();
    let nodes: Vec<(Rule, NodeId)> = report.findings.iter().map(|x| (x.rule, x.node)).collect();
    assert_eq!(nodes, vec![(Rule::HardCodedValue, inst_swatch), (Rule::Contrast, inst_label)], "{:#?}", report.findings);
}

#[test]
fn transparent_instance_over_a_different_backdrop_reports_contrast_failure_on_instance_child() {
    let mut f = fx();
    white_page(&mut f);
    let dark_host = f.frame(f.page, "Dark host", Some(Color::BLACK));
    resize(&mut f, dark_host, 220.0, 100.0);
    place(&mut f, dark_host, 0.0, 0.0);
    let comp = component(&mut f, "10:3");
    // Place the transparent-fill main component inside the dark host where white text passes (21:1).
    f.doc.reparent(comp, dark_host, 0).unwrap();
    f.doc.get_mut(comp).unwrap().fills.clear();
    place(&mut f, comp, 10.0, 10.0);
    let _comp_label = f.text(comp, "Badge", 14.0, 400, Color::WHITE);

    // Place an un-overridden instance on the white page (outside dark_host), where white on white fails (1:1).
    let inst = spawn_instance(&mut f, comp, 300.0, 0.0);
    let inst_label = f.doc.get(inst).unwrap().children[0];

    let report = f.report();
    let found = only(&report, Rule::Contrast);
    assert_eq!(found.node, inst_label, "the instance sits on white while the main component sits on black");
}

#[test]
fn fixing_an_instance_child_records_override_without_mutating_main_component_and_undoes_cleanly() {
    let mut f = fx();
    white_page(&mut f);
    let blue_var = f.color_var("Color/Primary", BLUE, BLUE);
    let comp = component(&mut f, "10:4");
    f.doc.get_mut(comp).unwrap().size = Vec2::new(200.0, 100.0);
    f.doc.get_mut(comp).unwrap().fills = vec![Paint::solid(Color::WHITE)];
    let comp_swatch = f.rect(comp, "Swatch", gray(0x20));
    let comp_label = f.text(comp, "Caption", 14.0, 400, gray(0x11));
    place(&mut f, comp_label, 60.0, 12.0);

    let inst = spawn_instance(&mut f, comp, 260.0, 0.0);
    let kids = f.doc.get(inst).unwrap().children.clone();
    let (inst_swatch, inst_label) = (kids[0], kids[1]);

    // Override swatch fill to BLUE (without yet recording OverrideField::Fills to prove the fix records it)
    // and override label colour to #777777.
    f.doc.get_mut(inst_swatch).unwrap().fills = vec![Paint::solid(BLUE)];
    if let NodeKind::Text(t) = &mut f.doc.get_mut(inst_label).unwrap().kind {
        t.style.fills = vec![Paint::solid(gray(0x77))];
    }

    let mut app = f.app();
    let before_comp = snapshot(&app, &[comp_swatch, comp_label]);
    let before_inst = snapshot(&app, &[inst_swatch, inst_label]);
    let findings = audit(&app.doc, &Scope::Page(app.page)).findings;
    assert_eq!(findings.len(), 2, "{findings:#?}");

    assert_eq!(app.check_designs_fix_all(&findings), 2);
    assert_eq!(snapshot(&app, &[comp_swatch, comp_label]), before_comp, "main component must not be mutated");
    assert_eq!(app.doc.get(inst_swatch).unwrap().variables.bindings.get("fills/0"), Some(&blue_var));
    assert!(app.doc.get(inst_swatch).unwrap().component.overrides.contains(&OverrideField::Fills));
    assert!(app.doc.get(inst_label).unwrap().component.overrides.contains(&OverrideField::TextStyle));
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());

    // Syncing the main component must preserve the fixed overrides on the instance.
    sync_instances(&mut app.doc, comp).apply(&mut app.doc).unwrap();
    assert_eq!(app.doc.get(inst_swatch).unwrap().variables.bindings.get("fills/0"), Some(&blue_var));
    assert!(audit(&app.doc, &Scope::Page(app.page)).findings.is_empty());

    app.undo();
    assert_eq!(snapshot(&app, &[inst_swatch, inst_label]), before_inst);
    assert!(!app.hist.can_undo());
}
