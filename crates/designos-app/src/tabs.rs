//! 1:1 Figma UI3 Multi-Tab Project Workspace (`US-10`) and Brand Wordmark Chrome (`DIR-12`).
//!
//! Manages multiple simultaneously open project tabs (`Design`, `Board`, `Deck`, `Spec`),
//! per-tab isolated state (`Document`, `History`, `page`, `selection`, `View`, `file_path`,
//! `lib_file`, `dev_mode`, `pinned`, `dirty`), seamless switching between the `Home` screen
//! and open project tabs without losing state, and the 32px top window tab bar.

use crate::app::{App, RailItem, View};
use crate::home::LibFile;
use crate::icons::{self, Icon};
use crate::theme;
use crate::widgets;
use crate::widgets::menu::{popup_menu, MenuItem, MenuResult};
use designos_doc::{Document, DocumentKind, History, NodeId};
use egui::{Align2, Color32, Pos2, Rect, Sense, Stroke, StrokeKind, Ui, Vec2};
use std::path::{Path, PathBuf};

/// Exact height of the 1:1 Figma UI3 Top Window Tab Bar (`32px`).
pub const TOP_TAB_BAR_HEIGHT: f32 = 32.0;

// Figma UI3 Top Tab Strip palette tokens
const TAB_BAR_BG: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
const TAB_BAR_BORDER: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
const TAB_ACTIVE_BG: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
const TAB_ACTIVE_BORDER: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
const TAB_HOVER_BG: Color32 = Color32::from_rgb(0x25, 0x25, 0x28);
const BADGE_DESIGN_BLUE: Color32 = Color32::from_rgb(0x0D, 0x99, 0xFF);
const BADGE_BOARD_PURPLE: Color32 = Color32::from_rgb(0x97, 0x47, 0xFF);
const BADGE_DECK_ORANGE: Color32 = Color32::from_rgb(0xFF, 0x6A, 0x3D);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectTabKind {
    Design,
    Board,
    Deck,
    Spec,
}

impl ProjectTabKind {
    pub fn from_doc(doc: &Document) -> Self {
        match doc.kind {
            DocumentKind::Design => Self::Design,
            DocumentKind::Board => Self::Board,
            DocumentKind::Deck => Self::Deck,
        }
    }

    pub fn badge_color(self) -> Color32 {
        match self {
            Self::Design => BADGE_DESIGN_BLUE,
            Self::Board => BADGE_BOARD_PURPLE,
            Self::Deck | Self::Spec => BADGE_DECK_ORANGE,
        }
    }

    pub fn icon(self) -> Icon {
        match self {
            Self::Design => Icon::Design,
            Self::Board => Icon::Sticky,
            Self::Deck => Icon::Page,
            Self::Spec => Icon::Specs,
        }
    }
}

pub struct ProjectTab {
    pub id: u64,
    pub title: String,
    pub kind: ProjectTabKind,
    pub doc: Document,
    pub hist: History,
    pub page: NodeId,
    pub selection: Vec<NodeId>,
    pub view: View,
    pub file_path: Option<PathBuf>,
    pub lib_file: Option<LibFile>,
    pub dev_mode: bool,
    pub pinned: bool,
    pub dirty: bool,
}

impl ProjectTab {
    fn from_app(id: u64, app: &App, kind_override: Option<ProjectTabKind>) -> Self {
        let title = Self::derive_title(app);
        let kind = kind_override.unwrap_or_else(|| {
            if app.ui.rail == Some(RailItem::Specs) && app.specs.open {
                ProjectTabKind::Spec
            } else {
                ProjectTabKind::from_doc(&app.doc)
            }
        });
        Self {
            id,
            title,
            kind,
            doc: app.doc.clone(),
            hist: History::default(),
            page: app.page,
            selection: app.selection.clone(),
            view: app.view,
            file_path: app.file_path.clone(),
            lib_file: app.lib_file.clone(),
            dev_mode: app.ui.dev_mode,
            pinned: false,
            dirty: app.hist.can_undo() || app.autosave.pending(),
        }
    }

    fn derive_title(app: &App) -> String {
        if let Some(lf) = &app.lib_file {
            if !lf.name.trim().is_empty() {
                return lf.name.clone();
            }
        }
        if !app.doc.name.trim().is_empty() {
            return app.doc.name.clone();
        }
        if let Some(stem) = app.file_path.as_ref().and_then(|p| p.file_stem()).and_then(|s| s.to_str()) {
            if !stem.trim().is_empty() {
                return stem.to_string();
            }
        }
        "Untitled".to_string()
    }
}

pub struct TabsState {
    pub entries: Vec<ProjectTab>,
    pub active_idx: usize,
    pub next_id: u64,
    pub ctx_menu_tab: Option<(usize, Pos2)>,
    pub new_menu_pos: Option<Pos2>,
    /// True in the live native window and screenshot runs so the 32px Top Tab Bar is always shown.
    pub always_visible: bool,
    /// True when the user navigated to `Home` while keeping open project tabs parked in `entries`.
    pub parked_in_home: bool,
    /// Tracks whether `entries[active_idx]`'s live `History` currently resides in `app.hist`.
    hist_in_app: bool,
}

impl Default for TabsState {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            active_idx: 0,
            next_id: 1,
            ctx_menu_tab: None,
            new_menu_pos: None,
            always_visible: false,
            parked_in_home: false,
            hist_in_app: true,
        }
    }
}

