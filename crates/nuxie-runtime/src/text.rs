use anyhow::{Context, Result, bail};
use harfrust::{
    Direction, Feature, FontRef as HarfFontRef, ShapeOptions, ShaperData, ShaperInstance,
    Tag as HarfTag, UnicodeBuffer,
};
use nuxie_binary::RuntimeFile;
use nuxie_graph::{
    ArtboardGraph, DataBindNode, PathGeometryNode, ShapePaintContainerNode, ShapePaintKind,
    ShapePaintPathKind, ShapePaintStateNode,
};
use nuxie_render_api::{Aabb as RenderAabb, Vec2D as RenderVec2D};
use nuxie_schema::definition_by_name;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::pen::PathStyle;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::setting::VariationSetting;
use skrifa::{FontRef as SkrifaFontRef, GlyphId, MetadataProvider, Tag as SkrifaTag};
use std::collections::BTreeSet;

use crate::data_bind_flags_apply_source_to_target;
use crate::draw::{
    RuntimeLayoutBounds, RuntimePathMeasure, runtime_path_geometry_commands,
    runtime_shape_paint_command,
};
use crate::properties::{joystick_x_property_key, joystick_y_property_key, property_key_for_name};
use crate::{ArtboardInstance, Mat2D, RuntimeDrawCommand, RuntimePathCommand};
use crate::{RuntimeShapePaintCommand, RuntimeShapePaintKind, RuntimeShapePaintPathKind};
use std::collections::BTreeMap;

const TEXT_SHAPE_SCALE: i32 = 2048;
const TEXT_SHAPE_SCALE_F32: f32 = TEXT_SHAPE_SCALE as f32;
const TEXT_SIZING_AUTO_WIDTH: u64 = 0;
const TEXT_SIZING_AUTO_HEIGHT: u64 = 1;
const TEXT_SIZING_FIXED: u64 = 2;
const TEXT_OVERFLOW_VISIBLE: u64 = 0;
const TEXT_OVERFLOW_HIDDEN: u64 = 1;
const TEXT_OVERFLOW_CLIPPED: u64 = 2;
const TEXT_OVERFLOW_ELLIPSIS: u64 = 3;
const TEXT_OVERFLOW_FIT: u64 = 4;
const TEXT_OVERFLOW_FIT_FONT_SIZE: u64 = 5;
const TEXT_TRIM_NONE: u64 = 0;
const TEXT_TRIM_TOP_CAP: u64 = 1;
const TEXT_TRIM_TOP_EX: u64 = 2;
const TEXT_TRIM_BOTTOM_ALPHABETIC: u64 = 1;
const TEXT_TRIM_BOTTOM_TEXT: u64 = 2;
const LAYOUT_SCALE_TYPE_HUG: u64 = 2;

/// Whether embedded font bytes can be parsed by both runtime text backends.
///
/// Dynamic authoring calls this before publishing a structural edit so a
/// committed scene cannot defer malformed-font failure until layout or draw.
#[must_use]
pub fn embedded_font_is_parseable(font_bytes: &[u8]) -> bool {
    HarfFontRef::new(font_bytes).is_ok() && SkrifaFontRef::new(font_bytes).is_ok()
}

pub fn static_text_support_error(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    text_local: usize,
) -> Option<String> {
    StaticTextSlice::from_graph(runtime, graph, text_local)
        .err()
        .map(|error| error.to_string())
}

pub(crate) fn static_text_constraint_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local)
        && let Ok(Some(bounds)) = slice.local_bounds(runtime, instance)
    {
        return Some(bounds);
    }
    static_fixed_text_constraint_bounds(runtime, graph, instance, text_local, None)
}

pub(crate) fn static_text_layout_measure_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local)
        && let Ok(Some(bounds)) =
            slice.measure_bounds_with_layout_constraint(runtime, instance, layout_constraint)
    {
        return Some(bounds);
    }
    static_fixed_text_constraint_bounds(runtime, graph, instance, text_local, None).map(
        |(_x, _y, width, height)| {
            (
                0.0,
                0.0,
                width.min(layout_constraint.width),
                height.min(layout_constraint.height),
            )
        },
    )
}

pub(crate) fn static_text_controlled_layout_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local)
        && let Ok(Some(bounds)) =
            slice.local_bounds_with_layout_constraint(runtime, instance, layout_constraint)
    {
        return Some(bounds);
    }
    static_fixed_text_constraint_bounds(
        runtime,
        graph,
        instance,
        text_local,
        Some(layout_constraint),
    )
}

pub(crate) fn static_text_clip_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Result<Option<StaticTextClipBounds>> {
    StaticTextSlice::from_graph(runtime, graph, text_local)?.clip_bounds(
        runtime,
        instance,
        layout_constraint,
    )
}

pub(crate) fn static_text_caret_geometry(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
    text_world: Mat2D,
    byte_offset: usize,
) -> Option<(RenderVec2D, RenderVec2D)> {
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local).ok()?;
    if !slice.text_geometry_supported(runtime, instance).ok()? {
        return None;
    }
    slice
        .shaped_layout(runtime, instance, layout_constraint, text_world)
        .ok()??
        .caret(byte_offset)
}

pub(crate) fn static_text_hit(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
    text_world: Mat2D,
    point: RenderVec2D,
) -> Option<usize> {
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local).ok()?;
    if !slice.text_geometry_supported(runtime, instance).ok()? {
        return None;
    }
    slice
        .shaped_layout(runtime, instance, layout_constraint, text_world)
        .ok()??
        .hit(point)
}

pub(crate) fn static_text_selection_rects(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
    text_world: Mat2D,
    range: std::ops::Range<usize>,
) -> Vec<RenderAabb> {
    let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local) else {
        return Vec::new();
    };
    if !slice
        .text_geometry_supported(runtime, instance)
        .unwrap_or(false)
    {
        return Vec::new();
    }
    let Ok(Some(layout)) = slice.shaped_layout(runtime, instance, layout_constraint, text_world)
    else {
        return Vec::new();
    };
    layout.selection_rects(range)
}

#[allow(clippy::arithmetic_side_effects)]
fn static_fixed_text_constraint_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Option<(f32, f32, f32, f32)> {
    let text_object = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == text_local && object.type_name == Some("Text"))?;
    let runtime_object = runtime.object(text_object.global_id as usize)?;
    let uint_property = |name| {
        let key = property_key_for_name("Text", name)?;
        instance
            .uint_property(text_local, key)
            .or_else(|| runtime_object.uint_property(name))
    };
    let authored_sizing = uint_property("sizingValue").unwrap_or(TEXT_SIZING_AUTO_WIDTH);
    let effective_sizing = layout_constraint
        .map(|constraint| constraint.effective_sizing(authored_sizing))
        .unwrap_or(authored_sizing);
    if effective_sizing != TEXT_SIZING_FIXED {
        return None;
    }
    let double_property = |name| {
        let key = property_key_for_name("Text", name)?;
        instance
            .double_property(text_local, key)
            .or_else(|| runtime_object.double_property(name))
    };
    let authored_width = double_property("width").unwrap_or(0.0);
    let authored_height = double_property("height").unwrap_or(0.0);
    let (width, height) = layout_constraint
        .map(|constraint| (constraint.width, constraint.height))
        .unwrap_or((authored_width, authored_height));
    let origin_x = double_property("originX").unwrap_or(0.0);
    let origin_y = double_property("originY").unwrap_or(0.0);
    Some((-width * origin_x, -height * origin_y, width, height))
}

pub(crate) fn text_input_layout_measure_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_input_local: usize,
    layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    StaticTextSlice::from_text_input_graph(runtime, graph, text_input_local)
        .ok()?
        .measure_bounds_with_layout_constraint(runtime, instance, layout_constraint)
        .ok()
        .flatten()
}

