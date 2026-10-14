//! designos document model.
//!
//! This crate is the contract shared by every other crate: importers produce a
//! [`Document`], the renderer draws it, the app edits it through [`Command`]s.
//! Coordinates are in Figma units (1 unit = 1 CSS px at 100% zoom), y down.

mod board;
mod comments;
mod components;
mod dev;
pub mod find;
mod geom;
mod history;
mod library;
mod library_ops;
mod node;
mod paint;
mod prototype;
mod settings;
pub mod spec;
mod textmodel;
mod variables;
mod variables_apply;
mod variables_edit;
pub mod vector;
mod vector_fit;

pub use board::*;
pub use comments::*;
pub use components::*;
pub use dev::*;
pub use geom::*;
pub use history::*;
pub use library::*;
pub use library_ops::*;
pub use node::*;
pub use paint::*;
pub use prototype::*;
pub use settings::*;
pub use spec::*;
pub use textmodel::*;
pub use variables::*;
pub use variables_apply::*;
pub use variables_edit::*;
pub use vector::{HandleMirror, VectorNetwork};
pub use vector_fit::{fit_polyline, Anchor};

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Stable node identifier. Imported Figma ids ("12:34") are kept in
/// [`Node::source_id`]; `NodeId` is our own dense key.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u64);

#[derive(Debug, thiserror::Error)]
pub enum DocError {
    #[error("node {0:?} not found")]
    NotFound(NodeId),
    #[error("cannot move node {0:?} into its own subtree")]
    Cycle(NodeId),
    #[error("node {0:?} cannot have children")]
    NotAContainer(NodeId),
}

/// A whole file: pages at the root, each page a tree of nodes.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Document {
    pub name: String,
    /// Design file or board (whiteboard) file; decides the toolbar and panels.
    #[serde(default)]
    pub kind: DocumentKind,
    pub nodes: BTreeMap<NodeId, Node>,
    /// Page node ids in display order. Each page is a node with [`NodeKind::Page`].
    pub pages: Vec<NodeId>,
    pub styles: BTreeMap<String, Style>,
    /// Display metadata for entries of `styles`, same keys.
    #[serde(default)]
    pub style_meta: BTreeMap<String, StyleMeta>,
    /// Prototype flows (starting points), in display order.
    #[serde(default)]
    pub flows: Vec<Flow>,
    /// Variables (design tokens) by id.
    #[serde(default)]
    pub variables: BTreeMap<String, Variable>,
    /// Variable collections (with their modes) by id.
    #[serde(default)]
    pub variable_collections: BTreeMap<String, VariableCollection>,
    /// Ruler guides per page.
    #[serde(default)]
    pub guides: BTreeMap<NodeId, Vec<Guide>>,
    /// Device used when presenting the prototype.
    #[serde(default)]
    pub prototype_device: PrototypeDevice,
    /// Comment threads on the canvas.
    #[serde(default)]
    pub comments: Vec<CommentThread>,
    /// Published / enabled libraries and local copies of library assets.
    #[serde(default)]
    pub library: LibraryState,
    /// Hidden page holding copies of library components; not in `pages`.
    #[serde(default)]
    pub library_page: Option<NodeId>,
    /// Embedded images keyed by content hash (Figma `imageRef`).
    #[serde(default, with = "serde_images", skip_serializing_if = "BTreeMap::is_empty")]
    pub images: BTreeMap<String, Vec<u8>>,
    next_id: u64,
}

mod serde_images {
    use serde::de::{self, MapAccess, SeqAccess, Visitor};
    use serde::ser::SerializeMap;
    use serde::{Deserializer, Serializer};
    use std::collections::BTreeMap;
    use std::fmt;

    const HEX: &[u8; 16] = b"0123456789abcdef";

