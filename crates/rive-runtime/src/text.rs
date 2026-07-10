use anyhow::{Context, Result, bail};
use harfrust::{
    Direction, Feature, FontRef as HarfFontRef, ShapeOptions, ShaperData, ShaperInstance,
    Tag as HarfTag, UnicodeBuffer,
};
use rive_binary::RuntimeFile;
use rive_graph::{
    ArtboardGraph, DataBindNode, PathGeometryNode, ShapePaintContainerNode, ShapePaintKind,
    ShapePaintPathKind, ShapePaintStateNode,
};
use rive_schema::definition_by_name;
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
const TEXT_OVERFLOW_ELLIPSIS: u64 = 3;
const TEXT_OVERFLOW_FIT_FONT_SIZE: u64 = 5;
const TEXT_TRIM_NONE: u64 = 0;
const TEXT_TRIM_TOP_CAP: u64 = 1;
const TEXT_TRIM_TOP_EX: u64 = 2;
const TEXT_TRIM_BOTTOM_ALPHABETIC: u64 = 1;
const TEXT_TRIM_BOTTOM_TEXT: u64 = 2;
const LAYOUT_SCALE_TYPE_HUG: u64 = 2;

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
    StaticTextSlice::from_graph(runtime, graph, text_local)
        .ok()?
        .local_bounds(runtime, instance)
        .ok()
        .flatten()
}

pub(crate) fn static_text_layout_measure_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    StaticTextSlice::from_graph(runtime, graph, text_local)
        .ok()?
        .local_bounds_with_layout_constraint(runtime, instance, layout_constraint)
        .ok()
        .flatten()
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
        .local_bounds_with_layout_constraint(runtime, instance, layout_constraint)
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

    let mut commands = Vec::new();
    for (style, path_buckets) in slice.styles.iter().zip(render_data.path_buckets_by_style) {
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
    container: Option<&'a ShapePaintContainerNode>,
    font_bytes: Option<&'a [u8]>,
    variations: Vec<(u32, f32)>,
}

#[derive(Debug, Clone, Copy)]
struct StaticTextLine<'a> {
    text: &'a str,
    char_start: usize,
    line_index: usize,
}

#[derive(Debug, Clone, Copy)]
struct StaticTextLayoutInfo {
    ellipsis_line: Option<usize>,
    is_ellipsis_line_last: bool,
    paragraph_width: f32,
    align_value: u64,
    top_trim: f32,
    x_offset: f32,
    y_offset: f32,
}

#[derive(Debug, Clone, Copy, Default)]
struct StaticVerticalTrim {
    top: f32,
    bottom: f32,
}

impl StaticVerticalTrim {
    fn apply_to_height(self, height: f32) -> f32 {
        (height - self.top - self.bottom).max(0.0)
    }
}

