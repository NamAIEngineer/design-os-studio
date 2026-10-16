//! Large-file performance benchmark for `SceneCompiler::compile` and `SceneIndex`.
//! Tests both a synthetic 50,000-node hierarchical design system page and
//! `/Users/jangtrinh/Downloads/VSF - PCP.fig` (112,474 nodes) when present on disk.

use designos_doc::*;
use designos_gpu::SceneCompiler;
use designos_render::Viewport;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BenchMetrics {
    label: &'static str,
    frames: usize,
    p50_ms: f64,
    p95_ms: f64,
    max_ms: f64,
    mean_ms: f64,
    draws: usize,
    programs: usize,
    triangles: usize,
}

fn summarize(label: &'static str, mut times_ms: Vec<f64>, draws: usize, programs: usize, triangles: usize) -> BenchMetrics {
    times_ms.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = times_ms.len().max(1);
    let p50_ms = times_ms[(n - 1) / 2];
    let p95_idx = ((n as f64 * 0.95).ceil() as usize).saturating_sub(1).min(n - 1);
    let p95_ms = times_ms[p95_idx];
    let max_ms = *times_ms.last().unwrap_or(&0.0);
    let mean_ms = times_ms.iter().sum::<f64>() / n as f64;
    BenchMetrics { label, frames: n, p50_ms, p95_ms, max_ms, mean_ms, draws, programs, triangles }
}

/// Builds a realistic 50,041-node hierarchical design system document:
/// - 1 Page with 40 top-level Section/Frame boards (8x5 grid, 1600x1200 each)
/// - 25 clipped component cards per board (1,000 cards total)
/// - 49 descendant nodes per card (header, avatar ellipse, vector icons, text labels, dividers, badges)
fn build_50k_design_system() -> (Document, NodeId, NodeId) {
    let mut doc = Document::new("50k Design System");
    let page = doc.add_page("Design System Overview");
    let mut drag_target = NodeId(0);

    let icon_path = Path {
        cmds: vec![
            PathCmd::MoveTo(Vec2::new(2.0, 8.0)),
            PathCmd::LineTo(Vec2::new(6.0, 12.0)),
            PathCmd::LineTo(Vec2::new(14.0, 4.0)),
            PathCmd::CubicTo(Vec2::new(12.0, 2.0), Vec2::new(8.0, 2.0), Vec2::new(4.0, 6.0)),
            PathCmd::Close,
        ],
        fill_rule: FillRule::NonZero,
    };

    for b_idx in 0..40 {
        let bx = (b_idx % 8) as f32 * 1800.0;
        let by = (b_idx / 8) as f32 * 1400.0;
        let board_id = doc.alloc_id();
        let mut board = Node::frame(board_id, Rect::new(bx, by, 1600.0, 1200.0));
        board.name = format!("Section Board {b_idx}");
        board.fills = vec![Paint::solid(Color::rgb(0xFA, 0xFA, 0xFC))];
        if let NodeKind::Frame(fp) = &mut board.kind {
            fp.clips_content = true;
        }
        doc.insert(page, board).unwrap();

        for c_idx in 0..25 {
            let cx = (c_idx % 5) as f32 * 310.0 + 20.0;
            let cy = (c_idx / 5) as f32 * 230.0 + 20.0;
            let card_id = doc.alloc_id();
            let mut card = Node::frame(card_id, Rect::new(cx, cy, 290.0, 210.0));
            card.name = format!("Card {b_idx}-{c_idx}");
            card.corner_radii = [10.0; 4];
            card.fills = vec![Paint::solid(Color::WHITE)];
            card.stroke.weight = 1.0;
            card.stroke.paints = vec![Paint::solid(Color::rgb(0xE4, 0xE4, 0xE7))];
            if let NodeKind::Frame(fp) = &mut card.kind {
                fp.clips_content = true;
            }
            doc.insert(board_id, card).unwrap();
            if b_idx == 10 && c_idx == 12 {
                drag_target = card_id;
            }

            // 49 child nodes inside each card (40 * 25 * (1 + 49) = 50,000 + 40 + 1 = 50,041 nodes)
            for k in 0..49 {
                let row = (k / 7) as f32;
                let col = (k % 7) as f32;
                let nx = 10.0 + col * 38.0;
                let ny = 10.0 + row * 28.0;
                let nid = doc.alloc_id();
                let node = match k % 5 {
                    0 => {
                        let mut v = Node::new(nid, NodeKind::Vector { paths: vec![icon_path.clone()], network: None });
                        v.transform = Affine::translate(nx, ny);
                        v.size = Vec2::new(14.0, 14.0);
                        v.fills = vec![Paint::solid(Color::rgb(0x3B, 0x82, 0xF6))];
                        v
                    }
                    1 => {
                        let style = TextStyle { font_size: 11.0, fills: vec![Paint::solid(Color::rgb(0x18, 0x18, 0x1B))], ..Default::default() };
                        Node::text(nid, Rect::new(nx, ny, 34.0, 16.0), "Label", style)
                    }
                    2 => {
                        let mut r = Node::rectangle(nid, Rect::new(nx, ny, 32.0, 18.0));
                        r.corner_radii = [4.0; 4];
                        r.fills = vec![Paint::solid(Color::rgb(0xF1, 0xF5, 0xF9))];
                        r
                    }
                    3 => {
                        let mut e = Node::ellipse(nid, Rect::new(nx, ny, 12.0, 12.0));
                        e.fills = vec![Paint::solid(Color::rgb(0x10, 0xB9, 0x81))];
                        e
                    }
                    _ => {
                        let mut d = Node::rectangle(nid, Rect::new(nx, ny, 34.0, 1.0));
                        d.fills = vec![Paint::solid(Color::rgb(0xE2, 0xE8, 0xF0))];
                        d
                    }
                };
                doc.insert(card_id, node).unwrap();
            }
        }
    }

    (doc, page, drag_target)
}

