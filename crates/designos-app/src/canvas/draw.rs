//! Canvas overlays: frame labels, hover/selection outlines, handles, size
//! badge, guides, marquee, pen preview and the text caret.

use super::handles::Handle;
use super::interact::selection_box;
use super::snap::GuideAxis;
use super::*;
use crate::theme;
use crate::util::*;
use designos_doc::{LayoutDirection, Node, NodeKind};
use egui::{Align2, Color32, Painter, Pos2, Stroke, StrokeKind, Vec2 as EV};
use std::time::Duration;

/// Figma UI3 signature component / variant / instance / slot purple (`#9747FF`).
pub const FIGMA_COMPONENT_PURPLE: Color32 = Color32::from_rgb(0x97, 0x47, 0xFF);
/// Figma UI3 primary selection & handle stroke blue (`#0D99FF`).
pub const FIGMA_BLUE: Color32 = Color32::from_rgb(0x0D, 0x99, 0xFF);
/// Figma UI3 auto-layout padding & gap handle pink (`#FF388C`).
pub const FIGMA_PINK: Color32 = Color32::from_rgb(0xFF, 0x38, 0x8C);
/// Figma UI3 export slice accent green (`#14AE5C`).
pub const FIGMA_SLICE_GREEN: Color32 = Color32::from_rgb(0x14, 0xAE, 0x5C);

/// Whether `n` is a Slice node (either imported `SLICE` or drawn with `Tool::Slice`).
pub fn is_slice_node(n: &Node) -> bool {
    matches!(&n.kind, NodeKind::Unknown { type_name } if type_name.eq_ignore_ascii_case("SLICE"))
        || (matches!(n.kind, NodeKind::Rectangle)
            && n.fills.is_empty()
            && n.stroke.paints.is_empty()
            && n.name.starts_with("Slice"))
}

pub fn sel_color(app: &App, id: NodeId) -> Color32 {
    let Some(n) = app.doc.get(id) else {
        return theme::CANVAS_SELECTION;
    };
    if is_slice_node(n) {
        return FIGMA_SLICE_GREEN;
    }
    if designos_doc::is_slot_container(&app.doc, id) {
        return FIGMA_COMPONENT_PURPLE;
    }
    match &n.kind {
        NodeKind::Component(_) | NodeKind::ComponentSet(_) | NodeKind::Instance { .. } => FIGMA_COMPONENT_PURPLE,
        // Layers inside an instance share the instance's purple.
        _ if n.component.source.is_some() => FIGMA_COMPONENT_PURPLE,
        _ => theme::CANVAS_SELECTION,
    }
}

fn page_is_light(app: &App) -> bool {
    match app.doc.get(app.page).map(|n| &n.kind) {
        Some(NodeKind::Page { background }) => {
            let f = |v: f32| (v * 255.0) as u8;
            theme::is_light(Color32::from_rgb(f(background.r), f(background.g), f(background.b)))
        }
        _ => true,
    }
}

/// Draws a dashed line segment between `a` and `b` in screen coordinates.
fn dashed_segment(p: &Painter, a: Pos2, b: Pos2, dash: f32, gap: f32, stroke: Stroke) {
    let d = b - a;
    let len = d.length();
    if len < 0.5 {
        return;
    }
    let dir = d / len;
    let step = (dash + gap).max(2.0);
    let mut t = 0.0f32;
    while t < len {
        let t1 = (t + dash).min(len);
        p.line_segment([a + dir * t, a + dir * t1], stroke);
        t += step;
    }
}

/// Draws a closed dashed quad given its 4 screen corners.
fn dashed_quad(p: &Painter, corners: &[Pos2; 4], dash: f32, gap: f32, stroke: Stroke) {
    for i in 0..4 {
        dashed_segment(p, corners[i], corners[(i + 1) % 4], dash, gap, stroke);
    }
}

/// Draws a hollow vector diamond glyph (`◇`) for an Instance label.
fn paint_instance_diamond(p: &Painter, center: Pos2, radius: f32, col: Color32) {
    let pts = vec![
        Pos2::new(center.x, center.y - radius),
        Pos2::new(center.x + radius, center.y),
        Pos2::new(center.x, center.y + radius),
        Pos2::new(center.x - radius, center.y),
    ];
    p.add(egui::Shape::closed_line(pts, Stroke::new(1.3, col)));
}

