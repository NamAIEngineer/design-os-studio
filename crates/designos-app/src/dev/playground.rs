//! Read-only Component Playground in Dev Mode (#159 / E3-14).
//!
//! Lets developers toggle Boolean, Text, Variant, and InstanceSwap component
//! properties on an in-memory scratch instance in the Dev Mode inspector without
//! ever mutating `app.doc` (`doc_hash` and `mutation_epoch` remain unchanged),
//! while live preview and code generation update immediately.

use crate::app::App;
use crate::panels::inspect::{section_title, small_button, Cur, PAD};
use crate::theme;
use crate::widgets::field;
use crate::widgets::menu::dropdown;
use crate::widgets::{self, elide};
use designos_doc::*;
use egui::{Align2, Color32, Context, Pos2, Rect, Sense, Stroke, TextureHandle, Ui, Vec2};
use std::collections::BTreeMap;

type PreviewCacheKey = (u64, u64, NodeId, u64);

#[derive(Default)]
pub struct PlaygroundState {
    /// Inspected node (`id`) this scratch session belongs to.
    pub node: Option<NodeId>,
    /// Scratch property overrides keyed by property name (`ComponentPropDef::name`).
    pub values: BTreeMap<String, PropValue>,
    /// Monotonic revision bumped on every scratch override edit or reset.
    pub rev: u64,
    /// Cached raster preview of the scratch instance for the inspector stage.
    preview_cache: Option<(PreviewCacheKey, Option<TextureHandle>)>,
}

impl PlaygroundState {
    pub fn ensure_node(&mut self, id: NodeId) {
        if self.node != Some(id) {
            self.node = Some(id);
            self.values.clear();
            self.rev = self.rev.wrapping_add(1);
            self.preview_cache = None;
        }
    }

    pub fn set_prop(&mut self, id: NodeId, name: String, value: PropValue) {
        self.ensure_node(id);
        self.values.insert(name, value);
        self.rev = self.rev.wrapping_add(1);
        self.preview_cache = None;
    }

    pub fn reset(&mut self, id: NodeId) {
        self.ensure_node(id);
        if !self.values.is_empty() {
            self.values.clear();
            self.rev = self.rev.wrapping_add(1);
            self.preview_cache = None;
        }
    }

    pub fn has_overrides_for(&self, id: NodeId) -> bool {
        self.node == Some(id) && !self.values.is_empty()
    }
}

/// Resolves `(target_node, main_component)` for `id` if it is (or lies inside) an
/// `Instance`, `Component`, or `ComponentSet`.
pub fn playground_target(doc: &Document, id: NodeId) -> Option<(NodeId, NodeId)> {
    let n = doc.get(id)?;
    if let Some(inst) = instance_root_of(doc, id) {
        if let Some(comp) = doc.get(inst).and_then(instance_component) {
            return Some((inst, comp));
        }
    }
    if let Some(comp) = instance_component(n) {
        return Some((id, comp));
    }
    match &n.kind {
        NodeKind::Component(_) => Some((id, id)),
        NodeKind::ComponentSet(_) => components_of(doc, id).into_iter().next().map(|c| (id, c)),
        _ => enclosing_component(doc, id).map(|c| (c, c)),
    }
}

fn comp_label(doc: &Document, comp: NodeId) -> String {
    let Some(c) = doc.get(comp) else { return "Missing component".into() };
    match c.parent.and_then(|p| doc.get(p)).filter(|p| matches!(p.kind, NodeKind::ComponentSet(_))) {
        Some(set) => format!("{} / {}", set.name, c.name),
        None => c.name.clone(),
    }
}

fn current_prop_value(doc: &Document, target_id: NodeId, comp_id: NodeId, def: &ComponentPropDef, pg: &PlaygroundState, id: NodeId) -> PropValue {
    if pg.node == Some(id) {
        if let Some(v) = pg.values.get(&def.name) {
            return v.clone();
        }
    }
    if doc.get(target_id).is_some_and(|n| matches!(n.kind, NodeKind::Instance { .. })) {
        return prop_value(doc, target_id, def);
    }
    if matches!(def.kind, PropKind::Variant { .. }) {
        if let Some(c) = doc.get(comp_id) {
            if let Some((_, v)) = parse_variant_name(&c.name).into_iter().find(|(k, _)| *k == def.name) {
                return PropValue::Variant(v);
            }
        }
    }
    def.default.clone()
}

