//! Agent edits that arrive while the user is dragging or typing: replaying them there would be
//! undone by the drag's own cancel (Escape reverts every node the drag touched), and the agent's
//! undo step would capture a half-dragged state. They wait until the canvas is idle.

use crate::app::App;
use crate::canvas::{Interaction, MoveState, TextSession};
use crate::edit::Tx;
use crate::live_sync_tests::unique_tab_id;
use crate::terminal::TerminalState;
use designos_doc::{Document, Node, NodeId, Rect, Vec2};
use serde_json::json;
use std::path::PathBuf;

struct Live {
    app: App,
    session: designos_mcp::server::Session,
    dir: PathBuf,
    workspace: PathBuf,
    dragged: NodeId,
    other: NodeId,
}

fn live() -> Live {
    let mut doc = Document::new("Live");
    let page = doc.add_page("Page");
    let mut ids = Vec::new();
    for name in ["Hero", "Footer"] {
        let id = doc.alloc_id();
        doc.insert(page, Node::frame(id, Rect::new(0.0, 0.0, 100.0, 100.0))).unwrap();
        doc.get_mut(id).unwrap().name = name.into();
        ids.push(id);
    }
    let mut app = App::new(doc);
    app.terminal = TerminalState::for_tab(unique_tab_id());
    let dir = app.terminal.session_dir.clone();
    let workspace = std::env::temp_dir().join(format!("designos-live-interaction-ws-{}", unique_tab_id()));
    std::fs::create_dir_all(&workspace).unwrap();
    app.terminal.sync_live_file_context(&app.doc, app.page, &app.selection, None);
    let session = designos_mcp::server::Session::with_session_dir(&workspace, &dir).expect("mcp session");
    Live { app, session, dir, workspace, dragged: ids[0], other: ids[1] }
}

impl Live {
    fn agent_renames(&mut self, id: NodeId, name: &str) {
        self.session.call("designos_update_node", json!({ "node_id": id.0, "name": name })).expect("update_node");
    }

    /// The editor reads the log and runs what it finds, as every frame does.
    fn deliver(&mut self) {
        let actions = self.app.terminal.poll_live_agent_bridge();
        crate::shell::execute_terminal_actions(&mut self.app, actions);
    }

    fn name(&self, id: NodeId) -> String {
        self.app.doc.get(id).unwrap().name.clone()
    }

    fn x(&self, id: NodeId) -> f32 {
        self.app.doc.get(id).unwrap().transform.tx
    }

    /// The user is dragging `dragged` 50 px to the right; the document already shows it.
    fn start_drag(&mut self) {
        let mut tx = Tx::new();
        tx.ed.node(&mut self.app.doc, self.dragged).unwrap().transform.tx += 50.0;
        let state = MoveState {
            tx,
            ids: vec![self.dragged],
            start: Vec2::ZERO,
            orig_bbox: Rect::new(0.0, 0.0, 100.0, 100.0),
            targets: Vec::new(),
            moved: true,
            reorder: None,
        };
        self.app.canvas.inter = Interaction::Move(state);
    }

    /// The pointer is released (`commit`) or Escape is pressed.
    fn end_drag(&mut self, commit: bool) {
        let Interaction::Move(state) = std::mem::take(&mut self.app.canvas.inter) else { panic!("no drag in progress") };
        if commit {
            assert!(state.tx.commit(&mut self.app.doc, &mut self.app.hist).unwrap());
        } else {
            state.tx.cancel(&mut self.app.doc);
        }
    }

    fn undo(&mut self) {
        assert!(self.app.hist.undo(&mut self.app.doc).unwrap(), "nothing left to undo");
    }

    fn finish(mut self) {
        self.app.terminal.shutdown();
        let _ = std::fs::remove_dir_all(&self.dir);
        let _ = std::fs::remove_dir_all(&self.workspace);
    }
}

