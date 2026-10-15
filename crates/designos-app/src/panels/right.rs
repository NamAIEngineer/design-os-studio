//! Right panel shell: present, tabs, zoom dropdown.

use super::properties;
use crate::app::{App, MenuKind, RightTab};
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets;
use egui::{Align2, Pos2, Rect, Sense, Ui, Vec2};

pub fn show(app: &mut App, ui: &mut Ui, rect: Rect) {
    // Version history and comment mode replace the whole inspector.
    if app.history.open {
        crate::history::panel::show(app, ui, rect);
        return;
    }
    if app.comment_mode() {
        crate::comments::panel::show(app, ui, rect);
        return;
    }
    let p = ui.painter().clone();
    p.rect_filled(rect, 0.0, theme::BG);
    widgets::vline(&p, rect.left(), rect.top(), rect.bottom(), theme::BORDER);
    let right = rect.right();
    let left = rect.left();

    // User avatar pill + chevron on top-left of right inspector header (Figma UI3).
    let av_center = Pos2::new(left + 21.0, rect.top() + theme::RIGHT_BTN_Y + theme::RIGHT_BTN_SIZE * 0.5);
    p.circle_filled(av_center, 11.0, theme::BG_BRAND);
    p.text(av_center, Align2::CENTER_CENTER, "J", theme::font_small_strong(), egui::Color32::WHITE);
    icons::paint(&p, Icon::ChevronDown, Pos2::new(av_center.x + 17.0, av_center.y), theme::CHEVRON_SMALL, theme::ICON_SECONDARY);

    // Share button (Figma UI3 #0D99FF in Design mode, #1BC47D in Dev mode).
    let share_r = Rect::from_min_size(
        Pos2::new(right - theme::RIGHT_PRESENT_FROM_RIGHT - 52.0, rect.top() + theme::RIGHT_BTN_Y + 3.0),
        Vec2::new(46.0, theme::RIGHT_BTN_SIZE - 6.0),
    );
    let share_resp = ui.interact(share_r, ui.id().with("share-btn"), Sense::click());
    let share_col = if app.ui.dev_mode {
        egui::Color32::from_rgb(0x1B, 0xC4, 0x7D)
    } else {
        egui::Color32::from_rgb(0x0D, 0x99, 0xFF)
    };
    p.rect_filled(share_r, theme::cr(6), if share_resp.hovered() { share_col.gamma_multiply(1.1) } else { share_col });
    p.text(share_r.center(), Align2::CENTER_CENTER, "Share", theme::font_body_strong(), egui::Color32::WHITE);
    crate::widgets::record(ui, "share-button", share_r);
    if share_resp.clicked() {
        app.show_toast("Copied DESIGN:OS file share link", false);
    }

    // Present button + chevron.
    let pb = Rect::from_min_size(Pos2::new(right - theme::RIGHT_PRESENT_FROM_RIGHT, rect.top() + theme::RIGHT_BTN_Y), Vec2::splat(theme::RIGHT_BTN_SIZE));
    let pr = ui.interact(pb, ui.id().with("present"), Sense::click());
    widgets::button_bg(&p, pb, &pr, false, theme::cr_each(theme::RADIUS_MEDIUM, 0, 0, theme::RADIUS_MEDIUM));
    // Outlined play triangle, ~16px tall.
    let (cx, cy) = (pb.center().x, pb.center().y);
    let (w, h) = (theme::PRESENT_ICON_W, theme::PRESENT_ICON_H);
    let x0 = cx - w / 2.0 + 0.5;
    p.add(egui::Shape::closed_line(
        vec![Pos2::new(x0, cy - h / 2.0), Pos2::new(x0 + w, cy), Pos2::new(x0, cy + h / 2.0)],
        egui::Stroke::new(theme::PRESENT_ICON_STROKE, theme::ICON),
    ));
    let cb = Rect::from_min_size(pb.right_top(), Vec2::new(theme::RIGHT_PRESENT_CHEVRON_W, theme::RIGHT_BTN_SIZE));
    let cr = ui.interact(cb, ui.id().with("present-chev"), Sense::click());
    widgets::button_bg(&p, cb, &cr, false, theme::cr_each(0, theme::RADIUS_MEDIUM, theme::RADIUS_MEDIUM, 0));
    icons::paint(&p, Icon::ChevronDown, cb.center(), theme::CHEVRON_SMALL, theme::ICON_SECONDARY);
    crate::widgets::record(ui, "present-button", pb);
    if pr.clicked() || cr.clicked() {
        app.start_present();
    }

    // Tabs: Design | Prototype (or Inspect | Plugins when Dev Mode is active, matching Figma UI3).
    let mut x = left + theme::RIGHT_TABS_X;
    let ty = rect.top() + theme::RIGHT_TABS_Y;
    let active = app.ui.right_tab.unwrap_or(RightTab::Design);
    let tab_labels = if app.ui.dev_mode {
        [(RightTab::Design, "Inspect"), (RightTab::Prototype, "Plugins")]
    } else {
        [(RightTab::Design, "Design"), (RightTab::Prototype, "Prototype")]
    };
    for (tab, label) in tab_labels {
        let sel = tab == active;
        let font = if sel { theme::font_body_strong() } else { theme::font_body() };
        let w = widgets::text_width(&p, label, &font) + theme::RIGHT_TAB_PAD_X * 2.0;
        let r = Rect::from_min_size(Pos2::new(x, ty), Vec2::new(w, theme::RIGHT_TABS_H));
        let resp = ui.interact(r, ui.id().with(("tab", label)), Sense::click());
        if sel {
            p.rect_filled(r, theme::cr(theme::RADIUS_MEDIUM), theme::BG_SECONDARY);
        } else if resp.hovered() {
            p.rect_filled(r, theme::cr(theme::RADIUS_MEDIUM), theme::BG_HOVER);
        }
        p.text(r.center(), Align2::CENTER_CENTER, label, font, if sel { theme::TEXT } else { theme::TEXT_SECONDARY });
        if resp.clicked() {
            app.ui.right_tab = Some(tab);
        }
        x += w + 4.0;
    }

    // Zoom dropdown.
    let zr = Rect::from_min_size(Pos2::new(right - theme::RIGHT_ZOOM_FROM_RIGHT, ty), Vec2::new(theme::RIGHT_ZOOM_W, theme::RIGHT_TABS_H));
    let zresp = ui.interact(zr, ui.id().with("zoom"), Sense::click());
    let open = app.ui.menu == Some(MenuKind::Zoom);
    widgets::button_bg(&p, zr, &zresp, open, theme::cr(theme::RADIUS_MEDIUM));
    p.text(Pos2::new(zr.left() + 8.0, zr.center().y), Align2::LEFT_CENTER, format!("{}%", (app.view.zoom * 100.0).round()), theme::font_body(), theme::TEXT);
    icons::paint(&p, Icon::ChevronDown, Pos2::new(zr.right() - 10.0, zr.center().y), theme::CHEVRON_SMALL, theme::ICON_SECONDARY);
    if zresp.clicked() {
        app.ui.menu = if open { None } else { Some(MenuKind::Zoom) };
        app.ui.menu_anchor = Some(zr);
    }

    let content_top = rect.top() + theme::RIGHT_CONTENT_TOP - 1.0;
    widgets::hline(&p, rect.left() + 1.0, right, content_top, theme::BORDER);
    let region = Rect::from_min_max(Pos2::new(rect.left() + 1.0, content_top + 1.0), rect.max);
    if active == RightTab::Prototype {
        super::prototype::show(app, ui, region);
    } else if app.ui.dev_mode {
        super::inspect::show(app, ui, region);
    } else {
        properties::show(app, ui, region);
    }
}