impl TabsState {
    fn alloc_id(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Whether the 32px Top Window Tab Bar should be rendered this frame.
    pub fn is_bar_visible(&self, _is_home: bool) -> bool {
        self.always_visible || self.entries.len() > 1
    }

    /// Snapshots lightweight active `app` metadata into `entries[active_idx]` (and initializes the
    /// first tab from `app` if `entries.is_empty()`). Never clones `app.doc` on per-frame ticks
    /// (`es:rust` Law 10).
    pub fn sync_from_app(&mut self, app: &App) {
        if self.entries.is_empty() {
            let id = self.alloc_id();
            self.entries.push(ProjectTab::from_app(id, app, None));
            self.active_idx = 0;
            self.hist_in_app = true;
            return;
        }
        if app.home.is_some() {
            return;
        }
        if self.active_idx >= self.entries.len() {
            self.active_idx = self.entries.len() - 1;
        }
        let tab = &mut self.entries[self.active_idx];
        tab.title = ProjectTab::derive_title(app);
        if tab.kind != ProjectTabKind::Spec {
            tab.kind = ProjectTabKind::from_doc(&app.doc);
        }
        tab.page = app.page;
        tab.selection = app.selection.clone();
        tab.view = app.view;
        tab.file_path = app.file_path.clone();
        tab.lib_file = app.lib_file.clone();
        tab.dev_mode = app.ui.dev_mode;
        tab.dirty = app.hist.can_undo() || tab.hist.can_undo() || app.autosave.pending();
    }

    /// Parks the active tab's state (including `app.doc`) AND moves `app.hist` into `entries[active_idx].hist`.
    fn park_active_from_app(&mut self, app: &mut App) {
        if self.entries.is_empty() {
            let id = self.alloc_id();
            let mut tab = ProjectTab::from_app(id, app, None);
            tab.hist = std::mem::take(&mut app.hist);
            tab.dirty = tab.hist.can_undo() || app.autosave.pending();
            self.entries.push(tab);
            self.active_idx = 0;
            self.hist_in_app = false;
            return;
        }
        if app.home.is_some() {
            return;
        }
        self.sync_from_app(app);
        if self.active_idx < self.entries.len() {
            self.entries[self.active_idx].doc = app.doc.clone();
        }
        if self.hist_in_app && self.active_idx < self.entries.len() {
            self.entries[self.active_idx].hist = std::mem::take(&mut app.hist);
            self.entries[self.active_idx].dirty =
                self.entries[self.active_idx].hist.can_undo() || app.autosave.pending();
            self.hist_in_app = false;
        }
    }

    /// Loads `entries[idx]` into `app.{doc, page, selection, view, file_path, lib_file, ui.dev_mode}`
    /// and sets `app.home = None`.
    pub fn apply_to_app(&self, idx: usize, app: &mut App) {
        let Some(tab) = self.entries.get(idx) else { return };
        crate::decks::ui::flush_notes(app);
        app.stop_plugin();
        app.exit_version_view();
        app.flush_autosave();

        let mut doc = tab.doc.clone();
        crate::io::ensure_page(&mut doc);
        let page = if doc.pages.contains(&tab.page) {
            tab.page
        } else {
            doc.pages[0]
        };
        app.doc = doc;
        app.page = page;
        app.selection = tab.selection.clone();
        app.view = tab.view;
        app.file_path = tab.file_path.clone();
        app.lib_file = tab.lib_file.clone();
        app.ui.dev_mode = tab.dev_mode;
        app.renaming_file = false;
        app.hover = None;
        app.hover_layer = None;
        app.canvas.reset_interaction();
        app.comments.close_popovers();
        app.layers.reset();
        app.fonts.load_for(&app.doc);
        if tab.kind == ProjectTabKind::Spec {
            app.ui.rail = Some(RailItem::Specs);
            app.specs.open = true;
            app.specs.ensure_loaded();
        } else if app.specs.open && app.ui.rail == Some(RailItem::Specs) {
            app.specs.open = false;
        }
        app.home = None;
        app.ui.menu = None;
        app.ui.ctx_menu = None;
        app.sanitize();
        app.changed();
    }

    /// Switches from the current tab (or `Home`) to `entries[idx]`, preserving `History` and view state.
    pub fn switch_to(&mut self, idx: usize, app: &mut App) {
        if self.entries.is_empty() {
            self.sync_from_app(app);
        }
        if idx >= self.entries.len() {
            return;
        }
        self.park_active_from_app(app);
        self.active_idx = idx;
        self.parked_in_home = false;
        self.apply_to_app(idx, app);
        app.hist = std::mem::take(&mut self.entries[idx].hist);
        self.entries[idx].dirty = app.hist.can_undo() || app.autosave.pending();
        self.hist_in_app = true;
    }

    /// Switches to the `Home` screen while keeping all open project tabs intact in the Top Tab Bar.
    pub fn enter_home(&mut self, app: &mut App) {
        if app.home.is_none() {
            self.park_active_from_app(app);
        }
        self.parked_in_home = true;
        if app.lib.is_some() {
            app.go_home();
        } else {
            app.home = Some(crate::home::Home::new());
            app.ui.menu = None;
            app.ui.ctx_menu = None;
        }
    }

    /// Reconciles tab state when the user opens or creates a file from the `Home` screen.
    pub fn reconcile_frame(&mut self, app: &mut App) {
        if app.home.is_some() {
            self.parked_in_home = true;
            return;
        }
        if self.entries.is_empty() {
            self.sync_from_app(app);
            return;
        }
        if self.parked_in_home {
            self.parked_in_home = false;
            // Check if the file now in `app` matches an existing tab
            if let Some(lf) = &app.lib_file {
                if let Some(existing) = self
                    .entries
                    .iter()
                    .position(|t| t.lib_file.as_ref().is_some_and(|tlf| tlf.id == lf.id))
                {
                    self.active_idx = existing;
                    self.hist_in_app = true;
                    self.sync_from_app(app);
                    return;
                }
            } else if let Some(path) = &app.file_path {
                if let Some(existing) = self
                    .entries
                    .iter()
                    .position(|t| t.file_path.as_ref() == Some(path))
                {
                    self.active_idx = existing;
                    self.hist_in_app = true;
                    self.sync_from_app(app);
                    return;
                }
            }
            // If it differs from the previously active tab, open it as a new tab!
            let differs = self.entries.get(self.active_idx).is_some_and(|cur| {
                cur.lib_file != app.lib_file || cur.file_path != app.file_path || cur.title != ProjectTab::derive_title(app)
            });
            if differs {
                let id = self.alloc_id();
                let tab = ProjectTab::from_app(id, app, None);
                self.entries.push(tab);
                self.active_idx = self.entries.len() - 1;
                self.hist_in_app = true;
                return;
            }
        }
        self.sync_from_app(app);
    }

    /// Saves the current tab, pushes a new `ProjectTab`, switches to it, and closes `Home`.
    pub fn open_new_tab(&mut self, app: &mut App, title: &str, mut doc: Document, kind: ProjectTabKind) -> usize {
        if self.entries.is_empty() && app.home.is_none() {
            self.park_active_from_app(app);
        } else if app.home.is_none() {
            self.park_active_from_app(app);
        }
        doc.name = title.to_string();
        doc.kind = match kind {
            ProjectTabKind::Board => DocumentKind::Board,
            ProjectTabKind::Deck => DocumentKind::Deck,
            ProjectTabKind::Design | ProjectTabKind::Spec => DocumentKind::Design,
        };
        crate::io::ensure_page(&mut doc);
        let page = doc.pages[0];
        let id = self.alloc_id();
        let tab = ProjectTab {
            id,
            title: title.to_string(),
            kind,
            doc,
            hist: History::default(),
            page,
            selection: Vec::new(),
            view: View::default(),
            file_path: None,
            lib_file: None,
            dev_mode: false,
            pinned: false,
            dirty: false,
        };
        self.entries.push(tab);
        let new_idx = self.entries.len() - 1;
        self.active_idx = new_idx;
        self.parked_in_home = false;
        self.apply_to_app(new_idx, app);
        app.hist = History::default();
        self.hist_in_app = true;
        app.fit_all();
        self.entries[new_idx].view = app.view;
        new_idx
    }

    /// Creates a new `"Untitled"` design tab and activates it.
    pub fn open_new_design_tab(&mut self, app: &mut App) -> usize {
        let mut doc = Document::new("Untitled");
        doc.add_page("Page 1");
        self.open_new_tab(app, "Untitled", doc, ProjectTabKind::Design)
    }

    /// Creates a new `"Untitled"` board tab and activates it.
    pub fn open_new_board_tab(&mut self, app: &mut App) -> usize {
        let doc = crate::board::create::new_document("Untitled");
        self.open_new_tab(app, "Untitled", doc, ProjectTabKind::Board)
    }

    /// Creates a new `"Untitled"` slide deck tab and activates it.
    pub fn open_new_deck_tab(&mut self, app: &mut App) -> usize {
        let mut doc = Document::new("Untitled");
        doc.kind = DocumentKind::Deck;
        doc.add_page("Slide 1");
        self.open_new_tab(app, "Untitled", doc, ProjectTabKind::Deck)
    }

    /// Focuses an existing tab with `file_path == Some(path)`, or opens `path` in a new tab.
    #[allow(dead_code)]
    pub fn open_or_focus_path(&mut self, app: &mut App, path: &Path) -> Option<usize> {
        if let Some(idx) = self
            .entries
            .iter()
            .position(|t| t.file_path.as_deref() == Some(path))
        {
            self.switch_to(idx, app);
            return Some(idx);
        }
        let doc = match crate::io::load_path(path) {
            Ok(d) => d,
            Err(e) => {
                app.show_toast(e, true);
                return None;
            }
        };
        let title = if doc.name.trim().is_empty() {
            path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Untitled")
                .to_string()
        } else {
            doc.name.clone()
        };
        let kind = ProjectTabKind::from_doc(&doc);
        let idx = self.open_new_tab(app, &title, doc, kind);
        let svg = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("svg"));
        if !svg {
            let pb = path.to_path_buf();
            self.entries[idx].file_path = Some(pb.clone());
            app.file_path = Some(pb);
        }
        Some(idx)
    }

