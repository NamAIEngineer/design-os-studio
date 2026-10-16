//! Window shell: lays out the panels, routes shortcuts and drives screenshots.

use crate::app::{App, MenuKind};
use crate::canvas::tools::Tool;
use crate::canvas::{self, interact};
use crate::cli::Options;
use crate::demo;
use crate::icons::Icon;
use crate::panels::menus::{self, Action};
use crate::panels::{self, left, rail, right, toolbar};
use crate::theme;
use crate::widgets;
use egui::{Align2, Context, Event, Key, Pos2, Rect, Vec2};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};

/// Set once an `--screenshot` run has written its PNG.
pub static SHOT_OK: AtomicBool = AtomicBool::new(false);

struct Shot {
    path: PathBuf,
    size: (u32, u32),
    /// Device pixel ratio: the PNG is `size * scale` pixels, laid out as `size` points.
    scale: f32,
    frames: u32,
    requested: bool,
}

/// Height of the custom top window title bar (`🟠 design:os studio`).
pub const TOP_TITLE_BAR_HEIGHT: f32 = 28.0;

#[allow(dead_code)]
pub fn is_typing_in_agent_panel(app: &App, ctx: &Context) -> bool {
    let agent_rail_open =
        !app.ui.left_collapsed && matches!(app.ui.rail, Some(crate::app::RailItem::Agents | crate::app::RailItem::Tools));
    ctx.egui_wants_keyboard_input() || app.terminal.is_capturing_keyboard(agent_rail_open)
}

pub struct Shell {
    app: App,
    shot: Option<Shot>,
    title: String,
}

impl Shell {
    pub fn new(cc: &eframe::CreationContext<'_>, mut opts: Options) -> Self {
        // A real launch always has a library; screenshot runs only touch it for the home screen or --open-id.
        if opts.library.is_none() && (opts.screenshot.is_none() || opts.home || opts.open_id.is_some()) {
            opts.library = Some(designos_library::Library::default_root());
        }
        opts.home = opts.wants_home();
        let screenshot = opts.screenshot.is_some();
        let panel_was_none = opts.panel.is_none();
        let (gpu_flag, library) = (opts.gpu, opts.library.clone());
        let mut shell = Self::build(&cc.egui_ctx, opts);
        if !screenshot && panel_was_none {
            shell.app.ui.rail = Some(crate::app::RailItem::Agents);
            shell.app.ui.left_collapsed = false;
        }
        if let Some(rs) = &cc.wgpu_render_state {
            if let Some(max) = canvas::gpu::init(rs) {
                shell.app.canvas.gpu.available = true;
                shell.app.canvas.gpu.set_max_texture(max);
            }
        }
        // GPU canvas on by default; screenshots stay on the deterministic CPU path unless --gpu
        let saved = library.as_deref().and_then(|l| crate::comments::settings::load_flag(l, "use_gpu"));
        shell.app.canvas.gpu.pref = if screenshot { gpu_flag } else { gpu_flag || saved.unwrap_or(true) };
        shell.app.tabs.always_visible = true;
        if shell.app.home.is_none() {
            let mut tabs = std::mem::take(&mut shell.app.tabs);
            tabs.sync_from_app(&shell.app);
            shell.app.tabs = tabs;
        }
        shell
    }

