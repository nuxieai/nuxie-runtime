//! Nuxie-owned text decoration: underlines and strikethroughs on text runs.
//!
//! Rive has no text decoration. `TextStyleDecoration` (type 60002) is a child
//! of a `TextStylePaint`, as upstream's `TextStyleBackground` is. When a style
//! carries one, `Text::build_render_styles` adds one rectangle per decorated
//! run on each drawn line to that style's own glyph path, so the line shares
//! the glyphs' paint, follows wrapping, alignment and ellipsis, and needs no
//! extra draw. Text without a decoration builds exactly the upstream path.
use crate::source::{
    component::Component,
    core::{
        Core, CoreHandle, CoreObject, CoreType, PropertySetterCompletion,
        binary_reader::BinaryReader,
        field_types::{core_callback_type::CallbackData, core_uint_type::CoreUintType},
    },
    core_context::CoreContext,
    generated::{
        component_base::ComponentBase,
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        text::text_style_paint_base::TextStylePaintBase,
    },
    importers::import_stack::ImportStack,
    math::aabb::Aabb,
    status_code::StatusCode,
    text::text_style_paint::TextStylePaint,
    text_engine::{DecorationMetric, DecorationMetrics, Font},
};
use std::any::Any;

/// `line` bit: a line under the run.
pub const UNDERLINE: u32 = 1;
/// `line` bit: a line through the run.
pub const LINE_THROUGH: u32 = 2;
/// The `line` property: a set of [`UNDERLINE`] and [`LINE_THROUGH`].
pub const LINE_PROPERTY_KEY: u16 = 60022;

/// Fallback geometry, per unit font size, for fonts that do not suggest
/// their own (or suggest no thickness).
const FALLBACK_THICKNESS: f32 = 0.06;
const FALLBACK_UNDERLINE_OFFSET: f32 = -0.1;
const FALLBACK_STRIKEOUT_OFFSET: f32 = 0.3;

#[derive(Default)]
pub struct TextStyleDecoration {
    component: Component,
    line: u32,
}

impl TextStyleDecoration {
    pub const TYPE_KEY: u16 = 60002;

    fn subtype(key: u16) -> bool {
        key == Self::TYPE_KEY || ComponentBase::is_type_of(key)
    }

    pub fn line(&self) -> u32 {
        self.line
    }

    /// Changes the lines and has the owning Text rebuild its paths.
    pub fn set_line(&mut self, line: u32) {
        if self.line == line {
            return;
        }
        self.line = line;
        let text = self.component.parent_handle().and_then(|style| {
            style
                .with(|style| style.as_component().and_then(|style| style.parent_handle()))
                .flatten()
        });
        if let Some(text) = text {
            text.with_mut(|text| {
                if let Some(text) = text.as_text_mut() {
                    text.mark_paint_dirty();
                }
            });
        }
    }

    fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.component.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(parent), Some(this)) = (
            self.component.parent_handle(),
            CoreObject::core(self).handle(),
        ) else {
            return StatusCode::InvalidObject;
        };
        if parent
            .with_downcast_mut::<TextStylePaint, _>(|style| style.set_decoration(this))
            .is_none()
        {
            return StatusCode::InvalidObject;
        }
        StatusCode::Ok
    }
}

impl CoreType for TextStyleDecoration {
    const TYPE_KEY: u16 = TextStyleDecoration::TYPE_KEY;
}

impl CoreObject for TextStyleDecoration {
    fn core(&self) -> &Core {
        &self.component.base.base
    }
    fn core_mut(&mut self) -> &mut Core {
        &mut self.component.base.base
    }
    fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        let mut cloned = Self {
            line: self.line,
            ..Self::default()
        };
        cloned
            .component
            .base
            .copy(&self.component.base, &mut Component::default());
        Some(Box::new(cloned))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        if key == LINE_PROPERTY_KEY {
            self.line = CoreUintType::deserialize(reader);
            return true;
        }
        let mut base = std::mem::take(&mut self.component.base);
        let handled = base.deserialize(key, reader, &mut self.component);
        self.component.base = base;
        handled
    }
}