    /// Focuses an existing tab with `lib_file.id == file_id`, or opens `file_id` from the library in a new tab.
    #[allow(dead_code)]
    pub fn open_or_focus_lib_file(&mut self, app: &mut App, file_id: &str) -> Option<usize> {
        if let Some(idx) = self
            .entries
            .iter()
            .position(|t| t.lib_file.as_ref().is_some_and(|lf| lf.id == file_id))
        {
            self.switch_to(idx, app);
            return Some(idx);
        }
        if app.home.is_none() {
            self.park_active_from_app(app);
        }
        app.open_library_file(file_id);
        if app.lib_file.as_ref().is_some_and(|lf| lf.id == file_id) {
            let id = self.alloc_id();
            let tab = ProjectTab::from_app(id, app, None);
            self.entries.push(tab);
            let idx = self.entries.len() - 1;
            self.active_idx = idx;
            self.parked_in_home = false;
            self.hist_in_app = true;
            Some(idx)
        } else {
            None
        }
    }

    /// Closes `entries[idx]` unless it is `pinned`. If `entries` becomes empty, opens `Home`;
    /// otherwise activates the nearest remaining tab.
    pub fn close_tab(&mut self, idx: usize, app: &mut App) -> bool {
        if idx >= self.entries.len() || self.entries[idx].pinned {
            return false;
        }
        if app.home.is_none() && self.active_idx < self.entries.len() {
            self.park_active_from_app(app);
        }
        self.entries.remove(idx);
        if self.entries.is_empty() {
            self.active_idx = 0;
            self.hist_in_app = false;
            self.parked_in_home = true;
            if app.lib.is_some() {
                app.go_home();
            } else {
                app.home = Some(crate::home::Home::new());
            }
            return true;
        }
        let next_idx = if idx < self.active_idx {
            self.active_idx - 1
        } else {
            self.active_idx.min(self.entries.len() - 1)
        };
        self.active_idx = next_idx;
        self.parked_in_home = false;
        self.apply_to_app(next_idx, app);
        app.hist = std::mem::take(&mut self.entries[next_idx].hist);
        self.hist_in_app = true;
        true
    }

