//! A patch log is plain text any local process can append to, so replay refuses what a real
//! agent session cannot produce instead of letting it into the document.

use super::*;
use crate::{Command, DocError, Document, Node, NodeId, NodeKind, Rect, MAX_TREE_DEPTH, MIN_ID_HEADROOM};

const LAST_SAFE_ID: u64 = u64::MAX - MIN_ID_HEADROOM;

fn editor() -> (Document, NodeId) {
    let mut d = Document::new("guard");
    let page = d.add_page("Page");
    (d, page)
}

fn frame(id: u64) -> Node {
    Node::frame(NodeId(id), Rect::new(0.0, 0.0, 10.0, 10.0))
}

/// `nodes[child]` becomes a listed child of `nodes[parent]`.
fn link(nodes: &mut [Node], parent: usize, child: usize) {
    let (p, c) = (nodes[parent].id, nodes[child].id);
    nodes[parent].children.push(c);
    nodes[child].parent = Some(p);
}

fn insert(page: NodeId, nodes: Vec<Node>) -> Vec<LiveOp> {
    vec![LiveOp::Insert { parent: page, at: SiblingHint::default(), nodes }]
}

/// Replays `ops` and asserts the document is exactly as before, with one reported rejection.
fn assert_rejected(ops: Vec<LiveOp>, expect: &str) {
    let (mut doc, _) = editor();
    let before = doc.clone().to_json().unwrap();
    let patch = LivePatch::new(ops);
    let report = replay_live_patch(&mut doc, &patch, &mut LivePatchState::default(), &mut |d, c| c.apply(d).map(|_| ())).unwrap();
    assert_eq!(report.skipped.len(), 1, "{:?}", report.skipped);
    assert!(report.skipped[0].contains(expect), "{:?} should mention {expect:?}", report.skipped);
    assert_eq!((report.applied, report.inserted_roots.len()), (0, 0));
    assert_eq!(doc.to_json().unwrap(), before, "a rejected op must leave the document untouched");
}

#[test]
fn a_well_formed_subtree_still_inserts() {
    let (mut doc, page) = editor();
    let mut nodes = vec![frame(100), frame(101), frame(102)];
    link(&mut nodes, 0, 1);
    link(&mut nodes, 1, 2);
    let patch = LivePatch::new(insert(page, nodes));
    let report = replay_live_patch(&mut doc, &patch, &mut LivePatchState::default(), &mut |d, c| c.apply(d).map(|_| ())).unwrap();
    assert!(report.skipped.is_empty(), "{:?}", report.skipped);
    assert_eq!(report.inserted_roots, [NodeId(100)]);
    assert_eq!(doc.get(NodeId(102)).and_then(|n| n.parent), Some(NodeId(101)));
}

#[test]
fn a_cycle_cut_off_from_the_root_is_rejected() {
    let (_, page) = editor();
    let mut nodes = vec![frame(100), frame(101), frame(102)];
    link(&mut nodes, 1, 2);
    link(&mut nodes, 2, 1);
    assert_rejected(insert(page, nodes), "not reachable");
}

#[test]
fn a_node_listed_by_a_parent_that_does_not_own_it_is_rejected() {
    let (_, page) = editor();
    let mut nodes = vec![frame(100), frame(101), frame(102)];
    link(&mut nodes, 0, 1);
    link(&mut nodes, 1, 2);
    nodes[2].children.push(NodeId(101));
    assert_rejected(insert(page, nodes), "101");
}

#[test]
fn a_duplicate_id_is_rejected() {
    let (_, page) = editor();
    let mut nodes = vec![frame(100), frame(101), frame(101)];
    link(&mut nodes, 0, 1);
    assert_rejected(insert(page, nodes), "duplicate");
}

#[test]
fn an_orphan_or_a_parent_outside_the_subtree_is_rejected() {
    let (_, page) = editor();
    let mut orphan = vec![frame(100), frame(101)];
    orphan[1].parent = None;
    assert_rejected(insert(page, orphan), "101");

    let mut elsewhere = vec![frame(100), frame(101)];
    elsewhere[1].parent = Some(page);
    assert_rejected(insert(page, elsewhere), "101");

    // Claims a parent that does not list it back.
    let mut unlisted = vec![frame(100), frame(101)];
    unlisted[1].parent = Some(NodeId(100));
    assert_rejected(insert(page, unlisted), "101");
}

