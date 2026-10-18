//! Slot data on [`ComponentLink`]: compact serialization, instance copies and garbage collection.

use crate::{
    create_component, create_instance, delete_prop, instances_of, rename_prop, sync_instances, ComponentPropDef, Document, Node, NodeId, NodeKind, PropKind,
    PropValue, Rect, SlotPropConfig, Vec2,
};

/// A component frame holding a slot frame, with a slot property, its settings and a preferred component.
fn component_with_slot() -> (Document, NodeId, NodeId, NodeId) {
    let mut d = Document::new("slots");
    let page = d.add_page("P");
    let id = d.alloc_id();
    let icon = d.insert(page, Node::frame(id, Rect::new(0.0, 0.0, 16.0, 16.0))).unwrap();
    create_component(&mut d, icon).apply(&mut d).unwrap();

    let id = d.alloc_id();
    let comp = d.insert(page, Node::frame(id, Rect::new(0.0, 100.0, 200.0, 100.0))).unwrap();
    let id = d.alloc_id();
    let mut body = Node::frame(id, Rect::new(0.0, 0.0, 200.0, 80.0));
    body.name = "body".into();
    let body = d.insert(comp, body).unwrap();
    create_component(&mut d, comp).apply(&mut d).unwrap();

    let node = d.get_mut(comp).unwrap();
    node.component.props.push(ComponentPropDef { name: "Body".into(), kind: PropKind::Slot, default: PropValue::Slot(vec![]) });
    node.component.slot_configs.insert("Body".into(), SlotPropConfig { max_children: Some(2), preferred: vec![icon], ..Default::default() });
    (d, comp, body, icon)
}

#[test]
fn a_node_without_slot_data_serializes_none_of_it() {
    let (d, ..) = component_with_slot();
    let json = d.to_json().unwrap();
    assert_eq!(json.matches("slot_configs").count(), 1, "only the component that has settings writes the key");
    assert!(!json.contains("is_slot"), "unset flags are not written");
    assert!(!json.contains("slot_content_id"));
}

#[test]
fn slot_data_survives_the_native_json_round_trip() {
    let (mut d, comp, body, _) = component_with_slot();
    d.get_mut(body).unwrap().component.is_slot = true;
    d.get_mut(comp).unwrap().component.slot_content_id = Some("7:42".into());
    let loaded = Document::from_json(&d.to_json().unwrap()).unwrap();
    assert_eq!(loaded.get(comp).unwrap().component.slot_configs, d.get(comp).unwrap().component.slot_configs);
    assert!(loaded.get(body).unwrap().component.is_slot);
    assert_eq!(loaded.get(comp).unwrap().component.slot_content_id.as_deref(), Some("7:42"));
}

#[test]
fn an_instance_gets_its_own_slot_frame_flag_but_not_the_components_settings() {
    let (mut d, comp, body, _) = component_with_slot();
    d.get_mut(body).unwrap().component.is_slot = true;
    let page = d.pages[0];
    create_instance(&mut d, comp, page, Vec2::new(400.0, 0.0)).apply(&mut d).unwrap();
    let inst = instances_of(&d, comp)[0];
    let inst_body = d.get(inst).unwrap().children[0];
    assert!(d.get(inst_body).unwrap().component.is_slot, "the instance's copy of the slot frame is a slot");
    assert!(d.get(inst).unwrap().component.slot_configs.is_empty(), "settings belong to the definition, not to the instance");

    // Turning the flag off on the component reaches the instance on the next sync.
    d.get_mut(body).unwrap().component.is_slot = false;
    sync_instances(&mut d, comp).apply(&mut d).unwrap();
    assert!(!d.get(inst_body).unwrap().component.is_slot);
    assert!(matches!(d.get(inst).unwrap().kind, NodeKind::Instance { .. }));
}

#[test]
fn garbage_collection_keeps_a_component_that_only_a_slot_prefers() {
    let (mut d, comp, _, icon) = component_with_slot();
    // Detach the icon from the page: only the slot settings still name it.
    let page = d.pages[0];
    d.get_mut(page).unwrap().children.retain(|c| *c != icon);
    d.get_mut(icon).unwrap().parent = None;
    d.library_page = None;
    d.prune_unreachable();
    assert!(d.get(icon).is_some(), "preferred slot component is reachable through the slot settings");
    assert!(d.get(comp).is_some());
}

fn body_config() -> SlotPropConfig {
    SlotPropConfig { max_children: Some(2), ..Default::default() }
}

#[test]
fn renaming_a_slot_property_carries_its_settings_along() {
    let (mut d, comp, _, icon) = component_with_slot();
    rename_prop(&mut d, comp, "Body", "Content").apply(&mut d).unwrap();
    let n = d.get(comp).unwrap();
    assert!(n.component.props.iter().any(|p| p.name == "Content" && p.kind == PropKind::Slot));
    assert_eq!(n.component.slot_configs.get("Content"), Some(&SlotPropConfig { preferred: vec![icon], ..body_config() }));
    assert!(!n.component.slot_configs.contains_key("Body"), "no settings are left under the old name");
}

