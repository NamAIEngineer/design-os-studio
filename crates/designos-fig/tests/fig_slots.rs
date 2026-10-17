//! Slots in `.fig`: the `isSlot` / `isSlotContent` flags, the SLOT component property with its
//! `SlotPropConfig` and preferred values, the `SLOT_CONTENT_ID` binding, and the instance
//! `slot-content-id`. Fixtures are hand-written; nothing here comes from a real file with slots.

use designos_doc::*;
use designos_fig::{export_fig, import_fig, import_fig_node_changes};
use serde_json::{json, Value};

fn find<'a>(doc: &'a Document, name: &str) -> &'a Node {
    doc.nodes.values().find(|n| n.name == name).unwrap_or_else(|| panic!("no node named {name}"))
}

fn full_config(preferred: Vec<NodeId>) -> SlotPropConfig {
    SlotPropConfig {
        stretch_child_on_insert: Some(true),
        display_by_default: Some(false),
        min_children: Some(1),
        max_children: Some(3),
        allow_preferred_values_only: Some(true),
        preferred,
    }
}

/// A `Panel` component with a slot property, a slot frame bound to it, one preferred component,
/// content flagged as slot content, and an instance that fills the slot.
fn slot_document() -> Document {
    let mut doc = Document::new("slots");
    let page = doc.add_page("Page");

    let icon_id = doc.alloc_id();
    let mut icon = Node::new(icon_id, NodeKind::Component(FrameProps::default()));
    icon.name = "Icon".into();
    icon.size = Vec2::new(16.0, 16.0);
    doc.insert(page, icon).unwrap();

    let panel_id = doc.alloc_id();
    let mut panel = Node::new(panel_id, NodeKind::Component(FrameProps::default()));
    panel.name = "Panel".into();
    panel.size = Vec2::new(320.0, 200.0);
    panel.component.description = "A panel with a body slot".into();
    panel.component.expose_nested_instances = true;
    panel.fig_metadata.insert("userFacingVersion".into(), "126".into());
    panel.component.props = vec![
        ComponentPropDef { name: "Dense#1:0".into(), kind: PropKind::Boolean, default: PropValue::Bool(true) },
        ComponentPropDef { name: "Body#1:1".into(), kind: PropKind::Slot, default: PropValue::Slot(vec![]) },
    ];
    panel.component.slot_configs.insert("Body#1:1".into(), full_config(vec![icon_id]));
    doc.insert(page, panel).unwrap();

    let body_id = doc.alloc_id();
    let mut body = Node::new(body_id, NodeKind::Frame(FrameProps::default()));
    body.name = "BodySlot".into();
    body.size = Vec2::new(280.0, 120.0);
    body.component.bindings.insert(OverrideField::SlotContent, "Body#1:1".into());
    body.component.is_slot = true;
    doc.insert(panel_id, body).unwrap();

    let content_id = doc.alloc_id();
    let mut content = Node::new(content_id, NodeKind::Rectangle);
    content.name = "Banner".into();
    content.size = Vec2::new(260.0, 60.0);
    content.component.is_slot_content = true;
    doc.insert(page, content).unwrap();

    create_instance(&mut doc, panel_id, page, Vec2::new(400.0, 0.0)).apply(&mut doc).expect("create_instance");
    let inst_id = instances_of(&doc, panel_id)[0];
    let inst = doc.get_mut(inst_id).unwrap();
    inst.name = "PanelInstance".into();
    inst.component.values.insert("Body#1:1".into(), PropValue::Slot(vec![content_id]));
    inst.component.slot_content_id = Some("7:42".into());
    doc
}

#[test]
fn slot_flags_config_and_instance_content_survive_a_fig_round_trip() {
    let doc = slot_document();
    let rt = import_fig(&export_fig(&doc).expect("export_fig")).expect("import_fig");

    let icon = find(&rt, "Icon");
    let panel = find(&rt, "Panel");
    assert_eq!(panel.component.slot_configs.len(), 1);
    assert_eq!(panel.component.slot_configs.get("Body"), Some(&full_config(vec![icon.id])));
    assert!(panel.component.props.iter().any(|p| p.name == "Body" && p.kind == PropKind::Slot));

    let body = find(&rt, "BodySlot");
    assert!(body.component.is_slot, "isSlot");
    assert!(!body.component.is_slot_content);
    assert_eq!(body.component.bindings.get(&OverrideField::SlotContent).map(String::as_str), Some("Body"));

    let banner = find(&rt, "Banner");
    assert!(banner.component.is_slot_content, "isSlotContent");
    assert!(!banner.component.is_slot);

    let inst = find(&rt, "PanelInstance");
    assert_eq!(inst.component.values.get("Body"), Some(&PropValue::Slot(vec![banner.id])), "instance keeps its slot content");
    assert_eq!(inst.component.slot_content_id.as_deref(), Some("7:42"));

    assert!(!icon.component.is_slot && !icon.component.is_slot_content && icon.component.slot_content_id.is_none());
}

