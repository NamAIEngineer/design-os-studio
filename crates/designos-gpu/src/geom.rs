//! Per-node geometry cache: shapes, stroke outlines and glyph outlines
//! tessellated into meshes in node-local space.
//!
//! Meshes depend only on the node's geometry-relevant fields and on a zoom
//! bucket (curve flattening accuracy), never on the node's position, so moving
//! a node or panning re-uses them as they are.

use crate::mesh::{tessellate, Mesh};
use designos_doc as d;
use designos_doc::{Document, Node, NodeId, NodeKind, Vec2};
use designos_render::tiny_skia as sk;
use designos_render::{font_generation, outline_of, stroke_outline, stroke_plan, text_outlines, NodeShape, StrokeClip};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::rc::Rc;
use std::sync::Arc;

/// Multiplicative hasher for dense integer ids and pre-hashed geometry keys.
#[derive(Default, Clone, Copy)]
pub(crate) struct IdHasher(u64);

impl Hasher for IdHasher {
    fn write(&mut self, bytes: &[u8]) {
        for b in bytes {
            self.write_u64(*b as u64);
        }
    }
    fn write_u64(&mut self, n: u64) {
        self.0 = (self.0.rotate_left(5) ^ n).wrapping_mul(0x517c_c1b7_2722_0a95);
    }
    fn write_u32(&mut self, n: u32) {
        self.write_u64(n as u64);
    }
    fn write_i32(&mut self, n: i32) {
        self.write_u64(n as u64);
    }
    fn write_u16(&mut self, n: u16) {
        self.write_u64(n as u64);
    }
    fn write_u8(&mut self, n: u8) {
        self.write_u64(n as u64);
    }
    fn write_usize(&mut self, n: usize) {
        self.write_u64(n as u64);
    }
    fn finish(&self) -> u64 {
        self.0 ^ (self.0 >> 32)
    }
}

type IdMap<K, V> = HashMap<K, V, BuildHasherDefault<IdHasher>>;

fn is_frame_like(k: &NodeKind) -> bool {
    matches!(k, NodeKind::Frame(_) | NodeKind::Component(_) | NodeKind::ComponentSet(_) | NodeKind::Instance { .. })
}

fn canonical_kind(k: &NodeKind) -> NodeKind {
    if is_frame_like(k) {
        NodeKind::Frame(d::FrameProps::default())
    } else {
        k.clone()
    }
}

fn same_kind(a: &NodeKind, b: &NodeKind) -> bool {
    if is_frame_like(a) && is_frame_like(b) {
        true
    } else {
        a == b
    }
}

#[inline]
fn fbits(v: f32) -> u32 {
    if v == 0.0 { 0 } else { v.to_bits() }
}

/// The fields of a node that determine its geometry.
struct GeomSrc {
    kind: NodeKind,
    size: Vec2,
    radii: [f32; 4],
    smoothing: f32,
    fill_geometry: Option<Vec<d::Path>>,
    stroke_geometry: Option<Vec<d::Path>>,
    stroke: StrokeKey,
    font_gen: usize,
    /// Children that feed a boolean operation (recursively).
    kids: Vec<KidSrc>,
}

#[derive(Clone, PartialEq)]
struct StrokeKey {
    weight: f32,
    side_weights: Option<[f32; 4]>,
    align: d::StrokeAlign,
    cap: d::StrokeCap,
    join: d::StrokeJoin,
    miter_limit: f32,
    dashes: Vec<f32>,
}

impl StrokeKey {
    fn of(n: &Node) -> Self {
        let s = &n.stroke;
        Self { weight: s.weight, side_weights: s.side_weights, align: s.align, cap: s.cap, join: s.join, miter_limit: s.miter_limit, dashes: s.dashes.clone() }
    }
    fn same(&self, n: &Node) -> bool {
        let s = &n.stroke;
        self.weight == s.weight
            && self.side_weights == s.side_weights
            && self.align == s.align
            && self.cap == s.cap
            && self.join == s.join
            && self.miter_limit == s.miter_limit
            && self.dashes == s.dashes
    }
}

struct KidSrc {
    id: NodeId,
    visible: bool,
    transform: d::Affine,
    src: GeomSrc,
}

fn feeds_boolean(n: &Node) -> bool {
    matches!(n.kind, NodeKind::BooleanOp(_) | NodeKind::Group)
}

