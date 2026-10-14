use designos_doc::*;
use designos_fig::testkit::sample_fig;
use designos_fig::{
    commands_blob_to_path, export_fig, import_fig, import_fig_node_changes, path_to_commands_blob,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn find<'a>(doc: &'a Document, name: &str) -> &'a Node {
    doc.nodes
        .values()
        .find(|n| n.name == name)
        .unwrap_or_else(|| panic!("no node named {name}"))
}

#[test]
fn commands_blob_roundtrip_preserves_all_verbs_and_winding_rules() {
    for (even_odd, fill_rule) in [(false, FillRule::NonZero), (true, FillRule::EvenOdd)] {
        let original = Path {
            cmds: vec![
                PathCmd::MoveTo(Vec2::new(10.0, 20.0)),
                PathCmd::LineTo(Vec2::new(110.0, 20.0)),
                PathCmd::QuadTo(Vec2::new(130.0, 20.0), Vec2::new(130.0, 40.0)),
                PathCmd::CubicTo(
                    Vec2::new(130.0, 70.0),
                    Vec2::new(90.0, 90.0),
                    Vec2::new(60.0, 90.0),
                ),
                PathCmd::Close,
            ],
            fill_rule,
        };
        let blob = path_to_commands_blob(&original);
        let decoded = commands_blob_to_path(&blob, even_odd);
        assert_eq!(decoded.fill_rule, fill_rule);
        assert_eq!(decoded.cmds, original.cmds);
    }
}

