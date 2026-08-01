//! Standalone text drawing without an authored `.riv` object.
//!
//! A `RawText` borrows one renderer factory for its complete lifetime. This
//! encodes the same factory-domain precondition as C++ while preventing a
//! dangling factory pointer in safe Rust.

pub use nuxie_runtime::{
    RawTextFont, RawTextFontError, RawTextPaint, TextAlign, TextOverflow, TextSizing,
};

use nuxie_render_api::{Aabb, ColorInt, Factory, Renderer};

pub struct RawText<'factory> {
    inner: nuxie_runtime::RuntimeRawText<'factory>,
}

impl std::fmt::Debug for RawText<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.inner.fmt(formatter)
    }
}

impl<'factory> RawText<'factory> {
    pub fn new(factory: &'factory mut dyn Factory) -> Self {
        Self {
            inner: nuxie_runtime::RuntimeRawText::new(factory),
        }
    }

    pub fn make_paint(&mut self) -> RawTextPaint {
        self.inner.make_paint()
    }

    pub fn empty(&self) -> bool {
        self.inner.empty()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn append(
        &mut self,
        text: &str,
        paint: Option<RawTextPaint>,
        font: &RawTextFont,
        size: f32,
        line_height: f32,
        letter_spacing: f32,
        foreground_color: ColorInt,
    ) {
        self.inner.append(
            text,
            paint,
            font,
            size,
            line_height,
            letter_spacing,
            foreground_color,
        );
    }

    pub fn append_default(&mut self, text: &str, paint: Option<RawTextPaint>, font: &RawTextFont) {
        self.inner.append_default(text, paint, font);
    }

    pub fn clear(&mut self) {
        self.inner.clear();
    }

    pub fn sizing(&self) -> TextSizing {
        self.inner.sizing()
    }

    pub fn set_sizing(&mut self, value: TextSizing) {
        self.inner.set_sizing(value);
    }

    pub fn overflow(&self) -> TextOverflow {
        self.inner.overflow()
    }

    pub fn set_overflow(&mut self, value: TextOverflow) {
        self.inner.set_overflow(value);
    }

    pub fn align(&self) -> TextAlign {
        self.inner.align()
    }

    pub fn set_align(&mut self, value: TextAlign) {
        self.inner.set_align(value);
    }

    pub fn max_width(&self) -> f32 {
        self.inner.max_width()
    }

    pub fn set_max_width(&mut self, value: f32) {
        self.inner.set_max_width(value);
    }

    pub fn max_height(&self) -> f32 {
        self.inner.max_height()
    }

    pub fn set_max_height(&mut self, value: f32) {
        self.inner.set_max_height(value);
    }

    pub fn paragraph_spacing(&self) -> f32 {
        self.inner.paragraph_spacing()
    }

    pub fn set_paragraph_spacing(&mut self, value: f32) {
        self.inner.set_paragraph_spacing(value);
    }

    pub fn bounds(&mut self) -> Aabb {
        self.inner.bounds()
    }

    pub fn render(&mut self, renderer: &mut dyn Renderer, override_paint: Option<&RawTextPaint>) {
        self.inner.render(renderer, override_paint);
    }

    #[doc(hidden)]
    pub fn debug_update_count(&self) -> u64 {
        self.inner.debug_update_count()
    }

    #[doc(hidden)]
    pub fn debug_dirty(&self) -> bool {
        self.inner.debug_dirty()
    }

    #[doc(hidden)]
    pub fn debug_style_count(&self) -> usize {
        self.inner.debug_style_count()
    }

    #[doc(hidden)]
    pub fn debug_style_foreground(&self, index: usize) -> Option<u32> {
        self.inner.debug_style_foreground(index)
    }

    #[doc(hidden)]
    pub fn debug_command_kinds(&self) -> Vec<&'static str> {
        self.inner.debug_command_kinds()
    }

    #[doc(hidden)]
    pub fn debug_has_clip(&self) -> bool {
        self.inner.debug_has_clip()
    }

    #[doc(hidden)]
    pub fn debug_style_path_bounds(&self) -> Vec<Aabb> {
        self.inner.debug_style_path_bounds()
    }
}