impl GeomSrc {
    fn shape_hash(n: &Node, bucket: i32, font_gen: usize) -> u64 {
        let mut h = IdHasher::default();
        let canonical_frame = NodeKind::Frame(d::FrameProps::default());
        let disc = if is_frame_like(&n.kind) {
            std::mem::discriminant(&canonical_frame)
        } else {
            std::mem::discriminant(&n.kind)
        };
        (
            bucket,
            font_gen,
            fbits(n.size.x),
            fbits(n.size.y),
            n.corner_radii.map(fbits),
            fbits(n.corner_smoothing),
            fbits(n.stroke.weight),
            disc,
        )
            .hash(&mut h);
        match &n.kind {
            NodeKind::Text(tp) => {
                (&tp.content, fbits(tp.style.font_size), &tp.style.font_family, tp.style.font_weight, tp.runs.len()).hash(&mut h);
            }
            NodeKind::Vector { paths, .. } => {
                (paths.len(), paths.first().map(|p| p.cmds.len()).unwrap_or(0)).hash(&mut h);
            }
            _ => {}
        }
        h.finish()
    }

    fn build(doc: &Document, n: &Node, nested: bool) -> Self {
        let mut kids = Vec::new();
        if feeds_boolean(n) && (nested || matches!(n.kind, NodeKind::BooleanOp(_))) {
            for c in &n.children {
                if let Some(child) = doc.get(*c) {
                    kids.push(KidSrc { id: *c, visible: child.visible, transform: child.transform, src: GeomSrc::build(doc, child, true) });
                }
            }
        }
        Self {
            kind: canonical_kind(&n.kind),
            size: n.size,
            radii: n.corner_radii,
            smoothing: n.corner_smoothing,
            fill_geometry: n.fill_geometry.clone(),
            stroke_geometry: n.stroke_geometry.clone(),
            stroke: StrokeKey::of(n),
            font_gen: font_generation(),
            kids,
        }
    }

    fn same(&self, doc: &Document, n: &Node, nested: bool) -> bool {
        if !same_kind(&self.kind, &n.kind)
            || self.size != n.size
            || self.radii != n.corner_radii
            || self.smoothing != n.corner_smoothing
            || self.fill_geometry != n.fill_geometry
            || self.stroke_geometry != n.stroke_geometry
            || !self.stroke.same(n)
            || (matches!(n.kind, NodeKind::Text(_)) && self.font_gen != font_generation())
        {
            return false;
        }
        let want_kids = feeds_boolean(n) && (nested || matches!(n.kind, NodeKind::BooleanOp(_)));
        if !want_kids {
            return self.kids.is_empty();
        }
        if self.kids.len() != n.children.len() {
            return false;
        }
        self.kids.iter().zip(&n.children).all(|(k, c)| {
            k.id == *c && doc.get(*c).is_some_and(|child| child.visible == k.visible && child.transform == k.transform && k.src.same(doc, child, true))
        })
    }
}

/// Glyph outlines of one text style.
pub struct TextGeom {
    pub mesh: Arc<Mesh>,
    pub fills: Vec<d::Paint>,
    path: sk::Path,
    /// Centered stroke outline of the glyphs (text strokes are always centered).
    stroke: RefCell<Option<Option<Arc<Mesh>>>>,
}

type StrokeMesh = Option<(Arc<Mesh>, StrokeClip)>;

/// Cached geometry of one node at one zoom bucket.
pub struct NodeGeom {
    pub shape: Option<NodeShape>,
    /// Fill mesh of `shape.paths`.
    pub fill: Option<Arc<Mesh>>,
    stroke: RefCell<Option<StrokeMesh>>,
    pub text: Vec<TextGeom>,
    /// Device scale the meshes were flattened for.
    pub scale: f32,
    lazy: RefCell<Lazy>,
}

#[derive(Default)]
struct Lazy {
    /// (width bits, mesh) of silhouette outsets and growth outlines.
    outlines: Vec<(u32, Option<Arc<Mesh>>)>,
}

impl NodeGeom {
    /// Stroke outline mesh and how it is limited to the shape (built on first use).
    pub fn stroke_mesh(&self, n: &Node) -> Option<(Arc<Mesh>, StrokeClip)> {
        if let Some(c) = self.stroke.borrow().as_ref() {
            return c.clone();
        }
        let built = self
            .shape
            .as_ref()
            .and_then(|sh| stroke_plan(n, sh, self.scale))
            .and_then(|plan| tessellate(&plan.outline, 0.1 / self.scale).map(|m| (Arc::new(m), plan.clip)));
        *self.stroke.borrow_mut() = Some(built.clone());
        built
    }

