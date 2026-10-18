//! Slots in `.fig`: the `isSlot` / `isSlotContent` flags, the SLOT component property with its
//! `SlotPropConfig` and preferred values, the `SLOT_CONTENT_ID` binding, and the instance
//! `slot-content-id`. Fixtures are hand-written and content-free; the last tests scan the owner's
//! private `VSF - PCP.fig` (never committed, skipped when absent) and assert only counts.

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

/// Slot record counts of one decoded Kiwi message.
#[derive(Debug, Default, PartialEq, Eq)]
struct SlotCounts {
    node_changes: usize,
    /// `isSlot` written as `true` (field 463) / written at all.
    is_slot_true: usize,
    is_slot_present: usize,
    /// `isSlotContent` written as `true` (field 495) / written at all.
    is_slot_content_true: usize,
    is_slot_content_present: usize,
    /// Control: every `componentPropDefs` entry of any type, so a zero slot result is known to come
    /// from a scan that does read component properties.
    prop_defs: usize,
    /// `componentPropDefs` of type `SLOT`, and those carrying a `slotPropConfig`.
    slot_props: usize,
    slot_prop_configs: usize,
    /// `componentPropRefs` binding `SLOT_CONTENT_ID`.
    slot_content_id_refs: usize,
    /// `componentPropAssignments` whose `defID` names a `SLOT` property definition (the path
    /// `fig.rs` reads and `fig_export.rs` writes).
    slot_assignments: usize,
    /// Nodes holding `jsonSerialized` -> `slot-content-id` (the instance slot content guid).
    slot_content_id_bags: usize,
}

/// `sessionID` / `localID` pair of a Kiwi `GUID` struct.
fn guid_key(v: &designos_fig::kiwi::KiwiValue) -> Option<(u64, u64)> {
    use designos_fig::kiwi::KiwiValue::{Uint, Uint64};
    let n = |f: &str| match v.get(f)? {
        Uint(x) => Some(u64::from(*x)),
        Uint64(x) => Some(*x),
        _ => None,
    };
    Some((n("sessionID")?, n("localID")?))
}

fn scan_slots(msg: &designos_fig::kiwi::KiwiValue) -> SlotCounts {
    use designos_fig::kiwi::KiwiValue;
    use std::collections::HashSet;
    let slot_type = KiwiValue::Enum("SLOT".into());
    let changes = msg.get("nodeChanges").and_then(KiwiValue::as_array).unwrap_or(&[]);
    let mut c = SlotCounts { node_changes: changes.len(), ..Default::default() };
    let defs_of = |nc: &KiwiValue| nc.get("componentPropDefs").and_then(KiwiValue::as_array).unwrap_or(&[]).to_vec();
    // Definitions live on the component; assignments on its instances, so collect the ids first.
    let mut slot_def_ids = HashSet::new();
    for nc in changes {
        for d in defs_of(nc) {
            c.prop_defs += 1;
            if d.get("type") == Some(&slot_type) {
                c.slot_props += 1;
                c.slot_prop_configs += usize::from(d.get("slotPropConfig").is_some());
                slot_def_ids.extend(d.get("id").and_then(guid_key));
            }
        }
    }
    for nc in changes {
        if let Some(v) = nc.get("isSlot") {
            c.is_slot_present += 1;
            c.is_slot_true += usize::from(v == &KiwiValue::Bool(true));
        }
        if let Some(v) = nc.get("isSlotContent") {
            c.is_slot_content_present += 1;
            c.is_slot_content_true += usize::from(v == &KiwiValue::Bool(true));
        }
        for r in nc.get("componentPropRefs").and_then(KiwiValue::as_array).unwrap_or(&[]) {
            c.slot_content_id_refs += usize::from(r.get("componentPropNodeField") == Some(&KiwiValue::Enum("SLOT_CONTENT_ID".into())));
        }
        for a in nc.get("componentPropAssignments").and_then(KiwiValue::as_array).unwrap_or(&[]) {
            c.slot_assignments += usize::from(a.get("defID").and_then(guid_key).is_some_and(|id| slot_def_ids.contains(&id)));
        }
        c.slot_content_id_bags += usize::from(nc.get("jsonSerialized").and_then(|j| j.get("slot-content-id")).and_then(|s| s.get("guid")).is_some());
    }
    c
}

