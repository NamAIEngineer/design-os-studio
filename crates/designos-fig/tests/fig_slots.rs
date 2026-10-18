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

#[test]
fn real_preferred_values_object_round_trips_for_slot_and_instance_swap_and_keeps_legacy_guid_list() {
    use designos_fig::container;
    use designos_fig::kiwi::{KiwiValue, Schema};

    let real_fixture = json!([
        nc((0, 0), (0, 0), "!", "DOCUMENT", "Document", json!({})),
        nc((0, 1), (0, 0), "a", "CANVAS", "Page", json!({})),
        nc((0, 2), (0, 0), "b", "CANVAS", "Internal Only Canvas", json!({"internalOnly": true})),
        nc((6, 1), (0, 2), "a", "SYMBOL", "LibIcon", json!({"componentKey": "K_LIB_ICON", "sourceLibraryKey": "lk-icons"})),
        nc((6, 2), (0, 1), "a", "FRAME", "BadgeSet", json!({"isStateGroup": true, "componentKey": "K_BADGE_SET"})),
        nc((6, 3), (6, 2), "a", "SYMBOL", "Variant=Default", json!({"componentKey": "K_BADGE_DEF"})),
        nc(
            (5, 1),
            (0, 1),
            "b",
            "SYMBOL",
            "Panel",
            json!({
                "componentKey": "K_PANEL",
                "componentPropDefs": [
                    {
                        "id": g(7, 1),
                        "name": "LeadIcon",
                        "type": "INSTANCE_SWAP",
                        "initialValue": {"guidValue": g(6, 1)},
                        "preferredValues": {
                            "stringValues": [],
                            "instanceSwapValues": [
                                {"type": "COMPONENT", "key": "K_LIB_ICON"},
                                {"type": "STATE_GROUP", "key": "K_BADGE_SET"}
                            ],
                            "parameterValues": []
                        }
                    },
                    {
                        "id": g(7, 2),
                        "name": "Body",
                        "type": "SLOT",
                        "preferredValues": {
                            "stringValues": [],
                            "instanceSwapValues": [
                                {"type": "COMPONENT", "key": "K_LIB_ICON"},
                                {"type": "STATE_GROUP", "key": "K_BADGE_SET"}
                            ],
                            "parameterValues": []
                        },
                        "slotPropConfig": {"allowPreferredValuesOnly": true}
                    }
                ]
            })
        )
    ]);

    // 1. Import the real preferredValues object shape and resolve keys via library-key / componentKey lookup.
    let doc = import_fig_node_changes(&real_fixture, "t").unwrap();
    let lib_icon = find(&doc, "LibIcon").id;
    let badge_set = find(&doc, "BadgeSet").id;
    let panel = find(&doc, "Panel");

    let swap_prop = panel.component.props.iter().find(|p| p.name == "LeadIcon").expect("LeadIcon prop");
    assert_eq!(swap_prop.kind, PropKind::InstanceSwap { preferred: vec![lib_icon, badge_set] });
    assert_eq!(
        panel.component.slot_configs.get("Body"),
        Some(&SlotPropConfig { allow_preferred_values_only: Some(true), preferred: vec![lib_icon, badge_set], ..Default::default() })
    );

    // 2. Export to .fig and verify the binary Kiwi schema + message uses tag 8 ComponentPropPreferredValues object.
    let exported_bytes = export_fig(&doc).expect("export_fig");
    let archive = container::open(&exported_bytes).expect("open container");
    let (schema_bin, msg_bin) = container::read_archive_chunks(&archive).expect("read archive chunks");
    let schema = Schema::decode(&schema_bin).expect("decode schema");
    let cpd_def = schema.defs.iter().find(|d| d.name == "ComponentPropDef").expect("ComponentPropDef in schema");
    let pv_field = cpd_def.fields.iter().find(|f| f.name.as_ref() == "preferredValues").expect("preferredValues field");
    assert_eq!(pv_field.value, 8, "preferredValues tag is 8 in ComponentPropDef");
    assert!(!pv_field.is_array, "preferredValues is a single ComponentPropPreferredValues message, not an array");

    let root = schema.root_message().unwrap();
    let decoded_msg = schema.decode_message(root, &msg_bin).unwrap();
    let changes = match decoded_msg.get("nodeChanges") {
        Some(KiwiValue::Array(arr)) => arr,
        other => panic!("expected nodeChanges array, got {other:?}"),
    };
    let panel_nc = changes.iter().find(|nc| nc.get("name") == Some(&KiwiValue::String("Panel".into()))).expect("Panel NodeChange");
    let defs = match panel_nc.get("componentPropDefs") {
        Some(KiwiValue::Array(arr)) => arr,
        other => panic!("expected componentPropDefs array, got {other:?}"),
    };
    for def_val in defs {
        let pv = def_val.get("preferredValues").expect("preferredValues object on exported prop def");
        let swap_vals = match pv.get("instanceSwapValues") {
            Some(KiwiValue::Array(arr)) => arr,
            other => panic!("expected instanceSwapValues array inside preferredValues, got {other:?}"),
        };
        assert_eq!(swap_vals.len(), 2);
        assert_eq!(swap_vals[0].get("type"), Some(&KiwiValue::Enum("COMPONENT".into())));
        assert_eq!(swap_vals[0].get("key"), Some(&KiwiValue::String("K_LIB_ICON".into())));
        assert_eq!(swap_vals[1].get("type"), Some(&KiwiValue::Enum("STATE_GROUP".into())));
        assert_eq!(swap_vals[1].get("key"), Some(&KiwiValue::String("K_BADGE_SET".into())));
    }

    // 3. Re-import the exported .fig and verify round-trip fidelity of preferred values for both prop types.
    let rt = import_fig(&exported_bytes).expect("import_fig");
    let rt_icon = find(&rt, "LibIcon").id;
    let rt_set = find(&rt, "BadgeSet").id;
    let rt_panel = find(&rt, "Panel");
    let rt_swap = rt_panel.component.props.iter().find(|p| p.name == "LeadIcon").expect("LeadIcon after round-trip");
    assert_eq!(rt_swap.kind, PropKind::InstanceSwap { preferred: vec![rt_icon, rt_set] });
    assert_eq!(
        rt_panel.component.slot_configs.get("Body"),
        Some(&SlotPropConfig { allow_preferred_values_only: Some(true), preferred: vec![rt_icon, rt_set], ..Default::default() })
    );

    // 4. Legacy guid-list form still imports for both INSTANCE_SWAP and SLOT.
    let legacy_fixture = json!([
        nc((0, 0), (0, 0), "!", "DOCUMENT", "Document", json!({})),
        nc((0, 1), (0, 0), "a", "CANVAS", "Page", json!({})),
        nc((6, 1), (0, 1), "a", "SYMBOL", "Icon", json!({})),
        nc(
            (5, 1),
            (0, 1),
            "b",
            "SYMBOL",
            "LegacyPanel",
            json!({
                "componentPropDefs": [
                    {"id": g(7, 1), "name": "Swap", "type": "INSTANCE_SWAP", "initialValue": {"guidValue": g(6, 1)}, "preferredValues": [g(6, 1)]},
                    {"id": g(7, 2), "name": "Body", "type": "SLOT", "preferredValues": [g(6, 1)]}
                ]
            })
        )
    ]);
    let legacy_doc = import_fig_node_changes(&legacy_fixture, "t").unwrap();
    let legacy_icon = find(&legacy_doc, "Icon").id;
    let legacy_panel = find(&legacy_doc, "LegacyPanel");
    let legacy_swap = legacy_panel.component.props.iter().find(|p| p.name == "Swap").unwrap();
    assert_eq!(legacy_swap.kind, PropKind::InstanceSwap { preferred: vec![legacy_icon] });
    assert_eq!(legacy_panel.component.slot_configs.get("Body"), Some(&SlotPropConfig { preferred: vec![legacy_icon], ..Default::default() }));
}
