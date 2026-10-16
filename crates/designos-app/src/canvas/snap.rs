//! Snapping of a moving rectangle (or of dragged edges) to other nodes'
//! edges/centers, plus the guide lines and distance measurements Figma shows.

use designos_doc::Rect;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum GuideAxis {
    /// Vertical line at x.
    X,
    /// Horizontal line at y.
    Y,
}

/// An alignment guide: a line at `pos` spanning `from..to` on the other axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Guide {
    pub axis: GuideAxis,
    pub pos: f32,
    pub from: f32,
    pub to: f32,
}

/// A distance measurement between two rects, drawn as a line with a label.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measure {
    pub a: (f32, f32),
    pub b: (f32, f32),
    pub value: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SnapResult {
    pub dx: f32,
    pub dy: f32,
    pub guides: Vec<Guide>,
    pub measures: Vec<Measure>,
}

/// A user guide as a snap target: a degenerate rect that is huge on one axis, so
/// only its position on the other axis can ever be within the snap threshold.
pub fn guide_rect(axis: GuideAxis, pos: f32) -> Rect {
    match axis {
        GuideAxis::X => Rect::new(pos, -GUIDE_EXTENT, 0.0, GUIDE_EXTENT * 3.0),
        GuideAxis::Y => Rect::new(-GUIDE_EXTENT, pos, GUIDE_EXTENT * 3.0, 0.0),
    }
}

const GUIDE_EXTENT: f32 = 1.0e7;

/// Whether `r` came from [`guide_rect`] (not a layer).
pub fn is_guide_rect(r: &Rect) -> bool {
    (r.w == 0.0 && r.h >= GUIDE_EXTENT) || (r.h == 0.0 && r.w >= GUIDE_EXTENT)
}

fn xs(r: &Rect) -> [f32; 3] {
    [r.x, r.x + r.w * 0.5, r.x + r.w]
}
fn ys(r: &Rect) -> [f32; 3] {
    [r.y, r.y + r.h * 0.5, r.y + r.h]
}

/// Best (smallest |delta|) shift taking any of `mine` onto any of `targets`
/// within `threshold`. Returns the shift and the target coordinate hit.
pub fn snap_values(mine: &[f32], targets: &[f32], threshold: f32) -> Option<(f32, f32)> {
    let mut best: Option<(f32, f32)> = None;
    for m in mine {
        for t in targets {
            let d = t - m;
            if d.abs() <= threshold && best.map(|(bd, _)| d.abs() < bd.abs()).unwrap_or(true) {
                best = Some((d, *t));
            }
        }
    }
    best
}

fn snap_axis_rects(mine: &[f32], others: &[Rect], coords: fn(&Rect) -> [f32; 3], threshold: f32) -> Option<(f32, f32)> {
    if mine.is_empty() {
        return None;
    }
    let mut best: Option<(f32, f32)> = None;
    for o in others {
        let ts = coords(o);
        for m in mine {
            for t in &ts {
                let d = *t - *m;
                if d.abs() <= threshold && best.map(|(bd, _)| d.abs() < bd.abs()).unwrap_or(true) {
                    best = Some((d, *t));
                }
            }
        }
    }
    best
}

/// Snaps `moving` (all of left/center/right and top/middle/bottom) to `others`.
/// `threshold` is in page units (callers divide the pixel threshold by zoom).
pub fn snap_rect(moving: &Rect, others: &[Rect], threshold: f32) -> SnapResult {
    let sx = snap_axis_rects(&xs(moving), others, xs, threshold);
    let sy = snap_axis_rects(&ys(moving), others, ys, threshold);
    finish(moving, others, sx, sy)
}