fn find_good_drag_target(doc: &Document, page: NodeId, center: Vec2) -> NodeId {
    let Some(p) = doc.get(page) else { return page };
    // Find a visible non-empty child or grandchild near the center
    let mut best = p.children.first().copied().unwrap_or(page);
    let mut best_dist = f32::MAX;
    for &top in &p.children {
        let Some(tn) = doc.get(top) else { continue };
        if !tn.visible {
            continue;
        }
        let cx = tn.transform.tx + tn.size.x * 0.5;
        let cy = tn.transform.ty + tn.size.y * 0.5;
        let d = (cx - center.x).hypot(cy - center.y);
        if d < best_dist {
            best_dist = d;
            best = tn.children.iter().copied().find(|&c| doc.get(c).is_some_and(|cn| cn.visible)).unwrap_or(top);
        }
    }
    best
}

fn run_benchmark_suite(suite_name: &str, doc: &mut Document, page: NodeId, default_drag_target: NodeId) -> Vec<BenchMetrics> {
    let mut compiler = SceneCompiler::new();
    compiler.geom.budget = Some(Duration::from_millis(8));
    let size = (1440, 900);
    let dpr = 2.0;

    // Initial index build to determine world bounds of the page
    let t_idx = Instant::now();
    compiler.index.ensure(doc, page, 1);
    let initial_index_ms = t_idx.elapsed().as_secs_f64() * 1000.0;

    let mut page_bb = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
    let mut dense_top_bb = None;
    let mut dense_top_id = None;
    let mut max_sub_nodes = 0usize;
    if let Some(p) = doc.get(page) {
        for &cid in &p.children {
            if let Some(b) = compiler.index.bounds_of(cid) {
                page_bb.0 = page_bb.0.min(b.0);
                page_bb.1 = page_bb.1.min(b.1);
                page_bb.2 = page_bb.2.max(b.2);
                page_bb.3 = page_bb.3.max(b.3);
                let cnt = doc.page_node_count(cid);
                if cnt > max_sub_nodes {
                    max_sub_nodes = cnt;
                    dense_top_bb = Some(b);
                    dense_top_id = Some(cid);
                }
            }
        }
    }
    if page_bb.0 > page_bb.2 {
        page_bb = (0.0, 0.0, 4000.0, 3000.0);
    }

    // Find the densest artboard-sized container on the page so Working/Detail/Drag viewports
    // always look directly at a dense cluster of visible UI nodes.
    let mut best_artboard: Option<(NodeId, (f32, f32, f32, f32), usize)> = None;
    doc.walk(page, |n, depth| {
        if depth == 0 || n.children.len() < 4 {
            return;
        }
        if let Some(b) = compiler.index.bounds_of(n.id) {
            let w = b.2 - b.0;
            let h = b.3 - b.1;
            if (300.0..=8000.0).contains(&w) && (300.0..=8000.0).contains(&h) {
                let score = n.children.len();
                if best_artboard.is_none_or(|(_, _, s)| score > s) {
                    best_artboard = Some((n.id, b, score));
                }
            }
        }
    });

    let focus_bb = dense_top_bb.filter(|_| max_sub_nodes > 5_000).unwrap_or(page_bb);
    let world_w = (focus_bb.2 - focus_bb.0).max(1000.0);
    let world_h = (focus_bb.3 - focus_bb.1).max(1000.0);
    let (center, drag_target, draw_parent) = if let Some((ab_id, ab_bb, _)) = best_artboard {
        let drag = doc.get(ab_id).and_then(|ab| ab.children.iter().copied().find(|&c| compiler.index.bounds_of(c).is_some())).unwrap_or(default_drag_target);
        (Vec2::new(ab_bb.0, ab_bb.1), drag, ab_id)
    } else {
        (Vec2::new(focus_bb.0 + world_w * 0.25, focus_bb.1 + world_h * 0.25), default_drag_target, page)
    };
    let overview_start = Vec2::new(center.x - 1200.0, center.y - 800.0);

    // Warm-up frame at each zoom bucket so steady-state panning & LOD are measured accurately,
    // while also recording the cold-start frame.
    let mut results = Vec::new();
    let mut cur_rev = 1u64;

    // 1. Overview Zoom (zoom = 0.08) — panning 10 frames
    {
        let zoom = 0.08f32;
        let vp_warm = Viewport { offset: overview_start, zoom, dpr };
        let _ = compiler.compile(doc, page, cur_rev, &vp_warm, size);

        let mut times = Vec::with_capacity(10);
        let mut last_stats = Default::default();
        for i in 0..10 {
            let vp = Viewport { offset: Vec2::new(overview_start.x + i as f32 * 600.0, overview_start.y + i as f32 * 400.0), zoom, dpr };
            let t0 = Instant::now();
            let frame = compiler.compile(doc, page, cur_rev, &vp, size);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            last_stats = frame.stats();
        }
        results.push(summarize("Overview Zoom (0.08x) Pan", times, last_stats.draws, last_stats.programs, last_stats.triangles));
    }

    // 2. Working Zoom (zoom = 0.50) — panning 10 frames
    {
        let zoom = 0.50f32;
        let vp_warm = Viewport { offset: center, zoom, dpr };
        let _ = compiler.compile(doc, page, cur_rev, &vp_warm, size);

        let mut times = Vec::with_capacity(10);
        let mut last_stats = Default::default();
        for i in 0..10 {
            let vp = Viewport { offset: Vec2::new(center.x + i as f32 * 120.0, center.y + i as f32 * 80.0), zoom, dpr };
            let t0 = Instant::now();
            let frame = compiler.compile(doc, page, cur_rev, &vp, size);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            last_stats = frame.stats();
        }
        results.push(summarize("Working Zoom (0.50x) Pan", times, last_stats.draws, last_stats.programs, last_stats.triangles));
    }

    // 3. Detail Zoom (zoom = 1.00) — panning 10 frames
    {
        let zoom = 1.00f32;
        let vp_warm = Viewport { offset: center, zoom, dpr };
        let _ = compiler.compile(doc, page, cur_rev, &vp_warm, size);

        let mut times = Vec::with_capacity(10);
        let mut last_stats = Default::default();
        for i in 0..10 {
            let vp = Viewport { offset: Vec2::new(center.x + i as f32 * 60.0, center.y + i as f32 * 40.0), zoom, dpr };
            let t0 = Instant::now();
            let frame = compiler.compile(doc, page, cur_rev, &vp, size);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            last_stats = frame.stats();
        }
        results.push(summarize("Detail Zoom (1.00x) Pan", times, last_stats.draws, last_stats.programs, last_stats.triangles));
    }

    // 4. Single-Node Drag (rev +1 mutation) — 5 frames at Working Zoom (0.50x)
    {
        let zoom = 0.50f32;
        let vp = Viewport { offset: center, zoom, dpr };
        let _ = compiler.compile(doc, page, cur_rev, &vp, size);

        let mut times = Vec::with_capacity(5);
        let mut last_stats = Default::default();
        for _ in 0..5u64 {
            cur_rev += 1;
            if let Some(n) = doc.get_mut(drag_target) {
                n.transform.tx += 4.0;
                n.transform.ty += 2.0;
            }
            let t0 = Instant::now();
            let frame = compiler.compile(doc, page, cur_rev, &vp, size);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            last_stats = frame.stats();
        }
        results.push(summarize("Single-Node Drag (rev+1)", times, last_stats.draws, last_stats.programs, last_stats.triangles));
    }

    // 5. Live Shape Creation + Resize + Delete (insert + 8 resize frames + remove) at Detail Zoom (1.00x)
    {
        let zoom = 1.00f32;
        let vp = Viewport { offset: center, zoom, dpr };
        let _ = compiler.compile(doc, page, cur_rev, &vp, size);

        let mut times = Vec::with_capacity(10);

        // Frame 1: Insert new 1x1 shape during Drag::Create start
        cur_rev += 1;
        let new_id = doc.try_alloc_id().unwrap();
        let mut rect_node = Node::rectangle(new_id, Rect::new(24.0, 24.0, 1.0, 1.0));
        rect_node.fills = vec![Paint::solid(Color::rgb(0x25, 0x63, 0xEB))];
        doc.insert(draw_parent, rect_node).unwrap();
        let t0 = Instant::now();
        let _ = compiler.compile(doc, page, cur_rev, &vp, size);
        times.push(t0.elapsed().as_secs_f64() * 1000.0);

        // Frames 2..=9: Live resize drag
        for step in 1..=8 {
            cur_rev += 1;
            if let Some(n) = doc.get_mut(new_id) {
                n.size = Vec2::new(1.0 + step as f32 * 24.0, 1.0 + step as f32 * 16.0);
            }
            let t0 = Instant::now();
            let _ = compiler.compile(doc, page, cur_rev, &vp, size);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
        }

        // Frame 10: Delete shape
        cur_rev += 1;
        doc.remove(new_id).unwrap();
        let t0 = Instant::now();
        let frame = compiler.compile(doc, page, cur_rev, &vp, size);
        times.push(t0.elapsed().as_secs_f64() * 1000.0);
        let last_stats = frame.stats();

        results.push(summarize("Create+Resize+Delete (10f)", times, last_stats.draws, last_stats.programs, last_stats.triangles));
    }

    // 6. Dragging an Entire Top-Level Section/Frame (`update_subtree` pure-translation fast-path)
    if let Some(top_section_id) = dense_top_id {
        let zoom = 0.50f32;
        let vp = Viewport { offset: center, zoom, dpr };
        let _ = compiler.compile(doc, page, cur_rev, &vp, size);

        let mut times = Vec::with_capacity(5);
        let mut last_stats = Default::default();
        for _ in 0..5u64 {
            cur_rev += 1;
            if let Some(n) = doc.get_mut(top_section_id) {
                n.transform.tx += 8.0;
                n.transform.ty += 4.0;
            }
            let t0 = Instant::now();
            let frame = compiler.compile(doc, page, cur_rev, &vp, size);
            times.push(t0.elapsed().as_secs_f64() * 1000.0);
            last_stats = frame.stats();
        }
        results.push(summarize("Top-Section Drag (rev+1)", times, last_stats.draws, last_stats.programs, last_stats.triangles));
    }

    let page_nodes = doc.page_node_count(page);
    let top_children = doc.get(page).map_or(0, |p| p.children.len());
    eprintln!("\n==========================================================================================");
    eprintln!(
        "BENCHMARK SUITE: {suite_name} (total doc nodes: {}, page nodes: {}, top-level: {}, initial index: {:.2} ms)",
        doc.nodes.len(),
        page_nodes,
        top_children,
        initial_index_ms
    );
    eprintln!("==========================================================================================");
    eprintln!(
        "| {:<26} | {:>6} | {:>8} | {:>8} | {:>8} | {:>8} | {:>7} | {:>5} | {:>9} |",
        "Scenario", "Frames", "p50 (ms)", "p95 (ms)", "max (ms)", "mean(ms)", "Draws", "Progs", "Triangles"
    );
    eprintln!("|----------------------------|--------|----------|----------|----------|----------|---------|-------|-----------|");
    for m in &results {
        eprintln!(
            "| {:<26} | {:>6} | {:>8.2} | {:>8.2} | {:>8.2} | {:>8.2} | {:>7} | {:>5} | {:>9} |",
            m.label, m.frames, m.p50_ms, m.p95_ms, m.max_ms, m.mean_ms, m.draws, m.programs, m.triangles
        );
    }
    eprintln!("==========================================================================================\n");

    results
}