#[test]
fn export_fig_then_import_fig_roundtrips_sample_and_rich_documents() {
    // 1. Roundtrip the canonical sample_fig fixture through export_fig -> import_fig
    let raw = sample_fig(designos_fig::container::Compression::Zstd, true);
    let doc1 = import_fig(&raw).expect("import sample_fig");
    let exported1 = export_fig(&doc1).expect("export_fig on sample_fig");
    let doc2 = import_fig(&exported1).expect("re-import exported sample_fig");

    assert_eq!(doc2.pages.len(), doc1.pages.len());
    let card = find(&doc2, "Card");
    assert_eq!(card.size, Vec2::new(200.0, 300.0));
    assert_eq!(card.corner_radii, [12.0; 4]);
    let f = card.kind.frame_props().expect("Card is a frame");
    let al = f.auto_layout.expect("Card has auto_layout");
    assert_eq!(al.direction, LayoutDirection::Vertical);
    assert_eq!(al.padding, [12.0, 16.0, 12.0, 16.0]);

    let title = find(&doc2, "Title");
    let NodeKind::Text(tp) = &title.kind else {
        panic!("expected Text")
    };
    assert_eq!(tp.content, "Hi \u{1F600}!");
    assert_eq!(tp.runs.len(), 1);
    assert_eq!(tp.runs[0].style.font_weight, 700);
    assert!(tp.runs[0].style.italic);

    let tri = find(&doc2, "Tri");
    let NodeKind::Vector { paths, .. } = &tri.kind else {
        panic!("expected Vector")
    };
    assert_eq!(paths[0].cmds.len(), 4);

    // 2. Build a rich Document with variables, asymmetric auto-layout, wrap, individual stroke weights,
    // dev status, and embedded images.
    let mut rich = Document::new("Parity Rich Doc");
    let page_id = rich.add_page("Page 1");

    let fake_png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 1, 2, 3, 4];
    let img_hash = "0102030405060708090a0b0c0d0e0f1011121314";
    rich.images.insert(img_hash.into(), fake_png.clone());

    rich.guides.insert(
        page_id,
        vec![Guide {
            axis: GuideAxis::X,
            offset: 120.0,
            frame: None,
        }],
    );

    // Variable collection + variables
    rich.variable_collections.insert(
        "VariableCollectionId:10:1".into(),
        VariableCollection {
            id: "VariableCollectionId:10:1".into(),
            name: "Theme".into(),
            modes: vec![
                VariableMode {
                    id: "10:0".into(),
                    name: "Light".into(),
                },
                VariableMode {
                    id: "10:1".into(),
                    name: "Dark".into(),
                },
            ],
            default_mode: "10:0".into(),
            variables: vec![
                "VariableID:10:10".into(),
                "VariableID:10:11".into(),
                "VariableID:10:12".into(),
            ],
            key: Some("theme_key".into()),
        },
    );
    let mut bg_vals = BTreeMap::new();
    bg_vals.insert("10:0".into(), VariableValue::Color(Color::rgb(240, 242, 245)));
    bg_vals.insert("10:1".into(), VariableValue::Color(Color::rgb(24, 28, 36)));
    rich.variables.insert(
        "VariableID:10:10".into(),
        Variable {
            id: "VariableID:10:10".into(),
            name: "surface/bg".into(),
            collection: "VariableCollectionId:10:1".into(),
            kind: VariableKind::Color,
            values: bg_vals,
            description: "Surface background".into(),
            scopes: vec!["ALL_FILLS".into()],
            hidden_from_publishing: false,
            key: Some("var_bg_key".into()),
        },
    );
    let mut pad_vals = BTreeMap::new();
    pad_vals.insert("10:0".into(), VariableValue::Number(16.0));
    pad_vals.insert("10:1".into(), VariableValue::Number(24.0));
    rich.variables.insert(
        "VariableID:10:11".into(),
        Variable {
            id: "VariableID:10:11".into(),
            name: "space/md".into(),
            collection: "VariableCollectionId:10:1".into(),
            kind: VariableKind::Number,
            values: pad_vals,
            description: String::new(),
            scopes: vec!["GAP".into()],
            hidden_from_publishing: false,
            key: Some("var_pad_key".into()),
        },
    );
    let mut alias_vals = BTreeMap::new();
    alias_vals.insert(
        "10:0".into(),
        VariableValue::Alias("VariableID:10:10".into()),
    );
    alias_vals.insert(
        "10:1".into(),
        VariableValue::Alias("VariableID:10:10".into()),
    );
    rich.variables.insert(
        "VariableID:10:12".into(),
        Variable {
            id: "VariableID:10:12".into(),
            name: "surface/alias".into(),
            collection: "VariableCollectionId:10:1".into(),
            kind: VariableKind::Color,
            values: alias_vals,
            description: String::new(),
            scopes: vec!["ALL_FILLS".into()],
            hidden_from_publishing: false,
            key: Some("var_alias_key".into()),
        },
    );

    // Component with asymmetric auto-layout, wrap, stroke_included, reverse_z, dev.status, bindings
    let mut comp_props = FrameProps::default();
    comp_props.clips_content = false;
    comp_props.auto_layout = Some(AutoLayout {
        direction: LayoutDirection::Horizontal,
        item_spacing: 10.0,
        padding: [8.0, 16.0, 12.0, 20.0], // top, right, bottom, left
        primary_align: LayoutAlign::Center,
        counter_align: LayoutAlign::Center,
        primary_sizing: Sizing::Fixed,
        counter_sizing: Sizing::Hug,
        wrap: true,
        counter_spacing: 14.0,
        stroke_included: true,
        reverse_z: true,
    });
    let comp_id = rich.alloc_id();
    let mut comp_node = Node::new(comp_id, NodeKind::Component(comp_props));
    comp_node.name = "PrimaryButton".into();
    comp_node.size = Vec2::new(160.0, 48.0);
    comp_node.dev.status = Some(DevStatus::ReadyForDev);
    comp_node
        .variables
        .bindings
        .insert("paddingLeft".into(), "VariableID:10:11".into());
    comp_node
        .variables
        .bindings
        .insert("itemSpacing".into(), "VariableID:10:11".into());
    rich.insert(page_id, comp_node).unwrap();

    // Child shape inside PrimaryButton with side_weights + image fill + variable binding
    let bg_id = rich.alloc_id();
    let mut bg_rect = Node::new(bg_id, NodeKind::Rectangle);
    bg_rect.name = "BgSurface".into();
    bg_rect.size = Vec2::new(120.0, 32.0);
    bg_rect.fills = vec![
        Paint::solid(Color::rgb(240, 242, 245)),
        Paint {
            kind: PaintKind::Image {
                image_ref: img_hash.into(),
                scale_mode: ImageScaleMode::Fill,
            },
            opacity: 0.9,
            visible: true,
            blend: BlendMode::Normal,
        },
    ];
    bg_rect.stroke = Stroke {
        paints: vec![Paint::solid(Color::rgb(50, 75, 100))],
        weight: 2.0,
        align: StrokeAlign::Inside,
        cap: StrokeCap::Round,
        join: StrokeJoin::Bevel,
        dashes: Vec::new(),
        miter_limit: 4.0,
        side_weights: Some([1.0, 2.0, 3.0, 4.0]),
    };
    bg_rect.layout_child = LayoutChild {
        horizontal: Sizing::Fill,
        vertical: Sizing::Fill,
        grow: 1.0,
        ..LayoutChild::default()
    };
    bg_rect
        .variables
        .bindings
        .insert("fills/0".into(), "VariableID:10:10".into());
    rich.insert(comp_id, bg_rect).unwrap();

    // Export and re-import
    let rich_bytes = export_fig(&rich).expect("export_fig rich doc");
    let rich_imp = import_fig(&rich_bytes).expect("import_fig rich doc");

    assert_eq!(rich_imp.images.get(img_hash), Some(&fake_png));
    assert_eq!(rich_imp.variable_collections.len(), 1);
    assert_eq!(rich_imp.variables.len(), 3);

    let imp_comp = find(&rich_imp, "PrimaryButton");
    assert_eq!(imp_comp.dev.status, Some(DevStatus::ReadyForDev));
    let imp_al = imp_comp
        .kind
        .frame_props()
        .unwrap()
        .auto_layout
        .expect("auto_layout");
    assert_eq!(imp_al.direction, LayoutDirection::Horizontal);
    assert_eq!(imp_al.padding, [8.0, 16.0, 12.0, 20.0]);
    assert_eq!(imp_al.item_spacing, 10.0);
    assert!(imp_al.wrap);
    assert_eq!(imp_al.counter_spacing, 14.0);
    assert!(imp_al.stroke_included);
    assert!(imp_al.reverse_z);
    assert!(imp_comp.variables.bindings.contains_key("paddingLeft"));
    assert!(imp_comp.variables.bindings.contains_key("itemSpacing"));

    let imp_bg = find(&rich_imp, "BgSurface");
    assert_eq!(imp_bg.stroke.side_weights, Some([1.0, 2.0, 3.0, 4.0]));
    assert_eq!(imp_bg.layout_child.horizontal, Sizing::Fill);
    assert_eq!(imp_bg.layout_child.vertical, Sizing::Fill);
    assert_eq!(imp_bg.layout_child.grow, 1.0);
    assert!(imp_bg.variables.bindings.contains_key("fills/0"));
}

