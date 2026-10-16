//! Canvas rendering on the GPU (`designos-gpu`): the visible page is compiled
//! to a frame while the UI is painted and drawn by an egui paint callback.
//! Pan and zoom only change the view transform: geometry is cached per node,
//! nothing is re-rasterised on the CPU.
//!
//! The CPU tile path (`tiles.rs`) stays the fallback: it is used when the GPU
//! cannot be initialised, when "Use GPU rendering" is off, and for
//! `--screenshot` unless `--gpu` is given (pixel diffs must be deterministic).

use crate::app::App;
use designos_gpu::{Canvas, Frame, SceneCompiler};
use designos_render::Viewport;
use eframe::egui_wgpu;
use egui::{Rect as ERect, Ui};
use std::sync::Arc;

type FrameKey = (designos_doc::NodeId, u64, [u32; 2], u32, u32, (u32, u32), usize);

/// App-side GPU state.
pub struct GpuView {
    /// Preferences -> "Use GPU rendering".
    pub pref: bool,
    /// The device and renderer initialised.
    pub available: bool,
    max_texture: u32,
    compiler: SceneCompiler,
    last_key: Option<FrameKey>,
    /// The frame for `last_key` was compiled with the offscreen-program cap
    /// lifted (the idle full-quality pass), so there is nothing left to refine.
    refined: bool,
    /// Milliseconds the last frame took to compile (CPU side).
    pub last_compile_ms: f32,
    pub last_stats: Option<designos_gpu::FrameStats>,
}

impl Default for GpuView {
    fn default() -> Self {
        let mut compiler = SceneCompiler::new();
        // re-flattening curves for a new zoom level may take at most this long per frame
        compiler.geom.budget = Some(std::time::Duration::from_millis(8));
        Self { pref: true, available: false, max_texture: 0, compiler, last_key: None, refined: false, last_compile_ms: 0.0, last_stats: None }
    }
}

impl GpuView {
    /// Whether the canvas is drawn by the GPU right now.
    pub fn active(&self) -> bool {
        self.pref && self.available
    }

    /// Cached spatial index of the current page's world bounds.
    pub fn scene_index(&self) -> &designos_render::SceneIndex {
        &self.compiler.index
    }

    /// Forgets cached geometry (the document was replaced).
    pub fn reset(&mut self) {
        self.compiler.geom.clear();
        self.compiler.index.clear();
        self.last_key = None;
        self.refined = false;
    }

    /// What this frame needs: the cached output, the capped interactive
    /// compile, or the one uncapped compile of a view that has gone idle.
    /// Idle = same frame key as the last compile, all geometry ready, and no
    /// pointer gesture in progress.
    fn plan(&self, key: FrameKey, pointer_down: bool) -> FramePlan {
        let still = self.last_key == Some(key) && self.compiler.geom.stale_hits == 0;
        match (still, self.refined, pointer_down) {
            (false, _, _) => FramePlan::Interactive,
            (true, false, false) => FramePlan::Refine,
            (true, _, _) => FramePlan::Reuse,
        }
    }

    /// Largest texture edge the device supports.
    pub fn set_max_texture(&mut self, max: u32) {
        self.compiler.opts.max_texture = max;
        self.max_texture = max;
    }
}

#[derive(Debug, PartialEq, Eq)]
enum FramePlan {
    /// Re-blit the cached output (`Cb { frame: None }`).
    Reuse,
    /// Compile with the interactive fast paths and the 64-program cap.
    Interactive,
    /// Compile once more with the program cap lifted and cache that frame.
    Refine,
}

struct Cb {
    frame: Option<Arc<Frame>>,
}

impl egui_wgpu::CallbackTrait for Cb {
    fn prepare(
        &self,
        device: &egui_wgpu::wgpu::Device,
        _queue: &egui_wgpu::wgpu::Queue,
        _screen: &egui_wgpu::ScreenDescriptor,
        encoder: &mut egui_wgpu::wgpu::CommandEncoder,
        resources: &mut egui_wgpu::CallbackResources,
    ) -> Vec<egui_wgpu::wgpu::CommandBuffer> {
        if let (Some(c), Some(frame)) = (resources.get_mut::<Canvas>(), &self.frame) {
            c.prepare(device, encoder, frame);
        }
        Vec::new()
    }

    fn paint(&self, _info: egui::PaintCallbackInfo, pass: &mut egui_wgpu::wgpu::RenderPass<'static>, resources: &egui_wgpu::CallbackResources) {
        if let Some(c) = resources.get::<Canvas>() {
            c.paint(pass);
        }
    }
}

impl App {
    /// Preferences -> "Use GPU rendering": switches between the GPU canvas and the CPU tiles.
    pub fn toggle_gpu(&mut self) {
        if !self.canvas.gpu.available {
            self.show_toast("GPU rendering is not available on this system", true);
            return;
        }
        self.canvas.gpu.pref = !self.canvas.gpu.pref;
        self.canvas.gpu.last_key = None;
        let on = self.canvas.gpu.pref;
        if let Some(lib) = &self.lib {
            if let Err(e) = crate::comments::settings::save_flag(lib.root(), "use_gpu", on) {
                self.show_toast(e, true);
            }
        }
        // the tile cache may be stale after a long GPU session
        self.canvas.tiles.invalidate_all();
    }
}

