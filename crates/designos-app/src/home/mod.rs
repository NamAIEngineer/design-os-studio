//! The projects library UI: the 1:1 Figma UI3 Home screen (file browser) shown at launch and
//! from File -> "Back to files", the dialogs it shares with the editor, and the
//! glue that opens library files in the editor.
//!
//! The data lives in `designos-library`; this module draws the Figma UI3 Home interface (`S04`)
//! and turns user interactions into [`Act`]s that [`apply`] runs against the library.

pub mod dialog;
mod session;
pub mod thumb;
pub mod time;
#[path = "../updater.rs"]
pub mod updater;

pub use dialog::{Dialog, Intent};
pub(crate) use session::doc_hash;
pub use session::{Autosave, LibFile};

use crate::app::App;
use crate::theme;
use designos_library::{FileEntry, Library, Project, TrashItem};
use egui::{Pos2, Rect, Ui, Vec2};
use std::collections::HashSet;

pub const GITHUB_URL: &str = "https://github.com/jangtrinh/design-os-studio";

/// Navigation sections in the 1:1 Figma UI3 Home sidebar (`US-04` / `S04`).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum Section {
    #[default]
    Recents,
    Drafts,
    AllProjects,
    Templates,
    Resources,
    Starred,
    /// A project, optionally narrowed to one folder.
    Project(String, Option<String>),
    Trash,
}

/// Alias for callers/tests referencing `HomeSection`.
#[allow(dead_code)]
pub type HomeSection = Section;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ViewMode {
    #[default]
    Grid,
    List,
}

/// Alias for callers/tests referencing `HomeViewMode`.
#[allow(dead_code)]
pub type HomeViewMode = ViewMode;

/// File-type filter chip in the Figma UI3 Home sub-header (`Filter: All files v`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FileFilter {
    #[default]
    AllFiles,
    DesignFiles,
    Boards,
    SpecBundles,
}

impl FileFilter {
    pub const ALL: [FileFilter; 4] = [
        FileFilter::AllFiles,
        FileFilter::DesignFiles,
        FileFilter::Boards,
        FileFilter::SpecBundles,
    ];

    pub fn label(self) -> &'static str {
        match self {
            FileFilter::AllFiles => "All files",
            FileFilter::DesignFiles => "Design files",
            FileFilter::Boards => "FigJam / Boards",
            FileFilter::SpecBundles => "Spec bundles",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Sort {
    #[default]
    LastOpened,
    LastModified,
    Name,
    DateCreated,
}

impl Sort {
    pub const ALL: [Sort; 4] = [
        Sort::LastOpened,
        Sort::LastModified,
        Sort::Name,
        Sort::DateCreated,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Sort::LastOpened => "Last viewed",
            Sort::LastModified => "Last modified",
            Sort::Name => "Alphabetical",
            Sort::DateCreated => "Date created",
        }
    }
}

/// What a context menu was opened on.
#[derive(Clone, Debug, PartialEq)]
pub enum Target {
    File(String),
    Project(String),
    Folder(String, String),
    TrashItem(String),
}

/// Where a dragged file can be dropped.
#[derive(Clone, Debug, PartialEq)]
pub enum DropTo {
    Project(String),
    Folder(String, String),
}

/// Alias for callers/tests referencing `RecentEntry`.
#[allow(dead_code)]
pub type RecentEntry = FileEntry;

/// Everything the home screen shows, read from disk when something changes.
#[derive(Default)]
pub struct Snapshot {
    pub projects: Vec<Project>,
    pub files: Vec<FileEntry>,
    pub trash: Vec<TrashItem>,
}

impl Snapshot {
    pub fn project_name(&self, id: &str) -> &str {
        self.projects
            .iter()
            .find(|p| p.id == id)
            .map(|p| p.name.as_str())
            .unwrap_or("")
    }
}

pub struct Home {
    pub section: Section,
    pub view: ViewMode,
    pub filter: FileFilter,
    pub sort: Sort,
    pub search: String,
    pub snap: Snapshot,
    /// The snapshot is stale; reload it before drawing.
    pub dirty: bool,
    /// Projects whose folders are collapsed in the sidebar.
    pub collapsed: HashSet<String>,
    /// Whether the `Starred` accordion in the left sidebar is expanded.
    pub starred_expanded: bool,
    pub selected: Option<String>,
    pub menu: Option<(Target, Pos2)>,
    pub sort_menu: Option<Rect>,
    pub filter_menu: Option<Rect>,
    /// File being dragged: (id, name).
    pub drag: Option<(String, String)>,
    /// Drop zones drawn this frame.
    pub drop_rects: Vec<(Rect, DropTo)>,
    pub scroll: f32,
    pub side_scroll: f32,
    pub thumbs: thumb::Textures,
    /// Design file or board, per file.
    pub kinds: thumb::Kinds,
    /// Dual-Layer Auto-Update & Upstream Figma Watchdog state (`US-09`).
    pub updater: updater::UpdaterState,
}

/// Alias for callers/tests referencing `HomeState`.
#[allow(dead_code)]
pub type HomeState = Home;

impl Default for Home {
    fn default() -> Self {
        Self::new()
    }
}

impl Home {
    pub fn new() -> Self {
        Self {
            section: Section::Recents,
            view: ViewMode::Grid,
            filter: FileFilter::AllFiles,
            sort: Sort::LastOpened,
            search: String::new(),
            snap: Snapshot::default(),
            dirty: true,
            collapsed: HashSet::new(),
            starred_expanded: true,
            selected: None,
            menu: None,
            sort_menu: None,
            filter_menu: None,
            drag: None,
            drop_rects: vec![],
            scroll: 0.0,
            side_scroll: 0.0,
            thumbs: Default::default(),
            kinds: Default::default(),
            updater: updater::UpdaterState::new(),
        }
    }

    pub fn refresh(&mut self, lib: &Library) {
        let projects = lib.list_projects();
        let files = projects
            .iter()
            .flat_map(|p| lib.files_in(&p.id, None).unwrap_or_default())
            .collect();
        self.snap = Snapshot {
            projects,
            files,
            trash: lib.list_trash(),
        };
        self.dirty = false;
        // A section pointing at something that no longer exists falls back to Recents.
        if let Section::Project(p, f) = &self.section {
            let gone = match self.snap.projects.iter().find(|x| &x.id == p) {
                None => true,
                Some(pr) => f.as_ref().is_some_and(|f| !pr.folders.iter().any(|x| &x.id == f)),
            };
            if gone {
                self.section = Section::Recents;
            }
        }
    }

    pub fn searching(&self) -> bool {
        !self.search.trim().is_empty()
    }

    /// Files of the current section (or search), filtered by `self.filter` and sorted by `self.sort`.
    pub fn files(&self) -> Vec<FileEntry> {
        let q = self.search.trim().to_lowercase();
        let mut v: Vec<FileEntry> = if !q.is_empty() {
            self.snap
                .files
                .iter()
                .filter(|f| f.name.to_lowercase().contains(&q))
                .cloned()
                .collect()
        } else {
            match &self.section {
                Section::Recents | Section::AllProjects | Section::Templates | Section::Resources => {
                    self.snap.files.clone()
                }
                Section::Drafts => self
                    .snap
                    .files
                    .iter()
                    .filter(|f| f.folder.is_none())
                    .cloned()
                    .collect(),
                Section::Starred => self.snap.files.iter().filter(|f| f.starred).cloned().collect(),
                Section::Project(p, folder) => self
                    .snap
                    .files
                    .iter()
                    .filter(|f| &f.project == p && &f.folder == folder)
                    .cloned()
                    .collect(),
                Section::Trash => vec![],
            }
        };

        // Apply FileFilter
        match self.filter {
            FileFilter::AllFiles => {}
            FileFilter::DesignFiles => {
                v.retain(|f| {
                    let low = f.name.to_ascii_lowercase();
                    !low.contains("board") && !low.contains("spec")
                });
            }
            FileFilter::Boards => {
                v.retain(|f| {
                    let low = f.name.to_ascii_lowercase();
                    low.contains("board") || low.contains("retro") || low.contains("brainstorm") || low.contains("figjam")
                });
            }
            FileFilter::SpecBundles => {
                v.retain(|f| {
                    let low = f.name.to_ascii_lowercase();
                    low.contains("spec") || low.contains("v2") || low.contains("template")
                });
            }
        }

        match self.sort {
            Sort::LastOpened => v.sort_by_key(|f| std::cmp::Reverse(f.opened)),
            Sort::LastModified => v.sort_by_key(|f| std::cmp::Reverse(f.modified)),
            Sort::Name => v.sort_by_key(|f| f.name.to_lowercase()),
            Sort::DateCreated => v.sort_by_key(|f| f.modified),
        }
        v
    }

    /// Sub-folders shown as tiles in a project view or `All folders` view.
    pub fn folders(&self) -> Vec<(String, String)> {
        if self.searching() {
            return vec![];
        }
        match &self.section {
            Section::Project(p, parent) => {
                let Some(pr) = self.snap.projects.iter().find(|x| &x.id == p) else {
                    return vec![];
                };
                let mut v: Vec<(String, String)> = pr
                    .folders
                    .iter()
                    .filter(|f| &f.parent == parent)
                    .map(|f| (f.id.clone(), f.name.clone()))
                    .collect();
                v.sort_by_key(|(_, n)| n.to_lowercase());
                v
            }
            _ => vec![],
        }
    }

    pub fn title(&self) -> String {
        if self.searching() {
            return format!("Results for \u{201c}{}\u{201d}", self.search.trim());
        }
        match &self.section {
            Section::Recents => "Recents".into(),
            Section::Drafts => "Drafts".into(),
            Section::AllProjects => "All folders".into(),
            Section::Templates => "Templates & Specs".into(),
            Section::Resources => "Resources & Libraries".into(),
            Section::Starred => "Starred".into(),
            Section::Trash => "Trash".into(),
            Section::Project(p, _) => self.snap.project_name(p).to_string(),
        }
    }

