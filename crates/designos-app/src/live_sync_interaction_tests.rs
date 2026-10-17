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
    assert_eq!(l.app.terminal.pending_live_patches.len(), 1);

    l.end_drag(false);
    assert_eq!(l.x(hero), 0.0, "Escape puts the dragged node back");
    assert!(crate::live_patch_apply::apply_pending_live_patches(&mut l.app));
    assert_eq!(l.name(hero), "Agent hero", "Escape must not take the agent's edit with it");
    assert!(l.app.terminal.pending_live_patches.is_empty());
    assert_eq!(l.name(footer), "Footer");

    // Nothing is applied twice.
    l.deliver();
    assert!(!crate::live_patch_apply::apply_pending_live_patches(&mut l.app));
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
    assert!(crate::live_patch_apply::apply_pending_live_patches(&mut l.app));
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
    assert!(crate::live_patch_apply::apply_pending_live_patches(&mut l.app));
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
    assert!(l.app.terminal.pending_live_patches.is_empty());
    l.finish();
}
