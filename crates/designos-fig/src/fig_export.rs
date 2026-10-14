//! `.fig` (fig-kiwi) binary writer: [`Document`] -> `.fig` archive bytes.
//!
//! Produces a standard Figma local-copy ZIP archive (`canvas.fig` + `meta.json` +
//! `images/<hash>`) whose `canvas.fig` payload is a `fig-kiwi` container encoded
//! against [`crate::testkit::figma_like_schema`].

use crate::container::{self, Compression};
use crate::kiwi::KiwiValue as V;
use crate::rest_import::utf16_to_byte_offsets;
use crate::testkit::{color, en, figma_like_schema, fl, guid, msg, st, vec2};
use crate::FigError;
use designos_doc::*;
use serde_json::json;
use std::collections::{BTreeMap, HashSet};

/// Encodes a [`Path`] into Figma's binary path-commands blob format
/// (`0` = Close, `1` = MoveTo, `2` = LineTo, `3` = QuadTo, `4` = CubicTo,
/// little-endian `f32` coordinates). Exact inverse of [`crate::commands_blob_to_path`].
pub fn path_to_commands_blob(path: &Path) -> Vec<u8> {
    let mut out = Vec::new();
    let push_pt = |out: &mut Vec<u8>, p: Vec2| {
        out.extend_from_slice(&p.x.to_le_bytes());
        out.extend_from_slice(&p.y.to_le_bytes());
    };
    for cmd in &path.cmds {
        match *cmd {
            PathCmd::Close => out.push(0),
            PathCmd::MoveTo(p) => {
                out.push(1);
                push_pt(&mut out, p);
            }
            PathCmd::LineTo(p) => {
                out.push(2);
                push_pt(&mut out, p);
            }
            PathCmd::QuadTo(c, p) => {
                out.push(3);
                push_pt(&mut out, c);
                push_pt(&mut out, p);
            }
            PathCmd::CubicTo(c1, c2, p) => {
                out.push(4);
                push_pt(&mut out, c1);
                push_pt(&mut out, c2);
                push_pt(&mut out, p);
            }
        }
    }
    out
}

/// Serializes `doc` into a `.fig` binary archive (ZIP containing `canvas.fig`,
/// `meta.json`, and `images/<hash>`).
pub fn export_fig(doc: &Document) -> Result<Vec<u8>, FigError> {
    let schema = figma_like_schema();
    let root = schema.find("Message").ok_or_else(|| FigError::Format("missing Message in schema".into()))?;
    let message = build_message(doc);
    let schema_bytes = schema.encode();
    let msg_bytes = schema.encode_message(root, &message).map_err(|e| FigError::Format(format!("kiwi encode: {e}")))?;
    let canvas = container::write_container(&schema_bytes, &msg_bytes, Compression::Zstd, 106);
    let meta = json!({
        "file_name": doc.name,
        "client_meta": {
            "background_color": {"r": 0.9, "g": 0.9, "b": 0.9, "a": 1.0},
            "thumbnail_size": {"width": 400, "height": 200}
        }
    });
    Ok(container::write_zip(&canvas, &doc.images, Some(&meta)))
}

// ------------------------------------------------------------------ state & IDs

struct ExportCtx<'a> {
    doc: &'a Document,
    used_guids: HashSet<(u32, u32)>,
    next_local: u32,
    node_guids: BTreeMap<NodeId, (u32, u32)>,
    coll_guids: BTreeMap<String, (u32, u32)>,
    mode_guids: BTreeMap<String, (u32, u32)>,
    var_guids: BTreeMap<String, (u32, u32)>,
    style_guids: BTreeMap<String, (u32, u32)>,
    blobs: Vec<Vec<u8>>,
    changes: Vec<V>,
}

fn parse_guid_pair(s: &str) -> Option<(u32, u32)> {
    let s = s.strip_prefix("VariableID:").or_else(|| s.strip_prefix("VariableCollectionId:")).unwrap_or(s);
    let (a, b) = s.split_once(':')?;
    if b.contains(':') || a.contains(';') || b.contains(';') {
        return None;
    }
    let session = a.parse::<u32>().ok()?;
    let local = b.parse::<u32>().ok()?;
    if session == 0 && (local == 0 || local == 999_999) {
        return None;
    }
    Some((session, local))
}

impl<'a> ExportCtx<'a> {
    fn new(doc: &'a Document) -> Self {
        let mut used = HashSet::new();
        used.insert((0, 0));
        used.insert((0, 999_999));
        let mut cx = Self {
            doc,
            used_guids: used,
            next_local: 1,
            node_guids: BTreeMap::new(),
            coll_guids: BTreeMap::new(),
            mode_guids: BTreeMap::new(),
            var_guids: BTreeMap::new(),
            style_guids: BTreeMap::new(),
            blobs: Vec::new(),
            changes: Vec::new(),
        };
        // First pass: reserve explicit `s:l` GUIDs so generated ones never collide.
        for (&id, n) in &doc.nodes {
            if let Some(g) = n.source_id.as_deref().and_then(parse_guid_pair) {
                if cx.used_guids.insert(g) {
                    cx.next_local = cx.next_local.max(g.1.saturating_add(1));
                    cx.node_guids.insert(id, g);
                }
            }
        }
        for id in doc.variable_collections.keys() {
            if let Some(g) = parse_guid_pair(id) {
                if cx.used_guids.insert(g) {
                    cx.next_local = cx.next_local.max(g.1.saturating_add(1));
                    cx.coll_guids.insert(id.clone(), g);
                }
            }
        }
        for c in doc.variable_collections.values() {
            for m in &c.modes {
                if let Some(g) = parse_guid_pair(&m.id) {
                    if cx.used_guids.insert(g) {
                        cx.next_local = cx.next_local.max(g.1.saturating_add(1));
                        cx.mode_guids.insert(m.id.clone(), g);
                    }
                }
            }
        }
        for id in doc.variables.keys() {
            if let Some(g) = parse_guid_pair(id) {
                if cx.used_guids.insert(g) {
                    cx.next_local = cx.next_local.max(g.1.saturating_add(1));
                    cx.var_guids.insert(id.clone(), g);
                }
            }
        }
        for id in doc.styles.keys() {
            if let Some(g) = parse_guid_pair(id) {
                if cx.used_guids.insert(g) {
                    cx.next_local = cx.next_local.max(g.1.saturating_add(1));
                    cx.style_guids.insert(id.clone(), g);
                }
            }
        }
        // Second pass: allocate fresh GUIDs for anything without an explicit `s:l`.
        for &id in doc.nodes.keys() {
            if !cx.node_guids.contains_key(&id) {
                let g = cx.alloc_guid(1);
                cx.node_guids.insert(id, g);
            }
        }
        for id in doc.variable_collections.keys() {
            if !cx.coll_guids.contains_key(id) {
                let g = cx.alloc_guid(3);
                cx.coll_guids.insert(id.clone(), g);
            }
        }
        for c in doc.variable_collections.values() {
            for m in &c.modes {
                if !cx.mode_guids.contains_key(&m.id) {
                    let g = cx.alloc_guid(3);
                    cx.mode_guids.insert(m.id.clone(), g);
                }
            }
        }
        for id in doc.variables.keys() {
            if !cx.var_guids.contains_key(id) {
                let g = cx.alloc_guid(3);
                cx.var_guids.insert(id.clone(), g);
            }
        }
        for id in doc.styles.keys() {
            if !cx.style_guids.contains_key(id) {
                let g = cx.alloc_guid(2);
                cx.style_guids.insert(id.clone(), g);
            }
        }
        cx
    }