#[test]
fn only_the_slot_fields_the_file_wrote_come_back() {
    let mut doc = slot_document();
    let panel_id = find(&doc, "Panel").id;
    let panel = doc.get_mut(panel_id).unwrap();
    // minChildren written as 0 stays 0; the other four fields were never written.
    panel.component.slot_configs.insert("Body#1:1".into(), SlotPropConfig { min_children: Some(0), ..Default::default() });
    let rt = import_fig(&export_fig(&doc).unwrap()).unwrap();
    assert_eq!(find(&rt, "Panel").component.slot_configs.get("Body"), Some(&SlotPropConfig { min_children: Some(0), ..Default::default() }));

    // A slot property with no settings at all gets no settings entry.
    let panel = doc.get_mut(panel_id).unwrap();
    panel.component.slot_configs.clear();
    let rt = import_fig(&export_fig(&doc).unwrap()).unwrap();
    assert!(find(&rt, "Panel").component.slot_configs.is_empty());
}

#[test]
fn data_beside_the_slots_is_not_disturbed() {
    let doc = slot_document();
    let rt = import_fig(&export_fig(&doc).unwrap()).unwrap();
    let panel = find(&rt, "Panel");
    assert_eq!(panel.component.description, "A panel with a body slot");
    assert!(panel.component.expose_nested_instances);
    assert_eq!(panel.fig_metadata.get("userFacingVersion").map(String::as_str), Some("126"));
    let dense = panel.component.props.iter().find(|p| p.name == "Dense").expect("Dense prop");
    assert_eq!((dense.kind.clone(), dense.default.clone()), (PropKind::Boolean, PropValue::Bool(true)));
    assert_eq!(panel.component.props.len(), 2);
}

#[test]
fn a_second_round_trip_changes_nothing() {
    let once = import_fig(&export_fig(&slot_document()).unwrap()).unwrap();
    let twice = import_fig(&export_fig(&once).unwrap()).unwrap();
    for name in ["Panel", "BodySlot", "Banner", "PanelInstance"] {
        assert_eq!(find(&once, name).component.slot_configs, find(&twice, name).component.slot_configs, "{name}");
        assert_eq!(find(&once, name).component.is_slot, find(&twice, name).component.is_slot, "{name}");
        assert_eq!(find(&once, name).component.is_slot_content, find(&twice, name).component.is_slot_content, "{name}");
        assert_eq!(find(&once, name).component.slot_content_id, find(&twice, name).component.slot_content_id, "{name}");
    }
}

#[test]
fn a_renamed_slot_property_keeps_its_settings_in_the_exported_file() {
    let mut doc = slot_document();
    let panel_id = find(&doc, "Panel").id;
    rename_prop(&mut doc, panel_id, "Body#1:1", "Content#1:1").apply(&mut doc).unwrap();
    let rt = import_fig(&export_fig(&doc).unwrap()).unwrap();
    let icon = find(&rt, "Icon").id;
    let panel = find(&rt, "Panel");
    assert_eq!(panel.component.slot_configs.get("Content"), Some(&full_config(vec![icon])));
    assert!(!panel.component.slot_configs.contains_key("Body"));
    assert_eq!(find(&rt, "BodySlot").component.bindings.get(&OverrideField::SlotContent).map(String::as_str), Some("Content"));
}

fn g(s: u64, l: u64) -> Value {
    json!({"sessionID": s, "localID": l})
}

fn nc(id: (u64, u64), parent: (u64, u64), pos: &str, ty: &str, name: &str, extra: Value) -> Value {
    let mut v = json!({
        "guid": g(id.0, id.1), "parentIndex": {"guid": g(parent.0, parent.1), "position": pos},
        "type": ty, "name": name, "phase": "CREATED", "visible": true, "opacity": 1.0,
        "size": {"x": 10.0, "y": 10.0},
        "transform": {"m00": 1.0, "m01": 0.0, "m02": 0.0, "m10": 0.0, "m11": 1.0, "m12": 0.0}
    });
    for (k, x) in extra.as_object().unwrap() {
        v[k] = x.clone();
    }
    v
}

