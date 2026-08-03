use anyhow::{Context, Result, bail};
use harfrust::{
    Direction, Feature, FontRef as HarfFontRef, Script as HarfScript, ShapeOptions, ShaperData,
    ShaperInstance, Tag as HarfTag, UnicodeBuffer,
};
use nuxie_binary::RuntimeFile;
use nuxie_graph::{
    ArtboardGraph, DataBindNode, PathGeometryNode, ShapePaintContainerNode, ShapePaintKind,
    ShapePaintStateNode,
};
use nuxie_render_api::{Aabb as RenderAabb, Vec2D as RenderVec2D};
use nuxie_schema::definition_by_name;
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::pen::{NullPen, PathStyle};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::setting::VariationSetting;
use skrifa::{FontRef as SkrifaFontRef, GlyphId, MetadataProvider, Tag as SkrifaTag};
use std::collections::BTreeSet;
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_script::{Script as UnicodeScript, UnicodeScript as UnicodeScriptProperty};

use crate::data_bind_flags_apply_source_to_target;
use crate::draw::{
    RuntimeLayoutBounds, RuntimePathMeasure, RuntimeTextPaintPoolSpec, RuntimeTextPaintPoolUse,
    runtime_live_shape_paint_path_kind, runtime_path_geometry_commands,
    runtime_shape_paint_command,
};
use crate::joystick::{joystick_x_property_key, joystick_y_property_key};
use crate::properties::{property_key_for_name, solid_color_value_property_key};
use crate::view_model::RuntimeFontAssetValue;
use crate::{ArtboardInstance, Mat2D, RuntimePathCommand};
use crate::{RuntimeShapePaintCommand, RuntimeShapePaintKind, RuntimeShapePaintPathKind};
use std::collections::BTreeMap;

pub(crate) mod cursor;
pub(crate) mod raw_text_input;
pub(crate) use raw_text_input::{TextInputGeometry, build_text_input_geometry};
mod text_input_cursor;
mod text_input_drawable;
pub(crate) mod text_input_selected_text;
mod text_input_selection;
mod text_input_text;
pub(crate) mod text_interface;
pub(crate) mod text_selection_path;

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

include!("text/font_hb.rs");

