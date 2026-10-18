//! Import checks against the owner's private design files. They are ignored by
//! default and skip when a file is absent (the files are local and never
//! committed). Run with `cargo test -p designos-fig --test real_files -- --ignored`;
//! set `DESIGNOS_REFERENCE_DIR` to point at the folder holding them.

use designos_doc::*;
use designos_fig::import_fig;
use std::path::PathBuf;

fn reference(name: &str) -> Option<Vec<u8>> {
    let mut dirs: Vec<PathBuf> = std::env::var_os("DESIGNOS_REFERENCE_DIR").map(PathBuf::from).into_iter().collect();
    // `reference/` of this checkout or of any ancestor (worktrees live inside the main checkout).
    let mut cur = Some(PathBuf::from(env!("CARGO_MANIFEST_DIR")));
    while let Some(d) = cur {
        dirs.push(d.join("reference"));
        cur = d.parent().map(PathBuf::from);
    }
    dirs.into_iter().map(|d| d.join(name)).find_map(|p| std::fs::read(p).ok())
}

fn is_clone(n: &Node) -> bool {
    n.source_id.as_deref().is_some_and(|s| s.starts_with('I'))
}

fn subtree(doc: &Document, root: NodeId) -> Vec<&Node> {
    let mut out = Vec::new();
    doc.walk(root, |n, _| out.push(doc.get(n.id).unwrap()));
    out
}

#[test]
#[ignore = "needs the private reference/private-ui-kit.fig"]
fn private_ui_kit_imports_with_library_instances_resolved() {
    let Some(bytes) = reference("private-ui-kit.fig") else {
        eprintln!("private-ui-kit.fig not found: skipped");
        return;
    };
    let doc = import_fig(&bytes).expect("import");
    assert!(!doc.nodes.values().any(|n| matches!(n.kind, NodeKind::Unknown { .. })), "no Unknown node kinds");

    let kit = doc.pages.iter().copied().find(|p| doc.get(*p).unwrap().name.contains("UI Kit")).expect("UI Kit page");
    let own: Vec<&Node> = subtree(&doc, kit).into_iter().filter(|n| !is_clone(n)).collect();
    assert_eq!(own.len(), 224, "nodes authored on the UI Kit page");
    let instances: Vec<&&Node> = own.iter().filter(|n| matches!(n.kind, NodeKind::Instance { .. })).collect();
    assert_eq!(instances.len(), 66);
    for i in &instances {
        let comp = instance_component(i).unwrap_or_else(|| panic!("instance {:?} unresolved", i.name));
        assert!(is_library_node(&doc, comp), "points at a library component");
        assert!(!i.children.is_empty(), "instance {:?} has no children", i.name);
    }

    let lp = doc.library_page.expect("library page");
    assert!(!doc.pages.contains(&lp));
    assert!(doc.library.external.len() > 50, "remote components are external assets");
    assert!(doc.library.external.values().all(|e| e.library_id.starts_with(REMOTE_LIBRARY_PREFIX)));

    assert_eq!(doc.variables.len(), 112);
    assert_eq!(doc.variable_collections.len(), 8);
    assert!(
        doc.variables.values().any(|v| v.values.values().any(|x| matches!(x, VariableValue::Alias(t) if doc.variables.contains_key(t)))),
        "aliases resolve"
    );
    assert!(doc.nodes.values().any(|n| !n.variables.bindings.is_empty()), "nodes bind variables");
    assert_eq!(doc.images.len(), 34);
}