fn refit_text_in_subtree(doc: &mut Document, root: NodeId) {
    let mut stack = vec![root];
    while let Some(cur) = stack.pop() {
        let kids = doc.get(cur).map(|n| n.children.clone()).unwrap_or_default();
        if let Some(n) = doc.get_mut(cur) {
            if let NodeKind::Text(t) = &n.kind {
                if matches!(t.auto_resize, TextAutoResize::WidthAndHeight | TextAutoResize::Height) {
                    n.size = crate::layout::estimate_text_size(t);
                }
            }
        }
        stack.extend(kids);
    }
}

fn apply_bindings_to_component_subtree(doc: &mut Document, root: NodeId, values: &BTreeMap<String, PropValue>) {
    let mut stack = vec![root];
    while let Some(cur) = stack.pop() {
        let kids = doc.get(cur).map(|n| n.children.clone()).unwrap_or_default();
        if let Some(n) = doc.get_mut(cur) {
            for (field, prop) in n.component.bindings.clone() {
                let Some(v) = values.get(&prop) else { continue };
                match (field, v) {
                    (OverrideField::Visible, PropValue::Bool(b)) => n.visible = *b,
                    (OverrideField::Characters, PropValue::Text(s)) => {
                        if let NodeKind::Text(t) = &mut n.kind {
                            t.content = s.clone();
                        }
                    }
                    _ => {}
                }
            }
        }
        stack.extend(kids);
    }
}

/// Builds an in-memory scratch `(Document, codegen_node_id, preview_node_id)` with
/// `pg.values` applied, without mutating `base_doc`.
pub fn build_scratch_doc(base_doc: &Document, id: NodeId, pg: &PlaygroundState) -> Option<(Document, NodeId, NodeId)> {
    let (target_id, comp_id) = playground_target(base_doc, id)?;
    let mut scratch = base_doc.clone();
    let empty = BTreeMap::new();
    let values = if pg.node == Some(id) { &pg.values } else { &empty };

    if scratch.get(target_id).is_some_and(|n| matches!(n.kind, NodeKind::Instance { .. })) {
        for (k, v) in values {
            let cmd = set_instance_prop(&mut scratch, target_id, k, Some(v.clone()));
            let _ = cmd.apply(&mut scratch);
        }
        refit_text_in_subtree(&mut scratch, target_id);
        scratch.relayout_subtree(target_id);
        return Some((scratch, id, target_id));
    }

    // `target_id` is a Component or ComponentSet: instantiate a scratch instance on its page.
    let page = scratch.pages.iter().copied().find(|&p| scratch.is_ancestor(p, comp_id)).unwrap_or_else(|| scratch.pages.first().copied().unwrap_or(comp_id));
    let cmd = create_instance(&mut scratch, comp_id, page, designos_doc::Vec2::ZERO);
    let scratch_inst = match &cmd {
        Command::Insert { nodes, .. } => nodes.first().map(|n| n.id),
        _ => None,
    };
    let _ = cmd.apply(&mut scratch);
    if let Some(inst_id) = scratch_inst {
        for (k, v) in values {
            let c = set_instance_prop(&mut scratch, inst_id, k, Some(v.clone()));
            let _ = c.apply(&mut scratch);
        }
        refit_text_in_subtree(&mut scratch, inst_id);
        // Also apply bindings onto the component subtree in `scratch` so inspecting a main Component directly updates its codegen live.
        if !values.is_empty() {
            let active_comp = scratch.get(inst_id).and_then(instance_component).unwrap_or(comp_id);
            apply_bindings_to_component_subtree(&mut scratch, active_comp, values);
            refit_text_in_subtree(&mut scratch, active_comp);
            scratch.relayout_subtree(active_comp);
            scratch.relayout_subtree(inst_id);
            let code_node = if matches!(scratch.get(id).map(|n| &n.kind), Some(NodeKind::ComponentSet(_))) {
                inst_id
            } else if id == comp_id {
                active_comp
            } else {
                id
            };
            return Some((scratch, code_node, inst_id));
        }
        scratch.relayout_subtree(inst_id);
        return Some((scratch, id, inst_id));
    }

    Some((scratch, id, comp_id))
}