/// Opens a `.fig` and decodes its root message the way import does (drops the multi-gigabyte
/// derived fields on large files). Returns the embedded schema next to the message.
fn decode_fig_for_scan(bytes: &[u8]) -> (designos_fig::kiwi::Schema, designos_fig::kiwi::KiwiValue) {
    use designos_fig::{container, kiwi::Schema};
    let archive = container::open(bytes).expect("open .fig container");
    let (schema_bin, msg_bin) = container::read_archive_chunks(&archive).expect("read archive chunks");
    let schema = Schema::decode(&schema_bin).expect("decode embedded Kiwi schema");
    let root = schema.root_message().expect("root message");
    let msg = schema.decode_message_for_import(root, &msg_bin).expect("decode message");
    (schema, msg)
}

/// Field tag of `def.field` in a decoded schema.
fn schema_tag(schema: &designos_fig::kiwi::Schema, def: &str, field: &str) -> Option<u32> {
    let d = schema.defs.iter().find(|d| d.name == def)?;
    d.fields.iter().find(|f| f.name.as_ref() == field).map(|f| f.value)
}

#[test]
fn the_slot_scan_finds_every_slot_record_a_fig_export_writes() {
    let (schema, msg) = decode_fig_for_scan(&export_fig(&slot_document()).expect("export_fig"));
    // The testkit schema numbers its own fields; only the names are shared with a real file.
    assert!(schema_tag(&schema, "NodeChange", "isSlot").is_some() && schema_tag(&schema, "NodeChange", "isSlotContent").is_some());
    let c = scan_slots(&msg);
    // Two slot frames: the component's own and its instance's copy, both bound to the property.
    let expected = SlotCounts {
        node_changes: 8,
        is_slot_true: 2,
        is_slot_present: 2,
        is_slot_content_true: 1,
        is_slot_content_present: 1,
        prop_defs: 2,
        slot_props: 1,
        slot_prop_configs: 1,
        slot_content_id_refs: 2,
        slot_assignments: 1,
        slot_content_id_bags: 1,
    };
    assert_eq!(c, expected, "a scan that cannot see slots would make a zero result meaningless");
}

/// Where the owner's private `VSF - PCP.fig` lives: `DESIGNOS_VSF_PCP`, else `$HOME/Downloads`.
fn vsf_pcp_path() -> Option<std::path::PathBuf> {
    if let Some(p) = std::env::var_os("DESIGNOS_VSF_PCP") {
        let p = std::path::PathBuf::from(p);
        assert!(p.is_file(), "DESIGNOS_VSF_PCP is set but {} is not a file", p.display());
        return Some(p);
    }
    let p = std::path::PathBuf::from(std::env::var_os("HOME")?).join("Downloads").join("VSF - PCP.fig");
    p.is_file().then_some(p)
}

/// #160 AC1/AC2 on the owner's `VSF - PCP.fig` (209 MB export of 2026-10-14; the 124 MB
/// `VSF - PCP (1).fig` measures the same slot counts). Measured: no slot record of any kind in
/// 155,297 node changes, although the file carries 4,481 other component property definitions.
/// Zero is the AC2 outcome: the file is not a slot reference and no fixture is derived from it
/// (the hand-written slot fixtures above stay the only slot data). If a newer export of the file
/// carries slots this fails, which is the cue to derive a content-free fixture of that shape.
/// Skipped when the file is absent (CI). Only counts are asserted; nothing from the file is committed.
#[test]
fn vsf_pcp_fig_slot_counts_are_the_measured_ones() {
    let Some(path) = vsf_pcp_path() else {
        eprintln!("SKIPPED: VSF - PCP.fig not found (set DESIGNOS_VSF_PCP or place it in ~/Downloads)");
        return;
    };
    let bytes = std::fs::read(&path).expect("read VSF - PCP.fig");
    let (schema, msg) = decode_fig_for_scan(&bytes);
    assert_eq!(schema_tag(&schema, "NodeChange", "isSlot"), Some(463));
    assert_eq!(schema_tag(&schema, "NodeChange", "isSlotContent"), Some(495));
    assert_eq!(schema_tag(&schema, "ComponentPropType", "SLOT"), Some(7));
    let counts = scan_slots(&msg);
    eprintln!("VSF - PCP.fig slot counts: {counts:?}");
    let measured = SlotCounts { node_changes: 155_297, prop_defs: 4_481, ..Default::default() };
    assert_eq!(counts, measured, "VSF - PCP.fig changed; re-measure and apply #160 AC2/AC3");
}
