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
#[ignore = "needs the private Untitled UI .fig"]
fn untitled_ui_instance_text_overrides_resolve() {
    let bytes = reference("untitled-ui.fig")
        .or_else(|| std::fs::read("/Users/jangtrinh/Downloads/❖ Untitled UI Figma – PRO VARIABLES (v8.0) gKXp7LqFY2NV (Copy).fig").ok());
    let Some(bytes) = bytes else {
        eprintln!("untitled-ui.fig not found: skipped");
        return;
    };
    let mut doc = import_fig(&bytes).expect("import");
    for page in doc.pages.clone() {
        doc.ensure_page_hydrated(page);
    }

    for (inst_src, expected_snip, expect_runs) in [
        ("3936:417924", "For apps and websites with a lot of user-generated content", false),
        ("3936:417929", "Things go wrong. When they do, it can be a frustrating experience", false),
        ("11185:8546", "Your brand color and base white and black styles.", false),
        ("9352:550138", "This component has been moved to Base components -> Progress indicators.", true),
    ] {
        let inst = doc.nodes.values().find(|n| n.source_id.as_deref() == Some(inst_src)).unwrap_or_else(|| panic!("missing instance {inst_src}"));
        let texts: Vec<&TextProps> = subtree(&doc, inst.id)
            .into_iter()
            .filter_map(|n| match &n.kind {
                NodeKind::Text(t) => Some(t),
                _ => None,
            })
            .collect();
        assert!(
            texts.iter().all(|t| !t.content.starts_with("Lorem ipsum")),
            "instance {inst_src} still has un-overridden Lorem ipsum: {:?}",
            texts.iter().map(|t| &t.content).collect::<Vec<_>>()
        );
        let body = texts.iter().find(|t| t.content.contains(expected_snip)).unwrap_or_else(|| {
            panic!("instance {inst_src} missing expected text {expected_snip:?}; found: {:?}", texts.iter().map(|t| &t.content).collect::<Vec<_>>())
        });
        if expect_runs {
            assert!(!body.runs.is_empty(), "instance {inst_src} should preserve mixed-style runs");
            assert!(
                body.runs.iter().any(|r| r.style.font_weight >= 600),
                "instance {inst_src} should have bold/semibold run weights: {:?}",
                body.runs.iter().map(|r| r.style.font_weight).collect::<Vec<_>>()
            );
        }
    }

    // Across the entire file, instances of Design note (2680:401975) and Figma Variables note (5263:372534)
    // that override their supporting text resolve to the overridden copy instead of falling back to Lorem ipsum.
    let note_comp_ids: Vec<NodeId> =
        doc.nodes.values().filter(|n| matches!(n.source_id.as_deref(), Some("2680:401975" | "5263:372534"))).map(|n| n.id).collect();
    assert_eq!(note_comp_ids.len(), 2);
    let mut overridden_notes = 0usize;
    for n in doc.nodes.values() {
        if let NodeKind::Instance { component: Some(c), .. } = n.kind {
            if note_comp_ids.contains(&c) {
                let has_overridden_body = subtree(&doc, n.id).into_iter().any(|sub| {
                    matches!(&sub.kind, NodeKind::Text(t) if sub.component.overrides.contains(&OverrideField::Characters) && !t.content.starts_with("Lorem ipsum"))
                });
                if has_overridden_body {
                    overridden_notes += 1;
                }
            }
        }
    }
    assert!(overridden_notes >= 40, "expected >= 40 overridden note instances, found {overridden_notes}");
}