#[test]
fn renaming_a_slot_property_can_be_undone() {
    let (mut d, comp, _, _) = component_with_slot();
    let before = d.get(comp).unwrap().component.slot_configs.clone();
    let inverse = rename_prop(&mut d, comp, "Body", "Content").apply(&mut d).unwrap();
    assert_eq!(d.get(comp).unwrap().component.slot_configs.keys().collect::<Vec<_>>(), ["Content"]);
    inverse.apply(&mut d).unwrap();
    assert_eq!(d.get(comp).unwrap().component.slot_configs, before);
}

#[test]
fn deleting_a_slot_property_drops_its_settings() {
    let (mut d, comp, _, _) = component_with_slot();
    delete_prop(&mut d, comp, "Body").apply(&mut d).unwrap();
    assert!(d.get(comp).unwrap().component.props.is_empty());
    assert!(d.get(comp).unwrap().component.slot_configs.is_empty(), "settings of a deleted property are gone");
}

#[test]
fn a_deleted_slot_property_no_longer_keeps_its_preferred_component_alive() {
    let (mut d, comp, _, icon) = component_with_slot();
    delete_prop(&mut d, comp, "Body").apply(&mut d).unwrap();
    // Detach the icon from the page: the slot settings were the only thing still naming it.
    let page = d.pages[0];
    d.get_mut(page).unwrap().children.retain(|c| *c != icon);
    d.get_mut(icon).unwrap().parent = None;
    d.library_page = None;
    d.prune_unreachable();
    assert!(d.get(icon).is_none());
    assert!(d.get(comp).is_some());
}

#[test]
fn deleting_a_slot_property_can_be_undone() {
    let (mut d, comp, _, _) = component_with_slot();
    let before = d.get(comp).unwrap().component.slot_configs.clone();
    let inverse = delete_prop(&mut d, comp, "Body").apply(&mut d).unwrap();
    assert!(d.get(comp).unwrap().component.slot_configs.is_empty());
    inverse.apply(&mut d).unwrap();
    assert_eq!(d.get(comp).unwrap().component.slot_configs, before);
}

#[test]
fn settings_of_the_other_slot_properties_are_not_touched() {
    let (mut d, comp, _, icon) = component_with_slot();
    let node = d.get_mut(comp).unwrap();
    node.component.props.push(ComponentPropDef { name: "Footer".into(), kind: PropKind::Slot, default: PropValue::Slot(vec![]) });
    node.component.slot_configs.insert("Footer".into(), SlotPropConfig { min_children: Some(1), ..Default::default() });
    rename_prop(&mut d, comp, "Body", "Content").apply(&mut d).unwrap();
    delete_prop(&mut d, comp, "Content").apply(&mut d).unwrap();
    let n = d.get(comp).unwrap();
    assert_eq!(n.component.slot_configs.len(), 1);
    assert_eq!(n.component.slot_configs.get("Footer"), Some(&SlotPropConfig { min_children: Some(1), ..Default::default() }));
    assert!(n.component.slot_configs.values().all(|c| !c.preferred.contains(&icon)));
}