    /// Centered stroke outline of text style `i`.
    pub fn text_stroke(&self, i: usize, n: &Node) -> Option<Arc<Mesh>> {
        let t = self.text.get(i)?;
        if let Some(c) = t.stroke.borrow().as_ref() {
            return c.clone();
        }
        let st = designos_render::node_stroke_style(n, n.stroke.weight);
        let built = stroke_outline(&t.path, &st, self.scale).and_then(|p| tessellate(&[(p, sk::FillRule::Winding)], 0.1 / self.scale)).map(Arc::new);
        *t.stroke.borrow_mut() = Some(built.clone());
        built
    }

    /// Mitered outline `width` wide around the shape's paths (drop-shadow
    /// silhouettes: stroke footprint and spread). Cached per width.
    pub fn grow_outline(&self, width: f32) -> Option<Arc<Mesh>> {
        let sh = self.shape.as_ref()?;
        let key = width.to_bits();
        if let Some((_, m)) = self.lazy.borrow().outlines.iter().find(|(k, _)| *k == key) {
            return m.clone();
        }
        let tol = 0.1 / self.scale;
        let outline: Vec<_> =
            sh.paths.iter().filter_map(|(p, _)| designos_render::miter_outline(p, width, self.scale)).map(|p| (p, sk::FillRule::Winding)).collect();
        let mesh = tessellate(&outline, tol).map(Arc::new);
        self.lazy.borrow_mut().outlines.push((key, mesh.clone()));
        mesh
    }

    /// Mesh of a precomputed stroke outline used as silhouette footprint.
    pub fn stroke_paths_mesh(&self) -> Option<Arc<Mesh>> {
        let sh = self.shape.as_ref()?;
        let sp = sh.stroke_paths.as_ref()?;
        let key = u32::MAX;
        if let Some((_, m)) = self.lazy.borrow().outlines.iter().find(|(k, _)| *k == key) {
            return m.clone();
        }
        let mesh = tessellate(sp, 0.1 / self.scale).map(Arc::new);
        self.lazy.borrow_mut().outlines.push((key, mesh.clone()));
        mesh
    }
}

struct Entry {
    src: Rc<GeomSrc>,
    bucket: i32,
    rev: u64,
    geom: Rc<NodeGeom>,
}

/// Cache of [`NodeGeom`] per node.
#[derive(Default)]
pub struct GeomCache {
    entries: IdMap<NodeId, Entry>,
    shared: IdMap<u64, Vec<(Rc<GeomSrc>, Rc<NodeGeom>)>>,
    /// Statistics: geometry builds since creation.
    pub builds: u64,
    /// Statistics: canonical geometry deduplication hits across nodes.
    pub shared_hits: u64,
    last_trim: u64,
    /// Time one frame may spend building/re-flattening geometry; past it,
    /// uncached complex nodes or changed zoom buckets wait for the next frame.
    pub budget: Option<std::time::Duration>,
    frame_build_nanos: u64,
    /// Nodes that kept stale-bucket or deferred geometry in the current frame.
    pub stale_hits: u32,
}

/// Zoom bucket for a device scale: the power of two at or above it.
pub fn bucket_of(scale: f32) -> i32 {
    let s = if scale.is_finite() && scale > 0.0 { scale } else { 1.0 };
    (s.log2().ceil() as i32).clamp(-8, 10)
}

pub fn bucket_scale(bucket: i32) -> f32 {
    (bucket as f32).exp2()
}

