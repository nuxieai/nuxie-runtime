#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextSizing {
    #[default]
    AutoWidth = 0,
    AutoHeight = 1,
    Fixed = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextOverflow {
    #[default]
    Visible = 0,
    Hidden = 1,
    Clipped = 2,
    Ellipsis = 3,
    Fit = 4,
    FitFontSize = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum TextAlign {
    #[default]
    Left = 0,
    Right = 1,
    Center = 2,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTextFontError {
    message: &'static str,
}

impl std::fmt::Display for RawTextFontError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message)
    }
}

impl std::error::Error for RawTextFontError {}

/// Metrics retained by the decoded font owner, in one-em units and with the
/// same y-down signs as upstream `Font::LineMetrics`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTextFontLineMetrics {
    pub ascent: f32,
    pub descent: f32,
    pub cap_height: f32,
    pub x_height: f32,
}

/// A variable-font axis exposed by the decoded font owner.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTextFontAxis {
    pub tag: u32,
    pub min: f32,
    pub default: f32,
    pub max: f32,
}

/// A variable-axis value applied by [`RawTextFont::with_options`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RawTextFontCoord {
    pub axis: u32,
    pub value: f32,
}

/// An OpenType feature value applied by [`RawTextFont::with_options`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawTextFontFeature {
    pub tag: u32,
    pub value: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeColorGlyphGradientStop {
    pub offset: f32,
    pub color: u32,
    pub uses_foreground: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeColorGlyphPaint {
    Solid {
        color: u32,
    },
    LinearGradient {
        x0: f32,
        y0: f32,
        x1: f32,
        y1: f32,
        stops: Vec<RuntimeColorGlyphGradientStop>,
    },
    RadialGradient {
        x0: f32,
        y0: f32,
        r0: f32,
        x1: f32,
        y1: f32,
        r1: f32,
        stops: Vec<RuntimeColorGlyphGradientStop>,
    },
    SweepGradient {
        x0: f32,
        y0: f32,
        start_angle: f32,
        end_angle: f32,
        stops: Vec<RuntimeColorGlyphGradientStop>,
    },
    Image {
        bytes: std::sync::Arc<[u8]>,
        width: u32,
        height: u32,
        bearing_x: f32,
        bearing_y: f32,
        extent_x: f32,
        extent_y: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeColorGlyphLayer {
    pub path: nuxie_render_api::RawPath,
    pub paint: RuntimeColorGlyphPaint,
    pub uses_foreground: bool,
}
// Public host DTOs over the translated text owners. No shaping, line breaking,
// layout, or render-command implementation is duplicated here.
use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    renderer::to_render_raw_path,
    text::{font_hb::HbFont, raw_text::RawText as NativeRawText},
    text_engine::{self as native, Font, FontRef},
};
use harfrust::FontRef as HarfFontRef;
use nuxie_render_api::{Aabb, Factory, RawPath, RenderPaint, Renderer};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::DrawSettings;
use skrifa::outline::pen::NullPen;
use skrifa::raw::TableProvider;
use skrifa::{FontRef as SkrifaFontRef, GlyphId, MetadataProvider};
use std::{cell::RefCell, marker::PhantomData, rc::Rc, sync::Arc};

/// Whether embedded font bytes are safe for both translated text backends.
///
/// Authoring uses this before attaching bytes so malformed outlines cannot
/// defer a parser panic until shaping or drawing.
#[must_use]
pub fn embedded_font_is_parseable(font_bytes: &[u8]) -> bool {
    if HarfFontRef::new(font_bytes).is_err() {
        return false;
    }

    std::panic::catch_unwind(|| {
        let Ok(font) = SkrifaFontRef::new(font_bytes) else {
            return false;
        };
        let Ok(maxp) = font.maxp() else {
            return false;
        };
        let outlines = font.outline_glyphs();
        for glyph_index in 0..u32::from(maxp.num_glyphs()) {
            let Some(outline) = outlines.get(GlyphId::new(glyph_index)) else {
                continue;
            };
            if outline
                .draw(
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                    &mut NullPen,
                )
                .is_err()
            {
                return false;
            }
        }
        true
    })
    .unwrap_or(false)
}

#[derive(Clone)]
pub struct RawTextFont {
    native: FontRef,
    fallbacks: Rc<[RawTextFont]>,
}
impl std::fmt::Debug for RawTextFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawTextFont")
            .field("identity", &Rc::as_ptr(&self.native))
            .field("fallback_count", &self.fallbacks.len())
            .finish()
    }
}
impl RawTextFont {
    pub fn decode(bytes: impl Into<Arc<[u8]>>) -> Result<Self, RawTextFontError> {
        Self::decode_face(bytes, 0)
    }
    pub fn decode_face(bytes: impl Into<Arc<[u8]>>, index: u32) -> Result<Self, RawTextFontError> {
        HbFont::decode_face(&bytes.into(), index)
            .map(Self::from_native)
            .ok_or(RawTextFontError {
                message: "font bytes or face index are invalid",
            })
    }
    pub fn from_native(native: FontRef) -> Self {
        Self {
            native,
            fallbacks: Rc::from([]),
        }
    }
    pub fn native_handle(&self) -> FontRef {
        self.native.clone()
    }
    pub fn source_bytes(&self) -> Arc<[u8]> {
        self.hb().source_bytes()
    }
    pub fn face_index(&self) -> u32 {
        self.hb().face_index()
    }
    fn hb(&self) -> &HbFont {
        self.native
            .as_any()
            .downcast_ref()
            .expect("approved native font backend")
    }
    pub fn with_fallbacks(mut self, fallbacks: impl IntoIterator<Item = Self>) -> Self {
        self.fallbacks = fallbacks.into_iter().collect::<Vec<_>>().into();
        self
    }
    pub fn line_metrics(&self) -> RawTextFontLineMetrics {
        let m = self.native.line_metrics();
        RawTextFontLineMetrics {
            ascent: m.ascent,
            descent: m.descent,
            cap_height: m.cap_height,
            x_height: m.x_height,
        }
    }
    pub fn ascent(&self, size: f32) -> f32 {
        self.native.ascent(size)
    }
    pub fn descent(&self, size: f32) -> f32 {
        self.native.descent(size)
    }
    pub fn cap_height(&self, size: f32) -> f32 {
        self.native.cap_height(size)
    }
    pub fn x_height(&self, size: f32) -> f32 {
        self.native.x_height(size)
    }
    pub fn axis_count(&self) -> u16 {
        self.native.get_axis_count()
    }
    pub fn axis(&self, index: u16) -> RawTextFontAxis {
        let a = self.native.get_axis(index);
        RawTextFontAxis {
            tag: a.tag,
            min: a.min,
            default: a.def,
            max: a.max,
        }
    }
    pub fn axis_value(&self, tag: u32) -> f32 {
        self.native.get_axis_value(tag)
    }
    pub fn feature_value(&self, tag: u32) -> u32 {
        self.native.get_feature_value(tag)
    }
    pub fn features(&self) -> Vec<u32> {
        self.native.features()
    }
    pub fn weight(&self) -> u16 {
        self.native.get_weight()
    }
    pub fn is_italic(&self) -> bool {
        self.native.is_italic()
    }
    pub fn make_at_coords(&self, coords: &[RawTextFontCoord]) -> Self {
        self.with_options(coords, &[])
    }
    pub fn with_options(
        &self,
        coords: &[RawTextFontCoord],
        features: &[RawTextFontFeature],
    ) -> Self {
        let coords: Vec<_> = coords
            .iter()
            .map(|c| native::Coord {
                axis: c.axis,
                value: c.value,
            })
            .collect();
        let features: Vec<_> = features
            .iter()
            .map(|f| native::Feature {
                tag: f.tag,
                value: f.value,
            })
            .collect();
        Self {
            native: self.native.with_options(&coords, &features),
            fallbacks: self.fallbacks.clone(),
        }
    }
    pub fn has_glyph(&self, character: char) -> bool {
        self.native.has_glyph(character as u32)
    }
    pub fn glyph_path(&self, glyph: u32) -> RawPath {
        to_render_raw_path(&self.native.get_path(glyph as u16))
    }
    pub fn has_color_glyphs(&self) -> bool {
        self.native.has_color_glyphs()
    }
    pub fn is_color_glyph(&self, glyph: u32) -> bool {
        self.native.is_color_glyph(glyph as u16)
    }
    pub fn color_layers(&self, glyph: u32, foreground: u32) -> Vec<RuntimeColorGlyphLayer> {
        let mut layers = Vec::new();
        self.native
            .get_color_layers(glyph as u16, &mut layers, foreground);
        layers
            .into_iter()
            .map(|l| {
                let stops = l
                    .stops
                    .iter()
                    .map(|s| RuntimeColorGlyphGradientStop {
                        offset: s.offset,
                        color: s.color,
                        uses_foreground: l.use_foreground,
                    })
                    .collect();
                let paint = match l.paint_type {
                    native::ColorGlyphPaintType::Solid => {
                        RuntimeColorGlyphPaint::Solid { color: l.color }
                    }
                    native::ColorGlyphPaintType::LinearGradient => {
                        RuntimeColorGlyphPaint::LinearGradient {
                            x0: l.x0,
                            y0: l.y0,
                            x1: l.x1,
                            y1: l.y1,
                            stops,
                        }
                    }
                    native::ColorGlyphPaintType::RadialGradient => {
                        RuntimeColorGlyphPaint::RadialGradient {
                            x0: l.x0,
                            y0: l.y0,
                            r0: l.r0,
                            x1: l.x1,
                            y1: l.y1,
                            r1: l.r1,
                            stops,
                        }
                    }
                    native::ColorGlyphPaintType::SweepGradient => {
                        RuntimeColorGlyphPaint::SweepGradient {
                            x0: l.x0,
                            y0: l.y0,
                            start_angle: l.start_angle,
                            end_angle: l.end_angle,
                            stops,
                        }
                    }
                    native::ColorGlyphPaintType::Image => RuntimeColorGlyphPaint::Image {
                        bytes: l.image_bytes.into(),
                        width: l.image_width,
                        height: l.image_height,
                        bearing_x: l.image_bearing_x,
                        bearing_y: l.image_bearing_y,
                        extent_x: l.image_extent_x,
                        extent_y: l.image_extent_y,
                    },
                };
                RuntimeColorGlyphLayer {
                    path: to_render_raw_path(&l.path),
                    paint,
                    uses_foreground: l.use_foreground,
                }
            })
            .collect()
    }
}