    fn alloc_guid(&mut self, session: u32) -> (u32, u32) {
        loop {
            let g = (session, self.next_local);
            self.next_local += 1;
            if self.used_guids.insert(g) {
                return g;
            }
        }
    }

    fn add_blob(&mut self, bytes: Vec<u8>) -> u32 {
        let idx = self.blobs.len() as u32;
        self.blobs.push(bytes);
        idx
    }

    fn resolve_var_guid(&self, id: &str) -> Option<(u32, u32)> {
        self.var_guids
            .get(id)
            .copied()
            .or_else(|| self.var_guids.get(&format!("VariableID:{id}")).copied())
            .or_else(|| parse_guid_pair(id))
    }

    fn resolve_coll_guid(&self, id: &str) -> Option<(u32, u32)> {
        self.coll_guids
            .get(id)
            .copied()
            .or_else(|| self.coll_guids.get(&format!("VariableCollectionId:{id}")).copied())
            .or_else(|| parse_guid_pair(id))
    }
}

fn pos_str(idx: usize) -> String {
    format!("!{idx:06}")
}

fn build_message(doc: &Document) -> V {
    let mut cx = ExportCtx::new(doc);
    // 1. Root DOCUMENT record.
    cx.changes.push(msg(vec![
        ("guid", guid(0, 0)),
        ("phase", en("CREATED")),
        ("type", en("DOCUMENT")),
        ("name", V::String(doc.name.clone())),
    ]));

    // 2. User pages (CANVAS records) and their subtrees.
    for (pidx, &pid) in doc.pages.iter().enumerate() {
        let Some(page) = doc.get(pid) else { continue };
        let pguid = cx.node_guids[&pid];
        let bg = match page.kind {
            NodeKind::Page { background } => background,
            _ => Color::rgb(0xE5, 0xE5, 0xE5),
        };
        let mut fields = vec![
            ("guid", guid(pguid.0, pguid.1)),
            ("phase", en("CREATED")),
            ("parentIndex", st(vec![("guid", guid(0, 0)), ("position", V::String(pos_str(pidx)))])),
            ("type", en("CANVAS")),
            ("name", V::String(page.name.clone())),
            ("backgroundColor", color(bg.r, bg.g, bg.b, bg.a)),
        ];
        if let Some(guides) = doc.guides.get(&pid) {
            let page_guides: Vec<V> = guides
                .iter()
                .filter(|g| g.frame.is_none())
                .map(|g| msg(vec![("axis", en(if g.axis == GuideAxis::X { "X" } else { "Y" })), ("offset", fl(g.offset))]))
                .collect();
            if !page_guides.is_empty() {
                fields.push(("guides", V::Array(page_guides)));
            }
        }
        let handoff = collect_page_handoff(&cx, pid);
        if !handoff.is_empty() {
            fields.push(("handoffStatusMap", msg(vec![("entries", V::Array(handoff))])));
        }
        cx.changes.push(msg(fields));
        let children = page.children.clone();
        for (cidx, cid) in children.into_iter().enumerate() {
            export_node_rec(&mut cx, cid, pguid, cidx, None, false);
        }
    }

    // 3. Internal-only canvas for variables, styles, and library components.
    let has_internal = !doc.variable_collections.is_empty() || !doc.variables.is_empty() || !doc.styles.is_empty() || doc.library_page.is_some();
    if has_internal {
        let iguid = (0, 999_999);
        cx.changes.push(msg(vec![
            ("guid", guid(iguid.0, iguid.1)),
            ("phase", en("CREATED")),
            ("parentIndex", st(vec![("guid", guid(0, 0)), ("position", V::String(pos_str(doc.pages.len())))])),
            ("type", en("CANVAS")),
            ("name", V::String("Internal Only Canvas".into())),
            ("internalOnly", V::Bool(true)),
        ]));
        let mut internal_idx = 0usize;
        export_variables_on_internal(&mut cx, iguid, &mut internal_idx);
        export_styles_on_internal(&mut cx, iguid, &mut internal_idx);
        if let Some(lp) = doc.library_page {
            if let Some(lib_page) = doc.get(lp) {
                let kids = lib_page.children.clone();
                for cid in kids {
                    export_node_rec(&mut cx, cid, iguid, internal_idx, None, false);
                    internal_idx += 1;
                }
            }
        }
    }

    let blobs: Vec<V> = cx.blobs.into_iter().map(|b| msg(vec![("bytes", V::Bytes(b))])).collect();
    msg(vec![
        ("type", en("NODE_CHANGES")),
        ("sessionID", V::Uint(0)),
        ("ackID", V::Uint(0)),
        ("nodeChanges", V::Array(cx.changes)),
        ("blobs", V::Array(blobs)),
    ])
}

fn collect_page_handoff(cx: &ExportCtx, page: NodeId) -> Vec<V> {
    let mut entries = Vec::new();
    let mut stack: Vec<NodeId> = cx.doc.get(page).map(|n| n.children.iter().rev().copied().collect()).unwrap_or_default();
    while let Some(id) = stack.pop() {
        let Some(n) = cx.doc.get(id) else { continue };
        let status = match n.dev.status {
            Some(DevStatus::ReadyForDev | DevStatus::Changed) => Some("READY_FOR_DEV"),
            Some(DevStatus::Completed) => Some("COMPLETED"),
            None => None,
        };
        if let Some(st_name) = status {
            let g = cx.node_guids[&id];
            entries.push(msg(vec![("guid", guid(g.0, g.1)), ("handoffStatus", msg(vec![("status", en(st_name))]))]));
        }
        stack.extend(n.children.iter().rev().copied());
    }
    entries
}