/// Creates the GPU renderer on eframe's device. Returns false (and the app keeps
/// using the CPU tiles) when that is not possible.
pub fn init(rs: &egui_wgpu::RenderState) -> Option<u32> {
    match Canvas::new(&rs.device, &rs.queue, rs.target_format, 1) {
        Ok(c) => {
            let max = c.max_texture_dimension();
            rs.renderer.write().callback_resources.insert(c);
            Some(max)
        }
        Err(e) => {
            eprintln!("designos: GPU rendering unavailable, using the CPU renderer ({e})");
            None
        }
    }
}

/// Paints the page with the GPU into `rect`; false when the CPU path must do it.
pub fn paint(app: &mut App, ui: &Ui, rect: ERect) -> bool {
    if !app.canvas.gpu.active() {
        return false;
    }
    let ppp = ui.ctx().pixels_per_point();
    let size = ((rect.width() * ppp).round().max(1.0) as u32, (rect.height() * ppp).round().max(1.0) as u32);
    if size.0 > app.canvas.gpu.max_texture.max(1) || size.1 > app.canvas.gpu.max_texture.max(1) {
        return false;
    }
    let rev = super::tiles::take_rev(app);
    let vp = Viewport { offset: app.view.offset, zoom: app.view.zoom, dpr: ppp };
    let key: FrameKey =
        (app.page, rev, [vp.offset.x.to_bits(), vp.offset.y.to_bits()], vp.zoom.to_bits(), vp.dpr.to_bits(), size, designos_render::font_generation());
    let (doc, page) = (&app.doc, app.page);
    let g = &mut app.canvas.gpu;
    let plan = g.plan(key, ui.input(|i| i.pointer.any_down()));
    if plan == FramePlan::Reuse {
        ui.painter().with_clip_rect(rect).add(egui_wgpu::Callback::new_paint_callback(rect, Cb { frame: None }));
        return true;
    }
    let start = std::time::Instant::now();
    let refine = plan == FramePlan::Refine;
    g.compiler.opts.uncap_programs = refine;
    let frame = g.compiler.compile(doc, page, rev, &vp, size);
    g.compiler.opts.uncap_programs = false;
    g.last_key = Some(key);
    g.refined = refine && g.compiler.geom.stale_hits == 0;
    g.last_stats = Some(frame.stats());
    g.last_compile_ms = start.elapsed().as_secs_f32() * 1000.0;
    if g.compiler.geom.stale_hits > 0 || !refine {
        // some meshes still have the previous zoom's flattening, or the view
        // may now be idle: come back for another frame (no timer needed)
        ui.ctx().request_repaint();
    }
    ui.painter().with_clip_rect(rect).add(egui_wgpu::Callback::new_paint_callback(rect, Cb { frame: Some(Arc::new(frame)) }));
    app.canvas.last_render_ms = start.elapsed().as_secs_f32() * 1000.0;
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(rev: u64) -> FrameKey {
        (designos_doc::NodeId(0), rev, [0, 0], 1.0f32.to_bits(), 1.0f32.to_bits(), (100, 100), 0)
    }

    /// A view whose last compile was for `key(1)`.
    fn view(refined: bool) -> GpuView {
        GpuView { last_key: Some(key(1)), refined, ..GpuView::default() }
    }

    #[test]
    fn changed_view_compiles_interactively() {
        assert_eq!(GpuView::default().plan(key(1), false), FramePlan::Interactive);
        assert_eq!(view(false).plan(key(2), false), FramePlan::Interactive, "a document change returns to the capped compile");
    }

    #[test]
    fn idle_view_refines_exactly_once_then_reuses() {
        assert_eq!(view(false).plan(key(1), false), FramePlan::Refine);
        assert_eq!(view(true).plan(key(1), false), FramePlan::Reuse);
        assert_eq!(view(true).plan(key(1), true), FramePlan::Reuse);
    }

    #[test]
    fn refine_waits_for_gesture_and_ready_geometry() {
        assert_eq!(view(false).plan(key(1), true), FramePlan::Reuse, "no refine while the pointer is down");
        let mut g = view(false);
        g.compiler.geom.stale_hits = 3;
        assert_eq!(g.plan(key(1), false), FramePlan::Interactive, "deferred geometry keeps the interactive compile");
    }

    #[test]
    fn viewport_change_after_refine_returns_to_interactive() {
        let mut moved = key(1);
        moved.2 = [1.0f32.to_bits(), 0];
        assert_eq!(view(true).plan(moved, false), FramePlan::Interactive);
    }

    #[test]
    fn reset_forgets_the_refined_frame() {
        let mut g = view(true);
        g.reset();
        assert_eq!(g.plan(key(1), false), FramePlan::Interactive);
    }
}