pub(crate) fn runtime_text_shape_paint_commands(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    command: &RuntimeDrawCommand,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Result<Vec<RuntimeShapePaintCommand>> {
    let text_local = command
        .local_id
        .context("text draw command missing local id")?;
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local)?;
    let render_opacity = instance
        .component(text_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let text_world =
        instance.runtime_component_world_transform_with_bounds(text_local, graph, layout_bounds);
    let render_data = slice.render_data(runtime, instance, layout_constraint, text_world)?;
    if render_data
        .path_buckets_by_style
        .iter()
        .all(|buckets| buckets.is_empty())
    {
        return Ok(Vec::new());
    }
    let shape_world = text_world.multiply(render_data.local_transform);
    // C++ text draw isolates the glyph path transform even when clipping
    // elides the drawable-level save.
    let needs_save_operation = true;
    let text_blend_mode_value = slice.text_blend_mode_value(runtime, instance)?;
    let style_order = slice.ordered_style_indices(runtime, instance)?;

    let mut commands = Vec::new();
    for style_index in style_order {
        let style = &slice.styles[style_index];
        let path_buckets = render_data.path_buckets_by_style[style_index].clone();
        let Some(container) = style.container else {
            continue;
        };
        let mut allocated_text_paint_pool = false;
        for path_bucket in order_opacity_buckets_like_cpp(path_buckets) {
            commands.extend(container.paints.iter().filter_map(|paint| {
                let mut path_commands = path_bucket.commands.clone();
                if paint.path_kind == Some(ShapePaintPathKind::World) {
                    transform_path_commands(&mut path_commands, shape_world);
                }
                let mut command = runtime_shape_paint_command(
                    instance,
                    paint,
                    text_blend_mode_value,
                    needs_save_operation,
                    render_opacity * path_bucket.opacity,
                    shape_world,
                    path_commands,
                    true,
                    false,
                    true,
                )?;
                command.shape_world_override = Some(shape_world);
                if command.paint_type == RuntimeShapePaintKind::Fill {
                    command.path_kind = RuntimeShapePaintPathKind::LocalClockwise;
                }
                command.uses_temporary_paint = path_bucket.opacity != 1.0;
                if !command.uses_temporary_paint && !allocated_text_paint_pool {
                    command.allocates_text_paint_pool = true;
                    allocated_text_paint_pool = true;
                }
                Some(command)
            }));
        }
    }
    Ok(commands)
}

pub(crate) fn runtime_text_input_shape_paint_commands(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    drawable_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Result<Vec<RuntimeShapePaintCommand>> {
    // Ported from C++ `src/text/text_input_drawable.cpp` and the draw-path
    // half of `src/text/raw_text_input.cpp`: TextInputDrawable paints paths
    // generated by its parent TextInput, but draws them with the TextInput's
    // world transform.
    let drawable_component = component_for_local(graph, drawable_local).with_context(|| {
        format!("TextInputDrawable local {drawable_local} component is missing")
    })?;
    let text_input_local = drawable_component
        .parent_local
        .context("TextInputDrawable missing TextInput parent")?;
    if type_for_local(graph, text_input_local) != Some("TextInput") {
        bail!("TextInputDrawable parent is not TextInput");
    }
    let container = graph
        .shape_paint_containers
        .iter()
        .find(|container| container.local_id == drawable_local)
        .with_context(|| {
            format!("TextInputDrawable local {drawable_local} missing shape paint container")
        })?;
    let slice = StaticTextSlice::from_text_input_graph(runtime, graph, text_input_local)?;
    let layout_constraint =
        instance.runtime_text_input_layout_constraint(text_input_local, graph, layout_bounds);
    let text_input_world = instance.runtime_component_world_transform_with_bounds(
        text_input_local,
        graph,
        layout_bounds,
    );
    let render_opacity = instance
        .component(drawable_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let needs_save_operation = true;

    let path_buckets = match container.type_name {
        "TextInputText" | "TextInputSelectedText" => {
            let render_data =
                slice.render_data(runtime, instance, layout_constraint, text_input_world)?;
            let shape_world = text_input_world.multiply(render_data.local_transform);
            return slice.text_input_paint_commands(
                instance,
                container,
                needs_save_operation,
                render_opacity,
                shape_world,
                render_data
                    .path_buckets_by_style
                    .into_iter()
                    .flatten()
                    .collect(),
            );
        }
        "TextInputCursor" => vec![StaticTextPathBucket {
            opacity: 1.0,
            commands: slice.text_input_cursor_path(runtime, instance)?,
        }],
        "TextInputSelection" => vec![StaticTextPathBucket {
            opacity: 1.0,
            commands: Vec::new(),
        }],
        type_name => bail!("unsupported TextInputDrawable type {type_name}"),
    };

    slice.text_input_paint_commands(
        instance,
        container,
        needs_save_operation,
        render_opacity,
        text_input_world,
        path_buckets,
    )
}

struct StaticTextSlice<'a> {
    kind: StaticTextKind,
    text_local: usize,
    text_global: u32,
    runs: Vec<StaticTextRun>,
    styles: Vec<StaticTextStyle<'a>>,
    modifiers: Vec<StaticTextModifierGroup>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticTextKind {
    Text,
    TextInput,
}

#[derive(Debug, Clone)]
struct StaticTextRun {
    local_id: usize,
    global_id: u32,
    style_local: usize,
    text_property_owner: &'static str,
}

#[derive(Debug, Clone)]
struct StaticResolvedRun {
    local_id: usize,
    global_id: u32,
    style_local: usize,
    char_start: usize,
    char_len: usize,
    text: String,
}

#[derive(Debug, Clone)]
struct StaticTextStyle<'a> {
    local_id: usize,
    global_id: u32,
    name: Option<String>,
    container: Option<&'a ShapePaintContainerNode>,
    font_asset_id: Option<u32>,
    font_bytes: Option<&'a [u8]>,
    variations: Vec<(u32, f32)>,
}

#[derive(Debug, Clone, Copy)]
struct StaticTextLine<'a> {
    text: &'a str,
    char_start: usize,
    line_index: usize,
    soft_wrap_skipped_start: Option<usize>,
    terminal_soft_wrap_skipped_end: Option<usize>,
}

/// The authoritative vertical geometry for one shaped line.
///
/// This is the pure-Rust equivalent of Rive's `GlyphLine::{top, baseline,
/// bottom}`. Every consumer must use these values rather than reconstructing a
/// global line height: authored line height is per style/run, and the first
/// line deliberately keeps the font's natural ascent.
#[derive(Debug, Clone, Copy, Default)]
struct StaticTextLineMetrics {
    top: f32,
    baseline: f32,
    bottom: f32,
}

#[derive(Debug, Clone, Copy)]
struct StaticTextLayoutInfo {
    ellipsis_line: Option<usize>,
    is_ellipsis_line_last: bool,
    paragraph_width: f32,
    align_value: u64,
    top_trim: f32,
    min_y: f32,
    x_offset: f32,
    y_offset: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticTextLineIteration {
    Draw,
    Skip,
    Stop,
}

#[derive(Debug, Clone, Copy, Default)]
struct StaticVerticalTrim {
    top: f32,
    bottom: f32,
}

#[derive(Debug, Clone)]
struct StaticTextRenderData {
    path_buckets_by_style: Vec<Vec<StaticTextPathBucket>>,
    local_transform: Mat2D,
}

/// One authoritative shaping/layout result shared by drawing and editor reads.
///
/// Keeping caret, hit, and selection geometry on this exact path prevents the
/// editor from growing an observation-side text layout that can drift from the
/// glyphs the runtime draws.
#[derive(Clone)]
struct StaticShapedTextLayout {
    text: String,
    lines: Vec<StaticShapedTextLine>,
    caret_boundaries: Option<Vec<StaticCaretBoundary>>,
    local_transform: Mat2D,
    shape_world: Mat2D,
    has_geometric_modifiers: bool,
    has_non_monotone_advances: bool,
}

#[derive(Clone)]
struct StaticShapedTextLine {
    line_index: usize,
    char_start: usize,
    char_end: usize,
    soft_wrap_skipped_start: Option<usize>,
    terminal_soft_wrap_skipped_end: Option<usize>,
    start_x: f32,
    end_x: f32,
    top: f32,
    baseline: f32,
    bottom: f32,
    glyphs: Vec<StaticPositionedTextGlyph>,
}

#[derive(Clone)]
struct StaticPositionedTextGlyph {
    glyph: StyledTextGlyph,
    x: f32,
    modifier_transform: Mat2D,
    modifier_opacity: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticCaretAffinity {
    Upstream,
    Downstream,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticShapedTextPurpose {
    Render,
    Geometry,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct StaticCaretSegment {
    top: RenderVec2D,
    bottom: RenderVec2D,
}

#[derive(Debug, Clone)]
struct StaticCaretBoundary {
    byte_offset: usize,
    upstream: Option<StaticCaretSegment>,
    downstream: Option<StaticCaretSegment>,
}

#[derive(Debug, Clone, Copy)]
struct StaticPositionedTextCluster {
    char_start: usize,
    char_end: usize,
    start_x: f32,
    end_x: f32,
    first_glyph: usize,
    last_glyph: usize,
}

#[derive(Debug, Default, Clone, Copy)]
struct StaticCaretBuildWork {
    glyph_visits: usize,
    boundary_visits: usize,
}

impl StaticCaretBoundary {
    fn segment(&self, affinity: StaticCaretAffinity) -> Option<StaticCaretSegment> {
        match affinity {
            StaticCaretAffinity::Upstream => self.upstream,
            StaticCaretAffinity::Downstream => self.downstream,
        }
    }
}

impl StaticShapedTextLayout {
    fn caret(&self, byte_offset: usize) -> Option<(RenderVec2D, RenderVec2D)> {
        if !self.geometry_is_finite() {
            return None;
        }
        let char_index = self.char_index_at_byte(byte_offset)?;
        self.caret_for_char_index(char_index, StaticCaretAffinity::Downstream)
            .filter(|(top, bottom)| text_point_is_finite(*top) && text_point_is_finite(*bottom))
    }

    fn caret_for_char_index(
        &self,
        char_index: usize,
        affinity: StaticCaretAffinity,
    ) -> Option<(RenderVec2D, RenderVec2D)> {
        let segment = self
            .caret_boundaries
            .as_ref()?
            .get(char_index)?
            .segment(affinity)?;
        Some((segment.top, segment.bottom))
    }

    fn hit(&self, point: RenderVec2D) -> Option<usize> {
        if !self.geometry_is_finite() || !text_point_is_finite(point) {
            return None;
        }
        let determinant = self.shape_world.determinant();
        if !determinant.is_finite() || determinant == 0.0 {
            return None;
        }
        if self.has_geometric_modifiers || self.has_non_monotone_advances {
            let mut best: Option<(f32, usize, StaticCaretAffinity)> = None;
            for boundary in self.caret_boundaries.as_ref()? {
                for affinity in [
                    StaticCaretAffinity::Upstream,
                    StaticCaretAffinity::Downstream,
                ] {
                    let Some(segment) = boundary.segment(affinity) else {
                        continue;
                    };
                    let distance =
                        point_segment_distance_squared(point, segment.top, segment.bottom);
                    if !distance.is_finite() {
                        continue;
                    }
                    if best.is_none_or(|best| {
                        distance < best.0
                            || (distance == best.0
                                && text_hit_candidate_wins_tie(
                                    boundary.byte_offset,
                                    affinity,
                                    best.1,
                                    best.2,
                                ))
                    }) {
                        best = Some((distance, boundary.byte_offset, affinity));
                    }
                }
            }
            return best.map(|(_, byte_offset, _)| byte_offset);
        }
        let inverse = self.shape_world.invert_or_identity();
        let (x, y) = inverse.transform_point(point.x, point.y);
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        let line_index = self
            .lines
            .iter()
            .position(|line| y <= line.bottom)
            .or_else(|| self.lines.len().checked_sub(1))?;
        let line = self.lines.get(line_index)?;
        let mut char_index = line.hit_char_index(x).min(self.text.chars().count());
        if char_index == line.char_end
            && let Some(next_line) = line_index
                .checked_add(1)
                .and_then(|next_line| self.lines.get(next_line))
            && next_line.soft_wrap_skipped_start == Some(char_index)
        {
            char_index = next_line.char_start;
        } else if char_index == line.char_end
            && let Some(skipped_end) = line.terminal_soft_wrap_skipped_end
        {
            char_index = skipped_end;
        }
        Some(char_byte_index(&self.text, char_index))
    }

    fn selection_rects(&self, range: std::ops::Range<usize>) -> Vec<RenderAabb> {
        if !self.geometry_is_finite() || range.is_empty() {
            return Vec::new();
        }
        let Some(start) = self.char_index_at_byte(range.start) else {
            return Vec::new();
        };
        let Some(end) = self.char_index_at_byte(range.end) else {
            return Vec::new();
        };
        if start >= end {
            return Vec::new();
        }

        let mut rects = Vec::new();
        for line in &self.lines {
            let selected_start = start.max(line.char_start);
            let selected_end = end.min(line.char_end);
            if selected_start >= selected_end {
                continue;
            }
            if let Some(rect) = line.selection_rect(
                selected_start,
                selected_end,
                self.shape_world,
                self.has_geometric_modifiers,
                self.has_non_monotone_advances
                    && line
                        .glyphs
                        .iter()
                        .any(|positioned| positioned.glyph.advance < 0.0),
            ) {
                rects.push(rect);
            }
        }
        if rects.iter().all(|rect| text_aabb_is_finite(*rect)) {
            rects
        } else {
            Vec::new()
        }
    }

    fn char_index_at_byte(&self, byte_offset: usize) -> Option<usize> {
        self.caret_boundaries
            .as_ref()?
            .binary_search_by_key(&byte_offset, |boundary| boundary.byte_offset)
            .ok()
    }

    fn geometry_is_finite(&self) -> bool {
        self.local_transform.0.iter().all(|value| value.is_finite())
            && self.shape_world.0.iter().all(|value| value.is_finite())
            && self
                .lines
                .iter()
                .all(StaticShapedTextLine::geometry_is_finite)
            && self.caret_boundaries.as_ref().is_some_and(|boundaries| {
                boundaries.iter().all(|boundary| {
                    [boundary.upstream, boundary.downstream]
                        .into_iter()
                        .flatten()
                        .all(|segment| {
                            text_point_is_finite(segment.top)
                                && text_point_is_finite(segment.bottom)
                        })
                })
            })
    }
}

fn build_static_caret_boundaries(
    text: &str,
    lines: &[StaticShapedTextLine],
    shape_world: Mat2D,
) -> (Vec<StaticCaretBoundary>, StaticCaretBuildWork) {
    let mut boundaries = text
        .char_indices()
        .map(|(byte_offset, _)| StaticCaretBoundary {
            byte_offset,
            upstream: None,
            downstream: None,
        })
        .chain(std::iter::once(StaticCaretBoundary {
            byte_offset: text.len(),
            upstream: None,
            downstream: None,
        }))
        .collect::<Vec<_>>();
    let mut work = StaticCaretBuildWork::default();
    for line in lines {
        line.write_caret_boundaries(shape_world, &mut boundaries, &mut work);
    }

    for (line_index, line) in lines.iter().enumerate() {
        let Some(skipped_start) = line.soft_wrap_skipped_start else {
            continue;
        };
        let Some(previous) = line_index
            .checked_sub(1)
            .and_then(|previous| lines.get(previous))
        else {
            continue;
        };
        let upstream = boundaries
            .get(previous.char_end)
            .and_then(|boundary| boundary.upstream);
        let downstream = boundaries
            .get(line.char_start)
            .and_then(|boundary| boundary.downstream);
        for char_index in skipped_start..=line.char_start {
            work.boundary_visits = work.boundary_visits.saturating_add(1);
            if let Some(boundary) = boundaries.get_mut(char_index) {
                boundary.upstream = upstream;
                boundary.downstream = downstream;
            }
        }
    }

    for line in lines {
        let Some(skipped_end) = line.terminal_soft_wrap_skipped_end else {
            continue;
        };
        let retained_end = boundaries
            .get(line.char_end)
            .and_then(|boundary| boundary.upstream.or(boundary.downstream));
        for char_index in line.char_end..=skipped_end {
            work.boundary_visits = work.boundary_visits.saturating_add(1);
            if let Some(boundary) = boundaries.get_mut(char_index) {
                boundary.upstream = retained_end;
                boundary.downstream = retained_end;
            }
        }
    }

    let glyph_count = lines
        .iter()
        .map(|line| line.glyphs.len())
        .fold(0usize, usize::saturating_add);
    let source_boundaries = text.chars().count().saturating_add(1);
    let linear_boundary_limit = source_boundaries
        .saturating_mul(3)
        .saturating_add(lines.len());
    debug_assert!(work.glyph_visits <= glyph_count);
    debug_assert!(work.boundary_visits <= linear_boundary_limit);
    (boundaries, work)
}

fn text_hit_candidate_wins_tie(
    candidate_byte: usize,
    candidate_affinity: StaticCaretAffinity,
    best_byte: usize,
    best_affinity: StaticCaretAffinity,
) -> bool {
    candidate_byte > best_byte
        || (candidate_byte == best_byte
            && candidate_affinity == StaticCaretAffinity::Downstream
            && best_affinity == StaticCaretAffinity::Upstream)
}

impl StaticShapedTextLine {
    fn geometry_is_finite(&self) -> bool {
        [
            self.start_x,
            self.end_x,
            self.top,
            self.baseline,
            self.bottom,
        ]
        .into_iter()
        .all(f32::is_finite)
            && self.glyphs.iter().all(|positioned| {
                positioned.x.is_finite()
                    && positioned.glyph.advance.is_finite()
                    && positioned.modifier_opacity.is_finite()
                    && positioned
                        .modifier_transform
                        .0
                        .iter()
                        .all(|value| value.is_finite())
            })
    }

    fn write_caret_boundaries(
        &self,
        shape_world: Mat2D,
        boundaries: &mut [StaticCaretBoundary],
        work: &mut StaticCaretBuildWork,
    ) {
        let clusters = self.positioned_clusters(work);
        let mut x_cluster = 0;
        let mut previous_cluster = None;
        let mut previous_cursor = 0;
        let mut next_cluster = 0;

        for char_index in self.char_start..=self.char_end {
            work.boundary_visits = work.boundary_visits.saturating_add(1);
            while clusters
                .get(x_cluster)
                .is_some_and(|cluster| cluster.char_end < char_index)
            {
                x_cluster = x_cluster.saturating_add(1);
            }
            while clusters
                .get(previous_cursor)
                .is_some_and(|cluster| cluster.char_end <= char_index)
            {
                previous_cluster = Some(previous_cursor);
                previous_cursor = previous_cursor.saturating_add(1);
            }
            while clusters
                .get(next_cluster)
                .is_some_and(|cluster| cluster.char_start < char_index)
            {
                next_cluster = next_cluster.saturating_add(1);
            }

            let current = clusters.get(x_cluster).copied();
            let containing = current
                .filter(|cluster| cluster.char_start < char_index && char_index < cluster.char_end);
            let x = if char_index <= self.char_start {
                self.start_x
            } else if let Some(cluster) = current {
                if char_index <= cluster.char_start {
                    cluster.start_x
                } else {
                    cluster.end_x
                }
            } else {
                self.end_x
            };

            let upstream_glyph = containing
                .map(|cluster| cluster.last_glyph)
                .or_else(|| {
                    previous_cluster
                        .and_then(|index| clusters.get(index))
                        .map(|cluster| cluster.last_glyph)
                })
                .or_else(|| clusters.first().map(|cluster| cluster.first_glyph));
            let downstream_glyph = containing
                .map(|cluster| cluster.last_glyph)
                .or_else(|| {
                    clusters
                        .get(next_cluster)
                        .map(|cluster| cluster.first_glyph)
                })
                .or_else(|| clusters.last().map(|cluster| cluster.last_glyph));
            let upstream = self.caret_segment(
                x,
                upstream_glyph.and_then(|index| self.glyphs.get(index)),
                shape_world,
            );
            let downstream = self.caret_segment(
                x,
                downstream_glyph.and_then(|index| self.glyphs.get(index)),
                shape_world,
            );

            if let Some(boundary) = boundaries.get_mut(char_index) {
                boundary.upstream.get_or_insert(upstream);
                boundary.downstream = Some(downstream);
            }
        }
    }

    fn positioned_clusters(
        &self,
        work: &mut StaticCaretBuildWork,
    ) -> Vec<StaticPositionedTextCluster> {
        let mut clusters = Vec::new();
        let mut glyph_index = 0;
        while let Some(first) = self.glyphs.get(glyph_index) {
            work.glyph_visits = work.glyph_visits.saturating_add(1);
            let char_start = first.glyph.char_index;
            let char_end = char_start.saturating_add(first.glyph.char_len);
            let first_glyph = glyph_index;
            let mut last_glyph = glyph_index;
            let mut end_x = first.x + first.glyph.advance;
            glyph_index = glyph_index.saturating_add(1);
            while let Some(next) = self.glyphs.get(glyph_index) {
                let next_end = next.glyph.char_index.saturating_add(next.glyph.char_len);
                if next.glyph.char_index != char_start || next_end != char_end {
                    break;
                }
                work.glyph_visits = work.glyph_visits.saturating_add(1);
                last_glyph = glyph_index;
                // A cluster's caret follows its final logical cursor. Its
                // visual extrema are retained separately by selection bounds.
                end_x = next.x + next.glyph.advance;
                glyph_index = glyph_index.saturating_add(1);
            }
            clusters.push(StaticPositionedTextCluster {
                char_start,
                char_end,
                start_x: first.x,
                end_x,
                first_glyph,
                last_glyph,
            });
        }
        clusters
    }

    fn caret_segment(
        &self,
        x: f32,
        glyph: Option<&StaticPositionedTextGlyph>,
        shape_world: Mat2D,
    ) -> StaticCaretSegment {
        let map = |y| {
            let (x, y) = glyph
                .map(|glyph| glyph.modified_point(x, y, self.baseline))
                .unwrap_or((x, y));
            let (x, y) = shape_world.transform_point(x, y);
            RenderVec2D::new(x, y)
        };
        StaticCaretSegment {
            top: map(self.top),
            bottom: map(self.bottom),
        }
    }

    fn caret_points(
        &self,
        char_index: usize,
        shape_world: Mat2D,
        affinity: StaticCaretAffinity,
    ) -> (RenderVec2D, RenderVec2D) {
        let x = self.caret_x(char_index);
        let glyph = self.caret_glyph(char_index, affinity);
        let segment = self.caret_segment(x, glyph, shape_world);
        (segment.top, segment.bottom)
    }

    fn selection_rect(
        &self,
        selected_start: usize,
        selected_end: usize,
        shape_world: Mat2D,
        has_geometric_modifiers: bool,
        has_non_monotone_advances: bool,
    ) -> Option<RenderAabb> {
        // A selected visual segment starts downstream and ends upstream.
        // Combining clusters and ligatures still remain indivisible because
        // both affinities snap internal source boundaries to the cluster end.
        let start_x = self.caret_x(selected_start);
        let end_x = self.caret_x(selected_end);
        if !has_geometric_modifiers && !has_non_monotone_advances {
            return (start_x != end_x).then(|| {
                transformed_text_rect(
                    shape_world,
                    start_x.min(end_x),
                    self.top,
                    start_x.max(end_x),
                    self.bottom,
                )
            });
        }

        let start_caret =
            self.caret_points(selected_start, shape_world, StaticCaretAffinity::Downstream);
        let end_caret = self.caret_points(selected_end, shape_world, StaticCaretAffinity::Upstream);
        let mut bounds = None;
        let mut has_selected_glyph_extent = false;
        extend_text_bounds(&mut bounds, start_caret.0);
        extend_text_bounds(&mut bounds, start_caret.1);
        extend_text_bounds(&mut bounds, end_caret.0);
        extend_text_bounds(&mut bounds, end_caret.1);
        for positioned in &self.glyphs {
            let glyph_start = positioned.glyph.char_index;
            if glyph_start < selected_start || glyph_start >= selected_end {
                continue;
            }
            let [top_left, top_right, bottom_right, bottom_left] = [
                (positioned.x, self.top),
                (positioned.x + positioned.glyph.advance, self.top),
                (positioned.x + positioned.glyph.advance, self.bottom),
                (positioned.x, self.bottom),
            ]
            .map(|(x, y)| {
                let (x, y) = positioned.modified_point(x, y, self.baseline);
                let (x, y) = shape_world.transform_point(x, y);
                RenderVec2D::new(x, y)
            });
            has_selected_glyph_extent |= top_left != top_right || bottom_right != bottom_left;
            for point in [top_left, top_right, bottom_right, bottom_left] {
                extend_text_bounds(&mut bounds, point);
            }
        }
        (start_caret != end_caret || has_selected_glyph_extent)
            .then_some(bounds)
            .flatten()
    }

    fn caret_glyph(
        &self,
        char_index: usize,
        affinity: StaticCaretAffinity,
    ) -> Option<&StaticPositionedTextGlyph> {
        if let Some(containing) = self.glyphs.iter().find(|positioned| {
            let start = positioned.glyph.char_index;
            let end = start.saturating_add(positioned.glyph.char_len);
            start < char_index && char_index < end
        }) {
            let start = containing.glyph.char_index;
            let end = start.saturating_add(containing.glyph.char_len);
            return self.glyphs.iter().rev().find(|positioned| {
                positioned.glyph.char_index == start
                    && positioned
                        .glyph
                        .char_index
                        .saturating_add(positioned.glyph.char_len)
                        == end
            });
        }
        match affinity {
            StaticCaretAffinity::Upstream => self
                .glyphs
                .iter()
                .rev()
                .find(|positioned| {
                    positioned
                        .glyph
                        .char_index
                        .saturating_add(positioned.glyph.char_len)
                        <= char_index
                })
                .or_else(|| self.glyphs.first()),
            StaticCaretAffinity::Downstream => self
                .glyphs
                .iter()
                .find(|positioned| positioned.glyph.char_index >= char_index)
                .or_else(|| self.glyphs.last()),
        }
    }

    fn caret_x(&self, char_index: usize) -> f32 {
        if char_index <= self.char_start {
            return self.start_x;
        }
        let mut glyphs = self.glyphs.iter().peekable();
        while let Some(positioned) = glyphs.next() {
            let glyph_start = positioned.glyph.char_index;
            let glyph_end = glyph_start.saturating_add(positioned.glyph.char_len);
            let cluster_x = positioned.x;
            let mut cluster_end_x = positioned.x + positioned.glyph.advance;
            while glyphs.peek().is_some_and(|next| {
                next.glyph.char_index == glyph_start
                    && next.glyph.char_index.saturating_add(next.glyph.char_len) == glyph_end
            }) {
                if let Some(next) = glyphs.next() {
                    // Preserve the final logical cursor even when glyphs in
                    // one cluster backtrack. Selection tracks visual bounds.
                    cluster_end_x = next.x + next.glyph.advance;
                }
            }
            if char_index < glyph_start {
                return cluster_x;
            }
            if char_index <= glyph_end && glyph_start < glyph_end {
                return if char_index == glyph_start {
                    cluster_x
                } else {
                    cluster_end_x
                };
            }
        }
        self.end_x
    }

    fn hit_char_index(&self, x: f32) -> usize {
        if x <= self.start_x {
            return self.char_start;
        }
        let mut glyphs = self.glyphs.iter().peekable();
        while let Some(positioned) = glyphs.next() {
            let glyph_start = positioned.glyph.char_index;
            let glyph_end = glyph_start.saturating_add(positioned.glyph.char_len);
            let cluster_x = positioned.x;
            let mut cluster_end_x = positioned.x + positioned.glyph.advance;
            while glyphs.peek().is_some_and(|next| {
                next.glyph.char_index == glyph_start
                    && next.glyph.char_index.saturating_add(next.glyph.char_len) == glyph_end
            }) {
                if let Some(next) = glyphs.next() {
                    cluster_end_x = cluster_end_x.max(next.x + next.glyph.advance);
                }
            }
            if x <= cluster_end_x {
                let midpoint = cluster_x + (cluster_end_x - cluster_x) / 2.0;
                return if x < midpoint { glyph_start } else { glyph_end };
            }
        }
        self.char_end
    }
}

impl StaticPositionedTextGlyph {
    fn modified_point(&self, x: f32, y: f32, baseline: f32) -> (f32, f32) {
        let center_x = self.x + self.glyph.advance * 0.5;
        let (x, y) = self
            .modifier_transform
            .transform_point(x - center_x, y - baseline);
        (center_x + x, baseline + y)
    }
}

fn extend_text_bounds(bounds: &mut Option<RenderAabb>, point: RenderVec2D) {
    *bounds = Some(match *bounds {
        Some(bounds) => RenderAabb::new(
            bounds.min_x.min(point.x),
            bounds.min_y.min(point.y),
            bounds.max_x.max(point.x),
            bounds.max_y.max(point.y),
        ),
        None => RenderAabb::new(point.x, point.y, point.x, point.y),
    });
}

fn text_point_is_finite(point: RenderVec2D) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn text_aabb_is_finite(bounds: RenderAabb) -> bool {
    [bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y]
        .into_iter()
        .all(f32::is_finite)
}

fn transformed_text_rect(
    transform: Mat2D,
    left: f32,
    top: f32,
    right: f32,
    bottom: f32,
) -> RenderAabb {
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (x, y) in [
        transform.transform_point(left, top),
        transform.transform_point(right, top),
        transform.transform_point(right, bottom),
        transform.transform_point(left, bottom),
    ] {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    RenderAabb::new(min_x, min_y, max_x, max_y)
}

fn point_segment_distance_squared(point: RenderVec2D, start: RenderVec2D, end: RenderVec2D) -> f32 {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length_squared = dx * dx + dy * dy;
    let t = if length_squared == 0.0 {
        0.0
    } else {
        (((point.x - start.x) * dx + (point.y - start.y) * dy) / length_squared).clamp(0.0, 1.0)
    };
    let nearest_x = start.x + dx * t;
    let nearest_y = start.y + dy * t;
    let distance_x = point.x - nearest_x;
    let distance_y = point.y - nearest_y;
    distance_x * distance_x + distance_y * distance_y
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StaticTextLayoutBoundsPurpose {
    Measure,
    Controlled,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeTextLayoutConstraint {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) width_scale_type: u64,
    pub(crate) height_scale_type: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct StaticTextClipBounds {
    pub(crate) bounds: (f32, f32, f32, f32),
    pub(crate) local_transform: Mat2D,
}

impl RuntimeTextLayoutConstraint {
    fn effective_sizing(self, authored_sizing: u64) -> u64 {
        // Ported from C++ `src/text/text.cpp` `Text::effectiveSizing`:
        // non-hug parent layout control sizes make text lay out as fixed.
        if self.width_scale_type == LAYOUT_SCALE_TYPE_HUG
            || self.height_scale_type == LAYOUT_SCALE_TYPE_HUG
        {
            authored_sizing
        } else {
            TEXT_SIZING_FIXED
        }
    }
}

#[derive(Debug, Clone)]
struct StaticTextModifierGroup {
    local_id: usize,
    global_id: u32,
    ranges: Vec<StaticTextModifierRange>,
    follow_path_modifiers: Vec<StaticTextFollowPathModifier>,
}

#[derive(Debug, Clone)]
struct StaticTextModifierRange {
    local_id: usize,
    global_id: u32,
    interpolator: Option<StaticCubicInterpolator>,
}

#[derive(Debug, Clone)]
struct StaticTextFollowPathModifier {
    local_id: usize,
    global_id: u32,
    paths: Vec<StaticTextFollowPathPath>,
}

#[derive(Debug, Clone)]
struct StaticTextFollowPathPath {
    local_id: usize,
    geometry: PathGeometryNode,
}

#[derive(Debug, Clone)]
struct StaticTextGlyphContext<'a> {
    origin_x: f32,
    origin_y: f32,
    line_index_in_paragraph: usize,
    paragraph_baselines: &'a [f32],
    text_world_inverse: Mat2D,
}

#[derive(Debug, Clone, Copy)]
struct StaticFollowPathGlyphTransform {
    x: f32,
    y: f32,
    rotation: f32,
}

#[derive(Debug, Clone, Copy)]
struct StaticRangeUnit {
    start: usize,
    len: usize,
}

#[derive(Debug, Clone)]
struct StaticTextPathBucket {
    opacity: f32,
    commands: Vec<RuntimePathCommand>,
}

#[derive(Debug, Clone, Copy)]
struct StaticCubicInterpolator {
    local_id: usize,
    global_id: u32,
}

fn static_text_data_bind_supported(data_bind: &DataBindNode) -> bool {
    if !data_bind_flags_apply_source_to_target(data_bind.flags) {
        return true;
    }
    let Ok(property_key) = u16::try_from(data_bind.property_key) else {
        return false;
    };
    match data_bind.target_type_name {
        Some("TextValueRun") => property_key_for_name("TextValueRun", "text") == Some(property_key),
        Some("SolidColor") => {
            property_key_for_name("SolidColor", "colorValue") == Some(property_key)
        }
        Some("Shape") => {
            (["x", "y"]
                .into_iter()
                .any(|name| property_key_for_name("Node", name) == Some(property_key))
                && (data_bind.converter_global.is_none()
                    || matches!(
                        data_bind.converter_type_name,
                        Some("DataConverterGroup" | "ScriptedDataConverter")
                    )))
                || (property_key_for_name("TransformComponent", "rotation") == Some(property_key)
                    && data_bind.converter_type_name == Some("DataConverterSystemDegsToRads"))
                || (["scaleX", "scaleY"].into_iter().any(|name| {
                    property_key_for_name("TransformComponent", name) == Some(property_key)
                }) && data_bind.converter_type_name == Some("DataConverterSystemNormalizer"))
        }
        Some("Node") => {
            (["x", "y"]
                .into_iter()
                .any(|name| property_key_for_name("Node", name) == Some(property_key))
                && data_bind.converter_global.is_none())
                || (property_key_for_name("TransformComponent", "rotation") == Some(property_key)
                    && data_bind.converter_type_name == Some("DataConverterGroup"))
        }
        Some("Artboard") => {
            (["x", "y"]
                .into_iter()
                .any(|name| property_key_for_name("Node", name) == Some(property_key))
                || property_key_for_name("Artboard", "clip") == Some(property_key))
                && data_bind.converter_global.is_none()
        }
        Some("Joystick") => {
            [joystick_x_property_key(), joystick_y_property_key()]
                .into_iter()
                .any(|key| key == Some(property_key))
                && (data_bind.converter_global.is_none()
                    || data_bind.converter_type_name == Some("DataConverterGroup"))
        }
        Some("Ellipse" | "Polygon" | "Rectangle" | "Star" | "Triangle") => {
            ["width", "height"]
                .into_iter()
                .any(|name| property_key_for_name("ParametricPath", name) == Some(property_key))
                && data_bind.converter_global.is_none()
        }
        Some("CubicMirroredVertex") => {
            property_key_for_name("CubicMirroredVertex", "distance") == Some(property_key)
                && data_bind.converter_global.is_none()
        }
        Some("LinearGradient") => {
            ["startX", "startY", "endX", "endY"]
                .into_iter()
                .any(|name| property_key_for_name("LinearGradient", name) == Some(property_key))
                && (data_bind.converter_global.is_none()
                    || matches!(
                        data_bind.converter_type_name,
                        Some("DataConverterOperationValue" | "DataConverterFormula")
                    ))
        }
        Some("NestedArtboard") => ["artboardId", "isPaused", "speed", "quantize"]
            .into_iter()
            .any(|name| property_key_for_name("NestedArtboard", name) == Some(property_key)),
        Some("ArtboardComponentList") => {
            property_key_for_name("ArtboardComponentList", "listSource") == Some(property_key)
        }
        Some("LayoutComponent") => {
            ["width", "height"]
                .into_iter()
                .any(|name| property_key_for_name("LayoutComponent", name) == Some(property_key))
                && (data_bind.converter_global.is_none()
                    || data_bind.converter_type_name == Some("DataConverterInterpolator"))
        }
        Some("LayoutComponentStyle") => {
            property_key_for_name("LayoutComponentStyle", "displayValue") == Some(property_key)
                && data_bind.converter_global.is_none()
        }
        Some(
            "ViewModelInstanceBoolean"
            | "ViewModelInstanceColor"
            | "ViewModelInstanceString"
            | "ViewModelInstanceEnum"
            | "ViewModelInstanceNumber",
        ) => {
            property_key_for_name(data_bind.target_type_name.unwrap_or(""), "propertyValue")
                == Some(property_key)
                && data_bind.converter_global.is_none()
        }
        Some("Solo") => property_key_for_name("Solo", "activeComponentId") == Some(property_key),
        Some("TextStylePaint") => {
            property_key_for_name("TextStyle", "fontSize") == Some(property_key)
        }
        Some("TrimPath") => {
            ["start", "end", "offset"]
                .into_iter()
                .any(|name| property_key_for_name("TrimPath", name) == Some(property_key))
                && (data_bind.converter_global.is_none()
                    || matches!(
                        data_bind.converter_type_name,
                        Some("DataConverterGroup" | "DataConverterRangeMapper")
                    ))
        }
        Some("FollowPathConstraint") => {
            property_key_for_name("FollowPathConstraint", "distance") == Some(property_key)
                && data_bind.converter_type_name == Some("DataConverterRangeMapper")
        }
        Some("Text") => {
            [
                "alignValue",
                "overflowValue",
                "verticalTrimTopValue",
                "verticalTrimBottomValue",
                "textRunListSource",
            ]
            .into_iter()
            .any(|name| property_key_for_name("Text", name) == Some(property_key))
                || (["width", "height"]
                    .into_iter()
                    .any(|name| property_key_for_name("Text", name) == Some(property_key))
                    && (data_bind.converter_global.is_none()
                        || data_bind.converter_type_name == Some("DataConverterFormula")))
        }
        Some("TextFollowPathModifier") => {
            ["start", "end", "strength", "offset"]
                .into_iter()
                .any(|name| {
                    property_key_for_name("TextFollowPathModifier", name) == Some(property_key)
                })
                && (data_bind.converter_global.is_none()
                    || data_bind.converter_type_name == Some("DataConverterFormula"))
                || (["radial", "orient"].into_iter().any(|name| {
                    property_key_for_name("TextFollowPathModifier", name) == Some(property_key)
                }) && data_bind.converter_global.is_none())
        }
        _ => false,
    }
}

fn static_text_data_bind_property_name(data_bind: &DataBindNode) -> Option<&'static str> {
    let target_type_name = data_bind.target_type_name?;
    let property_key = u16::try_from(data_bind.property_key).ok()?;
    definition_by_name(target_type_name)?
        .property_by_key_in_hierarchy(property_key)
        .map(|property| property.name)
}

fn static_text_data_bind_target_label(data_bind: &DataBindNode) -> String {
    let target_type_name = data_bind.target_type_name.unwrap_or("unknown");
    match static_text_data_bind_property_name(data_bind) {
        Some(property_name) => format!("{target_type_name}.{property_name}"),
        None => format!("{target_type_name} property {}", data_bind.property_key),
    }
}

impl StaticTextLayoutInfo {
    fn line_start_x(self, line_width: f32) -> f32 {
        match self.align_value {
            1 => self.paragraph_width - line_width,
            2 => self.paragraph_width / 2.0 - line_width / 2.0,
            _ => 0.0,
        }
    }
}

impl<'a> StaticTextSlice<'a> {
    fn text_geometry_supported(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<bool> {
        let overflow = self.text_uint_property(runtime, instance, "overflowValue")?;
        Ok(matches!(
            overflow,
            TEXT_OVERFLOW_VISIBLE | TEXT_OVERFLOW_FIT | TEXT_OVERFLOW_FIT_FONT_SIZE
        ))
    }

    fn text_geometry_inputs_supported(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text_world: Mat2D,
        resolved_runs: &[StaticResolvedRun],
    ) -> Result<bool> {
        if !text_world.0.iter().all(|value| value.is_finite())
            || layout_constraint.is_some_and(|constraint| {
                !constraint.width.is_finite() || !constraint.height.is_finite()
            })
            || !self
                .effective_width(runtime, instance, layout_constraint)?
                .is_finite()
            || !self
                .effective_height(runtime, instance, layout_constraint)?
                .is_finite()
        {
            return Ok(false);
        }

        let base_style = self.base_style()?;
        let mut participating_style_locals = BTreeSet::from([base_style.local_id]);
        for run in resolved_runs.iter().filter(|run| !run.text.is_empty()) {
            participating_style_locals.insert(run.style_local);
        }
        for style in self
            .styles
            .iter()
            .filter(|style| participating_style_locals.remove(&style.local_id))
        {
            if style.font_bytes(instance).is_none()
                || !self.style_font_size(runtime, instance, style)?.is_finite()
                || !self
                    .style_line_height(runtime, instance, style)?
                    .is_finite()
                || !self
                    .style_letter_spacing(runtime, instance, style)
                    .is_finite()
                || !style.variations.iter().all(|(_, value)| value.is_finite())
            {
                return Ok(false);
            }
        }
        Ok(participating_style_locals.is_empty())
    }

    fn from_graph(
        runtime: &'a RuntimeFile,
        graph: &'a ArtboardGraph,
        text_local: usize,
    ) -> Result<Self> {
        let text_object = graph
            .local_objects
            .iter()
            .find(|object| object.local_id == text_local)
            .context("text local object is missing")?;
        if text_object.type_name != Some("Text") {
            bail!("static text subset expected Text local {text_local}");
        }
        if let Some(data_bind) = graph
            .data_binds
            .iter()
            .find(|data_bind| !static_text_data_bind_supported(data_bind))
        {
            bail!(
                "static text subset does not support data binding target {} global {}",
                static_text_data_bind_target_label(data_bind),
                data_bind.target_global.unwrap_or(0)
            );
        }
        if let Some(object) = graph.local_objects.iter().find(|object| {
            object.type_name.is_some_and(|type_name| {
                !matches!(
                    type_name,
                    "Artboard"
                        | "Node"
                        | "NSlicedNode"
                        | "AxisX"
                        | "AxisY"
                        | "Text"
                        | "TextValueRun"
                        | "TextStylePaint"
                        | "TextStyleAxis"
                        | "TextModifierGroup"
                        | "TextModifierRange"
                        | "Solo"
                        | "CubicInterpolatorComponent"
                        | "Shape"
                        | "Image"
                        | "PointsPath"
                        | "StraightVertex"
                        | "CubicDetachedVertex"
                        | "CubicAsymmetricVertex"
                        | "CubicMirroredVertex"
                        | "Triangle"
                        | "Ellipse"
                        | "Rectangle"
                        | "Star"
                        | "ClippingShape"
                        | "DrawTarget"
                        | "DrawRules"
                        | "SolidColor"
                        | "LinearGradient"
                        | "RadialGradient"
                        | "GradientStop"
                        | "Fill"
                        | "Stroke"
                        | "TrimPath"
                        | "Backboard"
                        | "NestedArtboard"
                        | "NestedArtboardLayout"
                        | "NestedArtboardLeaf"
                        | "NestedSimpleAnimation"
                        | "NestedStateMachine"
                        | "NestedRemapAnimation"
                        | "Joystick"
                        | "NestedBool"
                        | "NestedNumber"
                        | "ArtboardComponentList"
                        | "RootBone"
                        | "Skin"
                        | "Tendon"
                        | "Weight"
                        | "CubicWeight"
                        | "KeyedObject"
                        | "KeyedProperty"
                        | "LinearAnimation"
                        | "CubicEaseInterpolator"
                        | "ElasticInterpolator"
                        | "DashPath"
                        | "Dash"
                        | "KeyFrameColor"
                        | "KeyFrameBool"
                        | "TransformConstraint"
                        | "TranslationConstraint"
                        | "FollowPathConstraint"
                        | "ScrollConstraint"
                        | "LayoutComponent"
                        | "LayoutComponentStyle"
                        | "ForegroundLayoutDrawable"
                        | "FocusData"
                        | "KeyboardInput"
                        | "GamepadInput"
                        | "ListenerInputTypeGamepad"
                        | "ScriptedDrawable"
                        | "ScriptInputArtboard"
                        | "ScriptInputViewModelProperty"
                        | "TextFollowPathModifier"
                        | "Feather"
                        | "CustomPropertyGroup"
                        | "CustomPropertyNumber"
                        | "CustomPropertyBoolean"
                        | "CustomPropertyString"
                        | "CustomPropertyColor"
                        | "CustomPropertyEnum"
                        | "CustomPropertyTrigger"
                        | "ViewModel"
                        | "ViewModelInstance"
                        | "ViewModelInstanceNumber"
                        | "ViewModelInstanceString"
                        | "ViewModelInstanceList"
                        | "ViewModelInstanceListItem"
                        | "ViewModelInstanceTrigger"
                        | "ViewModelInstanceViewModel"
                        | "ViewModelPropertyList"
                        | "ViewModelPropertyNumber"
                        | "ViewModelPropertyString"
                        | "ViewModelPropertyTrigger"
                        | "ViewModelPropertyViewModel"
                        | "Event"
                        | "StateMachine"
                        | "StateMachineLayer"
                        | "StateMachineBool"
                        | "ListenerBoolChange"
                        | "ListenerViewModelChange"
                        | "StateMachineListenerSingle"
                        | "BindablePropertyTrigger"
                        | "DataConverterTrigger"
                        | "AnimationState"
                        | "AnyState"
                        | "EntryState"
                        | "ExitState"
                        | "StateTransition"
                        | "TransitionBoolCondition"
                )
            })
        }) {
            bail!(
                "static text subset does not support sibling {} global {}",
                object.type_name.unwrap_or("unknown"),
                object.global_id
            );
        }
        if let Some(object) = graph.local_objects.iter().find(|object| {
            matches!(
                object.type_name,
                Some(
                    "TextInput"
                        | "TextInputDrawable"
                        | "TextInputCursor"
                        | "TextInputText"
                        | "TextInputSelection"
                        | "TextInputSelectedText"
                        | "TextStyleFeature"
                        | "TextModifier"
                        | "TextShapeModifier"
                        | "TextVariationModifier"
                        | "TextTargetModifier"
                )
            )
        }) {
            bail!(
                "static text subset does not support {} global {}",
                object.type_name.unwrap_or("unknown"),
                object.global_id
            );
        }

        let text_component = graph
            .components
            .iter()
            .find(|component| component.local_id == text_local)
            .context("text component is missing")?;
        if !static_text_parent_chain_supported(graph, text_component.parent_local) {
            bail!(
                "static text subset only supports top-level Text or Text under Node/Shape/LayoutComponent transforms"
            );
        }

        let child_type = |local_id| {
            graph
                .local_objects
                .iter()
                .find(|object| object.local_id == local_id)
                .and_then(|object| object.type_name)
        };
        let run_locals = text_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextValueRun"))
            .collect::<Vec<_>>();
        let style_locals = text_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextStylePaint"))
            .collect::<Vec<_>>();
        let has_text_run_list_source = graph.data_binds.iter().any(|data_bind| {
            data_bind.target_local == Some(text_local)
                && data_bind.target_type_name == Some("Text")
                && u16::try_from(data_bind.property_key).ok()
                    == property_key_for_name("Text", "textRunListSource")
                && data_bind_flags_apply_source_to_target(data_bind.flags)
        });
        if run_locals.is_empty() && !has_text_run_list_source {
            bail!(
                "static text subset requires at least one TextValueRun child, found {}",
                run_locals.len()
            );
        }
        if style_locals.is_empty() {
            bail!(
                "static text subset requires at least one TextStylePaint child, found {}",
                style_locals.len()
            );
        }
        let text_global = global_for_local(graph, text_local)?;
        let mut runs = Vec::new();
        for run_local in run_locals {
            let run_global = global_for_local(graph, run_local)?;
            let run = runtime
                .object(run_global as usize)
                .with_context(|| format!("missing TextValueRun global {run_global}"))?;
            let style_local = run
                .uint_property("styleId")
                .context("TextValueRun missing styleId")? as usize;
            if !style_locals.contains(&style_local) {
                bail!(
                    "static text subset requires every TextValueRun to reference a TextStylePaint child"
                );
            }
            let bytes = run
                .string_property_bytes("text")
                .context("static text subset requires serialized TextValueRun.text")?;
            std::str::from_utf8(bytes).context("TextValueRun text is not UTF-8")?;
            runs.push(StaticTextRun {
                local_id: run_local,
                global_id: run_global,
                style_local,
                text_property_owner: "TextValueRun",
            });
        }
        let mut styles = Vec::new();
        for style_local in style_locals {
            styles.push(StaticTextStyle::from_graph(runtime, graph, style_local)?);
        }
        let modifiers = text_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextModifierGroup"))
            .map(|group_local| StaticTextModifierGroup::from_graph(runtime, graph, group_local))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            kind: StaticTextKind::Text,
            text_local,
            text_global,
            runs,
            styles,
            modifiers,
        })
    }

    fn from_text_input_graph(
        runtime: &'a RuntimeFile,
        graph: &'a ArtboardGraph,
        text_input_local: usize,
    ) -> Result<Self> {
        let text_input_object = graph
            .local_objects
            .iter()
            .find(|object| object.local_id == text_input_local)
            .context("TextInput local object is missing")?;
        if text_input_object.type_name != Some("TextInput") {
            bail!("TextInput subset expected TextInput local {text_input_local}");
        }

        let text_input_component = component_for_local(graph, text_input_local)
            .with_context(|| format!("TextInput local {text_input_local} component is missing"))?;
        let child_type = |local_id| type_for_local(graph, local_id);
        let style_local = text_input_component
            .children
            .iter()
            .copied()
            .find(|local| child_type(*local) == Some("TextStyle"))
            .context("TextInput subset requires one TextStyle child")?;
        if text_input_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextStyle"))
            .count()
            != 1
        {
            bail!("TextInput subset currently supports exactly one TextStyle child");
        }
        if text_input_component.children.iter().copied().any(|local| {
            matches!(
                child_type(local),
                Some("TextStyleFeature" | "TextInputSelectedText")
            )
        }) {
            bail!("TextInput subset does not support style features or selected-text draws");
        }

        let text_input_global = global_for_local(graph, text_input_local)?;
        let style = StaticTextStyle::from_graph(runtime, graph, style_local)?;
        Ok(Self {
            kind: StaticTextKind::TextInput,
            text_local: text_input_local,
            text_global: text_input_global,
            runs: vec![StaticTextRun {
                local_id: text_input_local,
                global_id: text_input_global,
                style_local,
                text_property_owner: "TextInput",
            }],
            styles: vec![style],
            modifiers: Vec::new(),
        })
    }

    fn shaped_layout(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text_world: Mat2D,
    ) -> Result<Option<StaticShapedTextLayout>> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        self.shaped_layout_from_resolved_runs(
            runtime,
            instance,
            layout_constraint,
            text_world,
            &resolved_runs,
            StaticShapedTextPurpose::Geometry,
        )
    }

    fn shaped_layout_from_resolved_runs(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text_world: Mat2D,
        resolved_runs: &[StaticResolvedRun],
        purpose: StaticShapedTextPurpose,
    ) -> Result<Option<StaticShapedTextLayout>> {
        if purpose == StaticShapedTextPurpose::Geometry
            && !self.text_geometry_inputs_supported(
                runtime,
                instance,
                layout_constraint,
                text_world,
                resolved_runs,
            )?
        {
            return Ok(None);
        }
        let text = resolved_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let base_style = self.base_style()?;
        let font_size = self.style_font_size(runtime, instance, base_style)?;
        if font_size < 0.0 {
            return Ok(None);
        }
        let letter_spacing = self.letter_spacing(runtime, instance);
        let Some(font_bytes) = base_style.font_bytes(instance) else {
            // Mirrors src/importers/file_asset_importer.cpp: with no
            // FileAssetLoader and no in-band contents, a hosted FontAsset
            // resolves successfully but has no decoded font.
            return Ok(None);
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self
            .base_style()?
            .variations
            .iter()
            .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let shaper_instance = if harf_variations.is_empty() {
            None
        } else {
            Some(ShaperInstance::from_variations(
                &harf_font,
                harf_variations.iter().copied(),
            ))
        };
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data
            .shaper(&harf_font)
            .instance(shaper_instance.as_ref())
            .build();

        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for outlines")?;
        let disable_legacy_kern = disable_legacy_kern_for_advances(&skrifa_font);
        let font_scale = self.fit_font_scale(
            runtime,
            instance,
            layout_constraint,
            resolved_runs,
            &text,
            &shaper,
            disable_legacy_kern,
        )?;
        let scaled_font_size = font_size * font_scale;
        let scale = scaled_font_size / TEXT_SHAPE_SCALE_F32;
        let apply_ellipsis =
            self.should_apply_static_ellipsis(runtime, instance, layout_constraint)?;
        let lines = self.layout_static_text_lines(
            runtime,
            instance,
            layout_constraint,
            &text,
            &shaper,
            disable_legacy_kern,
            scale,
            letter_spacing,
        )?;
        let line_metrics =
            self.static_line_metrics(runtime, instance, &lines, resolved_runs, font_scale)?;
        let line_widths = lines
            .iter()
            .map(|line| self.styled_line_width(runtime, instance, line, resolved_runs, font_scale))
            .collect::<Result<Vec<_>>>()?;
        let measured_width = line_widths.iter().copied().fold(0.0f32, f32::max);
        let layout_info = self.static_layout_info(
            runtime,
            instance,
            &lines,
            &line_metrics,
            resolved_runs,
            measured_width,
            font_scale,
            apply_ellipsis,
            layout_constraint,
        )?;
        let minimum_line_x = line_widths
            .iter()
            .copied()
            .map(|width| layout_info.line_start_x(width))
            .fold(f32::INFINITY, f32::min);
        let total_height = line_metrics
            .last()
            .map(|metrics| metrics.bottom + layout_info.min_y)
            .unwrap_or(layout_info.min_y);
        let first_baseline = line_metrics
            .first()
            .map(|metrics| metrics.baseline)
            .unwrap_or(0.0);
        let local_transform = self.static_render_transform(
            runtime,
            instance,
            layout_constraint,
            layout_info,
            measured_width,
            if minimum_line_x.is_finite() {
                minimum_line_x
            } else {
                0.0
            },
            total_height,
            first_baseline,
        )?;
        let paragraph_baselines = line_metrics
            .iter()
            .map(|metrics| metrics.baseline + layout_info.min_y - layout_info.top_trim)
            .collect::<Vec<_>>();
        let overflow = self.text_uint_property(runtime, instance, "overflowValue")?;
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        let vertical_align = self.text_uint_property(runtime, instance, "verticalAlignValue")?;
        let fixed_height = self.effective_height(runtime, instance, layout_constraint)?;
        let modifier_coverages = self
            .modifiers
            .iter()
            .map(|modifier| {
                modifier.coverage_by_character(runtime, instance, &text, resolved_runs, &lines)
            })
            .collect::<Result<Vec<_>>>()?;
        let mut shaped_lines = Vec::new();
        for (line, metrics) in lines.into_iter().zip(line_metrics.iter().copied()) {
            let char_end = line
                .char_start
                .checked_add(line.text.chars().count())
                .context("static Text line character range overflow")?;
            if let Some(ellipsis_line) = layout_info.ellipsis_line {
                if line.line_index > ellipsis_line {
                    break;
                }
            }
            match static_text_line_iteration(
                overflow,
                sizing,
                vertical_align,
                metrics,
                layout_info.min_y - layout_info.top_trim,
                total_height,
                fixed_height,
            ) {
                StaticTextLineIteration::Draw => {}
                StaticTextLineIteration::Skip => continue,
                StaticTextLineIteration::Stop => break,
            }
            let mut glyphs =
                self.styled_line_glyphs(runtime, instance, &line, resolved_runs, font_scale)?;
            if layout_info.ellipsis_line == Some(line.line_index) {
                let max_width = self.effective_width(runtime, instance, layout_constraint)?;
                let ellipsis_style = glyphs.last().map(|glyph| glyph.style_index).unwrap_or(0);
                let ellipsis = self.styled_text_glyphs_for_style(
                    runtime,
                    instance,
                    "...",
                    char_end,
                    ellipsis_style,
                    font_scale,
                )?;
                let base_glyphs = shape_text_glyphs(&shaper, line.text, disable_legacy_kern);
                let line_end = self.first_static_wrapped_line_end(
                    runtime,
                    instance,
                    line.text,
                    &base_glyphs,
                    max_width,
                    scale,
                    letter_spacing,
                )?;
                let mut force_ellipsis = !layout_info.is_ellipsis_line_last;
                if line_end < base_glyphs.len()
                    && self.static_fixed_height_shows_first_line_only(
                        runtime,
                        instance,
                        layout_constraint,
                        &line_metrics,
                    )?
                {
                    glyphs.truncate(line_end);
                    force_ellipsis = true;
                }
                apply_static_ellipsis(&mut glyphs, ellipsis, max_width, force_ellipsis);
            }

            let line_width = glyphs.iter().map(|glyph| glyph.advance).sum();
            let mut cursor_x = layout_info.line_start_x(line_width);
            let line_baseline = metrics.baseline + layout_info.min_y - layout_info.top_trim;
            let start_x = cursor_x;
            let positioned_glyphs = glyphs
                .into_iter()
                .map(|glyph| {
                    let positioned = StaticPositionedTextGlyph {
                        glyph,
                        x: cursor_x,
                        modifier_transform: Mat2D::IDENTITY,
                        modifier_opacity: 1.0,
                    };
                    cursor_x += positioned.glyph.advance;
                    positioned
                })
                .collect();
            shaped_lines.push(StaticShapedTextLine {
                line_index: line.line_index,
                char_start: line.char_start,
                char_end,
                soft_wrap_skipped_start: line.soft_wrap_skipped_start,
                terminal_soft_wrap_skipped_end: line.terminal_soft_wrap_skipped_end,
                start_x,
                end_x: cursor_x,
                top: metrics.top + layout_info.min_y - layout_info.top_trim,
                baseline: line_baseline,
                bottom: metrics.bottom + layout_info.min_y - layout_info.top_trim,
                glyphs: positioned_glyphs,
            });
        }

        let text_world_inverse = text_world.invert_or_identity();
        let mut has_geometric_modifiers = false;
        let mut has_non_monotone_advances = false;
        for line in &mut shaped_lines {
            for positioned in &mut line.glyphs {
                let glyph = &positioned.glyph;
                let glyph_context = StaticTextGlyphContext {
                    origin_x: positioned.x + glyph.advance * 0.5,
                    origin_y: line.baseline,
                    line_index_in_paragraph: line.line_index,
                    paragraph_baselines: &paragraph_baselines,
                    text_world_inverse,
                };
                for (modifier, coverage) in self.modifiers.iter().zip(&modifier_coverages) {
                    let amount = glyph_coverage(coverage, glyph.char_index, glyph.char_len);
                    if amount != 0.0 {
                        positioned.modifier_transform = modifier
                            .transform(runtime, instance, amount, &glyph_context)?
                            .multiply(positioned.modifier_transform);
                    }
                    if modifier.modifies_opacity(runtime, instance)? {
                        positioned.modifier_opacity = modifier.opacity(
                            runtime,
                            instance,
                            positioned.modifier_opacity,
                            amount,
                        )?;
                    }
                }
                has_geometric_modifiers |= positioned.modifier_transform != Mat2D::IDENTITY;
                has_non_monotone_advances |= positioned.glyph.advance < 0.0;
            }
        }

        let shape_world = text_world.multiply(local_transform);
        let caret_boundaries = (purpose == StaticShapedTextPurpose::Geometry).then(|| {
            let (boundaries, _caret_build_work) =
                build_static_caret_boundaries(&text, &shaped_lines, shape_world);
            boundaries
        });
        Ok(Some(StaticShapedTextLayout {
            text,
            lines: shaped_lines,
            caret_boundaries,
            local_transform,
            shape_world,
            has_geometric_modifiers,
            has_non_monotone_advances,
        }))
    }

    fn render_data(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text_world: Mat2D,
    ) -> Result<StaticTextRenderData> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        if resolved_runs.iter().all(|run| run.text.is_empty()) {
            return Ok(StaticTextRenderData {
                path_buckets_by_style: vec![Vec::new(); self.styles.len()],
                local_transform: Mat2D::IDENTITY,
            });
        }
        let Some(layout) = self.shaped_layout_from_resolved_runs(
            runtime,
            instance,
            layout_constraint,
            text_world,
            &resolved_runs,
            StaticShapedTextPurpose::Render,
        )?
        else {
            return Ok(StaticTextRenderData {
                path_buckets_by_style: vec![Vec::new(); self.styles.len()],
                local_transform: Mat2D::IDENTITY,
            });
        };
        let mut commands_by_style = vec![Vec::new(); self.styles.len()];
        for line in &layout.lines {
            for positioned in &line.glyphs {
                let glyph = &positioned.glyph;
                let center_x = positioned.x + glyph.advance * 0.5;
                let glyph_id = GlyphId::new(glyph.glyph_id);
                let style = &self.styles[glyph.style_index];
                if let Some(style_font_bytes) = style.font_bytes(instance) {
                    let style_font = SkrifaFontRef::new(style_font_bytes)
                        .context("failed to parse font for outlines")?;
                    let outlines = style_font.outline_glyphs();
                    let skrifa_variations = style
                        .variations
                        .iter()
                        .map(|(tag, value)| {
                            VariationSetting::new(SkrifaTag::from_u32(*tag), *value)
                        })
                        .collect::<Vec<_>>();
                    let location = style_font
                        .axes()
                        .location(skrifa_variations.iter().copied());
                    let location_ref = LocationRef::from(&location);
                    if let Some(outline) = outlines.get(glyph_id) {
                        let mut pen = TextOutlinePen::new(
                            positioned.x,
                            line.baseline,
                            glyph.scale,
                            center_x,
                            line.baseline,
                            positioned.modifier_transform,
                        );
                        // C++ `src/text/font_hb.cpp` records static glyf contours
                        // at the font's authored start points; Skrifa's
                        // HarfBuzz-style conversion can rotate those starts.
                        let path_style = if style_font.axes().is_empty() {
                            PathStyle::FreeType
                        } else {
                            PathStyle::HarfBuzz
                        };
                        let draw_settings =
                            DrawSettings::unhinted(Size::new(TEXT_SHAPE_SCALE_F32), location_ref)
                                .with_path_style(path_style);
                        outline
                            .draw(draw_settings, &mut pen)
                            .with_context(|| format!("failed to draw glyph {}", glyph.glyph_id))?;
                        append_opacity_bucket(
                            &mut commands_by_style[glyph.style_index],
                            positioned.modifier_opacity,
                            pen.commands,
                        );
                    }
                }
            }
        }

        Ok(StaticTextRenderData {
            path_buckets_by_style: commands_by_style,
            local_transform: layout.local_transform,
        })
    }

    fn local_bounds(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        let text = resolved_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let base_style = self.base_style()?;
        let font_size = self.style_font_size(runtime, instance, base_style)?;
        if text.is_empty() || font_size < 0.0 {
            return self.unshaped_local_bounds(runtime, instance, None);
        }
        let Some(font_bytes) = base_style.font_bytes(instance) else {
            return self.unshaped_local_bounds(runtime, instance, None);
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self
            .base_style()?
            .variations
            .iter()
            .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let shaper_instance = if harf_variations.is_empty() {
            None
        } else {
            Some(ShaperInstance::from_variations(
                &harf_font,
                harf_variations.iter().copied(),
            ))
        };
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data
            .shaper(&harf_font)
            .instance(shaper_instance.as_ref())
            .build();

        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for metrics")?;
        let disable_legacy_kern = disable_legacy_kern_for_advances(&skrifa_font);
        let lines = split_static_text_lines(&text);
        let font_scale = self.fit_font_scale(
            runtime,
            instance,
            None,
            &resolved_runs,
            &text,
            &shaper,
            disable_legacy_kern,
        )?;
        let scaled_font_size = font_size * font_scale;
        let line_metrics =
            self.static_line_metrics(runtime, instance, &lines, &resolved_runs, font_scale)?;
        let scale = scaled_font_size / TEXT_SHAPE_SCALE_F32;
        let letter_spacing = self.letter_spacing(runtime, instance);
        let measured_width = lines
            .iter()
            .filter(|line| !line.text.is_empty())
            .map(|line| {
                let glyphs = shape_text_glyphs(&shaper, line.text, disable_legacy_kern);
                text_glyph_width(&glyphs, scale, letter_spacing)
            })
            .fold(0.0f32, f32::max);
        let sizing = self.effective_sizing(runtime, instance, None)?;
        let width = match sizing {
            1 | 2 => self.text_width(runtime, instance)?,
            _ => measured_width,
        };
        let min_y = self.static_text_min_y(runtime, instance, &line_metrics)?;
        let measured_bottom = line_metrics
            .last()
            .map(|metrics| min_y + metrics.bottom)
            .unwrap_or(min_y);
        let height = match sizing {
            TEXT_SIZING_FIXED => self.text_height(runtime, instance)?,
            _ => {
                let trim = self.static_vertical_trim(
                    runtime,
                    instance,
                    &lines,
                    &line_metrics,
                    &resolved_runs,
                    font_scale,
                )?;
                (measured_bottom - trim.top - trim.bottom).max(min_y) - min_y
            }
        };
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;

        Ok(Some((
            -width * origin_x,
            min_y - height * origin_y,
            width,
            height,
        )))
    }

    fn local_bounds_with_layout_constraint(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: RuntimeTextLayoutConstraint,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        self.layout_bounds_with_constraint(
            runtime,
            instance,
            layout_constraint,
            StaticTextLayoutBoundsPurpose::Controlled,
        )
    }

    fn measure_bounds_with_layout_constraint(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: RuntimeTextLayoutConstraint,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        self.layout_bounds_with_constraint(
            runtime,
            instance,
            layout_constraint,
            StaticTextLayoutBoundsPurpose::Measure,
        )
    }

    fn layout_bounds_with_constraint(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: RuntimeTextLayoutConstraint,
        purpose: StaticTextLayoutBoundsPurpose,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        let text = resolved_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let base_style = self.base_style()?;
        let font_size = self.style_font_size(runtime, instance, base_style)?;
        if text.is_empty() || font_size < 0.0 {
            return match purpose {
                StaticTextLayoutBoundsPurpose::Measure => Ok(Some((0.0, 0.0, 0.0, 0.0))),
                StaticTextLayoutBoundsPurpose::Controlled => {
                    self.unshaped_local_bounds(runtime, instance, Some(layout_constraint))
                }
            };
        }
        let Some(font_bytes) = base_style.font_bytes(instance) else {
            return match purpose {
                StaticTextLayoutBoundsPurpose::Measure => Ok(Some((0.0, 0.0, 0.0, 0.0))),
                StaticTextLayoutBoundsPurpose::Controlled => {
                    self.unshaped_local_bounds(runtime, instance, Some(layout_constraint))
                }
            };
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self
            .base_style()?
            .variations
            .iter()
            .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let shaper_instance = if harf_variations.is_empty() {
            None
        } else {
            Some(ShaperInstance::from_variations(
                &harf_font,
                harf_variations.iter().copied(),
            ))
        };
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data
            .shaper(&harf_font)
            .instance(shaper_instance.as_ref())
            .build();

        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for metrics")?;
        let disable_legacy_kern = disable_legacy_kern_for_advances(&skrifa_font);
        let font_scale = self.fit_font_scale(
            runtime,
            instance,
            Some(layout_constraint),
            &resolved_runs,
            &text,
            &shaper,
            disable_legacy_kern,
        )?;
        let scaled_font_size = font_size * font_scale;
        let scale = scaled_font_size / TEXT_SHAPE_SCALE_F32;
        let letter_spacing = self.letter_spacing(runtime, instance);
        let lines = self.layout_static_text_lines(
            runtime,
            instance,
            Some(layout_constraint),
            &text,
            &shaper,
            disable_legacy_kern,
            scale,
            letter_spacing,
        )?;
        let line_metrics =
            self.static_line_metrics(runtime, instance, &lines, &resolved_runs, font_scale)?;
        let measured_width = lines
            .iter()
            .map(|line| self.styled_line_width(runtime, instance, line, &resolved_runs, font_scale))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold(0.0f32, f32::max);
        let sizing = match purpose {
            StaticTextLayoutBoundsPurpose::Measure => self.authored_sizing(runtime, instance)?,
            StaticTextLayoutBoundsPurpose::Controlled => {
                self.effective_sizing(runtime, instance, Some(layout_constraint))?
            }
        };
        let width = match (purpose, sizing) {
            (
                StaticTextLayoutBoundsPurpose::Measure,
                TEXT_SIZING_AUTO_HEIGHT | TEXT_SIZING_FIXED,
            ) => self
                .text_width(runtime, instance)?
                .min(layout_constraint.width),
            (StaticTextLayoutBoundsPurpose::Measure, _) => {
                measured_width.min(layout_constraint.width)
            }
            (
                StaticTextLayoutBoundsPurpose::Controlled,
                TEXT_SIZING_AUTO_HEIGHT | TEXT_SIZING_FIXED,
            ) => self.effective_width(runtime, instance, Some(layout_constraint))?,
            (StaticTextLayoutBoundsPurpose::Controlled, _) => measured_width,
        };
        let min_y = self.static_text_min_y(runtime, instance, &line_metrics)?;
        let measured_bottom = line_metrics
            .last()
            .map(|metrics| min_y + metrics.bottom)
            .unwrap_or(min_y);
        let measured_height = || -> Result<f32> {
            let trim = self.static_vertical_trim(
                runtime,
                instance,
                &lines,
                &line_metrics,
                &resolved_runs,
                font_scale,
            )?;
            Ok((measured_bottom - trim.top - trim.bottom).max(min_y) - min_y)
        };
        let height = match (purpose, sizing) {
            (StaticTextLayoutBoundsPurpose::Measure, TEXT_SIZING_FIXED) => self
                .text_height(runtime, instance)?
                .min(layout_constraint.height),
            (StaticTextLayoutBoundsPurpose::Measure, _) => {
                measured_height()?.min(layout_constraint.height)
            }
            (StaticTextLayoutBoundsPurpose::Controlled, TEXT_SIZING_FIXED) => {
                self.effective_height(runtime, instance, Some(layout_constraint))?
            }
            (StaticTextLayoutBoundsPurpose::Controlled, _) => measured_height()?,
        };
        if purpose == StaticTextLayoutBoundsPurpose::Measure {
            return Ok(Some((0.0, 0.0, width, height)));
        }
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;
        Ok(Some((
            -width * origin_x,
            min_y - height * origin_y,
            width,
            height,
        )))
    }

    fn resolved_runs(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<Vec<StaticResolvedRun>> {
        let mut runs = Vec::new();
        let mut char_start = 0;
        for run in &self.runs {
            let property_key = property_key_for_name(run.text_property_owner, "text")
                .with_context(|| format!("missing {}.text key", run.text_property_owner))?;
            let bytes = instance
                .string_property(run.local_id, property_key)
                .or_else(|| {
                    runtime
                        .object(run.global_id as usize)
                        .and_then(|object| object.string_property_bytes("text"))
                })
                .context("TextValueRun missing text")?;
            let text = std::str::from_utf8(bytes)
                .with_context(|| format!("{}.text is not UTF-8", run.text_property_owner))?
                .to_owned();
            let char_len = text.chars().count();
            runs.push(StaticResolvedRun {
                local_id: run.local_id,
                global_id: run.global_id,
                style_local: run.style_local,
                char_start,
                char_len,
                text,
            });
            char_start += char_len;
        }
        for (text, style_name) in instance.text_list_runs(self.text_local) {
            let text = String::from_utf8(text).context("dynamic TextValueRun text is not UTF-8")?;
            let style_name = std::str::from_utf8(&style_name).unwrap_or_default();
            let style_local = self
                .styles
                .iter()
                .find(|style| style.name.as_deref() == Some(style_name))
                .or_else(|| self.styles.first())
                .context("dynamic TextValueRun requires a TextStylePaint")?
                .local_id;
            let char_len = text.chars().count();
            runs.push(StaticResolvedRun {
                local_id: self.text_local,
                global_id: self.text_global,
                style_local,
                char_start,
                char_len,
                text,
            });
            char_start += char_len;
        }
        Ok(runs)
    }

    fn ordered_style_indices(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<Vec<usize>> {
        let mut order = Vec::new();
        for run in self.resolved_runs(runtime, instance)? {
            let style_index = self.style_index_for_local(run.style_local)?;
            if !order.contains(&style_index) {
                order.push(style_index);
            }
        }
        Ok(order)
    }

    fn style_font_size(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        style: &StaticTextStyle<'_>,
    ) -> Result<f32> {
        let property_key = property_key_for_name("TextStyle", "fontSize")
            .context("missing TextStyle.fontSize key")?;
        Ok(instance
            .double_property(style.local_id, property_key)
            .or_else(|| {
                runtime
                    .object(style.global_id as usize)
                    .and_then(|object| object.double_property("fontSize"))
            })
            .unwrap_or(12.0))
    }

    fn letter_spacing(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> f32 {
        let Ok(style) = self.base_style() else {
            return 0.0;
        };
        self.style_letter_spacing(runtime, instance, style)
    }

    fn style_letter_spacing(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        style: &StaticTextStyle<'_>,
    ) -> f32 {
        let Some(property_key) = property_key_for_name("TextStyle", "letterSpacing") else {
            return 0.0;
        };
        instance
            .double_property(style.local_id, property_key)
            .or_else(|| {
                runtime
                    .object(style.global_id as usize)
                    .and_then(|object| object.double_property("letterSpacing"))
            })
            .unwrap_or(0.0)
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn static_line_metrics(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        lines: &[StaticTextLine<'_>],
        runs: &[StaticResolvedRun],
        font_scale: f32,
    ) -> Result<Vec<StaticTextLineMetrics>> {
        // Exact port of `GlyphLine::ComputeLineSpacing`. The C++ implementation
        // computes adjusted ascent/descent per participating run, but uses the
        // natural ascent for the very first line's baseline. Custom lineHeight
        // is an authored absolute value and therefore does not follow the
        // fit-font-size scale; natural font metrics do.
        let mut metrics = Vec::with_capacity(lines.len());
        let mut cursor_y = 0.0f32;
        for (line_index, line) in lines.iter().enumerate() {
            let mut style_indices = self.style_indices_for_line(line, runs)?;
            if style_indices.is_empty() {
                // Empty paragraphs still have a shaped run in C++. Choose the
                // run touching the insertion point, then fall back to the base
                // style for the same observable line geometry.
                if let Some(run) = runs.iter().find(|run| {
                    line.char_start >= run.char_start
                        && line.char_start <= run.char_start + run.char_len
                }) {
                    style_indices.push(self.style_index_for_local(run.style_local)?);
                } else if !self.styles.is_empty() {
                    style_indices.push(0);
                }
            }

            let mut natural_ascent = 0.0f32;
            let mut adjusted_ascent = 0.0f32;
            let mut adjusted_descent = 0.0f32;
            for style_index in style_indices {
                let style = self
                    .styles
                    .get(style_index)
                    .context("line references a missing TextStylePaint")?;
                let Some(font_bytes) = style.font_bytes(instance) else {
                    continue;
                };
                let font = SkrifaFontRef::new(font_bytes)
                    .context("failed to parse font for line metrics")?;
                let skrifa_variations = style
                    .variations
                    .iter()
                    .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
                    .collect::<Vec<_>>();
                let location = font.axes().location(skrifa_variations.iter().copied());
                let location_ref = LocationRef::from(&location);
                let (ascent, descent) = harfbuzz_line_metrics(&font, location_ref);
                let font_size = self.style_font_size(runtime, instance, style)? * font_scale;
                let natural_ascent_px = ascent * font_size / TEXT_SHAPE_SCALE_F32;
                let natural_descent_px = -descent * font_size / TEXT_SHAPE_SCALE_F32;
                natural_ascent = natural_ascent.max(natural_ascent_px);

                let line_height = self.style_line_height(runtime, instance, style)?;
                let (ascent_px, descent_px) = if line_height < 0.0 {
                    (natural_ascent_px, natural_descent_px)
                } else {
                    let natural_height = natural_ascent_px + natural_descent_px;
                    let baseline_factor = if natural_height > 0.0 {
                        natural_ascent_px / natural_height
                    } else {
                        0.0
                    };
                    let authored_ascent = baseline_factor * line_height;
                    (authored_ascent, line_height - authored_ascent)
                };
                adjusted_ascent = adjusted_ascent.max(ascent_px);
                adjusted_descent = adjusted_descent.max(descent_px);
            }

            let top = cursor_y;
            let baseline = if line_index == 0 {
                natural_ascent
            } else {
                cursor_y + adjusted_ascent
            };
            let bottom = baseline + adjusted_descent;
            metrics.push(StaticTextLineMetrics {
                top,
                baseline,
                bottom,
            });
            cursor_y = bottom;
        }
        Ok(metrics)
    }

    fn style_line_height(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        style: &StaticTextStyle<'_>,
    ) -> Result<f32> {
        let property_key = property_key_for_name("TextStyle", "lineHeight")
            .context("missing TextStyle.lineHeight key")?;
        Ok(instance
            .double_property(style.local_id, property_key)
            .or_else(|| {
                runtime
                    .object(style.global_id as usize)
                    .and_then(|object| object.double_property("lineHeight"))
            })
            .unwrap_or(-1.0))
    }

    fn static_vertical_trim(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        lines: &[StaticTextLine<'_>],
        line_metrics: &[StaticTextLineMetrics],
        resolved_runs: &[StaticResolvedRun],
        font_scale: f32,
    ) -> Result<StaticVerticalTrim> {
        // Ported from C++ `src/text/text.cpp::computeVerticalTrim` for the
        // static text subset. Glyph layout is unchanged; this only trims the
        // auto-sized text box and shifts rendered content up by the top trim.
        let packed = self.text_uint_property(runtime, instance, "verticalTrimValue")?;
        let trim_top = packed & 0xff;
        let trim_bottom = (packed >> 8) & 0xff;
        if lines.is_empty() || (trim_top == TEXT_TRIM_NONE && trim_bottom == TEXT_TRIM_NONE) {
            return Ok(StaticVerticalTrim::default());
        }

        let mut trim = StaticVerticalTrim::default();

        if matches!(trim_top, TEXT_TRIM_TOP_CAP | TEXT_TRIM_TOP_EX) {
            if let Some(first_line) = lines.first().filter(|line| !line.text.is_empty()) {
                let Some(metrics) = line_metrics.get(first_line.line_index) else {
                    return Ok(trim);
                };
                let edge_px = self
                    .style_indices_for_line(first_line, resolved_runs)?
                    .into_iter()
                    .map(|style_index| {
                        self.style_vertical_edge_px(
                            runtime,
                            instance,
                            style_index,
                            trim_top,
                            font_scale,
                        )
                    })
                    .collect::<Result<Vec<_>>>()?
                    .into_iter()
                    .fold(0.0f32, f32::max);
                trim.top = ((metrics.baseline - edge_px) - metrics.top).max(0.0);
            }
        }

        if matches!(
            trim_bottom,
            TEXT_TRIM_BOTTOM_ALPHABETIC | TEXT_TRIM_BOTTOM_TEXT
        ) {
            if let Some(last_line) = lines.iter().rev().find(|line| !line.text.is_empty()) {
                let Some(metrics) = line_metrics.get(last_line.line_index) else {
                    return Ok(trim);
                };
                let descent_band = metrics.bottom - metrics.baseline;
                trim.bottom = if trim_bottom == TEXT_TRIM_BOTTOM_ALPHABETIC {
                    descent_band.max(0.0)
                } else {
                    let descent_px = self
                        .style_indices_for_line(last_line, resolved_runs)?
                        .into_iter()
                        .map(|style_index| {
                            self.style_descent_px(runtime, instance, style_index, font_scale)
                        })
                        .collect::<Result<Vec<_>>>()?
                        .into_iter()
                        .fold(0.0f32, f32::max);
                    (descent_band - descent_px).max(0.0)
                };
            }
        }

        Ok(trim)
    }

    fn style_indices_for_line(
        &self,
        line: &StaticTextLine<'_>,
        resolved_runs: &[StaticResolvedRun],
    ) -> Result<Vec<usize>> {
        let line_start = line.char_start;
        let line_end = line.char_start + line.text.chars().count();
        let mut indices = Vec::new();
        for run in resolved_runs {
            let run_start = run.char_start;
            let run_end = run.char_start + run.char_len;
            if line_start < run_end && line_end > run_start {
                let style_index = self.style_index_for_local(run.style_local)?;
                if !indices.contains(&style_index) {
                    indices.push(style_index);
                }
            }
        }
        Ok(indices)
    }

    fn style_vertical_edge_px(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        style_index: usize,
        trim_top: u64,
        font_scale: f32,
    ) -> Result<f32> {
        let style = &self.styles[style_index];
        let Some(font_bytes) = style.font_bytes(instance) else {
            return Ok(0.0);
        };
        let font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for trim metrics")?;
        let skrifa_variations = style
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = font.axes().location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (ascent, _) = harfbuzz_line_metrics(&font, location_ref);
        let ch = if trim_top == TEXT_TRIM_TOP_CAP {
            'H'
        } else {
            'x'
        };
        let raw_edge = font
            .charmap()
            .map(ch)
            .and_then(|glyph_id| {
                font.glyph_metrics(Size::new(TEXT_SHAPE_SCALE_F32), location_ref)
                    .bounds(glyph_id)
                    .map(|bounds| bounds.y_max)
            })
            .unwrap_or(ascent);
        let edge = harfbuzz_scaled_glyph_top(raw_edge);
        Ok(
            edge * self.style_font_size(runtime, instance, style)? * font_scale
                / TEXT_SHAPE_SCALE_F32,
        )
    }

    fn style_descent_px(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        style_index: usize,
        font_scale: f32,
    ) -> Result<f32> {
        let style = &self.styles[style_index];
        let Some(font_bytes) = style.font_bytes(instance) else {
            return Ok(0.0);
        };
        let font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for trim metrics")?;
        let skrifa_variations = style
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = font.axes().location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (_, descent) = harfbuzz_line_metrics(&font, location_ref);
        Ok(
            (-descent + 1.0) * self.style_font_size(runtime, instance, style)? * font_scale
                / TEXT_SHAPE_SCALE_F32,
        )
    }

    fn fit_font_scale(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        runs: &[StaticResolvedRun],
        text: &str,
        shaper: &harfrust::Shaper<'_>,
        disable_legacy_kern: bool,
    ) -> Result<f32> {
        // Ported from C++ src/text/text.cpp::Text::fitFontScale for the
        // current static text subset.
        let overflow = self.text_uint_property(runtime, instance, "overflowValue")?;
        if overflow != TEXT_OVERFLOW_FIT_FONT_SIZE {
            return Ok(1.0);
        }
        let max_size = self.max_authored_font_size(runtime, instance, runs)?;
        self.fit_font_scale_for_max_size(
            runtime,
            instance,
            layout_constraint,
            runs,
            text,
            shaper,
            disable_legacy_kern,
            max_size,
        )
    }

    fn max_authored_font_size(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        runs: &[StaticResolvedRun],
    ) -> Result<f32> {
        let mut max_size = 0.0f32;
        for run in runs {
            if run.text.is_empty() {
                continue;
            }
            let style_index = self.style_index_for_local(run.style_local)?;
            let style = &self.styles[style_index];
            if style.font_bytes(instance).is_some() {
                max_size = max_size.max(self.style_font_size(runtime, instance, style)?);
            }
        }
        Ok(max_size)
    }

    fn fit_font_scale_for_max_size(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        runs: &[StaticResolvedRun],
        text: &str,
        shaper: &harfrust::Shaper<'_>,
        disable_legacy_kern: bool,
        max_size: f32,
    ) -> Result<f32> {
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        if max_size <= 1.0 || sizing == TEXT_SIZING_AUTO_WIDTH {
            return Ok(1.0);
        }

        let box_width = self.effective_width(runtime, instance, layout_constraint)?;
        let box_height = self.effective_height(runtime, instance, layout_constraint)?;
        let base_style = self.base_style()?;
        let base_font_size = self.style_font_size(runtime, instance, base_style)?;
        let letter_spacing = self.letter_spacing(runtime, instance);

        let fits = |top_size: i32| -> Result<bool> {
            let font_scale = top_size as f32 / max_size;
            let scale = base_font_size * font_scale / TEXT_SHAPE_SCALE_F32;
            let lines = self.layout_static_text_lines(
                runtime,
                instance,
                layout_constraint,
                text,
                shaper,
                disable_legacy_kern,
                scale,
                letter_spacing,
            )?;
            let max_width = lines
                .iter()
                .map(|line| self.styled_line_width(runtime, instance, line, runs, font_scale))
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .fold(0.0f32, f32::max);
            let line_metrics =
                self.static_line_metrics(runtime, instance, &lines, runs, font_scale)?;
            let height = line_metrics
                .last()
                .map(|metrics| metrics.bottom)
                .unwrap_or(0.0);
            Ok(max_width <= box_width && (sizing != TEXT_SIZING_FIXED || height <= box_height))
        };

        let mut lo = 1i32;
        let mut hi = (max_size as i32).max(1);
        let mut best = 1i32;
        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            if fits(mid)? {
                best = mid;
                let Some(next) = mid.checked_add(1) else {
                    break;
                };
                lo = next;
            } else {
                let Some(previous) = mid.checked_sub(1) else {
                    break;
                };
                hi = previous;
            }
        }
        Ok(best as f32 / max_size)
    }

    fn styled_line_glyphs(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        line: &StaticTextLine<'_>,
        runs: &[StaticResolvedRun],
        font_scale: f32,
    ) -> Result<Vec<StyledTextGlyph>> {
        let line_start = line.char_start;
        let line_end = line_start + line.text.chars().count();
        let mut glyphs = Vec::new();
        for run in runs {
            let run_start = run.char_start;
            let run_end = run.char_start + run.char_len;
            let segment_start = line_start.max(run_start);
            let segment_end = line_end.min(run_end);
            if segment_start >= segment_end {
                continue;
            }
            let style_index = self.style_index_for_local(run.style_local)?;
            let segment_text = char_slice(
                line.text,
                segment_start - line_start,
                segment_end - line_start,
            );
            glyphs.extend(self.styled_text_glyphs_for_style(
                runtime,
                instance,
                segment_text,
                segment_start,
                style_index,
                font_scale,
            )?);
        }
        Ok(glyphs)
    }

    fn styled_line_width(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        line: &StaticTextLine<'_>,
        runs: &[StaticResolvedRun],
        font_scale: f32,
    ) -> Result<f32> {
        Ok(self
            .styled_line_glyphs(runtime, instance, line, runs, font_scale)?
            .iter()
            .map(|glyph| glyph.advance)
            .sum())
    }

    fn styled_text_glyphs_for_style(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        char_start: usize,
        style_index: usize,
        font_scale: f32,
    ) -> Result<Vec<StyledTextGlyph>> {
        let style = self
            .styles
            .get(style_index)
            .with_context(|| format!("missing TextStylePaint index {style_index}"))?;
        let Some(font_bytes) = style.font_bytes(instance) else {
            return Ok(Vec::new());
        };
        let font_size = self.style_font_size(runtime, instance, style)?;
        let scale = font_size * font_scale / TEXT_SHAPE_SCALE_F32;
        let letter_spacing = self.style_letter_spacing(runtime, instance, style);
        let raw_glyphs = shape_text_glyphs_for_style(font_bytes, style, text)?;
        Ok(raw_glyphs
            .iter()
            .enumerate()
            .map(|(glyph_index, glyph)| StyledTextGlyph {
                glyph_id: glyph.glyph_id,
                char_index: char_start + character_index_for_cluster(text, glyph.cluster),
                char_len: glyph_character_len(text, &raw_glyphs, glyph_index),
                style_index,
                advance: glyph.advance * scale + letter_spacing,
                scale,
            })
            .collect())
    }

    fn base_style(&self) -> Result<&StaticTextStyle<'a>> {
        self.styles
            .first()
            .context("static text subset requires a base TextStylePaint")
    }

    fn text_width(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<f32> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(0.0);
        }
        let property_key =
            property_key_for_name("Text", "width").context("missing Text.width key")?;
        Ok(instance
            .double_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.double_property("width"))
            })
            .unwrap_or(0.0))
    }

    fn text_height(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<f32> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(0.0);
        }
        let property_key =
            property_key_for_name("Text", "height").context("missing Text.height key")?;
        Ok(instance
            .double_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.double_property("height"))
            })
            .unwrap_or(0.0))
    }

    fn effective_sizing(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<u64> {
        let authored = self.authored_sizing(runtime, instance)?;
        Ok(layout_constraint
            .map(|constraint| constraint.effective_sizing(authored))
            .unwrap_or(authored))
    }

    fn authored_sizing(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<u64> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(if self.text_input_multiline(runtime, instance)? {
                TEXT_SIZING_AUTO_HEIGHT
            } else {
                TEXT_SIZING_AUTO_WIDTH
            });
        }
        self.text_uint_property(runtime, instance, "sizingValue")
    }

    fn static_text_min_y(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        line_metrics: &[StaticTextLineMetrics],
    ) -> Result<f32> {
        let uses_baseline_origin = self.text_uint_property(runtime, instance, "originValue")? == 1;
        Ok(if uses_baseline_origin {
            -line_metrics
                .first()
                .map(|metrics| metrics.baseline)
                .unwrap_or(0.0)
        } else {
            0.0
        })
    }

    fn unshaped_local_bounds(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        if self.effective_sizing(runtime, instance, layout_constraint)? != TEXT_SIZING_FIXED {
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
        }
        let width = self.effective_width(runtime, instance, layout_constraint)?;
        let height = self.effective_height(runtime, instance, layout_constraint)?;
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;
        Ok(Some((-width * origin_x, -height * origin_y, width, height)))
    }

    fn clip_bounds(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<Option<StaticTextClipBounds>> {
        if self.effective_sizing(runtime, instance, layout_constraint)? != TEXT_SIZING_FIXED
            || self.text_uint_property(runtime, instance, "overflowValue")? != TEXT_OVERFLOW_CLIPPED
        {
            return Ok(None);
        }

        let resolved_runs = self.resolved_runs(runtime, instance)?;
        let text = resolved_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let mut lines = Vec::new();
        let mut line_metrics = Vec::new();
        let mut measured_width = 0.0f32;
        let mut font_scale = 1.0f32;

        if !text.is_empty()
            && let Ok(base_style) = self.base_style()
            && self.style_font_size(runtime, instance, base_style)? >= 0.0
            && let Some(font_bytes) = base_style.font_bytes(instance)
        {
            let harf_font =
                HarfFontRef::new(font_bytes).context("failed to parse font for clip layout")?;
            let harf_variations = base_style
                .variations
                .iter()
                .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
                .collect::<Vec<_>>();
            let shaper_instance = if harf_variations.is_empty() {
                None
            } else {
                Some(ShaperInstance::from_variations(
                    &harf_font,
                    harf_variations.iter().copied(),
                ))
            };
            let shaper_data = ShaperData::new(&harf_font);
            let shaper = shaper_data
                .shaper(&harf_font)
                .instance(shaper_instance.as_ref())
                .build();
            let skrifa_font =
                SkrifaFontRef::new(font_bytes).context("failed to parse font for clip layout")?;
            let disable_legacy_kern = disable_legacy_kern_for_advances(&skrifa_font);
            font_scale = self.fit_font_scale(
                runtime,
                instance,
                layout_constraint,
                &resolved_runs,
                &text,
                &shaper,
                disable_legacy_kern,
            )?;
            let scale = self.style_font_size(runtime, instance, base_style)? * font_scale
                / TEXT_SHAPE_SCALE_F32;
            lines = self.layout_static_text_lines(
                runtime,
                instance,
                layout_constraint,
                &text,
                &shaper,
                disable_legacy_kern,
                scale,
                self.letter_spacing(runtime, instance),
            )?;
            line_metrics =
                self.static_line_metrics(runtime, instance, &lines, &resolved_runs, font_scale)?;
            measured_width = lines
                .iter()
                .map(|line| {
                    self.styled_line_width(runtime, instance, line, &resolved_runs, font_scale)
                })
                .collect::<Result<Vec<_>>>()?
                .into_iter()
                .fold(0.0f32, f32::max);
        }

        let layout_info = self.static_layout_info(
            runtime,
            instance,
            &lines,
            &line_metrics,
            &resolved_runs,
            measured_width,
            font_scale,
            false,
            layout_constraint,
        )?;
        let total_height = line_metrics
            .last()
            .map(|metrics| metrics.bottom + layout_info.min_y)
            .unwrap_or(layout_info.min_y);
        let width = self.effective_width(runtime, instance, layout_constraint)?;
        let height = self.effective_height(runtime, instance, layout_constraint)?;
        let vertical_align_offset =
            match self.text_uint_property(runtime, instance, "verticalAlignValue")? {
                1 => total_height - height,
                2 => (total_height - height) / 2.0,
                _ => 0.0,
            };
        let local_transform = self.static_render_transform(
            runtime,
            instance,
            layout_constraint,
            layout_info,
            measured_width,
            0.0,
            total_height,
            line_metrics
                .first()
                .map(|metrics| metrics.baseline)
                .unwrap_or(0.0),
        )?;
        Ok(Some(StaticTextClipBounds {
            // Official `buildRenderStyles` cancels normalized origin before
            // constructing m_clipRect, then adds verticalAlignOffset. The same
            // local Text transform is applied to both clip and glyph paths.
            bounds: (
                0.0,
                layout_info.min_y + vertical_align_offset,
                width,
                height,
            ),
            local_transform,
        }))
    }

    fn effective_width(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<f32> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(layout_constraint
                .map(|constraint| constraint.width)
                .unwrap_or(0.0));
        }
        match layout_constraint {
            Some(constraint) => Ok(constraint.width),
            None => self.text_width(runtime, instance),
        }
    }

    fn effective_height(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<f32> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(layout_constraint
                .map(|constraint| constraint.height)
                .unwrap_or(0.0));
        }
        match layout_constraint {
            Some(constraint) => Ok(constraint.height),
            None => self.text_height(runtime, instance),
        }
    }

    fn text_double_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: f32,
    ) -> Result<f32> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(default);
        }
        let property_key = property_key_for_name("Text", property_name)
            .with_context(|| format!("missing Text.{property_name} key"))?;
        Ok(instance
            .double_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.double_property(property_name))
            })
            .unwrap_or(default))
    }

    fn text_bool_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: bool,
    ) -> Result<bool> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(default);
        }
        let property_key = property_key_for_name("Text", property_name)
            .with_context(|| format!("missing Text.{property_name} key"))?;
        Ok(instance
            .bool_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.bool_property(property_name))
            })
            .unwrap_or(default))
    }

    fn text_uint_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
    ) -> Result<u64> {
        if self.kind == StaticTextKind::TextInput {
            return Ok(match property_name {
                "sizingValue" => {
                    if self.text_input_multiline(runtime, instance)? {
                        TEXT_SIZING_AUTO_HEIGHT
                    } else {
                        TEXT_SIZING_AUTO_WIDTH
                    }
                }
                "alignValue" | "overflowValue" | "verticalAlignValue" | "verticalTrimValue"
                | "wrapValue" => 0,
                _ => 0,
            });
        }
        let property_key = property_key_for_name("Text", property_name)
            .with_context(|| format!("missing Text.{property_name} key"))?;
        Ok(instance
            .uint_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.uint_property(property_name))
            })
            .unwrap_or(0))
    }

    fn text_blend_mode_value(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<u32> {
        let property_key = property_key_for_name("Drawable", "blendModeValue")
            .context("missing Drawable.blendModeValue key")?;
        Ok(instance
            .uint_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.uint_property("blendModeValue"))
            })
            .unwrap_or(3) as u32)
    }

    fn should_apply_static_ellipsis(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<bool> {
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        let overflow = self.text_uint_property(runtime, instance, "overflowValue")?;
        Ok(sizing == TEXT_SIZING_FIXED
            && overflow == TEXT_OVERFLOW_ELLIPSIS
            && self.effective_width(runtime, instance, layout_constraint)? > 0.0)
    }

    fn first_static_wrapped_line_end(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        glyphs: &[TextGlyph],
        max_width: f32,
        scale: f32,
        letter_spacing: f32,
    ) -> Result<usize> {
        let wrap = self.text_uint_property(runtime, instance, "wrapValue")?;
        if wrap != 0 || glyphs.is_empty() || max_width <= 0.0 {
            return Ok(glyphs.len());
        }

        let mut line_end = 0;
        let mut saw_word = false;
        for (start, word) in text.split_word_bound_indices() {
            if word.chars().all(char::is_whitespace) {
                continue;
            }
            let word_end = start + word.len();
            let candidate_end = glyphs
                .iter()
                .rposition(|glyph| glyph.cluster < word_end as u32)
                .map(|index| index + 1)
                .unwrap_or(line_end);
            if candidate_end <= line_end {
                continue;
            }
            let width = text_glyph_width(&glyphs[..candidate_end], scale, letter_spacing);
            if width > max_width && saw_word {
                return Ok(line_end);
            }
            if width > max_width {
                return Ok(first_fitting_glyph_end(
                    glyphs,
                    max_width,
                    scale,
                    letter_spacing,
                ));
            }
            line_end = candidate_end;
            saw_word = true;
        }

        Ok(glyphs.len())
    }

    fn layout_static_text_lines<'text>(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text: &'text str,
        shaper: &harfrust::Shaper<'_>,
        disable_legacy_kern: bool,
        scale: f32,
        letter_spacing: f32,
    ) -> Result<Vec<StaticTextLine<'text>>> {
        let authored_lines = split_static_text_lines(text);
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        let wrap = self.text_uint_property(runtime, instance, "wrapValue")?;
        let max_width = self.effective_width(runtime, instance, layout_constraint)?;
        let parent_is_layout_not_artboard = layout_constraint.is_some();
        if (sizing == TEXT_SIZING_AUTO_WIDTH && !parent_is_layout_not_artboard)
            || wrap != 0
            || max_width <= 0.0
        {
            return Ok(authored_lines);
        }

        let mut lines = Vec::new();
        let mut line_index = 0;
        for authored_line in authored_lines {
            if authored_line.text.is_empty() {
                lines.push(StaticTextLine {
                    text: authored_line.text,
                    char_start: authored_line.char_start,
                    line_index,
                    soft_wrap_skipped_start: None,
                    terminal_soft_wrap_skipped_end: None,
                });
                line_index += 1;
                continue;
            }

            let mut remaining = authored_line.text;
            let mut char_start = authored_line.char_start;
            let mut soft_wrap_skipped_start = None;
            while !remaining.is_empty() {
                let glyphs = shape_text_glyphs(shaper, remaining, disable_legacy_kern);
                let glyph_end = self.first_static_wrapped_line_end(
                    runtime,
                    instance,
                    remaining,
                    &glyphs,
                    max_width,
                    scale,
                    letter_spacing,
                )?;
                let mut byte_end = byte_index_for_glyph_end(remaining, &glyphs, glyph_end);
                if byte_end == 0 {
                    byte_end = remaining
                        .char_indices()
                        .nth(1)
                        .map(|(index, _)| index)
                        .unwrap_or(remaining.len());
                }

                let line_text = &remaining[..byte_end];
                let char_end = char_start + line_text.chars().count();
                lines.push(StaticTextLine {
                    text: line_text,
                    char_start,
                    line_index,
                    soft_wrap_skipped_start,
                    terminal_soft_wrap_skipped_end: None,
                });
                line_index += 1;

                if byte_end >= remaining.len() {
                    break;
                }
                let skipped = leading_whitespace_bytes(&remaining[byte_end..]);
                char_start += remaining[..byte_end + skipped].chars().count();
                remaining = &remaining[byte_end + skipped..];
                if remaining.is_empty()
                    && skipped > 0
                    && let Some(line) = lines.last_mut()
                {
                    line.terminal_soft_wrap_skipped_end = Some(char_start);
                }
                soft_wrap_skipped_start = (!remaining.is_empty()).then_some(char_end);
            }
        }

        Ok(lines)
    }

    fn static_layout_info(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        lines: &[StaticTextLine<'_>],
        line_metrics: &[StaticTextLineMetrics],
        resolved_runs: &[StaticResolvedRun],
        measured_width: f32,
        font_scale: f32,
        apply_ellipsis: bool,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<StaticTextLayoutInfo> {
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        let bounds_width = match sizing {
            TEXT_SIZING_AUTO_HEIGHT | TEXT_SIZING_FIXED => {
                self.effective_width(runtime, instance, layout_constraint)?
            }
            _ => measured_width,
        };
        let paragraph_width = if layout_constraint.is_some() {
            // Ported from C++ `src/text/text.cpp` `Text::update` /
            // `buildRenderStyles`: a Text parented by a non-artboard
            // LayoutComponent keeps auto-width bounds based on measured text,
            // but line breaking/alignment uses the controlled layout width.
            self.effective_width(runtime, instance, layout_constraint)?
        } else {
            bounds_width
        };
        let vertical_trim = if sizing == TEXT_SIZING_FIXED {
            StaticVerticalTrim::default()
        } else {
            self.static_vertical_trim(
                runtime,
                instance,
                lines,
                line_metrics,
                resolved_runs,
                font_scale,
            )?
        };
        let min_y = self.static_text_min_y(runtime, instance, line_metrics)?;
        let full_height = line_metrics
            .last()
            .map(|metrics| min_y + metrics.bottom)
            .unwrap_or(min_y);
        let bounds_height = match sizing {
            TEXT_SIZING_FIXED => self.effective_height(runtime, instance, layout_constraint)?,
            _ => (full_height - vertical_trim.top - vertical_trim.bottom).max(min_y) - min_y,
        };
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;
        let align_value = self.text_uint_property(runtime, instance, "alignValue")?;
        let last_line_index = lines.last().map(|line| line.line_index).unwrap_or(0);
        let mut total_height = full_height;
        let mut ellipsis_line = None;
        let mut is_ellipsis_line_last = false;

        if apply_ellipsis && !lines.is_empty() {
            // Mirrors src/text/text.cpp::computeBoundsInfo for the static text
            // subset: choose the last visual line whose bottom fits the fixed
            // box, falling back to the first line when nothing fits.
            let mut ellipsed_height = 0.0;
            for (line, metrics) in lines.iter().zip(line_metrics) {
                let line_bottom = min_y + metrics.bottom;
                if line_bottom <= bounds_height {
                    ellipsed_height = line_bottom;
                    ellipsis_line = Some(line.line_index);
                }
            }
            let chosen_line = ellipsis_line.unwrap_or(0);
            ellipsis_line = Some(chosen_line);
            is_ellipsis_line_last = chosen_line == last_line_index;
            if chosen_line > 0 {
                total_height = ellipsed_height;
            }
        }

        let mut y_offset = -bounds_height * origin_y;
        if sizing == TEXT_SIZING_FIXED {
            // Mirrors src/text/text.cpp::buildRenderStyles fixed-size vertical
            // alignment transform for top/bottom/middle text.
            match self.text_uint_property(runtime, instance, "verticalAlignValue")? {
                1 => y_offset += bounds_height - total_height,
                2 => y_offset += (bounds_height - total_height) / 2.0,
                _ => {}
            }
        }

        Ok(StaticTextLayoutInfo {
            ellipsis_line,
            is_ellipsis_line_last,
            paragraph_width,
            align_value,
            top_trim: vertical_trim.top,
            min_y,
            x_offset: -bounds_width * origin_x,
            y_offset,
        })
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn static_render_transform(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        layout_info: StaticTextLayoutInfo,
        measured_width: f32,
        minimum_line_x: f32,
        total_height: f32,
        first_baseline: f32,
    ) -> Result<Mat2D> {
        if self.text_uint_property(runtime, instance, "overflowValue")? != TEXT_OVERFLOW_FIT {
            return Ok(Mat2D([
                1.0,
                0.0,
                0.0,
                1.0,
                layout_info.x_offset,
                layout_info.y_offset,
            ]));
        }

        // Ported from C++ `src/text/text.cpp::Text::buildRenderStyles` fit
        // handling. Fit keeps authored glyph sizes/line breaks, then applies
        // one uniform render transform so aspect ratio and run proportions are
        // preserved.
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        let bounds_width = if sizing == TEXT_SIZING_AUTO_WIDTH {
            measured_width
        } else {
            self.effective_width(runtime, instance, layout_constraint)?
        };
        let bounds_height = if sizing == TEXT_SIZING_FIXED {
            self.effective_height(runtime, instance, layout_constraint)?
        } else {
            total_height
        };
        let x_scale = if sizing != TEXT_SIZING_AUTO_WIDTH && measured_width > bounds_width {
            bounds_width / measured_width
        } else {
            1.0
        };
        let fit_from_baseline =
            self.text_bool_property(runtime, instance, "fitFromBaseline", true)?;
        let fit_baseline = if fit_from_baseline {
            first_baseline
        } else {
            0.0
        };
        let y_scale = if sizing == TEXT_SIZING_FIXED && total_height > bounds_height {
            (bounds_height - fit_baseline) / (total_height - fit_baseline)
        } else {
            1.0
        };
        let scale = x_scale.min(y_scale).max(0.0);
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;
        let mut x_offset = -bounds_width * origin_x;
        let mut y_offset = -bounds_height * origin_y + fit_baseline * (1.0 - scale);

        if scale != 1.0 {
            match self.text_uint_property(runtime, instance, "alignValue")? {
                1 => {
                    x_offset += bounds_width - measured_width * scale - minimum_line_x * scale;
                }
                2 => {
                    x_offset +=
                        (bounds_width - measured_width * scale) / 2.0 - minimum_line_x * scale;
                }
                _ => {}
            }
        }

        if sizing == TEXT_SIZING_FIXED {
            match self.text_uint_property(runtime, instance, "verticalAlignValue")? {
                1 => y_offset = -bounds_height * origin_y + bounds_height - total_height * scale,
                2 => {
                    y_offset =
                        -bounds_height * origin_y + (bounds_height - total_height * scale) / 2.0;
                }
                _ => {}
            }
        }

        Ok(Mat2D([scale, 0.0, 0.0, scale, x_offset, y_offset]))
    }

    fn style_index_for_local(&self, style_local: usize) -> Result<usize> {
        self.styles
            .iter()
            .position(|style| style.local_id == style_local)
            .with_context(|| format!("TextValueRun references missing style local {style_local}"))
    }

    fn static_fixed_height_shows_first_line_only(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        line_metrics: &[StaticTextLineMetrics],
    ) -> Result<bool> {
        let height = self.effective_height(runtime, instance, layout_constraint)?;
        let min_y = self.static_text_min_y(runtime, instance, line_metrics)?;
        let second_line_bottom = line_metrics.get(1).map(|metrics| min_y + metrics.bottom);
        Ok(height > 0.0 && second_line_bottom.is_some_and(|bottom| height < bottom))
    }

    fn text_input_multiline(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<bool> {
        let property_key = property_key_for_name("TextInput", "multiline")
            .context("missing TextInput.multiline key")?;
        Ok(instance
            .bool_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.bool_property("multiline"))
            })
            .unwrap_or(true))
    }

    fn text_input_cursor_path(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<Vec<RuntimePathCommand>> {
        let runs = self.resolved_runs(runtime, instance)?;
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let lines = split_static_text_lines(&text);
        let line_metrics = self.static_line_metrics(runtime, instance, &lines, &runs, 1.0)?;
        let line_height = line_metrics
            .first()
            .map(|metrics| metrics.bottom - metrics.top)
            .unwrap_or(0.0);
        Ok(vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 1.0, y: 0.0 },
            RuntimePathCommand::Line {
                x: 1.0,
                y: line_height,
            },
            RuntimePathCommand::Line {
                x: 0.0,
                y: line_height,
            },
            RuntimePathCommand::Close,
        ])
    }

    fn text_input_paint_commands(
        &self,
        instance: &ArtboardInstance,
        container: &ShapePaintContainerNode,
        needs_save_operation: bool,
        render_opacity: f32,
        shape_world: Mat2D,
        path_buckets: Vec<StaticTextPathBucket>,
    ) -> Result<Vec<RuntimeShapePaintCommand>> {
        let mut commands = Vec::new();
        for path_bucket in order_opacity_buckets_like_cpp(path_buckets) {
            for paint in &container.paints {
                let mut path_commands = path_bucket.commands.clone();
                if paint.path_kind == Some(ShapePaintPathKind::World) {
                    transform_path_commands(&mut path_commands, shape_world);
                }
                let Some(mut command) = runtime_shape_paint_command(
                    instance,
                    paint,
                    container.blend_mode_value,
                    needs_save_operation,
                    render_opacity * path_bucket.opacity,
                    shape_world,
                    path_commands,
                    false,
                    false,
                    true,
                ) else {
                    continue;
                };
                command.shape_world_override = Some(shape_world);
                if command.paint_type == RuntimeShapePaintKind::Fill {
                    command.path_kind = RuntimeShapePaintPathKind::LocalClockwise;
                }
                commands.push(command);
            }
        }
        Ok(commands)
    }
}

