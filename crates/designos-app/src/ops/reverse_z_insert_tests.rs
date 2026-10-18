//! New, duplicated, pasted and wrapped layers must land in front of their siblings in an
//! auto-layout frame that sets `reverse_z` ("First on top"), where the first stored child paints on top.

use super::*;
use designos_doc::{AutoLayout, Paint};

/// Page > auto-layout frame (100x100) > `n` absolutely placed rectangles that each cover the whole
/// frame, so every point inside it overlaps all of them. Rectangles are named `r0`, `r1`, ... in
/// stored order. With `reverse_z` the first stored child is in front.
fn stack(reverse_z: bool, n: usize) -> (App, NodeId, Vec<NodeId>) {
    let mut d = Document::new("t");
    let page = d.add_page("p");
    let hid = d.alloc_id();
    let mut host = Node::frame(hid, Rect::new(0.0, 0.0, 100.0, 100.0));
    if let NodeKind::Frame(fp) = &mut host.kind {
        fp.auto_layout = Some(AutoLayout { reverse_z, ..Default::default() });
    }
    let host = d.insert(page, host).unwrap();
    let mut ids = vec![];
    for i in 0..n {
        let id = d.alloc_id();
        let mut r = Node::rectangle(id, Rect::new(0.0, 0.0, 100.0, 100.0));
        r.layout_child.absolute = true;
        r.name = format!("r{i}");
        ids.push(d.insert(host, r).unwrap());
    }
    (App::new(d), host, ids)
}

fn stored(app: &App, host: NodeId) -> Vec<NodeId> {
    app.doc.get(host).unwrap().children.clone()
}

/// Children of `host` from the front-most paint position down.
fn front_to_back(app: &App, host: NodeId) -> Vec<NodeId> {
    app.doc.get(host).unwrap().children_front_to_back().copied().collect()
}

fn names(app: &App, ids: &[NodeId]) -> Vec<String> {
    ids.iter().map(|id| app.doc.get(*id).unwrap().name.clone()).collect()
}

/// Rectangles under the point, front-most first, as the canvas picks them.
fn picked_rects(app: &App) -> Vec<NodeId> {
    let hits = crate::canvas::hit::hit_all_under(&app.doc, app.page, v2(30.0, 30.0), None);
    hits.into_iter().filter(|id| app.doc.get(*id).is_some_and(|n| matches!(n.kind, NodeKind::Rectangle))).collect()
}

#[test]
fn duplicate_of_the_front_layer_is_the_new_front_layer() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        let front = front_to_back(&app, host)[0];
        assert!(ids.contains(&front));
        app.selection = vec![front];
        app.duplicate_selection();
        let dup = app.selection[0];
        assert_ne!(dup, front);
        assert_eq!(front_to_back(&app, host)[0], dup, "reverse_z={reverse_z}: the copy paints in front of everything");
        assert_eq!(picked_rects(&app)[0], dup, "reverse_z={reverse_z}: the copy is what a click picks");
        if reverse_z {
            assert_eq!(stored(&app, host)[0], dup, "front-first storage puts the copy at index 0");
        }
    }
}

#[test]
fn duplicate_of_a_middle_layer_sits_directly_in_front_of_the_original() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        let middle = ids[1];
        app.selection = vec![middle];
        app.duplicate_selection();
        let dup = app.selection[0];
        let order = front_to_back(&app, host);
        let at = order.iter().position(|c| *c == middle).unwrap();
        assert_eq!(order[at - 1], dup, "reverse_z={reverse_z}: {order:?}");
        let picked = picked_rects(&app);
        let at = picked.iter().position(|c| *c == middle).unwrap();
        assert_eq!(picked[at - 1], dup, "reverse_z={reverse_z}: a click reaches the copy before the original");
    }
}

#[test]
fn duplicate_is_one_undo_step() {
    let (mut app, host, ids) = stack(true, 3);
    app.selection = vec![ids[1]];
    app.duplicate_selection();
    assert_eq!(stored(&app, host).len(), 4);
    app.undo();
    assert_eq!(stored(&app, host), ids);
}