#[test]
fn an_agent_edit_made_during_a_drag_survives_escape() {
    let mut l = live();
    let (hero, footer) = (l.dragged, l.other);
    l.start_drag();
    l.agent_renames(hero, "Agent hero");
    l.deliver();
    assert_eq!(l.name(hero), "Hero", "the edit must wait for the drag to end");
    assert!(!l.app.hist.can_undo(), "no undo step may capture a half-dragged state");
    assert_eq!(l.app.terminal.live_patch.pending.len(), 1);

    l.end_drag(false);
    assert_eq!(l.x(hero), 0.0, "Escape puts the dragged node back");
    assert!(apply_pending(&mut l.app));
    assert_eq!(l.name(hero), "Agent hero", "Escape must not take the agent's edit with it");
    assert!(l.app.terminal.live_patch.pending.is_empty());
    assert_eq!(l.name(footer), "Footer");

    // Nothing is applied twice.
    l.deliver();
    assert!(!apply_pending(&mut l.app));
    l.undo();
    assert_eq!(l.name(hero), "Hero");
    l.finish();
}

#[test]
fn agent_edits_wait_for_the_drag_and_undo_in_the_order_they_happened() {
    let mut l = live();
    let (hero, footer) = (l.dragged, l.other);
    l.start_drag();
    l.agent_renames(footer, "First");
    l.agent_renames(footer, "Second");
    l.deliver();
    assert_eq!(l.name(footer), "Footer");

    l.end_drag(true);
    assert!(apply_pending(&mut l.app));
    assert_eq!((l.name(footer), l.x(hero)), ("Second".to_string(), 50.0));

    // Newest first: the agent's two calls, then the user's drag.
    l.undo();
    assert_eq!((l.name(footer), l.x(hero)), ("First".to_string(), 50.0));
    l.undo();
    assert_eq!((l.name(footer), l.x(hero)), ("Footer".to_string(), 50.0));
    l.undo();
    assert_eq!(l.x(hero), 0.0);
    l.finish();
}

#[test]
fn agent_edits_wait_while_a_text_session_is_open() {
    let mut l = live();
    let footer = l.other;
    l.app.canvas.text = Some(TextSession::new(l.dragged, 0, 0, Tx::new(), false));
    l.agent_renames(footer, "Agent footer");
    l.deliver();
    assert_eq!(l.name(footer), "Footer");

    l.app.canvas.text = None;
    assert!(apply_pending(&mut l.app));
    assert_eq!(l.name(footer), "Agent footer");
    l.finish();
}

#[test]
fn an_idle_canvas_applies_agent_edits_at_once() {
    let mut l = live();
    let hero = l.dragged;
    l.agent_renames(hero, "Agent hero");
    l.deliver();
    assert_eq!(l.name(hero), "Agent hero");
    assert!(l.app.terminal.live_patch.pending.is_empty());
    l.finish();
}

/// Adapter: the author's `apply_pending_live_patches` returns `()`; true when held patches were applied.
fn apply_pending(app: &mut App) -> bool {
    let had = !app.terminal.live_patch.pending.is_empty();
    crate::live_patch_apply::apply_pending_live_patches(app);
    had && app.terminal.live_patch.pending.is_empty()
}

// ---- reviewer probes ----

/// The agent switches documents while its earlier edit is held: the held edit must not land on the new document.
#[test]
fn probe_held_edit_does_not_land_on_a_document_the_agent_opened_later() {
    let mut l = live();
    let hero = l.dragged;
    l.start_drag();
    l.agent_renames(hero, "Agent hero");
    l.deliver();
    assert_eq!(l.app.terminal.live_patch.pending.len(), 1);
    let mut other = Document::new("Other");
    let op = other.add_page("Page");
    let theirs = other.alloc_id();
    other.insert(op, Node::frame(theirs, Rect::new(0.0, 0.0, 10.0, 10.0))).unwrap();
    other.get_mut(theirs).unwrap().name = "Fresh".into();
    std::fs::write(l.workspace.join("other.designos"), other.to_json().unwrap()).unwrap();
    l.session.call("designos_open_file", json!({ "path": "other.designos", "discard_changes": true })).expect("open_file");
    let fresh = theirs;
    std::thread::sleep(std::time::Duration::from_millis(20));
    l.deliver();
    println!("PROBE doc name after sync = {:?}, fresh id = {}, hero id = {}", l.app.doc.name, fresh.0, hero.0);
    l.app.canvas.inter = Interaction::None; // drag state belongs to the replaced doc; drop it
    apply_pending(&mut l.app);
    let names: Vec<String> = l.app.doc.nodes.values().map(|n| n.name.clone()).collect();
    println!("PROBE names after held patch applied = {names:?}");
    assert!(!names.iter().any(|n| n == "Agent hero"), "an edit for the old document landed on the new one");
    l.finish();
}