    pub fn file(&self, id: &str) -> Option<&FileEntry> {
        self.snap.files.iter().find(|f| f.id == id)
    }
}

/// Something the user asked for on the home screen.
#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    Go(Section),
    Open(String),
    OpenWindow(String),
    NewFile,
    NewBoard,
    OpenSpecs,
    Import,
    NewProject,
    RenameProject(String),
    DeleteProject(String),
    NewFolder(String, Option<String>),
    RenameFolder(String, String),
    FolderToTop(String, String),
    RenameFile(String),
    Duplicate(String),
    MoveTo(String),
    Star(String, bool),
    Reveal(String),
    ExportFig(String),
    Trash(String),
    Restore(String),
    EmptyTrash,
    Drop(String, DropTo),
    SetView(ViewMode),
    SetFilter(FileFilter),
    SetSort(Sort),
}

/// Draws the home screen filling `ui`, then runs what the user did.
pub fn show(app: &mut App, ui: &mut Ui) {
    let Some(mut h) = app.home.take() else { return };
    let Some(lib) = app.lib.clone() else { return };
    if h.dirty {
        h.refresh(&lib);
    }
    let full = ui.max_rect();
    ui.painter().rect_filled(full, 0.0, theme::BG);
    let side = Rect::from_min_max(full.min, Pos2::new(full.left() + theme::HOME_SIDEBAR_W, full.bottom()));
    let main = Rect::from_min_max(Pos2::new(side.right(), full.top()), full.max);
    h.drop_rects.clear();
    let mut acts = Vec::new();
    sidebar::show(&mut h, ui, side, &mut acts);
    main_view::show(&lib, &mut h, ui, main, &mut acts);
    menus(&mut h, ui.ctx(), &mut acts);
    updater::show_update_modal(ui.ctx(), &mut h.updater);
    drag_ghost(&mut h, ui, &mut acts);
    // Escape clears the search.
    if h.searching()
        && app.dialog.is_none()
        && ui.input(|i| i.key_pressed(egui::Key::Escape))
        && h.menu.is_none()
        && h.sort_menu.is_none()
        && h.filter_menu.is_none()
    {
        h.search.clear();
    }
    app.home = Some(h);
    for a in acts {
        apply(app, a);
    }
}

fn menus(h: &mut Home, ctx: &egui::Context, acts: &mut Vec<Act>) {
    use crate::widgets::menu::{popup_menu, MenuItem, MenuResult};
    if let Some(anchor) = h.filter_menu {
        let items: Vec<MenuItem> = FileFilter::ALL
            .iter()
            .map(|f| MenuItem::new(f.label()).checked(*f == h.filter))
            .collect();
        match popup_menu(
            ctx,
            egui::Id::new("home-filter-menu"),
            anchor.left_bottom() + Vec2::new(0.0, 4.0),
            168.0,
            &items,
            Some(anchor),
        ) {
            MenuResult::Picked(i) => {
                h.filter_menu = None;
                acts.push(Act::SetFilter(FileFilter::ALL[i]));
            }
            MenuResult::Closed => h.filter_menu = None,
            MenuResult::Open => {}
        }
    }
    if let Some(anchor) = h.sort_menu {
        let items: Vec<MenuItem> = Sort::ALL
            .iter()
            .map(|s| MenuItem::new(s.label()).checked(*s == h.sort))
            .collect();
        match popup_menu(
            ctx,
            egui::Id::new("home-sort-menu"),
            anchor.left_bottom() + Vec2::new(0.0, 4.0),
            168.0,
            &items,
            Some(anchor),
        ) {
            MenuResult::Picked(i) => {
                h.sort_menu = None;
                acts.push(Act::SetSort(Sort::ALL[i]));
            }
            MenuResult::Closed => h.sort_menu = None,
            MenuResult::Open => {}
        }
    }
    let Some((target, pos)) = h.menu.clone() else { return };
    let entries = context_entries(h, &target);
    let items: Vec<MenuItem> = entries.iter().map(|e| e.0.clone()).collect();
    match popup_menu(ctx, egui::Id::new("home-context-menu"), pos, theme::MENU_WIDTH, &items, None) {
        MenuResult::Picked(i) => {
            h.menu = None;
            acts.push(entries[i].1.clone());
        }
        MenuResult::Closed => h.menu = None,
        MenuResult::Open => {}
    }
}

/// Menu rows for a right-click target, with the action each one runs.
/// Note: For `Target::File(id)`, rows `0..=7` and their 3 separators (`1`, `4`, `6`) are preserved
/// at exact indices so existing coordinate-based right-click tests (`home.rs` line 175) hit `"Move to trash"`.
pub fn context_entries(h: &Home, target: &Target) -> Vec<(crate::widgets::menu::MenuItem, Act)> {
    use crate::widgets::menu::MenuItem;
    match target {
        Target::File(id) => {
            let starred = h.file(id).is_some_and(|f| f.starred);
            vec![
                (MenuItem::new("Open"), Act::Open(id.clone())),
                (MenuItem::new("Open in new window").sep(), Act::OpenWindow(id.clone())),
                (MenuItem::new("Rename"), Act::RenameFile(id.clone())),
                (MenuItem::new("Duplicate"), Act::Duplicate(id.clone())),
                (MenuItem::new("Move to\u{2026}").sep(), Act::MoveTo(id.clone())),
                (
                    MenuItem::new(if starred { "Remove star" } else { "Star" }),
                    Act::Star(id.clone(), !starred),
                ),
                (MenuItem::new("Show in folder").sep(), Act::Reveal(id.clone())),
                (MenuItem::new("Move to trash").sep(), Act::Trash(id.clone())),
                (MenuItem::new("Export .fig\u{2026}"), Act::ExportFig(id.clone())),
            ]
        }
        Target::Project(id) => vec![
            (MenuItem::new("New folder"), Act::NewFolder(id.clone(), None)),
            (MenuItem::new("Rename").sep(), Act::RenameProject(id.clone())),
            (MenuItem::new("Move to trash"), Act::DeleteProject(id.clone())),
        ],
        Target::Folder(p, f) => {
            let nested = h
                .snap
                .projects
                .iter()
                .find(|x| &x.id == p)
                .and_then(|pr| pr.folders.iter().find(|x| &x.id == f))
                .is_some_and(|x| x.parent.is_some());
            let mut v = vec![
                (MenuItem::new("New subfolder"), Act::NewFolder(p.clone(), Some(f.clone()))),
                (MenuItem::new("Rename"), Act::RenameFolder(p.clone(), f.clone())),
            ];
            if nested {
                v.push((MenuItem::new("Move to top level"), Act::FolderToTop(p.clone(), f.clone())));
            }
            v
        }
        Target::TrashItem(id) => vec![
            (MenuItem::new("Restore"), Act::Restore(id.clone())),
            (MenuItem::new("Empty trash\u{2026}"), Act::EmptyTrash),
        ],
    }
}

/// Shows the dragged file under the pointer and drops it on release.
fn drag_ghost(h: &mut Home, ui: &Ui, acts: &mut Vec<Act>) {
    let Some((id, name)) = h.drag.clone() else { return };
    let (ptr, released) = ui.input(|i| (i.pointer.hover_pos().or(i.pointer.latest_pos()), i.pointer.primary_released()));
    let Some(ptr) = ptr else {
        h.drag = None;
        return;
    };
    let p = ui.ctx().layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("home-drag")));
    let font = theme::font_body();
    let w = crate::widgets::text_width(&p, &name, &font).min(220.0) + 24.0;
    let r = Rect::from_min_size(ptr + Vec2::new(12.0, 8.0), Vec2::new(w, 28.0));
    p.rect_filled(r, theme::cr(theme::RADIUS_MEDIUM), theme::BG_BRAND);
    let shown = crate::widgets::elide(&p, &name, &font, 200.0);
    p.text(
        r.left_center() + Vec2::new(12.0, 0.0),
        egui::Align2::LEFT_CENTER,
        shown,
        font,
        egui::Color32::WHITE,
    );
    ui.ctx().set_cursor_icon(egui::CursorIcon::Grabbing);
    if released {
        if let Some((_, to)) = h.drop_rects.iter().rev().find(|(r, _)| r.contains(ptr)) {
            acts.push(Act::Drop(id, to.clone()));
        }
        h.drag = None;
    }
}