    /// Closes all unpinned tabs except `keep_idx` and activates `keep_idx`.
    pub fn close_other_tabs(&mut self, keep_idx: usize, app: &mut App) {
        if keep_idx >= self.entries.len() {
            return;
        }
        if app.home.is_none() {
            self.park_active_from_app(app);
        }
        let keep_id = self.entries[keep_idx].id;
        self.entries.retain(|t| t.id == keep_id || t.pinned);
        if let Some(new_idx) = self.entries.iter().position(|t| t.id == keep_id) {
            self.active_idx = new_idx;
            self.parked_in_home = false;
            self.apply_to_app(new_idx, app);
            app.hist = std::mem::take(&mut self.entries[new_idx].hist);
            self.hist_in_app = true;
        }
    }

    /// Clones `entries[idx]` as `"<title> (Copy)"` in a new tab right after `idx` and activates it.
    pub fn duplicate_tab(&mut self, idx: usize, app: &mut App) -> Option<usize> {
        if idx >= self.entries.len() {
            return None;
        }
        if app.home.is_none() {
            self.park_active_from_app(app);
        }
        let src = &self.entries[idx];
        let copy_title = format!("{} (Copy)", src.title);
        let mut copy_doc = src.doc.clone();
        copy_doc.name = copy_title.clone();
        let copy_kind = src.kind;
        let copy_page = src.page;
        let copy_selection = src.selection.clone();
        let copy_view = src.view;
        let copy_dev_mode = src.dev_mode;

        let new_id = self.alloc_id();
        let new_tab = ProjectTab {
            id: new_id,
            title: copy_title,
            kind: copy_kind,
            doc: copy_doc,
            hist: History::default(),
            page: copy_page,
            selection: copy_selection,
            view: copy_view,
            file_path: None,
            lib_file: None,
            dev_mode: copy_dev_mode,
            pinned: false,
            dirty: false,
        };
        let insert_idx = (idx + 1).min(self.entries.len());
        self.entries.insert(insert_idx, new_tab);
        self.active_idx = insert_idx;
        self.parked_in_home = false;
        self.apply_to_app(insert_idx, app);
        app.hist = History::default();
        self.hist_in_app = true;
        Some(insert_idx)
    }

    /// Toggles the `pinned` state of `entries[idx]`.
    pub fn toggle_pin(&mut self, idx: usize) {
        if let Some(tab) = self.entries.get_mut(idx) {
            tab.pinned = !tab.pinned;
        }
    }

    /// Cycles to the next open project tab (`Ctrl+Tab`).
    pub fn next_tab(&mut self, app: &mut App) {
        if self.entries.len() <= 1 {
            return;
        }
        let next = (self.active_idx + 1) % self.entries.len();
        self.switch_to(next, app);
    }

    /// Cycles to the previous open project tab (`Ctrl+Shift+Tab`).
    pub fn prev_tab(&mut self, app: &mut App) {
        if self.entries.len() <= 1 {
            return;
        }
        let prev = (self.active_idx + self.entries.len() - 1) % self.entries.len();
        self.switch_to(prev, app);
    }
}

#[derive(Clone, Debug)]
enum TabUiAction {
    ClickHome,
    SwitchTab(usize),
    CloseTab(usize),
    DuplicateTab(usize),
    TogglePin(usize),
    CloseOtherTabs(usize),
    NewDesignTab,
    NewBoardTab,
    NewDeckTab,
}

/// Renders the 1:1 Figma UI3 Top Window Tab Bar (`32px` height) inside `ui`.
#[allow(dead_code)]
pub fn show_top_tab_bar(ui: &mut Ui, app: &mut App) {
    egui::Panel::top("project_tabs_bar")
        .exact_size(TOP_TAB_BAR_HEIGHT)
        .frame(egui::Frame::NONE)
        .show(ui, |ui| {
            let rect = ui.max_rect();
            show_top_tab_bar_ui(ui, rect, app);
        });
}

