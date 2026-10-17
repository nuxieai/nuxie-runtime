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
    math::{aabb::Aabb, mat2d::Mat2D, path_types::PathDirection, raw_path::RawPath, vec2d::Vec2D},
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

    /// Changes the lines and has the owning Text rebuild its paths. Returns
    /// whether the value changed.
    pub fn set_line(&mut self, line: u32) -> bool {
        if self.line == line {
            return false;
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
        true
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

/// One decorated stretch of a drawn line: consecutive glyphs of one style.
struct Span {
    style: CoreHandle,
    line: u32,
    start_x: f32,
    end_x: f32,
    open: bool,
    baseline: f32,
    size: f32,
    metrics: DecorationMetrics,
}

/// One modified glyph's own piece of line: modifiers move, turn and fade
/// glyphs one by one, so each carries its line through its own transform.
struct Piece {
    style: CoreHandle,
    line: u32,
    x: f32,
    advance: f32,
    /// Maps the glyph's layout space (origin at its centre on the baseline)
    /// to the text's local space.
    transform: Mat2D,
    opacity: f32,
    size: f32,
    metrics: DecorationMetrics,
}

/// A path to add to a style's glyph path, at an opacity.
pub(crate) struct DecorationPath {
    pub style: CoreHandle,
    pub path: RawPath,
    pub opacity: f32,
}

/// Collects one drawn line's decorations while its glyphs are laid out, and
/// builds them when the line ends. Lines reach no further than the line's
/// ink: spaces inside the line are decorated, its leading and trailing spaces
/// are not, with or without modifiers.
#[derive(Default)]
pub(crate) struct LineDecorations {
    spans: Vec<Span>,
    pieces: Vec<Piece>,
    /// The left and right edges of the line's ink.
    ink: Option<(f32, f32)>,
}

impl LineDecorations {
    fn ink(&mut self, x: f32, advance: f32, inked: bool) {
        if inked {
            let end = x + advance;
            self.ink = Some(self.ink.map_or((x, end), |(start, ink_end)| {
                (start.min(x), ink_end.max(end))
            }));
        }
    }

    /// Records one laid-out glyph of a line no modifier touches.
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
    ) {
        self.ink(x, advance, inked);
        let open = self.spans.last_mut().filter(|span| span.open);
        if line == 0 {
            if let Some(span) = open {
                span.open = false;
            }
            return;
        }
        let metrics = font.decoration_metrics();
        if let Some(span) = open {
            // A fallback font's run takes its own font's lines.
            if span.style == *style
                && span.line == line
                && span.size == size
                && span.metrics == metrics
            {
                span.start_x = span.start_x.min(x);
                span.end_x = span.end_x.max(x + advance);
                return;
            }
            span.open = false;
        }
        self.spans.push(Span {
            style: style.clone(),
            line,
            start_x: x,
            end_x: x + advance,
            open: true,
            baseline,
            size,
            metrics,
        });
    }

    /// Records one glyph a modifier may have moved, turned or faded.
    #[allow(clippy::too_many_arguments)]
    pub fn modified_glyph(
        &mut self,
        style: &CoreHandle,
        line: u32,
        x: f32,
        advance: f32,
        inked: bool,
        transform: Mat2D,
        opacity: f32,
        size: f32,
        font: &dyn Font,
    ) {
        self.ink(x, advance, inked);
        if line != 0 {
            self.pieces.push(Piece {
                style: style.clone(),
                line,
                x,
                advance,
                transform,
                opacity,
                size,
                metrics: font.decoration_metrics(),
            });
        }
    }

    /// The line's decoration paths, trimmed to its ink at both edges: a
    /// line's trailing spaces sit at its left in right-to-left text.
    pub fn finish(self) -> Vec<DecorationPath> {
        let Some((ink_start, ink_end)) = self.ink else {
            return Vec::new();
        };
        let mut paths = Vec::new();
        for span in self.spans {
            let mut path = RawPath::default();
            for rect in line_rects(
                span.line,
                span.metrics,
                span.size,
                span.start_x.max(ink_start),
                span.end_x.min(ink_end),
                span.baseline,
            ) {
                path.add_rect(rect, PathDirection::Clockwise);
            }
            paths.push(DecorationPath {
                style: span.style,
                path,
                opacity: 1.0,
            });
        }
        for piece in self.pieces {
            let centre = piece.advance / 2.0;
            let start = piece.x.max(ink_start) - piece.x;
            let end = (piece.x + piece.advance).min(ink_end) - piece.x;
            let mut path = RawPath::default();
            for rect in line_rects(
                piece.line,
                piece.metrics,
                piece.size,
                start - centre,
                end - centre,
                0.0,
            ) {
                let corners = [
                    Vec2D::new(rect.left(), rect.top()),
                    Vec2D::new(rect.right(), rect.top()),
                    Vec2D::new(rect.right(), rect.bottom()),
                    Vec2D::new(rect.left(), rect.bottom()),
                ];
                path.move_to_point(piece.transform * corners[0]);
                for corner in &corners[1..] {
                    path.line_to_point(piece.transform * *corner);
                }
                path.close();
            }
            paths.push(DecorationPath {
                style: piece.style,
                path,
                opacity: piece.opacity,
            });
        }
        paths
    }
}