/// Screen rects of the name labels above top-level frames, components, instances, and sections.
pub fn label_rects(app: &App, rect: ERect) -> Vec<(NodeId, ERect)> {
    let mut out = Vec::new();
    if app.view.zoom < theme::CANVAS_LABEL_MAX_ZOOM_HIDE {
        return out;
    }
    let Some(page) = app.doc.get(app.page) else { return out };
    for id in &page.children {
        let Some(n) = app.doc.get(*id) else { continue };
        if !n.visible
            || !matches!(
                n.kind,
                NodeKind::Frame(_)
                    | NodeKind::Section
                    | NodeKind::Component(_)
                    | NodeKind::ComponentSet(_)
                    | NodeKind::Instance { .. }
            )
        {
            continue;
        }
        let bb = world_aabb(&app.doc, *id);
        let tl = app.view.to_screen(rect.min, v2(bb.x, bb.y));
        let w_px = bb.w * app.view.zoom;
        let is_comp_like = matches!(
            n.kind,
            NodeKind::Component(_) | NodeKind::ComponentSet(_) | NodeKind::Instance { .. }
        );
        let icon_w = if is_comp_like { 18.0 } else { 0.0 };
        let text_w = n.name.chars().count() as f32 * theme::FONT_SIZE_LABEL_CANVAS * 0.66 + icon_w + 16.0;
        let w = text_w.min(w_px.max(24.0));
        let h = theme::FONT_SIZE_LABEL_CANVAS + 3.0;
        let r = ERect::from_min_size(Pos2::new(tl.x, tl.y - theme::CANVAS_LABEL_GAP - h), EV::new(w, h));
        if r.intersects(rect) {
            out.push((*id, r));
        }
    }
    out
}

fn outline(app: &App, p: &Painter, rect: ERect, id: NodeId, col: Color32, width: f32) {
    // a connector is outlined along its line, not around its box
    if let Some(route) = designos_doc::connector_route(&app.doc, id) {
        let pw = parent_world(&app.doc, id);
        let pts: Vec<Pos2> = route.polyline().into_iter().map(|q| app.view.to_screen(rect.min, pw.apply(q))).collect();
        p.add(egui::Shape::line(pts, Stroke::new(width + 2.0, col.gamma_multiply(0.45))));
        return;
    }
    let corners = world_corners(&app.doc, id).map(|c| app.view.to_screen(rect.min, c));
    let kind = app.doc.get(id).map(|n| &n.kind);
    let is_line = matches!(kind, Some(NodeKind::Line));
    if is_line {
        p.line_segment([corners[0], corners[1]], Stroke::new(width.max(1.0), col));
    } else if matches!(kind, Some(NodeKind::ComponentSet(_))) {
        dashed_quad(p, &corners, 6.0, 4.0, Stroke::new(width.max(1.25), FIGMA_COMPONENT_PURPLE));
    } else if app.doc.get(id).is_some_and(is_slice_node) {
        dashed_quad(p, &corners, 5.0, 3.0, Stroke::new(width.max(1.25), col));
    } else {
        p.add(egui::Shape::closed_line(corners.to_vec(), Stroke::new(width, col)));
    }
}

fn badge(p: &Painter, center_top: Pos2, text: &str, col: Color32) {
    let font = theme::font_small_strong();
    let w = crate::widgets::text_width(p, text, &font) + theme::CANVAS_BADGE_PAD_X * 2.0;
    let r = ERect::from_min_size(Pos2::new(center_top.x - w / 2.0, center_top.y), EV::new(w, theme::CANVAS_BADGE_HEIGHT));
    p.rect_filled(r, theme::cr(theme::CANVAS_BADGE_RADIUS), col);
    p.text(r.center(), Align2::CENTER_CENTER, text, font, theme::CANVAS_SIZE_BADGE_TEXT);
}