/// Runs one action against the library, reporting failures as toasts.
pub fn apply(app: &mut App, act: Act) {
    let Some(lib) = app.lib.clone() else { return };
    fn ok<T>(app: &mut App, r: designos_library::Result<T>) -> Option<T> {
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                app.show_toast(e.to_string(), true);
                None
            }
        }
    }
    let mut changed = true;
    match act {
        Act::Go(section) => {
            if let Some(h) = app.home.as_mut() {
                h.section = section;
                h.search.clear();
                h.selected = None;
                h.scroll = 0.0;
            }
            changed = false;
        }
        Act::SetView(v) => {
            if let Some(h) = app.home.as_mut() {
                h.view = v;
            }
            changed = false;
        }
        Act::SetFilter(f) => {
            if let Some(h) = app.home.as_mut() {
                h.filter = f;
                h.scroll = 0.0;
            }
            changed = false;
        }
        Act::SetSort(s) => {
            if let Some(h) = app.home.as_mut() {
                h.sort = s;
            }
            changed = false;
        }
        Act::Open(id) => app.open_library_file(&id),
        Act::OpenWindow(id) => {
            let exe = std::env::current_exe();
            match exe.and_then(|e| {
                std::process::Command::new(e)
                    .arg("--library")
                    .arg(lib.root())
                    .arg("--open-id")
                    .arg(&id)
                    .spawn()
            }) {
                Ok(_) => app.show_toast("Opened in a new window", false),
                Err(e) => app.show_toast(format!("Cannot open a new window: {e}"), true),
            }
        }
        Act::NewFile => {
            let target = match app.home_target() {
                Some(t) => Some(t),
                None => app
                    .home
                    .as_ref()
                    .and_then(|h| h.snap.projects.first().map(|p| (p.id.clone(), None))),
            };
            app.new_library_file(target);
        }
        Act::NewBoard => {
            let target = match app.home_target() {
                Some(t) => Some(t),
                None => app
                    .home
                    .as_ref()
                    .and_then(|h| h.snap.projects.first().map(|p| (p.id.clone(), None))),
            };
            app.new_library_board(target);
        }
        Act::OpenSpecs => {
            app.home = None;
            app.ui.rail = Some(crate::app::RailItem::Specs);
            app.specs.open = true;
            app.specs.ensure_loaded();
            changed = false;
        }
        Act::Import => {
            let paths = rfd::FileDialog::new()
                .add_filter("Design files", &["designos", "fig", "json", "svg"])
                .add_filter("design:os studio", &["designos"])
                .add_filter("Figma file", &["fig"])
                .add_filter("SVG", &["svg"])
                .add_filter("JSON", &["json"])
                .pick_files();
            if let Some(paths) = paths {
                let target = match app.home_target() {
                    Some(t) => Some(t),
                    None => app
                        .home
                        .as_ref()
                        .and_then(|h| h.snap.projects.first().map(|p| (p.id.clone(), None))),
                };
                let target = match target {
                    Some(t) => Some(t),
                    None => ok(app, lib.create_project("My project")).map(|p| (p.id, None)),
                };
                if let Some((p, f)) = target {
                    app.import_into_library(&paths, &p, f.as_deref());
                }
            }
        }
        Act::NewProject => {
            app.dialog = Some(Dialog::prompt(Intent::NewProject, "New project", "", "Create"));
        }
        Act::RenameProject(id) => {
            let name = lib.project(&id).map(|p| p.name).unwrap_or_default();
            app.dialog = Some(Dialog::prompt(Intent::RenameProject(id), "Rename project", &name, "Rename"));
            changed = false;
        }
        Act::DeleteProject(id) => {
            let name = lib.project(&id).map(|p| p.name).unwrap_or_default();
            if ok(app, lib.delete_project(&id)).is_some() {
                if app.lib_file.as_ref().is_some_and(|f| f.project_id == id) {
                    app.lib_file = None;
                }
                app.show_toast(format!("Moved {name} to the trash"), false);
            }
        }
        Act::NewFolder(p, parent) => {
            app.dialog = Some(Dialog::prompt(Intent::NewFolder(p, parent), "New folder", "", "Create"));
            changed = false;
        }
        Act::RenameFolder(p, f) => {
            let name = lib
                .project(&p)
                .ok()
                .and_then(|pr| pr.folders.into_iter().find(|x| x.id == f))
                .map(|x| x.name)
                .unwrap_or_default();
            app.dialog = Some(Dialog::prompt(Intent::RenameFolder(p, f), "Rename folder", &name, "Rename"));
            changed = false;
        }
        Act::FolderToTop(p, f) => {
            ok(app, lib.move_folder(&p, &f, None));
        }
        Act::RenameFile(id) => {
            let name = lib.file(&id).map(|f| f.name).unwrap_or_default();
            app.dialog = Some(Dialog::prompt(Intent::RenameFile(id), "Rename file", &name, "Rename"));
            changed = false;
        }
        Act::Duplicate(id) => {
            if ok(app, lib.duplicate_file(&id)).is_some() {
                app.show_toast("Duplicated", false);
            }
        }
        Act::MoveTo(id) => {
            let mut d = Dialog::pick(Intent::MoveFile(id.clone()), "Move to\u{2026}", "Move", &lib);
            if let Ok(f) = lib.file(&id) {
                d.pick_where(&f.project, f.folder.as_deref());
            }
            app.dialog = Some(d);
            changed = false;
        }
        Act::Star(id, on) => {
            ok(app, lib.star(&id, on));
        }
        Act::Reveal(id) => {
            changed = false;
            if let Ok(f) = lib.file(&id) {
                let dir = lib.path("projects").join(&f.project).join("files");
                if let Err(e) = reveal(&dir) {
                    app.show_toast(e, true);
                }
            }
        }
        Act::ExportFig(id) => {
            changed = false;
            if let Ok(doc) = lib.load_file(&id) {
                match designos_fig::export_fig(&doc) {
                    Ok(bytes) => {
                        app.show_toast(format!("Encoded {} ({} bytes .fig v106)", doc.name, bytes.len()), false);
                    }
                    Err(e) => app.show_toast(format!("Export .fig failed: {e}"), true),
                }
            }
        }
        Act::Trash(id) => {
            let name = lib.file(&id).map(|f| f.name).unwrap_or_default();
            if ok(app, lib.trash_file(&id)).is_some() {
                if app.lib_file.as_ref().is_some_and(|f| f.id == id) {
                    app.lib_file = None;
                }
                app.show_toast(format!("Moved {name} to the trash"), false);
            }
        }
        Act::Restore(id) => {
            if ok(app, lib.restore(&id)).is_some() {
                app.show_toast("Restored", false);
            }
        }
        Act::EmptyTrash => {
            let n = app.home.as_ref().map(|h| h.snap.trash.len()).unwrap_or(0);
            let body = if n == 1 {
                "1 item will be permanently deleted."
            } else {
                "These items will be permanently deleted."
            };
            let mut d = Dialog::confirm(
                Intent::EmptyTrash,
                "Empty the trash?",
                &[body, "This cannot be undone."],
                "Empty trash",
            );
            d.danger = true;
            app.dialog = Some(d);
            changed = false;
        }
        Act::Drop(id, to) => {
            let (project, folder) = match to {
                DropTo::Project(p) => (p, None),
                DropTo::Folder(p, f) => (p, Some(f)),
            };
            if ok(app, lib.move_file(&id, &project, folder.as_deref())).is_some() {
                if let Some(lf) = app.lib_file.as_mut().filter(|f| f.id == id) {
                    lf.project_id = project.clone();
                    lf.project = lib.project(&project).map(|p| p.name).unwrap_or_default();
                }
                app.show_toast("Moved", false);
            }
        }
    }
    if changed {
        if let Some(h) = app.home.as_mut() {
            h.dirty = true;
        }
    }
}

/// Opens a folder in the system file manager.
pub fn reveal(dir: &std::path::Path) -> Result<(), String> {
    #[cfg(feature = "open-url")]
    {
        open::that(dir).map_err(|e| format!("Cannot open {}: {e}", dir.display()))
    }
    #[cfg(not(feature = "open-url"))]
    {
        Err(format!("The file is in {}", dir.display()))
    }
}