impl<'a> StaticTextStyle<'a> {
    fn from_graph(
        runtime: &'a RuntimeFile,
        graph: &'a ArtboardGraph,
        style_local: usize,
    ) -> Result<Self> {
        let style_global = global_for_local(graph, style_local)?;
        let container = graph
            .shape_paint_containers
            .iter()
            .find(|container| container.local_id == style_local);
        if let Some(container) = container {
            for paint in &container.paints {
                if !matches!(
                    paint.paint_type,
                    ShapePaintKind::Fill | ShapePaintKind::Stroke
                ) {
                    bail!("static text subset only supports Fill/Stroke text paints");
                }
                if !matches!(
                    paint.paint_state,
                    Some(
                        ShapePaintStateNode::SolidColor { .. }
                            | ShapePaintStateNode::LinearGradient { .. }
                            | ShapePaintStateNode::RadialGradient { .. }
                    )
                ) {
                    bail!("static text subset only supports solid/gradient text fill/stroke");
                }
                if paint
                    .effects
                    .iter()
                    .any(|effect| effect.type_name != "DashPath")
                {
                    bail!("static text subset only supports DashPath text paint effects");
                }
            }
        }

        let style = runtime
            .object(style_global as usize)
            .with_context(|| format!("missing TextStylePaint global {style_global}"))?;
        let (font_asset_id, font_bytes) = if style.property("fontAssetId").is_some() {
            let font_asset_index = style
                .uint_property("fontAssetId")
                .context("TextStylePaint serialized fontAssetId is not a uint")?;
            let font_asset = runtime
                .file_asset(
                    usize::try_from(font_asset_index).context("font asset id is too large")?,
                )
                .context("TextStylePaint fontAssetId did not resolve to a file asset")?;
            if font_asset.type_name != "FontAsset" {
                bail!(
                    "static text subset expected FontAsset, found {} global {}",
                    font_asset.type_name,
                    font_asset.id
                );
            }
            let asset_id = font_asset
                .uint_property("assetId")
                .context("FontAsset is missing its semantic assetId")?;
            let asset_id = u32::try_from(asset_id).context("FontAsset assetId is too large")?;
            (
                Some(asset_id),
                embedded_file_asset_bytes(runtime, font_asset.id),
            )
        } else {
            (None, None)
        };

        let style_component = graph
            .components
            .iter()
            .find(|component| component.local_id == style_local)
            .context("TextStylePaint component is missing")?;
        let mut variations = Vec::new();
        for axis_local in style_component.children.iter().copied().filter(|local| {
            graph
                .local_objects
                .iter()
                .find(|object| object.local_id == *local)
                .and_then(|object| object.type_name)
                == Some("TextStyleAxis")
        }) {
            let axis_global = global_for_local(graph, axis_local)?;
            let axis = runtime
                .object(axis_global as usize)
                .with_context(|| format!("missing TextStyleAxis global {axis_global}"))?;
            let tag = axis
                .uint_property("tag")
                .with_context(|| format!("TextStyleAxis global {axis_global} missing tag"))?
                as u32;
            let axis_value = axis.double_property("axisValue").unwrap_or(0.0);
            variations.push((tag, axis_value));
        }

        Ok(Self {
            local_id: style_local,
            global_id: style_global,
            name: style.string_property("name").map(str::to_owned),
            container,
            font_asset_id,
            font_bytes,
            variations,
        })
    }