/// Paints persistent dashed borders for `ComponentSet`, component slots, and `Slice` nodes on the active page.
fn draw_component_slot_and_slice_chrome(app: &App, p: &Painter, rect: ERect) {
    let mut stack: Vec<NodeId> = app.doc.get(app.page).map(|n| n.children.clone()).unwrap_or_default();
    while let Some(id) = stack.pop() {
        let Some(n) = app.doc.get(id) else { continue };
        if !n.visible {
            continue;
        }
        stack.extend(n.children.iter().copied());
        let corners = world_corners(&app.doc, id).map(|c| app.view.to_screen(rect.min, c));
        if matches!(n.kind, NodeKind::ComponentSet(_)) && !app.is_selected(id) {
            dashed_quad(p, &corners, 6.0, 4.0, Stroke::new(1.25, FIGMA_COMPONENT_PURPLE.gamma_multiply(0.85)));
        }
        if designos_doc::is_slot_container(&app.doc, id) {
            dashed_quad(p, &corners, 4.0, 3.0, Stroke::new(1.0, FIGMA_COMPONENT_PURPLE.gamma_multiply(0.75)));
            if app.is_selected(id) || app.hover == Some(id) || app.hover_layer == Some(id) {
                let font = theme::font_small_strong();
                let w = crate::widgets::text_width(p, "Slot", &font) + 8.0;
                let pill = ERect::from_min_size(Pos2::new(corners[0].x, corners[0].y - 17.0), EV::new(w, 15.0));
                p.rect_filled(pill, theme::cr(3), FIGMA_COMPONENT_PURPLE);
                p.text(pill.center(), Align2::CENTER_CENTER, "Slot", font, Color32::WHITE);
            }
        }
        if is_slice_node(n) {
            let col = if app.is_selected(id) || app.hover == Some(id) {
                Color32::from_rgb(0xF5, 0x9E, 0x0B)
            } else {
                FIGMA_SLICE_GREEN
            };
            dashed_quad(p, &corners, 5.0, 3.0, Stroke::new(1.0, col));
            let font = theme::font_small_strong();
            let w = crate::widgets::text_width(p, "Slice", &font) + 8.0;
            let pill = ERect::from_min_size(Pos2::new(corners[0].x, corners[0].y - 16.0), EV::new(w, 14.0));
            p.rect_filled(pill, theme::cr(2), col);
            p.text(pill.center(), Align2::CENTER_CENTER, "Slice", font, Color32::WHITE);
        }
    }
}

