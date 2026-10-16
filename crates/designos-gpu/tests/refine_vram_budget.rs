//! VRAM ceiling of the idle refine compile: with `uncap_programs` set, offscreen
//! programs are created only while their accounted GPU bytes stay within
//! `refine_vram_budget_bytes`. Compiling is CPU-only, no adapter is needed.
//!
//! The expected byte counts are recomputed here from the compiled frame with the
//! formula of `resources.rs` / `exec.rs`, independently of the compiler's own
//! running total.

use designos_doc::*;
use designos_gpu::{CompileOptions, Frame, SceneCompiler};
use designos_render::Viewport;
use std::time::Duration;

const BLURRED: usize = 100;
const SIZE: (u32, u32) = (1000, 1000);
const MIB: u64 = 1024 * 1024;

/// `count` layer-blurred rectangles inside the view.
fn blurred_page(count: usize) -> (Document, NodeId) {
    effect_page(count, Effect::LayerBlur { radius: 4.0, visible: true })
}

/// `count` rectangles inside the view, each carrying `effect`.
fn effect_page(count: usize, effect: Effect) -> (Document, NodeId) {
    let mut doc = Document::new("vram budget");
    let page = doc.add_page("Page");
    for i in 0..count {
        let id = doc.alloc_id();
        let (col, row) = ((i % 10) as f32, (i / 10) as f32);
        let mut n = Node::rectangle(id, Rect::new(col * 90.0 + 10.0, row * 90.0 + 10.0, 60.0, 60.0));
        n.fills = vec![Paint::solid(Color::rgb(0x33, 0x66, 0xCC))];
        n.effects = vec![effect.clone()];
        doc.insert(page, n).unwrap();
    }
    (doc, page)
}

fn interactive_compiler() -> SceneCompiler {
    let mut c = SceneCompiler::new();
    c.geom.budget = Some(Duration::from_millis(8));
    c
}

fn compile(c: &mut SceneCompiler, doc: &Document, page: NodeId) -> Frame {
    let vp = Viewport { offset: Vec2::new(0.0, 0.0), zoom: 1.0, dpr: 1.0 };
    c.compile(doc, page, 1, &vp, SIZE)
}

/// Pool rounding of `resources.rs`: 8..64 powers of two, then multiples of 64.
fn pool_extent(v: i32) -> u64 {
    let v = v.max(1) as u64;
    if v <= 64 {
        v.next_power_of_two().max(8)
    } else {
        v.div_ceil(64) * 64
    }
}

/// Bytes of every offscreen program of `frame` (the root renders into the
/// output texture): MSAA colour (4 samples) + resolve target, Rgba8Unorm, plus
/// two plain targets per blur. Only layer blurs occur in these documents.
fn recount(frame: &Frame) -> u64 {
    frame
        .programs
        .iter()
        .skip(1)
        .map(|p| {
            let px = pool_extent(p.rect.w()) * pool_extent(p.rect.h());
            px * 4 * (4 + 1) + px * 4 * 2 * p.post.len() as u64
        })
        .sum()
}

#[test]
fn default_budget_is_512_mib() {
    assert_eq!(CompileOptions::default().refine_vram_budget_bytes, 512 * MIB);
}

#[test]
fn one_blurred_node_costs_msaa_resolve_and_two_blur_targets() {
    // layer 76x76 (60 + 2 * (1 + 7) px blur reach) -> pooled 128x128, 28 B/px
    let (doc, page) = blurred_page(1);
    let mut c = interactive_compiler();
    c.opts.uncap_programs = true;
    let frame = compile(&mut c, &doc, page);
    assert_eq!(frame.programs.len(), 2, "root + one layer");
    assert_eq!(c.last_vram_bytes, 128 * 128 * 28);
    assert_eq!(recount(&frame), c.last_vram_bytes);
}