/// While an edit is held, the editor republishes its canvas without it; the agent's next call must still see its own edit.
#[test]
fn probe_agent_still_sees_its_held_edit_after_the_editor_republishes() {
    let mut l = live();
    let hero = l.dragged;
    l.start_drag();
    let made = l.session.call("designos_create_node", json!({ "kind": "frame", "name": "Card", "width": 10.0, "height": 10.0 })).expect("create");
    let card = made["node"]["id"].as_u64().unwrap();
    l.deliver();
    assert_eq!(l.app.terminal.live_patch.pending.len(), 1, "held during the drag");
    // The drag continues: the document changes and the editor republishes its canvas.
    std::thread::sleep(std::time::Duration::from_millis(20));
    if let Interaction::Move(s) = &mut l.app.canvas.inter {
        s.tx.ed.node(&mut l.app.doc, hero).unwrap().transform.tx += 5.0;
    }
    l.app.terminal.sync_live_file_context(&l.app.doc, l.app.page, &l.app.selection, None);
    let child = l.session.call("designos_create_node", json!({ "kind": "rectangle", "name": "Inside", "parent_id": card, "width": 5.0, "height": 5.0 }));
    println!("PROBE create child of held card -> {child:?}");
    assert!(child.is_ok(), "agent lost sight of its own held edit: {child:?}");
    l.finish();
}

/// Shape builder session open (between strokes): an agent move of one of its shapes must survive finishing the builder.
#[test]
fn probe_agent_move_survives_an_open_shape_builder() {
    let mut doc = Document::new("Live");
    let page = doc.add_page("Page");
    let a = doc.alloc_id();
    doc.insert(page, Node::rectangle(a, Rect::new(0.0, 0.0, 100.0, 100.0))).unwrap();
    let b = doc.alloc_id();
    doc.insert(page, Node::rectangle(b, Rect::new(50.0, 0.0, 100.0, 100.0))).unwrap();
    let mut app = App::new(doc);
    app.terminal = TerminalState::for_tab(unique_tab_id());
    let dir = app.terminal.session_dir.clone();
    let workspace = std::env::temp_dir().join(format!("designos-live-builder-ws-{}", unique_tab_id()));
    std::fs::create_dir_all(&workspace).unwrap();
    app.terminal.sync_live_file_context(&app.doc, app.page, &app.selection, None);
    let mut session = designos_mcp::server::Session::with_session_dir(&workspace, &dir).expect("mcp session");
    app.selection = vec![a, b];
    assert!(crate::canvas::builder::start(&mut app));
    session.call("designos_update_node", json!({ "node_id": a.0, "x": 500.0 })).expect("agent move");
    let actions = app.terminal.poll_live_agent_bridge();
    crate::shell::execute_terminal_actions(&mut app, actions);
    println!("PROBE pending={} a.x after agent move = {:?}", app.terminal.live_patch.pending.len(), app.doc.get(a).map(|n| n.transform.tx));
    // The user brushes the part of A that only A covers, then leaves the tool.
    crate::canvas::builder::press(&mut app, Vec2::new(25.0, 50.0), false);
    crate::canvas::builder::release(&mut app);
    crate::canvas::builder::finish(&mut app);
    apply_pending(&mut app);
    let xs: Vec<(String, f32)> = app.doc.nodes.values().filter(|n| n.parent == Some(page)).map(|n| (n.name.clone(), n.transform.tx)).collect();
    println!("PROBE layers after finish = {xs:?}");
    assert!(xs.iter().all(|(_, x)| *x < 400.0) == false, "agent's move of A to x=500 was lost by the builder");
    app.terminal.shutdown();
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&workspace);
}

/// No remap, no user edit: can the agent still undo its own last call once the editor applied it?
#[test]
fn probe_agent_undo_after_plain_echo() {
    let mut l = live();
    let made = l.session.call("designos_create_node", json!({ "kind": "frame", "name": "Card", "width": 10.0, "height": 10.0 })).expect("create");
    l.deliver();
    std::thread::sleep(std::time::Duration::from_millis(20));
    l.app.terminal.sync_live_file_context(&l.app.doc, l.app.page, &l.app.selection, None);
    let undone = l.session.call("designos_undo", json!({})).unwrap();
    println!("PROBE created={} undo -> {undone}", made["node"]["id"]);
    l.finish();
}