#[derive(Debug, Clone)]
struct StaticTextRenderData {
    path_buckets_by_style: Vec<Vec<StaticTextPathBucket>>,
    local_transform: Mat2D,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeTextLayoutConstraint {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) width_scale_type: u64,
    pub(crate) height_scale_type: u64,
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
struct StaticTextGlyphContext {
    origin_x: f32,
    origin_y: f32,
    line_index_in_paragraph: usize,
    paragraph_baselines: Vec<f32>,
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
                        | "ScriptedDrawable"
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
                        | "ViewModelInstanceViewModel"
                        | "ViewModelPropertyNumber"
                        | "ViewModelPropertyString"
                        | "ViewModelPropertyViewModel"
                        | "Event"
                        | "StateMachine"
                        | "StateMachineLayer"
                        | "StateMachineBool"
                        | "ListenerBoolChange"
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
        if run_locals.is_empty() {
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

    fn render_data(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: Option<RuntimeTextLayoutConstraint>,
        text_world: Mat2D,
    ) -> Result<StaticTextRenderData> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        let text = resolved_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let base_style = self.base_style()?;
        let font_size = self.style_font_size(runtime, instance, base_style)?;
        if text.is_empty() || font_size < 0.0 {
            return Ok(StaticTextRenderData {
                path_buckets_by_style: vec![Vec::new(); self.styles.len()],
                local_transform: Mat2D::IDENTITY,
            });
        }
        let letter_spacing = self.letter_spacing(runtime, instance);
        let Some(font_bytes) = base_style.font_bytes else {
            // Mirrors src/importers/file_asset_importer.cpp: with no
            // FileAssetLoader and no in-band contents, a hosted FontAsset
            // resolves successfully but has no decoded font.
            return Ok(StaticTextRenderData {
                path_buckets_by_style: vec![Vec::new(); self.styles.len()],
                local_transform: Mat2D::IDENTITY,
            });
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
        let skrifa_variations = self
            .base_style()?
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = skrifa_font
            .axes()
            .location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (ascent, descent) = harfbuzz_line_metrics(&skrifa_font, location_ref);
        let font_scale = self.fit_font_scale(
            runtime,
            instance,
            layout_constraint,
            &resolved_runs,
            &text,
            &shaper,
            disable_legacy_kern,
        )?;
        let scaled_font_size = font_size * font_scale;
        let (baseline, line_height) = self
            .max_static_line_metrics(runtime, instance, font_scale, Some(&resolved_runs))?
            .unwrap_or_else(|| {
                (
                    ascent * scaled_font_size / TEXT_SHAPE_SCALE_F32,
                    (ascent - descent) * scaled_font_size / TEXT_SHAPE_SCALE_F32,
                )
            });
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
        let measured_width = lines
            .iter()
            .map(|line| self.styled_line_width(runtime, instance, line, &resolved_runs, font_scale))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold(0.0f32, f32::max);
        let layout_info = self.static_layout_info(
            runtime,
            instance,
            &lines,
            &resolved_runs,
            baseline,
            line_height,
            measured_width,
            font_scale,
            apply_ellipsis,
            layout_constraint,
        )?;
        let local_transform = Mat2D([
            1.0,
            0.0,
            0.0,
            1.0,
            layout_info.x_offset,
            layout_info.y_offset,
        ]);
        let text_world_inverse = text_world.invert_or_identity();
        let paragraph_baselines = lines
            .iter()
            .map(|line| baseline + line.line_index as f32 * line_height - layout_info.top_trim)
            .collect::<Vec<_>>();
        let mut commands_by_style = vec![Vec::new(); self.styles.len()];
        let modifier_coverages = self
            .modifiers
            .iter()
            .map(|modifier| {
                modifier.coverage_by_character(runtime, instance, &text, &resolved_runs, &lines)
            })
            .collect::<Result<Vec<_>>>()?;
        for line in lines {
            if let Some(ellipsis_line) = layout_info.ellipsis_line {
                if line.line_index > ellipsis_line {
                    break;
                }
            }
            if line.text.is_empty() {
                continue;
            }
            let mut glyphs =
                self.styled_line_glyphs(runtime, instance, &line, &resolved_runs, font_scale)?;
            if layout_info.ellipsis_line == Some(line.line_index) {
                let max_width = self.effective_width(runtime, instance, layout_constraint)?;
                let ellipsis_style = glyphs.last().map(|glyph| glyph.style_index).unwrap_or(0);
                let ellipsis = self.styled_text_glyphs_for_style(
                    runtime,
                    instance,
                    "...",
                    line.char_start + line.text.chars().count(),
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
                        line_height,
                    )?
                {
                    glyphs.truncate(line_end);
                    force_ellipsis = true;
                }
                apply_static_ellipsis(&mut glyphs, ellipsis, max_width, force_ellipsis);
            }

            let line_width = glyphs.iter().map(|glyph| glyph.advance).sum();
            let mut cursor_x = layout_info.line_start_x(line_width);
            let line_baseline =
                baseline + line.line_index as f32 * line_height - layout_info.top_trim;
            for glyph in &glyphs {
                let mut glyph_transform = Mat2D::IDENTITY;
                let mut glyph_opacity = 1.0;
                let center_x = cursor_x + glyph.advance * 0.5;
                let glyph_context = StaticTextGlyphContext {
                    origin_x: center_x,
                    origin_y: line_baseline,
                    line_index_in_paragraph: line.line_index,
                    paragraph_baselines: paragraph_baselines.clone(),
                    text_world_inverse,
                };
                for (modifier, coverage) in self.modifiers.iter().zip(&modifier_coverages) {
                    let amount = glyph_coverage(coverage, glyph.char_index, glyph.char_len);
                    if amount != 0.0 {
                        glyph_transform = modifier
                            .transform(runtime, instance, amount, &glyph_context)?
                            .multiply(glyph_transform);
                    }
                    if modifier.modifies_opacity(runtime, instance)? {
                        glyph_opacity =
                            modifier.opacity(runtime, instance, glyph_opacity, amount)?;
                    }
                }
                let glyph_id = GlyphId::new(glyph.glyph_id);
                let style = &self.styles[glyph.style_index];
                if let Some(style_font_bytes) = style.font_bytes {
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
                            cursor_x,
                            line_baseline,
                            glyph.scale,
                            center_x,
                            line_baseline,
                            glyph_transform,
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
                            glyph_opacity,
                            pen.commands,
                        );
                    }
                }
                cursor_x += glyph.advance;
            }
        }