#[test]
fn a_child_listed_twice_is_rejected() {
    let (_, page) = editor();
    let mut nodes = vec![frame(100), frame(101)];
    link(&mut nodes, 0, 1);
    nodes[0].children.push(NodeId(101));
    assert_rejected(insert(page, nodes), "twice");
}

#[test]
fn an_id_too_close_to_the_end_of_the_id_space_is_rejected() {
    let (_, page) = editor();
    for id in [u64::MAX, LAST_SAFE_ID + 1] {
        assert_rejected(insert(page, vec![frame(id)]), "id");
    }
    let (mut doc, _) = editor();
    let ok = LivePatch::new(insert(page, vec![frame(LAST_SAFE_ID)]));
    let report = replay_live_patch(&mut doc, &ok, &mut LivePatchState::default(), &mut |d, c| c.apply(d).map(|_| ())).unwrap();
    assert_eq!(report.applied, 1, "{:?}", report.skipped);
}

#[test]
fn a_subtree_nested_deeper_than_the_tree_limit_is_rejected() {
    let (_, page) = editor();
    let mut nodes: Vec<Node> = (0..MAX_TREE_DEPTH as u64 + 5).map(|i| frame(100 + i)).collect();
    for i in 1..nodes.len() {
        link(&mut nodes, i - 1, i);
    }
    assert_rejected(insert(page, nodes), "deep");
}

#[test]
fn turning_a_node_with_children_into_a_leaf_is_rejected() {
    let (mut doc, page) = editor();
    let mut nodes = vec![frame(100), frame(101)];
    link(&mut nodes, 0, 1);
    let patch = LivePatch::new(insert(page, nodes));
    replay_live_patch(&mut doc, &patch, &mut LivePatchState::default(), &mut |d, c| c.apply(d).map(|_| ())).unwrap();

    let leaf_kind = serde_json::to_value(Node::rectangle(NodeId(1), Rect::new(0.0, 0.0, 1.0, 1.0))).unwrap()["kind"].clone();
    let update = LiveOp::Update { id: NodeId(100), set: [("kind".to_string(), leaf_kind)].into_iter().collect(), unset: Vec::new() };
    let report = replay_live_patch(&mut doc, &LivePatch::new(vec![update]), &mut LivePatchState::default(), &mut |d, c| c.apply(d).map(|_| ())).unwrap();
    assert_eq!(report.skipped.len(), 1, "{:?}", report.skipped);
    assert!(report.skipped[0].contains("children"), "{:?}", report.skipped);
    assert_eq!(report.applied, 0);
    assert!(matches!(doc.get(NodeId(100)).unwrap().kind, NodeKind::Frame(_)));
    assert_eq!(doc.get(NodeId(100)).unwrap().children, [NodeId(101)]);
}

#[test]
fn a_failed_replay_leaves_the_remap_state_as_it_was() {
    let (mut doc, page) = editor();
    // The editor already uses id 100, so the agent's frame 100 is remapped.
    doc.reserve_ids(NodeId(99));
    doc.insert(page, frame(100)).unwrap();
    doc.get_mut(NodeId(100)).unwrap().name = "User frame".into();
    let mut state = LivePatchState::default();
    let before = state.clone();
    let mut calls = 0;
    let ops = vec![
        LiveOp::Insert { parent: page, at: SiblingHint::default(), nodes: vec![frame(100)] },
        LiveOp::Update { id: NodeId(100), set: [("name".to_string(), serde_json::json!("x"))].into_iter().collect(), unset: Vec::new() },
    ];
    let failing = &mut |d: &mut Document, c: Command| {
        calls += 1;
        if calls == 2 {
            return Err(DocError::NotFound(NodeId(100)));
        }
        c.apply(d).map(|_| ())
    };
    assert!(replay_live_patch(&mut doc, &LivePatch::new(ops), &mut state, failing).is_err());
    assert_eq!(calls, 2, "precondition: the failure came after the colliding insert");
    assert_eq!(state, before, "ids remapped by a patch that was cancelled must not outlive it");
}
