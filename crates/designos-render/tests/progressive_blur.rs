//! Progressive blur on the CPU renderer: the blur radius ramps linearly between
//! two points of the layer, for the layer blur and the background blur.
mod common;
use common::*;
use designos_doc::*;
use designos_render::tiny_skia::Pixmap;

const STRIPE_PERIOD: f32 = 16.0;
const STRIPE_WIDTH: f32 = 8.0;
const SIDE: f32 = 80.0;

fn ramp(background: bool, start_radius: f32, end_radius: f32, start: (f32, f32), end: (f32, f32)) -> Effect {
    Effect::ProgressiveBlur(ProgressiveBlurEffect {
        background,
        start_radius,
        end_radius,
        start_offset: Vec2::new(start.0, start.1),
        end_offset: Vec2::new(end.0, end.1),
        visible: true,
    })
}

/// Top-to-bottom ramp over the whole layer.
fn top_to_bottom(background: bool, start_radius: f32, end_radius: f32) -> Effect {
    ramp(background, start_radius, end_radius, (0.5, 0.0), (0.5, 1.0))
}

fn stripes(t: &mut T, parent: Option<NodeId>) {
    let mut x = 1.0;
    while x < SIDE + 16.0 {
        t.rect(parent, Rect::new(x, 0.0, STRIPE_WIDTH, SIDE), Color::BLACK);
        x += STRIPE_PERIOD;
    }
}

/// Black vertical stripes inside a transparent frame that carries `effect`
/// (layer blur: the frame's own layer is blurred).
fn layer_scene(effect: Effect) -> (T, NodeId) {
    let mut t = T::new();
    let f = t.add(None, |id| Node::frame(id, Rect::new(0.0, 0.0, SIDE, SIDE)));
    t.node(f).fills.clear();
    t.node(f).effects = vec![effect];
    stripes(&mut t, Some(f));
    (t, f)
}

/// Stripes on the page with a transparent pane over them that carries `effect`.
fn background_scene(effect: Effect) -> T {
    let mut t = T::new();
    stripes(&mut t, None);
    let f = t.add(None, |id| Node::frame(id, Rect::new(0.0, 0.0, SIDE, SIDE)));
    t.node(f).fills.clear();
    t.node(f).effects = vec![effect];
    t
}

/// Spread of the red channel over `x0..x1` of row `y`: 255 for sharp black and
/// white stripes, near 0 once they are blurred away.
fn row_contrast(pm: &Pixmap, y: u32, x0: u32, x1: u32) -> i32 {
    let vals: Vec<i32> = (x0..x1).map(|x| px(pm, x, y)[0] as i32).collect();
    vals.iter().max().unwrap() - vals.iter().min().unwrap()
}

fn col_contrast(pm: &Pixmap, x: u32, y0: u32, y1: u32) -> i32 {
    let vals: Vec<i32> = (y0..y1).map(|y| px(pm, x, y)[0] as i32).collect();
    vals.iter().max().unwrap() - vals.iter().min().unwrap()
}

fn contrast_down(pm: &Pixmap) -> Vec<i32> {
    (4..SIDE as u32 - 2).step_by(8).map(|y| row_contrast(pm, y, 20, 60)).collect()
}

fn assert_non_increasing(v: &[i32], what: &str) {
    for w in v.windows(2) {
        assert!(w[1] <= w[0] + 3, "{what}: contrast must not recover along the ramp: {v:?}");
    }
}

#[test]
fn layer_ramp_sharp_at_start_and_blurred_at_end_and_monotonic_between() {
    let (t, _) = layer_scene(top_to_bottom(false, 0.0, 16.0));
    let c = contrast_down(&t.render(100, 80));
    assert!(c[0] >= 240, "start of the ramp stays sharp: {c:?}");
    assert!(*c.last().unwrap() <= 40, "end of the ramp is blurred away: {c:?}");
    assert_non_increasing(&c, "layer ramp");
    assert!(c[0] - c[c.len() - 1] >= 200, "ramp must actually change the blur: {c:?}");
}

#[test]
fn layer_ramp_runs_backwards_when_the_start_radius_is_the_larger() {
    let (t, _) = layer_scene(top_to_bottom(false, 16.0, 0.0));
    let c = contrast_down(&t.render(100, 80));
    assert!(c[0] <= 40, "start radius is the big one: {c:?}");
    assert!(*c.last().unwrap() >= 240, "end radius 0 is sharp: {c:?}");
    let rev: Vec<i32> = c.iter().rev().copied().collect();
    assert_non_increasing(&rev, "reversed ramp");
}