/// Paints the Figma UI3 2D CSS Grid Overlay (`LayoutDirection::Grid`) for selected/hovered grid containers.
fn draw_css_grid_overlays(app: &App, p: &Painter, rect: ERect) {
    let mut grid_ids: Vec<NodeId> = Vec::new();
    let mut push_unique = |id: NodeId| {
        if !grid_ids.contains(&id) && super::grids::node_css_grid_overlay(&app.doc, id).is_some() {
            grid_ids.push(id);
        }
    };
    for &id in &app.selection {
        push_unique(id);
        if let Some(parent) = app.doc.get(id).and_then(|n| n.parent) {
            push_unique(parent);
        }
    }
    for h in [app.hover, app.hover_layer].into_iter().flatten() {
        push_unique(h);
    }

    let cell_stroke = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x0D, 0x99, 0xFF, 140));
    let gap_fill = Color32::from_rgba_unmultiplied(0xFF, 0x38, 0x8C, 36);
    let pill_bg = Color32::from_rgba_unmultiplied(0x1E, 0x1E, 0x1E, 230);
    let pill_border = Stroke::new(1.0, Color32::from_rgba_unmultiplied(0x0D, 0x99, 0xFF, 180));

    for grid_id in grid_ids {
        let Some(n) = app.doc.get(grid_id) else { continue };
        let Some(ov) = super::grids::node_css_grid_overlay(&app.doc, grid_id) else { continue };
        let w_xform = app.doc.world_transform(grid_id);
        let to_s = |x: f32, y: f32| app.view.to_screen(rect.min, w_xform.apply(v2(x, y)));
        let [pt, pr, pb, pl] = ov.padding;
        let inner_x0 = pl;
        let inner_x1 = (n.size.x - pr).max(pl);
        let inner_y0 = pt;
        let inner_y1 = (n.size.y - pb).max(pt);

        // 1. Subtle tinted gap strips between columns and rows.
        if ov.col_gap > 0.0 && ov.cols > 1 {
            for c in 0..(ov.cols - 1) {
                let gx0 = ov.col_x[c] + ov.col_widths[c];
                let gx1 = gx0 + ov.col_gap;
                let quad = vec![to_s(gx0, inner_y0), to_s(gx1, inner_y0), to_s(gx1, inner_y1), to_s(gx0, inner_y1)];
                p.add(egui::Shape::convex_polygon(quad, gap_fill, Stroke::NONE));
            }
        }
        if ov.row_gap > 0.0 && ov.rows > 1 {
            for r in 0..(ov.rows - 1) {
                let gy0 = ov.row_y[r] + ov.row_heights[r];
                let gy1 = gy0 + ov.row_gap;
                let quad = vec![to_s(inner_x0, gy0), to_s(inner_x1, gy0), to_s(inner_x1, gy1), to_s(inner_x0, gy1)];
                p.add(egui::Shape::convex_polygon(quad, gap_fill, Stroke::NONE));
            }
        }

        // 2. Dotted/dashed cell boundary lines for every (row, col) cell rect.
        for cell in &ov.cells {
            let r = &cell.rect;
            let c_pts = [
                to_s(r.x, r.y),
                to_s(r.x + r.w, r.y),
                to_s(r.x + r.w, r.y + r.h),
                to_s(r.x, r.y + r.h),
            ];
            dashed_quad(p, &c_pts, 3.0, 3.0, cell_stroke);
        }

        // 3. Compact Column & Row Track Header Pills outside top and left edges when zoom >= 0.4.
        if app.view.zoom >= 0.4 {
            let font = theme::font_small_strong();
            for c in 0..ov.cols {
                let label = super::grids::format_grid_track(ov.col_tracks[c]);
                let top_mid = to_s(ov.col_x[c] + ov.col_widths[c] * 0.5, 0.0);
                let tw = crate::widgets::text_width(p, &label, &font) + 8.0;
                let pill = ERect::from_center_size(Pos2::new(top_mid.x, top_mid.y - 11.0), EV::new(tw, 15.0));
                p.rect_filled(pill, theme::cr(3), pill_bg);
                p.rect_stroke(pill, theme::cr(3), pill_border, StrokeKind::Inside);
                p.text(pill.center(), Align2::CENTER_CENTER, label, font.clone(), FIGMA_BLUE);
            }
            for r in 0..ov.rows {
                let label = super::grids::format_grid_track(ov.row_tracks[r]);
                let left_mid = to_s(0.0, ov.row_y[r] + ov.row_heights[r] * 0.5);
                let tw = crate::widgets::text_width(p, &label, &font) + 8.0;
                let pill = ERect::from_min_size(Pos2::new(left_mid.x - tw - 5.0, left_mid.y - 7.5), EV::new(tw, 15.0));
                p.rect_filled(pill, theme::cr(3), pill_bg);
                p.rect_stroke(pill, theme::cr(3), pill_border, StrokeKind::Inside);
                p.text(pill.center(), Align2::CENTER_CENTER, label, font.clone(), FIGMA_BLUE);
            }
        }
    }

    // 4. Child span badge (`2×1`) when a selected child inside a Grid frame has col_span > 1 or row_span > 1.
    for &id in &app.selection {
        let Some(n) = app.doc.get(id) else { continue };
        let Some(pid) = n.parent else { continue };
        let is_grid_parent = app
            .doc
            .get(pid)
            .and_then(|pn| pn.kind.frame_props())
            .and_then(|fp| fp.auto_layout.as_ref())
            .is_some_and(|al| al.direction == LayoutDirection::Grid);
        if !is_grid_parent {
            continue;
        }
        let cs = n.layout_child.col_span.max(1);
        let rs = n.layout_child.row_span.max(1);
        if cs > 1 || rs > 1 {
            let corners = world_corners(&app.doc, id).map(|c| app.view.to_screen(rect.min, c));
            let text = format!("{cs}×{rs}");
            let font = theme::font_small_strong();
            let tw = crate::widgets::text_width(p, &text, &font) + 8.0;
            let pill = ERect::from_min_size(Pos2::new(corners[1].x - tw - 4.0, corners[1].y + 4.0), EV::new(tw, 15.0));
            p.rect_filled(pill, theme::cr(3), FIGMA_COMPONENT_PURPLE);
            p.text(pill.center(), Align2::CENTER_CENTER, text, font, Color32::WHITE);
        }
    }
}