fn export_variables_on_internal(cx: &mut ExportCtx, internal_guid: (u32, u32), idx: &mut usize) {
    for c in cx.doc.variable_collections.values() {
        let cguid = cx.coll_guids[&c.id];
        // Ensure default_mode sorts first so `variables::from_fig_entries` picks it as `defaultModeId`.
        let mut ordered_modes: Vec<&VariableMode> = Vec::with_capacity(c.modes.len());
        if let Some(dm) = c.modes.iter().find(|m| m.id == c.default_mode) {
            ordered_modes.push(dm);
        }
        for m in &c.modes {
            if m.id != c.default_mode {
                ordered_modes.push(m);
            }
        }
        let modes: Vec<V> = ordered_modes
            .iter()
            .enumerate()
            .map(|(mi, m)| {
                let mguid = cx.mode_guids[&m.id];
                msg(vec![("id", guid(mguid.0, mguid.1)), ("name", V::String(m.name.clone())), ("sortPosition", V::String(pos_str(mi)))])
            })
            .collect();
        let mut fields = vec![
            ("guid", guid(cguid.0, cguid.1)),
            ("phase", en("CREATED")),
            ("parentIndex", st(vec![("guid", guid(internal_guid.0, internal_guid.1)), ("position", V::String(pos_str(*idx)))])),
            ("type", en("VARIABLE_SET")),
            ("name", V::String(c.name.clone())),
            ("variableSetModes", V::Array(modes)),
        ];
        if let Some(k) = &c.key {
            fields.push(("key", V::String(k.clone())));
        }
        cx.changes.push(msg(fields));
        *idx += 1;
    }

    for (vi, v) in cx.doc.variables.values().enumerate() {
        let vguid = cx.var_guids[&v.id];
        let Some(cguid) = cx.resolve_coll_guid(&v.collection) else { continue };
        let rtype = match v.kind {
            VariableKind::Color => "COLOR",
            VariableKind::Number => "FLOAT",
            VariableKind::String => "STRING",
            VariableKind::Boolean => "BOOLEAN",
        };
        let mut val_entries = Vec::new();
        for (mode_id, val) in &v.values {
            let Some(&mguid) = cx.mode_guids.get(mode_id) else { continue };
            let vdata = match val {
                VariableValue::Color(c) => msg(vec![("dataType", en("COLOR")), ("value", msg(vec![("colorValue", color(c.r, c.g, c.b, c.a))]))]),
                VariableValue::Number(f) => msg(vec![("dataType", en("FLOAT")), ("value", msg(vec![("floatValue", fl(*f))]))]),
                VariableValue::String(s) => msg(vec![("dataType", en("STRING")), ("value", msg(vec![("textValue", V::String(s.clone()))]))]),
                VariableValue::Boolean(b) => msg(vec![("dataType", en("BOOLEAN")), ("value", msg(vec![("boolValue", V::Bool(*b))]))]),
                VariableValue::Alias(target) => {
                    let Some(tguid) = cx.resolve_var_guid(target) else { continue };
                    msg(vec![("dataType", en("ALIAS")), ("value", msg(vec![("alias", msg(vec![("guid", guid(tguid.0, tguid.1))]))]))])
                }
            };
            val_entries.push(msg(vec![("modeID", guid(mguid.0, mguid.1)), ("variableData", vdata)]));
        }
        let scopes: Vec<V> = v.scopes.iter().map(|s| V::String(s.clone())).collect();
        let mut fields = vec![
            ("guid", guid(vguid.0, vguid.1)),
            ("phase", en("CREATED")),
            ("parentIndex", st(vec![("guid", guid(internal_guid.0, internal_guid.1)), ("position", V::String(pos_str(*idx)))])),
            ("type", en("VARIABLE")),
            ("name", V::String(v.name.clone())),
            ("description", V::String(v.description.clone())),
            ("sortPosition", V::String(pos_str(vi))),
            ("variableSetID", msg(vec![("guid", guid(cguid.0, cguid.1))])),
            ("variableResolvedType", en(rtype)),
            ("variableDataValues", msg(vec![("entries", V::Array(val_entries))])),
        ];
        if !scopes.is_empty() {
            fields.push(("variableScopes", V::Array(scopes)));
        }
        if let Some(k) = &v.key {
            fields.push(("key", V::String(k.clone())));
        }
        cx.changes.push(msg(fields));
        *idx += 1;
    }
}

fn export_styles_on_internal(cx: &mut ExportCtx, internal_guid: (u32, u32), idx: &mut usize) {
    let styles: Vec<(String, Style)> = cx.doc.styles.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    for (sid, style) in styles {
        let sguid = cx.style_guids[&sid];
        let meta = cx.doc.style_meta.get(&sid);
        let name = cx.doc.style_name(&sid).to_string();
        let desc = meta.map(|m| m.description.clone()).unwrap_or_default();
        let (node_ty, style_ty) = match &style {
            Style::Fill(_) => ("RECTANGLE", "FILL"),
            Style::Text(_) => ("TEXT", "TEXT"),
            Style::Effect(_) => ("RECTANGLE", "EFFECT"),
            Style::Grid(_) => ("FRAME", "GRID"),
        };
        let mut fields = vec![
            ("guid", guid(sguid.0, sguid.1)),
            ("phase", en("CREATED")),
            ("parentIndex", st(vec![("guid", guid(internal_guid.0, internal_guid.1)), ("position", V::String(pos_str(*idx)))])),
            ("type", en(node_ty)),
            ("name", V::String(name)),
            ("styleType", en(style_ty)),
            ("styleDescription", V::String(desc)),
            ("size", vec2(1.0, 1.0)),
        ];
        if let Some(k) = meta.and_then(|m| m.key.as_ref()) {
            fields.push(("key", V::String(k.clone())));
        }
        match &style {
            Style::Fill(paints) => {
                fields.push(("fillPaints", V::Array(paints.iter().map(|p| encode_paint(p, None)).collect())));
            }
            Style::Text(ts) => {
                append_text_style_fields(&mut fields, ts);
            }
            Style::Effect(effects) => {
                fields.push(("effects", V::Array(effects.iter().filter_map(|e| encode_effect(e, None)).collect())));
            }
            Style::Grid(grids) => {
                fields.push(("layoutGrids", V::Array(grids.iter().map(encode_grid).collect())));
            }
        }
        cx.changes.push(msg(fields));
        *idx += 1;
    }
}