fn copy_into_selected_host(app: &mut App, host: NodeId, source: &[NodeId]) {
    app.selection = source.to_vec();
    app.copy_selection().unwrap();
    app.selection = vec![host];
}

#[test]
fn paste_into_a_frame_lands_in_front() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        copy_into_selected_host(&mut app, host, &[ids[1]]);
        app.paste(None);
        let pasted = app.selection[0];
        assert_eq!(app.doc.get(pasted).unwrap().parent, Some(host));
        assert_eq!(front_to_back(&app, host)[0], pasted, "reverse_z={reverse_z}");
        assert_eq!(picked_rects(&app)[0], pasted, "reverse_z={reverse_z}: a click picks the pasted layer");
        if reverse_z {
            assert_eq!(stored(&app, host)[0], pasted);
        }
        app.undo();
        assert_eq!(stored(&app, host), ids, "reverse_z={reverse_z}: one undo removes the paste");
    }
}

#[test]
fn pasting_two_layers_keeps_their_paint_order_in_every_frame() {
    let paint_order = |reverse_z: bool| {
        let (mut app, host, ids) = stack(reverse_z, 3);
        copy_into_selected_host(&mut app, host, &[ids[0], ids[1]]);
        app.paste(None);
        let pasted = app.selection.clone();
        assert_eq!(pasted.len(), 2);
        let order = front_to_back(&app, host);
        assert_eq!(
            &order[..2],
            &{
                let mut p = pasted.clone();
                p.sort_by_key(|id| order.iter().position(|c| c == id));
                p
            }[..],
            "both copies are in front of the originals"
        );
        names(&app, &order[..2])
    };
    assert_eq!(paint_order(true), paint_order(false), "same relative order as in a plain frame");
}

#[test]
fn paste_over_selection_lands_directly_in_front_of_the_selected_layer() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        app.selection = vec![ids[0]];
        app.copy_selection().unwrap();
        app.selection = vec![ids[1]];
        app.paste_over_selection();
        let pasted = app.selection[0];
        let order = front_to_back(&app, host);
        let at = order.iter().position(|c| *c == ids[1]).unwrap();
        assert_eq!(order[at - 1], pasted, "reverse_z={reverse_z}: {order:?}");
        let picked = picked_rects(&app);
        let at = picked.iter().position(|c| *c == ids[1]).unwrap();
        assert_eq!(picked[at - 1], pasted, "reverse_z={reverse_z}: the pasted layer is picked before the selected one");
    }
}

#[test]
fn paste_over_the_front_layer_becomes_the_front_layer() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z, 3);
        let front = front_to_back(&app, host)[0];
        app.selection = vec![front];
        app.copy_selection().unwrap();
        app.paste_over_selection();
        let pasted = app.selection[0];
        assert_eq!(front_to_back(&app, host)[0], pasted, "reverse_z={reverse_z}");
        assert_eq!(picked_rects(&app)[0], pasted, "reverse_z={reverse_z}");
    }
}

#[test]
fn paste_over_selection_without_a_selection_inside_a_frame_lands_in_front() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        app.selection = vec![ids[1]];
        app.copy_selection().unwrap();
        app.selection = vec![host];
        app.paste_over_selection();
        let pasted = app.selection[0];
        assert_eq!(app.doc.get(pasted).unwrap().parent, Some(host));
        assert_eq!(front_to_back(&app, host)[0], pasted, "reverse_z={reverse_z}");
    }
}

#[test]
fn paste_over_and_paste_to_replace_keep_the_paint_order_of_several_layers() {
    let over = |reverse_z: bool| {
        let (mut app, host, ids) = stack(reverse_z, 4);
        app.selection = vec![ids[0], ids[1]];
        app.copy_selection().unwrap();
        app.selection = vec![ids[2]];
        app.paste_over_selection();
        let order = front_to_back(&app, host);
        let at = order.iter().position(|c| *c == ids[2]).unwrap();
        names(&app, &order[at - 2..at])
    };
    assert_eq!(over(true), over(false), "paste over selection: same relative order as in a plain frame");

    let replace = |reverse_z: bool| {
        let (mut app, host, ids) = stack(reverse_z, 4);
        app.selection = vec![ids[0], ids[1]];
        app.copy_selection().unwrap();
        app.selection = vec![ids[2]];
        app.paste_to_replace();
        let pasted = app.selection.clone();
        assert_eq!(pasted.len(), 2);
        let order = front_to_back(&app, host);
        let mut found: Vec<NodeId> = pasted;
        found.sort_by_key(|id| order.iter().position(|c| c == id));
        names(&app, &found)
    };
    assert_eq!(replace(true), replace(false), "paste to replace: same relative order as in a plain frame");
}