/// Paints Corner Radius Handles, Pink Auto-Layout Padding/Gap Handles, and Shape Control Handles for a single selected node.
fn draw_single_node_handles(app: &App, p: &Painter, rect: ERect, id: NodeId, dvs: &[DV; 4]) {
    let Some(n) = app.doc.get(id) else { return };
    let z = app.view.zoom;

    // 1. On-Canvas Corner Radius Handles (4px white circle with #0D99FF 1.5px stroke).
    if handles::supports_corner_radius_handles(&n.kind) {
        if let Some(cr_pts) = handles::corner_radius_handle_positions(dvs, n.corner_radii, z) {
            for pt in cr_pts {
                let c = Pos2::new(pt.x, pt.y);
                p.circle_filled(c, 4.0, Color32::WHITE);
                p.circle_stroke(c, 4.0, Stroke::new(1.5, FIGMA_BLUE));
            }
        }
    }

    // 2. On-Canvas Auto-Layout Padding & Gap Handles (`Horizontal` / `Vertical`).
    if let Some(al) = n.kind.frame_props().and_then(|fp| fp.auto_layout.as_ref()) {
        if matches!(al.direction, LayoutDirection::Horizontal | LayoutDirection::Vertical) {
            let mut child_rects = Vec::new();
            for &cid in &n.children {
                if let Some(c) = app.doc.get(cid) {
                    if c.visible && !c.layout_child.absolute {
                        child_rects.push(Rect::new(c.transform.tx, c.transform.ty, c.size.x, c.size.y));
                    }
                }
            }
            let w_xform = app.doc.world_transform(id);
            for hnd in handles::auto_layout_handles(al, n.size, &child_rects, z) {
                let sc = app.view.to_screen(rect.min, w_xform.apply(hnd.local_center));
                let pill = ERect::from_center_size(sc, EV::new(hnd.pill_size.x, hnd.pill_size.y));
                p.rect_filled(pill.expand(0.75), theme::cr(2), Color32::WHITE);
                p.rect_filled(pill, theme::cr(2), FIGMA_PINK);
            }
        }
    }

    // 3. On-Canvas Arc / Star / Polygon Control Handles.
    for pt in handles::shape_control_handles(&n.kind, dvs, z) {
        let c = Pos2::new(pt.x, pt.y);
        p.circle_filled(c, 4.0, Color32::WHITE);
        p.circle_stroke(c, 4.0, Stroke::new(1.5, FIGMA_BLUE));
    }

    // 4. Floating live parameter badge during CornerRadius or AutoLayoutDrag interaction.
    match &app.canvas.inter {
        Interaction::CornerRadius(cr) if cr.id == id => {
            let r_val = n.corner_radii[cr.corner].round() as i64;
            let c = Pos2::new(dvs[cr.corner].x, dvs[cr.corner].y - 22.0);
            badge(p, c, &format!("R {r_val}"), FIGMA_BLUE);
        }
        Interaction::AutoLayoutDrag(ad) if ad.id == id => {
            if let Some(al) = n.kind.frame_props().and_then(|fp| fp.auto_layout.as_ref()) {
                let text = match ad.kind {
                    handles::AutoLayoutHandleKind::Padding(side) => format!("Pad {}", al.padding[side.min(3)].round() as i64),
                    handles::AutoLayoutHandleKind::Gap(_) => format!("Gap {}", al.item_spacing.round() as i64),
                };
                let top_cx = (dvs[0].x + dvs[1].x) * 0.5;
                let top_cy = dvs[0].y.min(dvs[1].y) - 22.0;
                badge(p, Pos2::new(top_cx, top_cy), &text, FIGMA_PINK);
            }
        }
        _ => {}
    }
}