fn export_node_rec(
    cx: &mut ExportCtx,
    id: NodeId,
    parent_guid: (u32, u32),
    sibling_idx: usize,
    parent_dir: Option<LayoutDirection>,
    in_component: bool,
) {
    let Some(node) = cx.doc.get(id).cloned() else { return };
    let nguid = cx.node_guids[&id];
    let (ty, is_state_group) = match &node.kind {
        NodeKind::Page { .. } => ("CANVAS", false),
        NodeKind::Frame(_) => ("FRAME", false),
        NodeKind::Group => ("GROUP", false),
        NodeKind::Section => ("SECTION", false),
        NodeKind::Rectangle => ("ROUNDED_RECTANGLE", false),
        NodeKind::Ellipse(_) => ("ELLIPSE", false),
        NodeKind::Line => ("LINE", false),
        NodeKind::Polygon { .. } => ("REGULAR_POLYGON", false),
        NodeKind::Star { .. } => ("STAR", false),
        NodeKind::Vector { .. } => ("VECTOR", false),
        NodeKind::Text(_) => ("TEXT", false),
        NodeKind::BooleanOp(_) => ("BOOLEAN_OPERATION", false),
        NodeKind::Component(_) => ("SYMBOL", false),
        NodeKind::ComponentSet(_) => ("FRAME", true),
        NodeKind::Instance { .. } => ("INSTANCE", false),
        NodeKind::Sticky(_) => ("STICKY", false),
        NodeKind::ShapeWithText(_) => ("SHAPE_WITH_TEXT", false),
        NodeKind::Connector(_) => ("CONNECTOR", false),
        NodeKind::Stamp { .. } => ("STAMP", false),
        NodeKind::Unknown { type_name } => (
            match type_name.as_str() {
                "SLICE" => "SLICE",
                _ => "FRAME",
            },
            false,
        ),
    };

    let mut fields = vec![
        ("guid", guid(nguid.0, nguid.1)),
        ("phase", en("CREATED")),
        ("parentIndex", st(vec![("guid", guid(parent_guid.0, parent_guid.1)), ("position", V::String(pos_str(sibling_idx)))])),
        ("type", en(ty)),
        ("name", V::String(node.name.clone())),
        ("visible", V::Bool(node.visible)),
        ("locked", V::Bool(node.locked)),
        ("opacity", fl(node.opacity)),
        ("size", vec2(node.size.x, node.size.y)),
        ("transform", encode_matrix(&node.transform)),
        ("blendMode", en(blend_name(node.blend))),
    ];
    if is_state_group {
        fields.push(("isStateGroup", V::Bool(true)));
    }
    if node.is_mask {
        fields.push(("mask", V::Bool(true)));
    }
    if let Some(src) = node.component.source.and_then(|s| cx.node_guids.get(&s).copied()) {
        fields.push(("overrideKey", guid(src.0, src.1)));
    } else if in_component {
        fields.push(("overrideKey", guid(nguid.0, nguid.1)));
    }

    // Fills & strokes (with bound colorVar when present).
    let fill_color_var = |i: usize| -> Option<V> {
        let vid = node.variables.bindings.get(&format!("fills/{i}"))?;
        let vg = cx.resolve_var_guid(vid)?;
        Some(alias_var_data(vg))
    };
    let stroke_color_var = |i: usize| -> Option<V> {
        let vid = node.variables.bindings.get(&format!("strokes/{i}"))?;
        let vg = cx.resolve_var_guid(vid)?;
        Some(alias_var_data(vg))
    };
    let fills_slice = match &node.kind {
        NodeKind::Text(t) => &t.style.fills,
        _ => &node.fills,
    };
    let fill_paints: Vec<V> = fills_slice.iter().enumerate().map(|(i, p)| encode_paint(p, fill_color_var(i))).collect();
    fields.push(("fillPaints", V::Array(fill_paints)));

    let stroke_paints: Vec<V> = node.stroke.paints.iter().enumerate().map(|(i, p)| encode_paint(p, stroke_color_var(i))).collect();
    fields.push(("strokePaints", V::Array(stroke_paints)));
    fields.push(("strokeWeight", fl(node.stroke.weight)));
    fields.push((
        "strokeAlign",
        en(match node.stroke.align {
            StrokeAlign::Inside => "INSIDE",
            StrokeAlign::Center => "CENTER",
            StrokeAlign::Outside => "OUTSIDE",
        }),
    ));
    fields.push((
        "strokeJoin",
        en(match node.stroke.join {
            StrokeJoin::Miter => "MITER",
            StrokeJoin::Bevel => "BEVEL",
            StrokeJoin::Round => "ROUND",
        }),
    ));
    fields.push((
        "strokeCap",
        en(match node.stroke.cap {
            StrokeCap::None => "NONE",
            StrokeCap::Round => "ROUND",
            StrokeCap::Square => "SQUARE",
        }),
    ));
    if !node.stroke.dashes.is_empty() {
        fields.push(("dashPattern", V::Array(node.stroke.dashes.iter().map(|&d| fl(d)).collect())));
    }
    if node.stroke.miter_limit >= 1.0 {
        fields.push(("miterLimit", fl(node.stroke.miter_limit)));
    }
    if let Some([top, right, bottom, left]) = node.stroke.side_weights {
        fields.push(("borderStrokeWeightsIndependent", V::Bool(true)));
        fields.push(("borderTopWeight", fl(top)));
        fields.push(("borderRightWeight", fl(right)));
        fields.push(("borderBottomWeight", fl(bottom)));
        fields.push(("borderLeftWeight", fl(left)));
    }

    // Effects.
    let effect_color_var = |i: usize| -> Option<V> {
        let vid = node.variables.bindings.get(&format!("effects/{i}/color"))?;
        let vg = cx.resolve_var_guid(vid)?;
        Some(alias_var_data(vg))
    };
    let effects: Vec<V> = node.effects.iter().enumerate().filter_map(|(i, e)| encode_effect(e, effect_color_var(i))).collect();
    if !effects.is_empty() {
        fields.push(("effects", V::Array(effects)));
    }

    // Corner radii & smoothing.
    let r = node.corner_radii;
    if r.iter().all(|&x| (x - r[0]).abs() < 1e-5) {
        if r[0] != 0.0 {
            fields.push(("cornerRadius", fl(r[0])));
        }
    } else {
        fields.push(("cornerRadius", fl(r[0])));
        fields.push(("rectangleCornerRadiiIndependent", V::Bool(true)));
        fields.push(("rectangleTopLeftCornerRadius", fl(r[0])));
        fields.push(("rectangleTopRightCornerRadius", fl(r[1])));
        fields.push(("rectangleBottomRightCornerRadius", fl(r[2])));
        fields.push(("rectangleBottomLeftCornerRadius", fl(r[3])));
    }
    if node.corner_smoothing != 0.0 {
        fields.push(("cornerSmoothing", fl(node.corner_smoothing)));
    }

    // Constraints.
    let cname = |c: Constraint| match c {
        Constraint::Min => "MIN",
        Constraint::Center => "CENTER",
        Constraint::Max => "MAX",
        Constraint::Stretch => "STRETCH",
        Constraint::Scale => "SCALE",
    };
    fields.push(("horizontalConstraint", en(cname(node.constraints.horizontal))));
    fields.push(("verticalConstraint", en(cname(node.constraints.vertical))));

    // Frame & Auto-Layout properties.
    let my_dir = node.kind.frame_props().and_then(|f| f.auto_layout).map(|a| a.direction);
    if let Some(fp) = node.kind.frame_props() {
        fields.push(("frameMaskDisabled", V::Bool(!fp.clips_content)));
        if !fp.grids.is_empty() {
            fields.push(("layoutGrids", V::Array(fp.grids.iter().map(encode_grid).collect())));
        }
        if let Some(al) = fp.auto_layout {
            fields.push(("stackMode", en(if al.direction == LayoutDirection::Horizontal { "HORIZONTAL" } else { "VERTICAL" })));
            fields.push(("stackSpacing", fl(al.item_spacing)));
            fields.push(("stackCounterSpacing", fl(al.counter_spacing)));
            fields.push(("stackPaddingTop", fl(al.padding[0])));
            fields.push(("stackPaddingRight", fl(al.padding[1])));
            fields.push(("stackPaddingBottom", fl(al.padding[2])));
            fields.push(("stackPaddingLeft", fl(al.padding[3])));
            fields.push(("stackHorizontalPadding", fl(al.padding[3])));
            fields.push(("stackVerticalPadding", fl(al.padding[0])));
            let align_name = |a: LayoutAlign| match a {
                LayoutAlign::Start => "MIN",
                LayoutAlign::Center => "CENTER",
                LayoutAlign::End => "MAX",
                LayoutAlign::SpaceBetween => "SPACE_BETWEEN",
                LayoutAlign::Baseline => "BASELINE",
            };
            fields.push(("stackPrimaryAlignItems", en(align_name(al.primary_align))));
            fields.push(("stackCounterAlignItems", en(align_name(al.counter_align))));
            fields.push(("stackPrimarySizing", en(if al.primary_sizing == Sizing::Hug { "RESIZE_TO_FIT" } else { "FIXED" })));
            fields.push(("stackCounterSizing", en(if al.counter_sizing == Sizing::Hug { "RESIZE_TO_FIT_WITH_IMPLICIT_SIZE" } else { "FIXED" })));
            fields.push(("stackWrap", en(if al.wrap { "WRAP" } else { "NO_WRAP" })));
            fields.push(("bordersTakeSpace", V::Bool(al.stroke_included)));
            fields.push(("stackReverseZIndex", V::Bool(al.reverse_z)));
        }
    }

    // Child layout properties.
    let lc = &node.layout_child;
    let (prim_fill, counter_fill) = match parent_dir {
        Some(LayoutDirection::Horizontal) => (lc.horizontal == Sizing::Fill || lc.grow > 0.0, lc.vertical == Sizing::Fill),
        Some(LayoutDirection::Vertical) => (lc.vertical == Sizing::Fill || lc.grow > 0.0, lc.horizontal == Sizing::Fill),
        None => (lc.grow > 0.0, false),
    };
    if prim_fill {
        fields.push(("stackChildPrimaryGrow", fl(if lc.grow > 0.0 { lc.grow } else { 1.0 })));
    }
    if counter_fill {
        fields.push(("stackChildAlignSelf", en("STRETCH")));
    }
    if lc.absolute {
        fields.push(("stackPositioning", en("ABSOLUTE")));
    }
    if lc.min_w.is_some() || lc.min_h.is_some() {
        let mut s = Vec::new();
        if let Some(x) = lc.min_w {
            s.push(("x", fl(x)));
        }
        if let Some(y) = lc.min_h {
            s.push(("y", fl(y)));
        }
        fields.push(("minSize", msg(s)));
    }
    if lc.max_w.is_some() || lc.max_h.is_some() {
        let mut s = Vec::new();
        if let Some(x) = lc.max_w {
            s.push(("x", fl(x)));
        }
        if let Some(y) = lc.max_h {
            s.push(("y", fl(y)));
        }
        fields.push(("maxSize", msg(s)));
    }

    // Export settings & frame guides.
    if !node.export_settings.is_empty() {
        let exs: Vec<V> = node
            .export_settings
            .iter()
            .map(|e| {
                let img_ty = match e.format {
                    ExportFileFormat::Png => "PNG",
                    ExportFileFormat::Jpg => "JPEG",
                    ExportFileFormat::Svg => "SVG",
                    ExportFileFormat::Pdf => "PDF",
                };
                let (cty, cval) = match e.constraint {
                    ExportConstraint::Scale(v) => ("CONTENT_SCALE", v),
                    ExportConstraint::Width(v) => ("CONTENT_WIDTH", v),
                    ExportConstraint::Height(v) => ("CONTENT_HEIGHT", v),
                };
                msg(vec![
                    ("suffix", V::String(e.suffix.clone())),
                    ("imageType", en(img_ty)),
                    ("constraint", msg(vec![("type", en(cty)), ("value", fl(cval))])),
                ])
            })
            .collect();
        fields.push(("exportSettings", V::Array(exs)));
    }
    if let Some(page) = cx.doc.page_of(id) {
        if let Some(guides) = cx.doc.guides.get(&page) {
            let fg: Vec<V> = guides
                .iter()
                .filter(|g| g.frame == Some(id))
                .map(|g| msg(vec![("axis", en(if g.axis == GuideAxis::X { "X" } else { "Y" })), ("offset", fl(g.offset))]))
                .collect();
            if !fg.is_empty() {
                fields.push(("guides", V::Array(fg)));
            }
        }
    }

    // Style references.
    for (k, field_name) in [
        ("fill", "styleIdForFill"),
        ("stroke", "styleIdForStrokeFill"),
        ("text", "styleIdForText"),
        ("effect", "styleIdForEffect"),
        ("grid", "styleIdForGrid"),
    ] {
        if let Some(sid) = node.style_refs.get(k) {
            if let Some(&sg) = cx.style_guids.get(sid) {
                fields.push((field_name, msg(vec![("guid", guid(sg.0, sg.1))])));
            } else if let Some(key) = sid.strip_prefix("key:") {
                fields.push((field_name, msg(vec![("assetRef", msg(vec![("key", V::String(key.to_string()))]))])));
            }
        }
    }

    // Node variable bindings & explicit modes.
    let mut var_entries = Vec::new();
    for (k, vid) in &node.variables.bindings {
        if k.starts_with("fills/") || k.starts_with("strokes/") || k.starts_with("effects/") {
            continue;
        }
        let Some(vg) = cx.resolve_var_guid(vid) else { continue };
        let vfield = match k.as_str() {
            "paddingLeft" => "STACK_PADDING_LEFT",
            "paddingRight" => "STACK_PADDING_RIGHT",
            "paddingTop" => "STACK_PADDING_TOP",
            "paddingBottom" => "STACK_PADDING_BOTTOM",
            "itemSpacing" => "STACK_SPACING",
            "counterAxisSpacing" => "STACK_COUNTER_SPACING",
            "topLeftRadius" => "RECTANGLE_TOP_LEFT_CORNER_RADIUS",
            "topRightRadius" => "RECTANGLE_TOP_RIGHT_CORNER_RADIUS",
            "bottomLeftRadius" => "RECTANGLE_BOTTOM_LEFT_CORNER_RADIUS",
            "bottomRightRadius" => "RECTANGLE_BOTTOM_RIGHT_CORNER_RADIUS",
            "width" | "size/x" => "WIDTH",
            "height" | "size/y" => "HEIGHT",
            "minWidth" => "MIN_WIDTH",
            "maxWidth" => "MAX_WIDTH",
            "minHeight" => "MIN_HEIGHT",
            "maxHeight" => "MAX_HEIGHT",
            "opacity" => "OPACITY",
            "strokeWeight" => "STROKE_WEIGHT",
            "fontSize" => "FONT_SIZE",
            "fontFamily" => "FONT_FAMILY",
            "fontWeight" => "FONT_WEIGHT",
            "lineHeight" => "LINE_HEIGHT",
            "letterSpacing" => "LETTER_SPACING",
            "paragraphSpacing" => "PARAGRAPH_SPACING",
            "paragraphIndent" => "PARAGRAPH_INDENT",
            _ => continue,
        };
        var_entries.push(msg(vec![("variableField", V::String(vfield.into())), ("variableData", alias_var_data(vg))]));
    }
    if !var_entries.is_empty() {
        fields.push(("variableConsumptionMap", msg(vec![("entries", V::Array(var_entries))])));
    }
    if !node.variables.modes.is_empty() {
        let mut mode_entries = Vec::new();
        for (cid, mid) in &node.variables.modes {
            let Some(cg) = cx.resolve_coll_guid(cid) else { continue };
            let Some(&mg) = cx.mode_guids.get(mid) else { continue };
            mode_entries.push(msg(vec![
                ("variableSetID", msg(vec![("guid", guid(cg.0, cg.1))])),
                ("variableModeID", guid(mg.0, mg.1)),
            ]));
        }
        if !mode_entries.is_empty() {
            fields.push(("variableModeBySetMap", msg(vec![("entries", V::Array(mode_entries))])));
        }
    }

    // Geometry blobs.
    if !matches!(node.kind, NodeKind::Text(_)) {
        let fill_paths = node.fill_geometry.as_ref().or_else(|| match &node.kind {
            NodeKind::Vector { paths, .. } if !paths.is_empty() => Some(paths),
            _ => None,
        });
        if let Some(paths) = fill_paths {
            let fgs = encode_geometry_blobs(cx, paths);
            if !fgs.is_empty() {
                fields.push(("fillGeometry", V::Array(fgs)));
            }
        }
        if let Some(paths) = &node.stroke_geometry {
            let sgs = encode_geometry_blobs(cx, paths);
            if !sgs.is_empty() {
                fields.push(("strokeGeometry", V::Array(sgs)));
            }
        }
    }

    // Kind-specific fields.
    match &node.kind {
        NodeKind::Ellipse(arc) => {
            fields.push((
                "arcData",
                st(vec![("startingAngle", fl(arc.start)), ("endingAngle", fl(arc.end)), ("innerRadius", fl(arc.inner_radius))]),
            ));
        }
        NodeKind::Polygon { count } => {
            fields.push(("count", V::Uint(*count)));
        }
        NodeKind::Star { count, inner_ratio } => {
            fields.push(("count", V::Uint(*count)));
            fields.push(("starInnerScale", fl(*inner_ratio)));
        }
        NodeKind::BooleanOp(op) => {
            let op_name = match op {
                BooleanOp::Union => "UNION",
                BooleanOp::Subtract => "SUBTRACT",
                BooleanOp::Intersect => "INTERSECT",
                BooleanOp::Exclude => "XOR",
            };
            fields.push(("booleanOperation", en(op_name)));
        }
        NodeKind::Instance { component, component_key, .. } => {
            if let Some(comp_id) = component.and_then(|c| cx.node_guids.get(&c).copied()) {
                fields.push(("symbolData", msg(vec![("symbolID", guid(comp_id.0, comp_id.1))])));
            }
            if let Some(k) = component_key {
                fields.push(("componentKey", V::String(k.clone())));
            }
        }
        NodeKind::Vector { network: Some(net), .. } if !net.vertices.is_empty() => {
            let (blob_bytes, style_table) = encode_vector_network_blob(net);
            let blob_idx = cx.add_blob(blob_bytes);
            let mut vd = vec![("vectorNetworkBlob", V::Uint(blob_idx)), ("normalizedSize", vec2(node.size.x, node.size.y))];
            if !style_table.is_empty() {
                vd.push(("styleOverrideTable", V::Array(style_table)));
            }
            fields.push(("vectorData", msg(vd)));
        }
        NodeKind::Text(tp) => {
            encode_text_node(&mut fields, tp);
        }
        _ => {}
    }

    cx.changes.push(msg(fields));

    let child_in_comp = in_component || matches!(node.kind, NodeKind::Component(_) | NodeKind::ComponentSet(_));
    for (cidx, cid) in node.children.into_iter().enumerate() {
        export_node_rec(cx, cid, nguid, cidx, my_dir, child_in_comp);
    }
}