/// Kiwi-shaped node changes with the field names of the Figma 126 schema.
fn kiwi_fixture() -> Value {
    json!([
        nc((0, 0), (0, 0), "!", "DOCUMENT", "Document", json!({})),
        nc((0, 1), (0, 0), "a", "CANVAS", "Page", json!({})),
        nc(
            (5, 1),
            (0, 1),
            "a",
            "SYMBOL",
            "Panel",
            json!({
            "size": {"x": 200.0, "y": 100.0},
            "componentPropDefs": [{"id": g(7, 1), "name": "Body", "type": "SLOT",
                "slotPropConfig": {"stretchChildOnInsert": true, "displayByDefault": false,
                    "minChildren": 1, "maxChildren": 4, "allowPreferredValuesOnly": true}}]})
        ),
        nc(
            (5, 2),
            (5, 1),
            "a",
            "FRAME",
            "Body slot",
            json!({
            "isSlot": true,
            "componentPropRefs": [{"componentPropNodeField": "SLOT_CONTENT_ID", "defID": g(7, 1)}]})
        ),
        nc((5, 3), (5, 1), "b", "FRAME", "Header", json!({})),
        nc((1, 1), (0, 1), "b", "FRAME", "Card", json!({"isSlotContent": true})),
        nc(
            (1, 2),
            (0, 1),
            "c",
            "INSTANCE",
            "Panel copy",
            json!({
            "symbolData": {"symbolID": g(5, 1), "symbolOverrides": [], "uniformScaleFactor": 1.0},
            "jsonSerialized": {"slot-content-id": {"guid": g(9, 9)}}})
        )
    ])
}

#[test]
fn imports_slot_fields_from_a_kiwi_shaped_fixture() {
    let doc = import_fig_node_changes(&kiwi_fixture(), "t").unwrap();
    let panel = find(&doc, "Panel");
    assert_eq!(
        panel.component.slot_configs.get("Body"),
        Some(&SlotPropConfig {
            stretch_child_on_insert: Some(true),
            display_by_default: Some(false),
            min_children: Some(1),
            max_children: Some(4),
            allow_preferred_values_only: Some(true),
            preferred: vec![],
        })
    );
    let slot = find(&doc, "Body slot");
    assert!(slot.component.is_slot);
    assert_eq!(slot.component.bindings.get(&OverrideField::SlotContent).map(String::as_str), Some("Body"));
    assert!(!find(&doc, "Header").component.is_slot);
    assert!(find(&doc, "Card").component.is_slot_content);
    assert_eq!(find(&doc, "Panel copy").component.slot_content_id.as_deref(), Some("9:9"));
    assert!(find(&doc, "Page").component.slot_content_id.is_none());
}

/// A slot whose file lists preferred components but never wrote a `slotPropConfig`.
fn preferred_without_config_fixture() -> Value {
    json!([
        nc((0, 0), (0, 0), "!", "DOCUMENT", "Document", json!({})),
        nc((0, 1), (0, 0), "a", "CANVAS", "Page", json!({})),
        nc((6, 1), (0, 1), "a", "SYMBOL", "Icon", json!({})),
        nc(
            (5, 1),
            (0, 1),
            "b",
            "SYMBOL",
            "Panel",
            json!({
            "componentPropDefs": [{"id": g(7, 1), "name": "Body", "type": "SLOT", "preferredValues": [g(6, 1)]}]})
        )
    ])
}

#[test]
fn preferred_components_are_kept_when_the_file_wrote_no_slot_config() {
    let doc = import_fig_node_changes(&preferred_without_config_fixture(), "t").unwrap();
    let icon = find(&doc, "Icon").id;
    let cfg = find(&doc, "Panel").component.slot_configs.get("Body").cloned().expect("preferred values are kept");
    assert_eq!(cfg, SlotPropConfig { preferred: vec![icon], ..Default::default() });
}

#[test]
fn preferred_components_without_a_slot_config_survive_a_fig_round_trip() {
    let doc = import_fig_node_changes(&preferred_without_config_fixture(), "t").unwrap();
    let rt = import_fig(&export_fig(&doc).unwrap()).unwrap();
    let icon = find(&rt, "Icon").id;
    assert_eq!(find(&rt, "Panel").component.slot_configs.get("Body"), Some(&SlotPropConfig { preferred: vec![icon], ..Default::default() }));
}