/// Snaps only specific edge coordinates (used while resizing: only the dragged
/// edges move). `x_edges`/`y_edges` are the coordinates currently dragged.
pub fn snap_edges(moving: &Rect, x_edges: &[f32], y_edges: &[f32], others: &[Rect], threshold: f32) -> SnapResult {
    let sx = snap_axis_rects(x_edges, others, xs, threshold);
    let sy = snap_axis_rects(y_edges, others, ys, threshold);
    finish(moving, others, sx, sy)
}

fn finish(moving: &Rect, others: &[Rect], sx: Option<(f32, f32)>, sy: Option<(f32, f32)>) -> SnapResult {
    let dx = sx.map(|s| s.0).unwrap_or(0.0);
    let dy = sy.map(|s| s.0).unwrap_or(0.0);
    let snapped = Rect::new(moving.x + dx, moving.y + dy, moving.w, moving.h);
    let mut guides = Vec::new();
    if let Some((_, line)) = sx {
        let (mut from, mut to) = (snapped.y, snapped.y + snapped.h);
        for o in others.iter().filter(|o| !is_guide_rect(o)) {
            if xs(o).iter().any(|v| (v - line).abs() < 1e-3) {
                from = from.min(o.y);
                to = to.max(o.y + o.h);
            }
        }
        guides.push(Guide { axis: GuideAxis::X, pos: line, from, to });
    }
    if let Some((_, line)) = sy {
        let (mut from, mut to) = (snapped.x, snapped.x + snapped.w);
        for o in others.iter().filter(|o| !is_guide_rect(o)) {
            if ys(o).iter().any(|v| (v - line).abs() < 1e-3) {
                from = from.min(o.x);
                to = to.max(o.x + o.w);
            }
        }
        guides.push(Guide { axis: GuideAxis::Y, pos: line, from, to });
    }
    let measures = if guides.is_empty() { vec![] } else { measurements(&snapped, others) };
    SnapResult { dx, dy, guides, measures }
}

/// Distances from `r` to the nearest neighbour on each side that overlaps it
/// on the perpendicular axis.
pub fn measurements(r: &Rect, others: &[Rect]) -> Vec<Measure> {
    let mut out = Vec::new();
    let cy = r.y + r.h * 0.5;
    let cx = r.x + r.w * 0.5;
    let mut left: Option<f32> = None; // right edge of the nearest rect on the left
    let mut right: Option<f32> = None;
    let mut up: Option<f32> = None;
    let mut down: Option<f32> = None;
    for o in others.iter().filter(|o| !is_guide_rect(o)) {
        let y_overlap = o.y < r.y + r.h && o.y + o.h > r.y;
        let x_overlap = o.x < r.x + r.w && o.x + o.w > r.x;
        if y_overlap {
            if o.x + o.w <= r.x {
                left = Some(left.map_or(o.x + o.w, |l: f32| l.max(o.x + o.w)));
            } else if o.x >= r.x + r.w {
                right = Some(right.map_or(o.x, |l: f32| l.min(o.x)));
            }
        }
        if x_overlap {
            if o.y + o.h <= r.y {
                up = Some(up.map_or(o.y + o.h, |l: f32| l.max(o.y + o.h)));
            } else if o.y >= r.y + r.h {
                down = Some(down.map_or(o.y, |l: f32| l.min(o.y)));
            }
        }
    }
    if let Some(l) = left {
        out.push(Measure { a: (l, cy), b: (r.x, cy), value: r.x - l });
    }
    if let Some(l) = right {
        out.push(Measure { a: (r.x + r.w, cy), b: (l, cy), value: l - (r.x + r.w) });
    }
    if let Some(l) = up {
        out.push(Measure { a: (cx, l), b: (cx, r.y), value: r.y - l });
    }
    if let Some(l) = down {
        out.push(Measure { a: (cx, r.y + r.h), b: (cx, l), value: l - (r.y + r.h) });
    }
    out
}