pub fn overlays(app: &App, ui: &Ui, rect: ERect) {
    let p = ui.painter().with_clip_rect(rect);
    let to_s = |v: DV| app.view.to_screen(rect.min, v);
    let light = page_is_light(app);

    // Persistent ComponentSet dashed border, Slot indicator, and Slice export border.
    draw_component_slot_and_slice_chrome(app, &p, rect);

    // Frame / Component / Instance / Section labels.
    let lab_col = if light { theme::CANVAS_LABEL } else { theme::CANVAS_LABEL_DARK };
    for (id, r) in label_rects(app, rect) {
        let Some(n) = app.doc.get(id) else { continue };
        let selected = app.is_selected(id);
        let hovered = app.hover == Some(id) || app.hover_layer == Some(id);
        let is_comp = matches!(n.kind, NodeKind::Component(_) | NodeKind::ComponentSet(_));
        let is_inst = matches!(n.kind, NodeKind::Instance { .. });
        let is_comp_like = is_comp || is_inst;
        // Main components, component sets, and instances carry Figma's #9747FF purple label + diamond glyph.
        let col = if selected || hovered || is_comp_like { sel_color(app, id) } else { lab_col };
        let dx = if is_comp_like { 18.0 } else { 0.0 };
        if is_comp {
            crate::icons::paint(&p, crate::icons::Icon::Component, Pos2::new(r.left() + 7.0, r.center().y), 14.0, col);
        } else if is_inst {
            paint_instance_diamond(&p, Pos2::new(r.left() + 7.0, r.center().y), 5.0, col);
        }
        let raw_name = if is_comp {
            format!("❖ {}", n.name)
        } else if is_inst {
            format!("◇ {}", n.name)
        } else {
            n.name.clone()
        };
        let name = crate::widgets::elide(&p, &raw_name, &theme::font_body(), (r.width() - dx + 8.0).max(8.0));
        p.text(Pos2::new(r.left() + dx, r.center().y), Align2::LEFT_CENTER, name, theme::font_body(), col);
    }

    // 2D CSS Grid Overlay (`LayoutDirection::Grid`).
    draw_css_grid_overlays(app, &p, rect);

    // Hover outline.
    for h in [app.hover, app.hover_layer].into_iter().flatten() {
        if !app.is_selected(h) && app.doc.get(h).is_some() {
            outline(app, &p, rect, h, sel_color(app, h), theme::CANVAS_HOVER_WIDTH);
        }
    }

    // Selection (the shape builder paints its own regions instead).
    if !app.selection.is_empty() && app.canvas.builder.is_none() {
        let multi = app.selection.len() > 1;
        for id in &app.selection {
            if app.doc.get(*id).is_some() {
                outline(app, &p, rect, *id, sel_color(app, *id), theme::CANVAS_OUTLINE_WIDTH);
            }
        }
        if let Some((corners, is_line)) = selection_box(app) {
            let col = sel_color(app, app.selection[0]);
            let s = corners.map(&to_s);
            if multi {
                p.add(egui::Shape::closed_line(s.to_vec(), Stroke::new(theme::CANVAS_OUTLINE_WIDTH, col)));
            }
            let w = (s[1] - s[0]).length();
            let h = (s[3] - s[0]).length();
            let small = w.min(h) <= theme::CANVAS_HANDLE_SIZE * 3.0;
            let only_corners = small && !is_line;
            let dvs = corners.map(|c| dv(to_s(c)));
            let only_connectors = app.selection.iter().all(|id| matches!(app.doc.get(*id).map(|n| &n.kind), Some(NodeKind::Connector(_))));
            if app.canvas.text.is_none() && app.canvas.vedit.is_none() && matches!(app.tool, tools::Tool::Move | tools::Tool::Scale) && !only_connectors {
                for hnd in Handle::ALL {
                    if is_line && !matches!(hnd, Handle::L | Handle::R) {
                        continue;
                    }
                    if only_corners && !hnd.is_corner() {
                        continue;
                    }
                    let c = handles::handle_pos(&dvs, hnd);
                    let r = ERect::from_center_size(Pos2::new(c.x, c.y), EV::splat(theme::CANVAS_HANDLE_SIZE));
                    p.rect_filled(r, 0.0, theme::CANVAS_HANDLE_FILL);
                    p.rect_stroke(r, 0.0, Stroke::new(theme::CANVAS_HANDLE_BORDER, col), StrokeKind::Inside);
                }
                if let [one] = app.selection.as_slice() {
                    draw_single_node_handles(app, &p, rect, *one, &dvs);
                }
            }
            // Size badge under the lowest corner.
            let label = if let [one] = app.selection.as_slice() {
                app.doc.get(*one).map(|n| format!("{} × {}", n.size.x.round() as i64, n.size.y.round() as i64))
            } else {
                union_all(app.selection.iter().map(|i| world_aabb(&app.doc, *i))).map(|b| format!("{} × {}", b.w.round() as i64, b.h.round() as i64))
            };
            if let Some(label) = label.filter(|_| !only_connectors) {
                let bottom = s.iter().map(|c| c.y).fold(f32::MIN, f32::max);
                let cx = s.iter().map(|c| c.x).sum::<f32>() / 4.0;
                badge(&p, Pos2::new(cx, bottom + theme::CANVAS_BADGE_OFFSET + 3.0), &label, col);
            }
        }
    }

    vector::overlay(app, &p, rect);
    super::builder::overlay(app, &p, rect);

    // Snap guides & measurements.
    let gcol = theme::GUIDE_COLOR;
    for g in &app.canvas.guides {
        let (a, b) = match g.axis {
            GuideAxis::X => (to_s(v2(g.pos, g.from)), to_s(v2(g.pos, g.to))),
            GuideAxis::Y => (to_s(v2(g.from, g.pos)), to_s(v2(g.to, g.pos))),
        };
        p.line_segment([a, b], Stroke::new(theme::GUIDE_WIDTH, gcol));
        for e in [a, b] {
            let k = theme::GUIDE_X_SIZE;
            p.line_segment([e + EV::new(-k, -k), e + EV::new(k, k)], Stroke::new(theme::GUIDE_WIDTH, gcol));
            p.line_segment([e + EV::new(-k, k), e + EV::new(k, -k)], Stroke::new(theme::GUIDE_WIDTH, gcol));
        }
    }
    for m in &app.canvas.measures {
        let a = to_s(v2(m.a.0, m.a.1));
        let b = to_s(v2(m.b.0, m.b.1));
        p.line_segment([a, b], Stroke::new(theme::GUIDE_WIDTH, gcol));
        let horiz = (a.y - b.y).abs() < 0.5;
        let tick = if horiz { EV::new(0.0, 4.0) } else { EV::new(4.0, 0.0) };
        for e in [a, b] {
            p.line_segment([e - tick, e + tick], Stroke::new(theme::GUIDE_WIDTH, gcol));
        }
        let mid = Pos2::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
        let text = format!("{}", m.value.round() as i64);
        let font = theme::font_small_strong();
        let w = crate::widgets::text_width(&p, &text, &font) + theme::GUIDE_TEXT_PAD * 2.0;
        let r = ERect::from_center_size(mid + EV::new(0.0, if horiz { -10.0 } else { 0.0 }), EV::new(w, theme::GUIDE_TEXT_HEIGHT));
        p.rect_filled(r, theme::cr(theme::CANVAS_BADGE_RADIUS), gcol);
        p.text(r.center(), Align2::CENTER_CENTER, text, font, theme::GUIDE_TEXT);
    }

    // Auto-layout insertion indicator.
    if let Interaction::Move(m) = &app.canvas.inter {
        if let Some(r) = &m.reorder {
            p.line_segment([to_s(r.line[0]), to_s(r.line[1])], Stroke::new(theme::AL_INDICATOR_WIDTH, theme::DROP_INDICATOR));
        }
    }

    // Marquee.
    if let Interaction::Marquee { start, cur, .. } = &app.canvas.inter {
        let (a, b) = (to_s(*start), to_s(*cur));
        let r = ERect::from_two_pos(a, b);
        p.rect_filled(r, 0.0, theme::CANVAS_MARQUEE_FILL);
        p.rect_stroke(r, 0.0, Stroke::new(1.0, theme::CANVAS_SELECTION), StrokeKind::Inside);
    }

    // Pen / pencil previews.
    if let Some(pen) = &app.canvas.pen {
        draw_pen(app, &p, rect, &pen.pts, ui.input(|i| i.pointer.latest_pos()));
    }
    if let Interaction::Pencil(pts) = &app.canvas.inter {
        let line: Vec<Pos2> = pts.iter().map(|q| to_s(*q)).collect();
        if app.tool.is_brush() {
            // a marker stroke is previewed in its own ink and thickness
            let (w, a) = if app.tool == tools::Tool::Highlighter {
                (theme::BOARD_HIGHLIGHTER_WIDTH, theme::BOARD_HIGHLIGHTER_ALPHA)
            } else {
                (theme::BOARD_MARKER_WIDTH, 1.0)
            };
            let ink = crate::board::color32(app.board.ink).gamma_multiply(a);
            p.add(egui::Shape::line(line, Stroke::new(w * app.view.zoom, ink)));
        } else {
            p.add(egui::Shape::line(line, Stroke::new(theme::PEN_LINE_WIDTH, theme::CANVAS_SELECTION)));
        }
    }
    crate::board::canvas::overlay(app, &p, rect);

    draw_find_matches(app, &p, rect);

    // Text caret.
    if let Some(sess) = &app.canvas.text {
        if let Some(n) = app.doc.get(sess.id) {
            // a text layer, or the text area of a sticky, shape or connector label
            if let Some(f) = crate::board::create::text_frame(&app.doc, n) {
                let (t, o) = (&f.props, f.origin);
                let w = app.doc.world_transform(sess.id);
                for (x, y, rw, rh) in crate::textmeasure::selection_rects(t, f.size, sess.range()) {
                    let (x, y) = (x + o.x, y + o.y);
                    let quad = [v2(x, y), v2(x + rw, y), v2(x + rw, y + rh), v2(x, y + rh)].map(|c| to_s(w.apply(c)));
                    p.add(egui::Shape::convex_polygon(quad.to_vec(), theme::TEXT_SELECTION, Stroke::NONE));
                }
                let (x, y, lh) = crate::textmeasure::caret_rect_hint(t, f.size, sess.caret, sess.hint_x);
                let (x, y) = (x + o.x, y + o.y);
                let a = to_s(w.apply(v2(x, y)));
                let b = to_s(w.apply(v2(x, y + lh)));
                // Tell the platform where the caret is so IME candidate windows follow it.
                if let Some(cr) = super::textsession::caret_screen_rect(app, rect.min) {
                    ui.ctx().output_mut(|o| {
                        o.ime = Some(egui::output::IMEOutput {
                            rect: ERect::from_min_max(to_s(w.apply(v2(0.0, 0.0))), to_s(w.apply(v2(n.size.x, n.size.y)))),
                            cursor_rect: cr,
                            should_interrupt_composition: false,
                        })
                    });
                }
                let blink = sess.range().is_empty() && (ui.input(|i| i.time) * 2.0) as i64 % 2 == 0;
                if blink {
                    p.line_segment([a, b], Stroke::new(theme::TEXT_CARET_WIDTH, if light { Color32::BLACK } else { Color32::WHITE }));
                }
                ui.ctx().request_repaint_after(Duration::from_millis(250));
            }
        }
    }
}