impl CoreRegistryObject for TextStyleDecoration {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentParentId && self.component.base.set_parent_id_value(value) {
            completion.record(
                CoreObject::core(self),
                ComponentBase::PARENT_ID_PROPERTY_KEY,
            );
        }
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        if field == CoreField::ComponentName && self.component.base.set_name_value(value) {
            completion.record(CoreObject::core(self), ComponentBase::NAME_PROPERTY_KEY);
        }
    }
    fn set_color_with_completion(
        &mut self,
        _: CoreField,
        _: i32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_bool_with_completion(
        &mut self,
        _: CoreField,
        _: bool,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_double_with_completion(
        &mut self,
        _: CoreField,
        _: f32,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn set_int_with_completion(&mut self, _: CoreField, _: i32, _: &mut PropertySetterCompletion) {}
    fn set_callback_with_completion(
        &mut self,
        _: CoreField,
        _: CallbackData<'_>,
        _: &mut PropertySetterCompletion,
    ) {
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        match field {
            CoreField::ComponentParentId => self.component.base.parent_id(),
            _ => 0,
        }
    }
    fn get_string(&mut self, field: CoreField) -> String {
        match field {
            CoreField::ComponentName => self.component.base.name().to_owned(),
            _ => String::new(),
        }
    }
    fn get_color(&mut self, _: CoreField) -> i32 {
        0
    }
    fn get_bool(&mut self, _: CoreField) -> bool {
        false
    }
    fn get_double(&mut self, _: CoreField) -> f32 {
        0.0
    }
    fn get_int(&mut self, _: CoreField) -> i32 {
        0
    }
}

impl CoreCapabilities for TextStyleDecoration {
    fn component_on_dirty(&mut self, _: crate::source::component_dirt::ComponentDirt) -> bool {
        true
    }
    fn component_on_dirty_is_noop(&self) -> bool {
        true
    }
    fn lifecycle_validate(&mut self, context: &mut dyn CoreContext) -> Option<bool> {
        Some(
            self.component.validate(context)
                && context
                    .resolve(self.component.base.parent_id())
                    .is_some_and(|parent| parent.is_type_of(TextStylePaintBase::TYPE_KEY)),
        )
    }
    fn lifecycle_on_added_dirty(&mut self, context: &mut dyn CoreContext) -> Option<StatusCode> {
        Some(self.on_added_dirty(context))
    }
    fn lifecycle_import(&mut self, stack: &mut ImportStack) -> Option<StatusCode> {
        Some(self.component.import(stack))
    }
    fn as_component(&self) -> Option<&Component> {
        Some(&self.component)
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        Some(&mut self.component)
    }
}

/// The decoration `line` of the style a glyph is drawn with, or 0.
pub(crate) fn style_line(style: &CoreHandle) -> u32 {
    style
        .with_downcast::<TextStylePaint, _>(TextStylePaint::decoration)
        .flatten()
        .and_then(|decoration| decoration.with_downcast::<TextStyleDecoration, _>(|d| d.line()))
        .unwrap_or(0)
}

/// The decoration rectangles between `start_x` and `end_x` on a line whose
/// baseline is at `baseline` (the text's local space, y down).
pub(crate) fn line_rects(
    line: u32,
    metrics: DecorationMetrics,
    size: f32,
    start_x: f32,
    end_x: f32,
    baseline: f32,
) -> Vec<Aabb> {
    let mut rects = Vec::with_capacity(2);
    let mut add = |metric: DecorationMetric| {
        let top = baseline - metric.offset * size;
        let bottom = top + metric.thickness * size;
        if end_x > start_x && bottom > top {
            rects.push(Aabb::new(start_x, top, end_x, bottom));
        }
    };
    if line & UNDERLINE != 0 {
        add(resolved(metrics.underline, FALLBACK_UNDERLINE_OFFSET));
    }
    if line & LINE_THROUGH != 0 {
        add(resolved(metrics.strikeout, FALLBACK_STRIKEOUT_OFFSET));
    }
    rects
}

/// One decorated stretch of a drawn line: consecutive glyphs of one style.
pub(crate) struct DecorationSpan {
    pub style: CoreHandle,
    line: u32,
    start_x: f32,
    end_x: f32,
    baseline: f32,
    size: f32,
    metrics: DecorationMetrics,
}

impl DecorationSpan {
    /// The rectangles this span draws, in the text's local space (y down).
    pub fn rects(&self) -> Vec<Aabb> {
        line_rects(
            self.line,
            self.metrics,
            self.size,
            self.start_x,
            self.end_x,
            self.baseline,
        )
    }
}

fn resolved(metric: Option<DecorationMetric>, fallback_offset: f32) -> DecorationMetric {
    let metric = metric.unwrap_or(DecorationMetric {
        offset: fallback_offset,
        thickness: FALLBACK_THICKNESS,
    });
    DecorationMetric {
        offset: if metric.offset.is_finite() {
            metric.offset
        } else {
            fallback_offset
        },
        thickness: if metric.thickness.is_finite() && metric.thickness > 0.0 {
            metric.thickness
        } else {
            FALLBACK_THICKNESS
        },
    }
}

/// Collects decorated spans while a line's glyphs are laid out left to right.
/// A span reaches the end of its last inked glyph: spaces inside a run are
/// decorated, a line's trailing spaces are not.
#[derive(Default)]
pub(crate) struct DecorationSpans {
    current: Option<DecorationSpan>,
}

impl DecorationSpans {
    /// Records one glyph. Returns the span it closes, if any.
    #[allow(clippy::too_many_arguments)]
    pub fn glyph(
        &mut self,
        style: &CoreHandle,
        line: u32,
        x: f32,
        advance: f32,
        inked: bool,
        baseline: f32,
        size: f32,
        font: &dyn Font,
    ) -> Option<DecorationSpan> {
        if line == 0 {
            return self.current.take();
        }
        if let Some(span) = &mut self.current
            && span.style == *style
            && span.line == line
        {
            span.start_x = span.start_x.min(x);
            if inked {
                span.end_x = span.end_x.max(x + advance);
            }
            return None;
        }
        self.current.replace(DecorationSpan {
            style: style.clone(),
            line,
            start_x: x,
            end_x: if inked { x + advance } else { x },
            baseline,
            size,
            metrics: font.decoration_metrics(),
        })
    }

    /// Closes the line's last span.
    pub fn finish(&mut self) -> Option<DecorationSpan> {
        self.current.take()
    }
}