fn preview_texture(ctx: &Context, base_doc: &Document, id: NodeId, pg: &mut PlaygroundState) -> Option<TextureHandle> {
    let key: PreviewCacheKey = (base_doc.runtime_id(), base_doc.mutation_epoch(), id, pg.rev);
    if let Some((k, cached)) = &pg.preview_cache {
        if *k == key {
            return cached.clone();
        }
    }
    let (_, _, _) = playground_target(base_doc, id).map(|(t, c)| (t, c, ()))?;
    let (scratch, _, preview_id) = build_scratch_doc(base_doc, id, pg)?;
    let dim = scratch.get(preview_id).map(|n| n.size.x.max(n.size.y)).unwrap_or(1.0).max(1.0);
    let scale = (168.0 / dim).clamp(0.5, 1.0);
    let png = designos_export::export_png(&scratch, preview_id, scale);
    let tex = image::load_from_memory(&png).ok().map(|img| {
        let rgba = img.to_rgba8();
        let ci = egui::ColorImage::from_rgba_unmultiplied([rgba.width() as usize, rgba.height() as usize], rgba.as_raw());
        ctx.load_texture("dev-playground-preview-tex", ci, egui::TextureOptions::LINEAR)
    });
    pg.preview_cache = Some((key, tex.clone()));
    tex
}

fn toggle_switch(ui: &mut Ui, rect: Rect, key: &str, on: bool) -> Option<bool> {
    widgets::record(ui, key, rect);
    let r = ui.interact(rect, ui.id().with(("pg-tog", key)), Sense::click());
    let sw = Rect::from_min_size(Pos2::new(rect.right() - 28.0, rect.center().y - 8.0), Vec2::new(28.0, 16.0));
    let p = ui.painter();
    p.rect_filled(
        sw,
        theme::cr(8),
        if on {
            theme::BG_BRAND
        } else if r.hovered() {
            theme::BG_TERTIARY
        } else {
            theme::BG_SECONDARY
        },
    );
    let kx = if on { sw.right() - 8.0 } else { sw.left() + 8.0 };
    p.circle_filled(Pos2::new(kx, sw.center().y), 6.0, Color32::WHITE);
    r.clicked().then_some(!on)
}

fn live_text_input(ui: &mut Ui, rect: Rect, key: &str, value: &str) -> Option<String> {
    let id = ui.id().with(("pg-txt", key));
    field::field_bg(ui, rect, id);
    let mut buf = value.to_string();
    let te = egui::TextEdit::singleline(&mut buf)
        .id(id)
        .font(theme::font_body())
        .text_color(theme::TEXT)
        .frame(egui::Frame::NONE)
        .margin(egui::Margin::symmetric(theme::FIELD_PLAIN_X as i8, 0))
        .vertical_align(egui::Align::Center)
        .min_size(rect.size())
        .desired_width(rect.width());
    let out = ui.put(rect, te);
    (out.changed() && buf != value).then_some(buf)
}