/// Renders the 1:1 Figma UI3 Top Window Tab Bar (`32px` height) inside `bar_rect`.
pub fn show_top_tab_bar_ui(ui: &mut Ui, bar_rect: Rect, app: &mut App) {
    {
        let mut tabs = std::mem::take(&mut app.tabs);
        tabs.reconcile_frame(app);
        app.tabs = tabs;
    }

    let p = ui.painter_at(bar_rect);
    p.rect_filled(bar_rect, 0.0, TAB_BAR_BG);
    p.hline(
        bar_rect.x_range(),
        bar_rect.bottom() - 0.5,
        Stroke::new(1.0, TAB_BAR_BORDER),
    );

    let mut actions: Vec<TabUiAction> = Vec::new();
    let cy = bar_rect.center().y;
    let mut x = bar_rect.left() + 8.0;

    // 1. Leftmost Brand Wordmark Pill: `**design**:os studio` (all lowercase, "design" bold per DIR-12)
    {
        let font_bold = theme::font_body_strong();
        let font_reg = theme::font_body();
        let w_design = widgets::text_width(&p, "design", &font_bold);
        let w_rest = widgets::text_width(&p, ":os studio", &font_reg);
        let pill_w = (22.0 + w_design + w_rest + 10.0).round();
        let brand_rect = Rect::from_min_size(
            Pos2::new(x, bar_rect.top() + 4.0),
            Vec2::new(pill_w, 24.0),
        );
        widgets::record(ui, "tab-bar-brand", brand_rect);
        icons::paint(&p, Icon::Logo, Pos2::new(brand_rect.left() + 11.0, cy), 13.0, theme::BG_BRAND);
        let text_x = brand_rect.left() + 21.0;
        p.text(
            Pos2::new(text_x, cy),
            Align2::LEFT_CENTER,
            "design",
            font_bold,
            theme::TEXT,
        );
        p.text(
            Pos2::new(text_x + w_design, cy),
            Align2::LEFT_CENTER,
            ":os studio",
            font_reg,
            theme::TEXT_SECONDARY,
        );
        x = brand_rect.right() + 4.0;
    }

    // 2. Home Tab Button (`id: "tab-bar-home"`)
    {
        let is_home = app.home.is_some();
        let home_rect = Rect::from_min_size(Pos2::new(x, bar_rect.top() + 4.0), Vec2::new(72.0, 24.0));
        widgets::record(ui, "tab-bar-home", home_rect);
        let resp = ui.interact(home_rect, ui.id().with("tab-bar-home"), Sense::click());
        if resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }
        if is_home {
            p.rect_filled(home_rect, theme::cr(6), TAB_ACTIVE_BG);
            p.rect_stroke(
                home_rect,
                theme::cr(6),
                Stroke::new(1.0, TAB_ACTIVE_BORDER),
                StrokeKind::Inside,
            );
        } else if resp.hovered() {
            p.rect_filled(home_rect, theme::cr(6), TAB_HOVER_BG);
        }
        let fg = if is_home || resp.hovered() {
            theme::TEXT
        } else {
            theme::TEXT_SECONDARY
        };
        paint_home_glyph(&p, Pos2::new(home_rect.left() + 14.0, cy), fg);
        p.text(
            Pos2::new(home_rect.left() + 26.0, cy),
            Align2::LEFT_CENTER,
            "Home",
            if is_home { theme::font_body_strong() } else { theme::font_body() },
            fg,
        );
        if resp.clicked() {
            actions.push(TabUiAction::ClickHome);
        }
        x = home_rect.right() + 6.0;
    }

    // Divider between Home button and open project tabs
    p.vline(
        x,
        (bar_rect.top() + 8.0)..=(bar_rect.bottom() - 8.0),
        Stroke::new(1.0, TAB_ACTIVE_BORDER),
    );
    x += 6.0;

    // 3. Open Project Tabs Strip
    let is_home = app.home.is_some();
    let active_idx = app.tabs.active_idx;
    let active_dirty_live = app.hist.can_undo() || app.autosave.pending();
    let avail_for_tabs = (bar_rect.right() - x - 118.0).max(120.0);
    let tab_count = app.tabs.entries.len().max(1);
    let ideal_w = (avail_for_tabs / tab_count as f32).clamp(112.0, 188.0);

    for idx in 0..app.tabs.entries.len() {
        let tab = &app.tabs.entries[idx];
        let tab_id = tab.id;
        let is_active = !is_home && idx == active_idx;
        let is_dirty = tab.dirty || (is_active && active_dirty_live);
        let tab_w = if tab.pinned && tab.title.len() <= 8 {
            ideal_w.min(132.0)
        } else {
            ideal_w
        };
        let tab_rect = Rect::from_min_size(Pos2::new(x, bar_rect.top() + 3.0), Vec2::new(tab_w, 26.0));
        if tab_rect.right() > bar_rect.right() - 104.0 {
            break;
        }
        widgets::record(ui, &format!("project-tab-{idx}"), tab_rect);
        widgets::record(ui, &format!("project-tab-id-{tab_id}"), tab_rect);

        let close_rect = Rect::from_center_size(
            Pos2::new(tab_rect.right() - 13.0, tab_rect.center().y),
            Vec2::splat(16.0),
        );
        widgets::record(ui, &format!("tab-close-{tab_id}"), close_rect);

        let close_resp = if !tab.pinned {
            Some(ui.interact(
                close_rect,
                ui.id().with(("tab-close", tab_id)),
                Sense::click(),
            ))
        } else {
            None
        };
        let tab_resp = ui.interact(
            tab_rect,
            ui.id().with(("project-tab", tab_id)),
            Sense::click(),
        );

        let hovered = tab_resp.hovered() || close_resp.as_ref().is_some_and(|r| r.hovered());
        if hovered {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        if is_active {
            p.rect_filled(tab_rect, theme::cr_each(6, 6, 4, 4), TAB_ACTIVE_BG);
            p.rect_stroke(
                tab_rect,
                theme::cr_each(6, 6, 4, 4),
                Stroke::new(1.0, TAB_ACTIVE_BORDER),
                StrokeKind::Inside,
            );
        } else if hovered {
            p.rect_filled(tab_rect, theme::cr(5), TAB_HOVER_BG);
        } else {
            // Subtle right separator between inactive tabs
            p.vline(
                tab_rect.right() + 1.0,
                (tab_rect.top() + 5.0)..=(tab_rect.bottom() - 5.0),
                Stroke::new(1.0, TAB_BAR_BORDER),
            );
        }

        // Colored file-type icon badge (or Figma Dev Mode #1BC47D `</>` badge when dev_mode is active)
        let badge_rect = Rect::from_center_size(
            Pos2::new(tab_rect.left() + 13.0, tab_rect.center().y),
            Vec2::splat(14.0),
        );
        let in_dev_mode = tab.dev_mode || (is_active && app.ui.dev_mode);
        let (badge_col, badge_icon) = if in_dev_mode {
            (Color32::from_rgb(0x1B, 0xC4, 0x7D), Icon::Dev)
        } else {
            (tab.kind.badge_color(), tab.kind.icon())
        };
        if in_dev_mode {
            p.rect_filled(badge_rect, theme::cr(3), badge_col);
            icons::paint(&p, badge_icon, badge_rect.center(), 10.0, Color32::from_rgb(0x0D, 0x1F, 0x16));
        } else {
            p.rect_filled(
                badge_rect,
                theme::cr(3),
                Color32::from_rgba_unmultiplied(badge_col.r(), badge_col.g(), badge_col.b(), 38),
            );
            icons::paint(&p, badge_icon, badge_rect.center(), 11.0, badge_col);
        }

        let mut text_left = badge_rect.right() + 6.0;
        if tab.pinned {
            icons::paint(
                &p,
                Icon::Lock,
                Pos2::new(text_left + 5.0, tab_rect.center().y),
                10.0,
                theme::TEXT_SECONDARY,
            );
            text_left += 13.0;
        }

        let right_reserve = if tab.pinned { 10.0 } else { 24.0 } + if is_dirty { 10.0 } else { 0.0 };
        let max_title_w = (tab_rect.right() - right_reserve - text_left).max(24.0);
        let font = if is_active {
            theme::font_body_strong()
        } else {
            theme::font_body()
        };
        let fg = if is_active || hovered {
            theme::TEXT
        } else {
            theme::TEXT_SECONDARY
        };
        let display_title = truncate_tab_title(&p, &tab.title, &font, max_title_w);
        p.text(
            Pos2::new(text_left, tab_rect.center().y),
            Align2::LEFT_CENTER,
            &display_title,
            font,
            fg,
        );

        // Unsaved/modified dot `•`
        if is_dirty {
            let dot_x = if tab.pinned {
                tab_rect.right() - 10.0
            } else {
                close_rect.left() - 6.0
            };
            p.circle_filled(Pos2::new(dot_x, tab_rect.center().y), 3.0, BADGE_DESIGN_BLUE);
        }

        // Close `×` button
        if let Some(cr) = &close_resp {
            if cr.hovered() {
                p.rect_filled(close_rect, theme::cr(4), TAB_ACTIVE_BORDER);
            }
            if is_active || hovered {
                icons::paint(
                    &p,
                    Icon::Close,
                    close_rect.center(),
                    10.0,
                    if cr.hovered() { theme::TEXT } else { theme::TEXT_SECONDARY },
                );
            }
        }

        let close_clicked = close_resp.as_ref().is_some_and(|r| r.clicked());
        if close_clicked {
            actions.push(TabUiAction::CloseTab(idx));
        } else if tab_resp.clicked() {
            actions.push(TabUiAction::SwitchTab(idx));
        } else if tab_resp.middle_clicked() && !tab.pinned {
            actions.push(TabUiAction::CloseTab(idx));
        }
        if tab_resp.secondary_clicked() {
            let pos = ui
                .ctx()
                .input(|i| i.pointer.interact_pos())
                .unwrap_or_else(|| tab_rect.left_bottom());
            app.tabs.ctx_menu_tab = Some((idx, pos));
        }

        x = tab_rect.right() + 3.0;
    }

    // 4. Trailing `+` New Tab Button ("tab-bar-new") — clean 1:1 Figma UI3 end-cap
    {
        let plus_rect = Rect::from_min_size(Pos2::new(x + 2.0, bar_rect.top() + 4.0), Vec2::splat(24.0));
        widgets::record(ui, "tab-bar-new", plus_rect);
        let plus_resp = ui.interact(plus_rect, ui.id().with("tab-bar-new"), Sense::click());
        if plus_resp.hovered() {
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            p.rect_filled(plus_rect, theme::cr(5), TAB_HOVER_BG);
        }
        icons::paint(
            &p,
            Icon::Plus,
            plus_rect.center(),
            13.0,
            if plus_resp.hovered() { theme::TEXT } else { theme::TEXT_SECONDARY },
        );
        if plus_resp.clicked() {
            actions.push(TabUiAction::NewDesignTab);
        }
        if plus_resp.secondary_clicked() {
            let pos = ui
                .ctx()
                .input(|i| i.pointer.interact_pos())
                .unwrap_or_else(|| plus_rect.left_bottom());
            app.tabs.new_menu_pos = Some(pos);
        }
    }

    // 4b. Right-Click Menu on `+` Button (`New Design File` / `New FigJam Board` / `New Slide Deck`)
    if let Some(pos) = app.tabs.new_menu_pos {
        let items = vec![
            MenuItem::new("New Design File").shortcut("Ctrl+T"),
            MenuItem::new("New FigJam Board"),
            MenuItem::new("New Slide Deck"),
        ];
        match popup_menu(
            ui.ctx(),
            egui::Id::new("project-tab-new-menu"),
            pos,
            188.0,
            &items,
            None,
        ) {
            MenuResult::Picked(0) => {
                app.tabs.new_menu_pos = None;
                actions.push(TabUiAction::NewDesignTab);
            }
            MenuResult::Picked(1) => {
                app.tabs.new_menu_pos = None;
                actions.push(TabUiAction::NewBoardTab);
            }
            MenuResult::Picked(2) => {
                app.tabs.new_menu_pos = None;
                actions.push(TabUiAction::NewDeckTab);
            }
            MenuResult::Picked(_) | MenuResult::Closed => {
                app.tabs.new_menu_pos = None;
            }
            MenuResult::Open => {}
        }
    }

    // 5. Right-Click Tab Context Menu
    if let Some((menu_idx, pos)) = app.tabs.ctx_menu_tab {
        if menu_idx < app.tabs.entries.len() {
            let pinned = app.tabs.entries[menu_idx].pinned;
            let can_close_others = app.tabs.entries.len() > 1;
            let items = vec![
                MenuItem::new("New Design Tab").shortcut("Ctrl+T"),
                MenuItem::new("Duplicate Tab").sep(),
                MenuItem::new(if pinned { "Unpin Tab" } else { "Pin Tab" }),
                MenuItem::new("Close Tab")
                    .shortcut("Ctrl+W")
                    .disabled(pinned),
                MenuItem::new("Close Other Tabs").disabled(!can_close_others),
            ];
            match popup_menu(
                ui.ctx(),
                egui::Id::new("project-tab-ctx-menu"),
                pos,
                184.0,
                &items,
                None,
            ) {
                MenuResult::Picked(0) => {
                    app.tabs.ctx_menu_tab = None;
                    actions.push(TabUiAction::NewDesignTab);
                }
                MenuResult::Picked(1) => {
                    app.tabs.ctx_menu_tab = None;
                    actions.push(TabUiAction::DuplicateTab(menu_idx));
                }
                MenuResult::Picked(2) => {
                    app.tabs.ctx_menu_tab = None;
                    actions.push(TabUiAction::TogglePin(menu_idx));
                }
                MenuResult::Picked(3) => {
                    app.tabs.ctx_menu_tab = None;
                    actions.push(TabUiAction::CloseTab(menu_idx));
                }
                MenuResult::Picked(4) => {
                    app.tabs.ctx_menu_tab = None;
                    actions.push(TabUiAction::CloseOtherTabs(menu_idx));
                }
                MenuResult::Picked(_) | MenuResult::Closed => {
                    app.tabs.ctx_menu_tab = None;
                }
                MenuResult::Open => {}
            }
        } else {
            app.tabs.ctx_menu_tab = None;
        }
    }

    if !actions.is_empty() {
        let mut tabs = std::mem::take(&mut app.tabs);
        for act in actions {
            match act {
                TabUiAction::ClickHome => tabs.enter_home(app),
                TabUiAction::SwitchTab(idx) => tabs.switch_to(idx, app),
                TabUiAction::CloseTab(idx) => {
                    tabs.close_tab(idx, app);
                }
                TabUiAction::DuplicateTab(idx) => {
                    tabs.duplicate_tab(idx, app);
                }
                TabUiAction::TogglePin(idx) => {
                    tabs.toggle_pin(idx);
                }
                TabUiAction::CloseOtherTabs(idx) => {
                    tabs.close_other_tabs(idx, app);
                }
                TabUiAction::NewDesignTab => {
                    tabs.open_new_design_tab(app);
                }
                TabUiAction::NewBoardTab => {
                    tabs.open_new_board_tab(app);
                }
                TabUiAction::NewDeckTab => {
                    tabs.open_new_deck_tab(app);
                }
            }
        }
        app.tabs = tabs;
    }
}

