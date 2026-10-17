//! Reviewer probes: patch inputs the guard may not cover.
use super::*;
use crate::{Document, Node, NodeId, Rect};

fn run(doc: &mut Document, ops: Vec<LiveOp>) -> LivePatchReport {
    replay_live_patch(doc, &LivePatch::new(ops), &mut LivePatchState::default(), &mut |d, c| c.apply(d).map(|_| ())).unwrap()
}
fn reload_summary(doc: &Document) -> Option<String> {
    let (_, report) = Document::from_json_with_report(&doc.to_json().unwrap()).unwrap();
    report.structural_summary().map(|s| s.to_string())
}

#[test]
fn probe_leaf_with_child_in_insert() {
    let mut doc = Document::new("p");
    let page = doc.add_page("Page");
    let mut r = Node::rectangle(NodeId(100), Rect::new(0.0, 0.0, 1.0, 1.0));
    let mut c = Node::rectangle(NodeId(101), Rect::new(0.0, 0.0, 1.0, 1.0));
    r.children.push(c.id);
    c.parent = Some(r.id);
    let rep = run(&mut doc, vec![LiveOp::Insert { parent: page, at: SiblingHint::default(), nodes: vec![r, c] }]);
    println!("LEAF_CHILD skipped={:?} applied={} reload={:?}", rep.skipped, rep.applied, reload_summary(&doc));
}

#[test]
fn probe_page_kind_insert_and_update() {
    let mut doc = Document::new("p");
    let page = doc.add_page("Page");
    let pg_kind = serde_json::to_value(doc.get(page).unwrap()).unwrap()["kind"].clone();
    let mut pnode: Node = doc.get(page).unwrap().clone();
    pnode.id = NodeId(100);
    pnode.parent = None;
    pnode.children.clear();
    let rep = run(&mut doc, vec![LiveOp::Insert { parent: page, at: SiblingHint::default(), nodes: vec![pnode] }]);
    println!("PAGE_INSERT skipped={:?} applied={} reload={:?}", rep.skipped, rep.applied, reload_summary(&doc));

    let mut doc = Document::new("p");
    let page = doc.add_page("Page");
    let f = Node::frame(NodeId(100), Rect::new(0.0, 0.0, 1.0, 1.0));
    run(&mut doc, vec![LiveOp::Insert { parent: page, at: SiblingHint::default(), nodes: vec![f] }]);
    let rep = run(&mut doc, vec![LiveOp::Update { id: NodeId(100), set: [("kind".to_string(), pg_kind)].into_iter().collect(), unset: vec![] }]);
    println!("PAGE_UPDATE skipped={:?} applied={} reload={:?}", rep.skipped, rep.applied, reload_summary(&doc));
}