#[test]
#[ignore = "needs the private reference/shadcn-ui.fig"]
fn shadcn_ui_imports_with_local_components() {
    let Some(bytes) = reference("shadcn-ui.fig") else {
        eprintln!("shadcn-ui.fig not found: skipped");
        return;
    };
    let doc = import_fig(&bytes).expect("import");
    assert!(!doc.nodes.values().any(|n| matches!(n.kind, NodeKind::Unknown { .. })));
    assert_eq!(doc.pages.len(), 6);
    let (mut instances, mut resolved) = (0, 0);
    for n in doc.nodes.values().filter(|n| matches!(n.kind, NodeKind::Instance { .. })) {
        instances += 1;
        resolved += instance_component(n).is_some() as usize;
    }
    assert!(instances >= 400 && instances == resolved, "{resolved} of {instances} instances resolved");
    assert_eq!(doc.styles.len(), 35);
    assert!(doc.library.external.is_empty(), "everything is local");

    for (source, size, weight) in [("21:1325", 18.0, 600), ("13:2558", 16.0, 400)] {
        let n = doc.nodes.values().find(|n| n.source_id.as_deref() == Some(source)).expect("cover text");
        let NodeKind::Text(t) = &n.kind else { panic!("cover text node") };
        assert_eq!(t.style.font_size, size, "{} uses its linked style", n.name);
        assert_eq!(t.style.font_weight, weight);
    }
}

#[test]
#[ignore = "needs the private reference/untitled-ui.fig"]
fn untitled_ui_swapped_link_and_secondary_buttons_match_variant_shell() {
    let Some(bytes) = reference("untitled-ui.fig").or_else(|| reference("❖ Untitled UI Figma – PRO VARIABLES (v8.0) gKXp7LqFY2NV (Copy).fig")) else {
        eprintln!("untitled-ui.fig not found: skipped");
        return;
    };
    let mut doc = import_fig(&bytes).expect("import untitled-ui.fig");
    let mobile = doc.nodes.values().find(|n| n.source_id.as_deref() == Some("1685:415210")).expect("Settings pages 01 Mobile (1685:415210)").id;
    let page_of_mobile = doc.page_of(mobile).expect("page of mobile");
    doc.ensure_page_hydrated(page_of_mobile);

    // Detector across all hydrated instances: every instance pointing at a Link color / Link gray
    // button variant (`Hierarchy=Link`) must have an empty fill/stroke shell matching its variant.
    let mut link_buttons = 0usize;
    for inst in doc.nodes.values().filter(|n| matches!(n.kind, NodeKind::Instance { .. })) {
        let Some(comp_id) = instance_component(inst) else { continue };
        let Some(comp) = doc.get(comp_id) else { continue };
        if comp.name.contains("Hierarchy=Link") && comp.fills.is_empty() && !inst.component.overrides.contains(&OverrideField::Fills) {
            link_buttons += 1;
            assert!(inst.fills.is_empty(), "instance {:?} ({:?}) of {:?} kept stale fills: {:?}", inst.name, inst.source_id, comp.name, inst.fills);
            assert!(
                inst.stroke.paints.is_empty(),
                "instance {:?} ({:?}) of {:?} kept stale stroke: {:?}",
                inst.name,
                inst.source_id,
                comp.name,
                inst.stroke.paints
            );
        }
    }
    assert!(link_buttons >= 50, "expected >= 50 Link button instances, found {link_buttons}");

    // Specifically verify all 9 "View integration" buttons in Settings pages 01 Mobile (1685:415210).
    let mut view_integration_buttons = 0usize;
    for n in subtree(&doc, mobile) {
        if let NodeKind::Text(tp) = &n.kind {
            if tp.content == "View integration" {
                let mut cur = n.parent;
                while let Some(pid) = cur {
                    let p = doc.get(pid).unwrap();
                    if matches!(p.kind, NodeKind::Instance { .. }) && p.name.contains("Button") {
                        view_integration_buttons += 1;
                        let comp = doc.get(instance_component(p).expect("Button component")).unwrap();
                        assert!(comp.name.contains("Hierarchy=Link color"), "expected Hierarchy=Link color, got {:?}", comp.name);
                        assert!(p.fills.is_empty(), "View integration Button must have no fill");
                        assert!(p.stroke.paints.is_empty(), "View integration Button must have no stroke");
                        assert!(p.effects.is_empty(), "View integration Button must have no shadow");
                        break;
                    }
                    cur = p.parent;
                }
            }
        }
    }
    assert_eq!(view_integration_buttons, 9, "expected all 9 View integration link buttons in 1685:415210");
}