    pub fn build(ctx: &Context, opts: Options) -> Self {
        theme::install_fonts(ctx);
        theme::apply(ctx);
        let mut app = App::new(match opts.fixture.as_deref() {
            Some("no-selection") => demo::reference_document(),
            Some("variables") => demo::variables_document(),
            Some(crate::showcase::FIXTURE) => crate::showcase::document(),
            Some(crate::protovars::FIXTURE) => crate::protovars::document(),
            Some("deck") => crate::decks::demo::document(),
            Some("board") => crate::board::demo::document(),
            Some("board-empty") => crate::board::create::new_document("Untitled"),
            Some(f) if crate::vecfixture::FIXTURES.contains(&f) => crate::vecfixture::document(),
            _ => demo::demo_document(),
        });
        if !matches!(opts.fixture.as_deref(), Some("variables" | "deck" | crate::showcase::FIXTURE)) {
            app.doc.name = "Untitled".into();
        }
        if opts.fixture.as_deref() == Some("deck") {
            if let Some(id) = app.deck_slides().first().copied() {
                app.select_slide(id);
            }
        }
        if opts.fixture.as_deref() == Some("comments") {
            crate::comments::fixture::apply(&mut app);
        }
        if let Some(f) = opts.fixture.as_deref().filter(|f| crate::vecfixture::FIXTURES.contains(f)) {
            crate::vecfixture::apply(&mut app, f);
        }
        if let Some(root) = opts.library.clone() {
            app.attach_library(root);
        }
        if let Some(f) = &opts.file {
            app.open_path(f.clone());
        }
        if let Some(id) = &opts.open_id {
            app.open_library_file(id);
        } else if opts.home {
            app.go_home();
        }
        if let Some(name) = opts.select.as_ref().filter(|n| n.as_str() != "none") {
            // a top-level layer (screen) wins over same-named layers deeper in the tree
            let named =
                |top: bool| app.doc.nodes.values().find(|n| &n.name == name && (!top || n.parent.is_some_and(|p| app.doc.pages.contains(&p)))).map(|n| n.id);
            if let Some(id) = named(true).or_else(|| named(false)) {
                if let Some(pg) = app.doc.page_of(id) {
                    app.page = pg;
                }
                app.selection = vec![id];
            }
        }
        if let Some(spec) = &opts.menu {
            let (w, h) = opts.size.unwrap_or((theme::DEFAULT_WINDOW.0 as u32, theme::DEFAULT_WINDOW.1 as u32));
            let screen = Rect::from_min_size(Pos2::ZERO, Vec2::new(w as f32, h as f32));
            if !menus::preset_menu(ctx, &mut app, spec, screen) {
                eprintln!("designos: unknown menu '{spec}'");
            }
        }
        if let Some(l) = &opts.dev {
            app.ui.dev_mode = true;
            app.ui.right_tab = Some(crate::app::RightTab::Design);
            app.dev.lang = match l.as_str() {
                "tailwind" => crate::dev::Language::Tailwind,
                "swiftui" => crate::dev::Language::SwiftUi,
                "compose" => crate::dev::Language::Compose,
                "react" => crate::dev::Language::React,
                _ => crate::dev::Language::Css,
            };
            if opts.dev_demo {
                crate::dev::preset::apply(&mut app);
            }
        }
        app.ui.rulers = opts.rulers || !cfg!(test);
        if opts.grid {
            app.ui.pixel_grid = true;
        }
        app.startup_zoom = opts.zoom.map(|z| z / 100.0);
        if let Some(kind) = &opts.layout_grid {
            let target = app.selection.first().copied();
            if let Some(id) = target {
                let k = ["grid", "columns", "rows"].iter().position(|n| n == kind).unwrap_or(1);
                // screenshot aid: a red 10% grid, columns with the usual 5 x 20px gutter
                let grid = crate::panels::layoutguide::default_grid(k, designos_doc::Color::rgba(255, 0, 0, 26));
                app.edit_nodes(&[id], |n| {
                    if let Some(f) = n.kind.frame_props_mut() {
                        f.grids.push(grid.clone());
                    }
                });
            }
        }
        if let Some(name) = &opts.edit_text {
            let found = app.doc.nodes.values().find(|n| &n.name == name && matches!(n.kind, designos_doc::NodeKind::Text(_))).map(|n| n.id);
            if let Some(id) = found {
                if let Some(pg) = app.doc.page_of(id) {
                    app.page = pg;
                }
                canvas::interact::begin_text_edit(&mut app, id);
                if let (Some((a, b)), Some(sess)) = (opts.select_text, app.canvas.text.as_mut()) {
                    if let Some(designos_doc::NodeKind::Text(t)) = app.doc.get(id).map(|n| &n.kind) {
                        let byte = |c: usize| t.content.char_indices().nth(c).map(|(i, _)| i).unwrap_or(t.content.len());
                        sess.anchor = byte(a);
                        sess.caret = byte(b);
                    }
                }
            } else {
                eprintln!("designos: no text layer named '{name}'");
            }
        }
        if opts.tab.as_deref() == Some("prototype") {
            app.ui.right_tab = Some(crate::app::RightTab::Prototype);
        }
        if opts.present {
            app.start_present();
            if let Some(st) = &mut app.proto.present {
                st.start_fullscreen = opts.fullscreen;
            }
            if opts.panel.as_deref() == Some("present-variables") {
                if opts.fixture.as_deref() == Some(crate::protovars::FIXTURE) {
                    crate::protovars::prime_present(&mut app);
                } else if let Some(st) = app.proto.present.as_mut() {
                    st.show_vars = true;
                }
            }
        }
        if let Some(q) = &opts.find {
            app.find.query = q.clone();
            app.open_find();
            app.find.focus = false;
            if let Some(r) = &opts.replace_with {
                app.find.replacement = r.clone();
                app.find.show_replace = true;
            }
        }
        match opts.panel.as_deref() {
            Some("agents") => {
                app.ui.rail = Some(crate::app::RailItem::Agents);
                app.ui.left_collapsed = false;
                app.terminal.ensure_default_tab(app.file_path.as_deref());
            }
            Some("agents-chat") => {
                app.ui.rail = Some(crate::app::RailItem::Agents);
                app.ui.left_collapsed = false;
                app.terminal.submit_chat_prompt("Create a SaaS analytics card in Claude Code", app.file_path.as_deref());
                let deadline = std::time::Instant::now() + std::time::Duration::from_millis(800);
                while std::time::Instant::now() < deadline {
                    app.terminal.sync_chat_from_pty();
                    if app.terminal.chat_turns.last().is_some_and(|t| !t.text.trim().is_empty()) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
            }
            Some("specs") => {
                app.ui.rail = Some(crate::app::RailItem::Specs);
                app.specs.open = true;
                app.specs.ensure_loaded();
            }
            Some("terminal") => {
                app.ui.rail = Some(crate::app::RailItem::Agents);
                app.ui.left_collapsed = false;
                app.terminal.dock_open = true;
                app.terminal.ensure_default_tab(app.file_path.as_deref());
            }
            Some("cmdk") => {
                app.cmdk.open = true;
                app.cmdk.last_audit = Some(crate::cmdk::run_design_gate_audit(&app.doc, app.page, &app.selection));
            }
            Some("assets") => {
                app.ui.rail = Some(crate::app::RailItem::Assets);
            }
            Some("interaction") => {
                app.ui.right_tab = Some(crate::app::RightTab::Prototype);
                crate::protovars::open_editor(&mut app);
            }
            Some("present-variables") => {}
            Some("libraries") => app.open_libraries(),
            Some(panel @ ("plugins" | "plugin-install")) => {
                app.open_plugins();
                app.plugins.install_form = panel == "plugin-install";
                app.plugins.fresh = app.plugins.install_form;
            }
            Some("version-history") => crate::history::preset::apply(&mut app, opts.library.clone()),
            Some(p @ ("dev-annotate" | "dev-compare")) => crate::dev::preset::panel(&mut app, p),
            Some(panel @ ("variables" | "variables-floating")) => {
                app.vars.full = panel == "variables";
                app.open_variables();
            }
            Some(p @ (
                "micro-autolayout"
                | "micro-css-grid"
                | "micro-smart-selection"
                | "micro-shape-controls"
                | "micro-vector-edit"
                | "micro-layer-picker"
                | "micro-dev-boxmodel"
                | "micro-scale-tool"
            )) => {
                apply_micro_interaction_showcase_preset(&mut app, p);
            }
            Some(spec) => {
                let (w, h) = opts.size.unwrap_or((theme::DEFAULT_WINDOW.0 as u32, theme::DEFAULT_WINDOW.1 as u32));
                panels::properties::preset_popup(&mut app, spec, Vec2::new(w as f32, h as f32));
            }
            None => {}
        }
        let shot = opts.screenshot.map(|path| Shot {
            path,
            size: opts.size.unwrap_or((theme::DEFAULT_WINDOW.0 as u32, theme::DEFAULT_WINDOW.1 as u32)),
            scale: opts.scale.unwrap_or(1.0),
            frames: 0,
            requested: false,
        });
        Self { app, shot, title: String::new() }
    }

    fn drive_screenshot(&mut self, ctx: &Context) {
        let Some(shot) = &mut self.shot else { return };
        shot.frames += 1;
        ctx.request_repaint();
        if shot.frames == 1 {
            // `scale` physical pixels per egui point (1 by default, so the PNG matches the requested size).
            ctx.set_pixels_per_point(shot.scale);
            let npp = ctx.input(|i| i.viewport().native_pixels_per_point).unwrap_or(1.0).max(0.1);
            let phys = Vec2::new(shot.size.0 as f32 * shot.scale, shot.size.1 as f32 * shot.scale);
            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(phys / npp));
        }
        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            if let Event::Screenshot { image, .. } = ev {
                let (w, h) = (image.width() as u32, image.height() as u32);
                let raw: Vec<u8> = image.pixels.iter().flat_map(|c| c.to_array()).collect();
                if let Some(buf) = image::RgbaImage::from_raw(w, h, raw) {
                    let want = ((shot.size.0 as f32 * shot.scale).round() as u32, (shot.size.1 as f32 * shot.scale).round() as u32);
                    let buf = if (w, h) != want { image::imageops::resize(&buf, want.0, want.1, image::imageops::FilterType::Lanczos3) } else { buf };
                    if let Some(dir) = shot.path.parent() {
                        let _ = std::fs::create_dir_all(dir);
                    }
                    match buf.save(&shot.path) {
                        Ok(()) => SHOT_OK.store(true, Ordering::SeqCst),
                        Err(e) => eprintln!("designos: cannot write {}: {e}", shot.path.display()),
                    }
                }
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                return;
            }
        }
        let sized = {
            let r = ctx.content_rect();
            (r.width() - shot.size.0 as f32).abs() < 1.5 && (r.height() - shot.size.1 as f32).abs() < 1.5
        };
        if !shot.requested && ((shot.frames >= 12 && sized) || shot.frames >= 60) {
            shot.requested = true;
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(egui::UserData::default()));
        }
        if shot.frames > 240 {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }

    fn handle_keys(&mut self, ctx: &Context) {
        if self.app.plugins.open || self.app.dialog.is_some() || self.app.history.save_modal.is_some() || self.app.dev.compare.is_some() {
            return;
        }
        let agent_rail_open =
            !self.app.ui.left_collapsed && matches!(self.app.ui.rail, Some(crate::app::RailItem::Agents | crate::app::RailItem::Tools));
        let capturing_terminal = self.app.terminal.is_capturing_keyboard(agent_rail_open);
        if capturing_terminal {
            return;
        }
        let typing = ctx.egui_wants_keyboard_input();
        if self.app.vars.open {
            // The Variables view handles its own keys (and undo / redo).
            return;
        }
        // Open menus own the keyboard (arrows, Enter, Escape, typing in the palette).
        if self.app.ui.menu.is_some() || self.app.ui.ctx_menu.is_some() || self.app.vars.open {
            return;
        }
        let editing_text = self.app.canvas.text.is_some();
        let eyedropper_active = self.app.props.eyedropper.is_some()
            || self.app.props.picker.as_ref().is_some_and(|pk| pk.eyedrop);
        if typing && !editing_text && !eyedropper_active {
            return;
        }
        let events = ctx.input(|i| i.events.clone());
        for ev in events {
            match ev {
                Event::Copy if !editing_text => {
                    if let Some(s) = self.app.copy_selection() {
                        ctx.copy_text(s);
                    }
                }
                Event::Cut if !editing_text => {
                    if let Some(s) = self.app.cut_selection() {
                        ctx.copy_text(s);
                    }
                }
                Event::Paste(s) if !editing_text => self.app.paste(Some(&s)),
                Event::Key { key, pressed: true, modifiers, .. } => {
                    if self.try_typography_shortcut(key, modifiers) {
                        continue;
                    }
                    if editing_text {
                        continue;
                    }
                    self.key(ctx, key, modifiers);
                }
                _ => {}
            }
        }
    }

    /// Handles Figma typography keyboard shortcuts (`S2-09`) when editing text or when a Text node is selected:
    /// - `Cmd/Ctrl + Shift + .` / `,`: font size `±1px` (`Alt` -> `±10px`)
    /// - `Cmd/Ctrl + Alt + L / T / R / J`: horizontal text alignment (`Left`, `Center`, `Right`, `Justified`)
    /// - `Alt + ArrowUp / ArrowDown`: line height `±1px` (`Shift` -> `±10px`)
    /// - `Alt + ArrowLeft / ArrowRight`: letter spacing `±0.5px` (`Shift` -> `±2.0px`)
    fn try_typography_shortcut(&mut self, key: Key, m: egui::Modifiers) -> bool {
        let app = &mut self.app;
        let has_text = app.canvas.text.is_some()
            || app
                .selection
                .iter()
                .any(|id| app.doc.get(*id).is_some_and(|n| matches!(n.kind, designos_doc::NodeKind::Text(_))));
        if !has_text {
            return false;
        }
        let sel = app.selection.clone();
        if m.command && m.shift && matches!(key, Key::Period | Key::Comma) {
            let delta = if m.alt { 10.0 } else { 1.0 } * if key == Key::Period { 1.0 } else { -1.0 };
            crate::panels::typography::set_style(app, &sel, |st| {
                st.font_size = (st.font_size + delta).clamp(1.0, 1000.0);
            });
            return true;
        }
        if m.command && m.alt && !m.shift {
            let align = match key {
                Key::L => Some(designos_doc::TextAlignH::Left),
                Key::T => Some(designos_doc::TextAlignH::Center),
                Key::R => Some(designos_doc::TextAlignH::Right),
                Key::J => Some(designos_doc::TextAlignH::Justified),
                _ => None,
            };
            if let Some(a) = align {
                if app.canvas.text.is_some() {
                    crate::canvas::textsession::base_style(app, |s| s.align_h = a);
                } else {
                    app.edit_nodes(&sel, |n| {
                        if let designos_doc::NodeKind::Text(t) = &mut n.kind {
                            t.style.align_h = a;
                            n.size = crate::layout::fit_text(t, n.size);
                        }
                    });
                }
                return true;
            }
        }
        if !m.command && m.alt {
            match key {
                Key::ArrowUp | Key::ArrowDown => {
                    let step = if m.shift { 10.0 } else { 1.0 } * if key == Key::ArrowUp { 1.0 } else { -1.0 };
                    crate::panels::typography::set_style(app, &sel, |st| {
                        let base_px = crate::richtext::line_height_px(st.line_height, st.font_size)
                            .unwrap_or_else(|| (st.font_size * 1.2).round());
                        st.line_height = designos_doc::LineHeight::Pixels((base_px + step).max(1.0));
                    });
                    return true;
                }
                Key::ArrowLeft | Key::ArrowRight => {
                    let step = if m.shift { 2.0 } else { 0.5 } * if key == Key::ArrowRight { 1.0 } else { -1.0 };
                    crate::panels::typography::set_style(app, &sel, |st| {
                        st.letter_spacing = ((st.letter_spacing + step) * 100.0).round() / 100.0;
                    });
                    return true;
                }
                _ => {}
            }
        }
        false
    }

    fn key(&mut self, ctx: &Context, key: Key, m: egui::Modifiers) {
        let app = &mut self.app;
        let cmd = m.command;
        if cmd {
            let act = match (key, m.shift, m.alt) {
                (Key::E, false, _) => Some(Action::Flatten),
                (Key::O, false, true) => Some(Action::OutlineStroke),
                (Key::J, false, false) => Some(Action::VectorJoin),
                (Key::J, true, false) => Some(Action::VectorSplit),
                (Key::Z, false, _) => Some(Action::Undo),
                (Key::Z, true, _) | (Key::Y, false, _) => Some(Action::Redo),
                (Key::O, false, _) => Some(Action::Open),
                (Key::S, false, true) => Some(Action::SaveVersion),
                (Key::S, false, _) => Some(Action::Save),
                (Key::S, true, _) => Some(Action::SaveAs),
                (Key::D, false, _) => Some(Action::Duplicate),
                (Key::C, false, true) => Some(Action::CopyProperties),
                (Key::V, false, true) => Some(Action::PasteProperties),
                (Key::V, true, false) => Some(Action::PasteOverSelection),
                (Key::R, true, _) => Some(Action::PasteToReplace),
                (Key::G, false, false) => Some(Action::Group),
                (Key::G, true, _) => Some(Action::Ungroup),
                (Key::G, false, true) => Some(Action::FrameSelection),
                (Key::K, false, true) => Some(Action::CreateComponent),
                (Key::B, false, true) => Some(Action::Detach),
                (Key::K, true, false) => Some(Action::CombineVariants),
                (Key::A, true, false) => Some(Action::SelectInverse),
                (Key::A, false, true) => {
                    if app.select_matching_layers() == 0 {
                        return menus::run(app, ctx, Action::SelectMatching);
                    }
                    None
                }
                (Key::A, false, _) => Some(Action::SelectAll),
                (Key::F, false, false) => Some(Action::Find),
                (Key::Quote, false, _) => Some(Action::TogglePixelGrid),
                (Key::Quote, true, _) => Some(Action::ToggleSnapPixel),
                (Key::Num4, true, _) => Some(Action::ToggleLayoutGrids),
                (Key::B, false, false) => Some(Action::TextBold),
                (Key::I, false, false) => Some(Action::TextItalic),
                (Key::U, false, false) => Some(Action::TextUnderline),
                (Key::Period, true, false) => Some(Action::TextSizeUp),
                (Key::Comma, true, false) => Some(Action::TextSizeDown),
                (Key::Num8, true, false) => Some(Action::TextBulletList),
                (Key::Num7, true, false) => Some(Action::TextNumberedList),
                (Key::CloseBracket, false, true) => Some(Action::BringToFront),
                (Key::OpenBracket, false, true) => Some(Action::SendToBack),
                (Key::CloseBracket, false, false) => Some(Action::BringForward),
                (Key::OpenBracket, false, false) => Some(Action::SendBackward),
                (Key::CloseBracket, true, _) => Some(Action::BringToFront),
                (Key::OpenBracket, true, _) => Some(Action::SendToBack),
                (Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown, shift, false) => {
                    let step = if shift { theme::NUDGE_BIG } else { theme::NUDGE };
                    let (dw, dh) = match key {
                        Key::ArrowLeft => (-step, 0.0),
                        Key::ArrowRight => (step, 0.0),
                        Key::ArrowUp => (0.0, -step),
                        _ => (0.0, step),
                    };
                    app.resize_selection_by(dw, dh);
                    None
                }
                (Key::Num0, false, false) => Some(Action::Zoom100),
                (Key::R, false, false) => {
                    app.rename_selection();
                    None
                }
                (Key::H, true, false) => {
                    app.toggle_selection_visible();
                    None
                }
                (Key::L, true, false) => {
                    app.toggle_selection_locked();
                    None
                }
                (Key::Equals | Key::Plus, _, _) => Some(Action::ZoomIn),
                (Key::Minus, _, _) => Some(Action::ZoomOut),
                (Key::Backslash, _, _) => Some(Action::ToggleSidebar),
                (Key::K, false, false) => Some(Action::OpenPalette),
                (Key::Backtick, _, _) => {
                    app.terminal.dock_open = !app.terminal.dock_open;
                    if app.terminal.dock_open {
                        app.terminal.ensure_default_tab(app.file_path.as_deref());
                    }
                    None
                }
                (Key::P, false, false) => {
                    app.cmdk.open = !app.cmdk.open;
                    if app.cmdk.open {
                        app.cmdk.last_audit = Some(crate::cmdk::run_design_gate_audit(&app.doc, app.page, &app.selection));
                    }
                    None
                }
                (Key::Enter, false, true) => {
                    app.start_present();
                    None
                }
                (Key::Tab, false, _) => {
                    app.next_tab();
                    None
                }
                (Key::Tab, true, _) => {
                    app.prev_tab();
                    None
                }
                (Key::T, false, false) => {
                    app.open_new_design_tab();
                    None
                }
                (Key::W, false, false) => {
                    if !app.tabs.entries.is_empty() {
                        let idx = app.tabs.active_idx;
                        app.close_tab(idx);
                    }
                    None
                }
                _ => None,
            };
            if let Some(a) = act {
                menus::run(app, ctx, a);
            }
            return;
        }
        if app.canvas.vedit.is_some() && canvas::vector::key(app, key, m) {
            return;
        }
        if app.canvas.builder.is_some() && matches!(key, Key::Escape | Key::Enter) {
            // a brush stroke in progress is dropped first; otherwise the result is applied
            if matches!(app.canvas.inter, canvas::Interaction::Builder) {
                canvas::builder::cancel_brush(app);
            } else {
                canvas::builder::finish(app);
                app.tool = Tool::Move;
            }
            return;
        }
        match key {
            Key::Delete | Key::Backspace => menus::run(app, ctx, Action::Delete),
            // an open comment draft or thread takes the Escape itself (egui drops text focus on Escape,
            // so the shortcut handler cannot tell that the key was meant for the text box)
            Key::Escape if app.comments.captures_escape() => {}
            Key::Escape => {
                if !matches!(app.canvas.inter, canvas::Interaction::None) {
                    app.cancel_interaction();
                } else if app.canvas.pen.is_some() {
                    interact::finish_pen(app);
                    app.tool = Tool::Move;
                } else if app.props.eyedropper.is_some() {
                    app.props.eyedropper = None;
                } else if app.ui.menu.is_some() {
                    app.ui.menu = None;
                } else if app.proto.editing.is_some() {
                    app.proto.editing = None;
                } else if app.find.open {
                    app.close_find();
                } else if app.layers.filter_open {
                    app.close_layer_filter();
                } else if app.tool != Tool::Move {
                    app.tool = Tool::Move;
                } else {
                    app.clear_selection();
                }
            }
            Key::Enter => {
                if app.canvas.pen.is_some() {
                    interact::finish_pen(app);
                    app.tool = Tool::Move;
                } else if m.shift {
                    if !crate::canvas::interact::navigate_hierarchy_enter(app, true) {
                        menus::run(app, ctx, Action::SelectParent);
                    }
                } else if let Some(n) = app.selection.first().filter(|_| app.selection.len() == 1).and_then(|s| app.doc.get(*s)) {
                    // A single text or vector layer is entered for editing; anything else selects its children.
                    let id = n.id;
                    if matches!(n.kind, designos_doc::NodeKind::Text(_))
                        || matches!(n.kind, designos_doc::NodeKind::Sticky(_) | designos_doc::NodeKind::ShapeWithText(_))
                    {
                        interact::begin_text_edit(app, id);
                    } else if matches!(n.kind, designos_doc::NodeKind::Connector(_)) {
                        crate::board::create::edit_label(app, id);
                    } else if canvas::vector::is_vector(app, id) {
                        // vectors and basic shapes (which become vectors on the first edit)
                        canvas::vector::enter(app, id);
                    } else if !crate::canvas::interact::navigate_hierarchy_enter(app, false) {
                        menus::run(app, ctx, Action::SelectChildren);
                    }
                } else if !crate::canvas::interact::navigate_hierarchy_enter(app, false) {
                    menus::run(app, ctx, Action::SelectChildren);
                }
            }
            // board: Tab starts the next sticky note to the right of the selected one; design: Tab/Shift+Tab traverses siblings
            Key::Tab if !m.alt => {
                if let Some(fid) = ctx.memory(|mem| mem.focused()) {
                    ctx.memory_mut(|mem| mem.surrender_focus(fid));
                }
                if app.props.eyedropper.is_some() || app.props.picker.as_ref().is_some_and(|pk| pk.eyedrop) {
                    app.props.eyedropper_format = app.props.eyedropper_format.next();
                } else if app.is_board() && !m.shift {
                    crate::board::create::tab_next(app);
                } else {
                    app.select_sibling(!m.shift);
                }
            }
            Key::Backslash => menus::run(app, ctx, Action::SelectParent),
            Key::ArrowLeft | Key::ArrowRight | Key::ArrowUp | Key::ArrowDown => {
                if !app.selection.is_empty() {
                    let step = if m.shift { theme::NUDGE_BIG } else { theme::NUDGE };
                    let (dx, dy) = match key {
                        Key::ArrowLeft => (-step, 0.0),
                        Key::ArrowRight => (step, 0.0),
                        Key::ArrowUp => (0.0, -step),
                        _ => (0.0, step),
                    };
                    app.nudge(dx, dy);
                }
            }
            Key::Num1 if m.shift => menus::run(app, ctx, Action::FitAll),
            Key::Num2 if m.shift => menus::run(app, ctx, Action::FitSelection),
            Key::Num0 if m.shift => menus::run(app, ctx, Action::Zoom100),
            Key::A if m.shift && m.alt => app.remove_auto_layout(),
            Key::A if m.shift && !m.alt => menus::run(app, ctx, Action::AutoLayout),
            Key::A if m.alt && !m.shift => menus::run(app, ctx, Action::Align(crate::ops::Align::Left)),
            Key::W if m.alt && !m.shift => menus::run(app, ctx, Action::Align(crate::ops::Align::Top)),
            Key::S if m.alt && !m.shift => menus::run(app, ctx, Action::Align(crate::ops::Align::Bottom)),
            Key::D if m.alt && !m.shift => menus::run(app, ctx, Action::Align(crate::ops::Align::Right)),
            Key::H if m.shift && !m.alt => app.flip_selection(true),
            Key::V if m.shift && !m.alt => app.flip_selection(false),
            Key::H | Key::V if m.shift && m.alt => menus::run(app, ctx, Action::Distribute),
            Key::H if m.alt && !m.shift => menus::run(app, ctx, Action::Align(crate::ops::Align::HCenter)),
            Key::V if m.alt && !m.shift => menus::run(app, ctx, Action::Align(crate::ops::Align::VCenter)),
            Key::OpenBracket if !m.shift && !m.alt => menus::run(app, ctx, Action::SendToBack),
            Key::CloseBracket if !m.shift && !m.alt => menus::run(app, ctx, Action::BringToFront),
            Key::N if !m.alt && !app.is_board() => app.step_frame(!m.shift, true),
            Key::Home if !m.alt => app.step_frame(false, false),
            Key::End if !m.alt => app.step_frame(true, false),
            Key::PageUp => app.step_page(false),
            Key::PageDown => app.step_page(true),
            Key::Num0
            | Key::Num1
            | Key::Num2
            | Key::Num3
            | Key::Num4
            | Key::Num5
            | Key::Num6
            | Key::Num7
            | Key::Num8
            | Key::Num9
                if !m.shift && !m.alt && !app.selection.is_empty() =>
            {
                let digit = match key {
                    Key::Num0 => 0,
                    Key::Num1 => 1,
                    Key::Num2 => 2,
                    Key::Num3 => 3,
                    Key::Num4 => 4,
                    Key::Num5 => 5,
                    Key::Num6 => 6,
                    Key::Num7 => 7,
                    Key::Num8 => 8,
                    Key::Num9 => 9,
                    _ => return,
                };
                app.apply_opacity_digit(digit);
            }
            Key::Q if !m.shift && !m.alt && !app.selection.is_empty() => {
                app.select_matching_layers();
            }
            Key::I if !m.shift && !m.alt && !app.selection.is_empty() => {
                app.props.eyedropper = Some(crate::panels::properties::EyedropTarget::Fill(0));
            }
            Key::R if m.shift => menus::run(app, ctx, Action::ToggleRulers),
            Key::Quote if m.shift => menus::run(app, ctx, Action::TogglePixelGrid),
            k => {
                if !m.alt {
                    let tool = if app.is_board() { Tool::from_key_board(k, m.shift) } else { Tool::from_key(k, m.shift) };
                    if let Some(t) = tool {
                        app.set_tool(t);
                    }
                }
            }
        }
    }

    /// Right-click on the canvas selects the layer under the pointer and opens the selection menu
    /// (or the `"Select layer"` picker menu when `Cmd/Ctrl` is held `#27`).
    fn context_menu(&mut self, ui: &egui::Ui, canvas_rect: Rect) {
        let (clicked, pos, cmd) = ui.ctx().input(|i| (i.pointer.secondary_clicked(), i.pointer.interact_pos(), i.modifiers.command));
        let Some(pos) = pos.filter(|p| clicked && canvas_rect.contains(*p) && !self.app.vars.open) else { return };
        if self.app.ui.rulers && !self.app.is_board() && (pos.x - canvas_rect.left() <= theme::RULER_SIZE || pos.y - canvas_rect.top() <= theme::RULER_SIZE) {
            return;
        }
        let app = &mut self.app;
        if !matches!(app.canvas.inter, canvas::Interaction::None) || app.canvas.text.is_some() {
            return;
        }
        let page = app.view.to_page(app.canvas_rect.min, pos);
        if cmd {
            let idx = interact::active_index(app);
            let candidates = crate::canvas::hit::hit_all_under(&app.doc, app.page, page, idx);
            if !candidates.is_empty() {
                app.ui.ctx_menu = None;
                app.ui.layer_picker = Some((pos, candidates));
                return;
            }
        }
        match interact::hit_under(app, page, false) {
            Some(hit) if !app.is_selected(hit) => app.select_only(hit),
            Some(_) => {}
            None => app.clear_selection(),
        }
        app.ui.ctx_menu = Some(pos);
    }

    fn toast(&mut self, ui: &egui::Ui, screen: Rect) {
        let Some(t) = &self.app.toast else { return };
        if self.app.now > t.until {
            self.app.toast = None;
            return;
        }
        let p = ui.painter();
        let font = theme::font_body();
        let w = widgets::text_width(p, &t.text, &font) + 48.0;
        let r = Rect::from_center_size(
            Pos2::new(screen.center().x, screen.bottom() - theme::TOAST_BOTTOM - theme::TOAST_HEIGHT / 2.0),
            Vec2::new(w, theme::TOAST_HEIGHT),
        );
        widgets::elevation_shadow(p, r, theme::TOAST_RADIUS);
        p.rect_filled(r, theme::cr(theme::TOAST_RADIUS), theme::BG_MENU);
        p.circle_filled(Pos2::new(r.left() + 16.0, r.center().y), 4.0, if t.error { theme::DANGER } else { theme::BG_BRAND });
        p.text(Pos2::new(r.left() + 28.0, r.center().y), Align2::LEFT_CENTER, &t.text, font, theme::TEXT);
        ui.ctx().request_repaint_after(std::time::Duration::from_millis(200));
    }
}