    fn to_hex_prefixed(bytes: &[u8]) -> String {
        let mut out = Vec::with_capacity(4 + bytes.len() * 2);
        out.extend_from_slice(b"hex:");
        for &b in bytes {
            out.push(HEX[(b >> 4) as usize]);
            out.push(HEX[(b & 0x0f) as usize]);
        }
        // SAFETY: `out` only contains ASCII hex characters.
        unsafe { String::from_utf8_unchecked(out) }
    }

    fn from_hex_str(s: &str) -> Result<Vec<u8>, String> {
        let hex = s.strip_prefix("hex:").unwrap_or(s);
        if hex.len() % 2 != 0 {
            return Err("odd-length hex image payload".into());
        }
        let mut out = Vec::with_capacity(hex.len() / 2);
        let bytes = hex.as_bytes();
        let val = |c: u8| -> Option<u8> {
            match c {
                b'0'..=b'9' => Some(c - b'0'),
                b'a'..=b'f' => Some(c - b'a' + 10),
                b'A'..=b'F' => Some(c - b'A' + 10),
                _ => None,
            }
        };
        let mut i = 0;
        while i < bytes.len() {
            let hi = val(bytes[i]).ok_or_else(|| "invalid hex digit".to_string())?;
            let lo = val(bytes[i + 1]).ok_or_else(|| "invalid hex digit".to_string())?;
            out.push((hi << 4) | lo);
            i += 2;
        }
        Ok(out)
    }

    struct ImageBytes(Vec<u8>);

    impl<'de> serde::Deserialize<'de> for ImageBytes {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            struct BytesVisitor;
            impl<'de> Visitor<'de> for BytesVisitor {
                type Value = ImageBytes;
                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("a hex string or byte array")
                }
                fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
                    from_hex_str(v).map(ImageBytes).map_err(E::custom)
                }
                fn visit_bytes<E: de::Error>(self, v: &[u8]) -> Result<Self::Value, E> {
                    Ok(ImageBytes(v.to_vec()))
                }
                fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                    let mut out = Vec::with_capacity(seq.size_hint().unwrap_or(0));
                    while let Some(b) = seq.next_element::<u8>()? {
                        out.push(b);
                    }
                    Ok(ImageBytes(out))
                }
            }
            deserializer.deserialize_any(BytesVisitor)
        }
    }

    pub fn serialize<S: Serializer>(
        map: &BTreeMap<String, Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let mut m = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            let hex = to_hex_prefixed(v);
            m.serialize_entry(k, &hex)?;
        }
        m.end()
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<BTreeMap<String, Vec<u8>>, D::Error> {
        struct MapVisitor;
        impl<'de> Visitor<'de> for MapVisitor {
            type Value = BTreeMap<String, Vec<u8>>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a map of image hash to hex string or byte array")
            }
            fn visit_map<M: MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
                let mut map = BTreeMap::new();
                while let Some((k, ImageBytes(v))) = access.next_entry::<String, ImageBytes>()? {
                    map.insert(k, v);
                }
                Ok(map)
            }
        }
        deserializer.deserialize_map(MapVisitor)
    }
}

