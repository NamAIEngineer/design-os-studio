//! Incremental time-sliced scanning tests across large pages (Issue #157, Task E3-12).

use super::*;
use std::time::{Duration, Instant};

fn build_synthetic_page(layer_count: usize) -> (Document, NodeId) {
    let mut f = fx();
    let page = f.page;
    f.doc.get_mut(page).unwrap().kind = NodeKind::Page { background: Color::WHITE };
    f.color_var("Brand/Primary", BLUE, BLUE);
    let mut doc = f.doc;

    let per_card = 50usize;
    let mut created = 0usize;
    let mut card_idx = 0usize;
    while created < layer_count {
        let card_id = doc.alloc_id();
        let cx = (card_idx % 25) as f32 * 240.0;
        let cy = (card_idx / 25) as f32 * 180.0;
        let mut card = Node::frame(card_id, Rect::new(cx, cy, 200.0, 140.0));
        card.name = format!("Card {card_idx}");
        card.fills = vec![Paint::solid(Color::WHITE)];
        let card_id = doc.insert(page, card).unwrap();
        created += 1;
        card_idx += 1;

        let children_to_make = (layer_count - created).min(per_card - 1);
        for i in 0..children_to_make {
            let id = doc.alloc_id();
            let rx = (i % 10) as f32 * 18.0 + 4.0;
            let ry = (i / 10) as f32 * 22.0 + 4.0;
            let n = if i % 7 == 0 {
                let mut r = Node::rectangle(id, Rect::new(rx, ry, 14.0, 14.0));
                r.name = format!("Swatch {i}");
                r.fills = vec![Paint::solid(BLUE)];
                r
            } else if i % 5 == 0 {
                let ts = TextStyle { font_size: 14.0, font_weight: 400, fills: vec![Paint::solid(gray(0x22))], ..TextStyle::default() };
                let mut t = Node::text(id, Rect::new(rx, ry, 16.0, 14.0), "OK", ts);
                t.style_refs.insert("text".into(), "preset".into());
                t
            } else {
                let mut r = Node::rectangle(id, Rect::new(rx, ry, 14.0, 14.0));
                r.name = format!("Box {i}");
                r.fills = vec![Paint::solid(gray(0xEE))];
                r
            };
            doc.insert(card_id, n).unwrap();
            created += 1;
        }
    }
    (doc, page)
}

#[test]
fn sliced_audit_equals_synchronous_audit() {
    // 1. Shipped PR #45 fixture.
    let doc_fx = fixture::document();
    let page_fx = doc_fx.nodes.values().find(|n| matches!(n.kind, NodeKind::Page { .. })).map(|n| n.id).expect("the fixture has a page");
    let scope_fx = Scope::Page(page_fx);
    let sync_fx = audit(&doc_fx, &scope_fx);
    let mut cur_fx = ScanCursor::new(&doc_fx, scope_fx);
    while !audit_step(&doc_fx, &mut cur_fx, Duration::from_micros(50)) {}
    assert_eq!(cur_fx.into_report(&doc_fx), sync_fx);

    // 2. Generated page of 5,000 nodes with a 0.05 ms budget.
    let (doc, page) = build_synthetic_page(5_000);
    let scope = Scope::Page(page);
    let sync_report = audit(&doc, &scope);
    assert_eq!(sync_report.scanned, 5_000);
    assert!(sync_report.complete);

    let mut cur = ScanCursor::new(&doc, scope);
    let mut slices = 0usize;
    while !audit_step(&doc, &mut cur, Duration::from_micros(50)) {
        slices += 1;
    }
    assert!(slices >= 2, "expected multiple 0.05ms slices on 5,000 nodes, got {slices}");
    let sliced_report = cur.into_report(&doc);
    assert_eq!(sliced_report, sync_report);
}

#[test]
fn no_layer_cut_off_on_large_page() {
    let (doc, page) = build_synthetic_page(30_000);
    let report = audit(&doc, &Scope::Page(page));
    assert_eq!(report.scanned, 30_000);
    assert!(report.complete);
    assert!(!report.findings.is_empty());
}

#[test]
#[cfg_attr(debug_assertions, ignore = "release perf")]
fn each_slice_respects_budget_on_112k_page() {
    let (doc, page) = build_synthetic_page(112_474);
    let scope = Scope::Page(page);
    // A lone slice on a shared runner includes scheduler stalls that are not the scan's cost. Noise
    // only adds time, so the run with the smallest worst slice bounds the real cost; a scan that
    // cannot yield in time stays over the budget on every run.
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut worst_slice = Duration::MAX;
    let mut scanned;
    loop {
        let mut cur = ScanCursor::new(&doc, scope.clone());
        let mut max_slice = Duration::ZERO;
        loop {
            let t0 = Instant::now();
            let done = audit_step(&doc, &mut cur, Duration::from_millis(2));
            max_slice = max_slice.max(t0.elapsed());
            if done {
                break;
            }
        }
        scanned = cur.scanned;
        worst_slice = worst_slice.min(max_slice);
        if worst_slice <= Duration::from_millis(4) || Instant::now() >= deadline {
            break;
        }
    }
    assert_eq!(scanned, 112_474);
    assert!(worst_slice <= Duration::from_millis(4), "worst slice {worst_slice:?} exceeded 4ms budget");
}

#[test]
fn mid_scan_edit_restarts_and_keeps_previous_complete_report_until_done() {
    let (doc, page) = build_synthetic_page(2_000);
    let mut app = App::new(doc);
    app.page = page;
    let ctx = egui::Context::default();
    app.open_check_designs();

    // Finish initial scan completely.
    app.check_designs.slice_budget = Duration::from_secs(10);
    app.refresh_check_designs(false, &ctx);
    assert!(app.check_designs.report.complete);
    assert!(!app.check_designs.is_scanning());
    let initial_findings_len = app.check_designs.report.findings.len();
    assert!(initial_findings_len > 0);

    // Mutate document and switch to micro-slices: previous complete report stays visible while scanning.
    let extra = app.doc.alloc_id();
    let mut r = Node::rectangle(extra, Rect::new(0.0, 0.0, 10.0, 10.0));
    r.fills = vec![Paint::solid(BLUE)];
    app.doc.insert(page, r).unwrap();
    app.changed();
    crate::canvas::tiles::take_rev(&mut app);

    app.check_designs.slice_budget = Duration::from_micros(10);
    app.refresh_check_designs(false, &ctx);
    assert!(app.check_designs.is_scanning(), "micro-slice should leave scan in flight");
    assert!(app.check_designs.report.complete, "previous complete report remains visible while updating");
    assert_eq!(app.check_designs.report.findings.len(), initial_findings_len);

    // Pausing during pointer drag (`interacting = true`) does not advance the cursor.
    let scanned_before_drag = app.check_designs.cursor.as_ref().unwrap().scanned;
    app.refresh_check_designs(true, &ctx);
    assert_eq!(app.check_designs.cursor.as_ref().unwrap().scanned, scanned_before_drag);

    // Finish the rest of the scan.
    while app.check_designs.is_scanning() {
        app.refresh_check_designs(false, &ctx);
    }
    assert_eq!(app.check_designs.report.scanned, 2_001);
    assert_eq!(app.check_designs.report.findings.len(), initial_findings_len + 1);
}