impl eframe::App for Shell {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        [0.17, 0.17, 0.17, 1.0]
    }

    /// Tab belongs to the text session (it nests list items) or to sibling layer traversal on the
    /// canvas: take it out of the input before egui uses it to move keyboard focus between widgets.
    fn raw_input_hook(&mut self, ctx: &egui::Context, raw: &mut egui::RawInput) {
        if self.app.plugins.open {
            return;
        }
        if ctx.memory(|m| m.focused()).is_some() {
            return;
        }
        if let Some(sess) = self.app.canvas.text.as_mut() {
            raw.events.retain(|e| match e {
                egui::Event::Key { key: egui::Key::Tab, pressed, modifiers, .. } if !modifiers.command => {
                    if *pressed {
                        sess.tabs.push(modifiers.shift);
                    }
                    false
                }
                _ => true,
            });
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.app.poll_fonts();
        if self.app.fonts.is_loading() {
            ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
        }
        self.draw(ui);
    }
}

impl Shell {
    /// Files dropped on the window: `.svg` files are placed at the drop point,
    /// anything else (the first file) is opened as a document.
    fn handle_drops(&mut self, ctx: &Context) {
        let (files, pos) = ctx.input(|i| (i.raw.dropped_files.clone(), i.pointer.hover_pos().or(i.pointer.latest_pos())));
        let is_svg = |f: &egui::DroppedFile| {
            let name = f.path.as_ref().and_then(|p| p.to_str()).unwrap_or(&f.name);
            name.rsplit('.').next().is_some_and(|e| e.eq_ignore_ascii_case("svg"))
        };
        if let Some(path) = files.iter().filter(|f| !is_svg(f)).find_map(|f| f.path.clone()) {
            self.app.open_path(path);
            return;
        }
        for (nth, f) in files.iter().filter(|f| is_svg(f)).enumerate() {
            let text = match (&f.path, &f.bytes) {
                (Some(p), _) => std::fs::read_to_string(p).map_err(|e| format!("Cannot read {}: {e}", p.display())),
                (None, Some(b)) => String::from_utf8(b.to_vec()).map_err(|_| "The SVG is not valid UTF-8".to_string()),
                (None, None) => continue,
            };
            let stem = f
                .path
                .as_deref()
                .and_then(|p| p.file_stem())
                .and_then(|s| s.to_str())
                .or_else(|| f.name.strip_suffix(".svg"))
                .filter(|s| !s.is_empty())
                .unwrap_or("svg")
                .to_string();
            match text {
                Ok(t) => crate::clipboard::drop_svg(&mut self.app, &t, &stem, pos, nth),
                Err(e) => self.app.show_toast(e, true),
            }
        }
    }