/// Verifies the embedded Kiwi schema tags (`isSlot=463`, `isSlotContent=495`, `SLOT=7`) and reports
/// the actual slot record counts in the owner's `VSF - PCP.fig` when present on disk; skipped when
/// the private file is absent (such as on CI). Per #160 AC #2, no synthetic slot fixture is invented
/// when the file carries zero slot records.
#[test]
fn vsf_pcp_fig_slot_scan_skips_when_absent_and_verifies_schema_and_counts_when_present() {
    use designos_fig::container;
    use designos_fig::kiwi::{KiwiValue, Schema};

    let candidates = ["/Users/jang/Downloads/VSF - PCP.fig", "/Users/jangtrinh/Downloads/VSF - PCP.fig", "/Users/jangtrinh/Downloads/VSF - PCP (1).fig"];
    let Some((path, bytes)) = candidates.iter().find_map(|p| std::fs::read(p).ok().map(|b| (*p, b))) else {
        eprintln!("SKIPPED: owner's VSF - PCP.fig is not present on this machine");
        return;
    };

    let archive = container::open(&bytes).expect("open VSF - PCP.fig container");
    let (schema_bin, msg_bin) = container::read_archive_chunks(&archive).expect("read archive chunks");
    let schema = Schema::decode(&schema_bin).expect("decode embedded Kiwi schema");

    let nc_def = schema.defs.iter().find(|d| d.name == "NodeChange").expect("NodeChange definition");
    let is_slot_tag = nc_def.fields.iter().find(|f| f.name.as_ref() == "isSlot").map(|f| f.value);
    let is_slot_content_tag = nc_def.fields.iter().find(|f| f.name.as_ref() == "isSlotContent").map(|f| f.value);
    assert_eq!(is_slot_tag, Some(463), "NodeChange.isSlot field tag in VSF - PCP.fig");
    assert_eq!(is_slot_content_tag, Some(495), "NodeChange.isSlotContent field tag in VSF - PCP.fig");

    let cpt_def = schema.defs.iter().find(|d| d.name == "ComponentPropType").expect("ComponentPropType definition");
    let slot_tag = cpt_def.fields.iter().find(|f| f.name.as_ref() == "SLOT").map(|f| f.value);
    assert_eq!(slot_tag, Some(7), "ComponentPropType.SLOT enum value in VSF - PCP.fig");

    let root = schema.root_message().expect("root message");
    let msg = schema.decode_message(root, &msg_bin).expect("decode message");
    let mut total_nodes = 0usize;
    let mut is_slot_true = 0usize;
    let mut is_slot_present = 0usize;
    let mut is_slot_content_true = 0usize;
    let mut is_slot_content_present = 0usize;
    let mut slot_props = 0usize;
    let mut slot_prop_configs = 0usize;
    let mut slot_prop_refs = 0usize;
    let mut slot_content_id_assigns = 0usize;
    if let Some(KiwiValue::Array(changes)) = msg.get("nodeChanges") {
        total_nodes = changes.len();
        for nc in changes {
            if let Some(v) = nc.get("isSlot") {
                is_slot_present += 1;
                if v == &KiwiValue::Bool(true) {
                    is_slot_true += 1;
                }
            }
            if let Some(v) = nc.get("isSlotContent") {
                is_slot_content_present += 1;
                if v == &KiwiValue::Bool(true) {
                    is_slot_content_true += 1;
                }
            }
            if let Some(KiwiValue::Array(defs)) = nc.get("componentPropDefs") {
                for d in defs {
                    if d.get("type") == Some(&KiwiValue::Enum("SLOT".into())) {
                        slot_props += 1;
                        if d.get("slotPropConfig").is_some() {
                            slot_prop_configs += 1;
                        }
                    }
                }
            }
            if let Some(KiwiValue::Array(refs)) = nc.get("componentPropRefs") {
                for r in refs {
                    if r.get("componentPropNodeField") == Some(&KiwiValue::Enum("SLOT_CONTENT_ID".into())) {
                        slot_prop_refs += 1;
                    }
                }
            }
            if let Some(KiwiValue::Array(assigns)) = nc.get("componentPropAssignments") {
                for a in assigns {
                    if let Some(vv) = a.get("varValue") {
                        if vv.get("value").and_then(|v| v.get("slotContentIdValue")).is_some() {
                            slot_content_id_assigns += 1;
                        }
                    }
                }
            }
        }
    }
    eprintln!(
        "VSF - PCP.fig ({path}): total_nodeChanges={total_nodes}, isSlot(true={is_slot_true}, present={is_slot_present}), isSlotContent(true={is_slot_content_true}, present={is_slot_content_present}), SLOT_props={slot_props}, slotPropConfigs={slot_prop_configs}, slotPropRefs={slot_prop_refs}, slotContentIdValue_assigns={slot_content_id_assigns}"
    );
    assert!(total_nodes > 0);
}