#[test]
fn offsets_are_normalized_to_the_layer_size() {
    // ramp only between 25% and 75% of the height
    let (t, _) = layer_scene(ramp(false, 0.0, 16.0, (0.5, 0.25), (0.5, 0.75)));
    let pm = t.render(100, 80);
    assert!(row_contrast(&pm, 10, 20, 60) >= 240, "before the start offset: sharp");
    assert!(row_contrast(&pm, 70, 20, 60) <= 40, "past the end offset: fully blurred");
    let mid = row_contrast(&pm, 40, 20, 60);
    assert!(mid > 40 && mid < 240, "inside the ramp: partly blurred, got {mid}");
}

#[test]
fn zero_radius_ramp_changes_nothing() {
    let (blurless, _) = layer_scene(top_to_bottom(false, 0.0, 0.0));
    let (plain, f) = {
        let (mut t, f) = layer_scene(top_to_bottom(false, 0.0, 0.0));
        t.node(f).effects.clear();
        (t, f)
    };
    let _ = f;
    assert_eq!(blurless.render(100, 80).data(), plain.render(100, 80).data());
}

#[test]
fn hidden_ramp_changes_nothing() {
    let mut e = top_to_bottom(false, 0.0, 16.0);
    if let Effect::ProgressiveBlur(p) = &mut e {
        p.visible = false;
    }
    let (hidden, _) = layer_scene(e);
    let (mut plain, f) = layer_scene(top_to_bottom(false, 0.0, 16.0));
    plain.node(f).effects.clear();
    assert_eq!(hidden.render(100, 80).data(), plain.render(100, 80).data());
}

#[test]
fn ramp_follows_the_layer_transform() {
    // rotate the layer a quarter turn: its top (sharp end) lands on the right
    let (mut t, f) = layer_scene(top_to_bottom(false, 0.0, 16.0));
    t.node(f).transform = Affine { a: 0.0, b: 1.0, c: -1.0, d: 0.0, tx: SIDE, ty: 0.0 };
    let pm = t.render(100, 80);
    // device x runs from 80 (local top) to 0 (local bottom); the stripes turn horizontal
    let right = col_contrast(&pm, 76, 20, 60);
    let left = col_contrast(&pm, 3, 20, 60);
    assert!(right >= 240, "local start is on the right and sharp: {right}");
    assert!(left <= 40, "local end is on the left and blurred: {left}");
}

#[test]
fn ramp_radius_scales_with_the_viewport_zoom() {
    let (t, _) = layer_scene(top_to_bottom(false, 0.0, 16.0));
    let mut pm = Pixmap::new(200, 160).unwrap();
    let vp = designos_render::Viewport { zoom: 2.0, ..Default::default() };
    designos_render::render_page(&t.doc, t.page, &vp, &mut pm);
    let sharp = row_contrast(&pm, 8, 40, 120);
    let blurred = row_contrast(&pm, 150, 40, 120);
    assert!(sharp >= 200, "top stays sharp at zoom 2: {sharp}");
    assert!(blurred <= 60, "bottom is blurred at zoom 2: {blurred}");
}

#[test]
fn background_ramp_blurs_what_is_behind_progressively_inside_the_shape_only() {
    let t = background_scene(top_to_bottom(true, 0.0, 16.0));
    let pm = t.render(100, 80);
    let c = contrast_down(&pm);
    assert!(c[0] >= 240, "start of the ramp keeps the backdrop sharp: {c:?}");
    assert!(*c.last().unwrap() <= 40, "end of the ramp blurs the backdrop away: {c:?}");
    assert_non_increasing(&c, "background ramp");
    // the pane is 80 wide: stripes past it are untouched at every height
    assert!(row_contrast(&pm, 70, 84, 96) >= 240, "outside the shape the backdrop stays sharp");
}

#[test]
fn background_ramp_is_independent_of_the_layer_variant() {
    // a layer ramp must not blur the backdrop and a background ramp must not blur the layer
    let on_page = background_scene(top_to_bottom(false, 0.0, 16.0));
    // `on_page`: the pane itself is empty, so a layer blur of it changes nothing visible
    let pm = on_page.render(100, 80);
    assert!(row_contrast(&pm, 70, 20, 60) >= 240, "layer ramp on an empty pane leaves the backdrop alone");
    let (own, _) = layer_scene(top_to_bottom(true, 0.0, 16.0));
    // taller than the pane so the page edge does not fade the blurred backdrop
    let pm = own.render(100, 120);
    assert!(row_contrast(&pm, 70, 20, 60) >= 240, "background ramp does not blur the pane's own children");
}
