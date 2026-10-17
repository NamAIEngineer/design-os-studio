use crate::Vec2;
use serde::{Deserialize, Serialize};

/// Straight (non-premultiplied) sRGB color, components 0..=1.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub const BLACK: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Self = Self { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const TRANSPARENT: Self = Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };

    pub fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::rgba(r, g, b, 255)
    }
    pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r: r as f32 / 255.0, g: g as f32 / 255.0, b: b as f32 / 255.0, a: a as f32 / 255.0 }
    }
    /// Parses 1..8 digit hex (`#F`, `#F0`, `#123`, `#1234`, `#RRGGBB`, `#RRGGBBAA`),
    /// `"transparent"`, or CSS named colors (Figma module `640649`).
    pub fn from_hex(s: &str) -> Option<Self> {
        let s = s.trim();
        if s.is_empty() {
            return None;
        }
        if !s.starts_with('#') {
            let lower = s.to_ascii_lowercase();
            let named = match lower.as_str() {
                "transparent" => Some(Self::TRANSPARENT),
                "black" => Some(Self::rgb(0, 0, 0)),
                "white" => Some(Self::rgb(255, 255, 255)),
                "red" => Some(Self::rgb(255, 0, 0)),
                "green" => Some(Self::rgb(0, 128, 0)),
                "blue" => Some(Self::rgb(0, 0, 255)),
                "yellow" => Some(Self::rgb(255, 255, 0)),
                "cyan" | "aqua" => Some(Self::rgb(0, 255, 255)),
                "magenta" | "fuchsia" => Some(Self::rgb(255, 0, 255)),
                "gray" | "grey" => Some(Self::rgb(128, 128, 128)),
                "lightgray" | "lightgrey" => Some(Self::rgb(211, 211, 211)),
                "darkgray" | "darkgrey" => Some(Self::rgb(169, 169, 169)),
                "dimgray" | "dimgrey" => Some(Self::rgb(105, 105, 105)),
                "slategray" | "slategrey" => Some(Self::rgb(112, 128, 144)),
                "silver" => Some(Self::rgb(192, 192, 192)),
                "maroon" => Some(Self::rgb(128, 0, 0)),
                "olive" => Some(Self::rgb(128, 128, 0)),
                "lime" => Some(Self::rgb(0, 255, 0)),
                "teal" => Some(Self::rgb(0, 128, 128)),
                "navy" => Some(Self::rgb(0, 0, 128)),
                "purple" => Some(Self::rgb(128, 0, 128)),
                "rebeccapurple" => Some(Self::rgb(102, 51, 153)),
                "orange" => Some(Self::rgb(255, 165, 0)),
                "orangered" => Some(Self::rgb(255, 69, 0)),
                "coral" => Some(Self::rgb(255, 127, 80)),
                "tomato" => Some(Self::rgb(255, 99, 71)),
                "salmon" => Some(Self::rgb(250, 128, 114)),
                "gold" => Some(Self::rgb(255, 215, 0)),
                "pink" => Some(Self::rgb(255, 192, 203)),
                "hotpink" => Some(Self::rgb(255, 105, 180)),
                "deeppink" => Some(Self::rgb(255, 20, 147)),
                "violet" => Some(Self::rgb(238, 130, 238)),
                "plum" => Some(Self::rgb(221, 160, 221)),
                "orchid" => Some(Self::rgb(218, 112, 214)),
                "indigo" => Some(Self::rgb(75, 0, 130)),
                "blueviolet" => Some(Self::rgb(138, 43, 226)),
                "dodgerblue" => Some(Self::rgb(30, 144, 255)),
                "deepskyblue" => Some(Self::rgb(0, 191, 255)),
                "skyblue" => Some(Self::rgb(135, 206, 235)),
                "lightblue" => Some(Self::rgb(173, 216, 230)),
                "steelblue" => Some(Self::rgb(70, 130, 180)),
                "royalblue" => Some(Self::rgb(65, 105, 225)),
                "midnightblue" => Some(Self::rgb(25, 25, 112)),
                "turquoise" => Some(Self::rgb(64, 224, 208)),
                "aquamarine" => Some(Self::rgb(127, 255, 212)),
                "springgreen" => Some(Self::rgb(0, 255, 127)),
                "limegreen" => Some(Self::rgb(50, 205, 50)),
                "forestgreen" => Some(Self::rgb(34, 139, 34)),
                "seagreen" => Some(Self::rgb(46, 139, 87)),
                "yellowgreen" => Some(Self::rgb(154, 205, 50)),
                "chartreuse" => Some(Self::rgb(127, 255, 0)),
                "brown" => Some(Self::rgb(165, 42, 42)),
                "firebrick" => Some(Self::rgb(178, 34, 34)),
                "crimson" => Some(Self::rgb(220, 20, 60)),
                "darkred" => Some(Self::rgb(139, 0, 0)),
                "chocolate" => Some(Self::rgb(210, 105, 30)),
                "saddlebrown" => Some(Self::rgb(139, 69, 19)),
                "sienna" => Some(Self::rgb(160, 82, 45)),
                "peru" => Some(Self::rgb(205, 133, 63)),
                "tan" => Some(Self::rgb(210, 180, 140)),
                "wheat" => Some(Self::rgb(245, 222, 179)),
                "beige" => Some(Self::rgb(245, 245, 220)),
                "ivory" => Some(Self::rgb(255, 255, 240)),
                "linen" => Some(Self::rgb(250, 240, 230)),
                "lavender" => Some(Self::rgb(230, 230, 250)),
                "aliceblue" => Some(Self::rgb(240, 248, 255)),
                "ghostwhite" => Some(Self::rgb(248, 248, 255)),
                "mintcream" => Some(Self::rgb(245, 255, 250)),
                "honeydew" => Some(Self::rgb(240, 255, 240)),
                "seashell" => Some(Self::rgb(255, 245, 238)),
                "snow" => Some(Self::rgb(255, 250, 250)),
                "whitesmoke" => Some(Self::rgb(245, 245, 245)),
                "gainsboro" => Some(Self::rgb(220, 220, 220)),
                "khaki" => Some(Self::rgb(240, 230, 140)),
                _ => None,
            };
            if named.is_some() {
                return named;
            }
        }
        let s = s.trim_start_matches('#');
        let nib = |i: usize| u8::from_str_radix(s.get(i..i + 1)?, 16).ok();
        let byte = |i: usize| u8::from_str_radix(s.get(i..i + 2)?, 16).ok();
        match s.len() {
            1 => {
                let v = nib(0)? * 17;
                Some(Self::rgb(v, v, v))
            }
            2 => {
                let v = byte(0)?;
                Some(Self::rgb(v, v, v))
            }
            3 => Some(Self::rgb(nib(0)? * 17, nib(1)? * 17, nib(2)? * 17)),
            4 => Some(Self::rgba(nib(0)? * 17, nib(1)? * 17, nib(2)? * 17, nib(3)? * 17)),
            5 => Some(Self::rgba(nib(0)? * 17, nib(1)? * 17, nib(2)? * 17, byte(3)?)),
            6 => Some(Self::rgb(byte(0)?, byte(2)?, byte(4)?)),
            7 => Some(Self::rgba(byte(0)?, byte(2)?, byte(4)?, nib(6)? * 17)),
            8 => Some(Self::rgba(byte(0)?, byte(2)?, byte(4)?, byte(6)?)),
            _ => None,
        }
    }
    pub fn to_u8(&self) -> [u8; 4] {
        let c = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        [c(self.r), c(self.g), c(self.b), c(self.a)]
    }
    pub fn to_hex(&self) -> String {
        let [r, g, b, _] = self.to_u8();
        format!("{r:02X}{g:02X}{b:02X}")
    }
    pub fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    PassThrough,
    Darken,
    Multiply,
    ColorBurn,
    LinearBurn,
    Lighten,
    Screen,
    ColorDodge,
    LinearDodge,
    Overlay,
    SoftLight,
    HardLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct GradientStop {
    pub pos: f32,
    pub color: Color,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImageScaleMode {
    #[default]
    Fill,
    Fit,
    Crop,
    Tile,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaintKind {
    Solid(Color),
    /// Handles in node-normalized space (0..1 of the node's size), Figma convention:
    /// [start, end, width-direction].
    LinearGradient {
        handles: [Vec2; 3],
        stops: Vec<GradientStop>,
    },
    RadialGradient {
        handles: [Vec2; 3],
        stops: Vec<GradientStop>,
    },
    AngularGradient {
        handles: [Vec2; 3],
        stops: Vec<GradientStop>,
    },
    DiamondGradient {
        handles: [Vec2; 3],
        stops: Vec<GradientStop>,
    },
    /// `image_ref` keys into [`crate::Document::images`].
    Image {
        image_ref: String,
        scale_mode: ImageScaleMode,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Paint {
    pub kind: PaintKind,
    pub opacity: f32,
    pub visible: bool,
    pub blend: BlendMode,
}

impl Paint {
    pub fn solid(c: Color) -> Self {
        Self { kind: PaintKind::Solid(c), opacity: 1.0, visible: true, blend: BlendMode::Normal }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeAlign {
    #[default]
    Inside,
    Center,
    Outside,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeCap {
    #[default]
    None,
    Round,
    Square,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrokeJoin {
    #[default]
    Miter,
    Bevel,
    Round,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Stroke {
    pub paints: Vec<Paint>,
    pub weight: f32,
    /// Per-side weights (top, right, bottom, left) for rectangles/frames.
    pub side_weights: Option<[f32; 4]>,
    pub align: StrokeAlign,
    pub cap: StrokeCap,
    pub join: StrokeJoin,
    pub miter_limit: f32,
    pub dashes: Vec<f32>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Effect {
    DropShadow {
        color: Color,
        offset: Vec2,
        radius: f32,
        spread: f32,
        visible: bool,
        show_behind: bool,
    },
    InnerShadow {
        color: Color,
        offset: Vec2,
        radius: f32,
        spread: f32,
        visible: bool,
    },
    LayerBlur {
        radius: f32,
        visible: bool,
    },
    BackgroundBlur {
        radius: f32,
        visible: bool,
    },
    /// Grain over the layer's shape.
    Noise(NoiseEffect),
    /// Displaces the layer's edges with a rough, paper-like texture.
    Texture(TextureEffect),
    /// Frosted, refractive glass over what is behind the layer.
    Glass(GlassEffect),
    /// Blur whose radius ramps linearly between two points of the layer.
    ProgressiveBlur(ProgressiveBlurEffect),
}

impl Effect {
    pub fn visible(&self) -> bool {
        match self {
            Effect::DropShadow { visible, .. }
            | Effect::InnerShadow { visible, .. }
            | Effect::LayerBlur { visible, .. }
            | Effect::BackgroundBlur { visible, .. } => *visible,
            Effect::Noise(e) => e.visible,
            Effect::Texture(e) => e.visible,
            Effect::Glass(e) => e.visible,
            Effect::ProgressiveBlur(e) => e.visible,
        }
    }

    /// Radius of a visible blur of the layer itself (plain, or the largest
    /// radius of a progressive one); `None` when it blurs nothing.
    pub fn layer_blur_radius(&self) -> Option<f32> {
        match self {
            Effect::LayerBlur { radius, visible: true } if *radius > 0.0 => Some(*radius),
            Effect::ProgressiveBlur(p) if p.visible && !p.background && p.max_radius() > 0.0 => Some(p.max_radius()),
            _ => None,
        }
    }

    /// Radius of a visible blur of what lies behind the layer (plain, or the
    /// largest radius of a progressive one); `None` when it blurs nothing.
    pub fn background_blur_radius(&self) -> Option<f32> {
        match self {
            Effect::BackgroundBlur { radius, visible: true } if *radius > 0.0 => Some(*radius),
            Effect::ProgressiveBlur(p) if p.visible && p.background && p.max_radius() > 0.0 => Some(p.max_radius()),
            _ => None,
        }
    }
}

/// A layer or background blur whose radius ramps linearly from `start_radius`
/// at `start_offset` to `end_radius` at `end_offset` (Figma's progressive blur).
///
/// Offsets are positions in the layer's normalized object space: (0, 0) is its
/// top-left corner and (1, 1) its bottom-right, so (0.5, 0) -> (0.5, 1) is a
/// top-to-bottom ramp. Pixels before the start point use `start_radius`, pixels
/// past the end point use `end_radius`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ProgressiveBlurEffect {
    /// Blurs what is behind the layer (background blur) instead of the layer itself.
    pub background: bool,
    /// Blur radius (px) at `start_offset`.
    pub start_radius: f32,
    /// Blur radius (px) at `end_offset` (Figma's `radius`).
    pub end_radius: f32,
    pub start_offset: Vec2,
    pub end_offset: Vec2,
    pub visible: bool,
}

impl ProgressiveBlurEffect {
    /// The largest radius of the ramp (what the blur needs room for).
    pub fn max_radius(&self) -> f32 {
        self.start_radius.max(self.end_radius).max(0.0)
    }
}

impl Default for ProgressiveBlurEffect {
    /// Top-to-bottom ramp from sharp to a 16 px blur.
    fn default() -> Self {
        Self { background: false, start_radius: 0.0, end_radius: 16.0, start_offset: Vec2::new(0.5, 0.0), end_offset: Vec2::new(0.5, 1.0), visible: true }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NoiseMode {
    /// One color at `opacity`.
    Mono { color: Color },
    /// Two colors mixed per grain.
    Duo { color: Color, secondary: Color },
    /// Random hue per grain.
    Multi,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NoiseEffect {
    pub mode: NoiseMode,
    /// Grain size in px.
    pub size: f32,
    /// Share of covered pixels, 0..1.
    pub density: f32,
    pub opacity: f32,
    pub blend: BlendMode,
    /// Deterministic seed so renders are reproducible.
    pub seed: u32,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextureEffect {
    /// Texture scale in px.
    pub size: f32,
    /// Edge displacement in px.
    pub radius: f32,
    /// Keep the textured result inside the original shape.
    pub clip_to_shape: bool,
    pub seed: u32,
    pub visible: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlassEffect {
    /// Light direction in degrees (0 = from the right, clockwise).
    pub light_angle: f32,
    /// Highlight strength, 0..1.
    pub light_intensity: f32,
    /// How strongly the backdrop bends at the edges, 0..1.
    pub refraction: f32,
    /// Bevel depth in px.
    pub depth: f32,
    /// Chromatic split at the edges, 0..1.
    pub dispersion: f32,
    /// Backdrop blur radius in px.
    pub frost: f32,
    pub visible: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignH {
    #[default]
    Left,
    Center,
    Right,
    Justified,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAlignV {
    #[default]
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextAutoResize {
    #[default]
    None,
    WidthAndHeight,
    Height,
    Truncate,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextCase {
    #[default]
    Original,
    Upper,
    Lower,
    Title,
    SmallCaps,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TextDecoration {
    #[default]
    None,
    Underline,
    Strikethrough,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum LineHeight {
    /// The font's natural line height (ascent + descent + line gap).
    Auto,
    Pixels(f32),
    /// Percent of the font size: `Percent(120.0)` = 1.2 × font_size.
    Percent(f32),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextStyle {
    pub font_family: String,
    /// CSS weight 100..=900.
    pub font_weight: u16,
    pub italic: bool,
    pub font_size: f32,
    pub line_height: LineHeight,
    /// Letter spacing in px.
    pub letter_spacing: f32,
    pub paragraph_spacing: f32,
    pub align_h: TextAlignH,
    pub align_v: TextAlignV,
    pub case: TextCase,
    pub decoration: TextDecoration,
    /// Text colour lives here; `Node::fills` is unused for text nodes.
    pub fills: Vec<Paint>,
    /// OpenType feature settings, e.g. `("tnum", 1)`, `("liga", 0)`, `("ss01", 1)`.
    ///
    /// They reach the shaper (cosmic-text / harfrust) per style run. Limitation: cosmic-text shapes
    /// a word as one run with the features of its first character, so a feature set on part of a
    /// word applies to all of it (or to none of it when the word starts without it); on whole
    /// words it is exact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub features: Vec<(String, u32)>,
    /// Variable font axis settings, e.g. `("wght", 550.0)`, `("wdth", 85.0)`, `("slnt", -6.0)`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variation_axes: Vec<(String, f32)>,
}

impl TextStyle {
    pub fn get_axis(&self, tag: &str) -> Option<f32> {
        self.variation_axes.iter().find(|(k, _)| k.eq_ignore_ascii_case(tag)).map(|(_, v)| *v)
    }

    pub fn set_axis(&mut self, tag: &str, value: f32) {
        if let Some((_, v)) = self.variation_axes.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(tag)) {
            *v = value;
        } else {
            self.variation_axes.push((tag.to_ascii_lowercase(), value));
        }
    }

    pub fn remove_axis(&mut self, tag: &str) {
        self.variation_axes.retain(|(k, _)| !k.eq_ignore_ascii_case(tag));
    }

    pub fn is_feature_enabled(&self, tag: &str) -> bool {
        self.features.iter().find(|(k, _)| k.eq_ignore_ascii_case(tag)).is_some_and(|(_, v)| *v != 0)
    }

    pub fn set_feature(&mut self, tag: &str, enabled: bool) {
        let norm = tag.to_ascii_lowercase();
        if enabled {
            if let Some((_, v)) = self.features.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(&norm)) {
                *v = 1;
            } else {
                self.features.push((norm, 1));
            }
        } else if matches!(norm.as_str(), "liga" | "calt" | "kern" | "clig") {
            if let Some((_, v)) = self.features.iter_mut().find(|(k, _)| k.eq_ignore_ascii_case(&norm)) {
                *v = 0;
            } else {
                self.features.push((norm, 0));
            }
        } else {
            self.features.retain(|(k, _)| !k.eq_ignore_ascii_case(&norm));
        }
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_family: "Inter".into(),
            font_weight: 400,
            italic: false,
            font_size: 12.0,
            line_height: LineHeight::Auto,
            letter_spacing: 0.0,
            paragraph_spacing: 0.0,
            align_h: TextAlignH::Left,
            align_v: TextAlignV::Top,
            case: TextCase::Original,
            decoration: TextDecoration::None,
            fills: vec![Paint::solid(Color::BLACK)],
            features: Vec::new(),
            variation_axes: Vec::new(),
        }
    }
}

/// A run of characters (byte range into the text) with style overrides.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TextRun {
    pub start: usize,
    pub end: usize,
    pub style: TextStyle,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum LayoutGrid {
    Grid { size: f32, color: Color },
    Columns { count: u32, gutter: f32, offset: f32, color: Color },
    Rows { count: u32, gutter: f32, offset: f32, color: Color },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_hex_smart_expansion_and_named_colors() {
        assert_eq!(Color::from_hex(""), None);
        assert_eq!(Color::from_hex("   "), None);
        assert_eq!(Color::from_hex("transparent"), Some(Color::TRANSPARENT));
        assert_eq!(Color::from_hex("  TRANSPARENT "), Some(Color::TRANSPARENT));
        assert_eq!(Color::from_hex("#F"), Some(Color::rgb(0xFF, 0xFF, 0xFF)));
        assert_eq!(Color::from_hex("F"), Some(Color::rgb(0xFF, 0xFF, 0xFF)));
        assert_eq!(Color::from_hex("#F0"), Some(Color::rgb(0xF0, 0xF0, 0xF0)));
        assert_eq!(Color::from_hex("80"), Some(Color::rgb(0x80, 0x80, 0x80)));
        assert_eq!(Color::from_hex("#123"), Some(Color::rgb(0x11, 0x22, 0x33)));
        assert_eq!(Color::from_hex("#1234"), Some(Color::rgba(0x11, 0x22, 0x33, 0x44)));
        assert_eq!(Color::from_hex("#12345"), Some(Color::rgba(0x11, 0x22, 0x33, 0x45)));
        assert_eq!(Color::from_hex("#123456"), Some(Color::rgb(0x12, 0x34, 0x56)));
        assert_eq!(Color::from_hex("#1234567"), Some(Color::rgba(0x12, 0x34, 0x56, 0x77)));
        assert_eq!(Color::from_hex("#12345678"), Some(Color::rgba(0x12, 0x34, 0x56, 0x78)));
        assert_eq!(Color::from_hex("red"), Some(Color::rgb(255, 0, 0)));
        assert_eq!(Color::from_hex("blue"), Some(Color::rgb(0, 0, 255)));
        assert_eq!(Color::from_hex("coral"), Some(Color::rgb(255, 127, 80)));
        assert_eq!(Color::from_hex("rebeccapurple"), Some(Color::rgb(102, 51, 153)));
    }

    fn progressive(background: bool, start: f32, end: f32) -> ProgressiveBlurEffect {
        ProgressiveBlurEffect { background, start_radius: start, end_radius: end, ..Default::default() }
    }

    #[test]
    fn progressive_blur_reports_the_largest_radius_for_its_target_only() {
        let layer = Effect::ProgressiveBlur(progressive(false, 21.0, 4.0));
        assert_eq!(layer.layer_blur_radius(), Some(21.0));
        assert_eq!(layer.background_blur_radius(), None);
        let back = Effect::ProgressiveBlur(progressive(true, 0.0, 12.0));
        assert_eq!(back.background_blur_radius(), Some(12.0));
        assert_eq!(back.layer_blur_radius(), None);
        let flat = Effect::ProgressiveBlur(progressive(false, 0.0, 0.0));
        assert_eq!(flat.layer_blur_radius(), None, "a ramp of zero radii blurs nothing");
        let hidden = Effect::ProgressiveBlur(ProgressiveBlurEffect { visible: false, ..progressive(false, 5.0, 5.0) });
        assert!(!hidden.visible());
        assert_eq!(hidden.layer_blur_radius(), None);
    }

    #[test]
    fn progressive_blur_survives_the_compact_json_round_trip() {
        let e = Effect::ProgressiveBlur(ProgressiveBlurEffect {
            background: true,
            start_radius: 21.0,
            end_radius: 4.0,
            start_offset: Vec2::new(0.25, 0.1),
            end_offset: Vec2::new(0.75, 0.9),
            visible: true,
        });
        let json = serde_json::to_string(&e).unwrap();
        let back: Effect = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);
    }
}