    fn font_bytes<'instance>(
        &'instance self,
        instance: &'instance ArtboardInstance,
    ) -> Option<&'instance [u8]> {
        self.font_bytes.or_else(|| {
            self.font_asset_id
                .and_then(|asset_id| instance.external_font_asset_bytes(asset_id))
        })
    }
}

// Mirrors the static coverage/translation subset from C++
// src/text/text_modifier_group.cpp and src/text/text_modifier_range.cpp.
impl StaticTextModifierGroup {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierGroup global {global_id}"))?;
        let flags = object.uint_property("modifierFlags").unwrap_or(0);
        const MODIFY_TRANSLATION: u64 = 1 << 2;
        const MODIFY_ROTATION: u64 = 1 << 3;
        const MODIFY_SCALE: u64 = 1 << 4;
        const MODIFY_OPACITY: u64 = 1 << 5;
        const INVERT_OPACITY: u64 = 1 << 6;
        if flags
            & !(MODIFY_TRANSLATION
                | MODIFY_ROTATION
                | MODIFY_SCALE
                | MODIFY_OPACITY
                | INVERT_OPACITY)
            != 0
        {
            bail!(
                "static text subset only supports translation/rotation/scale/opacity TextModifierGroup flags, found {flags}"
            );
        }

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierGroup local {local_id} component is missing"))?;
        let mut ranges = Vec::new();
        let mut follow_path_modifiers = Vec::new();
        for child_local in &component.children {
            match type_for_local(graph, *child_local) {
                Some("TextModifierRange") => {
                    ranges.push(StaticTextModifierRange::from_graph(
                        runtime,
                        graph,
                        *child_local,
                    )?);
                }
                Some("TextFollowPathModifier") => {
                    follow_path_modifiers.push(StaticTextFollowPathModifier::from_graph(
                        runtime,
                        graph,
                        *child_local,
                    )?);
                }
                Some(type_name) => {
                    bail!("static text subset does not support TextModifierGroup child {type_name}")
                }
                None => bail!(
                    "static text subset does not support unknown TextModifierGroup child local {child_local}"
                ),
            }
        }