fn alias_var_data(vg: (u32, u32)) -> V {
    msg(vec![("dataType", en("ALIAS")), ("value", msg(vec![("alias", msg(vec![("guid", guid(vg.0, vg.1))]))]))])
}

fn encode_geometry_blobs(cx: &mut ExportCtx, paths: &[Path]) -> Vec<V> {
    paths
        .iter()
        .map(|p| {
            let blob_idx = cx.add_blob(path_to_commands_blob(p));
            let rule = if p.fill_rule == FillRule::EvenOdd { "ODD" } else { "NONZERO" };
            msg(vec![("windingRule", en(rule)), ("commandsBlob", V::Uint(blob_idx))])
        })
        .collect()
}

fn encode_vector_network_blob(net: &VectorNetwork) -> (Vec<u8>, Vec<V>) {
    let mut b = Vec::new();
    let mut style_table = Vec::new();
    for n in [net.vertices.len(), net.segments.len(), net.regions.len()] {
        b.extend_from_slice(&(n as u32).to_le_bytes());
    }
    for (i, v) in net.vertices.iter().enumerate() {
        let has_style = v.cap.is_some() || v.join.is_some() || v.corner_radius > 0.0 || v.mirror != HandleMirror::None;
        let sid = if has_style {
            let id = (style_table.len() + 1) as u32;
            let mut so = vec![("styleID", V::Uint(id))];
            if let Some(c) = v.cap {
                so.push((
                    "strokeCap",
                    en(match c {
                        StrokeCap::None => "NONE",
                        StrokeCap::Round => "ROUND",
                        StrokeCap::Square => "SQUARE",
                    }),
                ));
            }
            if let Some(j) = v.join {
                so.push((
                    "strokeJoin",
                    en(match j {
                        StrokeJoin::Miter => "MITER",
                        StrokeJoin::Bevel => "BEVEL",
                        StrokeJoin::Round => "ROUND",
                    }),
                ));
            }
            if v.corner_radius > 0.0 {
                so.push(("cornerRadius", fl(v.corner_radius)));
            }
            if v.mirror != HandleMirror::None {
                let m = match v.mirror {
                    HandleMirror::None => "NONE",
                    HandleMirror::Angle => "ANGLE",
                    HandleMirror::AngleAndLength => "ANGLE_AND_LENGTH",
                };
                so.push(("handleMirroring", V::String(m.into())));
            }
            style_table.push(msg(so));
            id
        } else {
            let _ = i;
            0
        };
        b.extend_from_slice(&sid.to_le_bytes());
        b.extend_from_slice(&v.pos.x.to_le_bytes());
        b.extend_from_slice(&v.pos.y.to_le_bytes());
    }
    for seg in &net.segments {
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&(seg.a as u32).to_le_bytes());
        b.extend_from_slice(&seg.ta.x.to_le_bytes());
        b.extend_from_slice(&seg.ta.y.to_le_bytes());
        b.extend_from_slice(&(seg.b as u32).to_le_bytes());
        b.extend_from_slice(&seg.tb.x.to_le_bytes());
        b.extend_from_slice(&seg.tb.y.to_le_bytes());
    }
    for reg in &net.regions {
        let rule = if reg.rule == FillRule::NonZero { 1u32 } else { 0u32 };
        b.extend_from_slice(&rule.to_le_bytes());
        b.extend_from_slice(&(reg.loops.len() as u32).to_le_bytes());
        for lp in &reg.loops {
            b.extend_from_slice(&(lp.len() as u32).to_le_bytes());
            for &seg_idx in lp {
                b.extend_from_slice(&(seg_idx as u32).to_le_bytes());
            }
        }
    }
    (b, style_table)
}