impl GeomCache {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.shared.clear();
    }

    /// Starts a frame: resets the re-tessellation budget.
    pub fn begin_frame(&mut self) {
        self.frame_build_nanos = 0;
        self.stale_hits = 0;
    }

    fn over_budget(&self) -> bool {
        match self.budget {
            Some(b) => self.frame_build_nanos >= b.as_nanos() as u64,
            None => false,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Geometry of `n` (already variable-resolved) for the given bucket.
    /// `rev` is the document revision: unchanged entries are not re-checked
    /// within the same revision.
    pub fn get(&mut self, doc: &Document, n: &Node, bucket: i32, rev: u64) -> Rc<NodeGeom> {
        let over = self.over_budget();
        if let Some(e) = self.entries.get_mut(&n.id) {
            if e.bucket == bucket && e.rev == rev {
                return e.geom.clone();
            }
            if e.bucket != bucket && e.rev == rev && over {
                // same document state, only the zoom bucket moved: reuse until budget allows
                self.stale_hits += 1;
                return e.geom.clone();
            }
            if e.src.same(doc, n, false) {
                if e.bucket == bucket {
                    e.rev = rev;
                    return e.geom.clone();
                }
                if over {
                    e.rev = rev;
                    self.stale_hits += 1;
                    return e.geom.clone();
                }
            }
        }
        let shared_key = (!feeds_boolean(n)).then(|| GeomSrc::shape_hash(n, bucket, font_generation()));
        if let Some(h) = shared_key {
            if let Some(bucket_vec) = self.shared.get(&h) {
                if let Some((src, geom)) = bucket_vec.iter().find(|(src, _)| src.same(doc, n, false)) {
                    let src = src.clone();
                    let geom = geom.clone();
                    self.shared_hits += 1;
                    self.entries.insert(n.id, Entry { src, bucket, rev, geom: geom.clone() });
                    return geom;
                }
            }
        }
        if over && matches!(n.kind, NodeKind::Text(_) | NodeKind::Vector { .. } | NodeKind::BooleanOp(_)) {
            self.stale_hits += 1;
            let scale = bucket_scale(bucket);
            return Rc::new(NodeGeom { shape: None, fill: None, stroke: RefCell::default(), text: Vec::new(), scale, lazy: RefCell::default() });
        }
        self.builds += 1;
        let scale = bucket_scale(bucket);
        let t0 = std::time::Instant::now();
        let geom = Rc::new(build_geom(doc, n, scale));
        self.frame_build_nanos = self.frame_build_nanos.saturating_add(t0.elapsed().as_nanos() as u64);
        let src = Rc::new(GeomSrc::build(doc, n, false));
        if let Some(h) = shared_key {
            let bucket_vec = self.shared.entry(h).or_default();
            if bucket_vec.len() < 8 {
                bucket_vec.push((src.clone(), geom.clone()));
            }
        }
        self.entries.insert(n.id, Entry { src, bucket, rev, geom: geom.clone() });
        geom
    }

    /// Drops entries not touched since `rev - keep` revisions (nodes deleted from
    /// the document, or far from the viewport for a long time).
    pub fn trim(&mut self, rev: u64) {
        if self.entries.len() > 20_000 && rev != self.last_trim {
            self.last_trim = rev;
            self.entries.retain(|_, e| e.rev + 64 >= rev);
            self.shared.clear();
        }
    }
}

fn build_geom(doc: &Document, n: &Node, scale: f32) -> NodeGeom {
    let tol = 0.1 / scale;
    let mut g = NodeGeom { shape: None, fill: None, stroke: RefCell::default(), text: Vec::new(), scale, lazy: RefCell::default() };
    if let NodeKind::Text(tp) = &n.kind {
        for o in text_outlines(tp, n.size) {
            let Some(mesh) = tessellate(&[(o.path.clone(), sk::FillRule::Winding)], tol) else { continue };
            g.text.push(TextGeom { mesh: Arc::new(mesh), fills: o.fills, path: o.path, stroke: RefCell::default() });
        }
        return g;
    }
    // boolean operations flatten with a tolerance tied to the device scale
    let Some(shape) = outline_of(doc, n, 0.15 / scale) else { return g };
    g.fill = tessellate(&shape.paths, tol).map(Arc::new);
    g.shape = Some(shape);
    g
}

#[cfg(test)]
mod tests {
    use super::*;
    use designos_doc::{AutoLayout, Color, FrameProps, LayoutGrid, Paint, Path, PathCmd, Rect, TextStyle};

    #[test]
    fn canonical_kind_deduplicates_frames_components_and_instances() {
        let mut doc = Document::new("test");
        let page = doc.add_page("Page 1");

        let comp_id = doc.alloc_id();
        let mut comp = Node::new(comp_id, NodeKind::Component(FrameProps { clips_content: true, auto_layout: Some(AutoLayout::default()), grids: Vec::new() }));
        comp.size = Vec2::new(120.0, 44.0);
        comp.corner_radii = [8.0; 4];
        comp.corner_smoothing = 0.6;
        doc.insert(page, comp).unwrap();

        let inst_id = doc.alloc_id();
        let mut inst = Node::new(
            inst_id,
            NodeKind::Instance {
                frame: FrameProps { clips_content: false, auto_layout: None, grids: vec![LayoutGrid::Grid { size: 8.0, color: Color::BLACK }] },
                component: Some(comp_id),
                component_key: Some("btn-primary".into()),
            },
        );
        inst.size = Vec2::new(120.0, 44.0);
        inst.corner_radii = [8.0; 4];
        inst.corner_smoothing = 0.6;
        doc.insert(page, inst).unwrap();

        let frame_id = doc.alloc_id();
        let mut frame = Node::frame(frame_id, Rect::new(200.0, 50.0, 120.0, 44.0));
        frame.corner_radii = [8.0; 4];
        frame.corner_smoothing = 0.6;
        frame.fills = vec![Paint::solid(Color::rgb(0x20, 0x40, 0x80))];
        doc.insert(page, frame).unwrap();

        let mut cache = GeomCache::new();
        let g_comp = cache.get(&doc, doc.get(comp_id).unwrap(), 0, 1);
        let g_inst = cache.get(&doc, doc.get(inst_id).unwrap(), 0, 1);
        let g_frame = cache.get(&doc, doc.get(frame_id).unwrap(), 0, 1);

        assert_eq!(cache.builds, 1);
        assert_eq!(cache.shared_hits, 2);
        assert!(Rc::ptr_eq(&g_comp, &g_inst));
        assert!(Rc::ptr_eq(&g_comp, &g_frame));

        // Mutating non-geometric FrameProps on frame_id does not trigger a rebuild on rev 2.
        if let NodeKind::Frame(fp) = &mut doc.get_mut(frame_id).unwrap().kind {
            fp.clips_content = true;
            fp.auto_layout = Some(AutoLayout::default());
        }
        let g_frame2 = cache.get(&doc, doc.get(frame_id).unwrap(), 0, 2);
        assert_eq!(cache.builds, 1);
        assert!(Rc::ptr_eq(&g_frame, &g_frame2));
    }

    #[test]
    fn shared_pool_deduplicates_repeated_text_and_vectors() {
        let mut doc = Document::new("test");
        let page = doc.add_page("Page 1");

        let t1_id = doc.alloc_id();
        let t1 = Node::text(t1_id, Rect::new(0.0, 0.0, 100.0, 24.0), "Submit", TextStyle::default());
        doc.insert(page, t1).unwrap();

        let t2_id = doc.alloc_id();
        let t2 = Node::text(t2_id, Rect::new(150.0, 40.0, 100.0, 24.0), "Submit", TextStyle::default());
        doc.insert(page, t2).unwrap();

        let path = Path {
            cmds: vec![PathCmd::MoveTo(Vec2::new(0.0, 0.0)), PathCmd::LineTo(Vec2::new(16.0, 0.0)), PathCmd::LineTo(Vec2::new(8.0, 16.0)), PathCmd::Close],
            fill_rule: d::FillRule::NonZero,
        };
        let v1_id = doc.alloc_id();
        let mut v1 = Node::new(v1_id, NodeKind::Vector { paths: vec![path.clone()], network: None });
        v1.size = Vec2::new(16.0, 16.0);
        doc.insert(page, v1).unwrap();

        let v2_id = doc.alloc_id();
        let mut v2 = Node::new(v2_id, NodeKind::Vector { paths: vec![path], network: None });
        v2.size = Vec2::new(16.0, 16.0);
        v2.component.source = Some(v1_id);
        doc.insert(page, v2).unwrap();

        let mut cache = GeomCache::new();
        let gt1 = cache.get(&doc, doc.get(t1_id).unwrap(), 0, 1);
        let gt2 = cache.get(&doc, doc.get(t2_id).unwrap(), 0, 1);
        let gv1 = cache.get(&doc, doc.get(v1_id).unwrap(), 0, 1);
        let gv2 = cache.get(&doc, doc.get(v2_id).unwrap(), 0, 1);

        assert_eq!(cache.builds, 2);
        assert_eq!(cache.shared_hits, 2);
        assert!(Rc::ptr_eq(&gt1, &gt2));
        assert!(Rc::ptr_eq(&gv1, &gv2));
    }
}
