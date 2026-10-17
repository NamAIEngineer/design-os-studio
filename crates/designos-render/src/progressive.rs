//! Progressive blur: a Gaussian blur whose sigma ramps linearly between two
//! points of a layer.
//!
//! The ramp is shared with the GPU renderer, which draws the same thing with
//! the same levels and weights, so both stay within the blur tolerance of each
//! other. The image is blurred at a few sigma levels and each pixel then
//! blends the two levels around its own sigma:
//!
//! `acc = level[0]; for k in 1..: acc = lerp(acc, level[k], w_k(pixel))`
//! with `w_k = clamp((sigma(pixel) - level[k-1]) / (level[k] - level[k-1]), 0, 1)`.
//!
//! For a pixel whose sigma lies between `level[j-1]` and `level[j]` every earlier
//! step already replaced `acc` by its own level (weight 1), step `j` blends the
//! two neighbours and later steps leave it alone (weight 0).

use crate::blur::{gaussian_blur_at, Edges};
use crate::effects::grid_pos;
use designos_doc as d;
use tiny_skia as sk;

/// Most sigma levels one ramp is drawn with (so at most `MAX_LEVELS - 1` blurs
/// besides the unblurred level).
pub const MAX_LEVELS: usize = 8;
/// Target distance between two neighbouring levels, in sigma px.
const LEVEL_STEP_SIGMA: f32 = 1.0;
/// Below this sigma a level is the unblurred image (what `gaussian_blur_at` does).
pub const MIN_BLUR_SIGMA: f32 = 0.3;

/// A progressive blur placed on screen: where the ramp lies in device pixels
/// and which sigma it has at each end.
#[derive(Clone, Debug, PartialEq)]
pub struct Ramp {
    /// Position along the ramp at a pixel centre `(x, y)` in target pixels:
    /// `t = coef[0] * x + coef[1] * y + coef[2]`, 0 at the start point and 1 at
    /// the end point.
    coef: [f32; 3],
    sigma_start: f32,
    sigma_end: f32,
}

impl Ramp {
    /// The ramp of `e` on a node of `size` drawn through `ts` (node space to
    /// target pixels). `None` when the effect blurs nothing or `ts` cannot be
    /// inverted.
    ///
    /// The start and end points are `offset * size` in node space; `t` is the
    /// projection onto the line between them, measured in node space, so the
    /// ramp rotates and shears with the node. When both points coincide the
    /// ramp is a step: every pixel gets the end radius.
    pub fn new(e: &d::ProgressiveBlurEffect, size: d::Vec2, ts: &sk::Transform) -> Option<Ramp> {
        if !e.visible || e.max_radius() <= 0.0 {
            return None;
        }
        let inv = ts.invert()?;
        let scale = (ts.sx * ts.sy - ts.kx * ts.ky).abs().sqrt().max(1e-6);
        let sigma = |r: f32| r.max(0.0) * scale / 2.0;
        let p0 = (e.start_offset.x * size.x, e.start_offset.y * size.y);
        let dir = ((e.end_offset.x - e.start_offset.x) * size.x, (e.end_offset.y - e.start_offset.y) * size.y);
        let dd = dir.0 * dir.0 + dir.1 * dir.1;
        let coef = if dd > 1e-12 && dd.is_finite() {
            [
                (dir.0 * inv.sx + dir.1 * inv.ky) / dd,
                (dir.0 * inv.kx + dir.1 * inv.sy) / dd,
                (dir.0 * (inv.tx - p0.0) + dir.1 * (inv.ty - p0.1)) / dd,
            ]
        } else {
            [0.0, 0.0, 1.0]
        };
        Some(Ramp { coef, sigma_start: sigma(e.start_radius), sigma_end: sigma(e.end_radius) })
    }

    /// `[a, b, c]` of `t = a * x + b * y + c` (pixel centres, target pixels).
    pub fn coef(&self) -> [f32; 3] {
        self.coef
    }

    /// Sigma at the ends of the ramp: `(start, end)`.
    pub fn sigmas(&self) -> (f32, f32) {
        (self.sigma_start, self.sigma_end)
    }

    /// Sigma at the centre of target pixel `(px, py)`.
    pub fn sigma_at(&self, px: i32, py: i32) -> f32 {
        let t = (self.coef[0] * (px as f32 + 0.5) + self.coef[1] * (py as f32 + 0.5) + self.coef[2]).clamp(0.0, 1.0);
        self.sigma_start + (self.sigma_end - self.sigma_start) * t
    }

    /// The largest sigma on the ramp (what the blur needs room for).
    pub fn max_sigma(&self) -> f32 {
        self.sigma_start.max(self.sigma_end)
    }

    /// Ascending sigma levels the ramp is drawn with, at least one. The lowest
    /// level below [`MIN_BLUR_SIGMA`] is exactly 0 (the unblurred image).
    pub fn levels(&self) -> Vec<f32> {
        let (lo, hi) = (self.sigma_start.min(self.sigma_end), self.max_sigma());
        let lo = if lo < MIN_BLUR_SIGMA { 0.0 } else { lo };
        if hi - lo < 0.05 {
            return vec![hi];
        }
        let steps = (((hi - lo) / LEVEL_STEP_SIGMA).ceil() as usize).clamp(1, MAX_LEVELS - 1);
        (0..=steps).map(|i| lo + (hi - lo) * i as f32 / steps as f32).collect()
    }
}

/// Weight of the next level for a pixel of sigma `sigma`, given the levels
/// `prev < next` (see the module comment).
pub fn level_weight(prev: f32, next: f32, sigma: f32) -> f32 {
    ((sigma - prev) / (next - prev)).clamp(0.0, 1.0)
}

/// Blurs `pm` in place with `ramp`. `at` is the target pixel of `pm`'s
/// top-left corner; `edges` as for [`gaussian_blur_at`].
pub fn progressive_blur_at(pm: &mut sk::Pixmap, ramp: &Ramp, at: (i32, i32), edges: Edges) {
    let levels = ramp.levels();
    let grid = grid_pos(at.0, at.1);
    let blurred = |sigma: f32| {
        let mut b = pm.clone();
        gaussian_blur_at(&mut b, sigma, grid, edges);
        b
    };
    let mut acc = blurred(levels[0]);
    let w = pm.width() as usize;
    for pair in levels.windows(2) {
        let next = blurred(pair[1]);
        let (a, b) = (acc.data_mut(), next.data());
        for (i, (pa, pb)) in a.chunks_exact_mut(4).zip(b.chunks_exact(4)).enumerate() {
            let (x, y) = ((i % w) as i32, (i / w) as i32);
            let k = level_weight(pair[0], pair[1], ramp.sigma_at(at.0 + x, at.1 + y));
            if k > 0.0 {
                for (ca, cb) in pa.iter_mut().zip(pb) {
                    *ca = (*ca as f32 + (*cb as f32 - *ca as f32) * k).round().clamp(0.0, 255.0) as u8;
                }
            }
        }
    }
    *pm = acc;
}
