//! `.pen` component expansion must not use call-stack depth proportional to the nesting of the cloned tree.
//! Each test runs the import on a thread with a deliberately small stack, so a recursion that tracks the depth
//! of the cloned subtree aborts the process instead of passing.

use designos_fig::import_pen_json_with_warnings;

const SMALL_STACK: usize = 256 * 1024;

fn on_small_stack<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(SMALL_STACK).spawn(f).expect("spawn").join().expect("import must not panic")
}

/// Deepest-first chain: `c{i}` wraps its single ref to `c{i+1}` in `nest` frames, so every level clones the already
/// expanded chain below it and the cloned subtree gets `nest` levels deeper per component.
fn nested_component_chain(components: usize, nest: usize) -> String {
    let wrap = |inner: String, tag: &str| -> String {
        let mut s = inner;
        for d in 0..nest {
            s = format!(r##"{{"id":"{tag}n{d}","type":"frame","children":[{s}]}}"##);
        }
        s
    };
    let mut children = vec![format!(r##"{{"id":"c{components}","type":"frame","reusable":true,"children":[{{"id":"leaf","name":"leaf","type":"frame"}}]}}"##)];
    for i in (0..components).rev() {
        let inner = wrap(format!(r##"{{"id":"a{i}","type":"ref","ref":"c{}"}}"##, i + 1), &format!("c{i}"));
        children.push(format!(r##"{{"id":"c{i}","type":"frame","reusable":true,"children":[{inner}]}}"##));
    }
    format!(r##"{{"version":"1.0","children":[{}]}}"##, children.join(","))
}

#[test]
fn expands_a_chain_of_deeply_nested_components_on_a_small_stack() {
    let (components, nest) = (150, 3);
    let json = nested_component_chain(components, nest);
    let (doc, warning) = on_small_stack(move || import_pen_json_with_warnings(&json).expect("import"));
    assert!(warning.is_none(), "{warning:?}");

    // c0 holds the whole chain: every level adds `nest` wrapping frames and one ref, and the chain ends in `leaf`.
    // (The JSON itself nests only `nest` + 2 deep; the cloned tree is what gets thousands of frames deep.)
    let leaves = doc.nodes.values().filter(|n| n.name == "leaf").count();
    // The leaf is cloned once into every component above it, plus the original.
    assert_eq!(leaves, components + 1);
    // The deepest clone sits `components * (nest + 1)` levels below c0's own contents: nothing was cut off.
    let c0 = doc.nodes.values().find(|n| n.source_id.as_deref() == Some("c0")).expect("c0").id;
    let mut deepest = 0;
    doc.walk(c0, |_, depth| deepest = deepest.max(depth));
    assert!(deepest > components * nest, "deepest level {deepest}");
}

#[test]
fn clone_keeps_sibling_order_and_depth_first_ids() {
    let json = r##"{"version":"1.0","children":[
        {"id":"comp","type":"frame","reusable":true,"children":[
            {"id":"a","name":"a","type":"frame","children":[{"id":"a1","name":"a1","type":"frame"},{"id":"a2","name":"a2","type":"frame"}]},
            {"id":"b","name":"b","type":"frame"}]},
        {"id":"inst","name":"inst","type":"ref","ref":"comp"}]}"##;
    let (doc, _) = import_pen_json_with_warnings(json).unwrap();
    let inst = doc.nodes.values().find(|n| n.name == "inst").unwrap();
    let names = |ids: &[designos_doc::NodeId]| ids.iter().map(|&i| doc.get(i).unwrap().name.clone()).collect::<Vec<_>>();
    assert_eq!(names(&inst.children), ["a", "b"]);
    let a = doc.get(inst.children[0]).unwrap();
    assert_eq!(names(&a.children), ["a1", "a2"]);
    // Parents are cloned before their children and a subtree is finished before the next sibling starts.
    let order: Vec<_> = ["a", "a1", "a2", "b"]
        .iter()
        .map(|n| doc.nodes.values().find(|x| &x.name == n && x.parent.is_some_and(|p| p != doc.pages[0]) && x.component.source.is_some()).unwrap().id)
        .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{order:?}");
}