fn paint_home_glyph(p: &egui::Painter, center: Pos2, color: Color32) {
    icons::paint(p, Icon::Home, center, 14.0, color);
}

fn truncate_tab_title(p: &egui::Painter, text: &str, font: &egui::FontId, max_w: f32) -> String {
    if widgets::text_width(p, text, font) <= max_w {
        return text.to_string();
    }
    let mut s = String::new();
    for ch in text.chars() {
        let cand = format!("{s}{ch}\u{2026}");
        if widgets::text_width(p, &cand, font) > max_w {
            break;
        }
        s.push(ch);
    }
    if s.is_empty() {
        "\u{2026}".to_string()
    } else {
        format!("{s}\u{2026}")
    }
}

#[allow(dead_code)]
impl App {
    pub fn open_home(&mut self) {
        let mut tabs = std::mem::take(&mut self.tabs);
        tabs.enter_home(self);
        self.tabs = tabs;
    }

    pub fn switch_tab(&mut self, idx: usize) {
        let mut tabs = std::mem::take(&mut self.tabs);
        tabs.switch_to(idx, self);
        self.tabs = tabs;
    }

    pub fn open_new_tab(&mut self, title: &str, doc: Document, kind: ProjectTabKind) -> usize {
        let mut tabs = std::mem::take(&mut self.tabs);
        let idx = tabs.open_new_tab(self, title, doc, kind);
        self.tabs = tabs;
        idx
    }