        Ok(Self {
            local_id,
            global_id,
            ranges,
            follow_path_modifiers,
        })
    }

    fn transform(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        amount: f32,
        glyph: &StaticTextGlyphContext<'_>,
    ) -> Result<Mat2D> {
        let flags = runtime_uint_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "modifierFlags",
            0,
        )?;
        const MODIFY_TRANSLATION: u64 = 1 << 2;
        const MODIFY_ROTATION: u64 = 1 << 3;
        const MODIFY_SCALE: u64 = 1 << 4;
        let follows_path = !self.follow_path_modifiers.is_empty();
        if amount == 0.0 && !follows_path {
            return Ok(Mat2D::IDENTITY);
        }

        let mut x = 0.0;
        let mut y = 0.0;
        let mut rotation = 0.0;
        let mut scale_x = 1.0;
        let mut scale_y = 1.0;

        if follows_path {
            // Ported from C++ `src/text/text_modifier_group.cpp`
            // `TextModifierGroup::transform` for follow-path modifiers.
            let mut current = StaticFollowPathGlyphTransform {
                x: glyph.origin_x,
                y: glyph.origin_y,
                rotation: 0.0,
            };
            let offset = if flags & MODIFY_TRANSLATION != 0 {
                (
                    runtime_double_property(
                        runtime,
                        instance,
                        "TextModifierGroup",
                        self.local_id,
                        self.global_id,
                        "x",
                        0.0,
                    )?,
                    runtime_double_property(
                        runtime,
                        instance,
                        "TextModifierGroup",
                        self.local_id,
                        self.global_id,
                        "y",
                        0.0,
                    )?,
                )
            } else {
                (0.0, 0.0)
            };
            for modifier in &self.follow_path_modifiers {
                current = modifier.transform_glyph(runtime, instance, current, glyph, offset)?;
            }
            x = (current.x - glyph.origin_x) * amount;
            y = (current.y - glyph.origin_y) * amount;
            rotation += current.rotation * amount;
        } else if flags & MODIFY_TRANSLATION != 0 {
            x = runtime_double_property(
                runtime,
                instance,
                "TextModifierGroup",
                self.local_id,
                self.global_id,
                "x",
                0.0,
            )? * amount;
            y = runtime_double_property(
                runtime,
                instance,
                "TextModifierGroup",
                self.local_id,
                self.global_id,
                "y",
                0.0,
            )? * amount;
        }

        if flags & MODIFY_ROTATION != 0 {
            rotation += runtime_double_property(
                runtime,
                instance,
                "TextModifierGroup",
                self.local_id,
                self.global_id,
                "rotation",
                0.0,
            )? * amount;
        }
        if flags & MODIFY_SCALE != 0 {
            let inverse_amount = 1.0 - amount;
            scale_x = inverse_amount
                + runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "scaleX",
                    1.0,
                )? * amount;
            scale_y = inverse_amount
                + runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "scaleY",
                    1.0,
                )? * amount;
        }
        let mut transform = Mat2D::from_rotation(rotation);
        transform.0[4] = x;
        transform.0[5] = y;
        transform.scale_by_values(scale_x, scale_y);
        Ok(transform)
    }

    fn modifies_opacity(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<bool> {
        let flags = runtime_uint_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "modifierFlags",
            0,
        )?;
        const MODIFY_OPACITY: u64 = 1 << 5;
        Ok(flags & MODIFY_OPACITY != 0)
    }

    fn opacity(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        current: f32,
        amount: f32,
    ) -> Result<f32> {
        let flags = runtime_uint_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "modifierFlags",
            0,
        )?;
        let opacity = runtime_double_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "opacity",
            1.0,
        )?;
        const INVERT_OPACITY: u64 = 1 << 6;
        if flags & INVERT_OPACITY != 0 {
            Ok(current * (1.0 - amount) + opacity * amount)
        } else {
            Ok(current * opacity * amount)
        }
    }

    fn coverage_by_character(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        runs: &[StaticResolvedRun],
        lines: &[StaticTextLine<'_>],
    ) -> Result<Vec<f32>> {
        let mut coverage = vec![0.0; text.chars().count()];
        for range in &self.ranges {
            range.apply_coverage(runtime, instance, text, runs, lines, &mut coverage)?;
        }
        Ok(coverage)
    }
}