impl Document {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into(), ..Default::default() }
    }

    pub fn alloc_id(&mut self) -> NodeId {
        self.next_id += 1;
        NodeId(self.next_id)
    }

    /// Allocate safely after importing a document whose stored counter may be
    /// stale, preserving previously allocated IDs and reporting exhaustion.
    pub fn try_alloc_id(&mut self) -> Option<NodeId> {
        let largest = self.nodes.keys().next_back().map_or(0, |id| id.0);
        self.next_id = self.next_id.max(largest).checked_add(1)?;
        Some(NodeId(self.next_id))
    }

    /// Ensures future [`Self::alloc_id`] calls return ids above `id`, so
    /// callers can assign deterministic ids themselves.
    pub fn reserve_ids(&mut self, id: NodeId) {
        self.next_id = self.next_id.max(id.0);
    }

    /// Display name of a shared style: its metadata name, else the key.
    pub fn style_name<'a>(&'a self, key: &'a str) -> &'a str {
        self.style_meta.get(key).map(|m| m.name.as_str()).filter(|n| !n.is_empty()).unwrap_or(key)
    }

    pub fn add_page(&mut self, name: impl Into<String>) -> NodeId {
        let id = self.alloc_id();
        let mut page = Node::new(id, NodeKind::Page { background: Color::rgb(0xF5, 0xF5, 0xF5) });
        page.name = name.into();
        self.nodes.insert(id, page);
        self.pages.push(id);
        id
    }

    /// Inserts `node` as the last (topmost) child of `parent`.
    pub fn insert(&mut self, parent: NodeId, mut node: Node) -> Result<NodeId, DocError> {
        let p = self.nodes.get_mut(&parent).ok_or(DocError::NotFound(parent))?;
        if !p.kind.is_container() {
            return Err(DocError::NotAContainer(parent));
        }
        let id = node.id;
        p.children.push(id);
        node.parent = Some(parent);
        self.next_id = self.next_id.max(id.0);
        self.nodes.insert(id, node);
        Ok(id)
    }

    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(&id)
    }

    /// Removes `id` and its subtree, returning the removed nodes (root first).
    pub fn remove(&mut self, id: NodeId) -> Result<Vec<Node>, DocError> {
        let parent = self.nodes.get(&id).ok_or(DocError::NotFound(id))?.parent;
        if let Some(p) = parent.and_then(|p| self.nodes.get_mut(&p)) {
            p.children.retain(|c| *c != id);
        }
        self.pages.retain(|p| *p != id);
        let mut out = Vec::new();
        let mut stack = vec![id];
        while let Some(n) = stack.pop() {
            if let Some(node) = self.nodes.remove(&n) {
                stack.extend(node.children.iter().rev().copied());
                out.push(node);
            }
        }
        Ok(out)
    }

    /// Moves `id` under `new_parent` at `index` (clamped).
    pub fn reparent(&mut self, id: NodeId, new_parent: NodeId, index: usize) -> Result<(), DocError> {
        if self.is_ancestor(id, new_parent) || id == new_parent {
            return Err(DocError::Cycle(id));
        }
        if !self.nodes.get(&new_parent).ok_or(DocError::NotFound(new_parent))?.kind.is_container() {
            return Err(DocError::NotAContainer(new_parent));
        }
        let old = self.nodes.get(&id).ok_or(DocError::NotFound(id))?.parent;
        if let Some(p) = old.and_then(|p| self.nodes.get_mut(&p)) {
            p.children.retain(|c| *c != id);
        }
        let p = self.nodes.get_mut(&new_parent).unwrap();
        let index = index.min(p.children.len());
        p.children.insert(index, id);
        self.nodes.get_mut(&id).unwrap().parent = Some(new_parent);
        Ok(())
    }

    pub fn is_ancestor(&self, ancestor: NodeId, mut of: NodeId) -> bool {
        while let Some(p) = self.nodes.get(&of).and_then(|n| n.parent) {
            if p == ancestor {
                return true;
            }
            of = p;
        }
        false
    }

    /// Absolute transform (page space) of a node.
    pub fn world_transform(&self, id: NodeId) -> Affine {
        let mut chain = Vec::new();
        let mut cur = Some(id);
        while let Some(c) = cur {
            let Some(n) = self.nodes.get(&c) else { break };
            chain.push(n.transform);
            cur = n.parent;
        }
        chain.iter().rev().fold(Affine::IDENTITY, |acc, t| acc * *t)
    }

    /// Page that contains `id`.
    pub fn page_of(&self, mut id: NodeId) -> Option<NodeId> {
        loop {
            let n = self.nodes.get(&id)?;
            match n.parent {
                Some(p) => id = p,
                None => return matches!(n.kind, NodeKind::Page { .. }).then_some(id),
            }
        }
    }

    /// Pre-order traversal of a subtree, back-to-front paint order.
    pub fn walk(&self, root: NodeId, mut f: impl FnMut(&Node, usize)) {
        fn rec(doc: &Document, id: NodeId, depth: usize, f: &mut dyn FnMut(&Node, usize)) {
            if let Some(n) = doc.nodes.get(&id) {
                f(n, depth);
                for c in &n.children {
                    rec(doc, *c, depth + 1, f);
                }
            }
        }
        rec(self, root, 0, &mut f);
    }

    /// Counts the number of nodes in `page`'s subtree in $O(|T_{\text{page}}|)$ time.
    pub fn page_node_count(&self, page: NodeId) -> usize {
        let mut count = 0;
        self.walk(page, |_, _| count += 1);
        count
    }

    /// Collects all [`NodeId`]s reachable from `self.pages` (following
    /// `n.children`, `NodeKind::Instance { component: Some(cid), .. }`,
    /// [`ComponentLink::source`], and [`PropValue::Instance`] /
    /// [`PropKind::InstanceSwap`]) and removes any unreachable orphan nodes
    /// from `self.nodes`, returning the number of removed nodes.
    pub fn prune_unreachable(&mut self) -> usize {
        let mut reachable = std::collections::BTreeSet::new();
        let mut stack: Vec<NodeId> = self.pages.clone();
        if let Some(lp) = self.library_page {
            stack.push(lp);
        }
        while let Some(id) = stack.pop() {
            if !reachable.insert(id) {
                continue;
            }
            if let Some(n) = self.nodes.get(&id) {
                stack.extend(n.children.iter().copied());
                if let NodeKind::Instance { component: Some(cid), .. } = &n.kind {
                    stack.push(*cid);
                }
                if let Some(src) = n.component.source {
                    stack.push(src);
                }
                for prop in &n.component.props {
                    if let PropValue::Instance(cid) = prop.default {
                        stack.push(cid);
                    }
                    if let PropKind::InstanceSwap { preferred } = &prop.kind {
                        stack.extend(preferred.iter().copied());
                    }
                }
                for val in n.component.values.values() {
                    if let PropValue::Instance(cid) = val {
                        stack.push(*cid);
                    }
                }
            }
        }
        let before = self.nodes.len();
        self.nodes.retain(|id, _| reachable.contains(id));
        before - self.nodes.len()
    }

    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Parses a native document. Text written by older versions that emulated lists with
    /// literal bullet and "1. " prefixes is converted to real lists.
    pub fn from_json(s: &str) -> serde_json::Result<Self> {
        let mut doc: Self = serde_json::from_str(s)?;
        doc.migrate_text();
        Ok(doc)
    }

    /// Converts literal list prefixes of every text node to real lists (see [`migrate_literal_lists`]).
    pub fn migrate_text(&mut self) {
        for node in self.nodes.values_mut() {
            if let NodeKind::Text(t) = &mut node.kind {
                migrate_literal_lists(t);
            }
        }
    }
}