#[test]
fn fig_import_resolves_symbol_overrides_bindings_nested_swaps_and_handoff_map() {
    let g = |s: u64, l: u64| -> Value { json!({"sessionID": s, "localID": l}) };
    let node = |id: (u64, u64),
                parent: (u64, u64),
                pos: &str,
                ty: &str,
                name: &str,
                extra: Value|
     -> Value {
        let mut v = json!({
            "guid": g(id.0, id.1),
            "parentIndex": {"guid": g(parent.0, parent.1), "position": pos},
            "type": ty,
            "name": name,
            "phase": "CREATED",
            "visible": true,
            "opacity": 1.0,
            "size": {"x": 24.0, "y": 24.0},
            "transform": {"m00": 1.0, "m01": 0.0, "m02": 0.0, "m10": 0.0, "m11": 1.0, "m12": 0.0}
        });
        for (k, x) in extra.as_object().unwrap() {
            v[k] = x.clone();
        }
        v
    };

    let changes = json!([
        node((0, 0), (0, 0), "!", "DOCUMENT", "Document", json!({})),
        node((1, 10), (0, 0), "a", "VARIABLE_SET", "Tokens", json!({
            "variableSetModes": [
                {"id": g(1, 100), "name": "Default", "sortPosition": "a"}
            ]
        })),
        node((1, 11), (1, 10), "a", "VARIABLE", "color/primary", json!({
            "variableSetID": g(1, 10),
            "variableResolvedType": "COLOR",
            "variableDataValues": {
                "entries": [{
                    "modeID": g(1, 100),
                    "variableData": {
                        "dataType": "COLOR",
                        "value": {"colorValue": {"r": 0.1, "g": 0.3, "b": 0.9, "a": 1.0}}
                    }
                }]
            }
        })),
        node((1, 12), (1, 10), "b", "VARIABLE", "color/accent", json!({
            "variableSetID": g(1, 10),
            "variableResolvedType": "COLOR",
            "variableDataValues": {
                "entries": [{
                    "modeID": g(1, 100),
                    "variableData": {
                        "dataType": "COLOR",
                        "value": {"colorValue": {"r": 0.95, "g": 0.35, "b": 0.25, "a": 1.0}}
                    }
                }]
            }
        })),
        node((1, 13), (1, 10), "c", "VARIABLE", "space/lg", json!({
            "variableSetID": g(1, 10),
            "variableResolvedType": "FLOAT",
            "variableDataValues": {
                "entries": [{
                    "modeID": g(1, 100),
                    "variableData": {
                        "dataType": "FLOAT",
                        "value": {"floatValue": 24.0}
                    }
                }]
            }
        })),
        node((1, 14), (1, 10), "d", "VARIABLE", "font/style", json!({
            "variableSetID": g(1, 10),
            "variableResolvedType": "STRING",
            "variableDataValues": {
                "entries": [{
                    "modeID": g(1, 100),
                    "variableData": {
                        "dataType": "STRING",
                        "value": {"textValue": "Bold"}
                    }
                }]
            }
        })),
        node((1, 2), (0, 0), "b", "CANVAS", "Page 1", json!({
            "handoffStatusMap": {
                "entries": [
                    {"guid": g(1, 50), "handoffStatus": "READY_FOR_DEV"}
                ]
            }
        })),
        // IconA (SYMBOL) with child StarShape
        node((1, 20), (1, 2), "a", "SYMBOL", "IconA", json!({})),
        node((1, 21), (1, 20), "a", "STAR", "StarShape", json!({})),
        // IconB (SYMBOL) with child CircleShape
        node((1, 30), (1, 2), "b", "SYMBOL", "IconB", json!({})),
        node((1, 31), (1, 30), "a", "ELLIPSE", "CircleShape", json!({})),
        // Card (SYMBOL) with nested IconSlot (instance of IconA) and Surface (bound to color/primary)
        node((1, 40), (1, 2), "c", "SYMBOL", "Card", json!({
            "size": {"x": 200.0, "y": 100.0},
            "stackMode": "HORIZONTAL",
            "stackSpacing": 8.0
        })),
        node((1, 41), (1, 40), "a", "INSTANCE", "IconSlot", json!({
            "symbolData": {"symbolID": g(1, 20)}
        })),
        node((1, 42), (1, 40), "b", "ROUNDED_RECTANGLE", "Surface", json!({
            "size": {"x": 160.0, "y": 80.0},
            "fillPaints": [{
                "type": "SOLID",
                "color": {"r": 0.1, "g": 0.3, "b": 0.9, "a": 1.0},
                "opacity": 1.0,
                "visible": true,
                "colorVar": {
                    "dataType": "ALIAS",
                    "value": {"alias": {"guid": g(1, 11)}}
                }
            }],
            "strokePaints": [{
                "type": "SOLID",
                "color": {"r": 0.1, "g": 0.3, "b": 0.9, "a": 1.0},
                "opacity": 1.0,
                "visible": true,
                "colorVar": {
                    "dataType": "ALIAS",
                    "value": {"alias": {"guid": g(1, 11)}}
                }
            }],
            "strokeWeight": 1.0
        })),
        // CardInstance (INSTANCE of Card)
        node((1, 50), (1, 2), "d", "INSTANCE", "CardInstance", json!({
            "size": {"x": 240.0, "y": 120.0},
            "stackMode": "HORIZONTAL",
            "parameterConsumptionMap": {
                "entries": [
                    {
                        "variableField": "STACK_HORIZONTAL_PADDING",
                        "variableData": {
                            "dataType": "ALIAS",
                            "value": {"alias": {"guid": g(1, 13)}}
                        }
                    },
                    {
                        "variableField": "FONT_STYLE",
                        "variableData": {
                            "dataType": "FONT_STYLE",
                            "value": {
                                "fontStyleValue": {
                                    "asString": {
                                        "dataType": "ALIAS",
                                        "value": {"alias": {"guid": g(1, 14)}}
                                    }
                                }
                            }
                        }
                    }
                ]
            },
            "symbolData": {
                "symbolID": g(1, 40),
                "symbolOverrides": [
                    {
                        "guidPath": {"guids": [g(1, 42)]},
                        "fillPaints": [{
                            "type": "SOLID",
                            "color": {"r": 1.0, "g": 1.0, "b": 1.0, "a": 1.0},
                            "opacity": 1.0,
                            "visible": true
                        }],
                        "strokePaints": [{
                            "type": "SOLID",
                            "color": {"r": 0.95, "g": 0.35, "b": 0.25, "a": 1.0},
                            "opacity": 1.0,
                            "visible": true,
                            "colorVar": {
                                "dataType": "ALIAS",
                                "value": {"alias": {"guid": g(1, 12)}}
                            }
                        }],
                        "borderStrokeWeightsIndependent": true,
                        "borderTopWeight": 2.0,
                        "borderRightWeight": 4.0,
                        "borderBottomWeight": 6.0,
                        "borderLeftWeight": 8.0,
                        "stackChildPrimaryGrow": 1.0,
                        "stackChildAlignSelf": "STRETCH"
                    },
                    {
                        "guidPath": {"guids": [g(1, 41)]},
                        "overriddenSymbolID": g(1, 30)
                    }
                ]
            }
        }))
    ]);

    let doc = import_fig_node_changes(&changes, "Override Fidelity Doc")
        .expect("import_fig_node_changes");

    let card_inst = find(&doc, "CardInstance");

    // 1. CANVAS.handoffStatusMap resolved READY_FOR_DEV on CardInstance
    assert_eq!(card_inst.dev.status, Some(DevStatus::ReadyForDev));

    // 2. parameterConsumptionMap resolved STACK_HORIZONTAL_PADDING and FONT_STYLE
    assert_eq!(
        card_inst
            .variables
            .bindings
            .get("paddingLeft")
            .map(String::as_str),
        Some("VariableID:1:13")
    );
    assert_eq!(
        card_inst
            .variables
            .bindings
            .get("paddingRight")
            .map(String::as_str),
        Some("VariableID:1:13")
    );
    assert_eq!(
        card_inst
            .variables
            .bindings
            .get("fontWeight")
            .map(String::as_str),
        Some("VariableID:1:14")
    );

    // 3. Surface inside CardInstance:
    // - literal fillPaints override cleared master's fills/0 variable binding
    // - bound strokePaints override updated strokes/0 binding to VariableID:1:12
    // - independent border weights and stackChild overrides applied
    let inst_surface = card_inst
        .children
        .iter()
        .filter_map(|id| doc.get(*id))
        .find(|n| n.name == "Surface")
        .expect("Surface inside CardInstance");
    assert!(
        !inst_surface.variables.bindings.contains_key("fills/0"),
        "stale fills/0 binding from master component must be cleared when fillPaints is overridden with literal color"
    );
    assert_eq!(
        inst_surface
            .variables
            .bindings
            .get("strokes/0")
            .map(String::as_str),
        Some("VariableID:1:12")
    );
    assert_eq!(inst_surface.stroke.side_weights, Some([2.0, 4.0, 6.0, 8.0]));
    assert_eq!(inst_surface.layout_child.grow, 1.0);
    assert_eq!(inst_surface.layout_child.horizontal, Sizing::Fill);
    assert_eq!(inst_surface.layout_child.vertical, Sizing::Fill);

    // 4. IconSlot inside CardInstance: overriddenSymbolID swapped it from IconA (StarShape) to IconB (CircleShape)
    let inst_icon_slot = card_inst
        .children
        .iter()
        .filter_map(|id| doc.get(*id))
        .find(|n| n.name == "IconSlot")
        .expect("IconSlot inside CardInstance");
    let icon_child_names: Vec<&str> = inst_icon_slot
        .children
        .iter()
        .filter_map(|id| doc.get(*id))
        .map(|n| n.name.as_str())
        .collect();
    assert_eq!(icon_child_names, vec!["CircleShape"]);
}