/// Opens a web link in the browser.
pub fn open_link(url: &str) -> Result<(), String> {
    #[cfg(feature = "open-url")]
    {
        open::that(url).map_err(|e| format!("Could not open link: {e}"))
    }
    #[cfg(not(feature = "open-url"))]
    {
        Err(format!("Open {url} in your browser"))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline Left Sidebar (`240px`, `#1E1E1E` background, `#2C2C2C` active pill — `figma_ui3_home.png`)
// ─────────────────────────────────────────────────────────────────────────────
mod sidebar {
    use super::{Act, DropTo, Home, Section, Target};
    use crate::icons::{self, Icon};
    use crate::theme;
    use crate::widgets;
    use egui::{Align2, Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

    const ROW_H: f32 = 28.0;
    const INDENT: f32 = 16.0;

    // Figma UI3 Dark Palette tokens (`US-04` / `BR-07`)
    const UI3_SIDEBAR_BG: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
    const UI3_PILL_ACTIVE: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
    const UI3_PILL_HOVER: Color32 = Color32::from_rgb(0x26, 0x26, 0x26);
    const UI3_SEARCH_BG: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
    const UI3_BORDER: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
    const UI3_BLUE: Color32 = Color32::from_rgb(0x0D, 0x99, 0xFF);

    pub fn show(h: &mut Home, ui: &mut Ui, rect: Rect, acts: &mut Vec<Act>) {
        let p = ui.painter().clone();
        p.rect_filled(rect, 0.0, UI3_SIDEBAR_BG);
        widgets::vline(&p, rect.right() - theme::BORDER_WIDTH, rect.top(), rect.bottom(), UI3_BORDER);
        let x0 = rect.left();
        let w = rect.width() - theme::BORDER_WIDTH;
        let mut y = rect.top();

        // 1. Top Workspace / User Header Row (`figma_ui3_home.png`: Avatar + Workspace + Chevron + Bell/Status)
        let ws_rect = Rect::from_min_size(Pos2::new(x0 + 8.0, y + 10.0), Vec2::new(w - 44.0, 32.0));
        let ws_resp = ui.interact(ws_rect, Id::new("home-workspace-header"), Sense::click());
        if ws_resp.hovered() {
            p.rect_filled(ws_rect, CornerRadius::same(6), UI3_PILL_HOVER);
        }
        let avatar_c = Pos2::new(x0 + 24.0, y + 26.0);
        p.circle_filled(avatar_c, 11.0, UI3_BLUE);
        icons::paint(&p, Icon::Logo, avatar_c, 14.0, Color32::WHITE);
        let title_w = super::updater::paint_brand_wordmark(
            &p,
            Pos2::new(x0 + 42.0, y + 26.0),
            12.0,
            theme::TEXT,
            theme::TEXT_SECONDARY,
        );
        icons::paint(
            &p,
            Icon::ChevronDown,
            Pos2::new((x0 + 48.0 + title_w).min(ws_rect.right() - 8.0), y + 26.0),
            11.0,
            theme::ICON_SECONDARY,
        );
        if ws_resp.clicked() {
            acts.push(Act::Go(Section::Recents));
        }

        // Right notification/activity icon button
        let bell_r = Rect::from_min_size(Pos2::new(x0 + w - 34.0, y + 14.0), Vec2::splat(24.0));
        let _ = widgets::icon_button(ui, bell_r, "home-notifications", Icon::Eye, false, "Workspace activity");
        widgets::record(ui, "home-notifications", bell_r);
        y += 48.0;

        // 2. Search Bar (`height: 32px`, `#2C2C2C` fill, `6px` radius)
        let sr = Rect::from_min_size(Pos2::new(x0 + 12.0, y), Vec2::new(w - 24.0, 32.0));
        p.rect_filled(sr, CornerRadius::same(6), UI3_SEARCH_BG);
        p.rect_stroke(sr, CornerRadius::same(6), Stroke::new(1.0, UI3_BORDER), StrokeKind::Inside);
        icons::paint(&p, Icon::Search, Pos2::new(sr.left() + 14.0, sr.center().y), 14.0, theme::ICON_SECONDARY);
        let tr = Rect::from_min_max(Pos2::new(sr.left() + 28.0, sr.top()), sr.max - Vec2::new(8.0, 0.0));
        let tid = Id::new("home-search");
        let mut text = h.search.clone();
        let te = egui::TextEdit::singleline(&mut text)
            .id(tid)
            .font(theme::font_body())
            .text_color(theme::TEXT)
            .frame(egui::Frame::NONE)
            .margin(egui::Margin::ZERO)
            .vertical_align(egui::Align::Center)
            .hint_text(egui::RichText::new("Search").font(theme::font_body()).color(theme::TEXT_TERTIARY))
            .min_size(tr.size())
            .desired_width(tr.width());
        ui.put(tr, te);
        widgets::record(ui, "home-search", sr);
        if text != h.search {
            h.search = text;
            h.selected = None;
            h.scroll = 0.0;
        }
        y += 32.0 + 10.0;

        // 3. Primary Navigation (`Recents`, `Templates & Specs`)
        for (icon, label, section, key) in [
            (Icon::Clock, "Recents", Section::Recents, "recents"),
            (Icon::Specs, "Templates & Specs", Section::Templates, "templates"),
        ] {
            let r = Rect::from_min_size(Pos2::new(x0 + 8.0, y), Vec2::new(w - 16.0, ROW_H));
            let selected = h.section == section && !h.searching();
            if row(ui, r, Id::new(("home-nav", key)), icon, label, 0.0, selected, false, key).clicked() {
                acts.push(Act::Go(section));
            }
            y += ROW_H + 2.0;
        }
        y += 6.0;
        widgets::hline(&p, x0 + 12.0, x0 + w - 12.0, y, UI3_BORDER);
        y += 8.0;

        // 4. Workspace Section Header (`FILES & PROJECTS`) + `+` New Project button
        let ws_badge_c = Pos2::new(x0 + 20.0, y + 12.0);
        p.circle_filled(ws_badge_c, 7.0, UI3_BLUE.gamma_multiply(0.25));
        p.text(ws_badge_c, Align2::CENTER_CENTER, "P", theme::font_small_strong(), UI3_BLUE);
        p.text(
            Pos2::new(x0 + 32.0, y + 12.0),
            Align2::LEFT_CENTER,
            "FILES & PROJECTS",
            theme::font_small_strong(),
            theme::TEXT_SECONDARY,
        );
        let plus = Rect::from_min_size(Pos2::new(x0 + w - 12.0 - 24.0, y), Vec2::splat(24.0));
        if widgets::icon_button(ui, plus, "home-new-project", Icon::Plus, false, "New project").clicked() {
            acts.push(Act::NewProject);
        }
        widgets::record(ui, "home-new-project", plus);
        y += 26.0;

        // Workspace items: `Drafts`, `All folders`, `Resources & Libraries`
        for (icon, label, section, key) in [
            (Icon::File, "Drafts", Section::Drafts, "drafts"),
            (Icon::GridView, "All folders", Section::AllProjects, "all-projects"),
            (Icon::Assets, "Resources & Libraries", Section::Resources, "resources"),
        ] {
            let r = Rect::from_min_size(Pos2::new(x0 + 8.0, y), Vec2::new(w - 16.0, ROW_H));
            let selected = h.section == section && !h.searching();
            if row(ui, r, Id::new(("home-ws", key)), icon, label, 0.0, selected, false, key).clicked() {
                acts.push(Act::Go(section));
            }
            y += ROW_H + 1.0;
        }
        y += 4.0;

        // Reserve bottom space for `Starred` expandable section + Auto-Update Watchdog card + `Trash` row
        let starred_files: Vec<_> = h.snap.files.iter().filter(|f| f.starred).cloned().collect();
        let starred_rows_n = if h.starred_expanded { starred_files.len().min(3) } else { 0 };
        let bottom_reserve = 52.0 + 48.0 + ROW_H + 16.0 + starred_rows_n as f32 * 26.0;

        // 5. Project Tree (`Getting started`, `Marketing`, and nested folders)
        let list = Rect::from_min_max(
            Pos2::new(x0, y),
            Pos2::new(x0 + w, (rect.bottom() - bottom_reserve).max(y + 64.0)),
        );
        let rows = tree_rows(h);
        let content_h = rows.len() as f32 * ROW_H + 8.0;
        crate::panels::wheel_scroll(ui, list, &mut h.side_scroll, content_h);
        let scroll = h.side_scroll;
        let ptr = ui.input(|i| i.pointer.hover_pos());
        let dragging = h.drag.is_some();
        let mut to_toggle = None;
        let mut secondary: Option<(Target, Pos2)> = None;
        let mut dropzones: Vec<(Rect, DropTo)> = vec![];
        crate::panels::clipped(ui, list, |ui| {
            if rows.is_empty() {
                ui.painter().text(
                    Pos2::new(x0 + 20.0, list.top() + 16.0),
                    Align2::LEFT_CENTER,
                    "No projects yet",
                    theme::font_body(),
                    theme::TEXT_TERTIARY,
                );
            }
            for (i, tr) in rows.iter().enumerate() {
                let r = Rect::from_min_size(
                    Pos2::new(x0 + 8.0, list.top() + i as f32 * ROW_H - scroll),
                    Vec2::new(w - 16.0, ROW_H),
                );
                if r.bottom() < list.top() || r.top() > list.bottom() {
                    continue;
                }
                let selected = !h.searching() && h.section == tr.section;
                let drop_hover = dragging && ptr.is_some_and(|p| r.contains(p) && list.contains(p));
                let icon = if tr.folder { Icon::Folder } else { Icon::Frame };
                let resp = row(
                    ui,
                    r,
                    Id::new(("home-tree", &tr.key)),
                    icon,
                    &tr.label,
                    tr.depth as f32 * INDENT,
                    selected,
                    drop_hover,
                    &format!("project-{}", tr.label),
                );
                if let Some(open) = tr.expanded {
                    let c = Pos2::new(r.left() + 10.0 + tr.depth as f32 * INDENT, r.center().y);
                    icons::paint(
                        ui.painter(),
                        if open { Icon::ChevronDown } else { Icon::ChevronRight },
                        c,
                        12.0,
                        theme::ICON_TERTIARY,
                    );
                }
                let on_chevron = tr.expanded.is_some() && ptr.is_some_and(|p| p.x < r.left() + 22.0 + tr.depth as f32 * INDENT);
                if resp.clicked() {
                    if on_chevron {
                        to_toggle = Some(tr.project.clone());
                    } else {
                        acts.push(Act::Go(tr.section.clone()));
                    }
                }
                if resp.secondary_clicked() {
                    let at = ui.input(|i| i.pointer.interact_pos()).unwrap_or(r.left_bottom());
                    let target = match &tr.section {
                        Section::Project(p, Some(f)) => Target::Folder(p.clone(), f.clone()),
                        _ => Target::Project(tr.project.clone()),
                    };
                    secondary = Some((target, at));
                }
                let to = match &tr.section {
                    Section::Project(p, Some(f)) => DropTo::Folder(p.clone(), f.clone()),
                    _ => DropTo::Project(tr.project.clone()),
                };
                dropzones.push((r.intersect(list), to));
            }
        });
        h.drop_rects.extend(dropzones);
        crate::panels::scrollbar(ui, list, scroll, content_h);
        if let Some(p) = to_toggle {
            if !h.collapsed.remove(&p) {
                h.collapsed.insert(p);
            }
        }
        if secondary.is_some() {
            h.menu = secondary;
        }

        // 6. Hairline Divider + `Starred` Expandable Section (`figma_ui3_home.png`)
        let mut sy = list.bottom() + 4.0;
        widgets::hline(&p, x0 + 12.0, x0 + w - 12.0, sy, UI3_BORDER);
        sy += 6.0;
        let star_r = Rect::from_min_size(Pos2::new(x0 + 8.0, sy), Vec2::new(w - 16.0, ROW_H));
        let star_selected = h.section == Section::Starred && !h.searching();
        let star_resp = row(
            ui,
            star_r,
            Id::new(("home-nav", "starred")),
            Icon::Star,
            "Starred",
            0.0,
            star_selected,
            false,
            "starred",
        );
        icons::paint(
            &p,
            if h.starred_expanded { Icon::ChevronDown } else { Icon::ChevronRight },
            Pos2::new(star_r.right() - 12.0, star_r.center().y),
            11.0,
            theme::ICON_TERTIARY,
        );
        if star_resp.clicked() {
            let on_right_chevron = ptr.is_some_and(|q| q.x > star_r.right() - 24.0);
            if on_right_chevron {
                h.starred_expanded = !h.starred_expanded;
            } else {
                acts.push(Act::Go(Section::Starred));
            }
        }
        sy += ROW_H + 2.0;

        if h.starred_expanded {
            for sf in starred_files.iter().take(3) {
                let sr_row = Rect::from_min_size(Pos2::new(x0 + 16.0, sy), Vec2::new(w - 24.0, 24.0));
                let sresp = ui.interact(sr_row, Id::new(("home-starred-item", &sf.id)), Sense::click());
                if sresp.hovered() {
                    p.rect_filled(sr_row, CornerRadius::same(5), UI3_PILL_HOVER);
                }
                icons::paint(
                    &p,
                    Icon::File,
                    Pos2::new(sr_row.left() + 12.0, sr_row.center().y),
                    13.0,
                    UI3_BLUE,
                );
                let stxt = widgets::elide(&p, &sf.name, &theme::font_small(), sr_row.width() - 28.0);
                p.text(
                    Pos2::new(sr_row.left() + 24.0, sr_row.center().y),
                    Align2::LEFT_CENTER,
                    stxt,
                    theme::font_small(),
                    theme::TEXT_SECONDARY,
                );
                if sresp.clicked() {
                    acts.push(Act::Open(sf.id.clone()));
                }
                sy += 25.0;
            }
        }

        // 7. Auto-Update & Upstream Figma Watchdog Status Card (`US-09` / `DIR-11`)
        let trash_h = 46.0;
        let updater_r = Rect::from_min_size(
            Pos2::new(x0 + 8.0, rect.bottom() - trash_h - 46.0),
            Vec2::new(w - 16.0, 40.0),
        );
        super::updater::show_sidebar_card_in_rect(ui, updater_r, &mut h.updater);

        // 8. Bottom `Trash` Row
        widgets::hline(&p, x0, x0 + w, rect.bottom() - trash_h, UI3_BORDER);
        let tr = Rect::from_min_size(Pos2::new(x0 + 8.0, rect.bottom() - trash_h + 9.0), Vec2::new(w - 16.0, ROW_H));
        let n = h.snap.trash.len();
        let label = if n > 0 { format!("Trash ({n})") } else { "Trash".to_string() };
        if row(
            ui,
            tr,
            Id::new("home-trash"),
            Icon::Trash,
            &label,
            0.0,
            h.section == Section::Trash && !h.searching(),
            false,
            "trash",
        )
        .clicked()
        {
            acts.push(Act::Go(Section::Trash));
        }
    }

    struct TreeRow {
        key: String,
        project: String,
        label: String,
        depth: usize,
        folder: bool,
        /// `Some(open)` when the row can be expanded.
        expanded: Option<bool>,
        section: Section,
    }

    /// Projects and (unless collapsed) their folder trees, flattened.
    fn tree_rows(h: &Home) -> Vec<TreeRow> {
        fn walk(out: &mut Vec<TreeRow>, p: &designos_library::Project, parent: Option<&str>, depth: usize) {
            let mut kids: Vec<_> = p.folders.iter().filter(|f| f.parent.as_deref() == parent).collect();
            kids.sort_by_key(|f| f.name.to_lowercase());
            for f in kids {
                out.push(TreeRow {
                    key: format!("{}/{}", p.id, f.id),
                    project: p.id.clone(),
                    label: f.name.clone(),
                    depth,
                    folder: true,
                    expanded: None,
                    section: Section::Project(p.id.clone(), Some(f.id.clone())),
                });
                walk(out, p, Some(&f.id), depth + 1);
            }
        }
        let mut out = vec![];
        for p in &h.snap.projects {
            let open = !h.collapsed.contains(&p.id);
            out.push(TreeRow {
                key: p.id.clone(),
                project: p.id.clone(),
                label: p.name.clone(),
                depth: 0,
                folder: false,
                expanded: (!p.folders.is_empty()).then_some(open),
                section: Section::Project(p.id.clone(), None),
            });
            if open {
                walk(&mut out, p, None, 1);
            }
        }
        out
    }

    /// One sidebar row: icon, label, hover / `#2C2C2C` selected / drop-target highlight.
    #[allow(clippy::too_many_arguments)]
    fn row(
        ui: &mut Ui,
        r: Rect,
        id: Id,
        icon: Icon,
        label: &str,
        indent: f32,
        selected: bool,
        drop_hover: bool,
        record: &str,
    ) -> Response {
        let resp = ui.interact(r, id, Sense::click());
        let p = ui.painter();
        if drop_hover {
            p.rect_filled(r, CornerRadius::same(6), theme::BG_SELECTED_HOVER);
        } else if selected {
            p.rect_filled(r, CornerRadius::same(6), UI3_PILL_ACTIVE);
        } else if resp.hovered() {
            p.rect_filled(r, CornerRadius::same(6), UI3_PILL_HOVER);
        }
        let ix = r.left() + 12.0 + indent;
        icons::paint(
            p,
            icon,
            Pos2::new(ix + 14.0, r.center().y),
            15.0,
            if selected { Color32::WHITE } else { theme::ICON_SECONDARY },
        );
        let font = if selected {
            theme::font_body_strong()
        } else {
            theme::font_body()
        };
        let shown = widgets::elide(p, label, &font, r.right() - (ix + 30.0) - 8.0);
        p.text(
            Pos2::new(ix + 30.0, r.center().y),
            Align2::LEFT_CENTER,
            shown,
            font,
            if selected { Color32::WHITE } else { theme::TEXT },
        );
        widgets::record(ui, &format!("home-{record}"), r);
        resp
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Inline Main Area (`figma_ui3_home.png`: Top Header, Creation Strip, Filter/Sort Bar, Grid/List Cards)
// ─────────────────────────────────────────────────────────────────────────────
mod main_view {
    use super::time::{edited, relative, wall_ms};
    use super::{Act, DropTo, Home, Section, Target, ViewMode};
    use crate::icons::{self, Icon};
    use crate::theme;
    use crate::widgets;
    use designos_library::{FileEntry, Library};
    use egui::{Align2, Color32, CornerRadius, Id, Pos2, Rect, Response, Sense, Stroke, StrokeKind, Ui, Vec2};

    // Figma UI3 Dark Mode tokens (`US-04` / `BR-07`)
    const UI3_MAIN_BG: Color32 = Color32::from_rgb(0x1E, 0x1E, 0x1E);
    const UI3_SURFACE: Color32 = Color32::from_rgb(0x2C, 0x2C, 0x2C);
    const UI3_BORDER: Color32 = Color32::from_rgb(0x38, 0x38, 0x38);
    const UI3_BLUE: Color32 = Color32::from_rgb(0x0D, 0x99, 0xFF);
    const UI3_BLUE_HOVER: Color32 = Color32::from_rgb(0x0B, 0x84, 0xE0);
    const UI3_PURPLE: Color32 = Color32::from_rgb(0x97, 0x47, 0xFF);
    const UI3_ORANGE: Color32 = Color32::from_rgb(0xFF, 0x6A, 0x3D);

    #[derive(Clone, Copy, PartialEq)]
    enum Kind {
        PrimaryBlue,
        Secondary,
        Danger,
    }

    pub fn show(lib: &Library, h: &mut Home, ui: &mut Ui, rect: Rect, acts: &mut Vec<Act>) {
        let p = ui.painter().clone();
        p.rect_filled(rect, 0.0, UI3_MAIN_BG);
        let pad = theme::HOME_PAD;
        let trash = h.section == Section::Trash && !h.searching();

        // 1. Top Header & Creation Strip (`+ Design file`, `+ FigJam board`, `+ Spec bundle`, `Import`)
        let head = Rect::from_min_size(rect.min, Vec2::new(rect.width(), theme::HOME_HEADER_H));
        breadcrumb(h, ui, &p, Pos2::new(rect.left() + pad, head.center().y), acts);
        let mut right = rect.right() - pad;
        if trash {
            if !h.snap.trash.is_empty() {
                let r = button(ui, &p, &mut right, head.center().y, "Empty trash", Kind::Danger, "home-empty-trash");
                if r.clicked() {
                    acts.push(Act::EmptyTrash);
                }
            }
        } else {
            let r = button(ui, &p, &mut right, head.center().y, "+ Design file", Kind::PrimaryBlue, "home-new-file");
            if r.clicked() {
                acts.push(Act::NewFile);
            }
            right -= 8.0;
            let r = button(ui, &p, &mut right, head.center().y, "+ FigJam board", Kind::Secondary, "home-new-board");
            if r.clicked() {
                acts.push(Act::NewBoard);
            }
            right -= 8.0;
            let r = button(ui, &p, &mut right, head.center().y, "+ Spec bundle", Kind::Secondary, "home-new-spec");
            if r.clicked() {
                acts.push(Act::OpenSpecs);
            }
            right -= 8.0;
            let r = button(ui, &p, &mut right, head.center().y, "Import", Kind::Secondary, "home-import");
            if r.clicked() {
                acts.push(Act::Import);
            }
        }
        let mut top = head.bottom();

        // 2. Sub-Header Filter & Sort Bar (`Filter: All files v`, `Sort: Last viewed v`, Grid/List toggle)
        if !trash {
            let y = top + 4.0;
            let filter_lbl = format!("Filter: {}", h.filter.label());
            let fw = widgets::text_width(&p, &filter_lbl, &theme::font_body()) + 36.0;
            let fr = Rect::from_min_size(Pos2::new(rect.left() + pad, y), Vec2::new(fw, 28.0));
            let fresp = ui.interact(fr, Id::new("home-filter"), Sense::click());
            p.rect_filled(
                fr,
                CornerRadius::same(6),
                if h.filter_menu.is_some() || fresp.hovered() {
                    UI3_BORDER
                } else {
                    UI3_SURFACE
                },
            );
            p.rect_stroke(fr, CornerRadius::same(6), Stroke::new(1.0, UI3_BORDER), StrokeKind::Inside);
            p.text(
                Pos2::new(fr.left() + 10.0, fr.center().y),
                Align2::LEFT_CENTER,
                &filter_lbl,
                theme::font_body(),
                theme::TEXT,
            );
            icons::paint(
                &p,
                Icon::ChevronDown,
                Pos2::new(fr.right() - 12.0, fr.center().y),
                theme::CHEVRON_SMALL,
                theme::ICON_SECONDARY,
            );
            widgets::record(ui, "home-filter", fr);
            if fresp.clicked() {
                h.filter_menu = if h.filter_menu.is_some() { None } else { Some(fr) };
                h.sort_menu = None;
            }

            let sort_lbl = format!("Sort: {}", h.sort.label());
            let sw = widgets::text_width(&p, &sort_lbl, &theme::font_body()) + 36.0;
            let sr = Rect::from_min_size(Pos2::new(fr.right() + 8.0, y), Vec2::new(sw, 28.0));
            let sresp = ui.interact(sr, Id::new("home-sort"), Sense::click());
            p.rect_filled(
                sr,
                CornerRadius::same(6),
                if h.sort_menu.is_some() || sresp.hovered() {
                    UI3_BORDER
                } else {
                    UI3_SURFACE
                },
            );
            p.rect_stroke(sr, CornerRadius::same(6), Stroke::new(1.0, UI3_BORDER), StrokeKind::Inside);
            p.text(
                Pos2::new(sr.left() + 10.0, sr.center().y),
                Align2::LEFT_CENTER,
                &sort_lbl,
                theme::font_body(),
                theme::TEXT,
            );
            icons::paint(
                &p,
                Icon::ChevronDown,
                Pos2::new(sr.right() - 12.0, sr.center().y),
                theme::CHEVRON_SMALL,
                theme::ICON_SECONDARY,
            );
            widgets::record(ui, "home-sort", sr);
            if sresp.clicked() {
                h.sort_menu = if h.sort_menu.is_some() { None } else { Some(sr) };
                h.filter_menu = None;
            }

            let mut x = rect.right() - pad;
            for (mode, icon, key, tip) in [
                (ViewMode::List, Icon::ListView, "home-view-list", "List view"),
                (ViewMode::Grid, Icon::GridView, "home-view-grid", "Grid view"),
            ] {
                let br = Rect::from_min_size(Pos2::new(x - 28.0, y), Vec2::splat(28.0));
                x -= 28.0 + 4.0;
                if widgets::icon_button(ui, br, key, icon, h.view == mode, tip).clicked() {
                    acts.push(Act::SetView(mode));
                }
                widgets::record(ui, key, br);
            }
            top += 38.0;
        }
        widgets::hline(&p, rect.left(), rect.right(), top, UI3_BORDER);
        top += 1.0;

        let content = Rect::from_min_max(Pos2::new(rect.left(), top), rect.max);
        if trash {
            trash_view(h, ui, content, acts);
        } else {
            files_view(lib, h, ui, content, acts);
        }
    }

    /// Title, or clickable "Project / Folder / ..." path inside a project.
    fn breadcrumb(h: &Home, ui: &mut Ui, p: &egui::Painter, at: Pos2, acts: &mut Vec<Act>) {
        let font = theme::font_title();
        let mut segs: Vec<(String, Section)> = vec![];
        match &h.section {
            Section::Project(pid, folder) if !h.searching() => {
                segs.push((h.snap.project_name(pid).to_string(), Section::Project(pid.clone(), None)));
                if let Some(pr) = h.snap.projects.iter().find(|x| &x.id == pid) {
                    let mut chain = vec![];
                    let mut cur = folder.clone();
                    while let Some(fid) = cur {
                        let Some(f) = pr.folders.iter().find(|x| x.id == fid) else { break };
                        chain.push((f.name.clone(), Section::Project(pid.clone(), Some(f.id.clone()))));
                        cur = f.parent.clone();
                    }
                    chain.reverse();
                    segs.extend(chain);
                }
            }
            _ => segs.push((h.title(), h.section.clone())),
        }
        let mut x = at.x;
        let last = segs.len() - 1;
        for (i, (label, section)) in segs.into_iter().enumerate() {
            let w = widgets::text_width(p, &label, &font);
            let r = Rect::from_min_size(Pos2::new(x, at.y - 14.0), Vec2::new(w, 28.0));
            let color = if i == last { theme::TEXT } else { theme::TEXT_SECONDARY };
            if i < last {
                let resp = ui.interact(r, Id::new(("home-crumb", i)), Sense::click());
                if resp.hovered() {
                    p.rect_filled(r.expand2(Vec2::new(4.0, 0.0)), theme::cr(theme::RADIUS_MEDIUM), theme::BG_HOVER);
                }
                if resp.clicked() {
                    acts.push(Act::Go(section));
                }
            }
            p.text(Pos2::new(x, at.y), Align2::LEFT_CENTER, &label, font.clone(), color);
            x += w;
            if i < last {
                p.text(Pos2::new(x + 8.0, at.y), Align2::LEFT_CENTER, "/", font.clone(), theme::TEXT_TERTIARY);
                x += 24.0;
            }
        }
    }

    /// Right-aligned button; moves `right` to its left edge.
    fn button(ui: &mut Ui, p: &egui::Painter, right: &mut f32, cy: f32, label: &str, kind: Kind, record: &str) -> Response {
        let font = theme::font_body_strong();
        let w = widgets::text_width(p, label, &font) + 26.0;
        let r = Rect::from_min_size(Pos2::new(*right - w, cy - 16.0), Vec2::new(w, 32.0));
        *right = r.left();
        let resp = ui.interact(r, Id::new(("home-button", record)), Sense::click());
        let radius = CornerRadius::same(6);
        let down = resp.is_pointer_button_down_on();
        match kind {
            Kind::PrimaryBlue => {
                let bg = if down || resp.hovered() { UI3_BLUE_HOVER } else { UI3_BLUE };
                p.rect_filled(r, radius, bg);
                p.text(r.center(), Align2::CENTER_CENTER, label, font, Color32::WHITE);
            }
            Kind::Secondary => {
                let bg = if down || resp.hovered() { UI3_BORDER } else { UI3_SURFACE };
                p.rect_filled(r, radius, bg);
                p.rect_stroke(r, radius, Stroke::new(1.0, UI3_BORDER), StrokeKind::Inside);
                p.text(r.center(), Align2::CENTER_CENTER, label, font, theme::TEXT);
            }
            Kind::Danger => {
                let bg = if resp.hovered() {
                    theme::DANGER.gamma_multiply(0.2)
                } else {
                    Color32::TRANSPARENT
                };
                p.rect_filled(r, radius, bg);
                p.rect_stroke(r, radius, Stroke::new(1.0, theme::DANGER), StrokeKind::Inside);
                p.text(r.center(), Align2::CENTER_CENTER, label, font, theme::DANGER);
            }
        }
        widgets::record(ui, record, r);
        resp
    }

    fn empty_state(h: &Home, ui: &Ui, area: Rect) {
        let (title, body) = if h.searching() {
            (
                format!("No files match \u{201c}{}\u{201d}", h.search.trim()),
                "Try a different name or clear the filter.".to_string(),
            )
        } else {
            match &h.section {
                Section::Recents | Section::Drafts | Section::AllProjects | Section::Templates | Section::Resources => (
                    "No files yet".into(),
                    "Create a design file, FigJam board, or import a .fig, .svg or .designos file to get started.".into(),
                ),
                Section::Starred => (
                    "Nothing starred".into(),
                    "Hover a file thumbnail or right-click a file and choose Star to keep it handy here.".into(),
                ),
                Section::Project(..) => (
                    "This project is empty".into(),
                    "Create a design file here, or drag files in from other projects.".into(),
                ),
                Section::Trash => (
                    "Trash is empty".into(),
                    "Files and projects you delete wait here until you empty the trash.".into(),
                ),
            }
        };
        let p = ui.painter();
        let c = area.center();
        icons::paint(p, Icon::File, Pos2::new(c.x, c.y - 44.0), 32.0, theme::ICON_TERTIARY);
        p.text(Pos2::new(c.x, c.y), Align2::CENTER_CENTER, title, theme::font_large_strong(), theme::TEXT);
        let galley = p.layout(body, theme::font_body(), theme::TEXT_SECONDARY, 380.0_f32.min(area.width() - 48.0));
        let pos = Pos2::new(c.x - galley.size().x / 2.0, c.y + 18.0);
        p.galley(pos, galley, theme::TEXT_SECONDARY);
    }

    fn files_view(lib: &Library, h: &mut Home, ui: &mut Ui, content: Rect, acts: &mut Vec<Act>) {
        let files = h.files();
        let folders = h.folders();
        if files.is_empty() && folders.is_empty() {
            empty_state(h, ui, content);
            return;
        }
        let now = wall_ms();
        let pad = theme::HOME_PAD;
        let list = h.view == ViewMode::List;
        let inner_w = content.width() - pad * 2.0;
        let (cw, ch, gap_x, gap_y) = (theme::HOME_CARD_W, theme::HOME_CARD_H, theme::HOME_CARD_GAP, 24.0);
        let cols = (((inner_w + gap_x) / (cw + gap_x)).floor() as usize).max(1);
        let folder_rows = if list {
            folders.len() as f32 * theme::HOME_LIST_ROW_H
        } else {
            folders.len().div_ceil(cols) as f32 * (theme::HOME_FOLDER_H + 12.0)
        };
        let file_rows = if list {
            files.len() as f32 * theme::HOME_LIST_ROW_H + 28.0
        } else {
            files.len().div_ceil(cols) as f32 * (ch + gap_y)
        };
        let content_h = pad + folder_rows + if folder_rows > 0.0 { 12.0 } else { 0.0 } + file_rows + pad;
        crate::panels::wheel_scroll(ui, content, &mut h.scroll, content_h);
        let scroll = h.scroll;
        let ptr = ui.input(|i| i.pointer.hover_pos());
        let dragging = h.drag.is_some();
        let mut budget = 4u32;
        let selected = h.selected.clone();
        let mut new_selected = None;
        let mut menu = None;
        let mut drag = None;
        let mut zones: Vec<(Rect, DropTo)> = vec![];
        let section = h.section.clone();
        let ctx = ui.ctx().clone();
        let project_id = if let Section::Project(p, _) = &section {
            Some(p.clone())
        } else {
            None
        };
        crate::panels::clipped(ui, content, |ui| {
            let p = ui.painter().clone();
            let mut y = content.top() + pad - scroll;
            let x0 = content.left() + pad;
            // ---- folder tiles
            for (i, (fid, name)) in folders.iter().enumerate() {
                let r = if list {
                    Rect::from_min_size(Pos2::new(x0, y + i as f32 * theme::HOME_LIST_ROW_H), Vec2::new(inner_w, theme::HOME_LIST_ROW_H))
                } else {
                    Rect::from_min_size(
                        Pos2::new(
                            x0 + (i % cols) as f32 * (cw + gap_x),
                            y + (i / cols) as f32 * (theme::HOME_FOLDER_H + 12.0),
                        ),
                        Vec2::new(cw, theme::HOME_FOLDER_H),
                    )
                };
                if r.bottom() < content.top() || r.top() > content.bottom() {
                    continue;
                }
                let Some(pid) = project_id.clone() else { continue };
                let resp = ui.interact(r, Id::new(("home-folder", fid)), Sense::click());
                let over = dragging && ptr.is_some_and(|q| r.contains(q));
                let fill = if over {
                    theme::BG_SELECTED_HOVER
                } else if resp.hovered() {
                    UI3_BORDER
                } else {
                    UI3_SURFACE
                };
                p.rect_filled(
                    r,
                    CornerRadius::same(8),
                    if list {
                        if over || resp.hovered() {
                            fill
                        } else {
                            Color32::TRANSPARENT
                        }
                    } else {
                        fill
                    },
                );
                icons::paint(&p, Icon::Folder, Pos2::new(r.left() + 24.0, r.center().y), 20.0, theme::ICON_SECONDARY);
                p.text(
                    Pos2::new(r.left() + 44.0, r.center().y),
                    Align2::LEFT_CENTER,
                    widgets::elide(&p, name, &theme::font_body_strong(), r.width() - 60.0),
                    theme::font_body_strong(),
                    theme::TEXT,
                );
                widgets::record(ui, &format!("home-folder-{name}"), r);
                if resp.double_clicked() || resp.clicked() {
                    acts.push(Act::Go(Section::Project(pid.clone(), Some(fid.clone()))));
                }
                if resp.secondary_clicked() {
                    menu = ui.input(|i| i.pointer.interact_pos()).map(|at| (Target::Folder(pid.clone(), fid.clone()), at));
                }
                zones.push((r.intersect(content), DropTo::Folder(pid, fid.clone())));
            }
            y += folder_rows + if folder_rows > 0.0 { 12.0 } else { 0.0 };

            if list {
                let hr = Rect::from_min_size(Pos2::new(x0, y), Vec2::new(inner_w, 28.0));
                let cols_x = list_columns(hr.right());
                for (x, label) in [
                    (hr.left() + 60.0, "Name"),
                    (cols_x.0, "Location"),
                    (cols_x.1, "Last modified"),
                    (cols_x.2, "Last viewed"),
                ] {
                    p.text(Pos2::new(x, hr.center().y), Align2::LEFT_CENTER, label, theme::font_small_medium(), theme::TEXT_TERTIARY);
                }
                y += 28.0;
            }
            for (i, f) in files.iter().enumerate() {
                let r = if list {
                    Rect::from_min_size(Pos2::new(x0, y + i as f32 * theme::HOME_LIST_ROW_H), Vec2::new(inner_w, theme::HOME_LIST_ROW_H))
                } else {
                    Rect::from_min_size(
                        Pos2::new(x0 + (i % cols) as f32 * (cw + gap_x), y + (i / cols) as f32 * (ch + gap_y)),
                        Vec2::new(cw, ch),
                    )
                };
                if r.bottom() < content.top() || r.top() > content.bottom() {
                    continue;
                }
                let is_sel = selected.as_deref() == Some(f.id.as_str());
                let resp = ui.interact(r, Id::new(("home-card", &f.id)), Sense::click_and_drag());
                if resp.double_clicked() {
                    acts.push(Act::Open(f.id.clone()));
                } else if resp.clicked() {
                    new_selected = Some(f.id.clone());
                }
                if resp.secondary_clicked() {
                    new_selected = Some(f.id.clone());
                    menu = ui.input(|i| i.pointer.interact_pos()).map(|at| (Target::File(f.id.clone()), at));
                }
                if resp.drag_started() {
                    drag = Some((f.id.clone(), f.name.clone()));
                }
                let tex = h.thumbs.get(&ctx, lib, f, &mut budget);
                let board = h.kinds.get(lib, f, &mut budget) == designos_doc::DocumentKind::Board;
                let project = h.snap.project_name(&f.project).to_string();
                let star_r = if list {
                    Rect::from_min_size(Pos2::new(r.right() - 40.0, r.center().y - 12.0), Vec2::splat(24.0))
                } else {
                    Rect::from_min_size(Pos2::new(r.left() + cw - 32.0, r.top() + 8.0), Vec2::splat(24.0))
                };
                let star = ui.interact(star_r, Id::new(("home-star", &f.id)), Sense::click());
                if star.clicked() {
                    acts.push(Act::Star(f.id.clone(), !f.starred));
                }
                if list {
                    list_row(&p, r, f, &project, tex, board, resp.hovered(), is_sel, now);
                } else {
                    card(&p, r, f, &project, tex, board, resp.hovered(), is_sel, now);
                }
                widgets::record(ui, &format!("home-kind-{}", if board { "board" } else { "design" }), r);
                if resp.hovered() || star.hovered() || f.starred {
                    if star.hovered() || (!list && resp.hovered()) {
                        p.circle_filled(
                            star_r.center(),
                            12.0,
                            Color32::from_black_alpha(if f.starred && !star.hovered() { 80 } else { 150 }),
                        );
                    }
                    paint_star(&p, star_r.center(), f.starred, star.hovered());
                }
                widgets::record(ui, &format!("home-card-{}", f.name), r);
                if dragging && selected.as_deref() == Some(f.id.as_str()) {
                    p.rect_filled(r, theme::cr(theme::RADIUS_LARGE), Color32::from_black_alpha(90));
                }
            }
        });
        crate::panels::scrollbar(ui, content, scroll, content_h);
        h.drop_rects.extend(zones);
        if let Some(s) = new_selected {
            h.selected = Some(s);
        }
        if menu.is_some() {
            h.menu = menu;
        }
        if drag.is_some() {
            h.drag = drag;
        }
    }

    fn paint_star(p: &egui::Painter, c: Pos2, on: bool, hot: bool) {
        let col = if on {
            Color32::from_rgb(0xFF, 0xCD, 0x29)
        } else if hot {
            theme::ICON
        } else {
            theme::ICON_SECONDARY
        };
        if on {
            icons::paint(p, Icon::Star, c, 14.0, col);
            icons::paint(p, Icon::Star, c, 11.0, col);
            icons::paint(p, Icon::Star, c, 8.0, col);
        } else {
            icons::paint(p, Icon::Star, c, 14.0, col);
        }
    }

    type Tex = Option<(egui::TextureId, Vec2)>;

    fn thumb_area(p: &egui::Painter, area: Rect, tex: Tex, board: bool, radius: u8) {
        if board {
            p.rect_filled(area, theme::cr(radius), theme::rgb(theme::BOARD_BG));
            match tex {
                Some((t, _)) => {
                    let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
                    p.add(egui::epaint::RectShape::filled(area, theme::cr(radius), Color32::WHITE).with_texture(t, uv));
                }
                None => {
                    let mut mesh = egui::Mesh::default();
                    let mut y = area.top() + 8.0;
                    while y < area.bottom() {
                        let mut x = area.left() + 8.0;
                        while x < area.right() {
                            mesh.add_colored_rect(Rect::from_center_size(Pos2::new(x, y), Vec2::splat(1.5)), theme::BOARD_DOT);
                            x += 16.0;
                        }
                        y += 16.0;
                    }
                    p.add(egui::Shape::mesh(mesh));
                    icons::paint(p, Icon::Sticky, area.center(), 24.0, Color32::from_black_alpha(110));
                }
            }
            return;
        }
        p.rect_filled(area, theme::cr(radius), theme::HOME_THUMB_BG);
        match tex {
            Some((t, size)) => {
                let inner = area.shrink(if area.width() > 80.0 { 12.0 } else { 3.0 });
                let k = (inner.width() / size.x.max(1.0))
                    .min(inner.height() / size.y.max(1.0))
                    .min(2.0);
                let fit = Rect::from_center_size(inner.center(), size * k);
                p.image(t, fit, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
            }
            None => icons::paint(p, Icon::File, area.center(), 24.0, theme::ICON_TERTIARY),
        }
    }

    fn board_badge(p: &egui::Painter, area: Rect) {
        let font = theme::font_small_strong();
        let w = widgets::text_width(p, "Board", &font) + 28.0;
        let r = Rect::from_min_size(Pos2::new(area.left() + 8.0, area.bottom() - 8.0 - 20.0), Vec2::new(w, 20.0));
        p.rect_filled(r, theme::cr(10), theme::BG_MENU);
        icons::paint(p, Icon::Sticky, Pos2::new(r.left() + 10.0, r.center().y), 12.0, theme::ICON);
        p.text(Pos2::new(r.left() + 19.0, r.center().y), Align2::LEFT_CENTER, "Board", font, theme::TEXT);
    }

    /// Draws the Figma UI3 File-Type Badge icon (`20x20` rounded badge: `#0D99FF` Design,
    /// `#9747FF` Board/Component, `#FF6A3D` Spec bundle).
    fn draw_file_type_badge(p: &egui::Painter, center: Pos2, name: &str, board: bool) {
        let low = name.to_ascii_lowercase();
        let (bg, icon) = if board {
            (UI3_PURPLE, Icon::Sticky)
        } else if low.contains("spec") || low.contains("v2") {
            (UI3_ORANGE, Icon::Specs)
        } else {
            (UI3_BLUE, Icon::Pen)
        };
        let badge_r = Rect::from_center_size(center, Vec2::splat(20.0));
        p.rect_filled(badge_r, CornerRadius::same(5), bg);
        icons::paint(p, icon, center, 12.0, Color32::WHITE);
    }

    /// A Figma UI3 grid card (`figma_ui3_home.png`): `16:10` thumbnail with `#383838` border
    /// (`#0D99FF` `2px` when selected), bottom metadata row with left File-Type Badge icon +
    /// Title + Subtitle (`Edited Xm ago · In <Project>`).
    #[allow(clippy::too_many_arguments)]
    fn card(p: &egui::Painter, r: Rect, f: &FileEntry, project: &str, tex: Tex, board: bool, hover: bool, selected: bool, now: u64) {
        let thumb = Rect::from_min_size(r.min, Vec2::new(r.width(), theme::HOME_THUMB_H));
        thumb_area(p, thumb, tex, board, theme::RADIUS_LARGE);
        if board {
            board_badge(p, thumb);
        }
        let stroke = if selected {
            Stroke::new(2.0, UI3_BLUE)
        } else if hover {
            Stroke::new(1.0, theme::TEXT_TERTIARY)
        } else {
            Stroke::new(1.0, UI3_BORDER)
        };
        p.rect_stroke(thumb, theme::cr(theme::RADIUS_LARGE), stroke, StrokeKind::Inside);

        let badge_c = Pos2::new(r.left() + 12.0, thumb.bottom() + 20.0);
        draw_file_type_badge(p, badge_c, &f.name, board);

        let tx = r.left() + 28.0;
        let w = (r.width() - 32.0).max(40.0);
        let y_title = thumb.bottom() + 13.0;
        p.text(
            Pos2::new(tx, y_title),
            Align2::LEFT_CENTER,
            widgets::elide(p, &f.name, &theme::font_body_strong(), w),
            theme::font_body_strong(),
            theme::TEXT,
        );
        let loc = if project.is_empty() { "Drafts" } else { project };
        let subtitle = format!("{} \u{b7} In {loc}", edited(now, f.modified));
        p.text(
            Pos2::new(tx, y_title + 17.0),
            Align2::LEFT_CENTER,
            widgets::elide(p, &subtitle, &theme::font_small(), w),
            theme::font_small(),
            theme::TEXT_SECONDARY,
        );
    }

    fn list_columns(right: f32) -> (f32, f32, f32) {
        (right - 520.0, right - 340.0, right - 180.0)
    }

    #[allow(clippy::too_many_arguments)]
    fn list_row(p: &egui::Painter, r: Rect, f: &FileEntry, project: &str, tex: Tex, board: bool, hover: bool, selected: bool, now: u64) {
        if selected {
            p.rect_filled(r, CornerRadius::same(6), UI3_SURFACE);
            p.rect_stroke(r, CornerRadius::same(6), Stroke::new(1.0, UI3_BLUE), StrokeKind::Inside);
        } else if hover {
            p.rect_filled(r, CornerRadius::same(6), theme::BG_HOVER);
        }
        let th = Rect::from_min_size(Pos2::new(r.left() + 8.0, r.top() + 5.0), Vec2::new(48.0, r.height() - 10.0));
        thumb_area(p, th, tex, board, theme::RADIUS_SMALL);
        draw_file_type_badge(p, Pos2::new(r.left() + 70.0, r.center().y), &f.name, board);
        let cols = list_columns(r.right());
        let font = theme::font_body();
        p.text(
            Pos2::new(r.left() + 86.0, r.center().y),
            Align2::LEFT_CENTER,
            widgets::elide(p, &f.name, &theme::font_body_strong(), (cols.0 - r.left() - 96.0).max(40.0)),
            theme::font_body_strong(),
            theme::TEXT,
        );
        let loc = if project.is_empty() { "Drafts" } else { project };
        p.text(
            Pos2::new(cols.0, r.center().y),
            Align2::LEFT_CENTER,
            widgets::elide(p, loc, &font, 150.0),
            font.clone(),
            theme::TEXT_SECONDARY,
        );
        p.text(
            Pos2::new(cols.1, r.center().y),
            Align2::LEFT_CENTER,
            relative(now, f.modified),
            font.clone(),
            theme::TEXT_SECONDARY,
        );
        p.text(
            Pos2::new(cols.2, r.center().y),
            Align2::LEFT_CENTER,
            relative(now, f.opened),
            font,
            theme::TEXT_SECONDARY,
        );
    }

    fn trash_view(h: &mut Home, ui: &mut Ui, content: Rect, acts: &mut Vec<Act>) {
        if h.snap.trash.is_empty() {
            empty_state(h, ui, content);
            return;
        }
        let now = wall_ms();
        let pad = theme::HOME_PAD;
        let row_h = 52.0;
        let note_h = 36.0;
        let content_h = pad + note_h + h.snap.trash.len() as f32 * row_h + pad;
        crate::panels::wheel_scroll(ui, content, &mut h.scroll, content_h);
        let scroll = h.scroll;
        let items = h.snap.trash.clone();
        let mut menu = None;
        crate::panels::clipped(ui, content, |ui| {
            let p = ui.painter().clone();
            let x0 = content.left() + pad;
            let w = content.width() - pad * 2.0;
            let mut y = content.top() + pad - scroll;
            p.text(
                Pos2::new(x0, y + 10.0),
                Align2::LEFT_CENTER,
                "Items stay here until you empty the trash. Emptying it deletes them permanently.",
                theme::font_body(),
                theme::TEXT_SECONDARY,
            );
            y += note_h;
            for it in items.iter() {
                let r = Rect::from_min_size(Pos2::new(x0, y), Vec2::new(w, row_h));
                y += row_h;
                if r.bottom() < content.top() || r.top() > content.bottom() {
                    continue;
                }
                let resp = ui.interact(r, Id::new(("home-trash-row", &it.id)), Sense::click());
                if resp.hovered() {
                    p.rect_filled(r, theme::cr(theme::RADIUS_MEDIUM), theme::BG_HOVER);
                }
                if resp.secondary_clicked() {
                    menu = ui.input(|i| i.pointer.interact_pos()).map(|at| (Target::TrashItem(it.id.clone()), at));
                }
                let is_file = it.kind == designos_library::TrashKind::File;
                icons::paint(
                    &p,
                    if is_file { Icon::File } else { Icon::Frame },
                    Pos2::new(r.left() + 24.0, r.center().y),
                    20.0,
                    theme::ICON_SECONDARY,
                );
                p.text(
                    Pos2::new(r.left() + 48.0, r.center().y - 9.0),
                    Align2::LEFT_CENTER,
                    &it.name,
                    theme::font_body_strong(),
                    theme::TEXT,
                );
                let from = match (&it.project_name, is_file) {
                    (Some(pn), true) => format!("Deleted {} \u{b7} from {pn}", relative(now, it.trashed)),
                    (_, false) => format!("Project \u{b7} deleted {}", relative(now, it.trashed)),
                    _ => format!("Deleted {}", relative(now, it.trashed)),
                };
                p.text(
                    Pos2::new(r.left() + 48.0, r.center().y + 9.0),
                    Align2::LEFT_CENTER,
                    from,
                    theme::font_body(),
                    theme::TEXT_SECONDARY,
                );
                let mut right = r.right() - 8.0;
                let b = button(
                    ui,
                    &p,
                    &mut right,
                    r.center().y,
                    "Restore",
                    Kind::Secondary,
                    &format!("home-restore-{}", it.name),
                );
                if b.clicked() {
                    acts.push(Act::Restore(it.id.clone()));
                }
            }
        });
        if menu.is_some() {
            h.menu = menu;
        }
        crate::panels::scrollbar(ui, content, scroll, content_h);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, name: &str, project: &str, folder: Option<&str>, starred: bool, opened: u64, modified: u64) -> FileEntry {
        FileEntry {
            id: id.into(),
            name: name.into(),
            project: project.into(),
            folder: folder.map(Into::into),
            starred,
            created: 50,
            modified,
            opened,
            thumbnail: None,
        }
    }

    #[test]
    fn ui3_home_sections_filters_and_sorts_files() {
        let mut home = Home::new();
        home.snap.files = vec![
            entry("f-design", "Design Token Sheet", "p1", None, false, 300, 100),
            entry("f-board", "Architecture Board", "p1", Some("fol1"), false, 200, 300),
            entry("f-spec", "US-04 Spec Bundle", "p1", None, true, 100, 200),
        ];

        assert_eq!(home.section, Section::Recents);
        assert_eq!(home.filter, FileFilter::AllFiles);
        assert_eq!(home.sort, Sort::LastOpened);
        assert_eq!(home.files().len(), 3);

        // Filter by Boards
        home.filter = FileFilter::Boards;
        let boards = home.files();
        assert_eq!(boards.len(), 1);
        assert_eq!(boards[0].id, "f-board");

        // Filter by SpecBundles
        home.filter = FileFilter::SpecBundles;
        let specs = home.files();
        assert_eq!(specs.len(), 1);
        assert_eq!(specs[0].id, "f-spec");

        // Filter by DesignFiles
        home.filter = FileFilter::DesignFiles;
        let designs = home.files();
        assert_eq!(designs.len(), 1);
        assert_eq!(designs[0].id, "f-design");

        // Reset filter, test Sort::Name
        home.filter = FileFilter::AllFiles;
        home.sort = Sort::Name;
        let sorted = home.files();
        assert_eq!(sorted[0].id, "f-board"); // "Architecture Board"
        assert_eq!(sorted[1].id, "f-design"); // "Design Token Sheet"
        assert_eq!(sorted[2].id, "f-spec"); // "US-04 Spec Bundle"

        // Drafts section (folder.is_none())
        home.section = Section::Drafts;
        assert_eq!(home.files().len(), 2);

        // Starred section
        home.section = Section::Starred;
        assert_eq!(home.files().len(), 1);
        assert_eq!(home.files()[0].id, "f-spec");

        // AllProjects, Templates, Resources sections
        home.section = Section::AllProjects;
        assert_eq!(home.files().len(), 3);
        home.section = Section::Templates;
        assert_eq!(home.files().len(), 3);
        home.section = Section::Resources;
        assert_eq!(home.files().len(), 3);
    }
}