fn encode_matrix(xf: &Affine) -> V {
    st(vec![("m00", fl(xf.a)), ("m01", fl(xf.c)), ("m02", fl(xf.tx)), ("m10", fl(xf.b)), ("m11", fl(xf.d)), ("m12", fl(xf.ty))])
}

fn gradient_matrix_from_handles(h: &[Vec2; 3]) -> V {
    // Inverse of `fig.rs` `convert_paint`: `inv` maps `(0, 0.5) -> h[0]`, `(1, 0.5) -> h[1]`, `(0, 1.0) -> h[2]`.
    let inv = Affine {
        a: h[1].x - h[0].x,
        b: h[1].y - h[0].y,
        c: 2.0 * (h[2].x - h[0].x),
        d: 2.0 * (h[2].y - h[0].y),
        tx: 2.0 * h[0].x - h[2].x,
        ty: 2.0 * h[0].y - h[2].y,
    };
    let xf = inv.invert().unwrap_or(Affine::IDENTITY);
    encode_matrix(&xf)
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if s.is_empty() || s.len() % 2 != 0 || !s.bytes().all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

fn encode_paint(p: &Paint, color_var: Option<V>) -> V {
    let mut fields = Vec::new();
    let push_grad = |fields: &mut Vec<(&str, V)>, ty: &str, handles: &[Vec2; 3], stops: &[GradientStop]| {
        fields.push(("type", en(ty)));
        let st_list: Vec<V> =
            stops.iter().map(|s| st(vec![("color", color(s.color.r, s.color.g, s.color.b, s.color.a)), ("position", fl(s.pos))])).collect();
        fields.push(("stops", V::Array(st_list)));
        fields.push(("transform", gradient_matrix_from_handles(handles)));
    };
    match &p.kind {
        PaintKind::Solid(c) => {
            fields.push(("type", en("SOLID")));
            fields.push(("color", color(c.r, c.g, c.b, c.a)));
        }
        PaintKind::LinearGradient { handles, stops } => push_grad(&mut fields, "GRADIENT_LINEAR", handles, stops),
        PaintKind::RadialGradient { handles, stops } => push_grad(&mut fields, "GRADIENT_RADIAL", handles, stops),
        PaintKind::AngularGradient { handles, stops } => push_grad(&mut fields, "GRADIENT_ANGULAR", handles, stops),
        PaintKind::DiamondGradient { handles, stops } => push_grad(&mut fields, "GRADIENT_DIAMOND", handles, stops),
        PaintKind::Image { image_ref, scale_mode } => {
            fields.push(("type", en("IMAGE")));
            if let Some(raw) = unhex(image_ref) {
                fields.push(("image", msg(vec![("hash", V::Bytes(raw))])));
            } else {
                fields.push(("imageHash", V::String(image_ref.clone())));
            }
            let sm = match scale_mode {
                ImageScaleMode::Fill => "FILL",
                ImageScaleMode::Fit => "FIT",
                ImageScaleMode::Crop => "STRETCH",
                ImageScaleMode::Tile => "TILE",
            };
            fields.push(("imageScaleMode", en(sm)));
        }
    }
    fields.push(("opacity", fl(p.opacity)));
    fields.push(("visible", V::Bool(p.visible)));
    fields.push(("blendMode", en(blend_name(p.blend))));
    if let Some(cv) = color_var {
        fields.push(("colorVar", cv));
    }
    msg(fields)
}

fn encode_effect(e: &Effect, color_var: Option<V>) -> Option<V> {
    let mut fields = match e {
        Effect::DropShadow { color: c, offset, radius, spread, visible, show_behind } => vec![
            ("type", en("DROP_SHADOW")),
            ("color", color(c.r, c.g, c.b, c.a)),
            ("offset", vec2(offset.x, offset.y)),
            ("radius", fl(*radius)),
            ("spread", fl(*spread)),
            ("visible", V::Bool(*visible)),
            ("showShadowBehindNode", V::Bool(*show_behind)),
        ],
        Effect::InnerShadow { color: c, offset, radius, spread, visible } => vec![
            ("type", en("INNER_SHADOW")),
            ("color", color(c.r, c.g, c.b, c.a)),
            ("offset", vec2(offset.x, offset.y)),
            ("radius", fl(*radius)),
            ("spread", fl(*spread)),
            ("visible", V::Bool(*visible)),
        ],
        Effect::LayerBlur { radius, visible } => vec![("type", en("FOREGROUND_BLUR")), ("radius", fl(*radius)), ("visible", V::Bool(*visible))],
        Effect::BackgroundBlur { radius, visible } => vec![("type", en("BACKGROUND_BLUR")), ("radius", fl(*radius)), ("visible", V::Bool(*visible))],
        _ => return None,
    };
    if let Some(cv) = color_var {
        fields.push(("colorVar", cv));
    }
    Some(msg(fields))
}

fn encode_grid(g: &LayoutGrid) -> V {
    match g {
        LayoutGrid::Grid { size, color: c } => msg(vec![
            ("pattern", en("GRID")),
            ("sectionSize", fl(*size)),
            ("color", color(c.r, c.g, c.b, c.a)),
            ("visible", V::Bool(true)),
        ]),
        LayoutGrid::Columns { count, gutter, offset, color: c } => msg(vec![
            ("pattern", en("COLUMNS")),
            ("count", V::Uint(*count)),
            ("gutterSize", fl(*gutter)),
            ("offset", fl(*offset)),
            ("color", color(c.r, c.g, c.b, c.a)),
            ("visible", V::Bool(true)),
        ]),
        LayoutGrid::Rows { count, gutter, offset, color: c } => msg(vec![
            ("pattern", en("ROWS")),
            ("count", V::Uint(*count)),
            ("gutterSize", fl(*gutter)),
            ("offset", fl(*offset)),
            ("color", color(c.r, c.g, c.b, c.a)),
            ("visible", V::Bool(true)),
        ]),
    }
}

fn style_name_from_weight(weight: u16, italic: bool) -> String {
    let w = match weight {
        0..=149 => "Thin",
        150..=249 => "ExtraLight",
        250..=349 => "Light",
        350..=449 => "Regular",
        450..=549 => "Medium",
        550..=649 => "SemiBold",
        650..=749 => "Bold",
        750..=849 => "ExtraBold",
        _ => "Black",
    };
    if italic {
        if w == "Regular" { "Italic".into() } else { format!("{w} Italic") }
    } else {
        w.into()
    }
}

fn append_text_style_fields(fields: &mut Vec<(&'static str, V)>, ts: &TextStyle) {
    let style_str = style_name_from_weight(ts.font_weight, ts.italic);
    fields.push((
        "fontName",
        st(vec![("family", V::String(ts.font_family.clone())), ("style", V::String(style_str)), ("postscript", V::String(String::new()))]),
    ));
    fields.push(("fontSize", fl(ts.font_size)));
    fields.push(("letterSpacing", st(vec![("value", fl(ts.letter_spacing)), ("units", en("PIXELS"))])));
    let lh = match ts.line_height {
        LineHeight::Auto => st(vec![("value", fl(1.0)), ("units", en("RAW"))]),
        LineHeight::Pixels(px) => st(vec![("value", fl(px)), ("units", en("PIXELS"))]),
        LineHeight::Percent(p) => st(vec![("value", fl(p)), ("units", en("PERCENT"))]),
    };
    fields.push(("lineHeight", lh));
    fields.push((
        "textAlignHorizontal",
        en(match ts.align_h {
            TextAlignH::Left => "LEFT",
            TextAlignH::Center => "CENTER",
            TextAlignH::Right => "RIGHT",
            TextAlignH::Justified => "JUSTIFIED",
        }),
    ));
    fields.push((
        "textAlignVertical",
        en(match ts.align_v {
            TextAlignV::Top => "TOP",
            TextAlignV::Center => "CENTER",
            TextAlignV::Bottom => "BOTTOM",
        }),
    ));
    fields.push((
        "textCase",
        en(match ts.case {
            TextCase::Original => "ORIGINAL",
            TextCase::Upper => "UPPER",
            TextCase::Lower => "LOWER",
            TextCase::Title => "TITLE",
            TextCase::SmallCaps => "SMALL_CAPS",
        }),
    ));
    fields.push((
        "textDecoration",
        en(match ts.decoration {
            TextDecoration::None => "NONE",
            TextDecoration::Underline => "UNDERLINE",
            TextDecoration::Strikethrough => "STRIKETHROUGH",
        }),
    ));
    if ts.paragraph_spacing != 0.0 {
        fields.push(("paragraphSpacing", fl(ts.paragraph_spacing)));
    }
}

fn encode_text_node(fields: &mut Vec<(&'static str, V)>, tp: &TextProps) {
    append_text_style_fields(fields, &tp.style);
    fields.push((
        "textAutoResize",
        en(match tp.auto_resize {
            TextAutoResize::None => "NONE",
            TextAutoResize::WidthAndHeight => "WIDTH_AND_HEIGHT",
            TextAutoResize::Height => "HEIGHT",
            TextAutoResize::Truncate => "TRUNCATE",
        }),
    ));
    if tp.auto_resize == TextAutoResize::Truncate {
        fields.push(("textTruncation", en("ENDING")));
    }
    if let Some(m) = tp.max_lines {
        fields.push(("maxLines", V::Uint(m)));
    }
    if tp.paragraph.indent != 0.0 {
        fields.push(("paragraphIndent", fl(tp.paragraph.indent)));
    }
    if tp.paragraph.list_spacing != 0.0 {
        fields.push(("listSpacing", fl(tp.paragraph.list_spacing)));
    }
    if tp.paragraph.hanging_punctuation {
        fields.push(("hangingPunctuation", V::Bool(true)));
    }
    if tp.paragraph.hanging_list {
        fields.push(("hangingList", V::Bool(true)));
    }
    if tp.paragraph.leading_trim == LeadingTrim::CapHeight {
        fields.push(("leadingTrim", en("CAP_HEIGHT")));
    }

    let mut td = vec![("characters", V::String(tp.content.clone()))];
    if !tp.runs.is_empty() {
        let offs = utf16_to_byte_offsets(&tp.content);
        let mut char_ids = vec![0u32; offs.len().saturating_sub(1)];
        let mut table = Vec::new();
        for (i, r) in tp.runs.iter().enumerate() {
            let sid = (i + 1) as u32;
            let s = offs.partition_point(|&o| o < r.start);
            let e = offs.partition_point(|&o| o < r.end);
            let cap = e.min(char_ids.len());
            for slot in char_ids.iter_mut().take(cap).skip(s) {
                *slot = sid;
            }
            let st_name = style_name_from_weight(r.style.font_weight, r.style.italic);
            let lh = match r.style.line_height {
                LineHeight::Auto => st(vec![("value", fl(1.0)), ("units", en("RAW"))]),
                LineHeight::Pixels(px) => st(vec![("value", fl(px)), ("units", en("PIXELS"))]),
                LineHeight::Percent(p) => st(vec![("value", fl(p)), ("units", en("PERCENT"))]),
            };
            let so = msg(vec![
                ("styleID", V::Uint(sid)),
                ("fontSize", fl(r.style.font_size)),
                (
                    "fontName",
                    st(vec![
                        ("family", V::String(r.style.font_family.clone())),
                        ("style", V::String(st_name)),
                        ("postscript", V::String(String::new())),
                    ]),
                ),
                ("fillPaints", V::Array(r.style.fills.iter().map(|p| encode_paint(p, None)).collect())),
                (
                    "textDecoration",
                    en(match r.style.decoration {
                        TextDecoration::None => "NONE",
                        TextDecoration::Underline => "UNDERLINE",
                        TextDecoration::Strikethrough => "STRIKETHROUGH",
                    }),
                ),
                (
                    "textCase",
                    en(match r.style.case {
                        TextCase::Original => "ORIGINAL",
                        TextCase::Upper => "UPPER",
                        TextCase::Lower => "LOWER",
                        TextCase::Title => "TITLE",
                        TextCase::SmallCaps => "SMALL_CAPS",
                    }),
                ),
                ("letterSpacing", st(vec![("value", fl(r.style.letter_spacing)), ("units", en("PIXELS"))])),
                ("lineHeight", lh),
            ]);
            table.push(so);
        }
        td.push(("characterStyleIDs", V::Array(char_ids.into_iter().map(V::Uint).collect())));
        td.push(("styleOverrideTable", V::Array(table)));
    }
    if tp.has_lists() || tp.direction != TextDirection::Auto {
        let dir = match tp.direction {
            TextDirection::Auto => "AUTO",
            TextDirection::Ltr => "LTR",
            TextDirection::Rtl => "RTL",
        };
        let lists = tp.paragraph_lists();
        let lines: Vec<V> = lists
            .iter()
            .map(|item| {
                let (lty, indent) = match item {
                    None => ("PLAIN", 0u32),
                    Some(ListItem { kind: ListKind::Numbered, level }) => ("ORDERED_LIST", *level as u32 + 1),
                    Some(ListItem { kind: ListKind::Bullet, level }) => ("UNORDERED_LIST", *level as u32 + 1),
                };
                msg(vec![("lineType", en(lty)), ("indentationLevel", V::Uint(indent)), ("sourceDirectionality", en(dir))])
            })
            .collect();
        if !lines.is_empty() {
            td.push(("lines", V::Array(lines)));
        }
    }
    fields.push(("textData", msg(td)));
}

fn blend_name(b: BlendMode) -> &'static str {
    match b {
        BlendMode::Normal => "NORMAL",
        BlendMode::PassThrough => "PASS_THROUGH",
        BlendMode::Darken => "DARKEN",
        BlendMode::Multiply => "MULTIPLY",
        BlendMode::ColorBurn => "COLOR_BURN",
        BlendMode::LinearBurn => "LINEAR_BURN",
        BlendMode::Lighten => "LIGHTEN",
        BlendMode::Screen => "SCREEN",
        BlendMode::ColorDodge => "COLOR_DODGE",
        BlendMode::LinearDodge => "LINEAR_DODGE",
        BlendMode::Overlay => "OVERLAY",
        BlendMode::SoftLight => "SOFT_LIGHT",
        BlendMode::HardLight => "HARD_LIGHT",
        BlendMode::Difference => "DIFFERENCE",
        BlendMode::Exclusion => "EXCLUSION",
        BlendMode::Hue => "HUE",
        BlendMode::Saturation => "SATURATION",
        BlendMode::Color => "COLOR",
        BlendMode::Luminosity => "LUMINOSITY",
    }
}