#[test]
fn refine_stops_creating_programs_once_the_budget_is_reached() {
    let (doc, page) = blurred_page(BLURRED);
    let mut unbounded = interactive_compiler();
    unbounded.opts.uncap_programs = true;
    let full = compile(&mut unbounded, &doc, page);
    assert!(full.programs.len() > BLURRED, "default budget must admit all {BLURRED} layers");

    for budget in [MIB, 4 * MIB, 16 * MIB] {
        let mut c = interactive_compiler();
        c.opts.uncap_programs = true;
        c.opts.refine_vram_budget_bytes = budget;
        let frame = compile(&mut c, &doc, page);
        let bytes = recount(&frame);
        assert!(frame.programs.len() > 1, "budget {budget}: at least one layer fits");
        assert!(frame.programs.len() < full.programs.len(), "budget {budget}: programs {} not below {}", frame.programs.len(), full.programs.len());
        assert!(bytes <= budget, "budget {budget}: frame holds {bytes} bytes");
        assert_eq!(c.last_vram_bytes, bytes, "running total must equal the recount");
        // a further layer (458_752 B) no longer fits: the budget is used up, not wasted
        assert!(bytes + 128 * 128 * 28 > budget, "budget {budget}: stopped early at {bytes} bytes");
    }
}

#[test]
fn bigger_budget_admits_more_programs() {
    let (doc, page) = blurred_page(BLURRED);
    let count = |budget: u64| {
        let mut c = interactive_compiler();
        c.opts.uncap_programs = true;
        c.opts.refine_vram_budget_bytes = budget;
        compile(&mut c, &doc, page).programs.len()
    };
    assert!(count(2 * MIB) < count(8 * MIB));
}

#[test]
fn capped_compile_ignores_the_vram_budget() {
    let (doc, page) = blurred_page(BLURRED);
    let capped = |budget: u64| {
        let mut c = interactive_compiler();
        c.opts.refine_vram_budget_bytes = budget;
        let f = compile(&mut c, &doc, page);
        let s = f.stats();
        (f.programs.len(), s.draws, s.composites, s.backdrops, s.triangles)
    };
    let (today, tiny, huge) = (capped(512 * MIB), capped(1), capped(u64::MAX));
    assert!(today.0 <= 64 && today.0 >= 60, "capped compile keeps the 64-program cap, got {}", today.0);
    assert_eq!(today, tiny, "budget must not change an interactive (capped) frame");
    assert_eq!(today, huge);
}

#[test]
fn headless_compile_stays_uncapped_whatever_the_budget() {
    let (doc, page) = blurred_page(BLURRED);
    let mut c = SceneCompiler::new();
    assert!(c.geom.budget.is_none());
    c.opts.uncap_programs = true;
    c.opts.refine_vram_budget_bytes = 1;
    let frame = compile(&mut c, &doc, page);
    assert!(frame.programs.len() > BLURRED, "headless frame must open a program per blurred node, got {}", frame.programs.len());
}

#[test]
fn effect_programs_of_a_node_are_budgeted_together() {
    // an inner shadow opens three programs (hole, grain layer, shape) and blurs one
    let shadow = Effect::InnerShadow { color: Color::rgba(0, 0, 0, 128), offset: Vec2::new(0.0, 2.0), radius: 6.0, spread: 0.0, visible: true };
    let (doc, page) = effect_page(BLURRED, shadow);
    let mut unbounded = interactive_compiler();
    unbounded.opts.uncap_programs = true;
    let full = compile(&mut unbounded, &doc, page).programs.len();
    assert!(full > 3 * BLURRED, "default budget must admit every inner shadow, got {full} programs");
    for budget in [4 * MIB, 16 * MIB, 32 * MIB] {
        let mut c = interactive_compiler();
        c.opts.uncap_programs = true;
        c.opts.refine_vram_budget_bytes = budget;
        let frame = compile(&mut c, &doc, page);
        let bytes = recount(&frame);
        assert!(frame.programs.len() > 1 && frame.programs.len() < full, "budget {budget}: {} of {full} programs", frame.programs.len());
        assert!(bytes <= budget, "budget {budget}: frame holds {bytes} bytes");
        assert_eq!(c.last_vram_bytes, bytes);
    }
}
