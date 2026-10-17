//! Checks on a subtree an `Insert` op carries, before it becomes a command.
//!
//! The patch log is text any local process can append to, so an inserted subtree is untrusted:
//! a cycle or a shared child would make recursive walkers loop, an orphan or a second root
//! would put a node in the document that no page reaches, and an id near `u64::MAX` would use
//! up the id space.

use crate::{Document, Node, NodeId, MAX_TREE_DEPTH, MIN_ID_HEADROOM};
use std::collections::BTreeMap;

/// Largest node id an inserted node may carry (matches the load-time id-space check).
const LAST_SAFE_ID: u64 = u64::MAX - MIN_ID_HEADROOM;

/// Whether `nodes` (root first), inserted under `parent`, is a well-formed subtree: unique ids
/// in range, one root, every other node's parent inside the subtree and listing it exactly once,
/// every node reachable from the root, and the deepest node within [`MAX_TREE_DEPTH`] counted
/// from the page. Children a node lists that are outside the subtree are existing nodes and are
/// ignored here; the replay drops them.
pub(super) fn check_inserted_subtree(doc: &Document, parent: NodeId, nodes: &[Node]) -> Result<(), String> {
    let mut index: BTreeMap<NodeId, usize> = BTreeMap::new();
    for (i, n) in nodes.iter().enumerate() {
        if n.id.0 == 0 || n.id.0 > LAST_SAFE_ID {
            return Err(format!("node id {} is outside the range an edit may use", n.id.0));
        }
        if index.insert(n.id, i).is_some() {
            return Err(format!("duplicate node id {}", n.id.0));
        }
    }
    for (i, n) in nodes.iter().enumerate() {
        let mut listed = std::collections::BTreeSet::new();
        for c in &n.children {
            if !listed.insert(*c) {
                return Err(format!("node {} lists child {} twice", n.id.0, c.0));
            }
            let Some(&ci) = index.get(c) else { continue };
            if ci == 0 || nodes[ci].parent != Some(n.id) {
                return Err(format!("node {} lists child {} that does not belong to it", n.id.0, c.0));
            }
        }
        if i == 0 {
            continue;
        }
        let owner = n.parent.and_then(|p| index.get(&p)).map(|&pi| &nodes[pi]);
        match owner {
            None => return Err(format!("node {} has no parent inside the inserted subtree", n.id.0)),
            Some(p) if !p.children.contains(&n.id) => return Err(format!("node {}'s parent does not list it", n.id.0)),
            Some(_) => {}
        }
    }

    // Walk down from the root: whatever it does not reach is detached (a cycle or a stray node).
    let base = depth_of(doc, parent)?;
    let mut reached = 0;
    let mut stack = vec![(0usize, base + 1)];
    while let Some((i, depth)) = stack.pop() {
        if depth > MAX_TREE_DEPTH {
            return Err(format!("the subtree nests deeper than {MAX_TREE_DEPTH} levels"));
        }
        reached += 1;
        stack.extend(nodes[i].children.iter().filter_map(|c| index.get(c)).map(|&ci| (ci, depth + 1)));
    }
    if reached != nodes.len() {
        return Err(format!("{} of {} inserted nodes are not reachable from its root", nodes.len() - reached, nodes.len()));
    }
    Ok(())
}

/// Levels between `id` and its page (a page is 0), refusing a chain longer than the tree limit.
fn depth_of(doc: &Document, id: NodeId) -> Result<usize, String> {
    let mut depth = 0;
    let mut cur = doc.get(id).and_then(|n| n.parent);
    while let Some(p) = cur {
        depth += 1;
        if depth > MAX_TREE_DEPTH {
            return Err(format!("the insertion point nests deeper than {MAX_TREE_DEPTH} levels"));
        }
        cur = doc.get(p).and_then(|n| n.parent);
    }
    Ok(depth)
}