#[derive(Clone)]
pub struct RawTextPaint {
    inner: Rc<RefCell<Box<dyn RenderPaint>>>,
}
impl std::fmt::Debug for RawTextPaint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawTextPaint")
            .field("identity", &Rc::as_ptr(&self.inner))
            .finish()
    }
}
impl RawTextPaint {
    pub fn new(factory: &mut dyn Factory) -> Self {
        Self {
            inner: Rc::new(RefCell::new(factory.make_render_paint())),
        }
    }
    pub fn with_paint<R>(&self, edit: impl FnOnce(&mut dyn RenderPaint) -> R) -> R {
        edit(self.inner.borrow_mut().as_mut())
    }
}

thread_local! { static HOST_FONTS: RefCell<Vec<RawTextFont>> = const { RefCell::new(Vec::new()) }; }
fn host_fallback(character: u32, index: u32, font: &dyn Font) -> Option<FontRef> {
    HOST_FONTS.with(|fonts| {
        fonts.borrow().iter().find_map(|primary| {
            let owns = std::ptr::eq(
                primary.native.as_ref() as *const dyn Font as *const (),
                font as *const dyn Font as *const (),
            ) || primary.fallbacks.iter().any(|fallback| {
                std::ptr::eq(
                    fallback.native.as_ref() as *const dyn Font as *const (),
                    font as *const dyn Font as *const (),
                )
            });
            if !owns {
                return None;
            }
            primary
                .fallbacks
                .iter()
                .filter(|fallback| fallback.native.has_glyph(character))
                .nth(index as usize)
                .map(RawTextFont::native_handle)
        })
    })
}
fn with_fonts<R>(fonts: &[RawTextFont], work: impl FnOnce() -> R) -> R {
    struct Restore(Vec<RawTextFont>);
    impl Drop for Restore {
        fn drop(&mut self) {
            HOST_FONTS.with(|fonts| {
                *fonts.borrow_mut() = std::mem::take(&mut self.0);
            });
        }
    }
    if fonts.iter().all(|font| font.fallbacks.is_empty()) {
        return work();
    }
    let previous = HOST_FONTS.with(|slot| slot.replace(fonts.to_vec()));
    let _restore = Restore(previous);
    native::with_host_fallback_proc(host_fallback, work)
}