/// Highlights the Find results inside text layers (the current one outlined), using the same
/// glyph geometry as the editor so the boxes sit exactly on the drawn characters.
fn draw_find_matches(app: &App, p: &Painter, rect: ERect) {
    if !app.find.open || app.find.results.is_empty() {
        return;
    }
    let to_s = |v: DV| app.view.to_screen(rect.min, v);
    let current = app.find.current;
    let mut drawn = 0;
    for (i, m) in app.find.results.iter().enumerate() {
        if m.field != designos_doc::find::MatchField::Text || drawn >= theme::FIND_CANVAS_MATCH_MAX {
            continue;
        }
        let Some(n) = app.doc.get(m.node) else { continue };
        let Some(f) = crate::board::create::text_frame(&app.doc, n) else { continue };
        let (t, o) = (&f.props, f.origin);
        if !n.visible || app.doc.page_of(m.node) != Some(app.page) {
            continue;
        }
        // skip layers outside the view
        let bb = world_aabb(&app.doc, m.node);
        let (a, b) = (to_s(v2(bb.x, bb.y)), to_s(v2(bb.x + bb.w, bb.y + bb.h)));
        if !ERect::from_two_pos(a, b).expand(4.0).intersects(rect) {
            continue;
        }
        let w = app.doc.world_transform(m.node);
        drawn += 1;
        for (x, y, rw, rh) in crate::textmeasure::selection_rects(t, f.size, m.range.clone()) {
            let (x, y) = (x + o.x, y + o.y);
            let quad = [v2(x, y), v2(x + rw, y), v2(x + rw, y + rh), v2(x, y + rh)].map(|c| to_s(w.apply(c)));
            let is_current = current == Some(i);
            p.add(egui::Shape::convex_polygon(
                quad.to_vec(),
                theme::FIND_CANVAS_MATCH,
                if is_current { Stroke::new(1.0, theme::FIND_CANVAS_MATCH_CURRENT) } else { Stroke::NONE },
            ));
        }
    }
}