    fn sync_title(&mut self, ctx: &Context) {
        let title = theme::APP_TITLE.to_string();
        if title != self.title {
            ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
            self.title = title;
        }
    }

    /// One frame of the home screen (file browser): files dropped on the window are imported.
    fn home_frame(&mut self, ui: &mut egui::Ui, ctx: &Context) {
        let dropped: Vec<PathBuf> = ctx.input(|i| i.raw.dropped_files.iter().filter_map(|f| f.path.clone()).collect());
        if !dropped.is_empty() && self.app.dialog.is_none() {
            let target = self.app.home_target().or_else(|| self.app.home.as_ref().and_then(|h| h.snap.projects.first().map(|p| (p.id.clone(), None))));
            if let Some((p, f)) = target {
                self.app.import_into_library(&dropped, &p, f.as_deref());
            }
        }
        crate::home::show(&mut self.app, ui);
        crate::home::dialog::show(&mut self.app, ctx);
        let full = ui.max_rect();
        self.toast(ui, full);
        self.sync_title(ctx);
    }

    /// One full UI pass over the window.
    pub fn draw(&mut self, ui: &mut egui::Ui) {
        let ctx = ui.ctx().clone();
        self.app.now = ctx.input(|i| i.time);
        self.app.poll_plugins();
        self.drive_screenshot(&ctx);
        if ctx.input(|i| i.viewport().close_requested()) {
            crate::decks::ui::flush_notes(&mut self.app);
            self.app.save_close_version();
            self.app.flush_autosave();
        }
        self.app.scrub_sync(&ctx);
        let window_full = ui.max_rect();
        let show_title_bar = self.app.tabs.always_visible && self.shot.is_none() && self.app.proto.present.is_none();
        let after_title_rect = if show_title_bar {
            let title_bar_rect =
                Rect::from_min_max(window_full.min, Pos2::new(window_full.right(), window_full.top() + TOP_TITLE_BAR_HEIGHT));
            let tp = ui.painter_at(title_bar_rect);
            tp.rect_filled(title_bar_rect, 0.0, egui::Color32::from_rgb(0x18, 0x18, 0x1B));
            tp.hline(
                title_bar_rect.x_range(),
                title_bar_rect.bottom() - 0.5,
                egui::Stroke::new(1.0, egui::Color32::from_rgb(0x27, 0x27, 0x2A)),
            );
            let cy = title_bar_rect.center().y;
            let x = title_bar_rect.left() + 76.0;
            let font_bold = theme::font_body_strong();
            let font_reg = theme::font_body();
            let w_design = widgets::text_width(&tp, "design", &font_bold);
            let w_rest = widgets::text_width(&tp, ":os studio", &font_reg);
            let brand_rect = Rect::from_min_size(Pos2::new(x, title_bar_rect.top() + 2.0), Vec2::new(22.0 + w_design + w_rest + 8.0, 24.0));
            widgets::record(ui, "window-title-brand", brand_rect);
            crate::icons::paint(&tp, Icon::Logo, Pos2::new(x + 8.0, cy), 14.0, theme::BG_BRAND);
            let text_x = x + 19.0;
            tp.text(Pos2::new(text_x, cy), Align2::LEFT_CENTER, "design", font_bold, theme::TEXT);
            tp.text(Pos2::new(text_x + w_design, cy), Align2::LEFT_CENTER, ":os studio", font_reg, theme::TEXT_SECONDARY);
            let drag_resp = ui.interact(title_bar_rect, ui.id().with("window-title-bar-drag"), egui::Sense::click_and_drag());
            if drag_resp.drag_started() {
                ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
            }
            Rect::from_min_max(Pos2::new(window_full.left(), title_bar_rect.bottom()), window_full.max)
        } else {
            window_full
        };
        let show_tab_bar = self.app.tabs.is_bar_visible(self.app.home.is_some()) && self.app.proto.present.is_none();
        let full = if show_tab_bar {
            let bar_rect = Rect::from_min_max(after_title_rect.min, Pos2::new(after_title_rect.right(), after_title_rect.top() + crate::tabs::TOP_TAB_BAR_HEIGHT));
            crate::tabs::show_top_tab_bar_ui(ui, bar_rect, &mut self.app);
            Rect::from_min_max(Pos2::new(after_title_rect.left(), bar_rect.bottom()), after_title_rect.max)
        } else {
            let mut tabs = std::mem::take(&mut self.app.tabs);
            tabs.reconcile_frame(&mut self.app);
            self.app.tabs = tabs;
            after_title_rect
        };
        if self.app.home.is_some() {
            if show_tab_bar {
                let mut home_ui = ui.new_child(egui::UiBuilder::new().max_rect(full));
                self.home_frame(&mut home_ui, &ctx);
            } else {
                self.home_frame(ui, &ctx);
            }
            crate::plugins::ui::show(&mut self.app, &ctx);
            return;
        }
        self.app.autosave_tick(&ctx);
        self.app.history_tick();
        self.app.dev_tick(&ctx);
        if self.app.proto.present.is_some() {
            crate::present::show(&mut self.app, ui);
            return;
        }
        self.handle_keys(&ctx);
        if !self.app.plugins.open {
            self.handle_drops(&ctx);
        }

        ui.painter().rect_filled(full, 0.0, theme::BG);
        let board = self.app.is_board();
        let right_w = if board { 0.0 } else { theme::RIGHT_PANEL_WIDTH + theme::BORDER_WIDTH };
        let rail_w = theme::RAIL_WIDTH + theme::BORDER_WIDTH;
        let is_ai_panel = matches!(self.app.ui.rail, Some(crate::app::RailItem::Agents | crate::app::RailItem::Tools));
        let max_left_w = (full.width() - right_w - rail_w - 160.0).max(220.0);
        let base_panel_w = if is_ai_panel {
            self.app.ui.ai_panel_width.clamp(260.0, max_left_w)
        } else {
            self.app.ui.left_panel_width.clamp(200.0, max_left_w)
        };
        let panel_w = base_panel_w + theme::BORDER_WIDTH;
        let rail_rect = Rect::from_min_max(full.min, Pos2::new(full.left() + rail_w, full.bottom()));
        let left_rect = Rect::from_min_max(Pos2::new(rail_rect.right(), full.top()), Pos2::new(rail_rect.right() + panel_w, full.bottom()));
        let right_rect = Rect::from_min_max(Pos2::new(full.right() - right_w, full.top()), full.max);
        let canvas_left = if self.app.ui.left_collapsed { rail_rect.right() } else { left_rect.right() };
        let deck = self.app.is_deck();
        let notes_h = if deck { 132.0_f32.min(full.height() * 0.28) } else { 0.0 };
        let canvas_rect = Rect::from_min_max(Pos2::new(canvas_left, full.top()), Pos2::new(right_rect.left(), full.bottom() - notes_h));
        let notes_rect = Rect::from_min_max(Pos2::new(canvas_left, canvas_rect.bottom()), Pos2::new(right_rect.left(), full.bottom()));

        let full_vars = self.app.vars.open && self.app.vars.full;
        if self.shot.is_none() {
            self.app.terminal.ensure_default_tab(self.app.file_path.as_deref());
        }
        self.app.terminal.sync_live_file_context(
            &self.app.doc,
            self.app.page,
            &self.app.selection,
            self.app.file_path.as_deref(),
        );
        if let Some(ai_copy) = self.app.specs.last_ai_copy.take() {
            self.app.terminal.send_copy_for_ai(&ai_copy);
        }
        let bridge_actions = self.app.terminal.poll_live_agent_bridge();
        if !bridge_actions.is_empty() {
            execute_terminal_actions(&mut self.app, bridge_actions);
        }
        if full_vars {
            // The local-variables full view replaces the left panel, canvas and right panel.
            rail::show(&mut self.app, ui, rail_rect);
            let content = Rect::from_min_max(Pos2::new(rail_rect.right(), full.top()), full.max);
            crate::variables::view::show(&mut self.app, ui, content);
        } else {
            let specs_reader_open = false;
            let max_dock_h = (canvas_rect.height() - 100.0).max(200.0);
            let dock_h = if self.app.terminal.dock_open {
                self.app.terminal.dock_height.clamp(160.0, max_dock_h)
            } else {
                0.0
            };
            let main_canvas_rect =
                Rect::from_min_max(canvas_rect.min, Pos2::new(canvas_rect.right(), (canvas_rect.bottom() - dock_h).max(canvas_rect.top() + 80.0)));
            let dock_rect = Rect::from_min_max(Pos2::new(canvas_rect.left(), main_canvas_rect.bottom()), canvas_rect.max);
            if specs_reader_open {
                self.app.canvas_rect = main_canvas_rect;
                crate::specs::show_reader(&mut self.app, ui, main_canvas_rect);
            } else {
                canvas::show(&mut self.app, ui, main_canvas_rect);
                if deck {
                    crate::decks::ui::notes(&mut self.app, ui, notes_rect);
                }
            }
            if self.app.terminal.dock_open && dock_rect.height() >= 80.0 {
                let file_path = self.app.file_path.clone();
                let term_actions = crate::terminal::show_dock(&mut self.app.terminal, ui, dock_rect, file_path.as_deref());
                execute_terminal_actions(&mut self.app, term_actions);
                let dock_handle_rect = Rect::from_min_max(
                    Pos2::new(dock_rect.left(), dock_rect.top() - 3.0),
                    Pos2::new(dock_rect.right(), dock_rect.top() + 3.0),
                );
                widgets::record(ui, "dock-resize-handle", dock_handle_rect);
                let dock_drag = ui.interact(dock_handle_rect, ui.id().with("dock-resize-handle"), egui::Sense::drag());
                if dock_drag.hovered() || dock_drag.dragged() {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeVertical);
                    ui.painter().hline(
                        dock_rect.x_range(),
                        dock_rect.top(),
                        egui::Stroke::new(2.0, theme::BG_BRAND),
                    );
                }
                if dock_drag.dragged() {
                    if let Some(pointer) = ctx.input(|i| i.pointer.interact_pos()) {
                        self.app.terminal.dock_height = (canvas_rect.bottom() - pointer.y).clamp(160.0, max_dock_h);
                    }
                }
            }
            rail::show(&mut self.app, ui, rail_rect);
            if self.app.ui.left_collapsed {
                let b = Rect::from_min_size(Pos2::new(canvas_rect.left() + 8.0, canvas_rect.top() + 8.0), Vec2::splat(theme::PANEL_ACTION_BUTTON));
                ui.painter().rect_filled(b, theme::cr(theme::RADIUS_MEDIUM), theme::BG);
                if widgets::icon_button(ui, b, "show-sidebar", Icon::Sidebar, false, "Show sidebar").clicked() {
                    self.app.ui.left_collapsed = false;
                }
            } else if deck {
                let mode = Rect::from_min_size(left_rect.min, Vec2::new(left_rect.width(), 36.0));
                crate::decks::ui::mode_bar(&mut self.app, ui, mode);
                let body = Rect::from_min_max(Pos2::new(left_rect.left(), mode.bottom()), left_rect.max);
                if self.app.decks.design_mode {
                    left::show(&mut self.app, ui, body);
                } else {
                    crate::decks::ui::sidebar(&mut self.app, ui, body);
                }
            } else {
                left::show(&mut self.app, ui, left_rect);
            }
            if !self.app.ui.left_collapsed {
                let handle_rect = Rect::from_min_max(
                    Pos2::new(left_rect.right() - 3.0, left_rect.top()),
                    Pos2::new(left_rect.right() + 3.0, left_rect.bottom()),
                );
                widgets::record(ui, "left-panel-resize-handle", handle_rect);
                let drag_resp = ui.interact(handle_rect, ui.id().with("left-panel-resize-handle"), egui::Sense::drag());
                if drag_resp.hovered() || drag_resp.dragged() {
                    ctx.set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    ui.painter().vline(
                        left_rect.right() - 0.5,
                        left_rect.y_range(),
                        egui::Stroke::new(2.0, theme::BG_BRAND),
                    );
                }
                if drag_resp.dragged() {
                    if let Some(pointer) = ctx.input(|i| i.pointer.interact_pos()) {
                        let new_w = (pointer.x - rail_rect.right()).clamp(200.0, max_left_w);
                        if is_ai_panel {
                            self.app.ui.ai_panel_width = new_w.max(260.0);
                        } else {
                            self.app.ui.left_panel_width = new_w;
                        }
                    }
                }
            }
            if board {
                if !specs_reader_open && !self.app.terminal.dock_open {
                    crate::board::toolbar::show(&mut self.app, ui, canvas_rect_for_toolbar(full));
                    crate::board::bar::show(&mut self.app, ui, main_canvas_rect);
                    crate::board::canvas::zoom_pill(&mut self.app, ui, main_canvas_rect);
                }
            } else {
                if specs_reader_open {
                    crate::specs::show_right(&mut self.app, ui, right_rect);
                } else {
                    right::show(&mut self.app, ui, right_rect);
                }
                if !specs_reader_open {
                    let tb_rect = if deck || self.app.terminal.dock_open {
                        main_canvas_rect
                    } else {
                        canvas_rect_for_toolbar(full)
                    };
                    toolbar::show(&mut self.app, ui, tb_rect);
                }
            }
            if !specs_reader_open {
                crate::history::banner::show(&mut self.app, ui, main_canvas_rect);
                self.context_menu(ui, main_canvas_rect);
            }
            menus::show(&mut self.app, &ctx);
            panels::context::show(&mut self.app, &ctx);
            // the floating panel (or only the inspector's binding picker when closed)
            crate::variables::view::show(&mut self.app, ui, full);
        }
        if let Some(cmdk_act) = crate::cmdk::show(&ctx, &mut self.app.cmdk, &self.app.doc, self.app.page, &self.app.selection) {
            execute_cmdk_action(&mut self.app, cmdk_act);
        }
        crate::home::dialog::show(&mut self.app, &ctx);
        crate::history::modal::show(&mut self.app, &ctx);
        crate::dev::compare::show(&mut self.app, &ctx);
        crate::libraries::show(&mut self.app, &ctx);
        crate::plugins::ui::show(&mut self.app, &ctx);
        self.toast(ui, full);

