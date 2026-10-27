use designos_doc::{NodeKind, PathCmd};
use designos_fig::container::{self, Compression};
use designos_fig::import_fig;
use designos_fig::kiwi::KiwiValue as V;
use designos_fig::testkit::{figma_like_schema, guid, msg, node, square_blob, st, vec2};

fn geometry(blob: u32) -> V {
    V::Array(vec![msg(vec![("windingRule", V::Enum("NONZERO".into())), ("commandsBlob", V::Uint(blob))])])
}

fn guid_path(path: &[(u32, u32)]) -> V {
    msg(vec![("guids", V::Array(path.iter().map(|&(s, l)| guid(s, l)).collect()))])
}

fn max_x(paths: &[designos_doc::Path]) -> f32 {
    paths
        .iter()
        .flat_map(|path| path.cmds.iter())
        .filter_map(|cmd| match cmd {
            PathCmd::MoveTo(p) | PathCmd::LineTo(p) => Some(p.x),
            _ => None,
        })
        .fold(0.0, f32::max)
}

#[test]
fn large_fig_resized_instance_scales_derived_fill_and_stroke_geometry() {
    let document = node((0, 0), (0, 0), "!", "DOCUMENT", "LargeDerivedGeometry", vec![]);
    let page = node((0, 1), (0, 0), "!", "CANVAS", "Page", vec![]);
    let component = node((1, 2), (0, 1), "!", "SYMBOL", "Icon", vec![("size", vec2(10.0, 10.0))]);
    let vector = node((1, 3), (1, 2), "!", "VECTOR", "Arrow", vec![("size", vec2(10.0, 10.0)), ("fillGeometry", geometry(0)), ("strokeGeometry", geometry(0))]);
    let instance = node(
        (1, 10),
        (0, 1),
        "\"",
        "INSTANCE",
        "IconSmall",
        vec![
            ("size", vec2(5.0, 5.0)),
            ("symbolData", msg(vec![("symbolID", guid(1, 2)), ("symbolOverrides", V::Array(vec![]))])),
            (
                "derivedSymbolData",
                V::Array(vec![msg(vec![
                    ("guidPath", guid_path(&[(1, 3)])),
                    ("size", vec2(5.0, 5.0)),
                    ("fillGeometry", geometry(0)),
                    ("strokeGeometry", geometry(0)),
                ])]),
            ),
        ],
    );

    let mut changes = vec![document, page, component, vector, instance];
    for i in 0..5_001u32 {
        changes.push(node((9, i + 1), (0, 1), &format!("p{i}"), "RECTANGLE", &format!("Pad {i}"), vec![("size", vec2(1.0, 1.0))]));
    }
    let message = msg(vec![("nodeChanges", V::Array(changes)), ("blobs", V::Array(vec![st(vec![("bytes", V::Bytes(square_blob()))])]))]);
    let schema = figma_like_schema();
    let root = schema.find("Message").expect("Message schema");
    let body = schema.encode_message(root, &message).expect("encode message");
    let bytes = container::write_container(&schema.encode(), &body, Compression::Zstd, 101);
    let doc = import_fig(&bytes).expect("import large fig");
    let instance = doc.nodes.values().find(|n| n.name == "IconSmall").expect("instance");
    let arrow = doc.get(instance.children[0]).expect("instance vector child");
    let NodeKind::Vector { paths, .. } = &arrow.kind else { panic!("expected vector child") };
    assert!((max_x(paths) - 5.0).abs() < 1e-4, "large-file derived fill geometry must scale; got {}", max_x(paths));
    let stroke = arrow.stroke_geometry.as_ref().expect("derived stroke geometry");
    assert!((max_x(stroke) - 5.0).abs() < 1e-4, "large-file derived stroke geometry must scale; got {}", max_x(stroke));
}