// Ported from C++ `src/text/text_follow_path_modifier.cpp`.
impl StaticTextFollowPathModifier {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextFollowPathModifier global {global_id}"))?;
        let target_local = object
            .uint_property("targetId")
            .and_then(|target| usize::try_from(target).ok());
        let paths = match target_local {
            Some(target_local) if type_for_local(graph, target_local) == Some("Shape") => graph
                .path_composers
                .iter()
                .find(|composer| composer.shape_local == target_local)
                .map(|composer| {
                    composer
                        .paths
                        .iter()
                        .filter_map(|path_ref| {
                            graph
                                .paths
                                .iter()
                                .find(|path| path.local_id == path_ref.local_id)
                                .cloned()
                                .map(|geometry| StaticTextFollowPathPath {
                                    local_id: path_ref.local_id,
                                    geometry,
                                })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Some(target_local) => graph
                .paths
                .iter()
                .find(|path| path.local_id == target_local)
                .cloned()
                .map(|geometry| {
                    vec![StaticTextFollowPathPath {
                        local_id: target_local,
                        geometry,
                    }]
                })
                .unwrap_or_default(),
            None => Vec::new(),
        };

        Ok(Self {
            local_id,
            global_id,
            paths,
        })
    }

    fn transform_glyph(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        current: StaticFollowPathGlyphTransform,
        glyph: &StaticTextGlyphContext<'_>,
        offset: (f32, f32),
    ) -> Result<StaticFollowPathGlyphTransform> {
        let path_measure = self.path_measure(instance, glyph.text_world_inverse);
        let path_length = path_measure.length();
        if path_length == 0.0 {
            return Ok(current);
        }

        let position_on_path = (glyph.origin_x + offset.0, glyph.origin_y + offset.1);
        let start = self.double_property(runtime, instance, "start", 0.0)?;
        let end = self.double_property(runtime, instance, "end", 1.0)?;
        let start_pct = start.min(end).clamp(0.0, 1.0);
        let end_pct = start.max(end).clamp(0.0, 1.0);
        let can_wrap = path_measure.raw_is_closed() && (end_pct - start_pct) == 1.0;
        let valid_length = (end_pct - start_pct) * path_length;
        let offset_pct = self
            .double_property(runtime, instance, "offset", 0.0)?
            .rem_euclid(1.0);
        let start_pct = start_pct + offset_pct;
        let end_pct = end_pct + offset_pct;

        let sample = if (!can_wrap && position_on_path.0 < 0.0) || start_pct == end_pct {
            let result = path_measure.at_percentage(start_pct);
            let tangent = normalize_point(result.tan);
            let extra = -position_on_path.0;
            RuntimePathSampleParts {
                position: (
                    result.pos.0 - tangent.0 * extra,
                    result.pos.1 - tangent.1 * extra,
                ),
                tangent,
            }
        } else if !can_wrap && position_on_path.0 > valid_length {
            let result = path_measure.at_percentage(end_pct);
            let tangent = normalize_point(result.tan);
            let extra = position_on_path.0 - valid_length;
            RuntimePathSampleParts {
                position: (
                    result.pos.0 + tangent.0 * extra,
                    result.pos.1 + tangent.1 * extra,
                ),
                tangent,
            }
        } else {
            let result = path_measure.at_percentage(start_pct + position_on_path.0 / path_length);
            RuntimePathSampleParts {
                position: result.pos,
                tangent: normalize_point(result.tan),
            }
        };

        let last_line_index = glyph.line_index_in_paragraph.checked_sub(1);
        let last_baseline = last_line_index
            .and_then(|index| glyph.paragraph_baselines.get(index).copied())
            .unwrap_or(0.0);
        let current_baseline = glyph
            .paragraph_baselines
            .get(glyph.line_index_in_paragraph)
            .copied()
            .unwrap_or(0.0);
        let translation = if self.bool_property(runtime, instance, "radial", false)? {
            let vertical_spacing = position_on_path.1 - current_baseline;
            let perpendicular = (-sample.tangent.1, sample.tangent.0);
            (
                sample.position.0 + vertical_spacing * perpendicular.0,
                sample.position.1 + vertical_spacing * perpendicular.1,
            )
        } else {
            (
                sample.position.0,
                position_on_path.1 - current_baseline + sample.position.1 + last_baseline,
            )
        };
        let rotation = if self.bool_property(runtime, instance, "orient", true)? {
            sample.tangent.1.atan2(sample.tangent.0)
        } else {
            0.0
        };

        let strength = self
            .double_property(runtime, instance, "strength", 1.0)?
            .clamp(0.0, 1.0);
        let inverse_strength = 1.0 - strength;
        Ok(StaticFollowPathGlyphTransform {
            x: translation.0 * strength + current.x * inverse_strength,
            y: translation.1 * strength + current.y * inverse_strength,
            rotation: rotation * strength + current.rotation * inverse_strength,
        })
    }

    fn path_measure(
        &self,
        instance: &ArtboardInstance,
        text_world_inverse: Mat2D,
    ) -> RuntimePathMeasure {
        let mut commands = Vec::new();
        for path in &self.paths {
            let Some(path_world) = instance
                .component(path.local_id)
                .map(|component| component.transform.world_transform)
            else {
                continue;
            };
            let mut path_commands =
                runtime_path_geometry_commands(instance, &path.geometry, path_world);
            transform_path_commands(&mut path_commands, text_world_inverse);
            commands.extend(path_commands);
        }
        RuntimePathMeasure::from_commands_with_tolerance(&commands, 0.1)
    }

    fn double_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: f32,
    ) -> Result<f32> {
        runtime_double_property(
            runtime,
            instance,
            "TextFollowPathModifier",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }

    fn bool_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: bool,
    ) -> Result<bool> {
        runtime_bool_property(
            runtime,
            instance,
            "TextFollowPathModifier",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }
}

#[derive(Debug, Clone, Copy)]
struct RuntimePathSampleParts {
    position: (f32, f32),
    tangent: (f32, f32),
}

fn normalize_point(point: (f32, f32)) -> (f32, f32) {
    let length = (point.0 * point.0 + point.1 * point.1).sqrt();
    if length == 0.0 {
        (0.0, 0.0)
    } else {
        (point.0 / length, point.1 / length)
    }
}

impl StaticTextModifierRange {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierRange global {global_id}"))?;
        let units = object.uint_property("unitsValue").unwrap_or(0);
        if units > 3 {
            bail!("static text subset does not support TextModifierRange units {units}");
        }

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierRange local {local_id} component is missing"))?;
        let mut interpolator = None;
        for child_local in &component.children {
            match type_for_local(graph, *child_local) {
                Some("CubicInterpolatorComponent") => {
                    if interpolator.is_some() {
                        bail!("static text subset supports one TextModifierRange interpolator");
                    }
                    interpolator = Some(StaticCubicInterpolator {
                        local_id: *child_local,
                        global_id: global_for_local(graph, *child_local)?,
                    });
                }
                Some(type_name) => {
                    bail!("static text subset does not support TextModifierRange child {type_name}")
                }
                None => bail!(
                    "static text subset does not support unknown TextModifierRange child local {child_local}"
                ),
            }
        }

        Ok(Self {
            local_id,
            global_id,
            interpolator,
        })
    }

    fn apply_coverage(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        runs: &[StaticResolvedRun],
        lines: &[StaticTextLine<'_>],
        coverage: &mut [f32],
    ) -> Result<()> {
        if coverage.is_empty() {
            return Ok(());
        }
        let (start, end) = self.character_range(runtime, instance, runs, coverage.len())?;
        let units_value = self.uint_property(runtime, instance, "unitsValue", 0)?;
        let units = self.range_units(units_value, text, start, end, lines)?;
        if units.is_empty() {
            return Ok(());
        }
        let unit_count = units.len() as f32;
        let offset = self.double_property(runtime, instance, "offset", 0.0)?;
        let range_type = self.uint_property(runtime, instance, "typeValue", 0)?;
        let (index_from, index_to, falloff_from, falloff_to) = match range_type {
            0 => (
                unit_count * (self.double_property(runtime, instance, "modifyFrom", 0.0)? + offset),
                unit_count * (self.double_property(runtime, instance, "modifyTo", 1.0)? + offset),
                unit_count
                    * (self.double_property(runtime, instance, "falloffFrom", 0.0)? + offset),
                unit_count * (self.double_property(runtime, instance, "falloffTo", 1.0)? + offset),
            ),
            1 => (
                self.double_property(runtime, instance, "modifyFrom", 0.0)? + offset,
                self.double_property(runtime, instance, "modifyTo", 1.0)? + offset,
                self.double_property(runtime, instance, "falloffFrom", 0.0)? + offset,
                self.double_property(runtime, instance, "falloffTo", 1.0)? + offset,
            ),
            other => bail!("static text subset does not support TextModifierRange type {other}"),
        };
        let strength = self.double_property(runtime, instance, "strength", 1.0)?;
        let mode = self.uint_property(runtime, instance, "modeValue", 0)?;
        let clamp = self.bool_property(runtime, instance, "clamp", false)?;

        for (unit_index, unit) in units.iter().enumerate() {
            let t = unit_index as f32 + 0.5;
            let c = strength
                * self.coverage_at(
                    runtime,
                    instance,
                    t,
                    index_from,
                    index_to,
                    falloff_from,
                    falloff_to,
                )?;
            for character_index in unit.start..unit.start + unit.len {
                let current = coverage[character_index];
                let next = match mode {
                    0 => current + c,
                    1 => current - c,
                    2 => current * c,
                    3 => current.min(c),
                    4 => current.max(c),
                    5 => (current - c).abs(),
                    other => {
                        bail!("static text subset does not support TextModifierRange mode {other}")
                    }
                };
                coverage[character_index] = if clamp { next.clamp(0.0, 1.0) } else { next };
            }

            let next_start = units
                .get(unit_index + 1)
                .map(|next| next.start)
                .unwrap_or(end);
            for character_index in unit.start + unit.len..next_start {
                coverage[character_index] = 0.0;
            }
        }

        Ok(())
    }

    fn character_range(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        runs: &[StaticResolvedRun],
        text_len: usize,
    ) -> Result<(usize, usize)> {
        let run_id = self.uint_property(runtime, instance, "runId", u32::MAX as u64)?;
        if run_id == u32::MAX as u64 {
            return Ok((0, text_len));
        }
        let run = runs
            .iter()
            .find(|run| run.local_id as u64 == run_id || run.global_id as u64 == run_id)
            .with_context(|| format!("TextModifierRange runId {run_id} did not resolve"))?;
        Ok((run.char_start, run.char_start + run.char_len))
    }

    fn range_units(
        &self,
        units_value: u64,
        text: &str,
        start: usize,
        end: usize,
        lines: &[StaticTextLine<'_>],
    ) -> Result<Vec<StaticRangeUnit>> {
        match units_value {
            0 => Ok((start..end)
                .map(|index| StaticRangeUnit {
                    start: index,
                    len: 1,
                })
                .collect()),
            1 => Ok(text
                .chars()
                .enumerate()
                .skip(start)
                .take(end.saturating_sub(start))
                .filter_map(|(index, ch)| {
                    (!ch.is_whitespace()).then_some(StaticRangeUnit {
                        start: index,
                        len: 1,
                    })
                })
                .collect()),
            2 => Ok(Self::word_range_units(text, start, end)),
            3 => Ok(Self::line_range_units(lines, start, end)),
            other => bail!("static text subset does not support TextModifierRange units {other}"),
        }
    }

    fn word_range_units(text: &str, start: usize, end: usize) -> Vec<StaticRangeUnit> {
        let mut units = Vec::new();
        let mut word_start = None;
        for (index, ch) in text.chars().enumerate() {
            if ch.is_whitespace() {
                if let Some(index_from) = word_start.take() {
                    add_range_unit(&mut units, index_from, index, start, end);
                }
            } else if word_start.is_none() {
                word_start = Some(index);
            }
        }
        if let Some(index_from) = word_start {
            add_range_unit(&mut units, index_from, text.chars().count(), start, end);
        }
        units
    }

    fn line_range_units(
        lines: &[StaticTextLine<'_>],
        start: usize,
        end: usize,
    ) -> Vec<StaticRangeUnit> {
        let mut units = Vec::new();
        for line in lines {
            add_range_unit(
                &mut units,
                line.char_start,
                line.char_start + line.text.chars().count(),
                start,
                end,
            );
        }
        units
    }

    fn coverage_at(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        t: f32,
        index_from: f32,
        index_to: f32,
        falloff_from: f32,
        falloff_to: f32,
    ) -> Result<f32> {
        let mut c = if index_to < index_from || t < index_from || t > index_to {
            0.0
        } else if t < falloff_from {
            let range = (falloff_from - index_from).max(0.0);
            if range == 0.0 {
                1.0
            } else {
                ((t - index_from).max(0.0) / range).max(0.0)
            }
        } else if t > falloff_to {
            let range = (index_to - falloff_to).max(0.0);
            if range == 0.0 {
                1.0
            } else {
                1.0 - ((t - falloff_to) / range).min(1.0)
            }
        } else {
            1.0
        };
        if c != 0.0
            && c != 1.0
            && let Some(interpolator) = self.interpolator
        {
            c = interpolator.transform(runtime, instance, c)?;
        }
        Ok(c)
    }

    fn double_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: f32,
    ) -> Result<f32> {
        runtime_double_property(
            runtime,
            instance,
            "TextModifierRange",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }

    fn uint_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: u64,
    ) -> Result<u64> {
        runtime_uint_property(
            runtime,
            instance,
            "TextModifierRange",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }

    fn bool_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: bool,
    ) -> Result<bool> {
        runtime_bool_property(
            runtime,
            instance,
            "TextModifierRange",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }
}

impl StaticCubicInterpolator {
    fn transform(
        self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        factor: f32,
    ) -> Result<f32> {
        let x1 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "x1",
            0.42,
        )?;
        let y1 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "y1",
            0.0,
        )?;
        let x2 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "x2",
            0.58,
        )?;
        let y2 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "y2",
            1.0,
        )?;
        let t = cubic_interpolator_get_t(factor, x1, x2);
        Ok(cubic_interpolator_calc_bezier(t, y1, y2))
    }
}

fn runtime_double_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    type_name: &str,
    local_id: usize,
    global_id: u32,
    property_name: &str,
    default: f32,
) -> Result<f32> {
    let property_key = property_key_for_name(type_name, property_name)
        .with_context(|| format!("missing {type_name}.{property_name} key"))?;
    Ok(instance
        .double_property(local_id, property_key)
        .or_else(|| {
            runtime
                .object(global_id as usize)
                .and_then(|object| object.double_property(property_name))
        })
        .unwrap_or(default))
}