/// Computes `Alt`/`Option` distance measurements from selection bounds `a` to either
/// a hovered `target` rect or `a`'s parent `container` rect (as in Figma).
pub fn compute_alt_measures(a: Rect, target: Option<Rect>, container: Option<Rect>) -> Vec<Measure> {
    let mut out = Vec::new();
    let cx = a.x + a.w * 0.5;
    let cy = a.y + a.h * 0.5;

    if let Some(b) = target {
        let x_disjoint_right = a.x + a.w <= b.x;
        let x_disjoint_left = b.x + b.w <= a.x;
        let y_disjoint_down = a.y + a.h <= b.y;
        let y_disjoint_up = b.y + b.h <= a.y;

        if x_disjoint_right || x_disjoint_left || y_disjoint_down || y_disjoint_up {
            let oy_min = a.y.max(b.y);
            let oy_max = (a.y + a.h).min(b.y + b.h);
            let my = if oy_min <= oy_max { (oy_min + oy_max) * 0.5 } else { cy };
            if x_disjoint_right {
                let gap = b.x - (a.x + a.w);
                if gap > 0.0 {
                    out.push(Measure { a: (a.x + a.w, my), b: (b.x, my), value: gap });
                }
            } else if x_disjoint_left {
                let gap = a.x - (b.x + b.w);
                if gap > 0.0 {
                    out.push(Measure { a: (b.x + b.w, my), b: (a.x, my), value: gap });
                }
            }

            let ox_min = a.x.max(b.x);
            let ox_max = (a.x + a.w).min(b.x + b.w);
            let mx = if ox_min <= ox_max { (ox_min + ox_max) * 0.5 } else { cx };
            if y_disjoint_down {
                let gap = b.y - (a.y + a.h);
                if gap > 0.0 {
                    out.push(Measure { a: (mx, a.y + a.h), b: (mx, b.y), value: gap });
                }
            } else if y_disjoint_up {
                let gap = a.y - (b.y + b.h);
                if gap > 0.0 {
                    out.push(Measure { a: (mx, b.y + b.h), b: (mx, a.y), value: gap });
                }
            }
        } else {
            // Overlapping / containing rects: measure distances between corresponding edges.
            let dl = (a.x - b.x).abs();
            if dl > 0.01 {
                out.push(Measure { a: (b.x.min(a.x), cy), b: (b.x.max(a.x), cy), value: dl });
            }
            let dr = ((b.x + b.w) - (a.x + a.w)).abs();
            if dr > 0.01 {
                out.push(Measure { a: ((a.x + a.w).min(b.x + b.w), cy), b: ((a.x + a.w).max(b.x + b.w), cy), value: dr });
            }
            let dt = (a.y - b.y).abs();
            if dt > 0.01 {
                out.push(Measure { a: (cx, b.y.min(a.y)), b: (cx, b.y.max(a.y)), value: dt });
            }
            let db = ((b.y + b.h) - (a.y + a.h)).abs();
            if db > 0.01 {
                out.push(Measure { a: (cx, (a.y + a.h).min(b.y + b.h)), b: (cx, (a.y + a.h).max(b.y + b.h)), value: db });
            }
        }
        return out;
    }

    if let Some(c) = container {
        if a.x > c.x {
            out.push(Measure { a: (c.x, cy), b: (a.x, cy), value: a.x - c.x });
        }
        if c.x + c.w > a.x + a.w {
            out.push(Measure { a: (a.x + a.w, cy), b: (c.x + c.w, cy), value: (c.x + c.w) - (a.x + a.w) });
        }
        if a.y > c.y {
            out.push(Measure { a: (cx, c.y), b: (cx, a.y), value: a.y - c.y });
        }
        if c.y + c.h > a.y + a.h {
            out.push(Measure { a: (cx, a.y + a.h), b: (cx, c.y + c.h), value: (c.y + c.h) - (a.y + a.h) });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snaps_left_edge_to_other_right_edge() {
        let other = Rect::new(0.0, 0.0, 100.0, 100.0);
        let moving = Rect::new(102.0, 300.0, 50.0, 50.0);
        let r = snap_rect(&moving, &[other], 4.0);
        assert_eq!(r.dx, -2.0);
        assert_eq!(r.dy, 0.0);
        assert_eq!(r.guides.len(), 1);
        assert_eq!(r.guides[0].axis, GuideAxis::X);
        assert_eq!(r.guides[0].pos, 100.0);
        assert_eq!(r.guides[0].from, 0.0);
        assert_eq!(r.guides[0].to, 350.0);
    }

    #[test]
    fn snaps_centers() {
        let other = Rect::new(0.0, 0.0, 100.0, 100.0); // cx = 50
        let moving = Rect::new(23.0, 200.0, 50.0, 50.0); // cx = 48 -> snap +2
        let r = snap_rect(&moving, &[other], 4.0);
        assert_eq!(r.dx, 2.0);
    }

    #[test]
    fn no_snap_outside_threshold() {
        let other = Rect::new(0.0, 0.0, 100.0, 100.0);
        let moving = Rect::new(300.0, 300.0, 50.0, 50.0);
        let r = snap_rect(&moving, &[other], 4.0);
        assert_eq!((r.dx, r.dy), (0.0, 0.0));
        assert!(r.guides.is_empty() && r.measures.is_empty());
    }

    #[test]
    fn guides_snap_without_measures_or_huge_lines() {
        let g = guide_rect(GuideAxis::X, 100.0);
        assert!(is_guide_rect(&g) && !is_guide_rect(&Rect::new(0.0, 0.0, 10.0, 10.0)));
        let moving = Rect::new(103.0, 20.0, 50.0, 50.0);
        let r = snap_rect(&moving, &[g], 4.0);
        assert_eq!((r.dx, r.dy), (-3.0, 0.0));
        assert_eq!(r.guides.len(), 1);
        assert!(r.guides[0].to < 1000.0 && r.measures.is_empty());
        // a horizontal guide only snaps y
        let h = guide_rect(GuideAxis::Y, 200.0);
        let r = snap_rect(&Rect::new(5.0, 197.0, 10.0, 10.0), &[h], 4.0);
        assert_eq!((r.dx, r.dy), (0.0, -2.0), "the rect's centre is the closest edge");
    }

    #[test]
    fn picks_closest_candidate() {
        assert_eq!(snap_values(&[10.0], &[7.0, 11.0, 13.0], 5.0), Some((1.0, 11.0)));
        assert_eq!(snap_values(&[10.0], &[], 5.0), None);
    }

    #[test]
    fn measures_gap_to_neighbour() {
        let a = Rect::new(0.0, 0.0, 100.0, 100.0);
        let r = Rect::new(150.0, 0.0, 50.0, 100.0);
        let m = measurements(&r, &[a]);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, 50.0);
    }

    #[test]
    fn edge_snap_only_moves_dragged_edge() {
        let other = Rect::new(0.0, 0.0, 100.0, 100.0);
        let moving = Rect::new(200.0, 3.0, 50.0, 50.0);
        // dragging only the top edge (y=3) should snap to y=0 and ignore x
        let r = snap_edges(&moving, &[], &[moving.y], &[other], 4.0);
        assert_eq!((r.dx, r.dy), (0.0, -3.0));
    }

    #[test]
    fn alt_measures_disjoint_and_container() {
        let a = Rect::new(20.0, 30.0, 50.0, 40.0);
        let b = Rect::new(110.0, 30.0, 50.0, 40.0);
        let m = compute_alt_measures(a, Some(b), None);
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].value, 40.0);

        let container = Rect::new(0.0, 0.0, 200.0, 100.0);
        let mc = compute_alt_measures(a, None, Some(container));
        assert_eq!(mc.len(), 4);
        let vals: Vec<f32> = mc.iter().map(|x| x.value).collect();
        assert_eq!(vals, vec![20.0, 130.0, 30.0, 30.0]);
    }
}