#[test]
fn editing_instance_slot_adds_reorders_and_removes_without_detaching_and_enforces_limits() {
    let (mut d, comp, body, icon) = component_with_slot();
    let id = d.alloc_id();
    let mut header = Node::frame(id, Rect::new(0.0, 0.0, 200.0, 20.0));
    header.name = "Header".into();
    let header = d.insert(comp, header).unwrap();
    d.get_mut(body).unwrap().component.is_slot = true;
    d.get_mut(body).unwrap().component.bindings.insert(crate::OverrideField::SlotContent, "Body".into());
    d.get_mut(comp).unwrap().component.slot_configs.insert(
        "Body".into(),
        SlotPropConfig { min_children: Some(1), max_children: Some(2), allow_preferred_values_only: Some(true), preferred: vec![icon], ..Default::default() },
    );

    let page = d.pages[0];
    create_instance(&mut d, comp, page, Vec2::new(400.0, 0.0)).apply(&mut d).unwrap();
    let inst = instances_of(&d, comp)[0];
    let inst_body = *d.get(inst).unwrap().children.iter().find(|&&c| d.get(c).unwrap().name == "body").unwrap();
    let inst_header = *d.get(inst).unwrap().children.iter().find(|&&c| d.get(c).unwrap().name == "Header").unwrap();

    // 1. Non-preferred plain frame is rejected when allow_preferred_values_only is true.
    let bad_id = d.alloc_id();
    let bad_rect = Node::frame(bad_id, Rect::new(0.0, 0.0, 20.0, 20.0));
    assert_eq!(crate::insert_into_slot(&mut d, inst_body, 0, vec![bad_rect]).unwrap_err(), crate::SlotConstraintError::PreferredOnly);

    // 2. Add two preferred instances of `icon` into `inst_body`.
    let make_icon_inst = |d: &mut Document, label: &str| -> Vec<Node> {
        let cmd = create_instance(d, icon, page, Vec2::ZERO);
        let mut nodes = match cmd {
            crate::Command::Insert { nodes, .. } => nodes,
            _ => panic!("expected Insert"),
        };
        nodes[0].name = label.into();
        nodes
    };
    let n1 = make_icon_inst(&mut d, "Chip 1");
    let c1_id = n1[0].id;
    crate::insert_into_slot(&mut d, inst_body, 0, n1).unwrap().apply(&mut d).unwrap();
    assert!(crate::is_linked(d.get(inst).unwrap()), "instance stays linked after adding slot child");
    assert!(matches!(d.get(inst).unwrap().kind, NodeKind::Instance { .. }));
    assert!(d.get(inst_body).unwrap().component.overrides.contains(&crate::OverrideField::SlotContent));

    let n2 = make_icon_inst(&mut d, "Chip 2");
    let c2_id = n2[0].id;
    crate::insert_into_slot(&mut d, inst_body, 1, n2).unwrap().apply(&mut d).unwrap();
    assert_eq!(d.get(inst_body).unwrap().children, vec![c1_id, c2_id]);

    // 3. Adding a 3rd child exceeds max_children = 2.
    let n3 = make_icon_inst(&mut d, "Chip 3");
    assert_eq!(crate::insert_into_slot(&mut d, inst_body, 2, n3).unwrap_err(), crate::SlotConstraintError::MaxChildren(2));

    // 4. Reorder children inside the instance slot without detaching.
    crate::reorder_in_slot(&mut d, inst_body, c2_id, 0).unwrap().apply(&mut d).unwrap();
    assert_eq!(d.get(inst_body).unwrap().children, vec![c2_id, c1_id]);
    assert!(crate::is_linked(d.get(inst).unwrap()));

    // 5. Main component updates propagate to non-slot parts while slot content is preserved.
    d.get_mut(header).unwrap().opacity = 0.35;
    sync_instances(&mut d, comp).apply(&mut d).unwrap();
    assert_eq!(d.get(inst_header).unwrap().opacity, 0.35, "non-slot part updated from main");
    assert_eq!(d.get(inst_body).unwrap().children, vec![c2_id, c1_id], "slot children preserved across sync");

    // 6. Removing one child succeeds (leaves 1 >= min_children), removing the last is blocked by min_children = 1.
    crate::remove_from_slot(&mut d, c2_id).unwrap().apply(&mut d).unwrap();
    assert_eq!(d.get(inst_body).unwrap().children, vec![c1_id]);
    assert_eq!(crate::remove_from_slot(&mut d, c1_id).unwrap_err(), crate::SlotConstraintError::MinChildren(1));
}

/// An instance whose slot frame holds the main component's default child "Default".
fn instance_with_default_slot_child() -> (Document, NodeId, NodeId, NodeId, NodeId, NodeId) {
    let (mut d, comp, body, _icon) = component_with_slot();
    let id = d.alloc_id();
    let mut default_child = Node::frame(id, Rect::new(0.0, 0.0, 10.0, 10.0));
    default_child.name = "Default".into();
    let main_default = d.insert(body, default_child).unwrap();
    d.get_mut(body).unwrap().component.is_slot = true;
    d.get_mut(body).unwrap().component.bindings.insert(crate::OverrideField::SlotContent, "Body".into());
    let page = d.pages[0];
    create_instance(&mut d, comp, page, Vec2::new(400.0, 0.0)).apply(&mut d).unwrap();
    let inst = instances_of(&d, comp)[0];
    let inst_body = *d.get(inst).unwrap().children.iter().find(|&&c| d.get(c).unwrap().name == "body").unwrap();
    let inst_default = d.get(inst_body).unwrap().children[0];
    assert_eq!(d.get(inst_default).unwrap().component.source, Some(main_default));
    (d, comp, inst, inst_body, inst_default, main_default)
}