        self.sync_title(&ctx);
        if self.app.ui.menu == Some(MenuKind::Main) && self.app.ui.menu_anchor.is_none() {
            self.app.ui.menu = None;
        }
    }
}

pub fn execute_terminal_actions(app: &mut App, actions: Vec<crate::terminal::TerminalAction>) {
    for action in actions {
        match action {
            crate::terminal::TerminalAction::ApplySpecJson(json) => match serde_json::from_str::<designos_doc::NodeSpec>(&json) {
                Ok(spec) => {
                    app.apply_node_spec(&spec, "Agent Terminal");
                }
                Err(e) => {
                    app.show_toast(format!("Spec parse error: {e}"), true);
                }
            },
            crate::terminal::TerminalAction::ApplySpecFile(path) => match std::fs::read_to_string(&path) {
                Ok(raw) => match serde_json::from_str::<designos_doc::NodeSpec>(&raw) {
                    Ok(spec) => {
                        let label = path.file_name().and_then(|s| s.to_str()).unwrap_or("Spec");
                        app.apply_node_spec(&spec, label);
                    }
                    Err(e) => app.show_toast(format!("Invalid spec JSON: {e}"), true),
                },
                Err(e) => app.show_toast(format!("Failed to read {}: {e}", path.display()), true),
            },
            crate::terminal::TerminalAction::SyncLiveDoc(path) => match std::fs::read_to_string(&path) {
                Ok(raw) => match designos_doc::Document::from_json(&raw) {
                    Ok(mut new_doc) => {
                        crate::io::ensure_page(&mut new_doc);
                        let keep_page = if new_doc.pages.contains(&app.page) {
                            app.page
                        } else {
                            new_doc.pages[0]
                        };
                        app.doc = new_doc;
                        app.page = keep_page;
                        app.sanitize();
                        app.changed();
                        app.fit_all();
                        app.terminal.sync_live_file_context(&app.doc, app.page, &app.selection, app.file_path.as_deref());
                        app.show_toast("Applied live Agent changes to current page", false);
                    }
                    Err(e) => app.show_toast(format!("Live doc parse error: {e}"), true),
                },
                Err(e) => app.show_toast(format!("Failed to read live doc {}: {e}", path.display()), true),
            },
            crate::terminal::TerminalAction::ReloadFromDisk => {
                if let Some(p) = app.file_path.clone() {
                    app.open_path(p);
                    app.show_toast("Synced Canvas from disk", false);
                } else {
                    app.show_toast("No file path open to sync", true);
                }
            }
            crate::terminal::TerminalAction::ExportSelectionToTerminal => {
                match crate::terminal::export_selection_and_comments_to_pty(&mut app.terminal, &app.doc, app.page, &app.selection, app.file_path.as_deref()) {
                    Ok(p) => app.show_toast(format!("Sent selection & comments -> {}", p.display()), false),
                    Err(e) => app.show_toast(e, true),
                }
            }
        }
    }
}