fn runtime_uint_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    type_name: &str,
    local_id: usize,
    global_id: u32,
    property_name: &str,
    default: u64,
) -> Result<u64> {
    let property_key = property_key_for_name(type_name, property_name)
        .with_context(|| format!("missing {type_name}.{property_name} key"))?;
    Ok(instance
        .uint_property(local_id, property_key)
        .or_else(|| {
            runtime
                .object(global_id as usize)
                .and_then(|object| object.uint_property(property_name))
        })
        .unwrap_or(default))
}

fn runtime_bool_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    type_name: &str,
    local_id: usize,
    global_id: u32,
    property_name: &str,
    default: bool,
) -> Result<bool> {
    let property_key = property_key_for_name(type_name, property_name)
        .with_context(|| format!("missing {type_name}.{property_name} key"))?;
    Ok(instance
        .bool_property(local_id, property_key)
        .or_else(|| {
            runtime
                .object(global_id as usize)
                .and_then(|object| object.bool_property(property_name))
        })
        .unwrap_or(default))
}

fn character_index_for_cluster(text: &str, cluster: u32) -> usize {
    let cluster = cluster as usize;
    text.char_indices()
        .take_while(|(byte_index, _)| *byte_index <= cluster)
        .count()
        .saturating_sub(1)
}

fn char_slice(text: &str, start: usize, end: usize) -> &str {
    let start_byte = char_byte_index(text, start);
    let end_byte = char_byte_index(text, end);
    &text[start_byte..end_byte]
}

fn char_byte_index(text: &str, char_index: usize) -> usize {
    text.char_indices()
        .nth(char_index)
        .map(|(byte_index, _)| byte_index)
        .unwrap_or(text.len())
}

fn glyph_character_len(text: &str, glyphs: &[TextGlyph], glyph_index: usize) -> usize {
    let char_index = character_index_for_cluster(text, glyphs[glyph_index].cluster);
    let next_char_index = glyphs
        .iter()
        .skip(glyph_index + 1)
        .find_map(|glyph| {
            (glyph.cluster != glyphs[glyph_index].cluster)
                .then_some(character_index_for_cluster(text, glyph.cluster))
        })
        .unwrap_or_else(|| text.chars().count());
    next_char_index.saturating_sub(char_index).max(1)
}

fn glyph_coverage(coverage: &[f32], char_index: usize, char_len: usize) -> f32 {
    let end = (char_index + char_len).min(coverage.len());
    if char_index >= end {
        return 0.0;
    }
    coverage[char_index..end].iter().copied().sum::<f32>() / (end - char_index) as f32
}

fn append_opacity_bucket(
    buckets: &mut Vec<StaticTextPathBucket>,
    opacity: f32,
    commands: Vec<RuntimePathCommand>,
) {
    if opacity <= 0.0 {
        return;
    }
    if let Some(bucket) = buckets.iter_mut().find(|bucket| bucket.opacity == opacity) {
        bucket.commands.extend(commands);
    } else {
        buckets.push(StaticTextPathBucket { opacity, commands });
    }
}

fn transform_path_commands(commands: &mut [RuntimePathCommand], transform: Mat2D) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                (*x, *y) = transform.transform_point(*x, *y);
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                (*x1, *y1) = transform.transform_point(*x1, *y1);
                (*x2, *y2) = transform.transform_point(*x2, *y2);
                (*x3, *y3) = transform.transform_point(*x3, *y3);
            }
            RuntimePathCommand::Close => {}
        }
    }
}

fn order_opacity_buckets_like_cpp(
    mut buckets: Vec<StaticTextPathBucket>,
) -> Vec<StaticTextPathBucket> {
    // rive-runtime 43dfc847 changed TextStylePaint::m_opacityPaths from
    // std::unordered_map<float, ...> to std::map<float, ...>. Modifier
    // falloff paths are now drawn in ascending opacity order.
    buckets.sort_by(|a, b| {
        a.opacity
            .partial_cmp(&b.opacity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    buckets
}

fn split_static_text_lines(text: &str) -> Vec<StaticTextLine<'_>> {
    let mut lines = Vec::new();
    let mut line_start_byte = 0;
    let mut line_start_char = 0;
    let mut line_index = 0;
    let mut iter = text.char_indices().peekable();

    while let Some((byte_index, ch)) = iter.next() {
        if !matches!(ch, '\n' | '\r' | '\u{2028}') {
            continue;
        }

        lines.push(StaticTextLine {
            text: &text[line_start_byte..byte_index],
            char_start: line_start_char,
            line_index,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
        });

        let mut next_start_byte = byte_index + ch.len_utf8();
        let mut separator_chars = 1;
        if ch == '\r'
            && let Some((next_byte_index, '\n')) = iter.peek().copied()
        {
            iter.next();
            next_start_byte = next_byte_index + '\n'.len_utf8();
            separator_chars = 2;
        }

        line_start_char += text[line_start_byte..byte_index].chars().count() + separator_chars;
        line_start_byte = next_start_byte;
        line_index += 1;
    }

    // Static Text matches Rive's line-break contract: a trailing separator
    // does not create another paragraph/GlyphLine. Editable RawTextInput adds
    // its own U+200B sentinel; that is a separate future geometry surface.
    if line_start_byte < text.len() || lines.is_empty() {
        lines.push(StaticTextLine {
            text: &text[line_start_byte..],
            char_start: line_start_char,
            line_index,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
        });
    }
    lines
}

fn static_text_line_iteration(
    overflow: u64,
    sizing: u64,
    vertical_align: u64,
    metrics: StaticTextLineMetrics,
    current_y: f32,
    total_height: f32,
    fixed_height: f32,
) -> StaticTextLineIteration {
    if sizing != TEXT_SIZING_FIXED
        || !matches!(overflow, TEXT_OVERFLOW_HIDDEN | TEXT_OVERFLOW_CLIPPED)
    {
        return StaticTextLineIteration::Draw;
    }

    // Exact `Text::shouldDrawLine` comparisons against the authoritative
    // GlyphLine geometry. Hidden requires the full line; clipped admits a
    // partially intersecting line.
    let line_top = current_y + metrics.top;
    let line_bottom = current_y + metrics.bottom;

    if overflow == TEXT_OVERFLOW_HIDDEN {
        match vertical_align {
            1 if line_top < total_height - fixed_height => StaticTextLineIteration::Skip,
            2 if line_top < total_height / 2.0 - fixed_height / 2.0 => {
                StaticTextLineIteration::Skip
            }
            2 if line_bottom > total_height / 2.0 + fixed_height / 2.0 => {
                StaticTextLineIteration::Stop
            }
            0 if line_bottom > fixed_height => StaticTextLineIteration::Stop,
            _ => StaticTextLineIteration::Draw,
        }
    } else {
        match vertical_align {
            1 if line_bottom < total_height - fixed_height => StaticTextLineIteration::Skip,
            2 if line_bottom < total_height / 2.0 - fixed_height / 2.0 => {
                StaticTextLineIteration::Skip
            }
            2 if line_top > total_height / 2.0 + fixed_height / 2.0 => {
                StaticTextLineIteration::Stop
            }
            0 if line_top > fixed_height => StaticTextLineIteration::Stop,
            _ => StaticTextLineIteration::Draw,
        }
    }
}

fn add_range_unit(
    units: &mut Vec<StaticRangeUnit>,
    index_from: usize,
    index_to: usize,
    start_offset: usize,
    end_offset: usize,
) {
    if index_to > start_offset && end_offset > index_from {
        let actual_start = start_offset.max(index_from);
        let actual_end = end_offset.min(index_to);
        if actual_end > actual_start {
            units.push(StaticRangeUnit {
                start: actual_start,
                len: actual_end - actual_start,
            });
        }
    }
}

fn byte_index_for_glyph_end(text: &str, glyphs: &[TextGlyph], glyph_end: usize) -> usize {
    if glyph_end >= glyphs.len() {
        return text.len();
    }
    let target = (glyphs[glyph_end].cluster as usize).min(text.len());
    if text.is_char_boundary(target) {
        return target;
    }
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= target)
        .last()
        .unwrap_or(0)
}

fn leading_whitespace_bytes(text: &str) -> usize {
    text.char_indices()
        .find_map(|(index, ch)| (!ch.is_whitespace()).then_some(index))
        .unwrap_or(text.len())
}

fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
}