/// Four overlapping layers r0..r3 in stored order; wraps r0 and r2 (r1 sits between them) and
/// returns the layers of the frame front to back, the new container shown as `G`.
fn wrap_front_to_back(reverse_z: bool, wrap: impl FnOnce(&mut App)) -> Vec<String> {
    let (mut app, host, ids) = stack(reverse_z, 4);
    app.selection = vec![ids[0], ids[2]];
    wrap(&mut app);
    assert_eq!(app.selection.len(), 1, "the wrapped result is selected");
    let wrapped = app.selection[0];
    front_to_back(&app, host).iter().map(|id| if *id == wrapped { "G".to_string() } else { app.doc.get(*id).unwrap().name.clone() }).collect()
}

/// The wrapper goes where the front-most wrapped layer was: in front of `r1` either way.
fn expected_wrap_order(reverse_z: bool) -> Vec<String> {
    // reverse_z: r0 is in front of everything. Plain: r3 is, and r2 is the front-most wrapped layer.
    if reverse_z { vec!["G", "r1", "r3"] } else { vec!["r3", "G", "r1"] }.into_iter().map(String::from).collect()
}

#[test]
fn grouping_puts_the_group_where_the_front_most_selected_layer_was() {
    for reverse_z in [true, false] {
        assert_eq!(wrap_front_to_back(reverse_z, |a| a.group_selection(false)), expected_wrap_order(reverse_z), "reverse_z={reverse_z}");
    }
}

#[test]
fn flattening_puts_the_vector_where_the_front_most_source_was() {
    for reverse_z in [true, false] {
        assert_eq!(wrap_front_to_back(reverse_z, |a| a.flatten_selection()), expected_wrap_order(reverse_z), "reverse_z={reverse_z}");
    }
}

#[test]
fn outlining_a_stroke_puts_the_outline_directly_in_front_of_the_shape() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        let target = ids[1];
        if let Some(n) = app.doc.get_mut(target) {
            n.stroke.paints = vec![Paint::solid(Color::BLACK)];
            n.stroke.weight = 4.0;
        }
        app.selection = vec![target];
        app.outline_stroke_selection();
        let outline = app.selection[0];
        assert_ne!(outline, target);
        let order = front_to_back(&app, host);
        let at = order.iter().position(|c| *c == target).unwrap();
        assert_eq!(order[at - 1], outline, "reverse_z={reverse_z}: {order:?}");
    }
}

#[test]
fn grouping_keeps_the_stacking_of_the_wrapped_layers() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        let before: Vec<String> = names(&app, &front_to_back(&app, host));
        app.selection = vec![ids[0], ids[1]];
        app.group_selection(false);
        let group = app.selection[0];
        let inside = names(&app, &front_to_back(&app, group));
        let expected: Vec<String> = before.iter().filter(|n| *n == "r0" || *n == "r1").cloned().collect();
        assert_eq!(inside, expected, "reverse_z={reverse_z}: front-to-back order inside the group matches the order they had");
    }
}

#[test]
fn ungrouping_keeps_the_stacking_of_the_released_layers() {
    for reverse_z in [true, false] {
        let (mut app, host, ids) = stack(reverse_z, 3);
        app.selection = vec![ids[0], ids[1]];
        app.group_selection(false);
        let group = app.selection[0];
        let inside = names(&app, &front_to_back(&app, group));
        app.selection = vec![group];
        app.ungroup_selection();
        let order = names(&app, &front_to_back(&app, host));
        let released: Vec<String> = order.iter().filter(|n| *n == "r0" || *n == "r1").cloned().collect();
        assert_eq!(released, inside, "reverse_z={reverse_z}: {order:?}");
    }
}