pub fn execute_cmdk_action(app: &mut App, action: crate::cmdk::CmdKAction) {
    match action {
        crate::cmdk::CmdKAction::ApplySpec(spec) => {
            if !app.terminal.tabs.is_empty() {
                let fp = app.file_path.clone();
                app.terminal.dispatch_figma_ai_prompt("Make Design", &spec.name, fp.as_deref());
            }
            app.apply_node_spec(&spec, "Cmd+K Synthesizer");
        }
        crate::cmdk::CmdKAction::OpenSpecReader => {
            app.ui.rail = Some(crate::app::RailItem::Agents);
            app.ui.left_collapsed = false;
            app.specs.open = false;
            app.terminal.ensure_default_tab(app.file_path.as_deref());
            app.cmdk.open = false;
        }
        crate::cmdk::CmdKAction::OpenTerminal(preset) => {
            let file_path = app.file_path.clone();
            let _ = app.terminal.spawn_tab(preset, file_path.as_deref());
            app.terminal.dock_open = true;
            app.ui.rail = Some(crate::app::RailItem::Agents);
            app.ui.left_collapsed = false;
            app.cmdk.open = false;
        }
        crate::cmdk::CmdKAction::SendSelectionToTerminal => {
            app.terminal.dock_open = true;
            app.ui.rail = Some(crate::app::RailItem::Agents);
            app.ui.left_collapsed = false;
            match crate::terminal::export_selection_and_comments_to_pty(&mut app.terminal, &app.doc, app.page, &app.selection, app.file_path.as_deref()) {
                Ok(p) => app.show_toast(format!("Sent selection & comments -> {}", p.display()), false),
                Err(e) => app.show_toast(e, true),
            }
            app.cmdk.open = false;
        }
        crate::cmdk::CmdKAction::ExportFig => {
            let out_path = app.file_path.as_ref().map(|p| p.with_extension("fig")).unwrap_or_else(|| std::path::PathBuf::from("/tmp/designos-export.fig"));
            match designos_fig::export_fig(&app.doc) {
                Ok(bytes) => {
                    if std::fs::write(&out_path, bytes).is_ok() {
                        app.show_toast(format!("Exported Figma .fig to {}", out_path.display()), false);
                    }
                }
                Err(e) => app.show_toast(format!("Export .fig failed: {e}"), true),
            }
            app.cmdk.open = false;
        }
        crate::cmdk::CmdKAction::SelectNode(id) => {
            app.select_only(id);
            app.fit_selection();
        }
        crate::cmdk::CmdKAction::AutoFixDesignGate => {
            let sel = app.selection.clone();
            let page = app.page;
            let fixed_count = crate::cmdk::auto_fix_design_gate(&mut app.doc, page, &sel);
            app.changed();
            app.cmdk.last_audit = Some(crate::cmdk::run_design_gate_audit(&app.doc, app.page, &app.selection));
            if !app.terminal.tabs.is_empty() {
                let fp = app.file_path.clone();
                app.terminal.dispatch_figma_ai_prompt("Figma AI Action", &format!("Applied {fixed_count} Canvas changes (Undo with Cmd+Z)"), fp.as_deref());
            }
            app.show_toast(format!("Applied {fixed_count} Figma AI change(s) (Cmd+Z to Undo)"), false);
        }
    }
}