#[test]
fn reset_of_slot_content_rebuilds_the_slot_from_the_main_component() {
    let (mut d, comp, inst, inst_body, inst_default, main_default) = instance_with_default_slot_child();

    // The user adds a plain frame: it has no source, so only the override flag marks it as theirs.
    let id = d.alloc_id();
    let mut extra = Node::frame(id, Rect::new(0.0, 0.0, 10.0, 10.0));
    extra.name = "Extra".into();
    let extra_id = extra.id;
    crate::insert_into_slot(&mut d, inst_body, 1, vec![extra]).unwrap().apply(&mut d).unwrap();
    assert_eq!(d.get(inst_body).unwrap().children, vec![inst_default, extra_id]);
    assert!(d.get(inst_body).unwrap().component.overrides.contains(&crate::OverrideField::SlotContent));

    crate::reset_overrides(&mut d, inst, &[crate::OverrideField::SlotContent]).apply(&mut d).unwrap();

    let kids = d.get(inst_body).unwrap().children.clone();
    assert!(d.get(extra_id).is_none(), "the user's frame is gone after Reset");
    assert_eq!(kids.len(), 1);
    assert_eq!(d.get(kids[0]).unwrap().component.source, Some(main_default), "the slot mirrors the main's slot children");
    assert!(!d.get(inst_body).unwrap().component.overrides.contains(&crate::OverrideField::SlotContent));
    assert!(matches!(d.get(inst).unwrap().kind, NodeKind::Instance { .. }));

    // Once reset, the slot follows the main component again.
    d.get_mut(main_default).unwrap().opacity = 0.25;
    sync_instances(&mut d, comp).apply(&mut d).unwrap();
    assert_eq!(d.get(d.get(inst_body).unwrap().children[0]).unwrap().opacity, 0.25);
}

#[test]
fn reset_of_slot_content_brings_back_a_removed_main_slot_child() {
    let (mut d, _, inst, inst_body, inst_default, main_default) = instance_with_default_slot_child();
    let id = d.alloc_id();
    let extra = Node::frame(id, Rect::new(0.0, 0.0, 10.0, 10.0));
    crate::insert_into_slot(&mut d, inst_body, 1, vec![extra]).unwrap().apply(&mut d).unwrap();
    crate::remove_from_slot(&mut d, inst_default).unwrap().apply(&mut d).unwrap();
    assert!(d.get(inst_default).is_none());

    crate::reset_overrides(&mut d, inst, &[crate::OverrideField::SlotContent]).apply(&mut d).unwrap();
    let kids = d.get(inst_body).unwrap().children.clone();
    assert_eq!(kids.len(), 1);
    assert_eq!(d.get(kids[0]).unwrap().component.source, Some(main_default));
}

#[test]
fn a_slot_that_still_holds_the_users_content_survives_a_main_sync() {
    let (mut d, comp, _inst, inst_body, inst_default, main_default) = instance_with_default_slot_child();
    let id = d.alloc_id();
    let extra = Node::frame(id, Rect::new(0.0, 0.0, 10.0, 10.0));
    let extra_id = extra.id;
    crate::insert_into_slot(&mut d, inst_body, 1, vec![extra]).unwrap().apply(&mut d).unwrap();
    d.get_mut(main_default).unwrap().opacity = 0.5;
    sync_instances(&mut d, comp).apply(&mut d).unwrap();
    assert_eq!(d.get(inst_body).unwrap().children, vec![inst_default, extra_id], "overridden slot content is left alone");
}

#[test]
fn a_swap_is_judged_on_the_count_the_slot_ends_with() {
    let (mut d, comp, body, _) = component_with_slot();
    d.get_mut(body).unwrap().component.is_slot = true;
    d.get_mut(body).unwrap().component.bindings.insert(crate::OverrideField::SlotContent, "Body".into());
    d.get_mut(comp)
        .unwrap()
        .component
        .slot_configs
        .insert("Body".into(), SlotPropConfig { min_children: Some(2), max_children: Some(2), ..Default::default() });
    for _ in 0..2 {
        let id = d.alloc_id();
        d.insert(body, Node::frame(id, Rect::new(0.0, 0.0, 4.0, 4.0))).unwrap();
    }
    let ids: Vec<NodeId> = (0..3).map(|_| d.alloc_id()).collect();
    let nodes: Vec<Node> = ids.iter().map(|id| Node::frame(*id, Rect::new(0.0, 0.0, 4.0, 4.0))).collect();
    let refs: Vec<&Node> = nodes.iter().collect();
    // 2 - 1 + 1 = 2: one for one is fine at both limits.
    assert!(crate::check_slot_replace(&d, body, 1, &refs[..1]).is_ok());
    // 2 - 1 + 2 = 3 > max.
    assert_eq!(crate::check_slot_replace(&d, body, 1, &refs[..2]).unwrap_err(), crate::SlotConstraintError::MaxChildren(2));
    // 2 - 2 + 1 = 1 < min.
    assert_eq!(crate::check_slot_replace(&d, body, 2, &refs[..1]).unwrap_err(), crate::SlotConstraintError::MinChildren(2));
    // a plain insert keeps the old rule: 2 + 1 > max.
    assert_eq!(crate::check_slot_insert(&d, body, &refs[..1]).unwrap_err(), crate::SlotConstraintError::MaxChildren(2));
}