    pub fn open_new_design_tab(&mut self) -> usize {
        let mut tabs = std::mem::take(&mut self.tabs);
        let idx = tabs.open_new_design_tab(self);
        self.tabs = tabs;
        idx
    }

    pub fn open_new_board_tab(&mut self) -> usize {
        let mut tabs = std::mem::take(&mut self.tabs);
        let idx = tabs.open_new_board_tab(self);
        self.tabs = tabs;
        idx
    }

    pub fn close_tab(&mut self, idx: usize) -> bool {
        let mut tabs = std::mem::take(&mut self.tabs);
        let ok = tabs.close_tab(idx, self);
        self.tabs = tabs;
        ok
    }

    pub fn duplicate_tab(&mut self, idx: usize) -> Option<usize> {
        let mut tabs = std::mem::take(&mut self.tabs);
        let res = tabs.duplicate_tab(idx, self);
        self.tabs = tabs;
        res
    }

    pub fn next_tab(&mut self) {
        let mut tabs = std::mem::take(&mut self.tabs);
        tabs.next_tab(self);
        self.tabs = tabs;
    }

    pub fn prev_tab(&mut self) {
        let mut tabs = std::mem::take(&mut self.tabs);
        tabs.prev_tab(self);
        self.tabs = tabs;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use designos_doc::{Node, Rect as DRect, Vec2 as DVec2};

    fn make_app_with_doc(name: &str) -> App {
        let mut doc = Document::new(name);
        doc.add_page("Page 1");
        let mut app = App::new(doc);
        let mut tabs = std::mem::take(&mut app.tabs);
        tabs.sync_from_app(&app);
        app.tabs = tabs;
        app
    }

    #[test]
    fn multi_tab_state_isolation_preserves_doc_selection_zoom_page_and_history() {
        let mut app = make_app_with_doc("Tab 1");
        let p1_b = app.doc.add_page("Page 2");
        app.page = p1_b;
        let r1_id = app.doc.alloc_id();
        let r1 = app
            .doc
            .insert(p1_b, Node::rectangle(r1_id, DRect::new(10.0, 20.0, 120.0, 80.0)))
            .unwrap();
        // Record an undoable edit in Tab 1
        app.rename(r1, "HeroRectTab1");
        app.selection = vec![r1];
        app.view = View {
            offset: DVec2::new(42.0, 84.0),
            zoom: 2.25,
        };
        assert!(app.hist.can_undo(), "Tab 1 has undo history");

        // Open Tab 2 and Tab 3
        let mut doc2 = Document::new("Tab 2");
        let p2 = doc2.add_page("Board Page");
        let r2_id = doc2.alloc_id();
        let r2 = doc2
            .insert(p2, Node::rectangle(r2_id, DRect::new(0.0, 0.0, 60.0, 60.0)))
            .unwrap();
        let idx2 = app.open_new_tab("Tab 2", doc2, ProjectTabKind::Board);
        assert_eq!(idx2, 1);
        assert_eq!(app.doc.name, "Tab 2");
        assert!(!app.hist.can_undo(), "Tab 2 starts with clean history");
        app.rename(r2, "BoardNodeTab2");
        app.selection = vec![r2];
        app.view = View {
            offset: DVec2::new(-120.0, 30.0),
            zoom: 0.75,
        };

        let doc3 = Document::new("Tab 3");
        let idx3 = app.open_new_tab("Tab 3", doc3, ProjectTabKind::Design);
        assert_eq!(idx3, 2);
        assert_eq!(app.tabs.entries.len(), 3);
        assert_eq!(app.doc.name, "Tab 3");
        assert!(app.selection.is_empty());

        // Switch back to Tab 1 and verify 100% state isolation
        app.switch_tab(0);
        assert_eq!(app.doc.name, "Tab 1");
        assert_eq!(app.doc.get(r1).unwrap().name, "HeroRectTab1");
        assert_eq!(app.page, p1_b);
        assert_eq!(app.selection, vec![r1]);
        assert!((app.view.zoom - 2.25).abs() < 1e-5);
        assert_eq!(app.view.offset, DVec2::new(42.0, 84.0));
        assert!(app.hist.can_undo(), "Tab 1 undo stack restored");

        // Switch to Tab 2 and verify its edits and view were preserved
        app.switch_tab(1);
        assert_eq!(app.doc.name, "Tab 2");
        assert_eq!(app.doc.get(r2).unwrap().name, "BoardNodeTab2");
        assert_eq!(app.selection, vec![r2]);
        assert!((app.view.zoom - 0.75).abs() < 1e-5);
        assert!(app.hist.can_undo(), "Tab 2 undo stack restored");
    }

    #[test]
    fn switching_to_home_and_back_preserves_open_tabs() {
        let mut app = make_app_with_doc("Design Alpha");
        app.open_new_board_tab();
        assert_eq!(app.tabs.entries.len(), 2);
        assert!(app.home.is_none());

        // Enter Home without closing any open project tabs
        app.open_home();
        assert!(app.home.is_some());
        assert_eq!(app.tabs.entries.len(), 2, "both project tabs remain open while on Home");

        // Switch from Home back into Tab 0
        app.switch_tab(0);
        assert!(app.home.is_none(), "switching to an open tab exits Home");
        assert_eq!(app.doc.name, "Design Alpha");
        assert_eq!(app.tabs.entries.len(), 2);
    }

    #[test]
    fn duplicate_pin_unpin_and_close_tab_lifecycle() {
        let mut app = make_app_with_doc("Main System");
        let dup_idx = app.duplicate_tab(0).expect("duplicated tab");
        assert_eq!(dup_idx, 1);
        assert_eq!(app.tabs.entries.len(), 2);
        assert_eq!(app.doc.name, "Main System (Copy)");

        // Pin Tab 0 and verify it cannot be closed while pinned
        app.tabs.toggle_pin(0);
        assert!(app.tabs.entries[0].pinned);
        assert!(!app.close_tab(0), "pinned tab refuses close");
        assert_eq!(app.tabs.entries.len(), 2);

        // Close Tab 1 (unpinned) -> activates Tab 0
        assert!(app.close_tab(1));
        assert_eq!(app.tabs.entries.len(), 1);
        assert_eq!(app.tabs.active_idx, 0);
        assert_eq!(app.doc.name, "Main System");

        // Unpin Tab 0 and close it -> opens Home when last tab closes
        app.tabs.toggle_pin(0);
        assert!(!app.tabs.entries[0].pinned);
        assert!(app.close_tab(0));
        assert!(app.tabs.entries.is_empty());
        assert!(app.home.is_some(), "closing the last tab returns to Home");
    }
}
