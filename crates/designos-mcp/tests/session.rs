use designos_mcp::server::Session;
use serde_json::json;
use std::{fs, path::PathBuf};

struct Workspace(PathBuf);
impl Workspace {
    fn new() -> Self {
        let nonce = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
        let p = std::env::temp_dir().join(format!("designos-mcp-{}-{nonce}", std::process::id()));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
}
impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn saved_file_roundtrip_and_unsaved_changes_guard() {
    let dir = Workspace::new();
    let mut s = Session::new(&dir.0).unwrap();
    let d = s.call("designos_get_document", json!({})).unwrap();
    let page = d["pages"][0]["node_id"].clone();
    let node = s.call("designos_create_node", json!({"parent_id":page,"kind":"text","name":"Heading","text":"Hello MCP"})).unwrap();
    assert!(s.call("designos_new_document", json!({"name":"Oops"})).is_err());
    s.call("designos_save_file", json!({"path":"first.designos"})).unwrap();
    s.call("designos_update_node", json!({"node_id":node["node"]["id"],"text":"Changed"})).unwrap();
    assert!(s.call("designos_open_file", json!({"path":"first.designos"})).is_err());
    s.call("designos_open_file", json!({"path":"first.designos","discard_changes":true})).unwrap();
    assert_eq!(s.call("designos_get_node", json!({"node_id":node["node"]["id"]})).unwrap()["node"], node["node"]);
    let original = fs::read(dir.0.join("first.designos")).unwrap();
    assert!(s.call("designos_save_file", json!({"path":"first.designos"})).is_err());
    assert_eq!(fs::read(dir.0.join("first.designos")).unwrap(), original);
    s.call("designos_save_file", json!({"path":"second.designos"})).unwrap();
    assert!(dir.0.join("second.designos").exists());
    let rendered = s.call("designos_render_png", json!({"path":"preview.png","node_id":node["node"]["id"],"scale":2.0})).unwrap();
    assert_eq!(rendered["rendered"], true);
    assert!(dir.0.join("preview.png").exists());
    let fig_exported = s.call("designos_export_fig_file", json!({"output_path":"exported.fig"})).unwrap();
    assert_eq!(fig_exported["saved"], true);
    assert!(fig_exported["bytes"].as_u64().unwrap() > 16);
    assert!(dir.0.join("exported.fig").exists());
}

#[test]
fn paths_and_bad_imports_cannot_replace_the_session() {
    let dir = Workspace::new();
    let outside = Workspace::new();
    fs::write(outside.0.join("outside.designos"), "{}").unwrap();
    let mut s = Session::new(&dir.0).unwrap();
    let before = s.call("designos_get_document", json!({})).unwrap();
    assert!(s.call("designos_open_file", json!({"path":outside.0.join("outside.designos")})).is_err());
    assert!(s.call("designos_save_file", json!({"path":outside.0.join("escape.designos")})).is_err());
    assert!(s.call("designos_save_file", json!({"path":"../escape.designos"})).is_err());
    assert!(s.call("designos_save_file", json!({"path":"bad.json"})).is_err());
    fs::write(dir.0.join("bad.designos"), "not JSON").unwrap();
    assert!(s.call("designos_open_file", json!({"path":"bad.designos"})).is_err());
    let mut broken = s.engine.doc.clone();
    let id = broken.pages[0];
    broken.get_mut(id).unwrap().parent = Some(id);
    fs::write(dir.0.join("cycle.designos"), broken.to_json().unwrap()).unwrap();
    assert!(s.call("designos_open_file", json!({"path":"cycle.designos"})).is_err());
    assert_eq!(s.call("designos_get_document", json!({})).unwrap(), before);
    assert!(!outside.0.join("escape.designos").exists());
}

#[test]
fn imported_stale_id_counter_does_not_overwrite_existing_nodes() {
    let dir = Workspace::new();
    let mut s = Session::new(&dir.0).unwrap();
    let mut native = serde_json::to_value(&s.engine.doc).unwrap();
    native["next_id"] = json!(0);
    fs::write(dir.0.join("stale.designos"), native.to_string()).unwrap();
    let doc = s.call("designos_open_file", json!({"path":"stale.designos"})).unwrap();
    let page = doc["pages"][0]["node_id"].clone();
    let created = s.call("designos_create_node", json!({"parent_id":page,"kind":"rectangle"})).unwrap();
    assert_ne!(created["node"]["id"], page);
    assert_eq!(s.call("designos_get_document", json!({})).unwrap()["node_count"], 2);
    s.call("designos_delete_node", json!({"node_id":created["node"]["id"]})).unwrap();
    let next = s.call("designos_create_node", json!({"parent_id":page,"kind":"rectangle"})).unwrap();
    assert!(next["node"]["id"].as_u64().unwrap() > created["node"]["id"].as_u64().unwrap());

    let mut exhausted = serde_json::to_value(&s.engine.doc).unwrap();
    exhausted["next_id"] = json!(u64::MAX);
    fs::write(dir.0.join("exhausted.designos"), exhausted.to_string()).unwrap();
    s.call("designos_open_file", json!({"path":"exhausted.designos","discard_changes":true})).unwrap();
    let before = s.engine.doc.to_json().unwrap();
    assert!(s.call("designos_create_node", json!({"parent_id":page,"kind":"rectangle"})).unwrap_err().contains("exhausted"));
    assert_eq!(s.engine.doc.to_json().unwrap(), before);
}