/// Renders the read-only Component Playground section in the Dev Mode Inspect panel.
pub fn playground_section(app: &mut App, ui: &mut Ui, c: &mut Cur, id: NodeId) {
    let Some((target_id, comp_id)) = playground_target(&app.doc, id) else { return };
    let defs = prop_defs(&app.doc, comp_id);
    if defs.is_empty() {
        return;
    }

    section_title(ui, c, "Component playground");
    let reset_r = c.rect(c.w - PAD - 56.0, -24.0, 56.0, 24.0);
    widgets::record(ui, "dev-playground-reset", reset_r);
    if small_button(ui, reset_r, "dev-playground-reset", "Reset") {
        app.dev.playground.reset(id);
    }

    // Read-only badge + live preview stage.
    let w = c.w - 2.0 * PAD;
    let stage_h = 96.0;
    let stage_r = c.rect(PAD, 0.0, w, stage_h);
    widgets::record(ui, "dev-playground-preview", stage_r);
    ui.painter().rect_filled(stage_r, theme::cr(theme::RADIUS_MEDIUM), Color32::from_rgb(0x1E, 0x1E, 0x1E));
    ui.painter().rect_stroke(stage_r, theme::cr(theme::RADIUS_MEDIUM), Stroke::new(1.0, theme::BORDER), egui::StrokeKind::Inside);

    // Subtle 12px dot grid inside the stage so both light and dark components read cleanly.
    let mut gx = stage_r.left() + 10.0;
    while gx < stage_r.right() - 6.0 {
        let mut gy = stage_r.top() + 26.0;
        while gy < stage_r.bottom() - 6.0 {
            ui.painter().circle_filled(Pos2::new(gx, gy), 0.75, Color32::from_white_alpha(18));
            gy += 12.0;
        }
        gx += 12.0;
    }

    let badge_text = if app.dev.playground.has_overrides_for(id) { "Scratch modified" } else { "Read-only" };
    let badge_w = if app.dev.playground.has_overrides_for(id) { 88.0 } else { 58.0 };
    let badge_r = Rect::from_min_size(Pos2::new(stage_r.left() + 8.0, stage_r.top() + 6.0), Vec2::new(badge_w, 16.0));
    ui.painter().rect_filled(badge_r, theme::cr(4), theme::BG_SECONDARY);
    ui.painter().text(badge_r.center(), Align2::CENTER_CENTER, badge_text, theme::font_small(), theme::TEXT_SECONDARY);

    let canvas_area = Rect::from_min_max(Pos2::new(stage_r.left() + 12.0, stage_r.top() + 24.0), Pos2::new(stage_r.right() - 12.0, stage_r.bottom() - 8.0));
    if let Some(tex) = preview_texture(ui.ctx(), &app.doc, id, &mut app.dev.playground) {
        let sz = tex.size_vec2();
        if sz.x > 0.0 && sz.y > 0.0 {
            let k = (canvas_area.width() / sz.x).min(canvas_area.height() / sz.y).min(1.0);
            let img_r = Rect::from_center_size(canvas_area.center(), sz * k);
            ui.painter().image(tex.id(), img_r, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
        }
    }
    c.y += stage_h + 8.0;

    let all_comps: Vec<NodeId> = app.doc.nodes.values().filter(|n| matches!(n.kind, NodeKind::Component(_))).map(|n| n.id).collect();

    for def in defs {
        if matches!(def.kind, PropKind::Slot) {
            continue;
        }
        let val = current_prop_value(&app.doc, target_id, comp_id, &def, &app.dev.playground, id);
        let is_overridden = app.dev.playground.node == Some(id) && app.dev.playground.values.contains_key(&def.name);
        let row_r = c.rect(PAD, 0.0, w, 28.0);
        let label_w = 92.0;
        let clean_name = def.name.split('#').next().unwrap_or(&def.name);
        let shown = elide(ui.painter(), clean_name, &theme::font_body(), label_w - 4.0);
        ui.painter().text(
            Pos2::new(row_r.left(), row_r.center().y),
            Align2::LEFT_CENTER,
            shown,
            if is_overridden { theme::font_body_strong() } else { theme::font_body() },
            if is_overridden { theme::TEXT } else { theme::TEXT_SECONDARY },
        );
        let ctl = Rect::from_min_size(Pos2::new(row_r.left() + label_w, row_r.top() + 2.0), Vec2::new(w - label_w, 24.0));
        let key = format!("dev-playground-prop-{clean_name}");
        widgets::record(ui, &key, ctl);
        match (&def.kind, &val) {
            (PropKind::Boolean, PropValue::Bool(b)) => {
                if let Some(next) = toggle_switch(ui, ctl, &key, *b) {
                    app.dev.playground.set_prop(id, def.name.clone(), PropValue::Bool(next));
                }
            }
            (PropKind::Text, PropValue::Text(s)) => {
                if let Some(next) = live_text_input(ui, ctl, &key, s) {
                    app.dev.playground.set_prop(id, def.name.clone(), PropValue::Text(next));
                }
            }
            (PropKind::Variant { options }, v) => {
                let cur = match v {
                    PropValue::Variant(s) => s.clone(),
                    _ => options.first().cloned().unwrap_or_default(),
                };
                if let Some(i) = dropdown(ui, ctl, &key, &cur, options) {
                    if let Some(chosen) = options.get(i) {
                        app.dev.playground.set_prop(id, def.name.clone(), PropValue::Variant(chosen.clone()));
                    }
                }
            }
            (PropKind::InstanceSwap { preferred }, v) => {
                let cur = match v {
                    PropValue::Instance(cid) => comp_label(&app.doc, *cid),
                    _ => String::new(),
                };
                let candidates: Vec<NodeId> =
                    if preferred.is_empty() { all_comps.clone() } else { preferred.iter().copied().filter(|cid| app.doc.get(*cid).is_some()).collect() };
                let names: Vec<String> = candidates.iter().map(|cid| comp_label(&app.doc, *cid)).collect();
                if let Some(i) = dropdown(ui, ctl, &key, &cur, &names) {
                    if let Some(&chosen) = candidates.get(i) {
                        app.dev.playground.set_prop(id, def.name.clone(), PropValue::Instance(chosen));
                    }
                }
            }
            _ => {}
        }
        c.y += 28.0;
    }
    c.y += 6.0;
}