/// Configures `app` with a rich, deterministic visual showcase scene for each of the 8
/// Figma A-to-Z Micro-Interaction & Feature categories (`--panel micro-*`).
pub fn apply_micro_interaction_showcase_preset(app: &mut App, preset: &str) {
    use designos_doc::{
        Affine, AutoLayout, Color, DevStatus, GridTrack, LayoutDirection, Node, NodeKind, Paint,
        Rect as DRect, Sizing, Stroke as DStroke, Vec2 as DV,
    };
    app.ui.rulers = true;
    let page = app.page;
    match preset {
        "micro-autolayout" => {
            let fid = app.doc.alloc_id();
            let mut frame = Node::frame(fid, DRect::new(2400.0, 200.0, 440.0, 220.0));
            frame.name = "Auto-Layout 5.0 · Padding & Negative Gap".into();
            frame.fills = vec![Paint::solid(Color::rgb(0x1E, 0x1E, 0x24))];
            frame.corner_radii = [16.0; 4];
            if let Some(fp) = frame.kind.frame_props_mut() {
                fp.auto_layout = Some(AutoLayout {
                    direction: LayoutDirection::Horizontal,
                    item_spacing: 20.0,
                    padding: [28.0, 28.0, 28.0, 28.0],
                    primary_sizing: Sizing::Hug,
                    counter_sizing: Sizing::Hug,
                    ..Default::default()
                });
            }
            let fid = app.doc.insert(page, frame).unwrap();
            for (i, (label, col)) in [
                ("Badge · Hug", Color::rgb(0xE3, 0x63, 0x23)),
                ("Metric · Fill", Color::rgb(0x0D, 0x99, 0xFF)),
                ("Status · Ready", Color::rgb(0x14, 0xAE, 0x5C)),
            ]
            .into_iter()
            .enumerate()
            {
                let cid = app.doc.alloc_id();
                let mut card = Node::rectangle(cid, DRect::new(0.0, 0.0, 116.0, 124.0));
                card.name = label.into();
                card.fills = vec![Paint::solid(col)];
                card.corner_radii = [10.0; 4];
                if i == 1 {
                    card.layout_child.horizontal = Sizing::Fill;
                }
                let _ = app.doc.insert(fid, card);
            }
            let mut ed = crate::edit::Edit::default();
            crate::layout::relayout_around(&mut app.doc, &mut ed, fid);
            app.selection = vec![fid];
            app.startup_zoom = Some(1.45);
            app.canvas.al_inline = Some(crate::canvas::AutoLayoutInlineEdit {
                id: fid,
                kind: crate::canvas::handles::AutoLayoutHandleKind::Gap(0),
                anchor_page: DV::new(2544.0, 290.0),
                text: "20".into(),
                request_focus: false,
            });
        }
        "micro-css-grid" => {
            let fid = app.doc.alloc_id();
            let mut grid_frame = Node::frame(fid, DRect::new(2400.0, 600.0, 560.0, 340.0));
            grid_frame.name = "2D CSS Grid · Track Pills & Span 2×1".into();
            grid_frame.fills = vec![Paint::solid(Color::rgb(0x18, 0x18, 0x1B))];
            grid_frame.corner_radii = [16.0; 4];
            let mut col_tracks = [GridTrack::Flex(1.0); 8];
            col_tracks[0] = GridTrack::Flex(1.0);
            col_tracks[1] = GridTrack::Fixed(160.0);
            col_tracks[2] = GridTrack::Hug;
            let mut row_tracks = [GridTrack::Hug; 8];
            row_tracks[0] = GridTrack::Fixed(130.0);
            row_tracks[1] = GridTrack::Flex(1.0);
            if let Some(fp) = grid_frame.kind.frame_props_mut() {
                fp.auto_layout = Some(AutoLayout {
                    direction: LayoutDirection::Grid,
                    grid_columns: 3,
                    grid_rows: 2,
                    item_spacing: 16.0,
                    counter_spacing: 16.0,
                    padding: [24.0, 24.0, 24.0, 24.0],
                    grid_col_tracks: col_tracks,
                    grid_row_tracks: row_tracks,
                    primary_sizing: Sizing::Fixed,
                    counter_sizing: Sizing::Fixed,
                    ..Default::default()
                });
            }
            let fid = app.doc.insert(page, grid_frame).unwrap();
            let specs = [
                ("Hero Span 2×1", Color::rgb(0x0D, 0x99, 0xFF), 2u16, 1u16, 260.0, 120.0),
                ("KPI Card", Color::rgb(0xE3, 0x63, 0x23), 1, 1, 130.0, 120.0),
                ("Revenue Cell", Color::rgb(0x14, 0xAE, 0x5C), 1, 1, 150.0, 120.0),
                ("Retention Cell", Color::rgb(0x27, 0x27, 0x2A), 2, 1, 280.0, 120.0),
            ];
            let mut first_child = None;
            for (name, col, cs, rs, w, h) in specs {
                let cid = app.doc.alloc_id();
                let mut c = Node::rectangle(cid, DRect::new(0.0, 0.0, w, h));
                c.name = name.into();
                c.fills = vec![Paint::solid(col)];
                c.corner_radii = [10.0; 4];
                c.layout_child.col_span = cs;
                c.layout_child.row_span = rs;
                let cid = app.doc.insert(fid, c).unwrap();
                if first_child.is_none() {
                    first_child = Some(cid);
                }
            }
            let mut ed = crate::edit::Edit::default();
            crate::layout::relayout_around(&mut app.doc, &mut ed, fid);
            app.selection = match first_child {
                Some(c0) => vec![fid, c0],
                None => vec![fid],
            };
            app.startup_zoom = Some(1.30);
        }
        "micro-smart-selection" => {
            let mut ids = Vec::new();
            for (i, (name, col)) in [
                ("Plan · Starter", Color::rgb(0x27, 0x27, 0x2A)),
                ("Plan · Pro", Color::rgb(0xE3, 0x63, 0x23)),
                ("Plan · Enterprise", Color::rgb(0x0D, 0x99, 0xFF)),
            ]
            .into_iter()
            .enumerate()
            {
                let cid = app.doc.alloc_id();
                let mut card = Node::rectangle(cid, DRect::new(2400.0 + (i as f32) * 184.0, 1100.0, 148.0, 190.0));
                card.name = name.into();
                card.fills = vec![Paint::solid(col)];
                card.corner_radii = [14.0; 4];
                ids.push(app.doc.insert(page, card).unwrap());
            }
            app.selection = ids;
            app.startup_zoom = Some(1.30);
        }
        "micro-shape-controls" => {
            let eid = app.doc.alloc_id();
            let mut ell = Node::ellipse(eid, DRect::new(2420.0, 1450.0, 220.0, 220.0));
            if let NodeKind::Ellipse(arc) = &mut ell.kind {
                arc.start = -0.45;
                arc.end = 4.15;
                arc.inner_radius = 0.52;
            }
            ell.name = "Donut Arc · Sweep & Ratio".into();
            ell.fills = vec![Paint::solid(Color::rgb(0x0D, 0x99, 0xFF))];
            let eid = app.doc.insert(page, ell).unwrap();

            let sid = app.doc.alloc_id();
            let mut star = Node::new(
                sid,
                NodeKind::Star {
                    count: 7,
                    inner_ratio: 0.44,
                },
            );
            star.name = "Star · 7 pts · 44%".into();
            star.transform = Affine::translate(2680.0, 1450.0);
            star.size = DV::new(220.0, 220.0);
            star.fills = vec![Paint::solid(Color::rgb(0xE3, 0x63, 0x23))];
            let _ = app.doc.insert(page, star);

            app.selection = vec![eid];
            app.canvas.inter = crate::canvas::Interaction::ShapeControl(crate::canvas::ShapeControlDragState {
                tx: crate::edit::Tx::new(),
                id: eid,
                kind: crate::canvas::handles::ShapeControlKind::EllipseSweep,
                start_page: DV::new(2530.0, 1560.0),
                start_count: 0,
                start_ratio: 0.52,
                start_end_angle: 4.15,
            });
            app.startup_zoom = Some(1.50);
        }
        "micro-vector-edit" => {
            let vid = app.doc.alloc_id();
            let mut vec_node = Node::new(vid, NodeKind::Polygon { count: 6 });
            vec_node.name = "Vector Network · Bezier & Midpoint Handles".into();
            vec_node.transform = Affine::translate(2420.0, 1820.0);
            vec_node.size = DV::new(340.0, 220.0);
            vec_node.fills = vec![Paint::solid(Color::rgba(13, 153, 255, 68))];
            vec_node.stroke = DStroke {
                paints: vec![Paint::solid(Color::rgb(0x0D, 0x99, 0xFF))],
                weight: 3.0,
                ..Default::default()
            };
            let vid = app.doc.insert(page, vec_node).unwrap();
            app.selection = vec![vid];
            crate::canvas::vector::enter(app, vid);
            if let Some(v) = app.canvas.vedit.as_mut() {
                v.sel = [0usize, 1usize, 2usize].into_iter().collect();
            }
            app.startup_zoom = Some(1.60);
        }
        "micro-layer-picker" => {
            if let Some(&sel) = app.selection.first() {
                let bb = crate::canvas::fast_world_aabb(&app.doc, None, sel);
                let center_page = DV::new(bb.x + bb.w * 0.5, bb.y + bb.h * 0.45);
                let mut cands = crate::canvas::hit::hit_all_under(&app.doc, app.page, center_page, None);
                if cands.is_empty() {
                    cands.push(sel);
                }
                if let Some(first) = cands.first().copied() {
                    app.hover_layer = Some(first);
                }
                app.ui.layer_picker = Some((Pos2::new(680.0, 410.0), cands));
            }
            app.startup_zoom = Some(1.05);
        }
        "micro-dev-boxmodel" => {
            app.ui.dev_mode = true;
            app.ui.right_tab = Some(crate::app::RightTab::Design);
            app.dev.lang = crate::dev::Language::React;
            crate::dev::preset::apply(app);
            // Ensure an Auto-Layout frame with generous padding is selected so the emerald Box-Model padding tint is visible.
            for (&id, n) in &mut app.doc.nodes {
                if n.parent == Some(page) && n.kind.frame_props().is_some() {
                    n.dev.status = Some(DevStatus::ReadyForDev);
                }
                if let Some(al) = n.kind.frame_props_mut().and_then(|fp| fp.auto_layout.as_mut()) {
                    if al.padding.iter().all(|&p| p >= 12.0) {
                        app.selection = vec![id];
                        app.hover = Some(id);
                    }
                }
            }
            app.startup_zoom = Some(1.40);
        }
        "micro-scale-tool" => {
            app.tool = crate::canvas::tools::Tool::Scale;
            let sid = app.doc.alloc_id();
            let mut card = Node::frame(sid, DRect::new(2420.0, 2200.0, 320.0, 200.0));
            card.name = "Scale Tool (K) · 1.25× Proportional".into();
            card.fills = vec![Paint::solid(Color::rgb(0x1E, 0x1E, 0x24))];
            card.stroke = DStroke {
                paints: vec![Paint::solid(Color::rgb(0x0D, 0x99, 0xFF))],
                weight: 2.5,
                ..Default::default()
            };
            card.corner_radii = [18.0; 4];
            let sid = app.doc.insert(page, card).unwrap();
            app.selection = vec![sid];
            app.canvas.inter = crate::canvas::Interaction::Resize(crate::canvas::ResizeState {
                tx: crate::edit::Tx::new(),
                handle: crate::canvas::handles::Handle::BR,
                kind: crate::canvas::ResizeKind::Single {
                    id: sid,
                    world0: Affine::translate(2420.0, 2200.0),
                    t0: Affine::translate(2420.0, 2200.0),
                    size0: DV::new(256.0, 160.0),
                },
                targets: vec![],
            });
            app.startup_zoom = Some(1.45);
        }
        _ => {}
    }
}

/// The toolbar is centered on the window (not the canvas).
fn canvas_rect_for_toolbar(full: Rect) -> Rect {
    full
}

#[cfg(test)]
mod tests;