fn cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    const SPLINE_TABLE_SIZE: usize = 11;
    const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
    const NEWTON_ITERATIONS: usize = 4;
    const NEWTON_MIN_SLOPE: f32 = 0.001;
    const SUBDIVISION_PRECISION: f32 = 0.0000001;
    const SUBDIVISION_MAX_ITERATIONS: usize = 10;

    let mut values = [0.0; SPLINE_TABLE_SIZE];
    for (i, value) in values.iter_mut().enumerate() {
        *value = cubic_interpolator_calc_bezier(i as f32 * SAMPLE_STEP_SIZE, x1, x2);
    }

    let mut interval_start = 0.0;
    let mut current_sample = 1;
    let last_sample = SPLINE_TABLE_SIZE - 1;
    while current_sample != last_sample && values[current_sample] <= x {
        interval_start += SAMPLE_STEP_SIZE;
        current_sample += 1;
    }
    current_sample -= 1;

    let dist = (x - values[current_sample]) / (values[current_sample + 1] - values[current_sample]);
    let mut guess_for_t = interval_start + dist * SAMPLE_STEP_SIZE;
    let initial_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
    if initial_slope >= NEWTON_MIN_SLOPE {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
            if current_slope == 0.0 {
                return guess_for_t;
            }
            let current_x = cubic_interpolator_calc_bezier(guess_for_t, x1, x2) - x;
            guess_for_t -= current_x / current_slope;
        }
        guess_for_t
    } else if initial_slope == 0.0 {
        guess_for_t
    } else {
        let mut upper_bound = interval_start + SAMPLE_STEP_SIZE;
        let mut iterations = 0;
        loop {
            let current_t = interval_start + (upper_bound - interval_start) / 2.0;
            let current_x = cubic_interpolator_calc_bezier(current_t, x1, x2) - x;
            if current_x > 0.0 {
                upper_bound = current_t;
            } else {
                interval_start = current_t;
            }
            iterations += 1;
            if current_x.abs() <= SUBDIVISION_PRECISION || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}

#[derive(Clone)]
struct TextGlyph {
    glyph_id: u32,
    cluster: u32,
    advance: f32,
}

#[derive(Clone)]
struct StyledTextGlyph {
    glyph_id: u32,
    char_index: usize,
    char_len: usize,
    style_index: usize,
    advance: f32,
    scale: f32,
}

fn harfbuzz_line_metrics(font: &SkrifaFontRef<'_>, location_ref: LocationRef<'_>) -> (f32, f32) {
    // Mirrors src/text/font_hb.cpp::make_lmx: HarfBuzz scales extents to
    // kStdScale and rounds them before Rive applies the authored font size. Its
    // OpenType funcs use the platform line-metric policy: OS/2 typo metrics
    // only when USE_TYPO_METRICS is set, hhea otherwise, with font fallbacks.
    let metrics = font.metrics(Size::new(TEXT_SHAPE_SCALE_F32), location_ref);
    (metrics.ascent.round(), metrics.descent.round())
}

fn harfbuzz_scaled_glyph_top(raw_edge: f32) -> f32 {
    // C++ asks HarfBuzz for integer glyph extents after scaling to 2048.
    // Skrifa returns exact varied bounds as floats, so keep integral bounds
    // stable and snap fractional varied tops to HarfBuzz's lower integer step.
    let rounded = raw_edge.round();
    if (raw_edge - rounded).abs() <= 1e-4 {
        rounded
    } else {
        raw_edge.trunc() - 1.0
    }
}

fn disable_legacy_kern_for_advances(font: &SkrifaFontRef<'_>) -> bool {
    font.kern().is_ok() && font.gpos().is_err()
}

fn shape_text_glyphs(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    disable_legacy_kern: bool,
) -> Vec<TextGlyph> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.guess_segment_properties();
    let kern_off = [Feature::new(HarfTag::new(b"kern"), 0, ..)];
    let shape_options = ShapeOptions::new().scale(Some(TEXT_SHAPE_SCALE));
    let shape_options = if disable_legacy_kern {
        shape_options.features(&kern_off)
    } else {
        shape_options
    };
    let glyphs = shaper.shape(buffer, shape_options);
    glyphs
        .glyph_infos()
        .iter()
        .zip(glyphs.glyph_positions())
        .map(|(info, position)| TextGlyph {
            glyph_id: info.glyph_id,
            cluster: info.cluster,
            advance: position.x_advance as f32,
        })
        .collect()
}

fn shape_text_glyphs_for_style(
    font_bytes: &[u8],
    style: &StaticTextStyle<'_>,
    text: &str,
) -> Result<Vec<TextGlyph>> {
    let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
    let harf_variations = style
        .variations
        .iter()
        .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
        .collect::<Vec<_>>();
    let shaper_instance = if harf_variations.is_empty() {
        None
    } else {
        Some(ShaperInstance::from_variations(
            &harf_font,
            harf_variations.iter().copied(),
        ))
    };
    let shaper_data = ShaperData::new(&harf_font);
    let shaper = shaper_data
        .shaper(&harf_font)
        .instance(shaper_instance.as_ref())
        .build();
    let skrifa_font = SkrifaFontRef::new(font_bytes).context("failed to parse font for shaping")?;
    Ok(shape_text_glyphs(
        &shaper,
        text,
        disable_legacy_kern_for_advances(&skrifa_font),
    ))
}

trait StaticTextWords {
    fn split_word_bound_indices(&self) -> Vec<(usize, &str)>;
}

impl StaticTextWords for str {
    fn split_word_bound_indices(&self) -> Vec<(usize, &str)> {
        let mut words = Vec::new();
        let mut start = None;
        for (index, ch) in self.char_indices() {
            if ch.is_whitespace() {
                if let Some(word_start) = start.take() {
                    words.push((word_start, &self[word_start..index]));
                }
            } else if start.is_none() {
                start = Some(index);
            }
        }
        if let Some(word_start) = start {
            words.push((word_start, &self[word_start..]));
        }
        words
    }
}

fn text_glyph_width(glyphs: &[TextGlyph], scale: f32, letter_spacing: f32) -> f32 {
    glyphs
        .iter()
        .map(|glyph| glyph.advance * scale + letter_spacing)
        .sum()
}

fn first_fitting_glyph_end(
    glyphs: &[TextGlyph],
    max_width: f32,
    scale: f32,
    letter_spacing: f32,
) -> usize {
    let mut width = 0.0;
    for (index, glyph) in glyphs.iter().enumerate() {
        let advance = glyph.advance * scale + letter_spacing;
        if width + advance > max_width {
            return index.max(1);
        }
        width += advance;
    }
    glyphs.len()
}

fn apply_static_ellipsis(
    glyphs: &mut Vec<StyledTextGlyph>,
    ellipsis: Vec<StyledTextGlyph>,
    max_width: f32,
    force: bool,
) {
    let ellipsis_width = ellipsis.iter().map(|glyph| glyph.advance).sum::<f32>();
    let mut width = 0.0;
    let mut keep = glyphs.len();
    for (index, glyph) in glyphs.iter().enumerate() {
        if width + glyph.advance + ellipsis_width > max_width {
            keep = index;
            break;
        }
        width += glyph.advance;
    }
    if keep < glyphs.len() {
        glyphs.truncate(keep);
        glyphs.extend(ellipsis);
    } else if force {
        glyphs.extend(ellipsis);
    }
}

struct TextOutlinePen {
    commands: Vec<RuntimePathCommand>,
    x: f32,
    y: f32,
    scale: f32,
    center_x: f32,
    center_y: f32,
    transform: Mat2D,
    current: Option<(f32, f32)>,
    contour_start: Option<(f32, f32)>,
}

impl TextOutlinePen {
    fn new(x: f32, y: f32, scale: f32, center_x: f32, center_y: f32, transform: Mat2D) -> Self {
        Self {
            commands: Vec::new(),
            x,
            y,
            scale,
            center_x,
            center_y,
            transform,
            current: None,
            contour_start: None,
        }
    }

    fn map(&self, x: f32, y: f32) -> (f32, f32) {
        let point = (self.x + x * self.scale, self.y - y * self.scale);
        let transformed = self
            .transform
            .transform_point(point.0 - self.center_x, point.1 - self.center_y);
        (self.center_x + transformed.0, self.center_y + transformed.1)
    }
}

impl OutlinePen for TextOutlinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        let point = self.map(x, y);
        self.commands.push(RuntimePathCommand::Move {
            x: point.0,
            y: point.1,
        });
        self.current = Some(point);
        self.contour_start = Some(point);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if self.scale == 0.0 {
            // C++ RawPath collapses zero-size glyph contours to move/close pairs.
            return;
        }
        let point = self.map(x, y);
        self.commands.push(RuntimePathCommand::Line {
            x: point.0,
            y: point.1,
        });
        self.current = Some(point);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        if self.scale == 0.0 {
            return;
        }
        let Some(current) = self.current else {
            self.move_to(x, y);
            return;
        };
        let control = self.map(cx0, cy0);
        let end = self.map(x, y);
        let x1 = current.0 + (control.0 - current.0) * (2.0 / 3.0);
        let y1 = current.1 + (control.1 - current.1) * (2.0 / 3.0);
        let x2 = end.0 + (control.0 - end.0) * (2.0 / 3.0);
        let y2 = end.1 + (control.1 - end.1) * (2.0 / 3.0);
        self.commands.push(RuntimePathCommand::Cubic {
            x1,
            y1,
            x2,
            y2,
            x3: end.0,
            y3: end.1,
        });
        self.current = Some(end);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        if self.scale == 0.0 {
            return;
        }
        let control0 = self.map(cx0, cy0);
        let control1 = self.map(cx1, cy1);
        let end = self.map(x, y);
        self.commands.push(RuntimePathCommand::Cubic {
            x1: control0.0,
            y1: control0.1,
            x2: control1.0,
            y2: control1.1,
            x3: end.0,
            y3: end.1,
        });
        self.current = Some(end);
    }

    fn close(&mut self) {
        if let (Some(current), Some(start)) = (self.current, self.contour_start)
            && ((current.0 - start.0).abs() > f32::EPSILON
                || (current.1 - start.1).abs() > f32::EPSILON)
        {
            self.commands.push(RuntimePathCommand::Line {
                x: start.0,
                y: start.1,
            });
        }
        self.commands.push(RuntimePathCommand::Close);
        self.current = self.contour_start;
    }
}

fn global_for_local(graph: &ArtboardGraph, local_id: usize) -> Result<u32> {
    graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)
        .map(|object| object.global_id)
        .with_context(|| format!("missing local object {local_id}"))
}

fn type_for_local(graph: &ArtboardGraph, local_id: usize) -> Option<&str> {
    graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)
        .and_then(|object| object.type_name)
}

fn component_for_local(
    graph: &ArtboardGraph,
    local_id: usize,
) -> Option<&nuxie_graph::ComponentNode> {
    graph
        .components
        .iter()
        .find(|component| component.local_id == local_id)
}

fn static_text_parent_chain_supported(
    graph: &ArtboardGraph,
    mut parent_local: Option<usize>,
) -> bool {
    while let Some(local_id) = parent_local {
        if local_id == 0 {
            return true;
        }
        if !matches!(
            type_for_local(graph, local_id),
            Some("Node" | "Shape" | "LayoutComponent" | "Solo")
        ) {
            return false;
        }
        let Some(component) = component_for_local(graph, local_id) else {
            return false;
        };
        parent_local = component.parent_local;
    }
    false
}

fn embedded_file_asset_bytes(runtime: &RuntimeFile, asset_global: u32) -> Option<&[u8]> {
    let file_asset_globals = runtime
        .file_assets()
        .into_iter()
        .map(|asset| asset.id)
        .collect::<BTreeSet<_>>();
    let mut after_asset = false;
    for object in runtime.objects.iter().flatten() {
        if object.id == asset_global {
            after_asset = true;
            continue;
        }
        if !after_asset {
            continue;
        }
        if file_asset_globals.contains(&object.id) {
            return None;
        }
        if object.type_name == "FileAssetContents" {
            return object.bytes_property("bytes");
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};
    use nuxie_graph::GraphFile;

    fn authoring_record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, property_name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}")),
            value,
        }
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn fixture_font_bytes() -> Vec<u8> {
        let mut accumulator = 0u32;
        let mut bit_count = 0u8;
        let mut decoded = Vec::new();
        for byte in include_bytes!("../../nuxie/tests/fixtures/roboto-a.ttf.base64")
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
        {
            if byte == b'=' {
                break;
            }
            let value = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => panic!("invalid base64 font fixture"),
            };
            accumulator = (accumulator << 6) | u32::from(value);
            bit_count += 6;
            if bit_count >= 8 {
                bit_count -= 8;
                decoded.push((accumulator >> bit_count) as u8);
                accumulator &= (1u32 << bit_count) - 1;
            }
        }
        decoded
    }

    fn baseline_origin_text_runtime_with_sizing(sizing_value: u64) -> (RuntimeFile, GraphFile) {
        let records = vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record(
                "FontAsset",
                vec![property("FontAsset", "assetId", AuthoringValue::Uint(0))],
            ),
            authoring_record(
                "FileAssetContents",
                vec![property(
                    "FileAssetContents",
                    "bytes",
                    AuthoringValue::Bytes(fixture_font_bytes()),
                )],
            ),
            authoring_record(
                "Artboard",
                vec![
                    property("Artboard", "width", AuthoringValue::Double(200.0)),
                    property("Artboard", "height", AuthoringValue::Double(100.0)),
                ],
            ),
            authoring_record(
                "Text",
                vec![
                    property("Text", "sizingValue", AuthoringValue::Uint(sizing_value)),
                    property("Text", "width", AuthoringValue::Double(80.0)),
                    property("Text", "height", AuthoringValue::Double(50.0)),
                    property(
                        "Text",
                        "overflowValue",
                        AuthoringValue::Uint(TEXT_OVERFLOW_CLIPPED),
                    ),
                    property("Text", "verticalAlignValue", AuthoringValue::Uint(1)),
                    property("Text", "originValue", AuthoringValue::Uint(1)),
                    property("Text", "originX", AuthoringValue::Double(0.25)),
                    property("Text", "originY", AuthoringValue::Double(0.5)),
                ],
            ),
            authoring_record(
                "TextStylePaint",
                vec![
                    property("TextStylePaint", "parentId", AuthoringValue::Uint(1)),
                    property("TextStylePaint", "fontSize", AuthoringValue::Double(20.0)),
                    property("TextStylePaint", "lineHeight", AuthoringValue::Double(40.0)),
                    property("TextStylePaint", "fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            authoring_record(
                "TextValueRun",
                vec![
                    property("TextValueRun", "parentId", AuthoringValue::Uint(1)),
                    property(
                        "TextValueRun",
                        "text",
                        AuthoringValue::String("a\na".to_owned()),
                    ),
                    property("TextValueRun", "styleId", AuthoringValue::Uint(2)),
                ],
            ),
        ];
        let runtime = RuntimeFile::from_authoring_records(records)
            .expect("baseline-origin Text records import");
        let graph =
            GraphFile::from_runtime_file(&runtime).expect("baseline-origin Text graph builds");
        (runtime, graph)
    }

    fn baseline_origin_text_runtime() -> (RuntimeFile, GraphFile) {
        baseline_origin_text_runtime_with_sizing(TEXT_SIZING_FIXED)
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn baseline_origin_and_raw_clip_share_authoritative_line_metrics() {
        let (runtime, graphs) = baseline_origin_text_runtime();
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).expect("Text slice builds");
        let runs = slice
            .resolved_runs(&runtime, &instance)
            .expect("Text runs resolve");
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let lines = split_static_text_lines(&text);
        let half_size_metrics = slice
            .static_line_metrics(&runtime, &instance, &lines, &runs, 0.5)
            .expect("half-size line metrics compute");
        let first = half_size_metrics.first().expect("first line metric");
        let second = half_size_metrics.get(1).expect("second line metric");
        assert_close(first.baseline, 9.277344);
        assert_close(second.baseline - first.baseline, 40.0);

        let local_bounds = slice
            .local_bounds(&runtime, &instance)
            .expect("Text bounds compute")
            .expect("Text has bounds");

        assert_close(local_bounds.0, -20.0);
        assert_close(local_bounds.1, -43.554688);
        assert_close(local_bounds.2, 80.0);
        assert_close(local_bounds.3, 50.0);

        let clip = slice
            .clip_bounds(&runtime, &instance, None)
            .expect("clip computes")
            .expect("clipped Text has a clip");
        assert_close(clip.bounds.0, 0.0);
        assert_close(clip.bounds.2, 80.0);
        assert_close(clip.bounds.3, 50.0);
        assert_ne!(clip.bounds.0, local_bounds.0, "raw clip cancels originX");

        let clip_top_left = clip
            .local_transform
            .transform_point(clip.bounds.0, clip.bounds.1);
        let clip_bottom_right = clip
            .local_transform
            .transform_point(clip.bounds.0 + clip.bounds.2, clip.bounds.1 + clip.bounds.3);
        assert_close(clip_top_left.0, local_bounds.0);
        assert_close(clip_top_left.1, local_bounds.1);
        assert_close(clip_bottom_right.0, local_bounds.0 + local_bounds.2);
        assert_close(clip_bottom_right.1, local_bounds.1 + local_bounds.3);
    }

    #[test]
    fn layout_measure_uses_authored_sizing_before_controlled_bounds() {
        let (runtime, graphs) = baseline_origin_text_runtime_with_sizing(TEXT_SIZING_AUTO_HEIGHT);
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).expect("Text slice builds");
        let constraint = RuntimeTextLayoutConstraint {
            width: 200.0,
            height: 100.0,
            width_scale_type: 0,
            height_scale_type: 0,
        };

        let measured = slice
            .measure_bounds_with_layout_constraint(&runtime, &instance, constraint)
            .expect("Text measure computes")
            .expect("Text has measured bounds");
        let controlled = slice
            .local_bounds_with_layout_constraint(&runtime, &instance, constraint)
            .expect("controlled Text bounds compute")
            .expect("Text has controlled bounds");

        assert_close(measured.2, 80.0);
        assert_close(controlled.2, 200.0);
    }

    #[test]
    fn caret_candidate_index_visits_each_glyph_and_boundary_once_for_one_line() {
        const CHARACTER_COUNT: usize = 4_096;
        let text = "a".repeat(CHARACTER_COUNT);
        let glyphs = (0..CHARACTER_COUNT)
            .map(|char_index| StaticPositionedTextGlyph {
                glyph: StyledTextGlyph {
                    glyph_id: 1,
                    char_index,
                    char_len: 1,
                    style_index: 0,
                    advance: 1.0,
                    scale: 1.0,
                },
                x: char_index as f32,
                modifier_transform: Mat2D::IDENTITY,
                modifier_opacity: 1.0,
            })
            .collect();
        let lines = vec![StaticShapedTextLine {
            line_index: 0,
            char_start: 0,
            char_end: CHARACTER_COUNT,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
            start_x: 0.0,
            end_x: CHARACTER_COUNT as f32,
            top: 0.0,
            baseline: 10.0,
            bottom: 12.0,
            glyphs,
        }];

        let (boundaries, work) = build_static_caret_boundaries(&text, &lines, Mat2D::IDENTITY);
        assert_eq!(work.glyph_visits, CHARACTER_COUNT);
        assert_eq!(work.boundary_visits, CHARACTER_COUNT + 1);
        assert_eq!(boundaries.len(), CHARACTER_COUNT + 1);
        assert!(
            boundaries
                .iter()
                .all(|boundary| { boundary.upstream.is_some() && boundary.downstream.is_some() })
        );
    }

    #[test]
    fn backtracking_cluster_uses_final_cursor_and_retains_visual_extrema() {
        let glyph = |x, advance| StaticPositionedTextGlyph {
            glyph: StyledTextGlyph {
                glyph_id: 1,
                char_index: 0,
                char_len: 2,
                style_index: 0,
                advance,
                scale: 1.0,
            },
            x,
            modifier_transform: Mat2D::IDENTITY,
            modifier_opacity: 1.0,
        };
        let line = StaticShapedTextLine {
            line_index: 0,
            char_start: 0,
            char_end: 2,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
            start_x: 4.0,
            end_x: 9.0,
            top: 0.0,
            baseline: 10.0,
            bottom: 12.0,
            glyphs: vec![glyph(4.0, 8.0), glyph(12.0, -3.0)],
        };

        assert_close(line.caret_x(1), 9.0);
        assert_close(line.caret_x(2), 9.0);
        let selection = line
            .selection_rect(0, 2, Mat2D::IDENTITY, false, true)
            .expect("the whole cluster has visual geometry");
        assert_close(selection.min_x, 4.0);
        assert_close(selection.max_x, 12.0);

        let (boundaries, _) = build_static_caret_boundaries("ab", &[line], Mat2D::IDENTITY);
        let internal = boundaries[1]
            .downstream
            .expect("the internal cluster boundary has a caret");
        let end = boundaries[2]
            .downstream
            .expect("the cluster end has a caret");
        assert_close(internal.top.x, 9.0);
        assert_eq!(internal, end);
    }

    #[test]
    fn render_layout_does_not_materialize_the_geometry_candidate_index() {
        let (runtime, graphs) = baseline_origin_text_runtime();
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).expect("Text slice builds");
        let runs = slice
            .resolved_runs(&runtime, &instance)
            .expect("Text runs resolve");

        let render = slice
            .shaped_layout_from_resolved_runs(
                &runtime,
                &instance,
                None,
                Mat2D::IDENTITY,
                &runs,
                StaticShapedTextPurpose::Render,
            )
            .expect("render layout shapes")
            .expect("render layout exists");
        let geometry = slice
            .shaped_layout_from_resolved_runs(
                &runtime,
                &instance,
                None,
                Mat2D::IDENTITY,
                &runs,
                StaticShapedTextPurpose::Geometry,
            )
            .expect("geometry layout shapes")
            .expect("geometry layout exists");

        assert!(render.caret_boundaries.is_none());
        assert!(geometry.caret_boundaries.is_some());
    }

    #[test]
    fn glyph_modifier_context_borrows_shared_paragraph_baselines() {
        let baselines = vec![10.0, 30.0, 50.0];
        let context = StaticTextGlyphContext {
            origin_x: 0.0,
            origin_y: 0.0,
            line_index_in_paragraph: 1,
            paragraph_baselines: &baselines,
            text_world_inverse: Mat2D::IDENTITY,
        };

        assert!(std::ptr::eq(
            context.paragraph_baselines,
            baselines.as_slice()
        ));
    }

    #[test]
    fn opacity_buckets_follow_cpp_ordered_map_iteration() {
        let buckets = [0.8, 0.2, 0.5]
            .into_iter()
            .map(|opacity| StaticTextPathBucket {
                opacity,
                commands: Vec::new(),
            })
            .collect();

        let ordered = order_opacity_buckets_like_cpp(buckets)
            .into_iter()
            .map(|bucket| bucket.opacity)
            .collect::<Vec<_>>();

        assert_eq!(ordered, vec![0.2, 0.5, 0.8]);
    }
}