pub struct RawText<'factory> {
    native: NativeRawText,
    factory: RuntimeFactoryHandle,
    fonts: Vec<RawTextFont>,
    update_count: u64,
    _factory: PhantomData<&'factory mut dyn Factory>,
}
impl<'factory> RawText<'factory> {
    pub fn new(factory: &'factory mut dyn Factory) -> Self {
        let factory = RuntimeFactoryHandle::from_factory(factory)
            .expect("RawText requires the host's persistent renderer factory");
        Self {
            native: NativeRawText::new(factory.clone()),
            factory,
            fonts: Vec::new(),
            update_count: 0,
            _factory: PhantomData,
        }
    }
    pub fn make_paint(&mut self) -> RawTextPaint {
        self.factory.with_factory_mut(RawTextPaint::new)
    }
    pub fn empty(&self) -> bool {
        self.native.empty()
    }
    pub fn append(
        &mut self,
        text: &str,
        paint: Option<RawTextPaint>,
        font: &RawTextFont,
        size: f32,
        line_height: f32,
        letter_spacing: f32,
        foreground: u32,
    ) {
        self.fonts.push(font.clone());
        self.native.append(
            text,
            paint.map(|p| p.inner),
            font.native_handle(),
            size,
            line_height,
            letter_spacing,
            foreground,
        )
    }
    pub fn append_default(&mut self, text: &str, paint: Option<RawTextPaint>, font: &RawTextFont) {
        self.append(text, paint, font, 16.0, -1.0, 0.0, 0xff000000)
    }
    pub fn clear(&mut self) {
        self.native.clear();
        self.fonts.clear();
    }
    pub fn sizing(&self) -> TextSizing {
        match self.native.sizing() {
            native::TextSizing::AutoWidth => TextSizing::AutoWidth,
            native::TextSizing::AutoHeight => TextSizing::AutoHeight,
            native::TextSizing::Fixed => TextSizing::Fixed,
            native::TextSizing::Unknown(_) => unreachable!(),
        }
    }
    pub fn overflow(&self) -> TextOverflow {
        match self.native.overflow() {
            native::TextOverflow::Visible => TextOverflow::Visible,
            native::TextOverflow::Hidden => TextOverflow::Hidden,
            native::TextOverflow::Clipped => TextOverflow::Clipped,
            native::TextOverflow::Ellipsis => TextOverflow::Ellipsis,
            native::TextOverflow::Fit => TextOverflow::Fit,
            native::TextOverflow::FitFontSize => TextOverflow::FitFontSize,
            native::TextOverflow::Unknown(_) => unreachable!(),
        }
    }
    pub fn align(&self) -> TextAlign {
        match self.native.align() {
            native::TextAlign::Left => TextAlign::Left,
            native::TextAlign::Right => TextAlign::Right,
            native::TextAlign::Center => TextAlign::Center,
            native::TextAlign::Unknown(_) => unreachable!(),
        }
    }
    pub fn max_width(&self) -> f32 {
        self.native.max_width()
    }
    pub fn max_height(&self) -> f32 {
        self.native.max_height()
    }
    pub fn paragraph_spacing(&self) -> f32 {
        self.native.paragraph_spacing()
    }
    pub fn set_sizing(&mut self, v: TextSizing) {
        self.native.set_sizing(match v {
            TextSizing::AutoWidth => native::TextSizing::AutoWidth,
            TextSizing::AutoHeight => native::TextSizing::AutoHeight,
            TextSizing::Fixed => native::TextSizing::Fixed,
        })
    }
    pub fn set_overflow(&mut self, v: TextOverflow) {
        self.native.set_overflow(match v {
            TextOverflow::Visible => native::TextOverflow::Visible,
            TextOverflow::Hidden => native::TextOverflow::Hidden,
            TextOverflow::Clipped => native::TextOverflow::Clipped,
            TextOverflow::Ellipsis => native::TextOverflow::Ellipsis,
            TextOverflow::Fit => native::TextOverflow::Fit,
            TextOverflow::FitFontSize => native::TextOverflow::FitFontSize,
        })
    }
    pub fn set_align(&mut self, v: TextAlign) {
        self.native.set_align(match v {
            TextAlign::Left => native::TextAlign::Left,
            TextAlign::Right => native::TextAlign::Right,
            TextAlign::Center => native::TextAlign::Center,
        })
    }
    pub fn set_max_width(&mut self, v: f32) {
        self.native.set_max_width(v)
    }
    pub fn set_max_height(&mut self, v: f32) {
        self.native.set_max_height(v)
    }
    pub fn set_paragraph_spacing(&mut self, v: f32) {
        self.native.set_paragraph_spacing(v)
    }
    pub fn bounds(&mut self) -> Aabb {
        self.update_count += u64::from(self.native.debug_dirty());
        let b = with_fonts(&self.fonts, || self.native.bounds());
        Aabb::new(b.min_x, b.min_y, b.max_x, b.max_y)
    }
    pub fn render(&mut self, renderer: &mut dyn Renderer, paint: Option<&RawTextPaint>) {
        self.update_count += u64::from(self.native.debug_dirty());
        with_fonts(&self.fonts, || {
            self.native.render(renderer, paint.map(|p| p.inner.clone()))
        })
    }
    #[doc(hidden)]
    pub fn debug_update_count(&self) -> u64 {
        self.update_count
    }
    #[doc(hidden)]
    pub fn debug_dirty(&self) -> bool {
        self.native.debug_dirty()
    }
    #[doc(hidden)]
    pub fn debug_style_count(&self) -> usize {
        self.native.debug_style_count()
    }
    #[doc(hidden)]
    pub fn debug_style_foreground(&self, index: usize) -> Option<u32> {
        self.native.debug_style_foreground(index)
    }
    #[doc(hidden)]
    pub fn debug_command_kinds(&self) -> Vec<&'static str> {
        self.native.debug_command_kinds()
    }
    #[doc(hidden)]
    pub fn debug_has_clip(&self) -> bool {
        self.native.debug_has_clip()
    }
    #[doc(hidden)]
    pub fn debug_style_path_bounds(&self) -> Vec<Aabb> {
        self.native
            .debug_style_path_bounds()
            .into_iter()
            .map(|b| Aabb::new(b.min_x, b.min_y, b.max_x, b.max_y))
            .collect()
    }
}
