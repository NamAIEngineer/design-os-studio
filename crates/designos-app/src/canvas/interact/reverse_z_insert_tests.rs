//! Layers created on the canvas (draw tools, text, images, pen, Option-drag copies, drops into a
//! frame) must land in front of their siblings in an auto-layout frame that sets `reverse_z`
//! ("First on top"), where the first stored child paints on top.

use super::*;
use designos_doc::{AutoLayout, Node};

/// Page > auto-layout frame (100x100) > two absolutely placed rectangles that each cover the whole frame.
fn stack(reverse_z: bool) -> (App, NodeId, Vec<NodeId>) {
    let mut d = Document::new("t");
    let page = d.add_page("p");
    let hid = d.alloc_id();
    let mut host = Node::frame(hid, Rect::new(0.0, 0.0, 100.0, 100.0));
    if let NodeKind::Frame(fp) = &mut host.kind {
        fp.auto_layout = Some(AutoLayout { reverse_z, ..Default::default() });
    }
    let host = d.insert(page, host).unwrap();
    let mut ids = vec![];
    for _ in 0..2 {
        let id = d.alloc_id();
        let mut r = Node::rectangle(id, Rect::new(0.0, 0.0, 100.0, 100.0));
        r.layout_child.absolute = true;
        ids.push(d.insert(host, r).unwrap());
    }
    (App::new(d), host, ids)
}

fn front_most(app: &App, host: NodeId) -> NodeId {
    *app.doc.get(host).unwrap().children_front_to_back().next().unwrap()
}

/// The new layer paints in front of every sibling; in a front-first frame it is stored first.
fn assert_in_front(app: &App, host: NodeId, id: NodeId, reverse_z: bool) {
    assert_eq!(app.doc.get(id).unwrap().parent, Some(host), "reverse_z={reverse_z}: created inside the frame");
    assert_eq!(front_most(app, host), id, "reverse_z={reverse_z}: paints in front of its siblings");
    if reverse_z {
        assert_eq!(app.doc.get(host).unwrap().children[0], id, "front-first storage puts it at index 0");
    }
}

#[test]
fn draw_tools_create_the_shape_in_front() {
    for reverse_z in [true, false] {
        for tool in [Tool::Rectangle, Tool::Ellipse, Tool::Frame] {
            let (mut app, host, _) = stack(reverse_z);
            start_draw(&mut app, tool, v2(30.0, 30.0));
            let id = app.selection[0];
            assert_in_front(&app, host, id, reverse_z);
        }
    }
}

#[test]
fn new_text_is_created_in_front() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z);
        text_press(&mut app, v2(30.0, 30.0));
        let id = app.selection[0];
        assert!(matches!(app.doc.get(id).unwrap().kind, NodeKind::Text(_)));
        assert_in_front(&app, host, id, reverse_z);
    }
}

fn encode_png() -> Vec<u8> {
    let img = image::DynamicImage::ImageRgb8(image::RgbImage::from_fn(8, 4, |x, y| image::Rgb([x as u8 * 30, y as u8 * 60, 90])));
    let mut out = std::io::Cursor::new(Vec::new());
    img.write_to(&mut out, image::ImageFormat::Png).unwrap();
    out.into_inner()
}

#[test]
fn placed_image_is_created_in_front_and_is_what_a_click_picks() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z);
        let id = place_image_bytes(&mut app, v2(30.0, 30.0), "photo", encode_png()).unwrap();
        assert_in_front(&app, host, id, reverse_z);
        let at = crate::util::world_aabb(&app.doc, id);
        let hits = hit::hit_all_under(&app.doc, app.page, v2(at.x + at.w / 2.0, at.y + at.h / 2.0), None);
        let rect_hits: Vec<NodeId> = hits.into_iter().filter(|h| matches!(app.doc.get(*h).map(|n| &n.kind), Some(NodeKind::Rectangle))).collect();
        assert_eq!(rect_hits.first(), Some(&id), "reverse_z={reverse_z}: {rect_hits:?}");
    }
}

#[test]
fn pen_path_is_created_in_front() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z);
        let pts = [create::PenPoint::new(v2(20.0, 20.0)), create::PenPoint::new(v2(80.0, 60.0))];
        let mut tx = Tx::new();
        let id = create::commit_pen(&mut app, &mut tx, &pts, false).unwrap();
        app.run_tx(tx);
        assert_in_front(&app, host, id, reverse_z);
    }
}

#[test]
fn option_drag_copy_of_the_front_layer_is_the_new_front_layer() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z);
        let front = front_most(&app, host);
        app.selection = vec![front];
        let m = start_move(&mut app, front, v2(30.0, 30.0), true).unwrap();
        let copy = m.ids[0];
        assert_ne!(copy, front);
        assert_in_front(&app, host, copy, reverse_z);
        let hits = hit::hit_all_under(&app.doc, app.page, v2(30.0, 30.0), None);
        let rect_hits: Vec<NodeId> = hits.into_iter().filter(|h| matches!(app.doc.get(*h).map(|n| &n.kind), Some(NodeKind::Rectangle))).collect();
        assert_eq!(rect_hits.first(), Some(&copy), "reverse_z={reverse_z}: a click picks the copy");
    }
}

#[test]
fn option_drag_copy_of_a_back_layer_sits_directly_in_front_of_the_original() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z);
        let back = *app.doc.get(host).unwrap().children_front_to_back().last().unwrap();
        app.selection = vec![back];
        let m = start_move(&mut app, back, v2(30.0, 30.0), true).unwrap();
        let copy = m.ids[0];
        let order: Vec<NodeId> = app.doc.get(host).unwrap().children_front_to_back().copied().collect();
        let at = order.iter().position(|c| *c == back).unwrap();
        assert_eq!(order[at - 1], copy, "reverse_z={reverse_z}: {order:?}");
    }
}

#[test]
fn dragging_a_layer_into_a_frame_drops_it_in_front() {
    for reverse_z in [true, false] {
        let (mut app, host, _) = stack(reverse_z);
        let oid = app.doc.alloc_id();
        let outsider = app.doc.insert(app.page, Node::rectangle(oid, Rect::new(300.0, 300.0, 40.0, 40.0))).unwrap();
        app.selection = vec![outsider];
        let mut m = start_move(&mut app, outsider, v2(310.0, 310.0), false).unwrap();
        reparent_after_move(&mut app, &mut m, v2(30.0, 30.0));
        assert_in_front(&app, host, outsider, reverse_z);
    }
}