#[test]
fn large_file_compile_benchmarks() {
    // 1. Synthetic 50,000-node Design System Benchmark
    let (mut doc_50k, page_50k, drag_50k) = build_50k_design_system();
    assert!(doc_50k.nodes.len() >= 50_000, "expected >= 50k nodes, got {}", doc_50k.nodes.len());
    let res_50k = run_benchmark_suite("Synthetic 50k Design System", &mut doc_50k, page_50k, drag_50k);

    // 2. Real-World 112k-Node File (`/Users/jangtrinh/Downloads/VSF - PCP.fig`) if present
    let fig_path = std::path::Path::new("/Users/jangtrinh/Downloads/VSF - PCP.fig");
    let mut res_fig_opt = None;
    if fig_path.exists() {
        let t_load = Instant::now();
        let bytes = std::fs::read(fig_path).expect("read VSF - PCP.fig");
        let mut doc_fig = designos_fig::import_fig(&bytes).expect("import VSF - PCP.fig");
        let load_ms = t_load.elapsed().as_secs_f64() * 1000.0;

        // Find the page with the largest subtree node count
        let (largest_page, max_page_nodes) =
            doc_fig.pages.iter().copied().map(|p| (p, doc_fig.page_node_count(p))).max_by_key(|&(_, count)| count).expect("at least one page");
        let page_name = doc_fig.get(largest_page).map(|n| n.name.clone()).unwrap_or_default();
        eprintln!(
            "Loaded VSF - PCP.fig in {:.1} ms: {} total nodes across {} pages; largest page {:?} ({:?}) has {} nodes",
            load_ms,
            doc_fig.nodes.len(),
            doc_fig.pages.len(),
            page_name,
            largest_page,
            max_page_nodes
        );

        let mut idx = designos_render::SceneIndex::new();
        idx.ensure(&doc_fig, largest_page, 1);
        let mut bb = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        if let Some(p) = doc_fig.get(largest_page) {
            for &cid in &p.children {
                if let Some(b) = idx.bounds_of(cid) {
                    bb.0 = bb.0.min(b.0);
                    bb.1 = bb.1.min(b.1);
                    bb.2 = bb.2.max(b.2);
                    bb.3 = bb.3.max(b.3);
                }
            }
        }
        let center = Vec2::new(bb.0 + (bb.2 - bb.0) * 0.35, bb.1 + (bb.3 - bb.1) * 0.35);
        let drag_fig = find_good_drag_target(&doc_fig, largest_page, center);

        res_fig_opt = Some(run_benchmark_suite("VSF - PCP.fig (Largest Page)", &mut doc_fig, largest_page, drag_fig));
    } else {
        eprintln!("NOTE: /Users/jangtrinh/Downloads/VSF - PCP.fig not found; skipping real-file benchmark");
    }

    // The budgets below are release-build frame budgets. An unoptimized build is ~10x slower
    // on 50k-node compilation, so its timings say nothing about the budget: the suite still
    // runs (and prints) in debug, but the assertions only gate release builds, which CI runs
    // in a dedicated `cargo test --release` step.
    if cfg!(debug_assertions) {
        eprintln!("NOTE: frame budgets are asserted only in release builds (cargo test --release -p designos-gpu --test large_file_perf)");
        return;
    }
    let frame_budget_ms = 16.0;
    let crud_p50_budget_ms = 2.0;
    let crud_p95_budget_ms = 5.0;
    let drag_p50_budget_ms = 5.0;

    for m in &res_50k {
        assert!(m.p50_ms < frame_budget_ms, "50k {} p50 {:.2} ms exceeded {:.1} ms budget", m.label, m.p50_ms, frame_budget_ms);
        if m.label == "Create+Resize+Delete (10f)" {
            assert!(m.p50_ms < crud_p50_budget_ms, "50k Create+Resize+Delete p50 {:.2} ms exceeded {:.1} ms budget", m.p50_ms, crud_p50_budget_ms);
            assert!(m.p95_ms < crud_p95_budget_ms, "50k Create+Resize+Delete p95 {:.2} ms exceeded {:.1} ms budget", m.p95_ms, crud_p95_budget_ms);
        }
        if m.label == "Top-Section Drag (rev+1)" {
            assert!(m.p50_ms < drag_p50_budget_ms, "50k Top-Section Drag p50 {:.2} ms exceeded {:.1} ms budget", m.p50_ms, drag_p50_budget_ms);
        }
    }
    if let Some(res_fig) = &res_fig_opt {
        for m in res_fig {
            assert!(m.p50_ms < frame_budget_ms, "VSF-PCP {} p50 {:.2} ms exceeded {:.1} ms budget", m.label, m.p50_ms, frame_budget_ms);
            if m.label == "Create+Resize+Delete (10f)" {
                assert!(m.p50_ms < crud_p50_budget_ms, "VSF-PCP Create+Resize+Delete p50 {:.2} ms exceeded {:.1} ms budget", m.p50_ms, crud_p50_budget_ms);
                assert!(m.p95_ms < crud_p95_budget_ms, "VSF-PCP Create+Resize+Delete p95 {:.2} ms exceeded {:.1} ms budget", m.p95_ms, crud_p95_budget_ms);
            }
            if m.label == "Top-Section Drag (rev+1)" {
                assert!(m.p50_ms < drag_p50_budget_ms, "VSF-PCP Top-Section Drag p50 {:.2} ms exceeded {:.1} ms budget", m.p50_ms, drag_p50_budget_ms);
            }
        }
    }
}