pub fn static_text_support_error(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    text_local: usize,
) -> Option<String> {
    StaticTextSlice::from_graph(runtime, graph, text_local)
        .err()
        .map(|error| error.to_string())
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTextLayoutDebugReport {
    pub text: String,
    pub paragraph_count: usize,
    pub run_count: usize,
    pub line_glyph_ids: Vec<Vec<u32>>,
    pub glyph_lookup_counts: Vec<usize>,
    pub font_size: Option<f32>,
    pub local_transform: [f32; 6],
    pub style_features: Vec<Vec<(u32, u32)>>,
    pub line_glyph_variations: Vec<Vec<Vec<(u32, f32)>>>,
    pub modifier_groups: Vec<RuntimeTextModifierDebugReport>,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeTextModifierDebugReport {
    pub range_count: usize,
    pub coverage: Vec<f32>,
    pub selected_runs: Vec<Option<RuntimeTextSelectedRunDebugReport>>,
    pub modifier_locals: Vec<usize>,
    pub shape_modifier_indices: Vec<usize>,
    pub targets: Vec<RuntimeTextTargetModifierDebugReport>,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTextTargetModifierDebugReport {
    pub target_id: u32,
    pub resolved: bool,
    pub has_text_component: bool,
}

pub(crate) fn static_text_target_debug_report(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    text_local: usize,
) -> Vec<RuntimeTextTargetModifierDebugReport> {
    let Some(text) =
        component_for_local(graph, text_local).filter(|component| component.type_name == "Text")
    else {
        return Vec::new();
    };
    text.children
        .iter()
        .copied()
        .filter(|local| type_for_local(graph, *local) == Some("TextModifierGroup"))
        .filter_map(|group_local| component_for_local(graph, group_local))
        .flat_map(|group| {
            group.children.iter().copied().filter_map(move |local| {
                let type_name = type_for_local(graph, local)?;
                if !nuxie_schema::definition_by_name(type_name)
                    .is_some_and(|definition| definition.is_a("TextTargetModifier"))
                {
                    return None;
                }
                let (target_id, resolved) = if type_name == "TextFollowPathModifier" {
                    let target =
                        StaticTextFollowPathModifier::from_graph(runtime, graph, local).ok()?;
                    (target.target_id, target.resolved_transform_local.is_some())
                } else {
                    let target =
                        StaticTextTargetModifier::from_graph(runtime, graph, local, group.local_id)
                            .ok()?;
                    (target.target_id, target.resolved_transform_local.is_some())
                };
                Some(RuntimeTextTargetModifierDebugReport {
                    target_id,
                    resolved,
                    has_text_component: true,
                })
            })
        })
        .collect()
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeTextSelectedRunDebugReport {
    pub offset: usize,
    pub length: usize,
    pub byte_length: usize,
}

pub(crate) fn static_text_layout_debug_report(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Option<RuntimeTextLayoutDebugReport> {
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local).ok()?;
    let resolved = slice.resolved_runs(runtime, instance).ok()?;
    let text = resolved
        .iter()
        .map(|run| run.text.as_str())
        .collect::<String>();
    let layout = slice
        .shaped_layout(runtime, instance, layout_constraint, Mat2D::IDENTITY)
        .ok()?;
    let source_lines = split_static_text_lines(&text);
    let modifier_groups = slice
        .modifiers
        .iter()
        .map(|group| {
            let coverage = group
                .coverage_by_character(runtime, instance, &text, &resolved, &source_lines)
                .ok()?;
            let selected_runs = group
                .ranges
                .iter()
                .map(|range| {
                    let run_id = range
                        .uint_property(runtime, instance, "runId", u32::MAX as u64)
                        .ok()?;
                    if run_id == u32::MAX as u64 {
                        return Some(None);
                    }
                    let run = resolved.iter().find(|run| {
                        run.local_id as u64 == run_id || run.global_id as u64 == run_id
                    })?;
                    Some(Some(RuntimeTextSelectedRunDebugReport {
                        offset: run.char_start,
                        length: run.char_len,
                        byte_length: run.text.len(),
                    }))
                })
                .collect::<Option<Vec<_>>>()?;
            Some(RuntimeTextModifierDebugReport {
                range_count: group.ranges.len(),
                coverage,
                selected_runs,
                modifier_locals: group
                    .modifiers
                    .iter()
                    .map(StaticTextModifier::local_id)
                    .collect(),
                shape_modifier_indices: group.shape_modifier_indices.clone(),
                targets: group
                    .modifiers
                    .iter()
                    .filter_map(|modifier| match modifier {
                        StaticTextModifier::Target(target) => {
                            Some(RuntimeTextTargetModifierDebugReport {
                                target_id: target.target_id,
                                resolved: target.resolved_transform_local.is_some(),
                                has_text_component: true,
                            })
                        }
                        StaticTextModifier::FollowPath(target) => {
                            Some(RuntimeTextTargetModifierDebugReport {
                                target_id: target.target_id,
                                resolved: target.resolved_transform_local.is_some(),
                                has_text_component: true,
                            })
                        }
                        _ => None,
                    })
                    .collect(),
            })
        })
        .collect::<Option<Vec<_>>>()?;
    let Some(layout) = layout else {
        let style_features = slice
            .styles
            .iter()
            .map(|style| {
                style
                    .features
                    .iter()
                    .map(|feature| feature.option(instance))
                    .collect()
            })
            .collect();
        return Some(RuntimeTextLayoutDebugReport {
            text,
            paragraph_count: 0,
            run_count: resolved.len(),
            line_glyph_ids: Vec::new(),
            glyph_lookup_counts: Vec::new(),
            font_size: None,
            local_transform: Mat2D::IDENTITY.0,
            style_features,
            line_glyph_variations: Vec::new(),
            modifier_groups,
        });
    };
    let mut glyph_lookup_counts = vec![0; text.chars().count()];
    for glyph in layout.lines.iter().flat_map(|line| &line.glyphs) {
        if let Some(count) = glyph_lookup_counts.get_mut(glyph.glyph.char_index) {
            *count = glyph.glyph.char_len;
        }
    }
    let font_size = layout
        .lines
        .iter()
        .flat_map(|line| &line.glyphs)
        .next()
        .map(|glyph| glyph.glyph.scale * TEXT_SHAPE_SCALE_F32);
    let style_features = slice
        .styles
        .iter()
        .map(|style| {
            style
                .features
                .iter()
                .map(|feature| feature.option(instance))
                .collect()
        })
        .collect();
    Some(RuntimeTextLayoutDebugReport {
        paragraph_count: (!text.is_empty())
            .then(|| text.split('\n').count())
            .unwrap_or(0),
        run_count: resolved.len(),
        line_glyph_ids: layout
            .lines
            .iter()
            .map(|line| {
                line.glyphs
                    .iter()
                    .map(|glyph| glyph.glyph.glyph_id)
                    .collect()
            })
            .collect(),
        glyph_lookup_counts,
        font_size,
        local_transform: layout.local_transform.0,
        style_features,
        line_glyph_variations: layout
            .lines
            .iter()
            .map(|line| {
                line.glyphs
                    .iter()
                    .map(|glyph| glyph.glyph.variations.clone())
                    .collect()
            })
            .collect(),
        modifier_groups,
        text,
    })
}

#[doc(hidden)]
pub fn debug_text_word_unit_count(text: &str) -> usize {
    text.split_word_bound_indices().len()
}

include!("text/text_engine.rs");

include!("text/raw_text.rs");

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

/// Test one authored `TextValueRun` against the glyph advance rectangles that
/// C++ records in `Text::shape` for `TextValueRun::hitTestPoint`.
pub(crate) fn static_text_value_run_hit(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    run_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
    text_world: Mat2D,
    point: RenderVec2D,
    hit_radius: f32,
) -> bool {
    let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local) else {
        return false;
    };
    if !slice
        .text_geometry_supported(runtime, instance)
        .unwrap_or(false)
    {
        return false;
    }
    let Ok(runs) = slice.resolved_runs(runtime, instance) else {
        return false;
    };
    let Some(run) = runs.iter().find(|run| run.local_id == run_local) else {
        return false;
    };
    let Ok(Some(layout)) = slice.shaped_layout_from_resolved_runs(
        runtime,
        instance,
        layout_constraint,
        text_world,
        &runs,
        StaticShapedTextPurpose::Geometry,
    ) else {
        return false;
    };
    let start = char_byte_index(&layout.text, run.char_start);
    let end = char_byte_index(&layout.text, run.char_start.saturating_add(run.char_len));
    layout
        .selection_rects(start..end)
        .into_iter()
        .any(|bounds| {
            point.x >= bounds.min_x - hit_radius
                && point.x <= bounds.max_x + hit_radius
                && point.y >= bounds.min_y - hit_radius
                && point.y <= bounds.max_y + hit_radius
        })
}

#[allow(clippy::arithmetic_side_effects)]
pub(crate) fn static_fixed_text_constraint_bounds(
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
    if layout_constraint.is_none() && effective_sizing != TEXT_SIZING_FIXED {
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
    let state = instance
        .component(text_input_local)?
        .concrete
        .text_input
        .as_ref()?;
    if let Some(bounds) = state
        .raw
        .borrow()
        .cached_measure(layout_constraint.width, layout_constraint.height)
    {
        return Some(bounds);
    }
    let bounds = StaticTextSlice::from_text_input_graph(runtime, graph, text_input_local)
        .ok()?
        .measure_bounds_with_layout_constraint(runtime, instance, layout_constraint)
        .ok()
        .flatten()?;
    state.raw.borrow_mut().retain_measure(
        layout_constraint.width,
        layout_constraint.height,
        bounds,
    );
    Some(bounds)
}

pub(crate) fn runtime_text_shape_paint_commands(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    text_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Result<Vec<RuntimeShapePaintCommand>> {
    Ok(runtime_text_draw_data(
        runtime,
        instance,
        graph,
        text_local,
        layout_bounds,
        layout_constraint,
    )?
    .commands)
}

pub(crate) fn runtime_text_draw_data(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    text_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Result<RuntimeTextDrawData> {
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
        && render_data.color_glyphs.is_empty()
    {
        return Ok(RuntimeTextDrawData::default());
    }
    let shape_world = text_world.multiply(render_data.local_transform);
    // C++ text draw isolates the glyph path transform even when clipping
    // elides the drawable-level save.
    let needs_save_operation = true;
    let text_blend_mode_value = slice.text_blend_mode_value(runtime, instance)?;
    let style_order = slice.ordered_style_indices(runtime, instance)?;

    let mut commands = Vec::new();
    let mut next_path_bucket_slot = 0usize;
    for style_index in style_order {
        let style = &slice.styles[style_index];
        let path_buckets = render_data.path_buckets_by_style[style_index].clone();
        let Some(container) = style.container else {
            continue;
        };
        let path_buckets = order_opacity_buckets_like_cpp(path_buckets);
        let paint_pool = RuntimeTextPaintPoolSpec {
            style_local: style.local_id,
            // C++ reserves one pooled paint per opacity bucket, including the
            // opaque bucket even though that bucket uses the authored paint.
            paint_count: path_buckets.len(),
        };
        let path_bucket_slot_start = next_path_bucket_slot;
        next_path_bucket_slot += path_buckets.len();
        let opaque_bucket = path_buckets
            .iter()
            .enumerate()
            .find(|(_, path_bucket)| path_bucket.opacity == 1.0);
        for paint in &container.paints {
            if let Some((bucket_index, path_bucket)) = opaque_bucket {
                let mut path_commands = path_bucket.commands.clone();
                if runtime_live_shape_paint_path_kind(instance, paint)
                    == Some(RuntimeShapePaintPathKind::World)
                {
                    transform_path_commands(&mut path_commands, shape_world);
                }
                if let Some(mut command) = runtime_shape_paint_command(
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
                ) {
                    command.shape_world_override = Some(shape_world);
                    if command.paint_type == RuntimeShapePaintKind::Fill {
                        command.path_kind = RuntimeShapePaintPathKind::LocalClockwise;
                    }
                    command.text_path_bucket_slot = Some(path_bucket_slot_start + bucket_index);
                    command.ensure_text_paint_pool_after_draw = Some(paint_pool);
                    commands.push(command);
                }
            }

            for (paint_index, (bucket_index, path_bucket)) in path_buckets
                .iter()
                .enumerate()
                .filter(|(_, path_bucket)| path_bucket.opacity != 1.0)
                .enumerate()
            {
                let mut path_commands = path_bucket.commands.clone();
                if runtime_live_shape_paint_path_kind(instance, paint)
                    == Some(RuntimeShapePaintPathKind::World)
                {
                    transform_path_commands(&mut path_commands, shape_world);
                }
                let Some(mut command) = runtime_shape_paint_command(
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
                ) else {
                    continue;
                };
                command.shape_world_override = Some(shape_world);
                if command.paint_type == RuntimeShapePaintKind::Fill {
                    command.path_kind = RuntimeShapePaintPathKind::LocalClockwise;
                }
                command.uses_temporary_paint = true;
                command.text_path_bucket_slot = Some(path_bucket_slot_start + bucket_index);
                command.text_paint_pool = Some(RuntimeTextPaintPoolUse {
                    spec: paint_pool,
                    paint_index,
                });
                commands.push(command);
            }
        }
    }
    Ok(RuntimeTextDrawData {
        commands,
        color_glyphs: render_data.color_glyphs,
        order: render_data.order,
        shape_world,
    })
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
    if !text_input_drawable::is_concrete(drawable_component.type_name)
        || !text_input_drawable::valid_parent(type_for_local(graph, text_input_local))
    {
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
    if !text_input_drawable::will_draw(true, render_opacity) {
        return Ok(Vec::new());
    }
    let needs_save_operation = true;

    let path_buckets = match container.type_name {
        text_input_text::TYPE_NAME | "TextInputSelectedText" => {
            let selection = instance.text_input_selection_range(text_input_local);
            let filter = match container.type_name {
                "TextInputSelectedText" => Some((selection.unwrap_or(0..0), true)),
                text_input_text::TYPE_NAME
                    if instance.text_input_separates_selection_text(text_input_local) =>
                {
                    selection.map(|range| (range, false))
                }
                _ => None,
            };
            let render_data = slice.render_data_filtered(
                runtime,
                instance,
                layout_constraint,
                text_input_world,
                filter,
            )?;
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
        "TextInputCursor" => {
            if !instance.text_input_is_focused(text_input_local) {
                return Ok(Vec::new());
            }
            vec![StaticTextPathBucket {
                opacity: 1.0,
                commands: text_input_cursor::local_clockwise_path(
                    instance,
                    text_input_local,
                    slice.text_input_fallback_cursor_height(runtime, instance)?,
                ),
            }]
        }
        "TextInputSelection" => vec![StaticTextPathBucket {
            opacity: 1.0,
            commands: text_input_selection::local_clockwise_path(instance, text_input_local),
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
    font_asset_global: Option<u32>,
    font_asset_id: Option<u32>,
    font_bytes: Option<&'a [u8]>,
    variations: Vec<StaticTextVariation>,
    features: Vec<StaticTextStyleFeature>,
}

include!("text/text_style_axis.rs");

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
    color_glyphs: Vec<RuntimeIntegratedColorGlyphCommand>,
    order: Vec<RuntimeTextDrawOrder>,
    local_transform: Mat2D,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTextDrawData {
    pub(crate) commands: Vec<RuntimeShapePaintCommand>,
    pub(crate) color_glyphs: Vec<RuntimeIntegratedColorGlyphCommand>,
    pub(crate) order: Vec<RuntimeTextDrawOrder>,
    pub(crate) shape_world: Mat2D,
}

impl Default for RuntimeTextDrawData {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            color_glyphs: Vec::new(),
            order: Vec::new(),
            shape_world: Mat2D::IDENTITY,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeTextDrawOrder {
    Style(usize),
    ColorGlyph(usize),
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeIntegratedColorGlyphCommand {
    pub(crate) font_identity: usize,
    pub(crate) glyph_id: u32,
    pub(crate) transform: Mat2D,
    pub(crate) opacity: f32,
    pub(crate) layers: Vec<RuntimeColorGlyphLayer>,
}

include!("text/fully_shaped_text.rs");

fn paragraph_char_range(chars: &[char], at: usize) -> std::ops::Range<usize> {
    let at = at.min(chars.len());
    let start = chars[..at]
        .iter()
        .rposition(|character| *character == '\n')
        .map_or(0, |index| index + 1);
    let end = chars[at..]
        .iter()
        .position(|character| *character == '\n')
        .map_or(chars.len(), |index| at + index);
    start..end
}

fn paragraph_base_is_rtl(text: &str, at: usize) -> bool {
    let chars = text.chars().collect::<Vec<_>>();
    let range = paragraph_char_range(&chars, at);
    let paragraph = chars[range].iter().collect::<String>();
    unicode_bidi::BidiInfo::new(&paragraph, None)
        .paragraphs
        .first()
        .is_some_and(|paragraph| paragraph.level.is_rtl())
}

fn text_has_rtl(text: &str) -> bool {
    unicode_bidi::BidiInfo::new(text, None).has_rtl()
}

fn reorder_text_input_bidi_geometry(text: &str, lines: &mut [StaticShapedTextLine]) {
    let bidi = unicode_bidi::BidiInfo::new(text, None);
    for line in lines {
        let mut clusters: Vec<Vec<StaticPositionedTextGlyph>> = Vec::new();
        for glyph in std::mem::take(&mut line.glyphs) {
            if let Some(cluster) = clusters.last_mut()
                && cluster.last().is_some_and(|previous| {
                    previous.glyph.char_index == glyph.glyph.char_index
                        && previous.glyph.char_len == glyph.glyph.char_len
                })
            {
                cluster.push(glyph);
            } else {
                clusters.push(vec![glyph]);
            }
        }
        let line_byte_start = char_byte_index(text, line.char_start);
        let line_byte_end = char_byte_index(text, line.char_end);
        let Some(paragraph) = bidi.paragraphs.iter().find(|paragraph| {
            paragraph.range.start <= line_byte_start && line_byte_end <= paragraph.range.end
        }) else {
            line.glyphs = clusters.into_iter().flatten().collect();
            continue;
        };
        let (levels, visual_runs) = bidi.visual_runs(paragraph, line_byte_start..line_byte_end);
        let mut visual_clusters = Vec::with_capacity(clusters.len());
        for run in visual_runs {
            let mut matching = clusters
                .iter_mut()
                .filter(|cluster| {
                    cluster.first().is_some_and(|glyph| {
                        let byte = char_byte_index(text, glyph.glyph.char_index);
                        run.contains(&byte)
                    })
                })
                .collect::<Vec<_>>();
            if levels[run.start].is_rtl() {
                matching.reverse();
            }
            for cluster in matching {
                visual_clusters.append(cluster);
            }
        }
        let mut cursor_x = line.start_x;
        for glyph in &mut visual_clusters {
            glyph.x = cursor_x;
            cursor_x += glyph.glyph.advance;
        }
        line.end_x = cursor_x;
        line.glyphs = visual_clusters;
    }
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
    rtl: bool,
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
    pub(crate) layout_direction: u64,
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

    pub(crate) fn effective_align(self, authored_align: u64) -> u64 {
        // `Text::align()` preserves center and otherwise resolves left/right
        // from the LayoutComponent direction supplied by `controlSize`.
        if authored_align == 2 || self.layout_direction == 0 {
            authored_align
        } else if self.layout_direction == 2 {
            1
        } else {
            0
        }
    }
}

include!("text/text_modifier_group.rs");

include!("text/text_modifier_range.rs");

include!("text/text_follow_path_modifier.rs");

include!("text/text_modifier.rs");

include!("text/text_target_modifier.rs");

include!("text/text_variation_modifier.rs");

fn static_text_data_bind_supported(data_bind: &DataBindNode) -> bool {
    if !data_bind_flags_apply_source_to_target(data_bind.flags) {
        return true;
    }
    let Ok(property_key) = u16::try_from(data_bind.property_key) else {
        return false;
    };
    match data_bind.target_type_name {
        Some(
            target_type @ ("KeyFrameDouble" | "KeyFrameColor" | "KeyFrameBool" | "KeyFrameString"),
        ) => property_key_for_name(target_type, "value") == Some(property_key),
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
                || (property_key_for_name("WorldTransformComponent", "opacity")
                    == Some(property_key)
                    && (data_bind.converter_global.is_none()
                        || data_bind.converter_type_name == Some("DataConverterGroup")))
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
            ["fontSize", "fontAssetId"]
                .into_iter()
                .any(|name| property_key_for_name("TextStyle", name) == Some(property_key))
                && data_bind.converter_global.is_none()
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
        Some("ScrollConstraint") => [
            "scrollOffsetX",
            "scrollOffsetY",
            "scrollPercentX",
            "scrollPercentY",
            "scrollIndex",
        ]
        .into_iter()
        .any(|name| property_key_for_name("ScrollConstraint", name) == Some(property_key)),
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
                || (property_key_for_name("WorldTransformComponent", "opacity")
                    == Some(property_key)
                    && data_bind.converter_type_name == Some("DataConverterGroup"))
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

fn static_text_data_bind_targets_subtree(
    graph: &ArtboardGraph,
    text_local: usize,
    data_bind: &DataBindNode,
) -> bool {
    let Some(mut target_local) = data_bind.target_local else {
        return false;
    };
    let mut visited = BTreeSet::new();
    loop {
        if target_local == text_local {
            return true;
        }
        if !visited.insert(target_local) {
            return false;
        }
        let Some(parent_local) =
            component_for_local(graph, target_local).and_then(|component| component.parent_local)
        else {
            return false;
        };
        target_local = parent_local;
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
            if style.font_bytes(runtime, instance).is_none()
                || !self.style_font_size(runtime, instance, style)?.is_finite()
                || !self
                    .style_line_height(runtime, instance, style)?
                    .is_finite()
                || !self
                    .style_letter_spacing(runtime, instance, style)
                    .is_finite()
                || !style.variations_are_finite(instance)
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
            .filter(|data_bind| static_text_data_bind_targets_subtree(graph, text_local, data_bind))
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
                        | "TextStyleFeature"
                        | "TextModifierGroup"
                        | "TextModifierRange"
                        | "TextModifier"
                        | "TextShapeModifier"
                        | "TextVariationModifier"
                        | "TextTargetModifier"
                        | "Solo"
                        | "CubicInterpolatorComponent"
                        | "Shape"
                        | "Image"
                        // C++ `Artboard::drawInternal` dispatches Text and Image as
                        // independent sibling Drawables; an Image-owned crop Mesh
                        // (`src/shapes/mesh.cpp::Mesh::draw`) does not narrow the
                        // supported Text subtree.
                        | "Mesh"
                        | "MeshVertex"
                        | "PointsPath"
                        | "StraightVertex"
                        | "CubicDetachedVertex"
                        | "CubicAsymmetricVertex"
                        | "CubicMirroredVertex"
                        | "Triangle"
                        | "Ellipse"
                        | "Polygon"
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
                        | "ArtboardListMapRule"
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
                        | "ViewModelInstanceColor"
                        | "ViewModelInstanceNumber"
                        | "ViewModelInstanceBoolean"
                        | "ViewModelInstanceString"
                        | "ViewModelInstanceList"
                        | "ViewModelInstanceListItem"
                        | "ViewModelInstanceTrigger"
                        | "ViewModelInstanceViewModel"
                        | "ViewModelPropertyList"
                        | "ViewModelPropertyColor"
                        | "ViewModelPropertyNumber"
                        | "ViewModelPropertyBoolean"
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
                )
            )
        }) {
            bail!(
                "static text subset does not support {} global {}",
                object.type_name.unwrap_or("unknown"),
                object.global_id
            );
        }

        if let Some(component) = graph.components.iter().find(|component| {
            definition_by_name(component.type_name)
                .is_some_and(|definition| definition.is_a("TextModifier"))
                && component
                    .parent_local
                    .and_then(|parent| type_for_local(graph, parent))
                    != Some("TextModifierGroup")
        }) {
            bail!(
                "{} local {} requires a direct TextModifierGroup parent",
                component.type_name,
                component.local_id
            );
        }
        if let Some(component) = graph.components.iter().find(|component| {
            component.type_name == "TextStyleFeature"
                && !matches!(
                    component
                        .parent_local
                        .and_then(|parent| type_for_local(graph, parent)),
                    Some("TextStyle" | "TextStylePaint")
                )
        }) {
            bail!(
                "TextStyleFeature local {} requires a direct TextStyle parent",
                component.local_id
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
        let Some(font_bytes) = base_style.font_bytes(runtime, instance) else {
            // Mirrors src/importers/file_asset_importer.cpp: with no
            // FileAssetLoader and no in-band contents, a hosted FontAsset
            // resolves successfully but has no decoded font.
            return Ok(None);
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self.base_style()?.harf_variations(instance);
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
        let features = base_style.harf_features(instance);

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
            &features,
        )?;
        let scaled_font_size = font_size * font_scale;
        let scale = scaled_font_size / TEXT_SHAPE_SCALE_F32;
        let apply_ellipsis =
            self.should_apply_static_ellipsis(runtime, instance, layout_constraint)?;
        let text_input_bidi = self.kind == StaticTextKind::TextInput && text_has_rtl(&text);
        let lines = self.layout_static_text_lines(
            runtime,
            instance,
            layout_constraint,
            &text,
            &shaper,
            disable_legacy_kern,
            &features,
            scale,
            letter_spacing,
            text_input_bidi,
        )?;
        let line_metrics =
            self.static_line_metrics(runtime, instance, &lines, resolved_runs, font_scale)?;
        let contextual_glyphs = if text_input_bidi {
            self.styled_resolved_run_glyphs_bidi(runtime, instance, resolved_runs, font_scale)?
        } else {
            self.styled_resolved_run_glyphs(runtime, instance, resolved_runs, font_scale)?
        };
        let line_widths = lines
            .iter()
            .map(|line| Self::styled_line_width(line, &contextual_glyphs))
            .collect::<Vec<_>>();
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
        let overflow_as_fixed = self.overflow_as_fixed(runtime, instance, layout_constraint)?;
        let line_iteration_sizing = if overflow_as_fixed {
            TEXT_SIZING_FIXED
        } else {
            self.text_uint_property(runtime, instance, "sizingValue")?
        };
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
                line_iteration_sizing,
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
            let mut glyphs = Self::styled_line_glyphs(&line, &contextual_glyphs);
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
                let base_glyphs = shape_text_glyphs_with_features(
                    &shaper,
                    line.text,
                    disable_legacy_kern,
                    &features,
                );
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

        if text_input_bidi {
            reorder_text_input_bidi_geometry(&text, &mut shaped_lines);
        }

        let text_world_inverse = text_world.invert_or_identity();
        let mut has_geometric_modifiers = false;
        let mut has_non_monotone_advances = false;
        for line in &mut shaped_lines {
            has_non_monotone_advances |= line
                .glyphs
                .windows(2)
                .any(|glyphs| glyphs[1].glyph.char_index < glyphs[0].glyph.char_index);
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
        self.render_data_filtered(runtime, instance, layout_constraint, text_world, None)
    }

    fn render_data_filtered(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text_world: Mat2D,
        selection_filter: Option<(std::ops::Range<usize>, bool)>,
    ) -> Result<StaticTextRenderData> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        if resolved_runs.iter().all(|run| run.text.is_empty()) {
            return Ok(StaticTextRenderData {
                path_buckets_by_style: vec![Vec::new(); self.styles.len()],
                color_glyphs: Vec::new(),
                order: Vec::new(),
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
                color_glyphs: Vec::new(),
                order: Vec::new(),
                local_transform: Mat2D::IDENTITY,
            });
        };
        let mut commands_by_style = vec![Vec::new(); self.styles.len()];
        let mut color_glyphs = Vec::new();
        let mut order = Vec::new();
        let mut established_styles = vec![false; self.styles.len()];
        for line in &layout.lines {
            for positioned in &line.glyphs {
                let glyph = &positioned.glyph;
                if let Some((selection, include_selected)) = &selection_filter {
                    let selected = glyph.char_index < selection.end
                        && glyph.char_index.saturating_add(glyph.char_len) > selection.start;
                    if selected != *include_selected {
                        continue;
                    }
                }
                let center_x = positioned.x + glyph.advance * 0.5;
                let glyph_id = GlyphId::new(glyph.glyph_id);
                let style = &self.styles[glyph.style_index];
                if let Some(style_font_bytes) = style.font_bytes(runtime, instance) {
                    if runtime_classify_color_glyph(style_font_bytes, glyph.glyph_id)
                        != RuntimeColorGlyphClassification::Monochrome
                    {
                        let layers = runtime_extract_color_glyph_layers(
                            style_font_bytes,
                            glyph.glyph_id,
                            style.foreground_color(instance),
                        );
                        if !layers.is_empty() {
                            let color_index = color_glyphs.len();
                            color_glyphs.push(RuntimeIntegratedColorGlyphCommand {
                                font_identity: style_font_bytes.as_ptr() as usize,
                                glyph_id: glyph.glyph_id,
                                transform: runtime_positioned_color_glyph_transform(
                                    positioned,
                                    line.baseline,
                                ),
                                opacity: positioned.modifier_opacity,
                                layers,
                            });
                            order.push(RuntimeTextDrawOrder::ColorGlyph(color_index));
                        }
                        continue;
                    }
                    let style_font = SkrifaFontRef::new(style_font_bytes)
                        .context("failed to parse font for outlines")?;
                    let outlines = style_font.outline_glyphs();
                    let skrifa_variations = glyph
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
                        if positioned.modifier_opacity > 0.0
                            && !established_styles[glyph.style_index]
                        {
                            established_styles[glyph.style_index] = true;
                            order.push(RuntimeTextDrawOrder::Style(style.local_id));
                        }
                    }
                }
            }
        }

        Ok(StaticTextRenderData {
            path_buckets_by_style: commands_by_style,
            color_glyphs,
            order,
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
        let Some(font_bytes) = base_style.font_bytes(runtime, instance) else {
            return self.unshaped_local_bounds(runtime, instance, None);
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self.base_style()?.harf_variations(instance);
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
        let features = base_style.harf_features(instance);

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
            &features,
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
                let glyphs = shape_text_glyphs_with_features(
                    &shaper,
                    line.text,
                    disable_legacy_kern,
                    &features,
                );
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
        let Some(font_bytes) = base_style.font_bytes(runtime, instance) else {
            return match purpose {
                StaticTextLayoutBoundsPurpose::Measure => Ok(Some((0.0, 0.0, 0.0, 0.0))),
                StaticTextLayoutBoundsPurpose::Controlled => {
                    self.unshaped_local_bounds(runtime, instance, Some(layout_constraint))
                }
            };
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self.base_style()?.harf_variations(instance);
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
        let features = base_style.harf_features(instance);

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
            &features,
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
            &features,
            scale,
            letter_spacing,
            self.kind == StaticTextKind::TextInput && text_has_rtl(&text),
        )?;
        let line_metrics =
            self.static_line_metrics(runtime, instance, &lines, &resolved_runs, font_scale)?;
        let contextual_glyphs = if self.kind == StaticTextKind::TextInput && text_has_rtl(&text) {
            self.styled_resolved_run_glyphs_bidi(runtime, instance, &resolved_runs, font_scale)?
        } else {
            self.styled_resolved_run_glyphs(runtime, instance, &resolved_runs, font_scale)?
        };
        let measured_width = lines
            .iter()
            .map(|line| Self::styled_line_width(line, &contextual_glyphs))
            .fold(0.0f32, f32::max);
        let sizing = match purpose {
            StaticTextLayoutBoundsPurpose::Measure => self.authored_sizing(runtime, instance)?,
            StaticTextLayoutBoundsPurpose::Controlled => {
                self.effective_sizing(runtime, instance, Some(layout_constraint))?
            }
        };
        let width = match (purpose, sizing) {
            (StaticTextLayoutBoundsPurpose::Measure, _)
                if self.kind == StaticTextKind::TextInput =>
            {
                measured_width.min(layout_constraint.width)
            }
            (
                StaticTextLayoutBoundsPurpose::Measure,
                TEXT_SIZING_AUTO_HEIGHT | TEXT_SIZING_FIXED,
            ) => self
                .text_width(runtime, instance)?
                .min(layout_constraint.width),
            (StaticTextLayoutBoundsPurpose::Measure, _) => {
                measured_width.min(layout_constraint.width)
            }
            (StaticTextLayoutBoundsPurpose::Controlled, _) => layout_constraint.width,
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
            (StaticTextLayoutBoundsPurpose::Controlled, _) => layout_constraint.height,
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
            let text = if run.text_property_owner == "TextInput" {
                instance
                    .text_input_display_text(run.local_id)
                    .or_else(|| {
                        instance
                            .string_property(run.local_id, property_key)
                            .and_then(|bytes| std::str::from_utf8(bytes).ok())
                            .map(str::to_owned)
                    })
                    .or_else(|| {
                        runtime
                            .object(run.global_id as usize)
                            .and_then(|object| object.string_property_bytes("text"))
                            .and_then(|bytes| std::str::from_utf8(bytes).ok())
                            .map(str::to_owned)
                    })
                    .context("TextInput missing text")?
            } else {
                let bytes = instance
                    .string_property(run.local_id, property_key)
                    .or_else(|| {
                        runtime
                            .object(run.global_id as usize)
                            .and_then(|object| object.string_property_bytes("text"))
                    })
                    .context("TextValueRun missing text")?;
                std::str::from_utf8(bytes)
                    .with_context(|| format!("{}.text is not UTF-8", run.text_property_owner))?
                    .to_owned()
            };
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
        // Exact port of C++ `src/text/line_breaker.cpp`
        // `computeLineMetrics` / `GlyphLine::ComputeLineSpacing`. An explicit
        // lineHeight preserves the font's natural baseline ratio; it is not a
        // CSS half-leading box. C++ always places the first line at the font's
        // real ascent, while subsequent lines use the adjusted ascent.
        // Authored lineHeight is absolute and therefore does not follow the
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
                let Some(font_bytes) = style.font_bytes(runtime, instance) else {
                    continue;
                };
                let font = SkrifaFontRef::new(font_bytes)
                    .context("failed to parse font for line metrics")?;
                let skrifa_variations = style.skrifa_variations(instance);
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
                    let baseline_factor = natural_ascent_px / natural_height;
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
        let Some(font_bytes) = style.font_bytes(runtime, instance) else {
            return Ok(0.0);
        };
        let font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for trim metrics")?;
        let skrifa_variations = style.skrifa_variations(instance);
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
        let Some(font_bytes) = style.font_bytes(runtime, instance) else {
            return Ok(0.0);
        };
        let font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for trim metrics")?;
        let skrifa_variations = style.skrifa_variations(instance);
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
        features: &[Feature],
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
            features,
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
            if style.font_bytes(runtime, instance).is_some() {
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
        features: &[Feature],
        max_size: f32,
    ) -> Result<f32> {
        let sizing = self.effective_sizing(runtime, instance, layout_constraint)?;
        let overflow_as_fixed = self.overflow_as_fixed(runtime, instance, layout_constraint)?;
        if max_size <= 1.0 || (sizing == TEXT_SIZING_AUTO_WIDTH && !overflow_as_fixed) {
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
                features,
                scale,
                letter_spacing,
                self.kind == StaticTextKind::TextInput && text_has_rtl(text),
            )?;
            let contextual_glyphs = if self.kind == StaticTextKind::TextInput && text_has_rtl(text)
            {
                self.styled_resolved_run_glyphs_bidi(runtime, instance, runs, font_scale)?
            } else {
                self.styled_resolved_run_glyphs(runtime, instance, runs, font_scale)?
            };
            let max_width = lines
                .iter()
                .map(|line| Self::styled_line_width(line, &contextual_glyphs))
                .fold(0.0f32, f32::max);
            let line_metrics =
                self.static_line_metrics(runtime, instance, &lines, runs, font_scale)?;
            let height = line_metrics
                .last()
                .map(|metrics| metrics.bottom)
                .unwrap_or(0.0);
            Ok(max_width <= box_width && (!overflow_as_fixed || height <= box_height))
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

    fn styled_resolved_run_glyphs(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        runs: &[StaticResolvedRun],
        font_scale: f32,
    ) -> Result<Vec<StyledTextGlyph>> {
        let full_text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let source_lines = split_static_text_lines(&full_text);
        let character_count = full_text.chars().count();
        let modifier_coverages = self
            .modifiers
            .iter()
            .map(|group| {
                if group.has_shape_modifiers() {
                    group.coverage_by_character(runtime, instance, &full_text, runs, &source_lines)
                } else {
                    // Paint-only modifier coverage must not fracture shaping
                    // context. C++ only splits runs for shape modifiers.
                    Ok(vec![0.0; character_count])
                }
            })
            .collect::<Result<Vec<_>>>()?;
        let mut glyphs = Vec::new();
        for run in runs {
            let style_index = self.style_index_for_local(run.style_local)?;
            for paragraph in split_static_text_lines(&run.text) {
                // C++ shapes each paragraph before `BreakLines` and keeps the
                // resulting advances when a soft wrap slices the glyph run.
                // In particular, a line-ending glyph retains kerning against
                // the first glyph on the next visual line.
                let paragraph_start = run.char_start + paragraph.char_start;
                let chars = paragraph.text.char_indices().collect::<Vec<_>>();
                let mut segment_start = 0usize;
                while segment_start < chars.len() {
                    let global_start = paragraph_start + segment_start;
                    let strengths = modifier_coverages
                        .iter()
                        .map(|coverage| coverage.get(global_start).copied().unwrap_or(0.0))
                        .collect::<Vec<_>>();
                    let mut segment_end = segment_start + 1;
                    while segment_end < chars.len()
                        && modifier_coverages
                            .iter()
                            .zip(&strengths)
                            .all(|(coverage, strength)| {
                                coverage
                                    .get(paragraph_start + segment_end)
                                    .copied()
                                    .unwrap_or(0.0)
                                    == *strength
                            })
                    {
                        segment_end += 1;
                    }
                    let byte_start = chars[segment_start].0;
                    let byte_end = chars
                        .get(segment_end)
                        .map(|value| value.0)
                        .unwrap_or(paragraph.text.len());
                    glyphs.extend(self.styled_text_glyphs_for_style_with_strengths(
                        runtime,
                        instance,
                        &paragraph.text[byte_start..byte_end],
                        global_start,
                        style_index,
                        font_scale,
                        &strengths,
                    )?);
                    segment_start = segment_end;
                }
            }
        }
        Ok(glyphs)
    }

    fn styled_resolved_run_glyphs_bidi(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        runs: &[StaticResolvedRun],
        font_scale: f32,
    ) -> Result<Vec<StyledTextGlyph>> {
        let mut glyphs = Vec::new();
        for run in runs {
            let style_index = self.style_index_for_local(run.style_local)?;
            for paragraph in split_static_text_lines(&run.text) {
                glyphs.extend(self.styled_text_glyphs_for_style_bidi(
                    runtime,
                    instance,
                    paragraph.text,
                    run.char_start + paragraph.char_start,
                    style_index,
                    font_scale,
                )?);
            }
        }
        glyphs.sort_by_key(|glyph| glyph.char_index);
        Ok(glyphs)
    }

    fn styled_line_glyphs(
        line: &StaticTextLine<'_>,
        contextual_glyphs: &[StyledTextGlyph],
    ) -> Vec<StyledTextGlyph> {
        let range = Self::styled_line_glyph_range(line, contextual_glyphs);
        contextual_glyphs[range].to_vec()
    }

    fn styled_line_glyph_range(
        line: &StaticTextLine<'_>,
        contextual_glyphs: &[StyledTextGlyph],
    ) -> std::ops::Range<usize> {
        let line_start = line.char_start;
        let line_end = line_start + line.text.chars().count();
        let start = contextual_glyphs.partition_point(|glyph| glyph.char_index < line_start);
        let end =
            start + contextual_glyphs[start..].partition_point(|glyph| glyph.char_index < line_end);
        start..end
    }

    fn styled_line_width(line: &StaticTextLine<'_>, contextual_glyphs: &[StyledTextGlyph]) -> f32 {
        contextual_glyphs[Self::styled_line_glyph_range(line, contextual_glyphs)]
            .iter()
            .map(|glyph| glyph.advance)
            .sum()
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
        self.styled_text_glyphs_for_style_with_strengths(
            runtime,
            instance,
            text,
            char_start,
            style_index,
            font_scale,
            &[],
        )
    }

    fn styled_text_glyphs_for_style_with_strengths(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        char_start: usize,
        style_index: usize,
        font_scale: f32,
        strengths: &[f32],
    ) -> Result<Vec<StyledTextGlyph>> {
        let style = self
            .styles
            .get(style_index)
            .with_context(|| format!("missing TextStylePaint index {style_index}"))?;
        let Some(font_bytes) = style.font_bytes(runtime, instance) else {
            return Ok(Vec::new());
        };
        let font_size = self.style_font_size(runtime, instance, style)?;
        let scale = font_size * font_scale / TEXT_SHAPE_SCALE_F32;
        let letter_spacing = self.style_letter_spacing(runtime, instance, style);
        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse localized variable font")?;
        let mut localized = BTreeMap::<u32, f32>::new();
        let mut effective = style
            .variation_values(instance)
            .into_iter()
            .collect::<BTreeMap<_, _>>();
        for (group, strength) in self.modifiers.iter().zip(strengths) {
            if *strength == 0.0 || !group.has_shape_modifiers() {
                continue;
            }
            let group_variations =
                group.variation_map(instance, &skrifa_font, *strength, &effective);
            effective.extend(group_variations.iter().map(|(tag, value)| (*tag, *value)));
            localized.extend(group_variations);
        }
        let raw_glyphs = shape_text_glyphs_for_style_with_variations(
            font_bytes, style, instance, text, &localized,
        )?;
        let mut variations = style.variation_values(instance);
        for (tag, value) in localized {
            if let Some(existing) = variations.iter_mut().find(|item| item.0 == tag) {
                existing.1 = value;
            } else {
                variations.push((tag, value));
            }
        }
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
                rtl: false,
                variations: variations.clone(),
            })
            .collect())
    }

    fn styled_text_glyphs_for_style_bidi(
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
        let Some(font_bytes) = style.font_bytes(runtime, instance) else {
            return Ok(Vec::new());
        };
        let font_size = self.style_font_size(runtime, instance, style)?;
        let scale = font_size * font_scale / TEXT_SHAPE_SCALE_F32;
        let letter_spacing = self.style_letter_spacing(runtime, instance, style);
        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = style.harf_variations(instance);
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
        let features = style.harf_features(instance);
        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for metrics")?;
        let raw_glyphs = shape_bidi_text_glyphs_with_features(
            &shaper,
            text,
            disable_legacy_kern_for_advances(&skrifa_font),
            &features,
        );
        let bidi = unicode_bidi::BidiInfo::new(text, None);
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
                rtl: bidi
                    .levels
                    .get(glyph.cluster as usize)
                    .is_some_and(|level| level.is_rtl()),
                variations: style.variation_values(instance),
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
        if let Some(constraint) = layout_constraint
            && let Some(text) = instance
                .component(self.text_local)
                .and_then(|component| component.concrete.text.as_ref())
        {
            text.retain_layout_scale_types(
                constraint.width_scale_type,
                constraint.height_scale_type,
            );
        }
        Ok(layout_constraint
            .map(|constraint| constraint.effective_sizing(authored))
            .unwrap_or(authored))
    }

    fn overflow_as_fixed(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
    ) -> Result<bool> {
        Ok(
            self.effective_sizing(runtime, instance, layout_constraint)? == TEXT_SIZING_FIXED
                || layout_constraint.is_some(),
        )
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
        if layout_constraint.is_none()
            && self.effective_sizing(runtime, instance, layout_constraint)? != TEXT_SIZING_FIXED
        {
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
        if !self.overflow_as_fixed(runtime, instance, layout_constraint)?
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
            && let Some(font_bytes) = base_style.font_bytes(runtime, instance)
        {
            let harf_font =
                HarfFontRef::new(font_bytes).context("failed to parse font for clip layout")?;
            let harf_variations = base_style.harf_variations(instance);
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
            let features = base_style.harf_features(instance);
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
                &features,
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
                &features,
                scale,
                self.letter_spacing(runtime, instance),
                self.kind == StaticTextKind::TextInput && text_has_rtl(&text),
            )?;
            line_metrics =
                self.static_line_metrics(runtime, instance, &lines, &resolved_runs, font_scale)?;
            let contextual_glyphs = if self.kind == StaticTextKind::TextInput && text_has_rtl(&text)
            {
                self.styled_resolved_run_glyphs_bidi(runtime, instance, &resolved_runs, font_scale)?
            } else {
                self.styled_resolved_run_glyphs(runtime, instance, &resolved_runs, font_scale)?
            };
            measured_width = lines
                .iter()
                .map(|line| Self::styled_line_width(line, &contextual_glyphs))
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
        let overflow = self.text_uint_property(runtime, instance, "overflowValue")?;
        Ok(
            self.overflow_as_fixed(runtime, instance, layout_constraint)?
                && overflow == TEXT_OVERFLOW_ELLIPSIS
                && self.effective_width(runtime, instance, layout_constraint)? > 0.0,
        )
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
        features: &[Feature],
        scale: f32,
        letter_spacing: f32,
        bidi: bool,
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
                let glyphs = if bidi {
                    shape_bidi_text_glyphs_with_features(
                        shaper,
                        remaining,
                        disable_legacy_kern,
                        features,
                    )
                } else {
                    shape_text_glyphs_with_features(
                        shaper,
                        remaining,
                        disable_legacy_kern,
                        features,
                    )
                };
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
        let bounds_width = match (layout_constraint, sizing) {
            (Some(constraint), _) => constraint.width,
            (None, TEXT_SIZING_AUTO_HEIGHT | TEXT_SIZING_FIXED) => {
                self.effective_width(runtime, instance, layout_constraint)?
            }
            (None, _) => measured_width,
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
        let bounds_height = match (layout_constraint, sizing) {
            (Some(constraint), _) => constraint.height,
            (None, TEXT_SIZING_FIXED) => {
                self.effective_height(runtime, instance, layout_constraint)?
            }
            (None, _) => {
                (full_height - vertical_trim.top - vertical_trim.bottom).max(min_y) - min_y
            }
        };
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;
        let authored_align = self.text_uint_property(runtime, instance, "alignValue")?;
        let align_value = layout_constraint
            .map(|constraint| constraint.effective_align(authored_align))
            .unwrap_or(authored_align);
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
        if self.overflow_as_fixed(runtime, instance, layout_constraint)? {
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
        let overflow_as_fixed = self.overflow_as_fixed(runtime, instance, layout_constraint)?;
        let bounds_width = match layout_constraint {
            Some(constraint) => constraint.width,
            None if sizing == TEXT_SIZING_AUTO_WIDTH => measured_width,
            None => self.effective_width(runtime, instance, None)?,
        };
        let bounds_height = match layout_constraint {
            Some(constraint) => constraint.height,
            None if sizing == TEXT_SIZING_FIXED => {
                self.effective_height(runtime, instance, None)?
            }
            None => total_height,
        };
        let x_scale = if (sizing != TEXT_SIZING_AUTO_WIDTH || overflow_as_fixed)
            && measured_width > bounds_width
        {
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
        let y_scale = if overflow_as_fixed && total_height > bounds_height {
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
            let authored_align = self.text_uint_property(runtime, instance, "alignValue")?;
            let align = layout_constraint
                .map(|constraint| constraint.effective_align(authored_align))
                .unwrap_or(authored_align);
            match align {
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

        if overflow_as_fixed {
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

    fn text_input_fallback_cursor_height(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<f32> {
        let runs = self.resolved_runs(runtime, instance)?;
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let lines = split_static_text_lines(&text);
        let line_metrics = self.static_line_metrics(runtime, instance, &lines, &runs, 1.0)?;
        let line_height = line_metrics
            .first()
            .map(|metrics| metrics.bottom - metrics.top)
            .unwrap_or(0.0);
        Ok(line_height)
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
                if runtime_live_shape_paint_path_kind(instance, paint)
                    == Some(RuntimeShapePaintPathKind::World)
                {
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
    fn foreground_color(&self, instance: &ArtboardInstance) -> u32 {
        self.container
            .into_iter()
            .flat_map(|container| container.paints.iter())
            .find(|paint| {
                paint.paint_type == ShapePaintKind::Fill
                    && matches!(
                        paint.paint_state,
                        Some(ShapePaintStateNode::SolidColor { .. })
                    )
            })
            .and_then(|paint| match paint.paint_state {
                Some(ShapePaintStateNode::SolidColor { color }) => Some(
                    paint
                        .mutator_local
                        .zip(solid_color_value_property_key())
                        .and_then(|(local, key)| instance.color_property(local, key))
                        .unwrap_or(color),
                ),
                _ => None,
            })
            .unwrap_or(0xff00_0000)
    }

    fn variation_values(&self, instance: &ArtboardInstance) -> Vec<(u32, f32)> {
        let tag_key = property_key_for_name("TextStyleAxis", "tag");
        let axis_value_key = property_key_for_name("TextStyleAxis", "axisValue");
        self.variations
            .iter()
            .map(|variation| {
                let value = axis_value_key
                    .and_then(|key| instance.double_property(variation.axis_local, key))
                    .unwrap_or(variation.authored_value);
                let tag = tag_key
                    .and_then(|key| instance.uint_property(variation.axis_local, key))
                    .map(|value| value as u32)
                    .unwrap_or(variation.tag);
                (tag, value)
            })
            .collect()
    }

    fn variations_are_finite(&self, instance: &ArtboardInstance) -> bool {
        self.variation_values(instance)
            .iter()
            .all(|(_, value)| value.is_finite())
    }

    fn harf_variations(&self, instance: &ArtboardInstance) -> Vec<(HarfTag, f32)> {
        self.variation_values(instance)
            .into_iter()
            .map(|(tag, value)| (HarfTag::from_u32(tag), value))
            .collect()
    }

    fn harf_features(&self, instance: &ArtboardInstance) -> Vec<Feature> {
        self.features
            .iter()
            .copied()
            .map(|feature| feature.harf_feature(instance))
            .collect()
    }

    fn skrifa_variations(&self, instance: &ArtboardInstance) -> Vec<VariationSetting> {
        self.variation_values(instance)
            .into_iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(tag), value))
            .collect()
    }

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
        let (font_asset_global, font_asset_id, font_bytes) =
            if style.property("fontAssetId").is_some() {
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
                    Some(font_asset.id),
                    Some(asset_id),
                    embedded_file_asset_bytes(runtime, font_asset.id),
                )
            } else {
                (None, None, None)
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
            variations.push(StaticTextVariation {
                tag,
                axis_local,
                authored_value: axis_value,
            });
        }
        let features = style_component
            .children
            .iter()
            .copied()
            .filter(|local| type_for_local(graph, *local) == Some("TextStyleFeature"))
            .map(|local| StaticTextStyleFeature::from_graph(runtime, graph, local))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            local_id: style_local,
            global_id: style_global,
            name: style.string_property("name").map(str::to_owned),
            container,
            font_asset_global,
            font_asset_id,
            font_bytes,
            variations,
            features,
        })
    }

    fn font_bytes<'instance>(
        &'instance self,
        runtime: &'instance RuntimeFile,
        instance: &'instance ArtboardInstance,
    ) -> Option<&'instance [u8]> {
        if let Some(value) = instance.text_style_font_override(self.local_id) {
            return runtime_font_asset_bytes(runtime, instance, value);
        }
        self.font_asset_global
            .and_then(|asset_global| instance.runtime_font_asset_bytes(asset_global))
            .or(self.font_bytes)
            .or_else(|| {
                self.font_asset_id
                    .and_then(|asset_id| instance.external_font_asset_bytes(asset_id))
            })
    }
}

fn runtime_positioned_color_glyph_transform(
    positioned: &StaticPositionedTextGlyph,
    baseline: f32,
) -> Mat2D {
    let size = positioned.glyph.scale * TEXT_SHAPE_SCALE_F32;
    let base = Mat2D([size, 0.0, 0.0, size, positioned.x, baseline]);
    if positioned.modifier_transform == Mat2D::IDENTITY {
        return base;
    }
    let center_x = positioned.x + positioned.glyph.advance * 0.5;
    let center = Mat2D([1.0, 0.0, 0.0, 1.0, center_x, baseline]);
    let inverse_center = Mat2D([1.0, 0.0, 0.0, 1.0, -center_x, -baseline]);
    center
        .multiply(positioned.modifier_transform)
        .multiply(inverse_center)
        .multiply(base)
}

// Mirrors the static coverage/translation subset from C++
// src/text/text_modifier_group.cpp and src/text/text_modifier_range.cpp.

// Ported from C++ `src/text/text_follow_path_modifier.cpp`.

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

include!("text/utf.rs");

include!("text/glyph_lookup.rs");

include!("text/line_breaker.rs");

include!("text/text_style_feature.rs");

include!("text/text_variation_helper.rs");

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
    runtime.imported_file_asset_contents(asset_global)
}

/// Resolve the effective bytes of a data-bound font value with the same
/// precedence as C++ `DataBindContextValueAssetFont::apply`.
///
/// A valid file FontAsset always wins, even when it has no loaded bytes. The
/// private live font is consulted only when the serialized index is missing,
/// out of range, or names a non-font asset.
pub(crate) fn runtime_font_asset_bytes<'a>(
    runtime: &'a RuntimeFile,
    instance: &'a ArtboardInstance,
    value: &'a RuntimeFontAssetValue,
) -> Option<&'a [u8]> {
    if let Ok(file_asset_index) = usize::try_from(value.file_asset_index())
        && let Some(font_asset) = runtime.file_asset(file_asset_index)
        && font_asset.type_name == "FontAsset"
    {
        return instance
            .runtime_font_asset_bytes(font_asset.id)
            .or_else(|| embedded_file_asset_bytes(runtime, font_asset.id))
            .or_else(|| {
                font_asset
                    .uint_property("assetId")
                    .and_then(|asset_id| u32::try_from(asset_id).ok())
                    .and_then(|asset_id| instance.external_font_asset_bytes(asset_id))
            });
    }

    value.live_font_bytes()
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

    fn font_table_range(bytes: &[u8], tag: &[u8; 4]) -> std::ops::Range<usize> {
        let table_count = usize::from(u16::from_be_bytes([bytes[4], bytes[5]]));
        (0..table_count)
            .map(|index| 12 + index * 16)
            .find_map(|record| {
                (bytes.get(record..record + 4) == Some(tag)).then(|| {
                    let offset = u32::from_be_bytes([
                        bytes[record + 8],
                        bytes[record + 9],
                        bytes[record + 10],
                        bytes[record + 11],
                    ]) as usize;
                    let length = u32::from_be_bytes([
                        bytes[record + 12],
                        bytes[record + 13],
                        bytes[record + 14],
                        bytes[record + 15],
                    ]) as usize;
                    offset..offset + length
                })
            })
            .unwrap_or_else(|| panic!("fixture has a {} table", String::from_utf8_lossy(tag)))
    }

    fn empty_glyph_with_padding_font_bytes() -> Vec<u8> {
        let mut bytes = fixture_font_bytes();
        let glyf = font_table_range(&bytes, b"glyf");
        assert_eq!(glyf.len(), 244, "fixture has one 244-byte glyph body");
        assert_eq!(
            i16::from_be_bytes([bytes[glyf.start], bytes[glyf.start + 1]]),
            2,
            "fixture glyph body starts as a two-contour simple glyph"
        );
        bytes[glyf.start..glyf.start + 2].copy_from_slice(&0i16.to_be_bytes());
        bytes[glyf.start + 10..glyf.start + 12].copy_from_slice(&0u16.to_be_bytes());
        bytes[glyf.start + 12..glyf.end].fill(0);
        bytes
    }

    fn malformed_outline_font_bytes() -> Vec<u8> {
        let mut bytes = fixture_font_bytes();
        let glyf = font_table_range(&bytes, b"glyf");
        assert_eq!(
            i16::from_be_bytes([bytes[glyf.start], bytes[glyf.start + 1]]),
            2,
            "fixture glyph body starts as a two-contour simple glyph"
        );
        bytes[glyf.start + 14..glyf.start + 16].copy_from_slice(&u16::MAX.to_be_bytes());
        bytes
    }

    fn baseline_origin_text_runtime_with_sizing_and_line_height(
        sizing_value: u64,
        line_height: Option<f32>,
    ) -> (RuntimeFile, GraphFile) {
        baseline_origin_text_runtime_with_sizing_line_height_and_font(
            sizing_value,
            line_height,
            fixture_font_bytes(),
        )
    }

    fn baseline_origin_text_runtime_with_sizing_and_font(
        sizing_value: u64,
        font_bytes: Vec<u8>,
    ) -> (RuntimeFile, GraphFile) {
        baseline_origin_text_runtime_with_sizing_line_height_and_font(
            sizing_value,
            Some(40.0),
            font_bytes,
        )
    }

    fn baseline_origin_text_runtime_with_sizing_line_height_and_font(
        sizing_value: u64,
        line_height: Option<f32>,
        font_bytes: Vec<u8>,
    ) -> (RuntimeFile, GraphFile) {
        let mut style_properties = vec![
            property("TextStylePaint", "parentId", AuthoringValue::Uint(1)),
            property("TextStylePaint", "fontSize", AuthoringValue::Double(20.0)),
            property("TextStylePaint", "fontAssetId", AuthoringValue::Uint(0)),
        ];
        if let Some(line_height) = line_height {
            style_properties.push(property(
                "TextStylePaint",
                "lineHeight",
                AuthoringValue::Double(line_height),
            ));
        }
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
                    AuthoringValue::Bytes(font_bytes),
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
            authoring_record("TextStylePaint", style_properties),
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

    fn baseline_origin_text_runtime_with_sizing(sizing_value: u64) -> (RuntimeFile, GraphFile) {
        baseline_origin_text_runtime_with_sizing_and_line_height(sizing_value, Some(40.0))
    }

    #[test]
    fn embedded_font_validation_accepts_empty_glyph_with_padding() {
        // Regression for googlefonts/fontations#1962: a zero-contour simple
        // glyph may carry trailing padding, and outline traversal must treat
        // it as an empty glyph rather than indexing an empty point buffer.
        let font = empty_glyph_with_padding_font_bytes();
        assert!(
            HarfFontRef::new(&font).is_ok() && SkrifaFontRef::new(&font).is_ok(),
            "the shallow parsers must accept the regression font"
        );
        assert!(embedded_font_is_parseable(&font));
    }

    #[test]
    fn embedded_font_catalog_accepts_empty_glyph_with_padding() {
        let (runtime, _) = baseline_origin_text_runtime_with_sizing_and_font(
            TEXT_SIZING_FIXED,
            empty_glyph_with_padding_font_bytes(),
        );
        assert!(embedded_fonts_are_parseable(&runtime));
    }

    #[test]
    fn embedded_font_validation_rejects_oversized_instruction_length() {
        let malformed = malformed_outline_font_bytes();
        assert!(
            HarfFontRef::new(&malformed).is_ok() && SkrifaFontRef::new(&malformed).is_ok(),
            "the shallow parsers must accept the regression font"
        );
        assert!(matches!(
            std::panic::catch_unwind(|| embedded_font_is_parseable(&malformed)),
            Ok(false)
        ));
    }

    #[test]
    fn embedded_font_catalog_rejects_a_malformed_in_band_font() {
        let (runtime, _) = baseline_origin_text_runtime_with_sizing_and_font(
            TEXT_SIZING_FIXED,
            malformed_outline_font_bytes(),
        );
        assert!(!embedded_fonts_are_parseable(&runtime));
    }

    #[test]
    fn embedded_font_validation_contains_dependency_panics() {
        assert!(!font_parser_panic_boundary(|| panic!(
            "synthetic font parser panic"
        )));
    }

    fn baseline_origin_text_runtime() -> (RuntimeFile, GraphFile) {
        baseline_origin_text_runtime_with_sizing(TEXT_SIZING_FIXED)
    }

    fn synthetic_data_bind(
        target_local: usize,
        target_global: u32,
        target_type_name: &'static str,
        property_key: u16,
    ) -> DataBindNode {
        DataBindNode {
            global_id: 10_000,
            type_name: "DataBind",
            property_key: u64::from(property_key),
            flags: 0,
            converter_id: 0,
            converter_global: None,
            converter_type_name: None,
            converter_duration: None,
            target_global: Some(target_global),
            target_type_name: Some(target_type_name),
            target_local: Some(target_local),
        }
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 0.001,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn cxx_script_itemization_keeps_leading_common_text_separate() {
        let runs = cxx_script_runs("[RIVE] EULAV LAITINI [END]");
        assert_eq!(
            runs,
            vec![
                CxxScriptRun {
                    text: "[",
                    byte_start: 0,
                    script: harfrust::script::COMMON,
                },
                CxxScriptRun {
                    text: "RIVE] EULAV LAITINI [END]",
                    byte_start: 1,
                    script: harfrust::script::LATIN,
                },
            ]
        );
    }

    #[test]
    fn cxx_script_itemization_propagates_common_and_nonspacing_marks() {
        let runs = cxx_script_runs("A \u{05b0}\u{05d0}");
        assert_eq!(
            runs,
            vec![
                CxxScriptRun {
                    text: "A \u{05b0}",
                    byte_start: 0,
                    script: harfrust::script::LATIN,
                },
                CxxScriptRun {
                    text: "\u{05d0}",
                    byte_start: 4,
                    script: harfrust::script::HEBREW,
                },
            ]
        );
    }

    #[test]
    fn data_bound_font_resolution_prefers_file_assets_then_private_live_font() {
        let (runtime, graphs) = baseline_origin_text_runtime();
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        let embedded =
            embedded_file_asset_bytes(&runtime, runtime.file_asset(0).expect("font asset").id)
                .expect("fixture embeds font bytes");
        let live: std::sync::Arc<[u8]> = vec![1, 2, 3, 4].into();
        let mut value = RuntimeFontAssetValue::default();

        assert!(value.set_live_font_bytes(Some(std::sync::Arc::clone(&live))));
        assert_eq!(
            runtime_font_asset_bytes(&runtime, &instance, &value),
            Some(live.as_ref())
        );

        assert!(value.set_file_asset_index(0));
        assert_eq!(
            runtime_font_asset_bytes(&runtime, &instance, &value),
            Some(embedded),
            "a valid file FontAsset wins over the retained private live font"
        );

        assert!(value.set_file_asset_index(u64::from(u32::MAX)));
        assert_eq!(
            runtime_font_asset_bytes(&runtime, &instance, &value),
            Some(live.as_ref())
        );
    }

    #[test]
    fn data_bound_live_font_override_is_used_and_dirties_text_geometry() {
        let (runtime, graphs) = baseline_origin_text_runtime();
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let mut instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).expect("Text slice builds");
        let style = slice.base_style().expect("fixture has a base style");
        let live: std::sync::Arc<[u8]> = fixture_font_bytes().into();
        let mut value = RuntimeFontAssetValue::default();
        assert!(value.set_live_font_bytes(Some(std::sync::Arc::clone(&live))));
        let path_epoch = instance.path_epoch();
        let layout_revision = instance.layout_revision();

        assert!(instance.set_text_style_font_override(style.local_id, value.clone()));
        assert_eq!(style.font_bytes(&runtime, &instance), Some(live.as_ref()));
        assert!(instance.path_epoch() > path_epoch);
        assert!(instance.layout_revision() > layout_revision);
        assert!(
            !instance.set_text_style_font_override(style.local_id, value),
            "reapplying the same live font must not re-dirty text"
        );
    }

    #[test]
    fn static_text_bind_validation_is_scoped_to_the_text_subtree() {
        let (runtime, mut graphs) = baseline_origin_text_runtime();
        let graph = graphs
            .artboards
            .first_mut()
            .expect("fixture has an artboard");
        let mut sibling = graph
            .components
            .iter()
            .find(|component| component.local_id == 1)
            .expect("fixture Text component")
            .clone();
        sibling.local_id = 4;
        sibling.global_id = 10_004;
        sibling.type_name = "NestedArtboard";
        sibling.parent_local = Some(0);
        sibling.parent_global = Some(graph.global_id);
        sibling.children.clear();
        graph.components.push(sibling);
        graph.data_binds.push(synthetic_data_bind(
            4,
            10_004,
            "NestedArtboard",
            property_key_for_name("WorldTransformComponent", "opacity").expect("opacity property"),
        ));

        StaticTextSlice::from_graph(&runtime, graph, 1)
            .expect("an unrelated sibling bind cannot invalidate the Text subset");

        graph.data_binds.clear();
        graph.data_binds.push(synthetic_data_bind(
            1,
            graph
                .components
                .iter()
                .find(|component| component.local_id == 1)
                .expect("fixture Text component")
                .global_id,
            "Text",
            property_key_for_name("Text", "sizingValue").expect("sizingValue property"),
        ));

        let Err(error) = StaticTextSlice::from_graph(&runtime, graph, 1) else {
            panic!("unsupported Text-owned bindings must remain fail-closed");
        };
        assert!(
            format!("{error:#}").contains("does not support data binding target Text"),
            "{error:#}"
        );
    }

    #[test]
    fn explicit_line_height_keeps_cxx_first_ascent_and_later_baseline_ratio() {
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
        assert_close(second.bottom - second.top, 40.0);
        assert_ne!(first.bottom - first.top, 40.0);
        assert_ne!(first.baseline - first.top, second.baseline - second.top);
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
    fn default_line_metrics_keep_rive_first_line_ascent_behavior() {
        let (runtime, graphs) =
            baseline_origin_text_runtime_with_sizing_and_line_height(TEXT_SIZING_FIXED, None);
        let graph = graphs.artboards.first().expect("fixture has an artboard");
        let instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).expect("Text slice builds");
        let runs = slice
            .resolved_runs(&runtime, &instance)
            .expect("Text runs resolve");
        let text = runs.iter().map(|run| run.text.as_str()).collect::<String>();
        let lines = split_static_text_lines(&text);
        let line_metrics = slice
            .static_line_metrics(&runtime, &instance, &lines, &runs, 1.0)
            .expect("default line metrics compute");
        let first = line_metrics.first().expect("first line metric");
        let second = line_metrics.get(1).expect("second line metric");

        assert_close(first.baseline, 18.554688);
        assert_close(second.baseline - first.baseline, first.bottom - first.top);
    }

    #[test]
    fn quadratic_outline_conversion_uses_cpp_non_fused_glyph_mapping() {
        let mut pen = TextOutlinePen::new(
            0.1,
            0.0,
            12.3 / TEXT_SHAPE_SCALE_F32,
            0.1,
            0.0,
            Mat2D::IDENTITY,
        );
        let start = (-2047.0, 119.0);
        let control = (-987.0, -37.0);
        let end = (805.0, -91.0);
        let start_outline = TextOutlinePen::normalize_outline_point(start.0, start.1);
        let control_outline = TextOutlinePen::normalize_outline_point(control.0, control.1);
        let t = 2.0 / 3.0;
        let expected_outline = (
            start_outline.0 + (control_outline.0 - start_outline.0) * t,
            start_outline.1 + (control_outline.1 - start_outline.1) * t,
        );
        let expected = pen.map_normalized(expected_outline.0, expected_outline.1);
        let mapped_start = pen.map(start.0, start.1).0;
        let mapped_control = pen.map(control.0, control.1).0;
        let mapped_then_lerped = (
            mapped_start.0 + (mapped_control.0 - mapped_start.0) * t,
            mapped_start.1 + (mapped_control.1 - mapped_start.1) * t,
        );
        assert_eq!(
            (expected.0.to_bits(), expected.1.to_bits()),
            (
                mapped_then_lerped.0.to_bits(),
                mapped_then_lerped.1.to_bits()
            ),
            "the pinned C++ multiply-then-add mapping preserves this affine equality"
        );

        pen.move_to(start.0, start.1);
        pen.quad_to(control.0, control.1, end.0, end.1);
        let RuntimePathCommand::Cubic { x1, y1, .. } = pen.commands[1] else {
            panic!("quadratic outline did not emit a cubic command");
        };
        assert_eq!(
            (x1.to_bits(), y1.to_bits()),
            (expected.0.to_bits(), expected.1.to_bits())
        );
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
            layout_direction: 0,
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
        assert_close(controlled.3, 100.0);
    }

    #[test]
    fn soft_wrap_retains_contextual_advance_from_paragraph_shape() {
        let contextual_glyphs = vec![
            StyledTextGlyph {
                glyph_id: 1,
                char_index: 0,
                char_len: 1,
                style_index: 0,
                advance: 650.0,
                scale: 1.0,
                rtl: false,
                variations: Vec::new(),
            },
            StyledTextGlyph {
                glyph_id: 2,
                char_index: 1,
                char_len: 1,
                style_index: 0,
                advance: 1_194.0,
                scale: 1.0,
                rtl: false,
                variations: Vec::new(),
            },
        ];
        let first_line = StaticTextLine {
            text: "t",
            char_start: 0,
            line_index: 0,
            soft_wrap_skipped_start: None,
            terminal_soft_wrap_skipped_end: None,
        };

        let glyphs = StaticTextSlice::styled_line_glyphs(&first_line, &contextual_glyphs);

        assert_eq!(glyphs.len(), 1);
        assert_eq!(glyphs[0].advance, 650.0);
        assert_eq!(
            StaticTextSlice::styled_line_width(&first_line, &contextual_glyphs),
            650.0
        );
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
                    rtl: false,
                    variations: Vec::new(),
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
                rtl: false,
                variations: Vec::new(),
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
    fn upstream_ltr_rtl_and_mixed_bidi_hit_boundaries_are_ported() {
        let glyph = |char_index, x, advance| StaticPositionedTextGlyph {
            glyph: StyledTextGlyph {
                glyph_id: 1,
                char_index,
                char_len: 1,
                style_index: 0,
                advance,
                scale: 1.0,
                rtl: false,
                variations: Vec::new(),
            },
            x,
            modifier_transform: Mat2D::IDENTITY,
            modifier_opacity: 1.0,
        };
        let layout = |text: &str, glyphs: Vec<StaticPositionedTextGlyph>, boundary_x: &[f32]| {
            let char_end = text.chars().count();
            let line = StaticShapedTextLine {
                line_index: 0,
                char_start: 0,
                char_end,
                soft_wrap_skipped_start: None,
                terminal_soft_wrap_skipped_end: None,
                start_x: 0.0,
                end_x: 40.0,
                top: 0.0,
                baseline: 10.0,
                bottom: 12.0,
                glyphs,
            };
            let lines = vec![line];
            let boundaries = text
                .char_indices()
                .map(|(byte, _)| byte)
                .chain(std::iter::once(text.len()))
                .zip(boundary_x.iter().copied())
                .map(|(byte_offset, x)| {
                    let segment = StaticCaretSegment {
                        top: RenderVec2D::new(x, 0.0),
                        bottom: RenderVec2D::new(x, 12.0),
                    };
                    StaticCaretBoundary {
                        byte_offset,
                        upstream: Some(segment),
                        downstream: Some(segment),
                    }
                })
                .collect();
            StaticShapedTextLayout {
                text: text.to_owned(),
                lines,
                caret_boundaries: Some(boundaries),
                local_transform: Mat2D::IDENTITY,
                shape_world: Mat2D::IDENTITY,
                has_geometric_modifiers: false,
                has_non_monotone_advances: true,
            }
        };

        let ltr = layout(
            "abcd",
            vec![
                glyph(0, 0.0, 10.0),
                glyph(1, 10.0, 10.0),
                glyph(2, 20.0, 10.0),
                glyph(3, 30.0, 10.0),
            ],
            &[0.0, 10.0, 20.0, 30.0, 40.0],
        );
        assert_eq!(ltr.hit(RenderVec2D::new(-20.0, 6.0)), Some(0));
        assert_eq!(ltr.hit(RenderVec2D::new(60.0, 6.0)), Some(4));

        let rtl = layout(
            "اربك",
            vec![
                glyph(0, 40.0, -10.0),
                glyph(1, 30.0, -10.0),
                glyph(2, 20.0, -10.0),
                glyph(3, 10.0, -10.0),
            ],
            &[40.0, 30.0, 20.0, 10.0, 0.0],
        );
        assert_eq!(rtl.hit(RenderVec2D::new(-20.0, 6.0)), Some(8));
        assert_eq!(rtl.hit(RenderVec2D::new(60.0, 6.0)), Some(0));

        let mixed = layout(
            "abرب",
            vec![
                glyph(0, 0.0, 10.0),
                glyph(1, 10.0, 10.0),
                glyph(2, 40.0, -10.0),
                glyph(3, 30.0, -10.0),
            ],
            &[0.0, 10.0, 40.0, 30.0, 20.0],
        );
        assert_eq!(mixed.hit(RenderVec2D::new(-20.0, 6.0)), Some(0));
        assert_eq!(mixed.hit(RenderVec2D::new(60.0, 6.0)), Some(2));
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

    #[test]
    fn static_text_preflight_accepts_runtime_scroll_bindings() {
        for property_name in [
            "scrollOffsetX",
            "scrollOffsetY",
            "scrollPercentX",
            "scrollPercentY",
            "scrollIndex",
        ] {
            let data_bind = DataBindNode {
                global_id: 1,
                type_name: "DataBind",
                property_key: u64::from(
                    property_key_for_name("ScrollConstraint", property_name)
                        .expect("scroll property exists"),
                ),
                flags: 0,
                converter_id: 0,
                converter_global: None,
                converter_type_name: None,
                converter_duration: None,
                target_global: Some(2),
                target_type_name: Some("ScrollConstraint"),
                target_local: Some(2),
            };
            assert!(
                static_text_data_bind_supported(&data_bind),
                "{property_name} should reach the runtime scroll binding path"
            );
        }
    }

    #[test]
    fn static_text_preflight_accepts_supported_key_frame_value_bindings() {
        for target_type_name in [
            "KeyFrameDouble",
            "KeyFrameColor",
            "KeyFrameBool",
            "KeyFrameString",
        ] {
            let data_bind = DataBindNode {
                global_id: 1,
                type_name: "DataBind",
                property_key: u64::from(
                    property_key_for_name(target_type_name, "value")
                        .expect("keyframe value property exists"),
                ),
                flags: 0,
                converter_id: 0,
                converter_global: None,
                converter_type_name: None,
                converter_duration: None,
                target_global: Some(2),
                target_type_name: Some(target_type_name),
                target_local: Some(2),
            };
            assert!(
                static_text_data_bind_supported(&data_bind),
                "{target_type_name}.value should reach the runtime keyframe binding path"
            );
        }
    }

    fn fl_e8_fixture(bytes: &[u8]) -> (RuntimeFile, GraphFile, ArtboardInstance) {
        let runtime = nuxie_binary::read_runtime_file(bytes).expect("FL-E8 fixture imports");
        let graphs = GraphFile::from_runtime_file(&runtime).expect("FL-E8 fixture graph builds");
        let graph = graphs.artboards.first().expect("FL-E8 fixture artboard");
        let instance =
            ArtboardInstance::from_graph(&runtime, graph).expect("FL-E8 instance builds");
        (runtime, graphs, instance)
    }

    // R-ST-OWNER: focused occurrence/source ratchets for the five WP1 owners.
    #[test]
    fn d_st_struct_registers_generic_modifiers_and_shape_subtype_indices_in_authored_order() {
        let (runtime, graphs, instance) = fl_e8_fixture(include_bytes!(
            "../../../fixtures/fl-e8/text_variation_modifier.riv"
        ));
        let graph = graphs.artboards.first().unwrap();
        let report = static_text_layout_debug_report(&runtime, graph, &instance, 1, None).unwrap();
        assert_eq!(report.modifier_groups.len(), 1);
        assert_eq!(report.modifier_groups[0].modifier_locals, [9, 10]);
        assert_eq!(report.modifier_groups[0].shape_modifier_indices, [0, 1]);
        assert!(
            report.modifier_groups[0]
                .coverage
                .iter()
                .any(|value| *value == 0.0)
        );
        assert!(
            report.modifier_groups[0]
                .coverage
                .iter()
                .any(|value| *value != 0.0)
        );
    }

    #[test]
    fn d_st_feature_preserves_order_defaults_duplicates_and_live_callback_inaction() {
        let (runtime, graphs, mut instance) = fl_e8_fixture(include_bytes!(
            "../../../fixtures/fl-e8/text_style_feature.riv"
        ));
        let graph = graphs.artboards.first().unwrap();
        let before = static_text_layout_debug_report(&runtime, graph, &instance, 1, None).unwrap();
        let liga = u32::from_be_bytes(*b"liga");
        assert_eq!(before.style_features[0], [(liga, 1), (liga, 0), (liga, 1)]);
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).unwrap();
        let style = &slice.styles[0];
        let feature_local = style.features.last().unwrap().local_id;
        assert!(instance.set_uint_property(
            feature_local,
            property_key_for_name("TextStyleFeature", "featureValue").unwrap(),
            0,
        ));
        let after = static_text_layout_debug_report(&runtime, graph, &instance, 1, None).unwrap();
        assert_eq!(after.style_features, before.style_features);
        assert_eq!(after.line_glyph_ids, before.line_glyph_ids);

        let font_bytes = style.font_bytes(&runtime, &instance).unwrap().to_vec();
        assert!(instance.debug_set_text_style_font_bytes(style.local_id, font_bytes));
        let reshaped =
            static_text_layout_debug_report(&runtime, graph, &instance, 1, None).unwrap();
        assert_eq!(
            reshaped.style_features[0],
            [(liga, 1), (liga, 0), (liga, 0)]
        );
        assert_ne!(reshaped.line_glyph_ids, before.line_glyph_ids);
    }

    #[test]
    fn d_st_variation_splits_coverage_and_applies_duplicate_unclamped_interpolation() {
        let (runtime, graphs, mut instance) = fl_e8_fixture(include_bytes!(
            "../../../fixtures/fl-e8/text_variation_modifier.riv"
        ));
        let graph = graphs.artboards.first().unwrap();
        let before = static_text_layout_debug_report(&runtime, graph, &instance, 1, None).unwrap();
        let untouched = ArtboardInstance::from_graph(&runtime, graph).unwrap();
        let wght = u32::from_be_bytes(*b"wght");
        assert!(before.line_glyph_variations.iter().flatten().any(|axes| {
            axes.iter()
                .any(|(tag, value)| *tag == wght && value.is_finite())
        }));
        assert!(instance.set_double_property(
            10,
            property_key_for_name("TextVariationModifier", "axisValue").unwrap(),
            900.0,
        ));
        let after = static_text_layout_debug_report(&runtime, graph, &instance, 1, None).unwrap();
        assert_ne!(after.line_glyph_variations, before.line_glyph_variations);
        let untouched_report =
            static_text_layout_debug_report(&runtime, graph, &untouched, 1, None).unwrap();
        assert_eq!(
            untouched_report.line_glyph_variations,
            before.line_glyph_variations
        );
        let mut slice = StaticTextSlice::from_graph(&runtime, graph, 1).unwrap();
        let group = slice.modifiers.remove(0);
        let style = slice.base_style().unwrap();
        let font = SkrifaFontRef::new(style.font_bytes(&runtime, &instance).unwrap()).unwrap();
        let inherited = BTreeMap::from([(wght, 400.0)]);
        assert_eq!(
            group
                .variation_map(&untouched, &font, 1.25, &inherited)
                .get(&wght),
            Some(&181.25)
        );
        assert_eq!(
            group
                .variation_map(&untouched, &font, -0.5, &inherited)
                .get(&wght),
            Some(&225.0)
        );
        assert_eq!(
            group
                .variation_map(&untouched, &font, 0.0, &inherited)
                .get(&wght),
            Some(&400.0)
        );
        let modifier_dirt = instance.debug_component_dirt(10);
        let group_dirt = instance.debug_component_dirt(7);
        let text_dirt = instance.debug_component_dirt(1);
        assert!(instance.set_uint_property(
            10,
            property_key_for_name("TextVariationModifier", "axisTag").unwrap(),
            u64::from(u32::from_be_bytes(*b"wdth")),
        ));
        assert_eq!(instance.debug_component_dirt(10), modifier_dirt);
        assert_eq!(instance.debug_component_dirt(7), group_dirt);
        assert_eq!(instance.debug_component_dirt(1), text_dirt);
        assert!(instance.set_uint_property(
            3,
            property_key_for_name("TextStyleAxis", "tag").unwrap(),
            u64::from(u32::from_be_bytes(*b"wdth")),
        ));
        assert!(
            instance.debug_component_dirt(3).is_some_and(|dirt| {
                dirt.contains(crate::components::ComponentDirt::TEXT_SHAPE)
            })
        );
        let glyph = StaticTextGlyphContext {
            origin_x: 0.0,
            origin_y: 0.0,
            line_index_in_paragraph: 0,
            paragraph_baselines: &[],
            text_world_inverse: Mat2D::IDENTITY,
        };
        let transform = group.transform(&runtime, &instance, 1.0, &glyph).unwrap();
        assert!((transform.0[4] - 0.05).abs() < 1e-6);
        assert!((transform.0[5] + 0.15).abs() < 1e-6);
    }

    #[test]
    fn d_st_font_live_swap_invalidates_the_retained_text_owner() {
        let (runtime, graphs) = baseline_origin_text_runtime();
        let graph = graphs.artboards.first().unwrap();
        let mut instance = ArtboardInstance::from_graph(&runtime, graph).unwrap();
        let slice = StaticTextSlice::from_graph(&runtime, graph, 1).unwrap();
        let style = slice.base_style().unwrap();
        let before = instance.layout_revision();
        let mut font = RuntimeFontAssetValue::default();
        assert!(font.set_live_font_bytes(Some(fixture_font_bytes().into())));
        assert!(instance.set_text_style_font_override(style.local_id, font));
        assert!(instance.layout_revision() > before);
    }

    #[test]
    fn d_st_target_missing_resolution_is_ok_and_retains_no_target() {
        let (runtime, graphs, _) = fl_e8_fixture(include_bytes!(
            "../../../fixtures/fl-e8/text_variation_modifier.riv"
        ));
        let graph = graphs.artboards.first().unwrap();
        let target = StaticTextTargetModifier::from_graph(&runtime, graph, 9, 7).unwrap();
        assert_eq!(target.target_id, u32::MAX);
        assert_eq!(target.resolved_transform_local, None);
        assert_eq!(target.group_local, 7);
    }

    #[test]
    fn d_st_target_wrong_parent_is_rejected_before_occurrence_registration() {
        let runtime = RuntimeFile::from_authoring_records(vec![
            authoring_record("Backboard", Vec::new()),
            authoring_record("Artboard", Vec::new()),
            authoring_record("Text", Vec::new()),
            authoring_record(
                "TextStylePaint",
                vec![property(
                    "TextStylePaint",
                    "parentId",
                    AuthoringValue::Uint(1),
                )],
            ),
            authoring_record(
                "TextValueRun",
                vec![
                    property("TextValueRun", "parentId", AuthoringValue::Uint(1)),
                    property(
                        "TextValueRun",
                        "text",
                        AuthoringValue::String("wrong parent".to_owned()),
                    ),
                    property("TextValueRun", "styleId", AuthoringValue::Uint(2)),
                ],
            ),
            authoring_record(
                "TextFollowPathModifier",
                vec![property(
                    "TextFollowPathModifier",
                    "parentId",
                    AuthoringValue::Uint(1),
                )],
            ),
        ])
        .unwrap();
        let graphs = GraphFile::from_runtime_file(&runtime).unwrap();
        let error = match StaticTextSlice::from_graph(&runtime, &graphs.artboards[0], 1) {
            Ok(_) => panic!("wrong-parent TextTargetModifier must be rejected"),
            Err(error) => error,
        };
        assert!(
            error
                .to_string()
                .contains("direct TextModifierGroup parent")
        );
    }

    #[test]
    fn d_rt_engine_shared_classifier_extracts_colr_and_raster_layers() {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        );
        let colr_bytes =
            std::fs::read(root.join("tests/unit_tests/assets/TwemojiMozilla.subset.ttf"))
                .expect("read COLR differential font");
        let colr_font = SkrifaFontRef::new(&colr_bytes).expect("parse COLR font");
        let heart = (0..4096)
            .find(|glyph| colr_font.color_glyphs().get(GlyphId::new(*glyph)).is_some())
            .expect("Twemoji contains a COLR glyph");
        assert_eq!(
            runtime_classify_color_glyph(&colr_bytes, heart),
            RuntimeColorGlyphClassification::Colr
        );
        assert!(!runtime_extract_color_glyph_layers(&colr_bytes, heart, 0xff12_3456).is_empty());

        let raster_bytes =
            std::fs::read(root.join("skia/dependencies/skia/resources/fonts/sbix.ttf"))
                .expect("read raster differential font");
        let raster_glyph = (0..256)
            .find(|glyph| {
                runtime_classify_color_glyph(&raster_bytes, *glyph)
                    == RuntimeColorGlyphClassification::Raster
            })
            .expect("sbix fixture contains a raster glyph");
        let layers = runtime_extract_color_glyph_layers(&raster_bytes, raster_glyph, 0xff00_0000);
        assert!(
            layers
                .iter()
                .any(|layer| { matches!(layer.paint, RuntimeColorGlyphPaint::Image { .. }) })
        );
    }
}