        Ok(StaticTextRenderData {
            path_buckets_by_style: commands_by_style,
            local_transform,
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
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
        }
        let Some(font_bytes) = base_style.font_bytes else {
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
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
        let skrifa_variations = self
            .base_style()?
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = skrifa_font
            .axes()
            .location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (ascent, descent) = harfbuzz_line_metrics(&skrifa_font, location_ref);
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
        let (baseline, line_height) = self
            .max_static_line_metrics(runtime, instance, font_scale, Some(&resolved_runs))?
            .unwrap_or_else(|| {
                (
                    ascent * scaled_font_size / TEXT_SHAPE_SCALE_F32,
                    (ascent - descent) * scaled_font_size / TEXT_SHAPE_SCALE_F32,
                )
            });
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
        let sizing = self.text_uint_property(runtime, instance, "sizingValue")?;
        let width = match sizing {
            1 | 2 => self.text_width(runtime, instance)?,
            _ => measured_width,
        };
        let measured_height = line_height * lines.len().max(1) as f32;
        let height = match sizing {
            TEXT_SIZING_FIXED => self.text_height(runtime, instance)?,
            _ => self
                .static_vertical_trim(
                    runtime,
                    instance,
                    &lines,
                    &resolved_runs,
                    baseline,
                    line_height,
                    font_scale,
                )?
                .apply_to_height(measured_height),
        };
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;

        Ok(Some((-width * origin_x, -height * origin_y, width, height)))
    }

    fn local_bounds_with_layout_constraint(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        layout_constraint: RuntimeTextLayoutConstraint,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        let resolved_runs = self.resolved_runs(runtime, instance)?;
        let text = resolved_runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<String>();
        let base_style = self.base_style()?;
        let font_size = self.style_font_size(runtime, instance, base_style)?;
        if text.is_empty() || font_size < 0.0 {
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
        }
        let Some(font_bytes) = base_style.font_bytes else {
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
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
        let skrifa_variations = self
            .base_style()?
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = skrifa_font
            .axes()
            .location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (ascent, descent) = harfbuzz_line_metrics(&skrifa_font, location_ref);
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
        let (baseline, line_height) = self
            .max_static_line_metrics(runtime, instance, font_scale, Some(&resolved_runs))?
            .unwrap_or_else(|| {
                (
                    ascent * scaled_font_size / TEXT_SHAPE_SCALE_F32,
                    (ascent - descent) * scaled_font_size / TEXT_SHAPE_SCALE_F32,
                )
            });
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
        let measured_width = lines
            .iter()
            .map(|line| self.styled_line_width(runtime, instance, line, &resolved_runs, font_scale))
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .fold(0.0f32, f32::max);
        let sizing = self.text_uint_property(runtime, instance, "sizingValue")?;
        let width = match sizing {
            TEXT_SIZING_AUTO_HEIGHT | TEXT_SIZING_FIXED => self.text_width(runtime, instance)?,
            _ => measured_width,
        };
        let measured_height = line_height * lines.len().max(1) as f32;
        let height = match sizing {
            TEXT_SIZING_FIXED => self.text_height(runtime, instance)?,
            _ => self
                .static_vertical_trim(
                    runtime,
                    instance,
                    &lines,
                    &resolved_runs,
                    baseline,
                    line_height,
                    font_scale,
                )?
                .apply_to_height(measured_height),
        };
        Ok(Some((
            0.0,
            0.0,
            width.min(layout_constraint.width),
            height.min(layout_constraint.height),
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
        Ok(runs)
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

    fn max_static_line_metrics(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        font_scale: f32,
        runs: Option<&[StaticResolvedRun]>,
    ) -> Result<Option<(f32, f32)>> {
        let used_styles = runs.map(|runs| {
            runs.iter()
                .map(|run| run.style_local)
                .collect::<BTreeSet<_>>()
        });
        let mut baseline = 0.0f32;
        let mut line_height = 0.0f32;
        for style in &self.styles {
            if used_styles
                .as_ref()
                .is_some_and(|styles| !styles.contains(&style.local_id))
            {
                continue;
            }
            let Some(font_bytes) = style.font_bytes else {
                continue;
            };
            let font =
                SkrifaFontRef::new(font_bytes).context("failed to parse font for line metrics")?;
            let skrifa_variations = style
                .variations
                .iter()
                .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
                .collect::<Vec<_>>();
            let location = font.axes().location(skrifa_variations.iter().copied());
            let location_ref = LocationRef::from(&location);
            let (ascent, descent) = harfbuzz_line_metrics(&font, location_ref);
            let scale =
                self.style_font_size(runtime, instance, style)? * font_scale / TEXT_SHAPE_SCALE_F32;
            baseline = baseline.max(ascent * scale);
            line_height = line_height.max((ascent - descent) * scale);
        }
        Ok((line_height > 0.0).then_some((baseline, line_height)))
    }

    fn static_vertical_trim(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        lines: &[StaticTextLine<'_>],
        resolved_runs: &[StaticResolvedRun],
        baseline: f32,
        line_height: f32,
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
                let line_top = first_line.line_index as f32 * line_height;
                let line_baseline = baseline + first_line.line_index as f32 * line_height;
                trim.top = ((line_baseline - edge_px) - line_top).max(0.0);
            }
        }

        if matches!(
            trim_bottom,
            TEXT_TRIM_BOTTOM_ALPHABETIC | TEXT_TRIM_BOTTOM_TEXT
        ) {
            if let Some(last_line) = lines.iter().rev().find(|line| !line.text.is_empty()) {
                let line_baseline = baseline + last_line.line_index as f32 * line_height;
                let line_bottom = (last_line.line_index + 1) as f32 * line_height;
                let descent_band = line_bottom - line_baseline;
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
        let Some(font_bytes) = style.font_bytes else {
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
        let Some(font_bytes) = style.font_bytes else {
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
            if style.font_bytes.is_some() {
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
            let line_height = self
                .max_static_line_metrics(runtime, instance, font_scale, Some(runs))?
                .map(|(_, line_height)| line_height)
                .unwrap_or(0.0);
            let height = line_height * lines.len().max(1) as f32;
            Ok(max_width <= box_width && (sizing != TEXT_SIZING_FIXED || height <= box_height))
        };

        let mut lo = 1i32;
        let mut hi = (max_size as i32).max(1);
        let mut best = 1i32;
        while lo <= hi {
            let mid = lo + (hi - lo) / 2;
            if fits(mid)? {
                best = mid;
                lo = mid + 1;
            } else {
                hi = mid - 1;
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
        let Some(font_bytes) = style.font_bytes else {
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
        if self.kind == StaticTextKind::TextInput {
            return Ok(if self.text_input_multiline(runtime, instance)? {
                TEXT_SIZING_AUTO_HEIGHT
            } else {
                TEXT_SIZING_AUTO_WIDTH
            });
        }
        let authored = self.text_uint_property(runtime, instance, "sizingValue")?;
        Ok(layout_constraint
            .map(|constraint| constraint.effective_sizing(authored))
            .unwrap_or(authored))
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
                });
                line_index += 1;
                continue;
            }

            let mut remaining = authored_line.text;
            let mut char_start = authored_line.char_start;
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
                lines.push(StaticTextLine {
                    text: line_text,
                    char_start,
                    line_index,
                });
                line_index += 1;

                if byte_end >= remaining.len() {
                    break;
                }
                let skipped = leading_whitespace_bytes(&remaining[byte_end..]);
                char_start += remaining[..byte_end + skipped].chars().count();
                remaining = &remaining[byte_end + skipped..];
            }
        }

        Ok(lines)
    }

    fn static_layout_info(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        lines: &[StaticTextLine<'_>],
        resolved_runs: &[StaticResolvedRun],
        baseline: f32,
        line_height: f32,
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
                resolved_runs,
                baseline,
                line_height,
                font_scale,
            )?
        };
        let bounds_height = match sizing {
            TEXT_SIZING_FIXED => self.effective_height(runtime, instance, layout_constraint)?,
            _ => {
                let measured_height = line_height * lines.len().max(1) as f32;
                vertical_trim.apply_to_height(measured_height)
            }
        };
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;
        let align_value = self.text_uint_property(runtime, instance, "alignValue")?;
        let last_line_index = lines.last().map(|line| line.line_index).unwrap_or(0);
        let full_height = line_height * (last_line_index + 1) as f32;
        let mut total_height = full_height;
        let mut ellipsis_line = None;
        let mut is_ellipsis_line_last = false;

        if apply_ellipsis && !lines.is_empty() {
            // Mirrors src/text/text.cpp::computeBoundsInfo for the static text
            // subset: choose the last visual line whose bottom fits the fixed
            // box, falling back to the first line when nothing fits.
            let mut ellipsed_height = 0.0;
            for line in lines {
                let line_bottom = line_height * (line.line_index + 1) as f32;
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
            x_offset: -bounds_width * origin_x,
            y_offset,
        })
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
        line_height: f32,
    ) -> Result<bool> {
        let height = self.effective_height(runtime, instance, layout_constraint)?;
        Ok(line_height > 0.0 && height > 0.0 && height < line_height * 2.0)
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
        let (_, line_height) = self
            .max_static_line_metrics(runtime, instance, 1.0, None)?
            .unwrap_or((0.0, 0.0));
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
        let font_bytes = if style.property("fontAssetId").is_some() {
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
            embedded_file_asset_bytes(runtime, font_asset.id)
        } else {
            None
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
            container,
            font_bytes,
            variations,
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
        glyph: &StaticTextGlyphContext,
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
        glyph: &StaticTextGlyphContext,
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

fn order_opacity_buckets_like_cpp(buckets: Vec<StaticTextPathBucket>) -> Vec<StaticTextPathBucket> {
    if buckets.len() < 2 {
        return buckets;
    }

    // C++ TextStylePaint buckets opacity paths in std::unordered_map<float,
    // ShapePaintPath>. The golden stream observes libc++'s small-map insertion
    // order: new empty buckets are linked at the front, while collisions are
    // inserted before the first node in that bucket.
    let mut bucket_count = 0usize;
    let mut ordered_indices = Vec::<usize>::new();
    for index in 0..buckets.len() {
        let size_after_insert = index + 1;
        if size_after_insert > bucket_count {
            bucket_count = next_cpp_unordered_bucket_count(bucket_count, size_after_insert);
        }
        let bucket_index = cpp_float_hash_bucket(buckets[index].opacity, bucket_count);
        let insertion_index = ordered_indices
            .iter()
            .position(|existing| {
                cpp_float_hash_bucket(buckets[*existing].opacity, bucket_count) == bucket_index
            })
            .unwrap_or(0);
        ordered_indices.insert(insertion_index, index);
    }

    ordered_indices
        .into_iter()
        .map(|index| buckets[index].clone())
        .collect()
}

fn cpp_float_hash_bucket(value: f32, bucket_count: usize) -> usize {
    (value.to_bits() as usize) % bucket_count
}

fn next_cpp_unordered_bucket_count(current: usize, desired: usize) -> usize {
    if current == 0 {
        return 2;
    }
    // Guard the rehash growth `current * 2` against usize overflow: in debug it
    // panics, in release it wraps -- a build-dependent divergence. Bucket counts
    // are bounded by the real element (glyph/run) count, so saturation is
    // unreachable on any input; saturating_mul just makes debug and release
    // agree deterministically.
    next_prime(current.saturating_mul(2).max(desired))
}

fn next_prime(mut value: usize) -> usize {
    while !is_prime(value) {
        value += 1;
    }
    value
}

fn is_prime(value: usize) -> bool {
    if value < 2 {
        return false;
    }
    let mut divisor = 2usize;
    // `divisor * divisor` can overflow usize when `value` is near usize::MAX
    // (divisor climbs toward sqrt(value)); saturating_mul makes the last
    // comparison `saturated > value` cleanly false and exits the loop, whereas a
    // wrapping product could wrap below `value` and loop past the real sqrt.
    while divisor.saturating_mul(divisor) <= value {
        if value % divisor == 0 {
            return false;
        }
        divisor += 1;
    }
    true
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

    if line_start_byte < text.len() || lines.is_empty() {
        lines.push(StaticTextLine {
            text: &text[line_start_byte..],
            char_start: line_start_char,
            line_index,
        });
    }
    lines
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
) -> Option<&rive_graph::ComponentNode> {
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