fn dv(p: Pos2) -> DV {
    DV::new(p.x, p.y)
}

fn draw_pen(app: &App, p: &Painter, rect: ERect, pts: &[create::PenPoint], pointer: Option<Pos2>) {
    let to_s = |v: DV| app.view.to_screen(rect.min, v);
    let col = theme::CANVAS_SELECTION;
    let stroke = Stroke::new(theme::PEN_LINE_WIDTH, col);
    for w in pts.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if a.hout.is_some() || b.hin.is_some() {
            let shape = egui::epaint::CubicBezierShape::from_points_stroke(
                [to_s(a.pos), to_s(a.hout.unwrap_or(a.pos)), to_s(b.hin.unwrap_or(b.pos)), to_s(b.pos)],
                false,
                Color32::TRANSPARENT,
                stroke,
            );
            p.add(shape);
        } else {
            p.line_segment([to_s(a.pos), to_s(b.pos)], stroke);
        }
    }
    if let (Some(last), Some(ptr)) = (pts.last(), pointer) {
        if rect.contains(ptr) {
            p.line_segment([to_s(last.pos), ptr], Stroke::new(1.0, col.gamma_multiply(0.6)));
        }
    }
    for (i, pt) in pts.iter().enumerate() {
        let c = to_s(pt.pos);
        for h in [pt.hin, pt.hout].into_iter().flatten() {
            p.line_segment([c, to_s(h)], Stroke::new(1.0, col));
            p.circle_filled(to_s(h), 2.5, col);
        }
        let near_first = i == 0 && pts.len() >= 3 && pointer.map(|q| (q - c).length() < 8.0).unwrap_or(false);
        let r = ERect::from_center_size(c, EV::splat(theme::PEN_POINT_SIZE * 2.0 * if near_first { 1.5 } else { 1.0 }));
        p.rect_filled(r, 0.0, theme::PEN_POINT);
        p.rect_stroke(r, 0.0, Stroke::new(1.0, col), StrokeKind::Inside);
    }
}
