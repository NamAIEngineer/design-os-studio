//! Export entry points must return "nothing rendered" for absurd sizes instead of aborting.

use designos_doc::{Document, Node, NodeId, Rect};
use designos_export::{export_jpg, export_png, render_export};

fn doc_with_frame(w: f32, h: f32) -> (Document, NodeId) {
    let mut d = Document::new("Big");
    let page = d.add_page("P");
    let id = d.alloc_id();
    d.insert(page, Node::frame(id, Rect::new(0.0, 0.0, w, h))).unwrap();
    (d, id)
}

#[test]
fn huge_node_at_4x_is_rejected() {
    let (d, id) = doc_with_frame(1e6, 1e6);
    assert!(render_export(&d, id, 4.0).is_none());
    assert!(export_png(&d, id, 4.0).is_empty());
    assert!(export_jpg(&d, id, 4.0, 80).is_empty());
}

#[test]
fn page_and_effect_overflow_paths_are_capped_too() {
    let (mut d, id) = doc_with_frame(100.0, 100.0);
    let page = d.pages[0];
    d.get_mut(id).unwrap().size.x = 1e7;
    assert!(render_export(&d, page, 32.0).is_none(), "page path");
    assert!(render_export(&d, id, 32.0).is_none(), "node path");
}

#[test]
fn ordinary_export_still_works() {
    let (d, id) = doc_with_frame(30.0, 20.0);
    let pm = render_export(&d, id, 2.0).unwrap();
    assert_eq!((pm.width(), pm.height()), (60, 40));
    assert!(export_png(&d, id, 1.0).starts_with(b"\x89PNG"));
}
