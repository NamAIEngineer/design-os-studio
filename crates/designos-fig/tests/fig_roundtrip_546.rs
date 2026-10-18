use designos_doc::*;
use designos_fig::{export_fig, import_fig};

#[test]
fn roundtrip_preserves_node_count_across_saves_and_keeps_instance_child_alignment() {
    let g = |s: u32, l: u32| json!({"sessionID": s, "localID": l});
    let node = |guid: (u32, u32), parent: (u32, u32), pos: &str, ty: &str, name: &str, extra: Value| -> Value {
        let mut m = json!({
            "guid": g(guid.0, guid.1),
            "phase": "CREATED",
            "parentIndex": {"guid": g(parent.0, parent.1), "position": pos},
            "type": ty,
            "name": name,
            "visible": true,
            "opacity": 1.0,
            "size": {"x": 100.0, "y": 40.0},
            "transform": {"m00": 1.0, "m01": 0.0, "m02": 0.0, "m10": 0.0, "m11": 1.0, "m12": 0.0}
        });
        if let (Some(dst), Some(src)) = (m.as_object_mut(), extra.as_object()) {
            for (k, v) in src {
                dst.insert(k.clone(), v.clone());
            }
        }
        m
    };

    // Document with:
    // - 2 shared styles on Internal Only Canvas (and an external component so library_page exists)
    // - A main component `Calendar` containing a nested auto-layout child `_Button group base`
    //   with `stackPrimaryAlignItems: "CENTER"` and `stackCounterAlignItems: "CENTER"`.
    // - An instance `CalendarInst` whose `symbolOverrides` overrides `stackPrimarySizing: "FIXED"`
    //   (or `stackChildPrimaryGrow: 1.0`) on `_Button group base` without overriding its alignment.
    let changes = json!([
        node((0, 0), (0, 0), "!", "DOCUMENT", "RoundtripDoc", json!({})),
        node((0, 1), (0, 0), "a", "CANVAS", "↳ Calendars", json!({})),
        node((0, 2), (0, 0), "b", "CANVAS", "Internal Only Canvas", json!({"internalOnly": true})),
        node(
            (5, 1),
            (0, 2),
            "a",
            "SYMBOL",
            "LibComp",
            json!({"componentKey": "LIB_COMP_1", "sourceLibraryKey": "lib_ext"})
        ),
        node(
            (7, 1),
            (0, 2),
            "b",
            "RECTANGLE",
            "Brand/Primary",
            json!({
                "styleType": "FILL",
                "key": "STYLE_FILL_1",
                "fillPaints": [{"type": "SOLID", "color": {"r": 0.1, "g": 0.2, "b": 0.8, "a": 1.0}, "opacity": 1.0, "visible": true}]
            })
        ),
        node(
            (7, 2),
            (0, 2),
            "c",
            "RECTANGLE",
            "Shadow/Sm",
            json!({
                "styleType": "EFFECT",
                "key": "STYLE_FX_1",
                "effects": [{"type": "DROP_SHADOW", "color": {"r": 0.0, "g": 0.0, "b": 0.0, "a": 0.2}, "offset": {"x": 0.0, "y": 2.0}, "radius": 4.0, "spread": 0.0, "visible": true}]
            })
        ),
        node(
            (1, 10),
            (0, 1),
            "a",
            "SYMBOL",
            "Calendar",
            json!({
                "size": {"x": 300.0, "y": 300.0},
                "stackMode": "VERTICAL",
                "stackPrimarySizing": "FIXED",
                "stackCounterSizing": "FIXED"
            })
        ),
        node(
            (1, 11),
            (1, 10),
            "a",
            "FRAME",
            "_Button group base",
            json!({
                "overrideKey": g(9, 11),
                "size": {"x": 263.0, "y": 36.0},
                "styleIdForFill": {"guid": g(7, 1)},
                "styleIdForEffect": {"guid": g(7, 2)},
                "stackMode": "HORIZONTAL",
                "stackPrimaryAlignItems": "CENTER",
                "stackCounterAlignItems": "CENTER",
                "stackPrimarySizing": "RESIZE_TO_FIT",
                "stackCounterSizing": "FIXED"
            })
        ),
        node(
            (1, 20),
            (0, 1),
            "b",
            "INSTANCE",
            "CalendarInst",
            json!({
                "size": {"x": 300.0, "y": 300.0},
                "symbolData": {
                    "symbolID": g(1, 10),
                    "symbolOverrides": [
                        {
                            "guidPath": {"guids": [g(9, 11)]},
                            "stackPrimarySizing": "FIXED",
                            "stackChildPrimaryGrow": 1.0
                        }
                    ]
                }
            })
        )
    ]);

    let doc1 = import_fig_node_changes(&changes, "RoundtripDoc").expect("import 1");
    let inst_btn_align = |doc: &Document| -> (LayoutAlign, LayoutAlign) {
        let inst = find(doc, "CalendarInst");
        let btn_id = inst.children[0];
        let btn = doc.get(btn_id).expect("_Button group base in instance");
        let al = btn.kind.frame_props().and_then(|f| f.auto_layout.as_ref()).expect("auto_layout on _Button group base");
        (al.primary_align, al.counter_align)
    };
    assert_eq!(inst_btn_align(&doc1), (LayoutAlign::Center, LayoutAlign::Center));
    assert_eq!(doc1.styles.len(), 2);

    let bytes1 = export_fig(&doc1).expect("export 1");
    let doc2 = import_fig(&bytes1).expect("import 2");
    assert_eq!(doc2.nodes.len(), doc1.nodes.len(), "save-and-reopen cycle 1 must not add extra nodes");
    assert_eq!(inst_btn_align(&doc2), (LayoutAlign::Center, LayoutAlign::Center), "_Button group base alignment must stay Center/Center after roundtrip 1");

    let bytes2 = export_fig(&doc2).expect("export 2");
    let doc3 = import_fig(&bytes2).expect("import 3");
    assert_eq!(doc3.nodes.len(), doc1.nodes.len(), "save-and-reopen cycle 2 must not add extra nodes");
    assert_eq!(inst_btn_align(&doc3), (LayoutAlign::Center, LayoutAlign::Center), "_Button group base alignment must stay Center/Center after roundtrip 2");
}