/// Which node property a shared style applies to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StyleProperty {
    #[default]
    Fill,
    Stroke,
    Text,
    Effect,
    Grid,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct StyleMeta {
    /// Name as shown in the UI, may contain "/" for grouping ("Brand/Primary").
    pub name: String,
    /// Library key (Figma style `key`), for linking to published libraries.
    pub key: Option<String>,
    pub description: String,
    /// Property the style is used for (a `Style::Fill` can be a stroke style).
    pub property: StyleProperty,
}

/// Shared (library) style referenced by nodes via `style_refs`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Style {
    Fill(Vec<Paint>),
    Text(TextStyle),
    Effect(Vec<Effect>),
    Grid(Vec<LayoutGrid>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_reparent_remove() {
        let mut d = Document::new("t");
        let page = d.add_page("Page 1");
        let fid = d.alloc_id();
        let frame = d.insert(page, Node::frame(fid, Rect::new(0.0, 0.0, 100.0, 100.0))).unwrap();
        let rid = d.alloc_id();
        let rect = d.insert(page, Node::rectangle(rid, Rect::new(10.0, 10.0, 20.0, 20.0))).unwrap();
        d.reparent(rect, frame, 0).unwrap();
        assert_eq!(d.get(frame).unwrap().children, vec![rect]);
        assert!(d.reparent(frame, rect, 0).is_err());
        assert_eq!(d.page_of(rect), Some(page));
        let removed = d.remove(frame).unwrap();
        assert_eq!(removed.len(), 2);
        assert!(d.get(rect).is_none());
    }

    #[test]
    fn json_roundtrip() {
        let mut d = Document::new("t");
        let page = d.add_page("P");
        let id = d.alloc_id();
        d.insert(page, Node::rectangle(id, Rect::new(0.0, 0.0, 5.0, 5.0))).unwrap();
        let back = Document::from_json(&d.to_json().unwrap()).unwrap();
        assert_eq!(back.nodes.len(), 2);
    }

    #[test]
    fn component_link_roundtrip_and_default() {
        let mut d = Document::new("t");
        let page = d.add_page("P");
        let id = d.alloc_id();
        let mut n = Node::rectangle(id, Rect::new(0.0, 0.0, 5.0, 5.0));
        n.component.source = Some(NodeId(99));
        n.component.overrides.insert(OverrideField::Fills);
        n.component.bindings.insert(OverrideField::Visible, "Show icon".into());
        d.insert(page, n).unwrap();
        let json = d.to_json().unwrap();
        let back = Document::from_json(&json).unwrap();
        assert_eq!(back.get(id).unwrap().component, d.get(id).unwrap().component);
        // Plain nodes omit the field, and files without it still load.
        assert!(!json.contains("\"component\": {}"));
        let page_json = serde_json::to_string(back.get(page).unwrap()).unwrap();
        assert!(!page_json.contains("\"component\""));
    }

    #[test]
    fn images_compact_and_legacy_roundtrip() {
        let mut d = Document::new("t");
        d.add_page("P");
        d.images.insert("hash1".into(), vec![0x89, 0x50, 0x4e, 0x47]);
        let json = d.to_json().unwrap();
        assert!(json.contains("\"hex:89504e47\""), "images serialize as compact hex: {json}");
        let back = Document::from_json(&json).unwrap();
        assert_eq!(back.images.get("hash1").unwrap(), &vec![0x89, 0x50, 0x4e, 0x47]);

        // Legacy array format still loads cleanly.
        let legacy = json.replace("\"hex:89504e47\"", "[137, 80, 78, 71]");
        let from_legacy = Document::from_json(&legacy).unwrap();
        assert_eq!(from_legacy.images.get("hash1").unwrap(), &vec![0x89, 0x50, 0x4e, 0x47]);
    }

    #[test]
    fn node_default_fields_omitted_and_roundtrip() {
        let mut d = Document::new("t");
        let page = d.add_page("P");
        let id = d.alloc_id();
        d.insert(page, Node::rectangle(id, Rect::new(1.0, 2.0, 30.0, 40.0))).unwrap();
        let rect_json = serde_json::to_string(d.get(id).unwrap()).unwrap();
        for omitted in ["\"source_id\"", "\"children\"", "\"effects\"", "\"style_refs\"", "\"fill_geometry\"", "\"stroke_geometry\"", "\"corner_radii\"", "\"stroke\""] {
            assert!(!rect_json.contains(omitted), "{omitted} should be omitted on default node: {rect_json}");
        }
        let back = Document::from_json(&d.to_json().unwrap()).unwrap();
        assert_eq!(back.get(id), d.get(id));
    }

    #[test]
    fn prune_unreachable_preserves_reachable_and_removes_orphans() {
        let mut d = Document::new("t");
        let page = d.add_page("Page 1");
        let fid = d.alloc_id();
        let frame = d.insert(page, Node::frame(fid, Rect::new(0.0, 0.0, 100.0, 100.0))).unwrap();
        let rid = d.alloc_id();
        let rect = d.insert(frame, Node::rectangle(rid, Rect::new(0.0, 0.0, 10.0, 10.0))).unwrap();

        // Detached component masters & referenced targets (e.g. from hidden/imported library nodes)
        let comp_id = d.alloc_id();
        let comp_child_id = d.alloc_id();
        let swap_default_id = d.alloc_id();
        let swap_pref_id = d.alloc_id();
        let swap_val_id = d.alloc_id();
        let source_target_id = d.alloc_id();

        let mut comp_node = Node::new(comp_id, NodeKind::Component(FrameProps::default()));
        comp_node.children.push(comp_child_id);
        comp_node.component.props.push(ComponentPropDef {
            name: "Icon".into(),
            kind: PropKind::InstanceSwap { preferred: vec![swap_pref_id] },
            default: PropValue::Instance(swap_default_id),
        });
        let mut comp_child = Node::rectangle(comp_child_id, Rect::new(0.0, 0.0, 8.0, 8.0));
        comp_child.parent = Some(comp_id);

        d.nodes.insert(comp_id, comp_node);
        d.nodes.insert(comp_child_id, comp_child);
        d.nodes.insert(swap_default_id, Node::new(swap_default_id, NodeKind::Component(FrameProps::default())));
        d.nodes.insert(swap_pref_id, Node::new(swap_pref_id, NodeKind::Component(FrameProps::default())));
        d.nodes.insert(swap_val_id, Node::new(swap_val_id, NodeKind::Component(FrameProps::default())));
        d.nodes.insert(source_target_id, Node::rectangle(source_target_id, Rect::new(0.0, 0.0, 4.0, 4.0)));

        let inst_id = d.alloc_id();
        let mut inst = Node::new(
            inst_id,
            NodeKind::Instance {
                frame: FrameProps::default(),
                component: Some(comp_id),
                component_key: None,
            },
        );
        inst.component.source = Some(source_target_id);
        inst.component.values.insert("Icon".into(), PropValue::Instance(swap_val_id));
        d.insert(page, inst).unwrap();

        // Unreachable orphan subtree
        let orphan_parent = d.alloc_id();
        let orphan_child = d.alloc_id();
        let mut op = Node::frame(orphan_parent, Rect::new(0.0, 0.0, 50.0, 50.0));
        op.children.push(orphan_child);
        let mut oc = Node::rectangle(orphan_child, Rect::new(0.0, 0.0, 10.0, 10.0));
        oc.parent = Some(orphan_parent);
        d.nodes.insert(orphan_parent, op);
        d.nodes.insert(orphan_child, oc);

        let removed = d.prune_unreachable();
        assert_eq!(removed, 2);

        for kept in [
            page,
            frame,
            rect,
            inst_id,
            comp_id,
            comp_child_id,
            swap_default_id,
            swap_pref_id,
            swap_val_id,
            source_target_id,
        ] {
            assert!(d.get(kept).is_some(), "expected node {kept:?} to be preserved");
        }
        assert!(d.get(orphan_parent).is_none());
        assert!(d.get(orphan_child).is_none());
    }

    #[test]
    fn page_node_count_measures_subtree() {
        let mut d = Document::new("t");
        let p1 = d.add_page("Page 1");
        let p2 = d.add_page("Page 2");
        assert_eq!(d.page_node_count(p1), 1);
        assert_eq!(d.page_node_count(p2), 1);
        assert_eq!(d.page_node_count(NodeId(9999)), 0);

        let fid = d.alloc_id();
        let frame = d.insert(p1, Node::frame(fid, Rect::new(0.0, 0.0, 100.0, 100.0))).unwrap();
        let r1 = d.alloc_id();
        d.insert(frame, Node::rectangle(r1, Rect::new(0.0, 0.0, 10.0, 10.0))).unwrap();
        let r2 = d.alloc_id();
        d.insert(frame, Node::rectangle(r2, Rect::new(20.0, 0.0, 10.0, 10.0))).unwrap();

        assert_eq!(d.page_node_count(p1), 4);
        assert_eq!(d.page_node_count(p2), 1);
    }
}
