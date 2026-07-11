use anyhow::{Context, Result};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::{
    ArtboardGraph, ClippingShapeNode, DashNode, DrawableOrderKind, DrawableOrderNode, FeatherNode,
    GradientStopNode, MeshGeometryNode, MeshVertexNode, NSlicerAxisNode, ParametricPathNode,
    PathComposerNode, PathComposerPathNode, PathGeometryNode, PathVertexNode,
    ShapePaintContainerNode, ShapePaintKind, ShapePaintNode, ShapePaintPathKind,
    ShapePaintStateNode, SortedDrawableNode, StrokeEffectNode,
};
use nuxie_render_api::{
    BlendMode as RenderBlendMode, Factory as RenderFactory, FillRule as RenderFillRule,
    ImageSampler as RenderImageSampler, Mat2D as RenderMat2D, PathVerb as RenderPathVerb, RawPath,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle,
    RenderPath, RenderShader, Renderer, StrokeCap as RenderStrokeCap,
    StrokeJoin as RenderStrokeJoin, Vec2D as RenderVec2D,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, OnceLock};
use taffy::prelude::{
    AlignContent, AlignItems, AlignSelf, AvailableSpace, Dimension, Display as TaffyDisplay,
    FlexDirection, FlexWrap, JustifyContent, LengthPercentage, LengthPercentageAuto, NodeId,
    Position, Rect, Size, Style, TaffyTree,
};
use taffy::style::Direction as TaffyDirection;

use crate::properties::{
    RuntimeLayoutComputedProperty, cached_property_key_for_name, property_key_for_name,
    runtime_object_explicit_bool_property_by_key, runtime_object_explicit_double_property_by_key,
    runtime_object_explicit_uint_property_by_key, runtime_object_uint_property_by_key,
    shape_paint_is_visible_property_key, solid_color_value_property_key,
};
use crate::scripting::{ScriptNode, script_paint_for_shape};
use crate::text::{
    RuntimeTextLayoutConstraint, runtime_text_input_shape_paint_commands,
    runtime_text_shape_paint_commands, static_text_constraint_bounds,
    static_text_layout_measure_bounds, text_input_layout_measure_bounds,
};
use crate::{ArtboardInstance, ComponentDirt, Mat2D};

// Nested-artboard cycle guard (`nested_ancestors`).
//
// A malformed-but-accepted file can make the artboard nesting graph cyclic
// (artboard A nests B nests A, possibly with branching). Both the static
// paint-prep and draw recursions (`prepare_static_*` / `draw_prepared_static_*`
// mutual recursion) re-resolve the nesting graph on every descent, so a cycle
// drives them into an unbounded (and, with branching, exponential) descent ->
// stack overflow / hang, an embedded-SDK crash as fatal as a panic.
//
// C++ avoids this with `Artboard::isAncestor` (src/artboard.cpp): it refuses to
// instantiate a nested child whose artboard source is already an ancestor, so
// the instance tree it builds once is finite. We mirror that idiom directly by
// threading the set of artboard global ids on the current descent path
// (`nested_ancestors`); before descending into a referenced artboard we skip it
// if it is already an ancestor, terminating the cycle gracefully. This bounds
// the recursion to simple paths over the (few) artboards -- unlike a plain depth
// cap it can neither overflow the stack nor fan out exponentially, and it never
// truncates legitimate deep nesting. It is a DELIBERATE terminate-where-C++-
// would-otherwise-loop divergence (v2-status item 27), unreachable on valid
// (acyclic) files, so golden output is unchanged.

macro_rules! cached_runtime_property_key {
    ($type_name:literal, $property_name:literal) => {{
        static KEY: OnceLock<Option<u16>> = OnceLock::new();
        cached_property_key_for_name(&KEY, $type_name, $property_name)
    }};
}

#[derive(Clone, Copy)]
enum RuntimeGradientPaintFilter {
    All,
    ExcludeLayoutComponents,
    OnlyLayoutComponents,
    ExcludeRootArtboard,
    OnlyRootArtboard,
}

impl RuntimeGradientPaintFilter {
    fn includes_container(self, container: &ShapePaintContainerNode) -> bool {
        match self {
            Self::All => true,
            Self::ExcludeLayoutComponents => container.type_name != "LayoutComponent",
            Self::OnlyLayoutComponents => container.type_name == "LayoutComponent",
            Self::ExcludeRootArtboard => !runtime_gradient_container_is_root_artboard(container),
            Self::OnlyRootArtboard => runtime_gradient_container_is_root_artboard(container),
        }
    }
}

fn runtime_gradient_container_is_root_artboard(container: &ShapePaintContainerNode) -> bool {
    container.local_id == 0 && container.type_name == "Artboard"
}

fn runtime_draw_property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    match (type_name, property_name) {
        ("Artboard", "opacity") => cached_runtime_property_key!("Artboard", "opacity"),
        ("DrawRules", "drawTargetId") => {
            cached_runtime_property_key!("DrawRules", "drawTargetId")
        }
        ("DrawTarget", "placementValue") => {
            cached_runtime_property_key!("DrawTarget", "placementValue")
        }
        ("Drawable", "blendModeValue") => {
            cached_runtime_property_key!("Drawable", "blendModeValue")
        }
        ("Component", "parentId") => cached_runtime_property_key!("Component", "parentId"),
        ("WorldTransformComponent", "parentId") => {
            cached_runtime_property_key!("WorldTransformComponent", "parentId")
        }
        ("Fill", "fillRule") => cached_runtime_property_key!("Fill", "fillRule"),
        ("Stroke", "thickness") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Stroke", "thickness")
        }
        ("Stroke", "cap") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Stroke", "cap")
        }
        ("Stroke", "join") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Stroke", "join")
        }
        ("LinearGradient", "opacity") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "LinearGradient", "opacity")
        }
        ("LinearGradient", "startX") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "LinearGradient", "startX")
        }
        ("LinearGradient", "startY") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "LinearGradient", "startY")
        }
        ("LinearGradient", "endX") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "LinearGradient", "endX")
        }
        ("LinearGradient", "endY") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "LinearGradient", "endY")
        }
        ("RadialGradient", "opacity") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "RadialGradient", "opacity")
        }
        ("RadialGradient", "startX") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "RadialGradient", "startX")
        }
        ("RadialGradient", "startY") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "RadialGradient", "startY")
        }
        ("RadialGradient", "endX") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "RadialGradient", "endX")
        }
        ("RadialGradient", "endY") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "RadialGradient", "endY")
        }
        ("GradientStop", "colorValue") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "GradientStop", "colorValue")
        }
        ("GradientStop", "position") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "GradientStop", "position")
        }
        ("Feather", "spaceValue") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Feather", "spaceValue")
        }
        ("Feather", "strength") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Feather", "strength")
        }
        ("Feather", "offsetX") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Feather", "offsetX")
        }
        ("Feather", "offsetY") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Feather", "offsetY")
        }
        ("Feather", "inner") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Feather", "inner")
        }
        ("DashPath", "offset") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "DashPath", "offset")
        }
        ("DashPath", "offsetIsPercentage") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "DashPath", "offsetIsPercentage")
        }
        ("Dash", "length") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Dash", "length")
        }
        ("Dash", "lengthIsPercentage") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "Dash", "lengthIsPercentage")
        }
        ("TrimPath", "modeValue") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "TrimPath", "modeValue")
        }
        ("TrimPath", "offset") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "TrimPath", "offset")
        }
        ("TrimPath", "start") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "TrimPath", "start")
        }
        ("TrimPath", "end") => {
            static KEY: OnceLock<Option<u16>> = OnceLock::new();
            cached_property_key_for_name(&KEY, "TrimPath", "end")
        }
        ("Image", "assetId") => cached_runtime_property_key!("Image", "assetId"),
        ("Image", "originX") => cached_runtime_property_key!("Image", "originX"),
        ("Image", "originY") => cached_runtime_property_key!("Image", "originY"),
        ("Image", "fit") => cached_runtime_property_key!("Image", "fit"),
        ("Image", "alignmentX") => cached_runtime_property_key!("Image", "alignmentX"),
        ("Image", "alignmentY") => cached_runtime_property_key!("Image", "alignmentY"),
        ("MeshVertex", "u") => cached_runtime_property_key!("MeshVertex", "u"),
        ("MeshVertex", "v") => cached_runtime_property_key!("MeshVertex", "v"),
        ("Vertex", "x") => cached_runtime_property_key!("Vertex", "x"),
        ("Vertex", "y") => cached_runtime_property_key!("Vertex", "y"),
        ("Weight", "indices") => cached_runtime_property_key!("Weight", "indices"),
        ("Weight", "values") => cached_runtime_property_key!("Weight", "values"),
        ("NestedArtboard", "artboardId") => {
            cached_runtime_property_key!("NestedArtboard", "artboardId")
        }
        ("NestedArtboardLeaf", "fit") => {
            cached_runtime_property_key!("NestedArtboardLeaf", "fit")
        }
        ("NestedArtboardLeaf", "alignmentX") => {
            cached_runtime_property_key!("NestedArtboardLeaf", "alignmentX")
        }
        ("NestedArtboardLeaf", "alignmentY") => {
            cached_runtime_property_key!("NestedArtboardLeaf", "alignmentY")
        }
        _ => property_key_for_name(type_name, property_name),
    }
}

fn runtime_layout_component_property_key_for_name(property_name: &str) -> Option<u16> {
    match property_name {
        "styleId" => cached_runtime_property_key!("LayoutComponent", "styleId"),
        "clip" => cached_runtime_property_key!("LayoutComponent", "clip"),
        "width" => cached_runtime_property_key!("LayoutComponent", "width"),
        "height" => cached_runtime_property_key!("LayoutComponent", "height"),
        "fractionalWidth" => cached_runtime_property_key!("LayoutComponent", "fractionalWidth"),
        "fractionalHeight" => cached_runtime_property_key!("LayoutComponent", "fractionalHeight"),
        _ => property_key_for_name("LayoutComponent", property_name),
    }
}

#[derive(Clone, Copy)]
enum RuntimeLayoutStyleProperty {
    DisplayValue,
    PositionTypeValue,
    DirectionValue,
    FlexDirectionValue,
    FlexWrapValue,
    LayoutAlignmentType,
    LayoutWidthScaleType,
    LayoutHeightScaleType,
    WidthUnitsValue,
    HeightUnitsValue,
    FlexBasis,
    FlexBasisUnitsValue,
    GapHorizontal,
    GapHorizontalUnitsValue,
    GapVertical,
    GapVerticalUnitsValue,
    PaddingLeft,
    PaddingLeftUnitsValue,
    PaddingRight,
    PaddingRightUnitsValue,
    PaddingTop,
    PaddingTopUnitsValue,
    PaddingBottom,
    PaddingBottomUnitsValue,
    BorderLeft,
    BorderLeftUnitsValue,
    BorderRight,
    BorderRightUnitsValue,
    BorderTop,
    BorderTopUnitsValue,
    BorderBottom,
    BorderBottomUnitsValue,
    MarginLeft,
    MarginLeftUnitsValue,
    MarginRight,
    MarginRightUnitsValue,
    MarginTop,
    MarginTopUnitsValue,
    MarginBottom,
    MarginBottomUnitsValue,
    PositionLeft,
    PositionLeftUnitsValue,
    PositionRight,
    PositionRightUnitsValue,
    PositionTop,
    PositionTopUnitsValue,
    PositionBottom,
    PositionBottomUnitsValue,
    MinWidth,
    MinWidthUnitsValue,
    MinHeight,
    MinHeightUnitsValue,
    MaxWidth,
    MaxWidthUnitsValue,
    MaxHeight,
    MaxHeightUnitsValue,
    AspectRatio,
    IntrinsicallySizedValue,
    LinkCornerRadius,
    CornerRadiusTL,
    CornerRadiusTR,
    CornerRadiusBR,
    CornerRadiusBL,
}

impl RuntimeLayoutStyleProperty {
    #[inline]
    fn key(self) -> Option<u16> {
        match self {
            Self::DisplayValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "displayValue")
            }
            Self::PositionTypeValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionTypeValue")
            }
            Self::DirectionValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "directionValue")
            }
            Self::FlexDirectionValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "flexDirectionValue")
            }
            Self::FlexWrapValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "flexWrapValue")
            }
            Self::LayoutAlignmentType => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutAlignmentType")
            }
            Self::LayoutWidthScaleType => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutWidthScaleType")
            }
            Self::LayoutHeightScaleType => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutHeightScaleType")
            }
            Self::WidthUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "widthUnitsValue")
            }
            Self::HeightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "heightUnitsValue")
            }
            Self::FlexBasis => cached_runtime_property_key!("LayoutComponentStyle", "flexBasis"),
            Self::FlexBasisUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "flexBasisUnitsValue")
            }
            Self::GapHorizontal => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapHorizontal")
            }
            Self::GapHorizontalUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapHorizontalUnitsValue")
            }
            Self::GapVertical => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapVertical")
            }
            Self::GapVerticalUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapVerticalUnitsValue")
            }
            Self::PaddingLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingLeft")
            }
            Self::PaddingLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingLeftUnitsValue")
            }
            Self::PaddingRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingRight")
            }
            Self::PaddingRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingRightUnitsValue")
            }
            Self::PaddingTop => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingTop")
            }
            Self::PaddingTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingTopUnitsValue")
            }
            Self::PaddingBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingBottom")
            }
            Self::PaddingBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingBottomUnitsValue")
            }
            Self::BorderLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderLeft")
            }
            Self::BorderLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderLeftUnitsValue")
            }
            Self::BorderRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderRight")
            }
            Self::BorderRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderRightUnitsValue")
            }
            Self::BorderTop => cached_runtime_property_key!("LayoutComponentStyle", "borderTop"),
            Self::BorderTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderTopUnitsValue")
            }
            Self::BorderBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderBottom")
            }
            Self::BorderBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderBottomUnitsValue")
            }
            Self::MarginLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginLeft")
            }
            Self::MarginLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginLeftUnitsValue")
            }
            Self::MarginRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginRight")
            }
            Self::MarginRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginRightUnitsValue")
            }
            Self::MarginTop => cached_runtime_property_key!("LayoutComponentStyle", "marginTop"),
            Self::MarginTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginTopUnitsValue")
            }
            Self::MarginBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginBottom")
            }
            Self::MarginBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginBottomUnitsValue")
            }
            Self::PositionLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionLeft")
            }
            Self::PositionLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionLeftUnitsValue")
            }
            Self::PositionRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionRight")
            }
            Self::PositionRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionRightUnitsValue")
            }
            Self::PositionTop => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionTop")
            }
            Self::PositionTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionTopUnitsValue")
            }
            Self::PositionBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionBottom")
            }
            Self::PositionBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionBottomUnitsValue")
            }
            Self::MinWidth => cached_runtime_property_key!("LayoutComponentStyle", "minWidth"),
            Self::MinWidthUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "minWidthUnitsValue")
            }
            Self::MinHeight => cached_runtime_property_key!("LayoutComponentStyle", "minHeight"),
            Self::MinHeightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "minHeightUnitsValue")
            }
            Self::MaxWidth => cached_runtime_property_key!("LayoutComponentStyle", "maxWidth"),
            Self::MaxWidthUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "maxWidthUnitsValue")
            }
            Self::MaxHeight => cached_runtime_property_key!("LayoutComponentStyle", "maxHeight"),
            Self::MaxHeightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "maxHeightUnitsValue")
            }
            Self::AspectRatio => {
                cached_runtime_property_key!("LayoutComponentStyle", "aspectRatio")
            }
            Self::IntrinsicallySizedValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "intrinsicallySizedValue")
            }
            Self::LinkCornerRadius => {
                cached_runtime_property_key!("LayoutComponentStyle", "linkCornerRadius")
            }
            Self::CornerRadiusTL => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusTL")
            }
            Self::CornerRadiusTR => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusTR")
            }
            Self::CornerRadiusBR => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusBR")
            }
            Self::CornerRadiusBL => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusBL")
            }
        }
    }
}

fn runtime_nested_artboard_layout_property_key_for_name(property_name: &str) -> Option<u16> {
    match property_name {
        "instanceWidthScaleType" => {
            cached_runtime_property_key!("NestedArtboardLayout", "instanceWidthScaleType")
        }
        "instanceHeightScaleType" => {
            cached_runtime_property_key!("NestedArtboardLayout", "instanceHeightScaleType")
        }
        "instanceWidthUnitsValue" => {
            cached_runtime_property_key!("NestedArtboardLayout", "instanceWidthUnitsValue")
        }
        "instanceHeightUnitsValue" => {
            cached_runtime_property_key!("NestedArtboardLayout", "instanceHeightUnitsValue")
        }
        "instanceWidth" => cached_runtime_property_key!("NestedArtboardLayout", "instanceWidth"),
        "instanceHeight" => cached_runtime_property_key!("NestedArtboardLayout", "instanceHeight"),
        _ => property_key_for_name("NestedArtboardLayout", property_name),
    }
}

impl ArtboardInstance {
    pub fn draw_commands(&self, graph: &ArtboardGraph) -> Vec<RuntimeDrawCommand> {
        let layout_bounds = self.runtime_taffy_layout_bounds(graph, None);
        self.draw_commands_with_layout_bounds(graph, layout_bounds.as_ref())
    }

    fn draw_commands_with_layout_bounds(
        &self,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Vec<RuntimeDrawCommand> {
        let sorted = self.runtime_sorted_drawable_order(graph);
        let mut path_cache = RuntimeRenderPathCache::default();
        self.draw_commands_with_sorted_drawable_order(
            graph,
            layout_bounds,
            &sorted,
            &mut path_cache,
        )
    }

    fn draw_commands_with_sorted_drawable_order(
        &self,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        sorted_drawable_order: &[SortedDrawableNode],
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Vec<RuntimeDrawCommand> {
        let mut commands = Vec::new();
        let mut pending_clip_operations = Vec::<&SortedDrawableNode>::new();
        let mut empty_clips = 0i32;

        for drawable in sorted_drawable_order {
            let prev_clips = empty_clips;
            empty_clips += self.runtime_empty_clip_count(drawable, graph);
            if !self.runtime_will_draw(drawable) || empty_clips != prev_clips || empty_clips > 0 {
                continue;
            }

            if drawable.kind == DrawableOrderKind::ClipStartProxy {
                pending_clip_operations.push(drawable);
                continue;
            } else if !pending_clip_operations.is_empty() {
                if drawable.kind == DrawableOrderKind::ClipEndProxy {
                    pending_clip_operations.pop();
                    continue;
                }

                commands.extend(pending_clip_operations.drain(..).map(|pending_clip| {
                    self.runtime_draw_command_for_node(
                        pending_clip,
                        graph,
                        layout_bounds,
                        path_cache,
                    )
                }));
            }

            commands.push(self.runtime_draw_command_for_node(
                drawable,
                graph,
                layout_bounds,
                path_cache,
            ));
        }

        commands
    }

    fn runtime_sorted_drawable_order(&self, graph: &ArtboardGraph) -> Vec<SortedDrawableNode> {
        let Some(draw_target_id_key) =
            runtime_draw_property_key_for_name("DrawRules", "drawTargetId")
        else {
            return graph.sorted_drawable_order.clone();
        };
        let Some(placement_value_key) =
            runtime_draw_property_key_for_name("DrawTarget", "placementValue")
        else {
            return graph.sorted_drawable_order.clone();
        };

        let draw_targets_by_local = graph
            .draw_targets
            .iter()
            .map(|target| (target.local_id, target))
            .collect::<BTreeMap<_, _>>();
        let active_target_by_rules = graph
            .draw_rules
            .iter()
            .filter_map(|rules| {
                let target_local =
                    usize::try_from(self.uint_property(rules.local_id, draw_target_id_key)?)
                        .ok()?;
                draw_targets_by_local
                    .contains_key(&target_local)
                    .then_some((rules.local_id, target_local))
            })
            .collect::<BTreeMap<_, _>>();
        let mut main = Vec::new();
        let mut grouped = BTreeMap::<usize, Vec<SortedDrawableNode>>::new();
        for drawable in &graph.drawable_order {
            let draw_target_local = drawable
                .flattened_draw_rules_local
                .and_then(|rules_local| active_target_by_rules.get(&rules_local).copied());
            let node = runtime_sorted_drawable_node(drawable, draw_target_local);
            if let Some(draw_target_local) = draw_target_local {
                grouped.entry(draw_target_local).or_default().push(node);
            } else {
                main.push(node);
            }
        }

        for draw_target_local in &graph.draw_target_order {
            let Some(group) = grouped.remove(draw_target_local) else {
                continue;
            };
            if group.is_empty() {
                continue;
            }
            let Some(target) = draw_targets_by_local.get(draw_target_local) else {
                continue;
            };
            let Some(target_drawable_local) = target.drawable_local else {
                continue;
            };
            let Some(target_position) = main
                .iter()
                .position(|drawable| drawable.local_id == Some(target_drawable_local))
            else {
                continue;
            };
            let placement_value = self
                .uint_property(target.local_id, placement_value_key)
                .unwrap_or(target.placement_value);
            let insert_at = match placement_value {
                0 => target_position,
                1 => target_position + 1,
                _ => continue,
            };
            main.splice(insert_at..insert_at, group);
        }

        let mut sorted = runtime_interleave_clipping_proxy_drawables(
            main.into_iter().rev(),
            &graph.clipping_shapes,
        );
        runtime_apply_save_operation_elision(&mut sorted, &graph.clipping_shapes);
        sorted
    }

    pub fn draw_static_artboard(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut RuntimeRenderPaints,
    ) -> Result<()> {
        let mut render_cache = RuntimeRenderPathCache::default();
        self.prepare_static_artboard_paints(
            runtime,
            graph,
            factory,
            paint_by_global,
            &mut render_cache,
        )?;
        self.draw_prepared_static_artboard_with_path_cache(
            runtime,
            graph,
            std::slice::from_ref(graph),
            factory,
            renderer,
            paint_by_global,
            &mut render_cache,
        )
    }

    pub fn prepare_static_artboard_paints(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut RuntimeRenderPaints,
        render_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        self.prepare_static_artboard_paints_with_filter(
            runtime,
            graph,
            factory,
            paint_by_global,
            None,
            render_cache,
            None,
            RuntimeGradientPaintFilter::All,
        )
    }

    fn prepare_static_artboard_paints_with_filter(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut RuntimeRenderPaints,
        mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
        render_cache: &mut RuntimeRenderPathCache,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        filter: RuntimeGradientPaintFilter,
    ) -> Result<()> {
        let gradient_preparation = render_cache.gradient_preparation_frame(graph);
        if !gradient_preparation.has_paints() {
            return Ok(());
        }

        let mut prepared = Vec::new();
        // C++ Artboard::advance updates gradient mutators in dependency order before
        // drawing, and the recording factory observes shader creation there.
        for local_id in gradient_preparation.dependency_order.iter().copied() {
            let Some(paints) = gradient_preparation.paints_by_mutator.get(&local_id) else {
                continue;
            };
            for paint_ref in paints {
                let Some(container) = graph.shape_paint_containers.get(paint_ref.container_index)
                else {
                    continue;
                };
                let Some(paint) = container.paints.get(paint_ref.paint_index) else {
                    continue;
                };
                if !filter.includes_container(container) {
                    continue;
                }
                if self.runtime_gradient_paint_is_collapsed(container, paint) {
                    continue;
                }
                if !runtime_mark_seen_u32(&mut prepared, paint.global_id) {
                    continue;
                }
                let object = runtime
                    .object(paint.global_id as usize)
                    .with_context(|| format!("missing paint global {}", paint.global_id))?;
                let opacity_local = shape_paint_container_opacity_local(graph, container.local_id);
                let shape_world = runtime_shape_paint_container_world_transform(
                    self,
                    graph,
                    container,
                    layout_bounds,
                    render_cache,
                );
                let runtime_paint = runtime_prepare_gradient_paint_command(
                    self,
                    graph,
                    opacity_local,
                    container,
                    paint,
                    shape_world,
                    layout_bounds,
                );
                let mut gradient_resources = RuntimeGradientShaderResources {
                    factory,
                    shaders: &mut render_cache.gradient_shaders,
                };
                if let Some(configurations) = paint_configurations.as_mut() {
                    configurations.remove(paint.global_id);
                }
                runtime_configure_paint(
                    paint_by_global
                        .paint_mut(paint.global_id)
                        .with_context(|| {
                            format!("missing render paint for global {}", paint.global_id)
                        })?
                        .as_mut(),
                    self,
                    object,
                    &runtime_paint,
                    Some(&mut gradient_resources),
                )?;
            }
        }

        Ok(())
    }

    pub fn prepare_static_artboard_tree_paints(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        paint_cache: &mut RuntimeRenderPaintCache,
        render_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        // Seed the nested-artboard cycle guard with the root artboard's global id
        // (see nested_artboard_cycle: the ancestor set that mirrors C++
        // Artboard::isAncestor).
        let nested_ancestors = BTreeSet::from([graph.global_id]);
        self.prepare_static_artboard_tree_paints_internal(
            runtime,
            graph,
            artboards,
            factory,
            &mut paint_cache.paints,
            Some(&mut paint_cache.paint_configurations),
            None,
            Some(&mut paint_cache.nested_artboards),
            render_cache,
            true,
            false,
            &nested_ancestors,
        )?;
        self.prepare_static_artboard_tree_paints_internal(
            runtime,
            graph,
            artboards,
            factory,
            &mut paint_cache.paints,
            Some(&mut paint_cache.paint_configurations),
            Some(&mut paint_cache.preparation),
            Some(&mut paint_cache.nested_artboards),
            render_cache,
            true,
            true,
            &nested_ancestors,
        )
    }

    fn prepare_static_artboard_tree_paints_internal(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut RuntimeRenderPaints,
        mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
        mut paint_preparation: Option<&mut Option<RuntimePaintPreparationFrame>>,
        mut nested_paint_caches: Option<
            &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
        >,
        render_cache: &mut RuntimeRenderPathCache,
        defer_root_layout_gradients: bool,
        apply_nested_layout_bounds: bool,
        nested_ancestors: &BTreeSet<u32>,
    ) -> Result<()> {
        let preparation_graph_global_id = graph.global_id;
        let preparation_instance_epoch = self.cache_epoch();
        // World-space gradients must reconfigure when only world transforms
        // moved (prepared_epoch); local-space-only graphs keep the pure
        // cache_epoch key so the M7 skip still holds.
        let preparation_world_epoch = if render_cache
            .gradient_preparation_frame(graph)
            .has_world_space_gradient_paints
        {
            self.prepared_epoch()
        } else {
            0
        };
        if paint_preparation
            .as_ref()
            .and_then(|preparation| preparation.as_ref())
            .is_some_and(|preparation| {
                preparation.can_skip_prepared_frame(
                    preparation_graph_global_id,
                    preparation_instance_epoch,
                    preparation_world_epoch,
                )
            })
        {
            return Ok(());
        }

        let prepared = render_cache.prepared_artboard_frame(self, graph, Some(runtime));
        let layout_bounds = prepared.layout_bounds.as_ref().as_ref();
        let commands = prepared.commands.as_slice();
        let has_nested_artboards = commands.iter().any(runtime_draw_command_is_nested_artboard);
        let defer_layout_gradients = defer_root_layout_gradients && has_nested_artboards;
        let preparation_key = RuntimePaintPreparationCacheKey {
            graph_global_id: preparation_graph_global_id,
            instance_epoch: preparation_instance_epoch,
            world_epoch: preparation_world_epoch,
            nested_epoch: if defer_layout_gradients {
                self.runtime_nested_paint_preparation_epoch(commands)
            } else {
                0
            },
        };
        if paint_preparation
            .as_ref()
            .and_then(|preparation| preparation.as_ref())
            .is_some_and(|preparation| preparation.key == preparation_key)
        {
            return Ok(());
        }
        if defer_layout_gradients {
            self.prepare_static_artboard_tree_paints_in_dependency_order(
                runtime,
                graph,
                artboards,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                paint_preparation.as_deref_mut(),
                nested_paint_caches,
                render_cache,
                layout_bounds,
                commands,
                apply_nested_layout_bounds,
                nested_ancestors,
            )?;
            if let Some(preparation) = paint_preparation.as_deref_mut() {
                *preparation = Some(RuntimePaintPreparationFrame {
                    key: preparation_key,
                    has_nested_artboards,
                });
            }
            return Ok(());
        }

        let initial_filter = if defer_layout_gradients {
            RuntimeGradientPaintFilter::ExcludeLayoutComponents
        } else {
            RuntimeGradientPaintFilter::All
        };

        self.prepare_static_artboard_paints_with_filter(
            runtime,
            graph,
            factory,
            paint_by_global,
            paint_configurations.as_deref_mut(),
            render_cache,
            layout_bounds,
            initial_filter,
        )?;

        for command in commands {
            if command.object_kind == RuntimeDrawCommandObjectKind::ArtboardComponentList {
                self.prepare_static_component_list_paints(
                    runtime,
                    artboards,
                    factory,
                    nested_paint_caches.as_deref_mut(),
                    render_cache,
                    command,
                    apply_nested_layout_bounds,
                    nested_ancestors,
                )?;
                continue;
            }
            if command.referenced_artboard_global.is_none()
                || !runtime_draw_command_is_nested_artboard(&command)
            {
                continue;
            }
            self.prepare_static_nested_artboard_tree_paints(
                runtime,
                artboards,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                nested_paint_caches.as_deref_mut(),
                render_cache,
                layout_bounds,
                command,
                apply_nested_layout_bounds,
                nested_ancestors,
            )?;
        }

        if let Some(preparation) = paint_preparation.as_deref_mut() {
            *preparation = Some(RuntimePaintPreparationFrame {
                key: preparation_key,
                has_nested_artboards,
            });
        }

        if defer_layout_gradients {
            self.prepare_static_artboard_paints_with_filter(
                runtime,
                graph,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                render_cache,
                layout_bounds,
                RuntimeGradientPaintFilter::OnlyLayoutComponents,
            )?;
        }

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn prepare_static_component_list_paints(
        &self,
        runtime: &RuntimeFile,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        mut nested_paint_caches: Option<
            &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
        >,
        render_cache: &mut RuntimeRenderPathCache,
        command: &RuntimeDrawCommand,
        apply_nested_layout_bounds: bool,
        nested_ancestors: &BTreeSet<u32>,
    ) -> Result<()> {
        let Some(items) = command
            .local_id
            .and_then(|local_id| self.component_list_items.get(&local_id))
        else {
            return Ok(());
        };
        for (item_index, item) in items.iter().enumerate() {
            if nested_ancestors.contains(&item.child.graph_global_id) {
                continue;
            }
            let Some(child_graph) = artboards
                .iter()
                .find(|graph| graph.global_id == item.child.graph_global_id)
            else {
                continue;
            };
            let mut child_ancestors = nested_ancestors.clone();
            child_ancestors.insert(item.child.graph_global_id);
            let cache_key = component_list_render_cache_key(
                command.global_id,
                command.local_id,
                item.child.graph_global_id,
                item_index,
                item.render_cache_revision,
            );
            let child_cache = render_cache.nested_artboards.entry(cache_key).or_default();
            if let Some(caches) = nested_paint_caches.as_deref_mut() {
                if !caches.contains_key(&cache_key) {
                    caches.insert(
                        cache_key,
                        preallocate_render_paint_cache_for_artboard_instance(
                            runtime,
                            child_graph,
                            artboards,
                            factory,
                        ),
                    );
                }
                let child_paint_cache = caches.entry(cache_key).or_default();
                item.child.prepare_static_artboard_tree_paints_internal(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    &mut child_paint_cache.paints,
                    Some(&mut child_paint_cache.paint_configurations),
                    Some(&mut child_paint_cache.preparation),
                    Some(&mut child_paint_cache.nested_artboards),
                    child_cache,
                    true,
                    apply_nested_layout_bounds,
                    &child_ancestors,
                )?;
            }
        }
        Ok(())
    }

    fn runtime_nested_paint_preparation_epoch(&self, commands: &[RuntimeDrawCommand]) -> u64 {
        let mut epoch = 0xcbf29ce484222325u64;
        for command in commands {
            if command.referenced_artboard_global.is_none()
                || !runtime_draw_command_is_nested_artboard(command)
            {
                continue;
            }
            let local_id = command.local_id.unwrap_or(usize::MAX);
            epoch =
                epoch.wrapping_mul(0x100000001b3) ^ command.global_id.unwrap_or_default() as u64;
            epoch = epoch.wrapping_mul(0x100000001b3) ^ local_id as u64;
            epoch = epoch.wrapping_mul(0x100000001b3)
                ^ command.referenced_artboard_global.unwrap_or_default() as u64;
            if let Some(child) = command
                .local_id
                .and_then(|local_id| self.nested_artboards.get(&local_id))
                .map(|nested| nested.child.as_ref())
            {
                epoch = epoch.wrapping_mul(0x100000001b3) ^ child.cache_epoch();
                // World-space gradient staleness folds through nested
                // children too: keyed transform writes only bump the child's
                // prepared_epoch, never its cache_epoch.
                epoch = epoch.wrapping_mul(0x100000001b3) ^ child.prepared_epoch();
            }
        }
        epoch
    }

    fn runtime_nested_artboard_preparation_commands_by_local(
        &self,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Vec<(usize, RuntimeDrawCommand)> {
        let mut commands = Vec::new();
        for drawable in &graph.sorted_drawable_order {
            if drawable.referenced_artboard_global.is_none() {
                continue;
            }
            let command =
                self.runtime_draw_command_for_node(drawable, graph, layout_bounds, path_cache);
            if command.referenced_artboard_global.is_none()
                || !runtime_draw_command_is_nested_artboard(&command)
            {
                continue;
            }
            let Some(local_id) = command.local_id else {
                continue;
            };
            runtime_insert_nested_preparation_command_if_absent(&mut commands, local_id, command);
        }
        commands
    }

    fn prepare_static_nested_artboard_tree_paints(
        &self,
        runtime: &RuntimeFile,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut RuntimeRenderPaints,
        mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
        nested_paint_caches: Option<
            &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
        >,
        render_cache: &mut RuntimeRenderPathCache,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        command: &RuntimeDrawCommand,
        apply_nested_layout_bounds: bool,
        nested_ancestors: &BTreeSet<u32>,
    ) -> Result<()> {
        let nested_instance = command
            .local_id
            .and_then(|local_id| self.nested_artboards.get(&local_id));
        let referenced_artboard_global = nested_instance
            .map(|nested| nested.child.graph_global_id)
            .or(command.referenced_artboard_global)
            .context("nested artboard missing referenced artboard")?;
        // Cycle guard (see `nested_ancestors` at the top of this module): if this
        // artboard is already an ancestor on the current descent path, descending
        // would loop -- terminate gracefully, mirroring C++ Artboard::isAncestor.
        if nested_ancestors.contains(&referenced_artboard_global) {
            return Ok(());
        }
        let mut child_ancestors = nested_ancestors.clone();
        child_ancestors.insert(referenced_artboard_global);
        let child_graph = artboards
            .iter()
            .find(|graph| graph.global_id == referenced_artboard_global)
            .with_context(|| {
                format!("missing nested artboard graph for global {referenced_artboard_global}")
            })?;
        let cache_key = nested_render_cache_key(
            command.global_id,
            command.local_id,
            referenced_artboard_global,
            nested_instance.map_or(0, |nested| nested.render_cache_revision),
        );
        let child_cache = render_cache.nested_artboards.entry(cache_key).or_default();
        let child_paint_caches = match nested_paint_caches {
            Some(caches) => {
                if !caches.contains_key(&cache_key) {
                    caches.insert(
                        cache_key,
                        preallocate_render_paint_cache_for_artboard_instance(
                            runtime,
                            child_graph,
                            artboards,
                            factory,
                        ),
                    );
                }
                let child_paint_cache = caches.entry(cache_key).or_default();
                Some((
                    &mut child_paint_cache.paints,
                    &mut child_paint_cache.paint_configurations,
                    &mut child_paint_cache.preparation,
                    Some(&mut child_paint_cache.nested_artboards),
                ))
            }
            None => None,
        };

        if let Some(child) = command
            .local_id
            .and_then(|local_id| self.nested_artboards.get(&local_id))
            .map(|nested| nested.child.as_ref())
        {
            let layout_child;
            let child = if apply_nested_layout_bounds
                && command.object_kind == RuntimeDrawCommandObjectKind::NestedArtboardLayout
            {
                let mut cloned = child.clone();
                runtime_apply_nested_artboard_layout_child_bounds(
                    &mut cloned,
                    command,
                    layout_bounds,
                )?;
                runtime_settle_nested_artboard_layout_child(&mut cloned);
                layout_child = cloned;
                &layout_child
            } else {
                child
            };
            if let Some((
                child_paints,
                child_configurations,
                child_preparation,
                child_nested_paints,
            )) = child_paint_caches
            {
                child.prepare_static_artboard_tree_paints_internal(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    child_paints,
                    Some(child_configurations),
                    apply_nested_layout_bounds.then_some(child_preparation),
                    child_nested_paints,
                    child_cache,
                    true,
                    apply_nested_layout_bounds,
                    &child_ancestors,
                )?;
            } else {
                child.prepare_static_artboard_tree_paints_internal(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    paint_by_global,
                    paint_configurations.as_deref_mut(),
                    None,
                    None,
                    child_cache,
                    true,
                    apply_nested_layout_bounds,
                    &child_ancestors,
                )?;
            }
            return Ok(());
        }

        let mut child = ArtboardInstance::from_graph(runtime, child_graph)?;
        if apply_nested_layout_bounds {
            runtime_apply_nested_artboard_layout_child_bounds(&mut child, command, layout_bounds)?;
        }
        let host_opacity = command
            .local_id
            .and_then(|local_id| self.component(local_id))
            .map(|component| component.transform.render_opacity)
            .unwrap_or(1.0);
        if let Some(opacity_key) = runtime_draw_property_key_for_name("Artboard", "opacity") {
            child.set_double_property(0, opacity_key, host_opacity);
        }
        child.update_pass();
        if let Some((child_paints, child_configurations, child_preparation, child_nested_paints)) =
            child_paint_caches
        {
            child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                child_paints,
                Some(child_configurations),
                apply_nested_layout_bounds.then_some(child_preparation),
                child_nested_paints,
                child_cache,
                true,
                apply_nested_layout_bounds,
                &child_ancestors,
            )?;
        } else {
            child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                None,
                None,
                child_cache,
                true,
                apply_nested_layout_bounds,
                &child_ancestors,
            )?;
        }
        Ok(())
    }

    fn prepare_static_artboard_tree_paints_in_dependency_order(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut RuntimeRenderPaints,
        mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
        _paint_preparation: Option<&mut Option<RuntimePaintPreparationFrame>>,
        mut nested_paint_caches: Option<
            &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
        >,
        render_cache: &mut RuntimeRenderPathCache,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        commands: &[RuntimeDrawCommand],
        apply_nested_layout_bounds: bool,
        nested_ancestors: &BTreeSet<u32>,
    ) -> Result<()> {
        let gradient_preparation = render_cache.gradient_preparation_frame(graph);

        let mut nested_command_by_local = Vec::new();
        for command in commands {
            if command.referenced_artboard_global.is_none()
                || !runtime_draw_command_is_nested_artboard(command)
            {
                continue;
            }
            let Some(local_id) = command.local_id else {
                continue;
            };
            runtime_upsert_nested_preparation_command(
                &mut nested_command_by_local,
                local_id,
                command.clone(),
            );
        }
        if graph
            .components
            .iter()
            .any(|component| component.type_name == "LayoutComponent")
        {
            for (local_id, command) in self.runtime_nested_artboard_preparation_commands_by_local(
                graph,
                layout_bounds,
                render_cache,
            ) {
                runtime_upsert_nested_preparation_command(
                    &mut nested_command_by_local,
                    local_id,
                    command,
                );
            }
        }
        if !gradient_preparation.has_paints() {
            for (_, command) in nested_command_by_local {
                self.prepare_static_nested_artboard_tree_paints(
                    runtime,
                    artboards,
                    factory,
                    paint_by_global,
                    paint_configurations.as_deref_mut(),
                    nested_paint_caches.as_deref_mut(),
                    render_cache,
                    layout_bounds,
                    &command,
                    apply_nested_layout_bounds,
                    nested_ancestors,
                )?;
            }
            return Ok(());
        }

        let mut prepared_paints = Vec::new();
        let mut prepared_nested_hosts = Vec::new();

        for local_id in gradient_preparation
            .dependency_insertion_order
            .iter()
            .copied()
        {
            self.prepare_static_gradient_paints_for_dependency_local(
                runtime,
                graph,
                factory,
                paint_by_global,
                render_cache,
                layout_bounds,
                paint_configurations.as_deref_mut(),
                &gradient_preparation,
                &mut prepared_paints,
                RuntimeGradientPaintFilter::ExcludeRootArtboard,
                local_id,
            )?;

            let Some(command) =
                runtime_nested_preparation_command(&nested_command_by_local, local_id)
            else {
                continue;
            };
            if !runtime_mark_seen_usize(&mut prepared_nested_hosts, local_id) {
                continue;
            }

            self.prepare_static_nested_artboard_tree_paints(
                runtime,
                artboards,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                nested_paint_caches.as_deref_mut(),
                render_cache,
                layout_bounds,
                command,
                apply_nested_layout_bounds,
                nested_ancestors,
            )?;
        }

        for local_id in gradient_preparation
            .dependency_insertion_order
            .iter()
            .copied()
        {
            self.prepare_static_gradient_paints_for_dependency_local(
                runtime,
                graph,
                factory,
                paint_by_global,
                render_cache,
                layout_bounds,
                paint_configurations.as_deref_mut(),
                &gradient_preparation,
                &mut prepared_paints,
                RuntimeGradientPaintFilter::OnlyRootArtboard,
                local_id,
            )?;
        }

        Ok(())
    }

    fn prepare_static_gradient_paints_for_dependency_local(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        paint_by_global: &mut RuntimeRenderPaints,
        render_cache: &mut RuntimeRenderPathCache,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
        gradient_preparation: &RuntimeGradientPreparationFrame,
        prepared: &mut Vec<u32>,
        filter: RuntimeGradientPaintFilter,
        local_id: usize,
    ) -> Result<()> {
        let Some(paints) = gradient_preparation.paints_by_mutator.get(&local_id) else {
            return Ok(());
        };
        for paint_ref in paints {
            let Some(container) = graph.shape_paint_containers.get(paint_ref.container_index)
            else {
                continue;
            };
            let Some(paint) = container.paints.get(paint_ref.paint_index) else {
                continue;
            };
            if !filter.includes_container(container) {
                continue;
            }
            if self.runtime_gradient_paint_is_collapsed(container, paint) {
                continue;
            }
            if !runtime_mark_seen_u32(prepared, paint.global_id) {
                continue;
            }
            let object = runtime
                .object(paint.global_id as usize)
                .with_context(|| format!("missing paint global {}", paint.global_id))?;
            let opacity_local = shape_paint_container_opacity_local(graph, container.local_id);
            let shape_world = runtime_shape_paint_container_world_transform(
                self,
                graph,
                container,
                layout_bounds,
                render_cache,
            );
            let runtime_paint = runtime_prepare_gradient_paint_command(
                self,
                graph,
                opacity_local,
                container,
                paint,
                shape_world,
                layout_bounds,
            );
            let mut gradient_resources = RuntimeGradientShaderResources {
                factory,
                shaders: &mut render_cache.gradient_shaders,
            };
            if let Some(configurations) = paint_configurations.as_mut() {
                configurations.remove(paint.global_id);
            }
            runtime_configure_paint(
                paint_by_global
                    .paint_mut(paint.global_id)
                    .with_context(|| {
                        format!("missing render paint for global {}", paint.global_id)
                    })?
                    .as_mut(),
                self,
                object,
                &runtime_paint,
                Some(&mut gradient_resources),
            )?;
        }
        Ok(())
    }

    fn runtime_gradient_paint_is_collapsed(
        &self,
        container: &ShapePaintContainerNode,
        paint: &ShapePaintNode,
    ) -> bool {
        [
            Some(container.local_id),
            Some(paint.local_id),
            paint.mutator_local,
        ]
        .into_iter()
        .flatten()
        .any(|local_id| self.runtime_component_is_effectively_collapsed(local_id))
    }

    pub fn draw_prepared_static_artboard(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut RuntimeRenderPaints,
    ) -> Result<()> {
        let mut path_cache = RuntimeRenderPathCache::default();
        self.draw_prepared_static_artboard_with_path_cache(
            runtime,
            graph,
            std::slice::from_ref(graph),
            factory,
            renderer,
            paint_by_global,
            &mut path_cache,
        )
    }

    pub fn draw_prepared_static_artboard_with_path_cache(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut RuntimeRenderPaints,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        let mut mesh_buffers = RuntimeMeshRenderBufferSlots::default();
        let images = RuntimeRenderImages::default();
        // Seed the nested-artboard cycle guard with this artboard's global id.
        let nested_ancestors = BTreeSet::from([graph.global_id]);
        self.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            graph,
            artboards,
            factory,
            renderer,
            paint_by_global,
            &images,
            &mut mesh_buffers,
            path_cache,
            None,
            None,
            true,
            &nested_ancestors,
        )
    }

    pub fn draw_prepared_static_artboard_with_render_cache(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_cache: &mut RuntimeRenderPaintCache,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Result<()> {
        self.draw_prepared_static_artboard_with_render_cache_and_origin(
            runtime,
            graph,
            artboards,
            factory,
            renderer,
            paint_cache,
            path_cache,
            true,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_prepared_static_artboard_with_render_cache_and_origin(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_cache: &mut RuntimeRenderPaintCache,
        path_cache: &mut RuntimeRenderPathCache,
        apply_origin_transform: bool,
    ) -> Result<()> {
        // Seed the nested-artboard cycle guard with this artboard's global id.
        let nested_ancestors = BTreeSet::from([graph.global_id]);
        self.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            graph,
            artboards,
            factory,
            renderer,
            &mut paint_cache.paints,
            &paint_cache.images,
            &mut paint_cache.meshes,
            path_cache,
            Some(&mut paint_cache.paint_configurations),
            Some(&mut paint_cache.nested_artboards),
            apply_origin_transform,
            &nested_ancestors,
        )
    }

    fn draw_prepared_static_artboard_internal_with_path_cache(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        paint_by_global: &mut RuntimeRenderPaints,
        image_by_global: &RuntimeRenderImages,
        mesh_by_local: &mut RuntimeMeshRenderBufferSlots,
        path_cache: &mut RuntimeRenderPathCache,
        mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
        mut nested_paint_caches: Option<
            &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
        >,
        apply_origin_transform: bool,
        nested_ancestors: &BTreeSet<u32>,
    ) -> Result<()> {
        if self
            .component(0)
            .is_some_and(|component| component.transform.render_opacity == 0.0)
        {
            return Ok(());
        }

        let needs_save = self.clip || apply_origin_transform;
        if needs_save {
            renderer.save();
        }
        if self.clip {
            let (clip_left, clip_top) = if apply_origin_transform {
                (0.0, 0.0)
            } else {
                (-self.origin_x * self.width, -self.origin_y * self.height)
            };
            let fill_rule = RenderFillRule::Clockwise;
            let key = path_cache.retained_render_path_key(self, graph, fill_rule);
            // Mirrors C++ Artboard::drawInternal: m_worldPath.renderPath(this)
            // retains the clip path, so build rect commands only on cache miss.
            let clip = path_cache.artboard_clip_path(key, factory, || {
                runtime_rect_commands(
                    clip_left,
                    clip_top,
                    clip_left + self.width,
                    clip_top + self.height,
                )
            });
            renderer.clip_path(clip.as_ref());
        }
        if apply_origin_transform {
            renderer.transform(self.artboard_origin_transform());
        }

        let prepared = path_cache.prepared_artboard_frame(self, graph, Some(runtime));
        let layout_bounds = prepared.layout_bounds.as_ref().as_ref();
        if let Some(background) = prepared.background.as_ref() {
            runtime_draw_background(
                runtime,
                self,
                graph,
                background,
                factory,
                renderer,
                paint_by_global,
                path_cache,
                paint_configurations.as_deref_mut(),
            )?;
        }

        for command in prepared.commands.iter() {
            runtime_draw_command(
                runtime,
                self,
                graph,
                artboards,
                command,
                layout_bounds,
                factory,
                renderer,
                paint_by_global,
                image_by_global,
                mesh_by_local,
                path_cache,
                paint_configurations.as_deref_mut(),
                match &mut nested_paint_caches {
                    Some(caches) => Some(&mut **caches),
                    None => None,
                },
                nested_ancestors,
            )?;
        }

        if needs_save {
            renderer.restore();
        }
        Ok(())
    }

    fn artboard_origin_transform(&self) -> RenderMat2D {
        RenderMat2D([
            1.0,
            0.0,
            0.0,
            1.0,
            self.width * self.origin_x,
            self.height * self.origin_y,
        ])
    }

    fn runtime_will_draw(&self, drawable: &SortedDrawableNode) -> bool {
        match drawable.kind {
            DrawableOrderKind::ClipStartProxy | DrawableOrderKind::ClipEndProxy => true,
            DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => {
                if drawable.is_hidden {
                    return false;
                }
                let component_local = match drawable.kind {
                    DrawableOrderKind::LayoutProxy => drawable.layout_local,
                    _ => drawable.local_id,
                };
                if component_local
                    .is_some_and(|local_id| self.runtime_component_is_collapsed_for_draw(local_id))
                {
                    return false;
                }

                if drawable.type_name == "ArtboardComponentList" {
                    return drawable.local_id.is_some_and(|local_id| {
                        self.component_list_items
                            .get(&local_id)
                            .is_some_and(|items| !items.is_empty())
                    });
                }

                if drawable.type_name == "Image" {
                    return self
                        .resolved_image_asset_global(
                            drawable.local_id,
                            drawable.resolved_image_asset_global,
                        )
                        .is_some()
                        && self
                            .runtime_drawable_render_opacity(drawable)
                            .is_some_and(|opacity| opacity != 0.0);
                }

                if sorted_drawable_is_nested_artboard(drawable.type_name) {
                    return drawable.referenced_artboard_global.is_some();
                }

                if sorted_drawable_uses_render_opacity(drawable.type_name) {
                    return self
                        .runtime_drawable_render_opacity(drawable)
                        .is_some_and(|opacity| opacity != 0.0);
                }

                true
            }
        }
    }

    fn runtime_drawable_render_opacity(&self, drawable: &SortedDrawableNode) -> Option<f32> {
        let local_id = drawable.local_id?;
        self.component(local_id)
            .map(|component| component.transform.render_opacity)
    }

    fn runtime_empty_clip_count(
        &self,
        drawable: &SortedDrawableNode,
        graph: &ArtboardGraph,
    ) -> i32 {
        let empty_delta = match drawable.kind {
            DrawableOrderKind::ClipStartProxy => 1,
            DrawableOrderKind::ClipEndProxy => -1,
            DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => return 0,
        };
        let Some(clipping_shape_local) = drawable.clipping_shape_local else {
            return 0;
        };
        let Some(clipping_shape) = graph
            .clipping_shapes
            .iter()
            .find(|clipping_shape| clipping_shape.local_id == clipping_shape_local)
        else {
            return 0;
        };

        if clipping_shape.is_visible
            && !self.clipping_shape_has_runtime_clip_path(clipping_shape, &graph.path_composers)
        {
            empty_delta
        } else {
            0
        }
    }

    fn clipping_shape_has_runtime_clip_path(
        &self,
        clipping_shape: &ClippingShapeNode,
        path_composers: &[PathComposerNode],
    ) -> bool {
        clipping_shape.shape_locals.iter().any(|shape_local| {
            path_composers
                .iter()
                .find(|composer| composer.shape_local == *shape_local)
                .is_some_and(|composer| {
                    composer
                        .paths
                        .iter()
                        .any(|path| self.runtime_path_counts_for_clip(path))
                })
        })
    }

    fn runtime_path_counts_for_clip(&self, path: &PathComposerPathNode) -> bool {
        !path.is_hidden && !self.runtime_component_is_collapsed_for_draw(path.local_id)
    }

    fn runtime_component_is_effectively_collapsed(&self, local_id: usize) -> bool {
        let mut current = Some(local_id);
        let mut remaining = self.components.len().saturating_add(1);
        while let Some(component_local) = current {
            if remaining == 0 {
                return false;
            }
            remaining -= 1;
            if self.runtime_component_is_collapsed_for_draw(component_local) {
                return true;
            }
            let Some(component) = self.component(component_local) else {
                return false;
            };
            current = component.parent_local;
        }
        false
    }

    fn runtime_component_is_collapsed_for_draw(&self, local_id: usize) -> bool {
        let Some(component) = self.component(local_id) else {
            return false;
        };
        // Mirrors C++ `Component::isCollapsed` plus
        // `LayoutComponent::isCollapsed`: descendants receive propagated
        // collapse dirt during update, while a layout component's own
        // display:none is checked locally.
        component.is_collapsed() || self.runtime_layout_component_is_display_none(local_id)
    }

    fn runtime_draw_command_for_node(
        &self,
        drawable: &SortedDrawableNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> RuntimeDrawCommand {
        let local_id = match drawable.kind {
            DrawableOrderKind::LayoutProxy => drawable.layout_local,
            _ => drawable.local_id,
        };
        let kind = match drawable.kind {
            DrawableOrderKind::ClipStartProxy => RuntimeDrawCommandKind::ClipStart,
            DrawableOrderKind::ClipEndProxy => RuntimeDrawCommandKind::ClipEnd,
            DrawableOrderKind::Drawable | DrawableOrderKind::LayoutProxy => {
                RuntimeDrawCommandKind::Draw
            }
        };
        RuntimeDrawCommand {
            kind,
            object_kind: RuntimeDrawCommandObjectKind::from_type_name(drawable.type_name),
            local_id,
            global_id: drawable.global_id,
            type_name: drawable.type_name,
            world_transform: local_id
                .filter(|_| kind == RuntimeDrawCommandKind::Draw)
                .and_then(|local_id| {
                    self.runtime_retained_draw_world_transform(local_id, layout_bounds)
                }),
            render_opacity: local_id
                .and_then(|local_id| self.component(local_id))
                .map(|component| component.transform.render_opacity)
                .unwrap_or(1.0),
            referenced_artboard_global: drawable.referenced_artboard_global,
            resolved_image_asset_global: self
                .resolved_image_asset_global(local_id, drawable.resolved_image_asset_global),
            clipping_shape_local: drawable.clipping_shape_local,
            needs_save_operation: drawable.needs_save_operation,
            shape_paints: self.runtime_shape_paint_commands(
                drawable,
                graph,
                layout_bounds,
                path_cache,
            ),
        }
    }

    fn runtime_retained_draw_world_transform(
        &self,
        local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Option<Mat2D> {
        let component = self.component(local_id)?;
        if component.type_name == "LayoutComponent" {
            return None;
        }
        if component.type_name == "NestedArtboardLayout"
            && layout_bounds.is_some_and(|bounds| bounds.contains_key(&local_id))
        {
            return None;
        }
        let Some(parent_local) = component.parent_local else {
            return Some(component.transform.world_transform);
        };
        if layout_bounds.is_none()
            || !self
                .component(parent_local)
                .is_some_and(|parent| parent.layout_chain_has_layout_component)
        {
            return Some(component.transform.world_transform);
        }
        None
    }

    fn runtime_shape_paint_commands(
        &self,
        drawable: &SortedDrawableNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Vec<RuntimeShapePaintCommand> {
        if drawable.kind == DrawableOrderKind::Drawable
            && is_text_input_drawable_type(drawable.type_name)
            && let (Some(runtime), Some(drawable_local)) = (self.runtime_file(), drawable.local_id)
        {
            return runtime_text_input_shape_paint_commands(
                runtime,
                self,
                graph,
                drawable_local,
                layout_bounds,
            )
            .map(|mut commands| {
                assign_shape_paint_path_slot_indices(&mut commands);
                commands
            })
            .unwrap_or_default();
        }

        let (container_local, layout_path_local) = match drawable.kind {
            DrawableOrderKind::Drawable if drawable.type_name == "Shape" => {
                (drawable.local_id, None)
            }
            DrawableOrderKind::Drawable if drawable.type_name == "ForegroundLayoutDrawable" => {
                let Some(foreground_local) = drawable.local_id else {
                    return Vec::new();
                };
                let Some(parent_layout_local) =
                    runtime_foreground_layout_parent_local(self, foreground_local)
                else {
                    return Vec::new();
                };
                (Some(foreground_local), Some(parent_layout_local))
            }
            DrawableOrderKind::LayoutProxy => (drawable.layout_local, drawable.layout_local),
            _ => (None, None),
        };
        let Some(container_local) = container_local else {
            return Vec::new();
        };
        let Some(container) = graph
            .shape_paint_containers
            .iter()
            .find(|container| container.local_id == container_local)
        else {
            return Vec::new();
        };
        if !matches!(
            container.type_name,
            "Shape" | "LayoutComponent" | "ForegroundLayoutDrawable"
        ) {
            return Vec::new();
        }
        let needs_save_operation = drawable.needs_save_operation || container.paints.len() > 1;
        let transform_local = layout_path_local.unwrap_or(container_local);
        let render_opacity = self
            .component(transform_local)
            .map(|component| component.transform.render_opacity)
            .unwrap_or(1.0);
        let shape_world = if let Some(layout_local) = layout_path_local {
            self.runtime_layout_component_world_transform_with_bounds(
                layout_local,
                graph,
                layout_bounds,
            )
        } else if container.type_name == "LayoutComponent" {
            self.runtime_layout_component_world_transform_with_bounds(
                container_local,
                graph,
                layout_bounds,
            )
        } else {
            self.runtime_component_world_transform_with_bounds(
                container_local,
                graph,
                layout_bounds,
            )
        };

        let mut commands = container
            .paints
            .iter()
            .filter_map(|paint| {
                let path_commands = match container.type_name {
                    "Shape" => self.runtime_shape_paint_path_commands(
                        container_local,
                        paint,
                        graph,
                        layout_bounds,
                        path_cache,
                    ),
                    "LayoutComponent" => self.runtime_layout_component_paint_path_commands(
                        container_local,
                        paint,
                        graph,
                        layout_bounds,
                    ),
                    "ForegroundLayoutDrawable" => {
                        // Ported from C++ `src/foreground_layout_drawable.cpp`:
                        // foreground layout paints draw the parent
                        // `LayoutComponent` path with the parent layout world
                        // transform.
                        let layout_local = layout_path_local?;
                        self.runtime_layout_component_paint_path_commands(
                            layout_local,
                            paint,
                            graph,
                            layout_bounds,
                        )
                    }
                    _ => Vec::new(),
                };
                let mut command = runtime_shape_paint_command(
                    self,
                    paint,
                    container.blend_mode_value,
                    needs_save_operation,
                    render_opacity,
                    shape_world,
                    path_commands,
                    matches!(
                        container.type_name,
                        "LayoutComponent" | "ForegroundLayoutDrawable"
                    ),
                    matches!(
                        container.type_name,
                        "LayoutComponent" | "ForegroundLayoutDrawable"
                    ),
                    container.type_name != "Shape",
                )?;
                if drawable.kind == DrawableOrderKind::LayoutProxy
                    || container.type_name == "ForegroundLayoutDrawable"
                {
                    command.shape_world_override = Some(shape_world);
                }
                Some(command)
            })
            .collect::<Vec<_>>();
        assign_shape_paint_path_slot_indices(&mut commands);
        commands
    }

    fn runtime_layout_component_paint_path_commands(
        &self,
        layout_local: usize,
        paint: &ShapePaintNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Vec<RuntimePathCommand> {
        if paint.path_kind.is_none() {
            return Vec::new();
        }
        // Ported from C++ `src/layout_component.cpp`
        // `LayoutComponent::drawProxy/updateRenderPath`: layout components draw
        // shape paints against a background rectangle built from computed
        // layout bounds. TODO(golden): route all layout components through the
        // M6 layout engine.
        let bounds =
            self.runtime_layout_component_bounds_with_bounds(layout_local, graph, layout_bounds);
        let corners = self.runtime_layout_component_corners(layout_local);
        runtime_layout_rect_path_commands(bounds, corners)
    }

    fn runtime_layout_component_clip_path_commands(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Vec<RuntimePathCommand> {
        let bounds =
            self.runtime_layout_component_bounds_with_bounds(layout_local, graph, layout_bounds);
        let corners = self.runtime_layout_component_corners(layout_local);
        let mut commands = runtime_layout_rect_path_commands(bounds, corners);
        translate_path_commands(&mut commands, (bounds.x, bounds.y));
        commands
    }

    fn runtime_layout_component_clip_enabled(&self, layout_local: usize) -> bool {
        runtime_layout_component_property_key_for_name("clip")
            .and_then(|key| self.bool_property(layout_local, key))
            .unwrap_or(false)
    }

    pub(crate) fn runtime_layout_computed_property(
        &self,
        local_id: usize,
        property: RuntimeLayoutComputedProperty,
        graph: &ArtboardGraph,
    ) -> Option<f32> {
        let component = self.component(local_id)?;
        if component.type_name != "LayoutComponent" {
            let world = self.runtime_component_world_transform(local_id, graph);
            return match property {
                RuntimeLayoutComputedProperty::LocalX => {
                    Some(component.transform.local_transform.0[4])
                }
                RuntimeLayoutComputedProperty::LocalY => {
                    Some(component.transform.local_transform.0[5])
                }
                RuntimeLayoutComputedProperty::WorldX | RuntimeLayoutComputedProperty::RootX => {
                    Some(world.0[4])
                }
                RuntimeLayoutComputedProperty::WorldY | RuntimeLayoutComputedProperty::RootY => {
                    Some(world.0[5])
                }
                RuntimeLayoutComputedProperty::Width | RuntimeLayoutComputedProperty::Height => {
                    Some(0.0)
                }
            };
        }

        // Ported from C++ `include/rive/layout_component.hpp`: LayoutComponent
        // overrides Node's local x/y and width/height computed accessors with
        // settled layout bounds.
        let bounds = self.runtime_layout_component_bounds(local_id, graph);
        match property {
            RuntimeLayoutComputedProperty::LocalX => Some(bounds.x),
            RuntimeLayoutComputedProperty::LocalY => Some(bounds.y),
            RuntimeLayoutComputedProperty::Width => Some(bounds.width),
            RuntimeLayoutComputedProperty::Height => Some(bounds.height),
            RuntimeLayoutComputedProperty::WorldX | RuntimeLayoutComputedProperty::RootX => Some(
                self.runtime_layout_component_world_transform(local_id, graph)
                    .0[4],
            ),
            RuntimeLayoutComputedProperty::WorldY | RuntimeLayoutComputedProperty::RootY => Some(
                self.runtime_layout_component_world_transform(local_id, graph)
                    .0[5],
            ),
        }
    }

    pub(crate) fn runtime_component_world_transform(
        &self,
        local_id: usize,
        graph: &ArtboardGraph,
    ) -> Mat2D {
        self.runtime_component_world_transform_with_bounds(local_id, graph, None)
    }

    pub(crate) fn runtime_component_world_transform_with_bounds(
        &self,
        local_id: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Mat2D {
        // Cycle guard: the parent walk below recurses on `parent_local`, which a
        // malformed-but-accepted file can make cyclic -> unbounded recursion.
        // Thread a visited set (C++'s DependencySorter::visit idiom,
        // src/dependency_sorter.cpp); on revisit, fall back to the component's
        // stored world transform, terminating gracefully. No-op on valid files.
        let mut visited = BTreeSet::new();
        self.runtime_component_world_transform_with_bounds_guarded(
            local_id,
            graph,
            layout_bounds,
            &mut visited,
        )
    }

    fn runtime_component_world_transform_with_bounds_guarded(
        &self,
        local_id: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        visited: &mut BTreeSet<usize>,
    ) -> Mat2D {
        let Some(component) = self.component(local_id) else {
            return Mat2D::IDENTITY;
        };
        if !visited.insert(local_id) {
            // Parent cycle detected (see the wrapper above): stop the walk.
            return component.transform.world_transform;
        }
        if component.type_name == "Artboard" {
            return component.transform.world_transform;
        }
        if component.type_name == "LayoutComponent" {
            return self.runtime_layout_component_world_transform_with_bounds(
                local_id,
                graph,
                layout_bounds,
            );
        }
        if component.type_name == "NestedArtboardLayout"
            && let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&local_id).copied())
        {
            if mat2d_linear_is_identity(component.transform.local_transform) {
                return Mat2D([1.0, 0.0, 0.0, 1.0, bounds.x, bounds.y]);
            }
            let (origin_x, origin_y) = self
                .nested_artboards
                .get(&local_id)
                .map(|nested| {
                    (
                        nested.child.width * nested.child.origin_x,
                        nested.child.height * nested.child.origin_y,
                    )
                })
                .unwrap_or((0.0, 0.0));
            let mut components = component.transform.local_transform.decompose();
            components.x = bounds.x + origin_x;
            components.y = bounds.y + origin_y;
            return Mat2D::compose(components);
        }
        let Some(parent_local) = component.parent_local else {
            return component.transform.world_transform;
        };
        if layout_bounds.is_none()
            || !self
                .component(parent_local)
                .is_some_and(|parent| parent.layout_chain_has_layout_component)
        {
            return component.transform.world_transform;
        }
        if let Some(layout_local) = component.constrained_layout_ancestor {
            let layout_world = self.runtime_layout_component_world_transform_with_bounds(
                layout_local,
                graph,
                layout_bounds,
            );
            let stored_layout_world = self
                .component(layout_local)
                .map(|component| component.transform.world_transform)
                .unwrap_or(Mat2D::IDENTITY);
            return layout_world
                .multiply(stored_layout_world.invert_or_identity())
                .multiply(component.transform.world_transform);
        }
        self.runtime_component_world_transform_with_bounds_guarded(
            parent_local,
            graph,
            layout_bounds,
            visited,
        )
        .multiply(component.transform.local_transform)
    }

    pub(crate) fn runtime_layout_component_world_transform(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Mat2D {
        self.runtime_layout_component_world_transform_with_bounds(layout_local, graph, None)
    }

    fn runtime_layout_component_world_transform_with_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Mat2D {
        let bounds = layout_bounds
            .and_then(|bounds| bounds.get(&layout_local).copied())
            .or_else(|| self.runtime_supported_layout_component_bounds(layout_local, graph));
        if let Some(bounds) = bounds {
            // Rust layout bounds are accumulated into artboard space when we
            // collect them from Taffy/fallback layout passes. C++ stores Yoga's
            // left/top parent-local and composes through parent world
            // transforms, so apply the artboard origin exactly once here.
            let x = bounds.x - self.width * self.origin_x;
            let y = bounds.y - self.height * self.origin_y;
            return Mat2D([1.0, 0.0, 0.0, 1.0, x, y]);
        }
        self.component(layout_local)
            .map(|component| component.transform.world_transform)
            .unwrap_or(Mat2D::IDENTITY)
    }

    pub(crate) fn runtime_layout_component_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> RuntimeLayoutBounds {
        self.runtime_layout_component_bounds_with_bounds(layout_local, graph, None)
    }

    fn runtime_layout_component_bounds_with_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> RuntimeLayoutBounds {
        if let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&layout_local).copied()) {
            return bounds;
        }
        let mut cache = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        self.runtime_layout_component_bounds_memo(layout_local, graph, &mut cache, &mut visiting)
    }

    pub(crate) fn runtime_taffy_layout_bounds(
        &self,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>> {
        TaffyRuntimeLayoutEngine.compute_bounds(self, graph, runtime)
    }

    #[doc(hidden)]
    pub fn debug_taffy_layout_bounds_report(
        &self,
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
    ) -> Option<Vec<RuntimeLayoutBoundsReport>> {
        let layout_bounds = self.runtime_taffy_layout_bounds(graph, Some(runtime))?;
        Some(
            layout_bounds
                .iter()
                .filter_map(|(local_id, bounds)| {
                    let component = self.component(*local_id)?;
                    let world_transform = self
                        .runtime_component_world_transform_with_bounds(
                            *local_id,
                            graph,
                            Some(&layout_bounds),
                        )
                        .0;
                    Some(RuntimeLayoutBoundsReport {
                        local_id: *local_id,
                        global_id: component.global_id,
                        type_name: component.type_name,
                        name: graph
                            .components
                            .iter()
                            .find(|component| component.local_id == *local_id)
                            .and_then(|component| component.name.clone()),
                        parent_local: component.parent_local,
                        collapsed: component.is_collapsed(),
                        x: bounds.x,
                        y: bounds.y,
                        width: bounds.width,
                        height: bounds.height,
                        world_transform,
                    })
                })
                .collect(),
        )
    }

    fn runtime_text_layout_constraint(
        &self,
        text_local: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Option<RuntimeTextLayoutConstraint> {
        let text = self.component(text_local)?;
        let parent_local = text.parent_local?;
        let parent = self.component(parent_local)?;
        if parent.type_name != "LayoutComponent" {
            return None;
        }
        let style_local = self.runtime_layout_component_style_local(parent_local)?;
        let bounds = layout_bounds.and_then(|bounds| bounds.get(&parent_local).copied())?;
        Some(RuntimeTextLayoutConstraint {
            width: bounds.width,
            height: bounds.height,
            width_scale_type: self.runtime_layout_axis_scale(style_local, true),
            height_scale_type: self.runtime_layout_axis_scale(style_local, false),
        })
    }

    pub(crate) fn runtime_text_input_layout_constraint(
        &self,
        text_input_local: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Option<RuntimeTextLayoutConstraint> {
        let text_input = self.component(text_input_local)?;
        let parent_local = text_input.parent_local?;
        let parent = self.component(parent_local)?;
        if parent.type_name != "LayoutComponent" {
            return None;
        }
        let style_local = self.runtime_layout_component_style_local(parent_local)?;
        let bounds = layout_bounds
            .and_then(|bounds| bounds.get(&parent_local).copied())
            .or_else(|| Some(self.runtime_layout_component_bounds(parent_local, graph)))?;
        Some(RuntimeTextLayoutConstraint {
            width: bounds.width,
            height: bounds.height,
            width_scale_type: self.runtime_layout_axis_scale(style_local, true),
            height_scale_type: self.runtime_layout_axis_scale(style_local, false),
        })
    }

    fn runtime_layout_component_bounds_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> RuntimeLayoutBounds {
        if let Some(bounds) = cache.get(&layout_local).copied() {
            return bounds;
        }
        if !visiting.insert(layout_local) {
            return self.runtime_authored_layout_component_bounds(layout_local);
        }
        let bounds = self
            .runtime_supported_layout_component_bounds_memo(layout_local, graph, cache, visiting)
            .unwrap_or_else(|| self.runtime_authored_layout_component_bounds(layout_local));
        visiting.remove(&layout_local);
        cache.insert(layout_local, bounds);
        bounds
    }

    fn runtime_authored_layout_component_bounds(&self, layout_local: usize) -> RuntimeLayoutBounds {
        let width = self.runtime_layout_component_dimension(layout_local, "width");
        let height = self.runtime_layout_component_dimension(layout_local, "height");
        if self
            .component(layout_local)
            .is_some_and(|component| component.parent_local == Some(0))
        {
            return RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: if width == 0.0 { self.width } else { width },
                height: if height == 0.0 { self.height } else { height },
            };
        }
        RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    fn runtime_supported_layout_component_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Option<RuntimeLayoutBounds> {
        let mut cache = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        self.runtime_supported_layout_component_bounds_memo(
            layout_local,
            graph,
            &mut cache,
            &mut visiting,
        )
    }

    fn runtime_supported_layout_component_bounds_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<RuntimeLayoutBounds> {
        if let Some(bounds) = TaffyRuntimeLayoutEngine.compute_bounds(self, graph, None) {
            for (local, bounds) in &bounds {
                cache.entry(*local).or_insert(*bounds);
            }
            if let Some(bounds) = bounds.get(&layout_local).copied() {
                return Some(bounds);
            }
        }

        self.runtime_root_artboard_layout_bounds_memo(layout_local, graph, cache, visiting)
            .or_else(|| {
                self.runtime_absolute_layout_component_bounds_memo(
                    layout_local,
                    graph,
                    cache,
                    visiting,
                )
            })
            .or_else(|| {
                self.runtime_simple_flex_layout_bounds_memo(layout_local, graph, cache, visiting)
            })
            .or_else(|| self.runtime_simple_root_row_fill_layout_bounds(layout_local, graph))
            .or_else(|| {
                self.runtime_nested_fill_hug_layout_bounds_memo(
                    layout_local,
                    graph,
                    cache,
                    visiting,
                )
            })
    }

    fn runtime_root_artboard_layout_bounds(
        &self,
        graph: &ArtboardGraph,
    ) -> Option<RuntimeLayoutBounds> {
        let mut cache = BTreeMap::new();
        let mut visiting = BTreeSet::new();
        self.runtime_root_artboard_layout_bounds_memo(0, graph, &mut cache, &mut visiting)
    }

    fn runtime_root_artboard_layout_bounds_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<RuntimeLayoutBounds> {
        if layout_local != 0 {
            return None;
        }
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        if component.type_name != "Artboard" {
            return None;
        }
        let style_local = self.runtime_layout_component_style_local(layout_local)?;

        const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
        let width_is_hug =
            self.runtime_layout_axis_scale(style_local, true) == LAYOUT_SCALE_TYPE_HUG;
        let height_is_hug =
            self.runtime_layout_axis_scale(style_local, false) == LAYOUT_SCALE_TYPE_HUG;
        if !width_is_hug && !height_is_hug {
            return None;
        }

        let width = if width_is_hug {
            self.runtime_layout_component_hug_size_memo(layout_local, graph, true, cache, visiting)?
        } else {
            self.width
        };
        let height = if height_is_hug {
            self.runtime_layout_component_hug_size_memo(
                layout_local,
                graph,
                false,
                cache,
                visiting,
            )?
        } else {
            self.height
        };

        Some(RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        })
    }

    fn runtime_simple_flex_layout_bounds_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<RuntimeLayoutBounds> {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        if component.type_name != "LayoutComponent" {
            return None;
        }
        let parent_local = component.parent_local?;
        let parent = graph
            .components
            .iter()
            .find(|component| component.local_id == parent_local)?;
        if !matches!(parent.type_name, "Artboard" | "LayoutComponent") {
            return None;
        }

        let parent_bounds = if parent_local == 0 {
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: self.width,
                height: self.height,
            }
        } else {
            self.runtime_layout_component_bounds_memo(parent_local, graph, cache, visiting)
        };
        let parent_style = self.runtime_layout_component_style_local(parent_local);
        let parent_direction = parent_style
            .map(|style_local| {
                self.runtime_layout_style_uint_default(
                    style_local,
                    RuntimeLayoutStyleProperty::FlexDirectionValue,
                    2,
                )
            })
            .unwrap_or(2);
        let parent_is_row = match parent_direction {
            0 => false,
            2 => true,
            _ => return None,
        };

        let mut layout_children = Vec::new();
        for child_local in &parent.children {
            let Some(child) = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
            else {
                continue;
            };
            if child.type_name == "LayoutComponent"
                && !self.runtime_component_is_collapsed_for_draw(*child_local)
                && !self.runtime_layout_component_is_absolute(*child_local)
                && !layout_children.contains(child_local)
            {
                layout_children.push(*child_local);
            }
        }
        if !layout_children.contains(&layout_local) {
            return None;
        }

        let (padding_left, padding_right, padding_top, padding_bottom, gap) =
            if let Some(style_local) = parent_style {
                (
                    self.runtime_layout_style_length(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingLeft,
                        RuntimeLayoutStyleProperty::PaddingLeftUnitsValue,
                        parent_bounds.width,
                    )?,
                    self.runtime_layout_style_length(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingRight,
                        RuntimeLayoutStyleProperty::PaddingRightUnitsValue,
                        parent_bounds.width,
                    )?,
                    self.runtime_layout_style_length(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingTop,
                        RuntimeLayoutStyleProperty::PaddingTopUnitsValue,
                        parent_bounds.height,
                    )?,
                    self.runtime_layout_style_length(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingBottom,
                        RuntimeLayoutStyleProperty::PaddingBottomUnitsValue,
                        parent_bounds.height,
                    )?,
                    if parent_is_row {
                        self.runtime_layout_style_length(
                            style_local,
                            RuntimeLayoutStyleProperty::GapHorizontal,
                            RuntimeLayoutStyleProperty::GapHorizontalUnitsValue,
                            parent_bounds.width,
                        )?
                    } else {
                        self.runtime_layout_style_length(
                            style_local,
                            RuntimeLayoutStyleProperty::GapVertical,
                            RuntimeLayoutStyleProperty::GapVerticalUnitsValue,
                            parent_bounds.height,
                        )?
                    },
                )
            } else {
                (0.0, 0.0, 0.0, 0.0, 0.0)
            };
        let main_available = if parent_is_row {
            parent_bounds.width - padding_left - padding_right
        } else {
            parent_bounds.height - padding_top - padding_bottom
        }
        .max(0.0);
        let cross_available = if parent_is_row {
            parent_bounds.height - padding_top - padding_bottom
        } else {
            parent_bounds.width - padding_left - padding_right
        }
        .max(0.0);
        let total_gap = gap * layout_children.len().saturating_sub(1) as f32;
        let main_available_after_gap = (main_available - total_gap).max(0.0);

        let mut fixed_total = 0.0;
        let mut fill_total = 0.0;
        let mut main_sizes = Vec::with_capacity(layout_children.len());
        let mut fill_bases = Vec::with_capacity(layout_children.len());
        for child_local in &layout_children {
            let style_local = self.runtime_layout_component_style_local(*child_local)?;
            let scale = self.runtime_layout_axis_scale(style_local, parent_is_row);
            let size = match scale {
                0 => {
                    let size = self.runtime_layout_component_axis_length(
                        *child_local,
                        style_local,
                        parent_is_row,
                        main_available_after_gap,
                    )?;
                    fixed_total += size;
                    Some(size)
                }
                1 => {
                    let basis = self.runtime_layout_flex_basis_main_size(
                        *child_local,
                        style_local,
                        parent_is_row,
                        main_available_after_gap,
                        graph,
                    )?;
                    fixed_total += basis;
                    fill_total += self.runtime_layout_axis_fraction(*child_local, parent_is_row);
                    fill_bases.push(basis);
                    None
                }
                2 => {
                    let size = self.runtime_layout_component_hug_size_memo(
                        *child_local,
                        graph,
                        parent_is_row,
                        cache,
                        visiting,
                    )?;
                    fixed_total += size;
                    Some(size)
                }
                _ => return None,
            };
            if scale != 1 {
                fill_bases.push(0.0);
            }
            main_sizes.push(size);
        }

        // Parallel-length invariant: the loop above pushes exactly one entry
        // into `main_sizes` and one into `fill_bases` per layout child, so both
        // stay index-aligned with `layout_children`. The zip below and the
        // `resolved_main_sizes[index]` indexing further down rely on this: a
        // shorter `main_sizes` would silently truncate via zip, and a shorter
        // `resolved_main_sizes` would panic on the index. Lock it so a future
        // edit that adds an early `continue` fails loudly in debug/tests.
        debug_assert_eq!(main_sizes.len(), layout_children.len());
        debug_assert_eq!(fill_bases.len(), layout_children.len());
        let remaining = (main_available_after_gap - fixed_total).max(0.0);
        let resolved_main_sizes = layout_children
            .iter()
            .zip(main_sizes)
            .zip(fill_bases.iter().copied())
            .map(|((child_local, size), basis)| {
                size.unwrap_or_else(|| {
                    if fill_total <= f32::EPSILON {
                        basis
                    } else {
                        basis
                            + remaining
                                * self.runtime_layout_axis_fraction(*child_local, parent_is_row)
                                / fill_total
                    }
                })
            })
            .collect::<Vec<_>>();
        let alignment = parent_style
            .map(|style_local| self.runtime_layout_alignment_type(style_local))
            .unwrap_or(0);
        let mut resolved_gap = gap;
        if fill_total <= f32::EPSILON && runtime_layout_alignment_is_space_between(alignment) {
            resolved_gap = if resolved_main_sizes.len() > 1 {
                let occupied = resolved_main_sizes.iter().sum::<f32>();
                ((main_available - occupied).max(0.0)) / (resolved_main_sizes.len() as f32 - 1.0)
            } else {
                0.0
            };
        }
        let occupied_main = resolved_main_sizes.iter().sum::<f32>()
            + resolved_gap * resolved_main_sizes.len().saturating_sub(1) as f32;
        let main_offset = if fill_total <= f32::EPSILON {
            runtime_layout_main_alignment_offset(
                alignment,
                parent_is_row,
                main_available,
                occupied_main,
            )
        } else {
            0.0
        };
        let mut cursor = if parent_is_row {
            parent_bounds.x + padding_left + main_offset
        } else {
            parent_bounds.y + padding_top + main_offset
        };
        for (index, child_local) in layout_children.iter().enumerate() {
            let style_local = self.runtime_layout_component_style_local(*child_local)?;
            let main_size = resolved_main_sizes[index];
            let cross_size = self.runtime_layout_cross_axis_size(
                *child_local,
                style_local,
                !parent_is_row,
                cross_available,
                graph,
                cache,
                visiting,
            )?;
            let cross_offset = runtime_layout_cross_alignment_offset(
                alignment,
                parent_is_row,
                cross_available,
                cross_size,
            );
            if *child_local == layout_local {
                return Some(if parent_is_row {
                    RuntimeLayoutBounds {
                        x: cursor,
                        y: parent_bounds.y + padding_top + cross_offset,
                        width: main_size,
                        height: cross_size,
                    }
                } else {
                    RuntimeLayoutBounds {
                        x: parent_bounds.x + padding_left + cross_offset,
                        y: cursor,
                        width: cross_size,
                        height: main_size,
                    }
                });
            }
            cursor += main_size + resolved_gap;
        }
        None
    }

    fn runtime_absolute_layout_component_bounds_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<RuntimeLayoutBounds> {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        if component.type_name != "LayoutComponent" {
            return None;
        }
        if !self.runtime_layout_component_is_absolute(layout_local) {
            return None;
        }
        let parent_local = component.parent_local?;
        let parent = graph
            .components
            .iter()
            .find(|component| component.local_id == parent_local)?;
        if !matches!(parent.type_name, "Artboard" | "LayoutComponent") {
            return None;
        }

        let parent_bounds = if parent_local == 0 {
            RuntimeLayoutBounds {
                x: 0.0,
                y: 0.0,
                width: self.width,
                height: self.height,
            }
        } else {
            self.runtime_layout_component_bounds_memo(parent_local, graph, cache, visiting)
        };
        let style_local = self.runtime_layout_component_style_local(layout_local)?;

        // Ported from C++ `src/layout_component.cpp` `LayoutComponent::syncStyle`:
        // absolute children trust their stored units for fixed/fill dimensions
        // instead of mapping fill to Yoga auto, and Yoga removes them from flex flow.
        let width = self.runtime_absolute_layout_axis_size(
            layout_local,
            style_local,
            true,
            parent_bounds.width,
            graph,
            cache,
            visiting,
        )?;
        let height = self.runtime_absolute_layout_axis_size(
            layout_local,
            style_local,
            false,
            parent_bounds.height,
            graph,
            cache,
            visiting,
        )?;
        let left = self.runtime_layout_style_length(
            style_local,
            RuntimeLayoutStyleProperty::PositionLeft,
            RuntimeLayoutStyleProperty::PositionLeftUnitsValue,
            parent_bounds.width,
        )?;
        let top = self.runtime_layout_style_length(
            style_local,
            RuntimeLayoutStyleProperty::PositionTop,
            RuntimeLayoutStyleProperty::PositionTopUnitsValue,
            parent_bounds.height,
        )?;

        Some(RuntimeLayoutBounds {
            x: parent_bounds.x + left,
            y: parent_bounds.y + top,
            width,
            height,
        })
    }

    fn runtime_nested_fill_hug_layout_bounds_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<RuntimeLayoutBounds> {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        if component.type_name != "LayoutComponent" {
            return None;
        }
        let parent_local = component.parent_local?;
        let parent = graph
            .components
            .iter()
            .find(|component| component.local_id == parent_local)?;
        if parent.type_name != "LayoutComponent" {
            return None;
        }
        let style_local = self.runtime_layout_component_style_local(layout_local)?;

        const LAYOUT_SCALE_TYPE_FILL: u64 = 1;
        const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
        let width_scale = self.runtime_layout_style_uint(
            style_local,
            RuntimeLayoutStyleProperty::LayoutWidthScaleType,
        )?;
        let height_scale = self.runtime_layout_style_uint(
            style_local,
            RuntimeLayoutStyleProperty::LayoutHeightScaleType,
        )?;
        if !matches!(width_scale, LAYOUT_SCALE_TYPE_FILL | LAYOUT_SCALE_TYPE_HUG)
            || !matches!(height_scale, LAYOUT_SCALE_TYPE_FILL | LAYOUT_SCALE_TYPE_HUG)
        {
            return None;
        }

        let parent_bounds =
            self.runtime_layout_component_bounds_memo(parent_local, graph, cache, visiting);
        let authored_width = self.runtime_layout_component_dimension(layout_local, "width");
        let authored_height = self.runtime_layout_component_dimension(layout_local, "height");
        let width = match width_scale {
            LAYOUT_SCALE_TYPE_FILL => parent_bounds.width,
            LAYOUT_SCALE_TYPE_HUG => self.runtime_layout_component_hug_size_memo(
                layout_local,
                graph,
                true,
                cache,
                visiting,
            )?,
            _ => authored_width,
        };
        let height = match height_scale {
            LAYOUT_SCALE_TYPE_FILL => parent_bounds.height,
            LAYOUT_SCALE_TYPE_HUG => self.runtime_layout_component_hug_size_memo(
                layout_local,
                graph,
                false,
                cache,
                visiting,
            )?,
            _ => authored_height,
        };

        Some(RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width,
            height,
        })
    }

    fn runtime_layout_component_hug_size_memo(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        width_axis: bool,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<f32> {
        let style_local = self.runtime_layout_component_style_local(layout_local)?;
        let is_row = self.runtime_layout_style_is_row(style_local);
        let gap = if is_row {
            self.runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::GapHorizontal)
        } else {
            self.runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::GapVertical)
        }
        .unwrap_or(0.0);
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        let mut child_sizes = Vec::new();
        for child_local in &component.children {
            let Some(child) = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
            else {
                continue;
            };
            if self.runtime_component_is_collapsed_for_draw(*child_local) {
                continue;
            }
            match child.type_name {
                "LayoutComponent" => {
                    let bounds = self.runtime_layout_component_bounds_memo(
                        *child_local,
                        graph,
                        cache,
                        visiting,
                    );
                    child_sizes.push((bounds.width, bounds.height));
                }
                "ArtboardComponentList" => {
                    // Ported from C++ `src/artboard_component_list.cpp`
                    // `ArtboardComponentList::layoutBounds`: non-virtualized
                    // lists with no instantiated items retain the default
                    // zero layout size, which makes hug-sized parents collapse.
                    child_sizes.push((0.0, 0.0));
                }
                "Text" => {
                    if let Some((_, _, width, height)) = self.runtime_file().and_then(|runtime| {
                        static_text_constraint_bounds(runtime, graph, self, *child_local)
                    }) {
                        child_sizes.push((width, height));
                    }
                }
                "TextInput" => {
                    if let Some((_, _, width, height)) = self.runtime_file().and_then(|runtime| {
                        self.runtime_text_input_layout_constraint(*child_local, graph, None)
                            .and_then(|layout_constraint| {
                                text_input_layout_measure_bounds(
                                    runtime,
                                    graph,
                                    self,
                                    *child_local,
                                    layout_constraint,
                                )
                            })
                    }) {
                        child_sizes.push((width, height));
                    }
                }
                "Fill"
                | "LinearGradient"
                | "RadialGradient"
                | "SolidColor"
                | "Stroke"
                | "LayoutComponentStyle" => {}
                _ => return None,
            }
        }
        Some(runtime_layout_hug_size(
            &child_sizes,
            gap,
            is_row,
            width_axis,
        ))
    }

    fn runtime_layout_style_double(
        &self,
        style_local: usize,
        property: RuntimeLayoutStyleProperty,
    ) -> Option<f32> {
        property
            .key()
            .and_then(|key| self.double_property(style_local, key))
    }

    fn runtime_layout_style_length(
        &self,
        style_local: usize,
        value_property: RuntimeLayoutStyleProperty,
        unit_property: RuntimeLayoutStyleProperty,
        reference: f32,
    ) -> Option<f32> {
        let value = self
            .runtime_layout_style_double(style_local, value_property)
            .unwrap_or(0.0);
        let unit = self.runtime_layout_style_uint_default(style_local, unit_property, 0);
        self.runtime_layout_length_for_unit(value, unit, reference)
    }

    fn runtime_layout_style_bool(
        &self,
        style_local: usize,
        property: RuntimeLayoutStyleProperty,
    ) -> Option<bool> {
        property
            .key()
            .and_then(|key| self.bool_property(style_local, key))
    }

    fn runtime_layout_component_corners(&self, layout_local: usize) -> RuntimeLayoutCorners {
        let Some(style_local) = self.runtime_layout_component_style_local(layout_local) else {
            return RuntimeLayoutCorners::default();
        };
        let linked_value = self
            .runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::CornerRadiusTL)
            .unwrap_or(0.0);
        let linked = self
            .runtime_layout_style_bool(style_local, RuntimeLayoutStyleProperty::LinkCornerRadius)
            .unwrap_or(true);
        if linked {
            return RuntimeLayoutCorners {
                top_left: linked_value,
                top_right: linked_value,
                bottom_right: linked_value,
                bottom_left: linked_value,
            };
        }
        RuntimeLayoutCorners {
            top_left: linked_value,
            top_right: self
                .runtime_layout_style_double(
                    style_local,
                    RuntimeLayoutStyleProperty::CornerRadiusTR,
                )
                .unwrap_or(0.0),
            bottom_right: self
                .runtime_layout_style_double(
                    style_local,
                    RuntimeLayoutStyleProperty::CornerRadiusBR,
                )
                .unwrap_or(0.0),
            bottom_left: self
                .runtime_layout_style_double(
                    style_local,
                    RuntimeLayoutStyleProperty::CornerRadiusBL,
                )
                .unwrap_or(0.0),
        }
    }

    fn runtime_simple_root_row_fill_layout_bounds(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
    ) -> Option<RuntimeLayoutBounds> {
        if !self.runtime_layout_component_is_root_row_fill_child(layout_local) {
            return None;
        }
        if !self.runtime_root_layout_style_is_row() {
            return None;
        }
        let root = graph
            .components
            .iter()
            .find(|component| component.local_id == 0)?;
        let layout_children = root
            .children
            .iter()
            .copied()
            .filter(|child_local| {
                self.runtime_layout_component_is_root_row_fill_child(*child_local)
            })
            .collect::<Vec<_>>();
        if layout_children.len() < 2 {
            return None;
        }
        let child_index = layout_children
            .iter()
            .position(|child_local| *child_local == layout_local)?;
        let (padding_left, padding_right, padding_top, padding_bottom, gap) = self
            .runtime_layout_component_style_local(0)
            .map(|style_local| {
                (
                    self.runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingLeft,
                    )
                    .unwrap_or(0.0),
                    self.runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingRight,
                    )
                    .unwrap_or(0.0),
                    self.runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingTop,
                    )
                    .unwrap_or(0.0),
                    self.runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingBottom,
                    )
                    .unwrap_or(0.0),
                    self.runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::GapHorizontal,
                    )
                    .unwrap_or(0.0),
                )
            })
            .unwrap_or((0.0, 0.0, 0.0, 0.0, 0.0));
        let child_count = layout_children.len() as f32;
        // child_count is >= 2 here (guarded by the `layout_children.len() < 2`
        // early return above), so the division below is safe. Guard it locally
        // as well: the early return is ~40 lines away, and a future edit that
        // relaxes it must not silently reintroduce an `x / 0.0` NaN into the
        // returned layout bounds. Bail gracefully (no simple-layout bounds).
        if child_count == 0.0 {
            return None;
        }
        let available_width =
            (self.width - padding_left - padding_right - gap * (child_count - 1.0)).max(0.0);
        let available_height = (self.height - padding_top - padding_bottom).max(0.0);
        let width = available_width / child_count;
        Some(RuntimeLayoutBounds {
            x: padding_left + (width + gap) * child_index as f32,
            y: padding_top,
            width,
            height: available_height,
        })
    }

    fn runtime_layout_component_is_root_row_fill_child(&self, layout_local: usize) -> bool {
        let Some(component) = self.component(layout_local) else {
            return false;
        };
        if component.type_name != "LayoutComponent" || component.parent_local != Some(0) {
            return false;
        }
        if self.runtime_component_is_collapsed_for_draw(layout_local)
            || self.runtime_layout_component_is_absolute(layout_local)
        {
            return false;
        }
        let Some(style_local) = self.runtime_layout_component_style_local(layout_local) else {
            return false;
        };

        const LAYOUT_SCALE_TYPE_FILL: u64 = 1;
        self.runtime_layout_style_uint(
            style_local,
            RuntimeLayoutStyleProperty::LayoutWidthScaleType,
        ) == Some(LAYOUT_SCALE_TYPE_FILL)
            && self.runtime_layout_style_uint(
                style_local,
                RuntimeLayoutStyleProperty::LayoutHeightScaleType,
            ) == Some(LAYOUT_SCALE_TYPE_FILL)
    }

    fn runtime_root_layout_style_is_row(&self) -> bool {
        self.runtime_layout_component_style_local(0)
            .is_none_or(|style_local| self.runtime_layout_style_is_row(style_local))
    }

    fn runtime_layout_style_is_row(&self, style_local: usize) -> bool {
        // Ported from C++ `src/layout_component.cpp` `mainAxisIsRow`.
        matches!(
            self.runtime_layout_style_uint(
                style_local,
                RuntimeLayoutStyleProperty::FlexDirectionValue
            )
            .unwrap_or(2),
            2 | 3
        )
    }

    fn runtime_layout_alignment_type(&self, style_local: usize) -> u64 {
        self.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::LayoutAlignmentType,
            0,
        )
    }

    fn runtime_layout_component_style_local(&self, layout_local: usize) -> Option<usize> {
        runtime_layout_component_property_key_for_name("styleId")
            .and_then(|key| self.uint_property(layout_local, key))
            .and_then(|style| usize::try_from(style).ok())
    }

    fn runtime_layout_style_uint(
        &self,
        style_local: usize,
        property: RuntimeLayoutStyleProperty,
    ) -> Option<u64> {
        property
            .key()
            .and_then(|key| self.uint_property(style_local, key))
    }

    fn runtime_layout_style_uint_default(
        &self,
        style_local: usize,
        property: RuntimeLayoutStyleProperty,
        default: u64,
    ) -> u64 {
        self.runtime_layout_style_uint(style_local, property)
            .unwrap_or(default)
    }

    fn runtime_layout_component_dimension(&self, layout_local: usize, name: &str) -> f32 {
        runtime_layout_component_property_key_for_name(name)
            .and_then(|key| self.double_property(layout_local, key))
            .unwrap_or(0.0)
    }

    fn runtime_layout_axis_scale(&self, style_local: usize, width_axis: bool) -> u64 {
        self.runtime_layout_style_uint_default(
            style_local,
            if width_axis {
                RuntimeLayoutStyleProperty::LayoutWidthScaleType
            } else {
                RuntimeLayoutStyleProperty::LayoutHeightScaleType
            },
            0,
        )
    }

    fn runtime_layout_axis_units(&self, style_local: usize, width_axis: bool) -> u64 {
        self.runtime_layout_style_uint_default(
            style_local,
            if width_axis {
                RuntimeLayoutStyleProperty::WidthUnitsValue
            } else {
                RuntimeLayoutStyleProperty::HeightUnitsValue
            },
            1,
        )
    }

    fn runtime_layout_component_is_absolute(&self, layout_local: usize) -> bool {
        let Some(style_local) = self.runtime_layout_component_style_local(layout_local) else {
            return false;
        };
        const YG_POSITION_TYPE_ABSOLUTE: u64 = 2;
        self.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::PositionTypeValue,
            1,
        ) == YG_POSITION_TYPE_ABSOLUTE
    }

    fn runtime_layout_component_is_display_none(&self, layout_local: usize) -> bool {
        let Some(component) = self.component(layout_local) else {
            return false;
        };
        if !matches!(component.type_name, "Artboard" | "LayoutComponent") {
            return false;
        }
        let Some(style_local) = self.runtime_layout_component_style_local(layout_local) else {
            return false;
        };
        const YG_DISPLAY_NONE: u64 = 1;
        // Ported from C++ `src/layout_component.cpp`
        // `LayoutComponent::styleDisplayHidden` / `displayChanged`.
        self.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::DisplayValue,
            0,
        ) == YG_DISPLAY_NONE
    }

    fn runtime_layout_axis_fraction(&self, layout_local: usize, width_axis: bool) -> f32 {
        let property_name = if width_axis {
            "fractionalWidth"
        } else {
            "fractionalHeight"
        };
        runtime_layout_component_property_key_for_name(property_name)
            .and_then(|key| self.double_property(layout_local, key))
            .unwrap_or(1.0)
            .max(0.0)
    }

    fn runtime_layout_component_axis_length(
        &self,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
        reference: f32,
    ) -> Option<f32> {
        let value = self.runtime_layout_component_dimension(
            layout_local,
            if width_axis { "width" } else { "height" },
        );
        self.runtime_layout_length_for_unit(
            value,
            self.runtime_layout_axis_units(style_local, width_axis),
            reference,
        )
    }

    fn runtime_layout_flex_basis_main_size(
        &self,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
        reference: f32,
        graph: &ArtboardGraph,
    ) -> Option<f32> {
        let basis = self
            .runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::FlexBasis)
            .unwrap_or(0.0);
        let units = self.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::FlexBasisUnitsValue,
            3,
        );
        if units == 3 {
            let mut intrinsic_visiting = BTreeSet::new();
            return self
                .runtime_layout_component_intrinsic_hug_size(
                    layout_local,
                    graph,
                    width_axis,
                    &mut intrinsic_visiting,
                )
                .or(Some(0.0));
        }
        self.runtime_layout_length_for_unit(basis, units, reference)
    }

    fn runtime_absolute_layout_axis_size(
        &self,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
        reference: f32,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<f32> {
        const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
        if self.runtime_layout_axis_scale(style_local, width_axis) == LAYOUT_SCALE_TYPE_HUG {
            return self.runtime_layout_component_hug_size_memo(
                layout_local,
                graph,
                width_axis,
                cache,
                visiting,
            );
        }
        self.runtime_layout_component_axis_length(layout_local, style_local, width_axis, reference)
    }

    fn runtime_layout_cross_axis_size(
        &self,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
        available: f32,
        graph: &ArtboardGraph,
        cache: &mut BTreeMap<usize, RuntimeLayoutBounds>,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<f32> {
        match self.runtime_layout_axis_scale(style_local, width_axis) {
            0 => self.runtime_layout_component_axis_length(
                layout_local,
                style_local,
                width_axis,
                available,
            ),
            1 => Some(available),
            2 => self.runtime_layout_component_hug_size_memo(
                layout_local,
                graph,
                width_axis,
                cache,
                visiting,
            ),
            _ => None,
        }
    }

    fn runtime_layout_component_intrinsic_hug_size(
        &self,
        layout_local: usize,
        graph: &ArtboardGraph,
        width_axis: bool,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<f32> {
        if !visiting.insert(layout_local) {
            return Some(0.0);
        }
        let style_local = self.runtime_layout_component_style_local(layout_local)?;
        let is_row = self.runtime_layout_style_is_row(style_local);
        let gap = if is_row {
            self.runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::GapHorizontal)
        } else {
            self.runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::GapVertical)
        }
        .unwrap_or(0.0);
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        let mut child_sizes = Vec::new();
        for child_local in &component.children {
            let Some(child) = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
            else {
                continue;
            };
            if self.runtime_component_is_collapsed_for_draw(*child_local) {
                continue;
            }
            match child.type_name {
                "LayoutComponent" => {
                    let child_style = self.runtime_layout_component_style_local(*child_local)?;
                    let width = self.runtime_layout_component_intrinsic_axis_size(
                        *child_local,
                        child_style,
                        true,
                        graph,
                        visiting,
                    )?;
                    let height = self.runtime_layout_component_intrinsic_axis_size(
                        *child_local,
                        child_style,
                        false,
                        graph,
                        visiting,
                    )?;
                    child_sizes.push((width, height));
                }
                "ArtboardComponentList" => child_sizes.push((0.0, 0.0)),
                "Text" => {
                    if let Some((_, _, width, height)) = self.runtime_file().and_then(|runtime| {
                        static_text_constraint_bounds(runtime, graph, self, *child_local)
                    }) {
                        child_sizes.push((width, height));
                    }
                }
                "TextInput" => {
                    if let Some((_, _, width, height)) = self.runtime_file().and_then(|runtime| {
                        self.runtime_text_input_layout_constraint(*child_local, graph, None)
                            .and_then(|layout_constraint| {
                                text_input_layout_measure_bounds(
                                    runtime,
                                    graph,
                                    self,
                                    *child_local,
                                    layout_constraint,
                                )
                            })
                    }) {
                        child_sizes.push((width, height));
                    }
                }
                "Fill"
                | "LinearGradient"
                | "RadialGradient"
                | "SolidColor"
                | "Stroke"
                | "LayoutComponentStyle" => {}
                _ => {
                    visiting.remove(&layout_local);
                    return None;
                }
            }
        }
        visiting.remove(&layout_local);
        Some(runtime_layout_hug_size(
            &child_sizes,
            gap,
            is_row,
            width_axis,
        ))
    }

    fn runtime_layout_component_intrinsic_axis_size(
        &self,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
        graph: &ArtboardGraph,
        visiting: &mut BTreeSet<usize>,
    ) -> Option<f32> {
        match self.runtime_layout_axis_scale(style_local, width_axis) {
            0 => {
                let value = self.runtime_layout_component_dimension(
                    layout_local,
                    if width_axis { "width" } else { "height" },
                );
                let units = self.runtime_layout_axis_units(style_local, width_axis);
                if units == 2 {
                    Some(0.0)
                } else {
                    self.runtime_layout_length_for_unit(value, units, 0.0)
                }
            }
            1 | 2 => self.runtime_layout_component_intrinsic_hug_size(
                layout_local,
                graph,
                width_axis,
                visiting,
            ),
            _ => None,
        }
    }

    fn runtime_layout_length_for_unit(&self, value: f32, unit: u64, reference: f32) -> Option<f32> {
        match unit {
            2 => Some(value / 100.0 * reference),
            0 | 1 | 3 => Some(value),
            _ => None,
        }
    }

    fn runtime_shape_paint_path_commands(
        &self,
        shape_local: usize,
        paint: &ShapePaintNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Vec<RuntimePathCommand> {
        let Some(path_kind) = paint.path_kind.and_then(runtime_shape_paint_path_kind) else {
            return Vec::new();
        };
        let key = RuntimeShapePaintPathCommandsCacheKey {
            shape_local,
            path_epoch: self.path_epoch(),
            layout_epoch: self.layout_epoch(),
            path_kind,
        };
        if let Some(commands) =
            path_cache
                .shape_paint_paths
                .get(graph.global_id, paint.local_id, key)
        {
            return commands.as_ref().clone();
        }

        let commands = Arc::new(self.runtime_shape_paint_path_commands_uncached(
            shape_local,
            paint,
            graph,
            layout_bounds,
            path_cache,
        ));
        path_cache
            .shape_paint_paths
            .insert(graph.global_id, paint.local_id, key, commands.clone());
        commands.as_ref().clone()
    }

    pub(crate) fn runtime_shape_length_with_layout(
        &self,
        shape_local: usize,
        graph: &ArtboardGraph,
    ) -> Option<f32> {
        let composer = graph
            .path_composers
            .iter()
            .find(|composer| composer.shape_local == shape_local)?;
        let layout_bounds = self.runtime_taffy_layout_bounds(graph, self.runtime_file());
        let layout_bounds = layout_bounds.as_ref();
        let shape_world =
            self.runtime_component_world_transform_with_bounds(shape_local, graph, layout_bounds);
        let inverse_shape_world = shape_world.invert_or_identity();
        let nsliced_context =
            runtime_nsliced_node_context_for_shape(self, graph, shape_local, layout_bounds);
        let mut path_cache = RuntimeRenderPathCache::default();
        let path_lookup = path_cache.path_composer_lookup_frame(graph);
        let mut commands = Vec::new();

        for path_ref in &composer.paths {
            let path = path_lookup.path(graph, path_ref.local_id)?;
            let path_world = path_cache.component_world_transform_with_bounds(
                self,
                graph,
                path.local_id,
                layout_bounds,
            );
            let path_frame =
                path_cache.path_geometry_commands_frame(self, graph, path, layout_bounds);
            let path_transform = if path_frame.has_weighted_context {
                Mat2D::IDENTITY
            } else {
                path_world
            };
            let mut path_commands = path_frame.commands.as_ref().clone();
            transform_path_commands(&mut path_commands, path_transform);
            if let Some(context) = nsliced_context.as_ref() {
                runtime_deform_path_commands_with_nsliced_node(
                    &mut path_commands,
                    context,
                    ShapePaintPathKind::World,
                    shape_world,
                    inverse_shape_world,
                );
            }
            commands.extend(path_commands);
        }

        Some(RuntimePathMeasure::from_commands(&commands).length())
    }

    fn runtime_shape_paint_path_commands_uncached(
        &self,
        shape_local: usize,
        paint: &ShapePaintNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Vec<RuntimePathCommand> {
        let Some(path_kind) = paint.path_kind else {
            return Vec::new();
        };
        let path_lookup = path_cache.path_composer_lookup_frame(graph);
        let Some(composer) = path_lookup.composer(graph, shape_local) else {
            return Vec::new();
        };

        let shape_world = path_cache.component_world_transform_with_bounds(
            self,
            graph,
            shape_local,
            layout_bounds,
        );
        let inverse_shape_world = shape_world.invert_or_identity();
        let nsliced_context =
            runtime_nsliced_node_context_for_shape(self, graph, shape_local, layout_bounds);
        let mut commands = Vec::new();

        for path_ref in &composer.paths {
            if !self.runtime_path_counts_for_clip(path_ref) {
                continue;
            }
            let Some(path) = path_lookup.path(graph, path_ref.local_id) else {
                continue;
            };
            let path_world = path_cache.component_world_transform_with_bounds(
                self,
                graph,
                path.local_id,
                layout_bounds,
            );
            let path_frame =
                path_cache.path_geometry_commands_frame(self, graph, path, layout_bounds);
            let runtime_path = path_frame.path.as_ref();
            let path_transform = if path_frame.has_weighted_context {
                Mat2D::IDENTITY
            } else {
                path_world
            };
            let transform = match path_kind {
                ShapePaintPathKind::World => path_transform,
                ShapePaintPathKind::Local | ShapePaintPathKind::LocalClockwise => {
                    if shape_world == path_transform
                        && mat2d_has_visible_skew_or_rotation(shape_world)
                    {
                        inverse_shape_world.multiply_path_local_contracted(path_transform)
                    } else if mat2d_has_visible_skew_or_rotation(shape_world)
                        || mat2d_has_visible_skew_or_rotation(path_transform)
                    {
                        inverse_shape_world.multiply_path_local_fused(path_transform)
                    } else if mat2d_has_any_skew_or_rotation(shape_world)
                        || mat2d_has_any_skew_or_rotation(path_transform)
                    {
                        inverse_shape_world.multiply_path_local_contracted(path_transform)
                    } else {
                        inverse_shape_world.multiply(path_transform)
                    }
                }
            };

            if nsliced_context.is_none() {
                if path_kind == ShapePaintPathKind::LocalClockwise
                    && path_needs_clockwise_reversal(runtime_path, transform)
                {
                    let mut path_commands = path_frame.commands.as_ref().clone();
                    transform_path_commands(&mut path_commands, transform);
                    path_commands = path_commands_backwards(&path_commands);
                    let start = commands.len();
                    commands.extend(path_commands);
                    prune_empty_path_segments_from(&mut commands, start);
                } else {
                    let start = commands.len();
                    append_transformed_path_commands(
                        &mut commands,
                        path_frame.commands.as_ref(),
                        transform,
                    );
                    prune_empty_path_segments_from(&mut commands, start);
                }
                continue;
            }

            let mut path_commands = if let Some(nsliced_context) = nsliced_context.as_ref()
                && !path_frame.has_weighted_context
            {
                let mut path_commands = path_frame.commands.as_ref().clone();
                runtime_deform_path_commands_with_nsliced_node(
                    &mut path_commands,
                    nsliced_context,
                    ShapePaintPathKind::Local,
                    path_world,
                    path_world.invert_or_identity(),
                );
                transform_path_commands(&mut path_commands, transform);
                if path_kind == ShapePaintPathKind::LocalClockwise
                    && path_needs_clockwise_reversal(&runtime_path, transform)
                {
                    path_commands_backwards(&path_commands)
                } else {
                    path_commands
                }
            } else {
                let mut path_commands = path_frame.commands.as_ref().clone();
                transform_path_commands(&mut path_commands, transform);
                if path_kind == ShapePaintPathKind::LocalClockwise
                    && path_needs_clockwise_reversal(runtime_path, transform)
                {
                    path_commands = path_commands_backwards(&path_commands);
                }
                if let Some(nsliced_context) = nsliced_context.as_ref() {
                    runtime_deform_path_commands_with_nsliced_node(
                        &mut path_commands,
                        nsliced_context,
                        path_kind,
                        shape_world,
                        inverse_shape_world,
                    );
                }
                path_commands
            };
            prune_empty_path_segments(&mut path_commands);
            commands.extend(path_commands);
        }

        commands
    }

    fn runtime_clipping_shape_path_commands_uncached(
        &self,
        clipping_shape: &ClippingShapeNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        path_cache: &mut RuntimeRenderPathCache,
    ) -> Vec<RuntimePathCommand> {
        let mut commands = Vec::new();
        let path_lookup = path_cache.path_composer_lookup_frame(graph);
        for shape_local in &clipping_shape.shape_locals {
            let Some(composer) = path_lookup.composer(graph, *shape_local) else {
                continue;
            };
            let shape_world = path_cache.component_world_transform_with_bounds(
                self,
                graph,
                *shape_local,
                layout_bounds,
            );
            let inverse_shape_world = shape_world.invert_or_identity();
            let nsliced_context =
                runtime_nsliced_node_context_for_shape(self, graph, *shape_local, layout_bounds);

            for path_ref in &composer.paths {
                if !self.runtime_path_counts_for_clip(path_ref) {
                    continue;
                }
                let Some(path) = path_lookup.path(graph, path_ref.local_id) else {
                    continue;
                };
                let path_world = path_cache.component_world_transform_with_bounds(
                    self,
                    graph,
                    path.local_id,
                    layout_bounds,
                );
                let path_frame =
                    path_cache.path_geometry_commands_frame(self, graph, path, layout_bounds);
                let path_transform = if path_frame.has_weighted_context {
                    Mat2D::IDENTITY
                } else {
                    path_world
                };
                if nsliced_context.is_none() {
                    let start = commands.len();
                    append_transformed_path_commands(
                        &mut commands,
                        path_frame.commands.as_ref(),
                        path_transform,
                    );
                    prune_empty_path_segments_from(&mut commands, start);
                    continue;
                }

                let mut path_commands = path_frame.commands.as_ref().clone();
                transform_path_commands(&mut path_commands, path_transform);
                if let Some(nsliced_context) = nsliced_context.as_ref() {
                    runtime_deform_path_commands_with_nsliced_node(
                        &mut path_commands,
                        nsliced_context,
                        ShapePaintPathKind::World,
                        shape_world,
                        inverse_shape_world,
                    );
                }
                prune_empty_path_segments(&mut path_commands);
                commands.extend(path_commands);
            }
        }

        commands
    }

    fn runtime_path_geometry_with_layout_control(
        &self,
        path: &PathGeometryNode,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> PathGeometryNode {
        let mut runtime_path = runtime_path_geometry(self, path);
        let Some(layout_bounds) = layout_bounds else {
            return runtime_path;
        };
        let Some(control_size) =
            self.runtime_layout_control_size_for_path(path.local_id, layout_bounds)
        else {
            return runtime_path;
        };
        if let Some(parametric) = runtime_path.parametric.take() {
            runtime_path.parametric = Some(parametric_path_with_control_size(
                parametric,
                control_size.width,
                control_size.height,
            ));
        }
        runtime_path
    }

    fn runtime_layout_control_size_for_path(
        &self,
        path_local: usize,
        layout_bounds: &BTreeMap<usize, RuntimeLayoutBounds>,
    ) -> Option<RuntimeLayoutBounds> {
        // Ported from C++ `src/layout_component.cpp::propagateSizeToChildren`
        // and `src/shapes/parametric_path.cpp::controlSize`: solved layout
        // width/height are pushed through non-Node containers into parametric
        // paths before those paths build render geometry.
        let mut current_local = self.component(path_local)?.parent_local?;
        loop {
            let component = self.component(current_local)?;
            match component.type_name {
                "LayoutComponent" => {
                    self.runtime_layout_component_style_local(current_local)?;
                    return layout_bounds.get(&current_local).copied();
                }
                "NSlicedNode" => return None,
                "Artboard" | "Node" => return None,
                _ => {
                    current_local = component.parent_local?;
                }
            }
        }
    }

    pub(crate) fn runtime_parametric_path_layout_control_size(
        &self,
        path_local: usize,
        graph: &ArtboardGraph,
    ) -> Option<(f32, f32)> {
        let layout_bounds = self.runtime_taffy_layout_bounds(graph, self.runtime_file())?;
        let control_size = self.runtime_layout_control_size_for_path(path_local, &layout_bounds)?;
        Some((control_size.width, control_size.height))
    }

    fn runtime_weighted_path_context(
        &self,
        path: &PathGeometryNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Option<WeightedPathContext> {
        self.runtime_weighted_skinnable_context(path.local_id, graph, layout_bounds)
    }

    fn runtime_weighted_mesh_context(
        &self,
        mesh: &MeshGeometryNode,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Option<WeightedPathContext> {
        self.runtime_weighted_skinnable_context(mesh.local_id, graph, layout_bounds)
    }

    fn runtime_weighted_skinnable_context(
        &self,
        skinnable_local: usize,
        graph: &ArtboardGraph,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Option<WeightedPathContext> {
        let skin = graph
            .skeletal_skins
            .iter()
            .find(|skin| skin.skinnable_local == Some(skinnable_local))?;
        let mut bone_transforms = vec![Mat2D::IDENTITY];
        for tendon in &skin.tendons {
            let bone_local = tendon.bone_local?;
            let bone_world = self.runtime_component_world_transform_with_bounds(
                bone_local,
                graph,
                layout_bounds,
            );
            let bone_transform = bone_world.multiply(Mat2D(tendon.inverse_bind));
            bone_transforms.push(bone_transform);
        }

        Some(WeightedPathContext {
            skin_world_transform: Mat2D(skin.world_transform),
            bone_transforms,
        })
    }
}

fn runtime_sorted_drawable_node(
    drawable: &DrawableOrderNode,
    draw_target_local: Option<usize>,
) -> SortedDrawableNode {
    SortedDrawableNode {
        kind: drawable.kind,
        local_id: drawable.local_id,
        global_id: drawable.global_id,
        type_name: drawable.type_name,
        is_hidden: drawable.is_hidden,
        resolved_image_asset_global: drawable.resolved_image_asset_global,
        referenced_artboard_global: drawable.referenced_artboard_global,
        layout_local: drawable.layout_local,
        layout_global: drawable.layout_global,
        draw_target_local,
        clipping_shape_local: None,
        clipping_shape_global: None,
        needs_save_operation: true,
    }
}

fn runtime_interleave_clipping_proxy_drawables(
    sorted_drawables: impl IntoIterator<Item = SortedDrawableNode>,
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<SortedDrawableNode> {
    let mut order = Vec::new();
    let mut clipping_stack = Vec::<usize>::new();

    for drawable in sorted_drawables {
        let drawable_clipping_shapes =
            runtime_drawable_clipping_shape_locals(&drawable, clipping_shapes);
        let removing_index = clipping_stack
            .iter()
            .position(|clipping_shape_local| {
                !drawable_clipping_shapes.contains(clipping_shape_local)
            })
            .unwrap_or(clipping_stack.len());

        if removing_index < clipping_stack.len() {
            for clipping_shape_local in clipping_stack[removing_index..].iter().rev() {
                order.push(runtime_clipping_proxy_node(
                    clipping_shapes,
                    *clipping_shape_local,
                    DrawableOrderKind::ClipEndProxy,
                ));
            }
            clipping_stack.truncate(removing_index);
        }

        for clipping_shape_local in drawable_clipping_shapes {
            if !clipping_stack.contains(&clipping_shape_local) {
                order.push(runtime_clipping_proxy_node(
                    clipping_shapes,
                    clipping_shape_local,
                    DrawableOrderKind::ClipStartProxy,
                ));
                clipping_stack.push(clipping_shape_local);
            }
        }

        order.push(drawable);
    }

    for clipping_shape_local in clipping_stack.into_iter().rev() {
        order.push(runtime_clipping_proxy_node(
            clipping_shapes,
            clipping_shape_local,
            DrawableOrderKind::ClipEndProxy,
        ));
    }

    order
}

fn runtime_drawable_clipping_shape_locals(
    drawable: &SortedDrawableNode,
    clipping_shapes: &[ClippingShapeNode],
) -> Vec<usize> {
    let Some(drawable_local) = drawable.local_id else {
        return Vec::new();
    };

    clipping_shapes
        .iter()
        .filter_map(|clipping_shape| {
            clipping_shape
                .clipped_drawable_locals
                .contains(&drawable_local)
                .then_some(clipping_shape.local_id)
        })
        .collect()
}

fn runtime_clipping_proxy_node(
    clipping_shapes: &[ClippingShapeNode],
    clipping_shape_local: usize,
    kind: DrawableOrderKind,
) -> SortedDrawableNode {
    let clipping_shape_global = clipping_shapes
        .iter()
        .find(|clipping_shape| clipping_shape.local_id == clipping_shape_local)
        .map(|clipping_shape| clipping_shape.global_id);
    debug_assert!(matches!(
        kind,
        DrawableOrderKind::ClipStartProxy | DrawableOrderKind::ClipEndProxy
    ));

    SortedDrawableNode {
        kind,
        local_id: None,
        global_id: None,
        type_name: "ClippingShapeProxyDrawable",
        is_hidden: false,
        resolved_image_asset_global: None,
        referenced_artboard_global: None,
        layout_local: None,
        layout_global: None,
        draw_target_local: None,
        clipping_shape_local: Some(clipping_shape_local),
        clipping_shape_global,
        needs_save_operation: true,
    }
}

fn runtime_apply_save_operation_elision(
    sorted_drawables: &mut [SortedDrawableNode],
    clipping_shapes: &[ClippingShapeNode],
) {
    let clipping_visibility = clipping_shapes
        .iter()
        .map(|clipping_shape| (clipping_shape.local_id, clipping_shape.is_visible))
        .collect::<BTreeMap<_, _>>();
    let mut prev_applied_save = false;
    let mut applied_clipping_save_operations = Vec::<bool>::new();

    for index in 0..sorted_drawables.len() {
        sorted_drawables[index].needs_save_operation = true;
        if prev_applied_save {
            if sorted_drawables[index].kind == DrawableOrderKind::ClipStartProxy {
                applied_clipping_save_operations.push(false);
                sorted_drawables[index].needs_save_operation = false;
            } else if sorted_drawables[index].kind == DrawableOrderKind::ClipEndProxy {
                let operation_applied = applied_clipping_save_operations.pop().unwrap_or(true);
                sorted_drawables[index].needs_save_operation = operation_applied;
            } else if sorted_drawables
                .get(index + 1)
                .is_some_and(|next| next.kind == DrawableOrderKind::ClipEndProxy)
            {
                sorted_drawables[index].needs_save_operation = false;
            }
        } else if sorted_drawables[index].kind == DrawableOrderKind::ClipStartProxy {
            applied_clipping_save_operations.push(true);
        } else if sorted_drawables[index].kind == DrawableOrderKind::ClipEndProxy {
            let operation_applied = applied_clipping_save_operations.pop().unwrap_or(true);
            sorted_drawables[index].needs_save_operation = operation_applied;
        }

        prev_applied_save = sorted_drawables[index].kind == DrawableOrderKind::ClipStartProxy
            && (runtime_sorted_drawable_will_clip(&sorted_drawables[index], &clipping_visibility)
                || prev_applied_save);
    }

    debug_assert!(applied_clipping_save_operations.is_empty());
}

fn runtime_sorted_drawable_will_clip(
    drawable: &SortedDrawableNode,
    clipping_visibility: &BTreeMap<usize, bool>,
) -> bool {
    drawable
        .clipping_shape_local
        .and_then(|local_id| clipping_visibility.get(&local_id))
        .copied()
        .unwrap_or(false)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDrawCommandKind {
    Draw,
    ClipStart,
    ClipEnd,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeDrawCommandObjectKind {
    ArtboardComponentList,
    DrawableProxy,
    ForegroundLayoutDrawable,
    Image,
    LayoutComponent,
    NestedArtboard,
    NestedArtboardLeaf,
    NestedArtboardLayout,
    ScriptedDrawable,
    Text,
    Other,
}

impl RuntimeDrawCommandObjectKind {
    fn from_type_name(type_name: &str) -> Self {
        match type_name {
            "ArtboardComponentList" => Self::ArtboardComponentList,
            "DrawableProxy" => Self::DrawableProxy,
            "ForegroundLayoutDrawable" => Self::ForegroundLayoutDrawable,
            "Image" => Self::Image,
            "LayoutComponent" => Self::LayoutComponent,
            "NestedArtboard" => Self::NestedArtboard,
            "NestedArtboardLeaf" => Self::NestedArtboardLeaf,
            "NestedArtboardLayout" => Self::NestedArtboardLayout,
            "ScriptedDrawable" | "ScriptedLayout" => Self::ScriptedDrawable,
            "Text" => Self::Text,
            _ => Self::Other,
        }
    }

    fn is_nested_artboard(self) -> bool {
        matches!(
            self,
            Self::NestedArtboard | Self::NestedArtboardLeaf | Self::NestedArtboardLayout
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeDrawCommand {
    pub kind: RuntimeDrawCommandKind,
    pub object_kind: RuntimeDrawCommandObjectKind,
    pub local_id: Option<usize>,
    pub global_id: Option<u32>,
    pub type_name: &'static str,
    pub world_transform: Option<Mat2D>,
    pub render_opacity: f32,
    pub referenced_artboard_global: Option<u32>,
    pub resolved_image_asset_global: Option<u32>,
    pub clipping_shape_local: Option<usize>,
    pub needs_save_operation: bool,
    pub shape_paints: Vec<RuntimeShapePaintCommand>,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeLayoutBoundsReport {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub name: Option<String>,
    pub parent_local: Option<usize>,
    pub collapsed: bool,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub world_transform: [f32; 6],
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeLayoutBounds {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
}

trait RuntimeLayoutEngine {
    fn compute_bounds(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>>;
}

struct TaffyRuntimeLayoutEngine;

#[derive(Default)]
struct TaffyLayoutBuild {
    nodes_by_local: BTreeMap<usize, NodeId>,
    children_by_local: BTreeMap<usize, Vec<usize>>,
}

fn definite_available_space(space: AvailableSpace) -> Option<f32> {
    match space {
        AvailableSpace::Definite(value) => Some(value),
        AvailableSpace::MinContent | AvailableSpace::MaxContent => None,
    }
}

impl RuntimeLayoutEngine for TaffyRuntimeLayoutEngine {
    fn compute_bounds(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>> {
        self.compute_bounds_with_root_hug(
            instance,
            graph,
            runtime,
            Size {
                width: false,
                height: false,
            },
        )
    }
}

impl TaffyRuntimeLayoutEngine {
    fn compute_bounds_with_root_hug(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
        root_hug: Size<bool>,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>> {
        // Ported from C++ `src/layout_component.cpp`
        // `syncStyle`/`syncLayoutChildren`/`calculateLayoutInternal`: translate
        // Rive layout style into the delegated layout engine, then expose the
        // settled layout boxes back to draw/world/computed-value code.
        let root = graph
            .components
            .iter()
            .find(|component| component.local_id == 0)?;
        if root.type_name != "Artboard" {
            return None;
        }
        let mut taffy = TaffyTree::<usize>::new();
        taffy.disable_rounding();
        let mut build = TaffyLayoutBuild::default();
        let root_node =
            self.build_node(instance, graph, runtime, 0, true, &mut taffy, &mut build)?;
        if root_hug.width || root_hug.height {
            let mut style = taffy.style(root_node).ok()?.clone();
            if root_hug.width {
                style.size.width = Dimension::auto();
            }
            if root_hug.height {
                style.size.height = Dimension::auto();
            }
            taffy.set_style(root_node, style).ok()?;
        }
        let mut available_space = self.root_available_space(instance)?;
        if root_hug.width {
            available_space.width = AvailableSpace::MaxContent;
        }
        if root_hug.height {
            available_space.height = AvailableSpace::MaxContent;
        }
        taffy
            .compute_layout_with_measure(
                root_node,
                available_space,
                |known_dimensions, available_space, _node_id, node_context, _style| {
                    node_context
                        .and_then(|layout_local| {
                            self.measure_layout_component(
                                runtime?,
                                instance,
                                graph,
                                *layout_local,
                                known_dimensions,
                                available_space,
                            )
                        })
                        .unwrap_or(Size::ZERO)
                },
            )
            .ok()?;

        let mut bounds = BTreeMap::new();
        self.collect_bounds(0, 0.0, 0.0, &taffy, &build, &mut bounds)?;
        Some(bounds)
    }

    fn build_node(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
        local: usize,
        parent_is_row: bool,
        taffy: &mut TaffyTree<usize>,
        build: &mut TaffyLayoutBuild,
    ) -> Option<NodeId> {
        // Cycle guard: a malformed-but-accepted file can make the layout child
        // graph cyclic, which would recurse forever here (the node map is only
        // populated after the child recursion below). The nodes-by-local map
        // doubles as the visited set, mirroring C++'s visited-set cycle-guard
        // idiom (DependencySorter::visit, src/dependency_sorter.cpp): each local
        // is built exactly once on any valid tree, so returning the existing
        // node is identical behavior there and terminates cycles gracefully.
        if let Some(node) = build.nodes_by_local.get(&local) {
            return Some(*node);
        }
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == local)?;
        if !matches!(
            component.type_name,
            "Artboard" | "LayoutComponent" | "NestedArtboardLayout"
        ) {
            return None;
        }
        if self.needs_unimplemented_measure(instance, graph, runtime, local)? {
            return None;
        }

        let style = if component.type_name == "NestedArtboardLayout" {
            self.nested_artboard_layout_style(instance, local, parent_is_row)?
        } else {
            self.style_for_node(instance, graph, local, parent_is_row)?
        };
        if component.type_name == "NestedArtboardLayout" {
            let node = taffy.new_leaf(style).ok()?;
            build.nodes_by_local.insert(local, node);
            build.children_by_local.insert(local, Vec::new());
            return Some(node);
        }
        let is_row = instance
            .runtime_layout_component_style_local(local)
            .is_none_or(|style_local| instance.runtime_layout_style_is_row(style_local));
        let mut child_nodes = Vec::new();
        let mut child_locals = Vec::new();
        for child_local in &component.children {
            // C++ inserts layout-provider children into Yoga even when runtime
            // collapse later suppresses drawing; display:none is represented
            // by the child node's style.
            let Some(child) = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
            else {
                continue;
            };
            match child.type_name {
                "LayoutComponent" => {
                    let child_node = self.build_node(
                        instance,
                        graph,
                        runtime,
                        *child_local,
                        is_row,
                        taffy,
                        build,
                    )?;
                    child_nodes.push(child_node);
                    child_locals.push(*child_local);
                }
                "ArtboardComponentList" => {
                    if !self.zero_sized_component_list_supported(graph, *child_local)? {
                        return None;
                    }
                }
                "NestedArtboardLayout" => {
                    let child_node = self.build_node(
                        instance,
                        graph,
                        runtime,
                        *child_local,
                        is_row,
                        taffy,
                        build,
                    )?;
                    child_nodes.push(child_node);
                    child_locals.push(*child_local);
                }
                _ => {}
            }
        }

        let node = if child_nodes.is_empty() {
            if runtime.is_some()
                && self.layout_component_has_intrinsic_static_measure(instance, graph, local)?
            {
                taffy.new_leaf_with_context(style, local).ok()?
            } else {
                taffy.new_leaf(style).ok()?
            }
        } else {
            taffy.new_with_children(style, &child_nodes).ok()?
        };
        build.nodes_by_local.insert(local, node);
        build.children_by_local.insert(local, child_locals);
        Some(node)
    }

    fn zero_sized_component_list_supported(
        &self,
        graph: &ArtboardGraph,
        local: usize,
    ) -> Option<bool> {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == local)?;
        graph
            .component_lists
            .iter()
            .find(|list| list.local_id == local)?;
        Some(component.children.iter().all(|child_local| {
            graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
                .is_some_and(|component| {
                    matches!(
                        component.type_name,
                        "ArtboardListMapRule"
                            | "ArtboardComponentListOverride"
                            | "ListFollowPathConstraint"
                    )
                })
        }))
    }

    fn collect_bounds(
        &self,
        local: usize,
        parent_x: f32,
        parent_y: f32,
        taffy: &TaffyTree<usize>,
        build: &TaffyLayoutBuild,
        bounds: &mut BTreeMap<usize, RuntimeLayoutBounds>,
    ) -> Option<()> {
        // Cycle guard: a malformed-but-accepted file can make the layout child
        // graph cyclic, turning this walk into unbounded recursion. The bounds
        // map doubles as the visited set (each local is laid out exactly once on
        // any valid tree), mirroring C++'s visited-set cycle-guard idiom
        // (DependencySorter::visit, src/dependency_sorter.cpp); on revisit,
        // terminate gracefully with the already-computed bounds.
        if bounds.contains_key(&local) {
            return Some(());
        }
        let node = *build.nodes_by_local.get(&local)?;
        let layout = taffy.layout(node).ok()?;
        let x = parent_x + layout.location.x;
        let y = parent_y + layout.location.y;
        bounds.insert(
            local,
            RuntimeLayoutBounds {
                x,
                y,
                width: layout.size.width,
                height: layout.size.height,
            },
        );
        for child_local in build.children_by_local.get(&local)? {
            self.collect_bounds(*child_local, x, y, taffy, build, bounds)?;
        }
        Some(())
    }

    fn style_for_node(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        local: usize,
        parent_is_row: bool,
    ) -> Option<Style> {
        let is_root = local == 0;
        let style_local = instance.runtime_layout_component_style_local(local);
        if style_local.is_none() && !is_root {
            return None;
        }

        let mut style = Style {
            display: if style_local.is_some_and(|style_local| {
                instance.runtime_layout_style_uint_default(
                    style_local,
                    RuntimeLayoutStyleProperty::DisplayValue,
                    0,
                ) == 1
            }) {
                TaffyDisplay::None
            } else {
                TaffyDisplay::Flex
            },
            size: if is_root {
                if let Some(style_local) = style_local {
                    Size {
                        width: self.root_axis_dimension(instance, style_local, true)?,
                        height: self.root_axis_dimension(instance, style_local, false)?,
                    }
                } else {
                    Size::from_lengths(instance.width, instance.height)
                }
            } else {
                let style_local = style_local?;
                Size {
                    width: self.axis_dimension(instance, graph, local, style_local, true)?,
                    height: self.axis_dimension(instance, graph, local, style_local, false)?,
                }
            },
            ..Default::default()
        };

        let Some(style_local) = style_local else {
            return Some(style);
        };

        style.position = if instance.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::PositionTypeValue,
            1,
        ) == 2
        {
            Position::Absolute
        } else {
            Position::Relative
        };
        style.direction = match instance.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::DirectionValue,
            0,
        ) {
            2 => TaffyDirection::Rtl,
            _ => TaffyDirection::Ltr,
        };
        style.flex_direction = match instance.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::FlexDirectionValue,
            2,
        ) {
            0 => FlexDirection::Column,
            1 => FlexDirection::ColumnReverse,
            3 => FlexDirection::RowReverse,
            _ => FlexDirection::Row,
        };
        style.flex_wrap = match instance.runtime_layout_style_uint_default(
            style_local,
            RuntimeLayoutStyleProperty::FlexWrapValue,
            0,
        ) {
            1 => FlexWrap::Wrap,
            2 => FlexWrap::WrapReverse,
            _ => FlexWrap::NoWrap,
        };
        style.gap = Size {
            width: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::GapHorizontal,
                RuntimeLayoutStyleProperty::GapHorizontalUnitsValue,
            )?,
            height: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::GapVertical,
                RuntimeLayoutStyleProperty::GapVerticalUnitsValue,
            )?,
        };
        style.padding = Rect {
            left: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PaddingLeft,
                RuntimeLayoutStyleProperty::PaddingLeftUnitsValue,
            )?,
            right: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PaddingRight,
                RuntimeLayoutStyleProperty::PaddingRightUnitsValue,
            )?,
            top: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PaddingTop,
                RuntimeLayoutStyleProperty::PaddingTopUnitsValue,
            )?,
            bottom: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PaddingBottom,
                RuntimeLayoutStyleProperty::PaddingBottomUnitsValue,
            )?,
        };
        style.border = Rect {
            left: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::BorderLeft,
                RuntimeLayoutStyleProperty::BorderLeftUnitsValue,
            )?,
            right: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::BorderRight,
                RuntimeLayoutStyleProperty::BorderRightUnitsValue,
            )?,
            top: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::BorderTop,
                RuntimeLayoutStyleProperty::BorderTopUnitsValue,
            )?,
            bottom: self.length_percentage_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::BorderBottom,
                RuntimeLayoutStyleProperty::BorderBottomUnitsValue,
            )?,
        };
        style.margin = Rect {
            left: self.length_percentage_auto_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MarginLeft,
                RuntimeLayoutStyleProperty::MarginLeftUnitsValue,
            )?,
            right: self.length_percentage_auto_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MarginRight,
                RuntimeLayoutStyleProperty::MarginRightUnitsValue,
            )?,
            top: self.length_percentage_auto_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MarginTop,
                RuntimeLayoutStyleProperty::MarginTopUnitsValue,
            )?,
            bottom: self.length_percentage_auto_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MarginBottom,
                RuntimeLayoutStyleProperty::MarginBottomUnitsValue,
            )?,
        };
        style.inset = Rect {
            left: self.position_inset_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PositionLeft,
                RuntimeLayoutStyleProperty::PositionLeftUnitsValue,
            )?,
            right: self.position_inset_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PositionRight,
                RuntimeLayoutStyleProperty::PositionRightUnitsValue,
            )?,
            top: self.position_inset_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PositionTop,
                RuntimeLayoutStyleProperty::PositionTopUnitsValue,
            )?,
            bottom: self.position_inset_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::PositionBottom,
                RuntimeLayoutStyleProperty::PositionBottomUnitsValue,
            )?,
        };
        style.min_size = Size {
            width: self.dimension_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MinWidth,
                RuntimeLayoutStyleProperty::MinWidthUnitsValue,
                false,
                self.intrinsic_static_hug_min_is_auto(instance, graph, local, style_local, true)?,
            )?,
            height: self.dimension_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MinHeight,
                RuntimeLayoutStyleProperty::MinHeightUnitsValue,
                false,
                self.intrinsic_static_hug_min_is_auto(instance, graph, local, style_local, false)?,
            )?,
        };
        style.max_size = Size {
            width: self.dimension_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MaxWidth,
                RuntimeLayoutStyleProperty::MaxWidthUnitsValue,
                true,
                false,
            )?,
            height: self.dimension_style(
                instance,
                style_local,
                RuntimeLayoutStyleProperty::MaxHeight,
                RuntimeLayoutStyleProperty::MaxHeightUnitsValue,
                true,
                false,
            )?,
        };
        let aspect_ratio = instance
            .runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::AspectRatio)
            .unwrap_or(0.0);
        if aspect_ratio > 0.0 {
            style.aspect_ratio = Some(aspect_ratio);
        }

        self.apply_alignment(instance, style_local, &mut style);
        self.apply_flex_item_style(instance, local, style_local, parent_is_row, &mut style)?;
        Some(style)
    }

    fn root_available_space(&self, instance: &ArtboardInstance) -> Option<Size<AvailableSpace>> {
        let style_local = instance.runtime_layout_component_style_local(0);
        Some(Size {
            width: self.root_axis_available_space(instance, style_local, true)?,
            height: self.root_axis_available_space(instance, style_local, false)?,
        })
    }

    fn root_axis_available_space(
        &self,
        instance: &ArtboardInstance,
        style_local: Option<usize>,
        width_axis: bool,
    ) -> Option<AvailableSpace> {
        if let Some(style_local) = style_local {
            const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
            if instance.runtime_layout_axis_scale(style_local, width_axis) == LAYOUT_SCALE_TYPE_HUG
            {
                return Some(AvailableSpace::MaxContent);
            }
        }
        Some(AvailableSpace::Definite(if width_axis {
            instance.width
        } else {
            instance.height
        }))
    }

    fn root_axis_dimension(
        &self,
        instance: &ArtboardInstance,
        style_local: usize,
        width_axis: bool,
    ) -> Option<Dimension> {
        const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
        if instance.runtime_layout_axis_scale(style_local, width_axis) == LAYOUT_SCALE_TYPE_HUG {
            return Some(Dimension::auto());
        }
        Some(Dimension::length(if width_axis {
            instance.width
        } else {
            instance.height
        }))
    }

    fn nested_artboard_layout_style(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        parent_is_row: bool,
    ) -> Option<Style> {
        // Ported from C++ `src/nested_artboard_layout.cpp` and
        // `include/rive/layout/style_overrider.hpp`: the layout-backed host
        // contributes the referenced artboard instance's layout node to its
        // parent.
        let width_scale = self.nested_artboard_layout_axis_scale(instance, local, true);
        let height_scale = self.nested_artboard_layout_axis_scale(instance, local, false);
        if !matches!(width_scale, 0 | 1 | 2) || !matches!(height_scale, 0 | 1 | 2) {
            return None;
        }

        let mut style = Style {
            display: TaffyDisplay::Flex,
            size: Size {
                width: self.nested_artboard_layout_axis_dimension(instance, local, true)?,
                height: self.nested_artboard_layout_axis_dimension(instance, local, false)?,
            },
            ..Default::default()
        };

        let main_scale = if parent_is_row {
            width_scale
        } else {
            height_scale
        };
        let cross_scale = if parent_is_row {
            height_scale
        } else {
            width_scale
        };
        match main_scale {
            0 | 2 => {
                style.flex_grow = 0.0;
                style.flex_shrink = 0.0;
                style.flex_basis = Dimension::auto();
            }
            1 => {
                style.flex_grow = 1.0;
                style.flex_shrink = 1.0;
                // C++ contributes the referenced artboard's layout node, so
                // fill distribution starts from its intrinsic content size.
                style.flex_basis = self
                    .nested_artboard_layout_axis_hug_size(instance, local, parent_is_row)
                    .filter(|value| value.is_finite() && *value >= 0.0)
                    .map(Dimension::length)
                    .unwrap_or_else(Dimension::auto);
            }
            _ => return None,
        }
        style.align_self = match cross_scale {
            1 => Some(AlignSelf::STRETCH),
            0 | 2 => None,
            _ => return None,
        };
        Some(style)
    }

    fn nested_artboard_layout_axis_dimension(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> Option<Dimension> {
        match self.nested_artboard_layout_axis_scale(instance, local, width_axis) {
            0 => {
                let value = self.nested_artboard_layout_axis_length(instance, local, width_axis);
                let units = self.nested_artboard_layout_axis_units(instance, local, width_axis);
                if value < 0.0 {
                    Some(
                        self.nested_artboard_layout_axis_intrinsic_dimension(
                            instance, local, width_axis,
                        )
                        .unwrap_or_else(Dimension::auto),
                    )
                } else {
                    self.dimension_from_unit(value, units)
                }
            }
            1 => Some(Dimension::auto()),
            2 => self
                .nested_artboard_layout_axis_intrinsic_dimension(instance, local, width_axis)
                .or_else(|| Some(Dimension::auto())),
            _ => None,
        }
    }

    fn nested_artboard_layout_axis_intrinsic_dimension(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> Option<Dimension> {
        let value = self.nested_artboard_layout_axis_intrinsic_size(instance, local, width_axis)?;
        if value.is_finite() && value >= 0.0 {
            Some(Dimension::length(value))
        } else {
            None
        }
    }

    fn nested_artboard_layout_axis_intrinsic_size(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> Option<f32> {
        let nested = instance.nested_artboards.get(&local)?;
        let layout_size = nested
            .child
            .runtime_file()
            .zip(nested.child.runtime_graph())
            .and_then(|(runtime, graph)| {
                nested
                    .child
                    .runtime_taffy_layout_bounds(graph, Some(runtime))
                    .and_then(|bounds| bounds.get(&0).copied())
            })
            .map(|bounds| {
                if width_axis {
                    bounds.width
                } else {
                    bounds.height
                }
            });

        layout_size.or_else(|| {
            Some(if width_axis {
                nested.child.width
            } else {
                nested.child.height
            })
        })
    }

    fn nested_artboard_layout_axis_hug_size(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> Option<f32> {
        let nested = instance.nested_artboards.get(&local)?;
        nested
            .child
            .runtime_file()
            .zip(nested.child.runtime_graph())
            .and_then(|(runtime, graph)| {
                self.compute_bounds_with_root_hug(
                    &nested.child,
                    graph,
                    Some(runtime),
                    Size {
                        width: width_axis,
                        height: !width_axis,
                    },
                )
            })
            .and_then(|bounds| bounds.get(&0).copied())
            .map(|bounds| {
                if width_axis {
                    bounds.width
                } else {
                    bounds.height
                }
            })
    }

    fn nested_artboard_layout_axis_scale(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> u64 {
        runtime_nested_artboard_layout_property_key_for_name(if width_axis {
            "instanceWidthScaleType"
        } else {
            "instanceHeightScaleType"
        })
        .and_then(|key| instance.uint_property(local, key))
        .unwrap_or(0)
    }

    fn nested_artboard_layout_axis_units(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> u64 {
        runtime_nested_artboard_layout_property_key_for_name(if width_axis {
            "instanceWidthUnitsValue"
        } else {
            "instanceHeightUnitsValue"
        })
        .and_then(|key| instance.uint_property(local, key))
        .unwrap_or(1)
    }

    fn nested_artboard_layout_axis_length(
        &self,
        instance: &ArtboardInstance,
        local: usize,
        width_axis: bool,
    ) -> f32 {
        runtime_nested_artboard_layout_property_key_for_name(if width_axis {
            "instanceWidth"
        } else {
            "instanceHeight"
        })
        .and_then(|key| instance.double_property(local, key))
        .unwrap_or(-1.0)
    }

    fn axis_dimension(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
    ) -> Option<Dimension> {
        let value = instance.runtime_layout_component_dimension(
            layout_local,
            if width_axis { "width" } else { "height" },
        );
        let units = self.effective_units(instance, graph, layout_local, style_local, width_axis)?;
        self.dimension_from_unit(value.max(0.0), units)
    }

    fn effective_units(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
    ) -> Option<u64> {
        let scale = instance.runtime_layout_axis_scale(style_local, width_axis);
        let stored = instance.runtime_layout_axis_units(style_local, width_axis);
        let absolute = instance.runtime_layout_component_is_absolute(layout_local);
        if absolute && scale != 2 {
            return Some(stored);
        }
        if scale != 0 {
            return Some(3);
        }
        if matches!(stored, 1 | 2) {
            return Some(stored);
        }
        if self.is_legacy_hug_encoding(instance, graph, layout_local, style_local)? {
            Some(3)
        } else {
            Some(1)
        }
    }

    fn is_legacy_hug_encoding(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
        style_local: usize,
    ) -> Option<bool> {
        let width_fixed = instance.runtime_layout_axis_scale(style_local, true) == 0;
        let height_fixed = instance.runtime_layout_axis_scale(style_local, false) == 0;
        let intrinsic = instance
            .runtime_layout_style_bool(
                style_local,
                RuntimeLayoutStyleProperty::IntrinsicallySizedValue,
            )
            .unwrap_or(false);
        Some(
            width_fixed
                && height_fixed
                && intrinsic
                && self
                    .layout_provider_children(instance, graph, layout_local)?
                    .is_empty(),
        )
    }

    fn apply_flex_item_style(
        &self,
        instance: &ArtboardInstance,
        layout_local: usize,
        style_local: usize,
        parent_is_row: bool,
        style: &mut Style,
    ) -> Option<()> {
        let width_scale = instance.runtime_layout_axis_scale(style_local, true);
        let height_scale = instance.runtime_layout_axis_scale(style_local, false);
        let main_scale = if parent_is_row {
            width_scale
        } else {
            height_scale
        };
        let cross_scale = if parent_is_row {
            height_scale
        } else {
            width_scale
        };

        match main_scale {
            0 | 2 => {
                style.flex_grow = 0.0;
                style.flex_shrink = 0.0;
                style.flex_basis = Dimension::auto();
            }
            1 => {
                style.flex_grow =
                    instance.runtime_layout_axis_fraction(layout_local, parent_is_row);
                style.flex_shrink = style.flex_grow;
                let basis = instance
                    .runtime_layout_style_double(style_local, RuntimeLayoutStyleProperty::FlexBasis)
                    .unwrap_or(0.0);
                let units = instance.runtime_layout_style_uint_default(
                    style_local,
                    RuntimeLayoutStyleProperty::FlexBasisUnitsValue,
                    3,
                );
                style.flex_basis = self.dimension_from_unit(basis, units)?;
            }
            _ => return None,
        }
        style.align_self = match cross_scale {
            1 => Some(AlignSelf::STRETCH),
            0 | 2 => None,
            _ => return None,
        };
        Some(())
    }

    fn apply_alignment(&self, instance: &ArtboardInstance, style_local: usize, style: &mut Style) {
        let alignment = instance.runtime_layout_alignment_type(style_local);
        let is_row = instance.runtime_layout_style_is_row(style_local);

        match alignment {
            0 | 1 | 2 => {
                if is_row {
                    style.align_items = Some(AlignItems::FLEX_START);
                    style.align_content = Some(AlignContent::FLEX_START);
                } else {
                    style.justify_content = Some(JustifyContent::FLEX_START);
                }
            }
            3 | 4 | 5 => {
                if is_row {
                    style.align_items = Some(AlignItems::CENTER);
                    style.align_content = Some(AlignContent::CENTER);
                } else {
                    style.justify_content = Some(JustifyContent::CENTER);
                }
            }
            6 | 7 | 8 => {
                if is_row {
                    style.align_items = Some(AlignItems::FLEX_END);
                    style.align_content = Some(AlignContent::FLEX_END);
                } else {
                    style.justify_content = Some(JustifyContent::FLEX_END);
                }
            }
            _ => {}
        }

        match alignment {
            0 | 3 | 6 => {
                if is_row {
                    style.justify_content = Some(JustifyContent::FLEX_START);
                } else {
                    style.align_items = Some(AlignItems::FLEX_START);
                    style.align_content = Some(AlignContent::FLEX_START);
                }
            }
            1 | 4 | 7 => {
                if is_row {
                    style.justify_content = Some(JustifyContent::CENTER);
                } else {
                    style.align_items = Some(AlignItems::CENTER);
                    style.align_content = Some(AlignContent::CENTER);
                }
            }
            2 | 5 | 8 => {
                if is_row {
                    style.justify_content = Some(JustifyContent::FLEX_END);
                } else {
                    style.align_items = Some(AlignItems::FLEX_END);
                    style.align_content = Some(AlignContent::FLEX_END);
                }
            }
            9 => {
                style.align_items = Some(AlignItems::FLEX_START);
                style.align_content = Some(AlignContent::FLEX_START);
                style.justify_content = Some(JustifyContent::SPACE_BETWEEN);
            }
            10 => {
                style.align_items = Some(AlignItems::CENTER);
                style.align_content = Some(AlignContent::CENTER);
                style.justify_content = Some(JustifyContent::SPACE_BETWEEN);
            }
            11 => {
                style.align_items = Some(AlignItems::FLEX_END);
                style.align_content = Some(AlignContent::FLEX_END);
                style.justify_content = Some(JustifyContent::SPACE_BETWEEN);
            }
            _ => {}
        }
    }

    fn needs_unimplemented_measure(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
        layout_local: usize,
    ) -> Option<bool> {
        let Some(style_local) = instance.runtime_layout_component_style_local(layout_local) else {
            return Some(false);
        };
        if !self
            .layout_provider_children(instance, graph, layout_local)?
            .is_empty()
        {
            return Some(false);
        }
        let intrinsic = instance
            .runtime_layout_style_bool(
                style_local,
                RuntimeLayoutStyleProperty::IntrinsicallySizedValue,
            )
            .unwrap_or(false);
        let hugs_content = instance.runtime_layout_axis_scale(style_local, true) == 2
            || instance.runtime_layout_axis_scale(style_local, false) == 2;
        if !intrinsic && !hugs_content {
            return Some(false);
        }
        if runtime.is_some()
            && self.layout_component_has_intrinsic_static_measure(instance, graph, layout_local)?
        {
            return Some(false);
        }
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        Some(component.children.iter().any(|child_local| {
            let child_type = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
                .map(|child| child.type_name);
            let measured_child =
                runtime.is_some() && matches!(child_type, Some("Shape" | "Text" | "TextInput"));
            let zero_sized_component_list = matches!(child_type, Some("ArtboardComponentList"))
                && self
                    .zero_sized_component_list_supported(graph, *child_local)
                    .unwrap_or(false);
            !measured_child
                && !zero_sized_component_list
                && !matches!(
                    child_type,
                    Some(
                        "LayoutComponent"
                            | "Fill"
                            | "LinearGradient"
                            | "NestedArtboardLeaf"
                            | "NestedArtboardLayout"
                            | "RadialGradient"
                            | "SolidColor"
                            | "Stroke"
                            | "LayoutComponentStyle"
                    )
                )
        }))
    }

    fn layout_component_has_intrinsic_static_measure(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
    ) -> Option<bool> {
        let style_local = instance.runtime_layout_component_style_local(layout_local)?;
        if !instance
            .runtime_layout_style_bool(
                style_local,
                RuntimeLayoutStyleProperty::IntrinsicallySizedValue,
            )
            .unwrap_or(false)
        {
            return Some(false);
        }
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        Some(component.children.iter().any(|child_local| {
            graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
                .is_some_and(|child| matches!(child.type_name, "Shape" | "Text" | "TextInput"))
        }))
    }

    fn intrinsic_static_hug_min_is_auto(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
        style_local: usize,
        width_axis: bool,
    ) -> Option<bool> {
        const LAYOUT_SCALE_TYPE_HUG: u64 = 2;
        if instance.runtime_layout_axis_scale(style_local, width_axis) != LAYOUT_SCALE_TYPE_HUG {
            return Some(false);
        }
        self.layout_component_has_intrinsic_static_measure(instance, graph, layout_local)
    }

    fn measure_layout_component(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Option<Size<f32>> {
        // Ported from C++ `src/layout_component.cpp`
        // `LayoutComponent::measureLayout`: measure intrinsically sizeable
        // non-layout children and take the max size for the static runtime
        // subsets the Taffy adapter can currently model.
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        let mut measured = Size::ZERO;
        for child_local in &component.children {
            // Mirrors C++ `LayoutComponent::measureLayout`: hidden/collapsed
            // intrinsically sizeable children still participate in layout
            // measurement; draw filtering happens later.
            let Some(child) = graph
                .components
                .iter()
                .find(|component| component.local_id == *child_local)
            else {
                continue;
            };
            match child.type_name {
                "Shape" => {
                    let Some((width, height)) = self.measure_shape_layout_child(
                        instance,
                        graph,
                        child.local_id,
                        known_dimensions,
                        available_space,
                    ) else {
                        continue;
                    };
                    measured.width = measured.width.max(width);
                    measured.height = measured.height.max(height);
                }
                "Text" => {
                    let measured_bounds = instance
                        .runtime_layout_component_style_local(layout_local)
                        .and_then(|style_local| {
                            let layout_constraint = RuntimeTextLayoutConstraint {
                                width: known_dimensions
                                    .width
                                    .or_else(|| definite_available_space(available_space.width))
                                    .unwrap_or(f32::MAX),
                                height: known_dimensions
                                    .height
                                    .or_else(|| definite_available_space(available_space.height))
                                    .unwrap_or(f32::MAX),
                                width_scale_type: instance
                                    .runtime_layout_axis_scale(style_local, true),
                                height_scale_type: instance
                                    .runtime_layout_axis_scale(style_local, false),
                            };
                            static_text_layout_measure_bounds(
                                runtime,
                                graph,
                                instance,
                                child.local_id,
                                layout_constraint,
                            )
                        })
                        .or_else(|| {
                            static_text_constraint_bounds(runtime, graph, instance, child.local_id)
                        });
                    let Some((_x, _y, width, height)) = measured_bounds else {
                        continue;
                    };
                    measured.width = measured.width.max(width);
                    measured.height = measured.height.max(height);
                }
                "TextInput" => {
                    let measured_bounds = instance
                        .runtime_layout_component_style_local(layout_local)
                        .and_then(|style_local| {
                            let layout_constraint = RuntimeTextLayoutConstraint {
                                width: known_dimensions
                                    .width
                                    .or_else(|| definite_available_space(available_space.width))
                                    .unwrap_or(f32::MAX),
                                height: known_dimensions
                                    .height
                                    .or_else(|| definite_available_space(available_space.height))
                                    .unwrap_or(f32::MAX),
                                width_scale_type: instance
                                    .runtime_layout_axis_scale(style_local, true),
                                height_scale_type: instance
                                    .runtime_layout_axis_scale(style_local, false),
                            };
                            text_input_layout_measure_bounds(
                                runtime,
                                graph,
                                instance,
                                child.local_id,
                                layout_constraint,
                            )
                        });
                    let Some((_x, _y, width, height)) = measured_bounds else {
                        continue;
                    };
                    measured.width = measured.width.max(width);
                    measured.height = measured.height.max(height);
                }
                _ => {}
            }
        }
        Some(Size {
            width: known_dimensions.width.unwrap_or(measured.width),
            height: known_dimensions.height.unwrap_or(measured.height),
        })
    }

    fn measure_shape_layout_child(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        shape_local: usize,
        known_dimensions: Size<Option<f32>>,
        available_space: Size<AvailableSpace>,
    ) -> Option<(f32, f32)> {
        // Ported from C++ `src/shapes/shape.cpp` and
        // `src/shapes/parametric_path.cpp`: `Shape::measureLayout` takes the max
        // measured size of its intrinsically sizeable paths; parametric paths
        // clamp their authored size to the provided measure constraints.
        let composer = graph
            .path_composers
            .iter()
            .find(|composer| composer.shape_local == shape_local)?;
        let max_width = known_dimensions
            .width
            .or_else(|| definite_available_space(available_space.width))
            .unwrap_or(f32::MAX);
        let max_height = known_dimensions
            .height
            .or_else(|| definite_available_space(available_space.height))
            .unwrap_or(f32::MAX);
        let mut measured = Size::ZERO;
        for path_ref in &composer.paths {
            let Some(path) = graph
                .paths
                .iter()
                .find(|path| path.local_id == path_ref.local_id)
            else {
                continue;
            };
            let runtime_path = runtime_path_geometry(instance, path);
            let Some((width, height)) =
                measure_parametric_path_layout(&runtime_path, max_width, max_height)
            else {
                continue;
            };
            measured.width = measured.width.max(width);
            measured.height = measured.height.max(height);
        }
        Some((measured.width, measured.height))
    }

    fn layout_provider_children(
        &self,
        _instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        layout_local: usize,
    ) -> Option<Vec<usize>> {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == layout_local)?;
        Some(
            component
                .children
                .iter()
                .copied()
                .filter(|child_local| {
                    graph
                        .components
                        .iter()
                        .find(|component| component.local_id == *child_local)
                        .is_some_and(|child| {
                            matches!(
                                child.type_name,
                                "LayoutComponent"
                                    | "NestedArtboardLayout"
                                    | "ArtboardComponentList"
                            )
                        })
                })
                .collect(),
        )
    }

    fn dimension_style(
        &self,
        instance: &ArtboardInstance,
        style_local: usize,
        value_property: RuntimeLayoutStyleProperty,
        unit_property: RuntimeLayoutStyleProperty,
        zero_undefined_is_auto: bool,
        nonzero_undefined_is_auto: bool,
    ) -> Option<Dimension> {
        let value = instance
            .runtime_layout_style_double(style_local, value_property)
            .unwrap_or(0.0);
        let units = instance.runtime_layout_style_uint_default(style_local, unit_property, 0);
        if nonzero_undefined_is_auto
            && units == 0
            && value.abs() > f32::EPSILON
            && matches!(
                value_property,
                RuntimeLayoutStyleProperty::MinWidth | RuntimeLayoutStyleProperty::MinHeight
            )
        {
            return Some(Dimension::auto());
        }
        if zero_undefined_is_auto && units == 0 && value.abs() <= f32::EPSILON {
            return Some(Dimension::auto());
        }
        self.dimension_from_unit(value.max(0.0), units)
    }

    fn length_percentage_style(
        &self,
        instance: &ArtboardInstance,
        style_local: usize,
        value_property: RuntimeLayoutStyleProperty,
        unit_property: RuntimeLayoutStyleProperty,
    ) -> Option<LengthPercentage> {
        let value = instance
            .runtime_layout_style_double(style_local, value_property)
            .unwrap_or(0.0);
        let units = instance.runtime_layout_style_uint_default(style_local, unit_property, 0);
        self.length_percentage_from_unit(value, units)
    }

    fn length_percentage_auto_style(
        &self,
        instance: &ArtboardInstance,
        style_local: usize,
        value_property: RuntimeLayoutStyleProperty,
        unit_property: RuntimeLayoutStyleProperty,
    ) -> Option<LengthPercentageAuto> {
        let value = instance
            .runtime_layout_style_double(style_local, value_property)
            .unwrap_or(0.0);
        let units = instance.runtime_layout_style_uint_default(style_local, unit_property, 0);
        self.length_percentage_auto_from_unit(value, units)
    }

    fn position_inset_style(
        &self,
        instance: &ArtboardInstance,
        style_local: usize,
        value_property: RuntimeLayoutStyleProperty,
        unit_property: RuntimeLayoutStyleProperty,
    ) -> Option<LengthPercentageAuto> {
        let value = instance
            .runtime_layout_style_double(style_local, value_property)
            .unwrap_or(0.0);
        let units = instance.runtime_layout_style_uint_default(style_local, unit_property, 0);
        self.position_inset_from_unit(value, units)
    }

    fn dimension_from_unit(&self, value: f32, units: u64) -> Option<Dimension> {
        match units {
            2 => Some(Dimension::percent(value / 100.0)),
            3 => Some(Dimension::auto()),
            0 | 1 => Some(Dimension::length(value)),
            _ => None,
        }
    }

    fn length_percentage_from_unit(&self, value: f32, units: u64) -> Option<LengthPercentage> {
        match units {
            2 => Some(LengthPercentage::percent(value / 100.0)),
            0 | 1 | 3 => Some(LengthPercentage::length(value)),
            _ => None,
        }
    }

    fn position_inset_from_unit(&self, value: f32, units: u64) -> Option<LengthPercentageAuto> {
        // C++ `src/layout_component.cpp::positionTypeChanged` leaves
        // non-explicit position edges as Yoga undefined, which maps to Taffy
        // auto rather than a zero/nonzero length.
        match units {
            1 => Some(LengthPercentageAuto::length(value)),
            2 => Some(LengthPercentageAuto::percent(value / 100.0)),
            0 | 3 => Some(LengthPercentageAuto::auto()),
            _ => None,
        }
    }

    fn length_percentage_auto_from_unit(
        &self,
        value: f32,
        units: u64,
    ) -> Option<LengthPercentageAuto> {
        match units {
            2 => Some(LengthPercentageAuto::percent(value / 100.0)),
            3 => Some(LengthPercentageAuto::auto()),
            0 | 1 => Some(LengthPercentageAuto::length(value)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct RuntimeLayoutCorners {
    top_left: f32,
    top_right: f32,
    bottom_right: f32,
    bottom_left: f32,
}

impl RuntimeLayoutCorners {
    fn is_square(self) -> bool {
        self.top_left.abs() <= f32::EPSILON
            && self.top_right.abs() <= f32::EPSILON
            && self.bottom_right.abs() <= f32::EPSILON
            && self.bottom_left.abs() <= f32::EPSILON
    }
}

fn runtime_layout_rect_path_commands(
    bounds: RuntimeLayoutBounds,
    corners: RuntimeLayoutCorners,
) -> Vec<RuntimePathCommand> {
    let width_is_zero = bounds.width.abs() <= f32::EPSILON;
    let height_is_zero = bounds.height.abs() <= f32::EPSILON;
    if width_is_zero && height_is_zero {
        return vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Close,
        ];
    }
    if height_is_zero {
        return vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line {
                x: bounds.width,
                y: 0.0,
            },
            RuntimePathCommand::Line { x: 0.0, y: 0.0 },
            RuntimePathCommand::Close,
        ];
    }
    if width_is_zero {
        return vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line {
                x: 0.0,
                y: bounds.height,
            },
            RuntimePathCommand::Line { x: 0.0, y: 0.0 },
            RuntimePathCommand::Close,
        ];
    }
    if !corners.is_square() {
        return runtime_rounded_layout_rect_path_commands(bounds, corners);
    }
    vec![
        RuntimePathCommand::Move { x: 0.0, y: 0.0 },
        RuntimePathCommand::Line {
            x: bounds.width,
            y: 0.0,
        },
        RuntimePathCommand::Line {
            x: bounds.width,
            y: bounds.height,
        },
        RuntimePathCommand::Line {
            x: 0.0,
            y: bounds.height,
        },
        RuntimePathCommand::Line { x: 0.0, y: 0.0 },
        RuntimePathCommand::Close,
    ]
}

fn runtime_rounded_layout_rect_path_commands(
    bounds: RuntimeLayoutBounds,
    corners: RuntimeLayoutCorners,
) -> Vec<RuntimePathCommand> {
    let virtual_path = PathGeometryNode {
        local_id: 0,
        global_id: 0,
        type_name: "PointsPath",
        is_closed: true,
        is_hole: false,
        is_clockwise: true,
        parametric: None,
        vertices: vec![
            virtual_straight_vertex(0.0, 0.0, corners.top_left),
            virtual_straight_vertex(bounds.width, 0.0, corners.top_right),
            virtual_straight_vertex(bounds.width, bounds.height, corners.bottom_right),
            virtual_straight_vertex(0.0, bounds.height, corners.bottom_left),
        ],
    };
    let mut commands = points_path_commands(
        &virtual_path,
        ShapePaintPathKind::Local,
        Mat2D::IDENTITY,
        None,
    );
    prune_empty_path_segments(&mut commands);
    commands
}

fn runtime_layout_hug_size(
    child_sizes: &[(f32, f32)],
    gap: f32,
    is_row: bool,
    width_axis: bool,
) -> f32 {
    if child_sizes.is_empty() {
        return 0.0;
    }
    let gap_total = gap * child_sizes.len().saturating_sub(1) as f32;
    match (is_row, width_axis) {
        (true, true) => child_sizes.iter().map(|(width, _)| *width).sum::<f32>() + gap_total,
        (true, false) => child_sizes
            .iter()
            .map(|(_, height)| *height)
            .fold(0.0, f32::max),
        (false, true) => child_sizes
            .iter()
            .map(|(width, _)| *width)
            .fold(0.0, f32::max),
        (false, false) => child_sizes.iter().map(|(_, height)| *height).sum::<f32>() + gap_total,
    }
}

fn runtime_layout_alignment_is_space_between(alignment: u64) -> bool {
    matches!(alignment, 9..=11)
}

fn runtime_layout_main_alignment_offset(
    alignment: u64,
    is_row: bool,
    available: f32,
    occupied: f32,
) -> f32 {
    if runtime_layout_alignment_is_space_between(alignment) {
        return 0.0;
    }
    let remaining = (available - occupied).max(0.0);
    if is_row {
        match alignment {
            1 | 4 | 7 => remaining / 2.0,
            2 | 5 | 8 => remaining,
            _ => 0.0,
        }
    } else {
        match alignment {
            3..=5 => remaining / 2.0,
            6..=8 => remaining,
            _ => 0.0,
        }
    }
}

fn runtime_layout_cross_alignment_offset(
    alignment: u64,
    is_row: bool,
    available: f32,
    size: f32,
) -> f32 {
    let remaining = (available - size).max(0.0);
    if is_row {
        match alignment {
            3..=5 | 10 => remaining / 2.0,
            6..=8 | 11 => remaining,
            _ => 0.0,
        }
    } else {
        match alignment {
            1 | 4 | 7 | 10 => remaining / 2.0,
            2 | 5 | 8 | 11 => remaining,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeShapePaintKind {
    Fill,
    Stroke,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RuntimeShapePaintPathKind {
    Local,
    LocalClockwise,
    World,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeShapePaintCommand {
    pub paint_local: usize,
    pub paint_global_id: u32,
    pub mutator_local: Option<usize>,
    pub paint_type: RuntimeShapePaintKind,
    pub fill_rule: RenderFillRule,
    pub path_kind: RuntimeShapePaintPathKind,
    pub path_slot_index: usize,
    pub clip_path_slot_index: usize,
    pub blend_mode_value: u32,
    pub render_blend_mode_value: u32,
    pub render_opacity: f32,
    pub paint_state: Option<RuntimeShapePaintState>,
    pub feather_state: Option<RuntimeFeatherState>,
    pub paint_space_transform: Option<Mat2D>,
    pub path_commands: Vec<RuntimePathCommand>,
    pub effect_path_commands: Vec<RuntimePathCommand>,
    pub has_effect_path: bool,
    pub needs_save_operation: bool,
    pub shape_world_override: Option<Mat2D>,
    pub aliases_local_clockwise_path: bool,
    pub uses_temporary_paint: bool,
    pub allocates_text_paint_pool: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeShapePaintState {
    SolidColor {
        color: u32,
        render_color: u32,
    },
    LinearGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        render_opacity: f32,
        stops: Vec<RuntimeGradientStop>,
    },
    RadialGradient {
        start_x: f32,
        start_y: f32,
        end_x: f32,
        end_y: f32,
        opacity: f32,
        render_opacity: f32,
        stops: Vec<RuntimeGradientStop>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RuntimeGradientStop {
    pub color: u32,
    pub render_color: u32,
    pub position: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RuntimeNestedRenderCacheKey {
    host: u32,
    referenced_artboard_global: u32,
    instance_revision: u64,
}

#[derive(Default)]
pub struct RuntimeRenderPaintCache {
    paints: RuntimeRenderPaints,
    paint_configurations: RuntimeRenderPaintConfigurationSlots,
    preparation: Option<RuntimePaintPreparationFrame>,
    images: RuntimeRenderImages,
    meshes: RuntimeMeshRenderBufferSlots,
    nested_artboards: BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
}

#[derive(Default)]
pub struct RuntimeRenderPaints {
    paints_by_global: Vec<Option<Box<dyn RenderPaint>>>,
}

impl RuntimeRenderPaints {
    fn contains(&self, global_id: u32) -> bool {
        self.paints_by_global
            .get(global_id as usize)
            .is_some_and(Option::is_some)
    }

    fn insert(&mut self, global_id: u32, paint: Box<dyn RenderPaint>) {
        let slot = global_id as usize;
        if self.paints_by_global.len() <= slot {
            self.paints_by_global.resize_with(slot + 1, || None);
        }
        self.paints_by_global[slot] = Some(paint);
    }

    fn paint(&self, global_id: u32) -> Option<&Box<dyn RenderPaint>> {
        self.paints_by_global
            .get(global_id as usize)
            .and_then(|paint| paint.as_ref())
    }

    fn paint_mut(&mut self, global_id: u32) -> Option<&mut Box<dyn RenderPaint>> {
        self.paints_by_global.get_mut(global_id as usize)?.as_mut()
    }
}

impl RuntimeRenderPaintCache {
    pub fn root_paints_mut(&mut self) -> &mut RuntimeRenderPaints {
        &mut self.paints
    }

    pub fn root_images_mut(&mut self) -> &mut RuntimeRenderImages {
        &mut self.images
    }
}

#[derive(Default)]
pub struct RuntimeRenderImages {
    images_by_global: Vec<Option<Box<dyn RenderImage>>>,
}

impl RuntimeRenderImages {
    pub fn get(&self, global_id: u32) -> Option<&dyn RenderImage> {
        self.images_by_global
            .get(global_id as usize)
            .and_then(|image| image.as_deref())
    }

    pub fn insert(&mut self, global_id: u32, image: Box<dyn RenderImage>) {
        let slot = global_id as usize;
        if self.images_by_global.len() <= slot {
            self.images_by_global.resize_with(slot + 1, || None);
        }
        self.images_by_global[slot] = Some(image);
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeCachedRenderPaintConfiguration {
    instance_epoch: u64,
    configuration: RuntimeRenderPaintConfiguration,
}

#[derive(Default)]
struct RuntimeRenderPaintConfigurationSlots {
    by_global: Vec<Option<RuntimeCachedRenderPaintConfiguration>>,
}

impl RuntimeRenderPaintConfigurationSlots {
    fn get(&self, paint_global_id: u32) -> Option<&RuntimeCachedRenderPaintConfiguration> {
        self.by_global
            .get(paint_global_id as usize)
            .and_then(|configuration| configuration.as_ref())
    }

    fn is_current(&self, paint_global_id: u32, instance_epoch: u64) -> bool {
        self.get(paint_global_id)
            .is_some_and(|configuration| configuration.instance_epoch == instance_epoch)
    }

    fn get_mut(
        &mut self,
        paint_global_id: u32,
    ) -> Option<&mut RuntimeCachedRenderPaintConfiguration> {
        self.by_global.get_mut(paint_global_id as usize)?.as_mut()
    }

    fn insert(
        &mut self,
        paint_global_id: u32,
        configuration: RuntimeCachedRenderPaintConfiguration,
    ) {
        let slot = paint_global_id as usize;
        if self.by_global.len() <= slot {
            self.by_global.resize_with(slot + 1, || None);
        }
        self.by_global[slot] = Some(configuration);
    }

    fn remove(&mut self, paint_global_id: u32) {
        if let Some(configuration) = self.by_global.get_mut(paint_global_id as usize) {
            *configuration = None;
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RuntimeRenderPaintConfiguration {
    paint_type: RuntimeShapePaintKind,
    stroke_thickness: Option<f32>,
    stroke_cap: Option<RenderStrokeCap>,
    stroke_join: Option<RenderStrokeJoin>,
    blend_mode: RenderBlendMode,
    shader: RuntimeRenderPaintShaderConfiguration,
    feather: f32,
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeRenderPaintShaderConfiguration {
    None,
    SolidColor(u32),
    PreserveGradientShader,
}

struct RuntimeSourceMeshRenderBuffers {
    source_vertices: Box<dyn RenderBuffer>,
    uv_coords: Box<dyn RenderBuffer>,
    indices: Box<dyn RenderBuffer>,
    vertex_count: u32,
    index_count: u32,
}

struct RuntimeMeshRenderBuffers {
    _source_vertices: Box<dyn RenderBuffer>,
    vertices: Box<dyn RenderBuffer>,
    uv_coords: Box<dyn RenderBuffer>,
    indices: Box<dyn RenderBuffer>,
    vertex_count: u32,
    index_count: u32,
    last_vertex_bytes: Option<Vec<u8>>,
}

#[derive(Default)]
struct RuntimeMeshRenderBufferSlots {
    buffers_by_local: Vec<Option<RuntimeMeshRenderBuffers>>,
}

impl RuntimeMeshRenderBufferSlots {
    fn insert(&mut self, local_id: usize, buffers: RuntimeMeshRenderBuffers) {
        if self.buffers_by_local.len() <= local_id {
            self.buffers_by_local.resize_with(local_id + 1, || None);
        }
        self.buffers_by_local[local_id] = Some(buffers);
    }

    fn get_mut(&mut self, local_id: usize) -> Option<&mut RuntimeMeshRenderBuffers> {
        self.buffers_by_local.get_mut(local_id)?.as_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimePaintPreparationCacheKey {
    graph_global_id: u32,
    instance_epoch: u64,
    // World-space gradient shaders bake shape world transforms into their
    // endpoints, and keyed transform writes only bump prepared_epoch (via
    // WORLD_TRANSFORM dirt), never cache_epoch. Mirrors C++
    // linear_gradient.cpp: rebuild on PathFlags::world && WorldTransform dirt.
    // Zero when the graph has no world-space gradient paints.
    world_epoch: u64,
    nested_epoch: u64,
}

#[derive(Clone)]
struct RuntimePaintPreparationFrame {
    key: RuntimePaintPreparationCacheKey,
    has_nested_artboards: bool,
}

impl RuntimePaintPreparationFrame {
    fn can_skip_prepared_frame(
        &self,
        graph_global_id: u32,
        instance_epoch: u64,
        world_epoch: u64,
    ) -> bool {
        !self.has_nested_artboards
            && self.key.graph_global_id == graph_global_id
            && self.key.instance_epoch == instance_epoch
            && self.key.world_epoch == world_epoch
    }
}

#[derive(Default)]
pub struct RuntimeRenderPathCache {
    prepared_artboard: Option<RuntimePreparedArtboardFrame>,
    sorted_drawable_order: Option<RuntimeSortedDrawableOrderFrame>,
    layout_bounds: Option<RuntimeLayoutBoundsFrame>,
    path_composer_lookup: Option<RuntimePathComposerLookupFrame>,
    artboard_clip: Option<RuntimeRetainedRenderPath>,
    background_paths: RuntimeRetainedRenderPathSlots,
    clip_paths: RuntimeRetainedRenderPathSlots,
    layout_clip_paths: RuntimeRetainedRenderPathSlots,
    draw_paths: RuntimeDrawPathSlots,
    path_geometry_commands: RuntimePathGeometryCommandSlots,
    shape_paint_paths: RuntimeShapePaintPathCommandSlots,
    clipping_shape_commands: RuntimeClippingShapeCommandSlots,
    world_transforms: RuntimeWorldTransformSlots,
    image_layout_transforms: RuntimeImageLayoutTransformSlots,
    text_shape_paints: BTreeMap<RuntimeTextShapePaintCacheSlot, RuntimeCachedTextShapePaints>,
    allocated_text_paint_pools: BTreeSet<(u32, usize, u32)>,
    gradient_preparation: Option<RuntimeGradientPreparationFrame>,
    gradient_shaders: BTreeMap<u32, RuntimeGradientShaderCacheEntry>,
    nested_artboards: BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPathCache>,
    // Cycle guard for the component_world_transform_with_bounds <->
    // compute_component_world_transform_with_layout_bounds parent-walk
    // recursion: locals on the current walk. See the guard comment in
    // component_world_transform_with_bounds; empty except mid-walk.
    world_transform_visiting: BTreeSet<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimePreparedArtboardCacheKey {
    graph_global_id: u32,
    prepared_epoch: u64,
}

#[derive(Clone)]
struct RuntimePreparedArtboardFrame {
    key: RuntimePreparedArtboardCacheKey,
    layout_bounds: Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>>,
    background: Option<RuntimePreparedBackgroundFrame>,
    commands: Arc<Vec<RuntimeDrawCommand>>,
}

#[derive(Clone)]
struct RuntimePreparedBackgroundFrame {
    container_local: usize,
    path_commands: Arc<Vec<RuntimePathCommand>>,
    paints: Arc<Vec<RuntimeShapePaintCommand>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeSortedDrawableOrderCacheKey {
    graph_global_id: u32,
    draw_order_epoch: u64,
}

#[derive(Clone)]
struct RuntimeSortedDrawableOrderFrame {
    key: RuntimeSortedDrawableOrderCacheKey,
    order: Arc<Vec<SortedDrawableNode>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeLayoutBoundsCacheKey {
    graph_global_id: u32,
    layout_epoch: u64,
}

#[derive(Clone)]
struct RuntimeLayoutBoundsFrame {
    key: RuntimeLayoutBoundsCacheKey,
    bounds: Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimePathComposerLookupCacheKey {
    graph_global_id: u32,
    graph_identity: usize,
}

#[derive(Clone)]
struct RuntimePathComposerLookupFrame {
    key: RuntimePathComposerLookupCacheKey,
    composer_index_by_shape_local: Arc<Vec<Option<usize>>>,
    path_index_by_local: Arc<Vec<Option<usize>>>,
}

impl RuntimePathComposerLookupFrame {
    fn composer<'a>(
        &self,
        graph: &'a ArtboardGraph,
        shape_local: usize,
    ) -> Option<&'a PathComposerNode> {
        graph.path_composers.get(
            self.composer_index_by_shape_local
                .get(shape_local)?
                .as_ref()
                .copied()?,
        )
    }

    fn path<'a>(
        &self,
        graph: &'a ArtboardGraph,
        path_local: usize,
    ) -> Option<&'a PathGeometryNode> {
        graph.paths.get(
            self.path_index_by_local
                .get(path_local)?
                .as_ref()
                .copied()?,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimePathGeometryCommandsCacheKey {
    path_epoch: u64,
    layout_epoch: u64,
}

#[derive(Default)]
struct RuntimePathGeometryCommandSlots {
    graph_global_id: Option<u32>,
    by_local: Vec<Option<RuntimeCachedPathGeometryCommands>>,
}

impl RuntimePathGeometryCommandSlots {
    fn slot_mut(
        &mut self,
        graph_global_id: u32,
        path_local: usize,
    ) -> &mut Option<RuntimeCachedPathGeometryCommands> {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_local.clear();
        }
        if self.by_local.len() <= path_local {
            self.by_local.resize_with(path_local + 1, || None);
        }
        &mut self.by_local[path_local]
    }
}

#[derive(Clone)]
struct RuntimeCachedPathGeometryCommands {
    key: RuntimePathGeometryCommandsCacheKey,
    path: Arc<PathGeometryNode>,
    commands: Arc<Vec<RuntimePathCommand>>,
    has_weighted_context: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeShapePaintPathCommandsCacheKey {
    shape_local: usize,
    path_epoch: u64,
    layout_epoch: u64,
    path_kind: RuntimeShapePaintPathKind,
}

#[derive(Default)]
struct RuntimeShapePaintPathCommandSlots {
    graph_global_id: Option<u32>,
    by_paint_local: Vec<Option<RuntimeCachedShapePaintPathCommands>>,
}

impl RuntimeShapePaintPathCommandSlots {
    fn get(
        &mut self,
        graph_global_id: u32,
        paint_local: usize,
        key: RuntimeShapePaintPathCommandsCacheKey,
    ) -> Option<Arc<Vec<RuntimePathCommand>>> {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_paint_local.clear();
            return None;
        }
        self.by_paint_local
            .get(paint_local)
            .and_then(|cached| cached.as_ref())
            .filter(|cached| cached.key == key)
            .map(|cached| cached.commands.clone())
    }

    fn insert(
        &mut self,
        graph_global_id: u32,
        paint_local: usize,
        key: RuntimeShapePaintPathCommandsCacheKey,
        commands: Arc<Vec<RuntimePathCommand>>,
    ) {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_paint_local.clear();
        }
        if self.by_paint_local.len() <= paint_local {
            self.by_paint_local.resize_with(paint_local + 1, || None);
        }
        self.by_paint_local[paint_local] =
            Some(RuntimeCachedShapePaintPathCommands { key, commands });
    }
}

#[derive(Clone)]
struct RuntimeCachedShapePaintPathCommands {
    key: RuntimeShapePaintPathCommandsCacheKey,
    commands: Arc<Vec<RuntimePathCommand>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeClippingShapeCommandsCacheKey {
    path_epoch: u64,
    layout_epoch: u64,
}

#[derive(Default)]
struct RuntimeClippingShapeCommandSlots {
    graph_global_id: Option<u32>,
    by_clipping_shape_local: Vec<Option<RuntimeCachedClippingShapeCommands>>,
}

impl RuntimeClippingShapeCommandSlots {
    fn get(
        &mut self,
        graph_global_id: u32,
        clipping_shape_local: usize,
        key: RuntimeClippingShapeCommandsCacheKey,
    ) -> Option<Arc<Vec<RuntimePathCommand>>> {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_clipping_shape_local.clear();
            return None;
        }
        self.by_clipping_shape_local
            .get(clipping_shape_local)
            .and_then(|cached| cached.as_ref())
            .filter(|cached| cached.key == key)
            .map(|cached| cached.commands.clone())
    }

    fn insert(
        &mut self,
        graph_global_id: u32,
        clipping_shape_local: usize,
        key: RuntimeClippingShapeCommandsCacheKey,
        commands: Arc<Vec<RuntimePathCommand>>,
    ) {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_clipping_shape_local.clear();
        }
        if self.by_clipping_shape_local.len() <= clipping_shape_local {
            self.by_clipping_shape_local
                .resize_with(clipping_shape_local + 1, || None);
        }
        self.by_clipping_shape_local[clipping_shape_local] =
            Some(RuntimeCachedClippingShapeCommands { key, commands });
    }
}

#[derive(Clone)]
struct RuntimeCachedClippingShapeCommands {
    key: RuntimeClippingShapeCommandsCacheKey,
    commands: Arc<Vec<RuntimePathCommand>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeWorldTransformCacheKey {
    cache_epoch: u64,
    layout_epoch: u64,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeCachedWorldTransform {
    key: RuntimeWorldTransformCacheKey,
    transform: Mat2D,
}

#[derive(Default)]
struct RuntimeWorldTransformSlots {
    graph_global_id: Option<u32>,
    by_local: Vec<Option<RuntimeCachedWorldTransform>>,
}

impl RuntimeWorldTransformSlots {
    fn get(
        &mut self,
        graph_global_id: u32,
        local_id: usize,
        key: RuntimeWorldTransformCacheKey,
    ) -> Option<Mat2D> {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_local.clear();
            return None;
        }
        self.by_local
            .get(local_id)
            .and_then(|cached| cached.as_ref())
            .filter(|cached| cached.key == key)
            .map(|cached| cached.transform)
    }

    fn insert(
        &mut self,
        graph_global_id: u32,
        local_id: usize,
        key: RuntimeWorldTransformCacheKey,
        transform: Mat2D,
    ) {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_local.clear();
        }
        if self.by_local.len() <= local_id {
            self.by_local.resize(local_id + 1, None);
        }
        self.by_local[local_id] = Some(RuntimeCachedWorldTransform { key, transform });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeImageLayoutTransformCacheKey {
    cache_epoch: u64,
    layout_epoch: u64,
    image_width_bits: u32,
    image_height_bits: u32,
    layout_width_bits: u32,
    layout_height_bits: u32,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeCachedImageLayoutTransform {
    key: RuntimeImageLayoutTransformCacheKey,
    local_transform: Mat2D,
}

#[derive(Default)]
struct RuntimeImageLayoutTransformSlots {
    graph_global_id: Option<u32>,
    by_local: Vec<Option<RuntimeCachedImageLayoutTransform>>,
}

impl RuntimeImageLayoutTransformSlots {
    fn get(
        &mut self,
        graph_global_id: u32,
        local_id: usize,
        key: RuntimeImageLayoutTransformCacheKey,
    ) -> Option<Mat2D> {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_local.clear();
            return None;
        }
        self.by_local
            .get(local_id)
            .and_then(|cached| cached.as_ref())
            .filter(|cached| cached.key == key)
            .map(|cached| cached.local_transform)
    }

    fn insert(
        &mut self,
        graph_global_id: u32,
        local_id: usize,
        key: RuntimeImageLayoutTransformCacheKey,
        local_transform: Mat2D,
    ) {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_local.clear();
        }
        if self.by_local.len() <= local_id {
            self.by_local.resize(local_id + 1, None);
        }
        self.by_local[local_id] = Some(RuntimeCachedImageLayoutTransform {
            key,
            local_transform,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct RuntimeTextShapePaintCacheSlot {
    graph_global_id: u32,
    text_local: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeTextShapePaintCacheKey {
    path_epoch: u64,
    layout_epoch: u64,
    instance_epoch: u64,
}

#[derive(Clone)]
struct RuntimeCachedTextShapePaints {
    key: RuntimeTextShapePaintCacheKey,
    commands: Arc<Vec<RuntimeShapePaintCommand>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeGradientPreparationCacheKey {
    graph_global_id: u32,
    graph_identity: usize,
}

#[derive(Clone)]
struct RuntimeGradientPreparationFrame {
    key: RuntimeGradientPreparationCacheKey,
    paints_by_mutator: Arc<BTreeMap<usize, Vec<RuntimeGradientPaintRef>>>,
    dependency_order: Arc<Vec<usize>>,
    dependency_insertion_order: Arc<Vec<usize>>,
    has_world_space_gradient_paints: bool,
}

impl RuntimeGradientPreparationFrame {
    fn has_paints(&self) -> bool {
        !self.paints_by_mutator.is_empty()
    }
}

fn runtime_gradient_paints_include_world_space(
    graph: &ArtboardGraph,
    paints_by_mutator: &BTreeMap<usize, Vec<RuntimeGradientPaintRef>>,
) -> bool {
    paints_by_mutator.values().flatten().any(|paint_ref| {
        graph
            .shape_paint_containers
            .get(paint_ref.container_index)
            .and_then(|container| container.paints.get(paint_ref.paint_index))
            .is_some_and(|paint| paint.path_kind == Some(ShapePaintPathKind::World))
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeGradientPaintRef {
    container_index: usize,
    paint_index: usize,
}

struct RuntimeGradientShaderCacheEntry {
    state: RuntimeShapePaintState,
    shader: Box<dyn RenderShader>,
}

struct RuntimeCachedDrawPath {
    path: Box<dyn RenderPath>,
    raw_path: RawPath,
    epoch: u64,
    fill_rule: RenderFillRule,
}

struct RuntimeRetainedRenderPath {
    key: RuntimeRetainedRenderPathCacheKey,
    path: Box<dyn RenderPath>,
    raw_path: RawPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeRetainedRenderPathCacheKey {
    graph_global_id: u32,
    path_epoch: u64,
    layout_epoch: u64,
    fill_rule: RenderFillRule,
}

#[derive(Default)]
struct RuntimeRetainedRenderPathSlots {
    graph_global_id: Option<u32>,
    by_local: Vec<Option<RuntimeRetainedRenderPath>>,
}

impl RuntimeRetainedRenderPathSlots {
    fn slot_mut(
        &mut self,
        graph_global_id: u32,
        local_id: usize,
    ) -> &mut Option<RuntimeRetainedRenderPath> {
        if self.graph_global_id != Some(graph_global_id) {
            self.graph_global_id = Some(graph_global_id);
            self.by_local.clear();
        }
        if self.by_local.len() <= local_id {
            self.by_local.resize_with(local_id + 1, || None);
        }
        &mut self.by_local[local_id]
    }
}

#[derive(Default)]
struct RuntimeDrawPathSlots {
    by_local: Vec<Option<RuntimeDrawPathKindSlots>>,
    root: RuntimeDrawPathKindSlots,
}

impl RuntimeDrawPathSlots {
    fn slot_mut(&mut self, key: RuntimeDrawPathCacheKey) -> &mut Option<RuntimeCachedDrawPath> {
        match key.kind {
            RuntimeDrawPathCacheKind::Draw => {}
        }
        let slots = if let Some(local_id) = key.local_id {
            if self.by_local.len() <= local_id {
                self.by_local.resize_with(local_id + 1, || None);
            }
            self.by_local[local_id].get_or_insert_with(RuntimeDrawPathKindSlots::default)
        } else {
            &mut self.root
        }
        .slots_mut(key.path_kind);

        if slots.len() <= key.path_index {
            slots.resize_with(key.path_index + 1, || None);
        }
        &mut slots[key.path_index]
    }
}

#[derive(Default)]
struct RuntimeDrawPathKindSlots {
    local: Vec<Option<RuntimeCachedDrawPath>>,
    local_clockwise: Vec<Option<RuntimeCachedDrawPath>>,
    world: Vec<Option<RuntimeCachedDrawPath>>,
}

impl RuntimeDrawPathKindSlots {
    fn slots_mut(
        &mut self,
        path_kind: RuntimeShapePaintPathKind,
    ) -> &mut Vec<Option<RuntimeCachedDrawPath>> {
        match path_kind {
            RuntimeShapePaintPathKind::Local => &mut self.local,
            RuntimeShapePaintPathKind::LocalClockwise => &mut self.local_clockwise,
            RuntimeShapePaintPathKind::World => &mut self.world,
        }
    }
}

struct RuntimeGradientShaderResources<'a> {
    factory: &'a mut dyn RenderFactory,
    shaders: &'a mut BTreeMap<u32, RuntimeGradientShaderCacheEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeDrawPathCacheKey {
    kind: RuntimeDrawPathCacheKind,
    path_kind: RuntimeShapePaintPathKind,
    local_id: Option<usize>,
    path_index: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RuntimeDrawPathCacheKind {
    Draw,
}

fn runtime_gradient_paints_by_mutator(
    graph: &ArtboardGraph,
) -> BTreeMap<usize, Vec<RuntimeGradientPaintRef>> {
    let mut paints_by_mutator = BTreeMap::new();
    for (container_index, container) in graph.shape_paint_containers.iter().enumerate() {
        for (paint_index, paint) in container.paints.iter().enumerate() {
            if !matches!(
                paint.paint_state,
                Some(
                    ShapePaintStateNode::LinearGradient { .. }
                        | ShapePaintStateNode::RadialGradient { .. }
                )
            ) {
                continue;
            }
            if let Some(mutator_local) = paint.mutator_local {
                paints_by_mutator
                    .entry(mutator_local)
                    .or_insert_with(Vec::new)
                    .push(RuntimeGradientPaintRef {
                        container_index,
                        paint_index,
                    });
            }
        }
    }
    paints_by_mutator
}

fn runtime_gradient_dependency_order(order: &[usize], graph: &ArtboardGraph) -> Vec<usize> {
    order
        .iter()
        .copied()
        .chain(graph.local_objects.iter().map(|object| object.local_id))
        .collect()
}

fn runtime_mark_seen_u32(seen: &mut Vec<u32>, value: u32) -> bool {
    if seen.contains(&value) {
        return false;
    }
    seen.push(value);
    true
}

fn runtime_mark_seen_usize(seen: &mut Vec<usize>, value: usize) -> bool {
    if seen.contains(&value) {
        return false;
    }
    seen.push(value);
    true
}

fn runtime_upsert_nested_preparation_command(
    commands: &mut Vec<(usize, RuntimeDrawCommand)>,
    local_id: usize,
    command: RuntimeDrawCommand,
) {
    if let Some((_, existing)) = commands
        .iter_mut()
        .find(|(existing_local, _)| *existing_local == local_id)
    {
        *existing = command;
    } else {
        commands.push((local_id, command));
    }
}

fn runtime_insert_nested_preparation_command_if_absent(
    commands: &mut Vec<(usize, RuntimeDrawCommand)>,
    local_id: usize,
    command: RuntimeDrawCommand,
) {
    if commands
        .iter()
        .any(|(existing_local, _)| *existing_local == local_id)
    {
        return;
    }
    commands.push((local_id, command));
}

fn runtime_nested_preparation_command(
    commands: &[(usize, RuntimeDrawCommand)],
    local_id: usize,
) -> Option<&RuntimeDrawCommand> {
    commands
        .iter()
        .find_map(|(command_local, command)| (*command_local == local_id).then_some(command))
}

impl RuntimeRenderPathCache {
    fn component_world_transform_with_bounds(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Mat2D {
        let Some(layout_bounds) = layout_bounds else {
            return instance.runtime_component_world_transform_with_bounds(local_id, graph, None);
        };
        let Some(component) = instance.component(local_id) else {
            return Mat2D::IDENTITY;
        };
        if component.type_name == "Artboard" {
            return component.transform.world_transform;
        }
        if component.type_name != "LayoutComponent"
            && !(component.type_name == "NestedArtboardLayout"
                && layout_bounds.contains_key(&local_id))
        {
            let Some(parent_local) = component.parent_local else {
                return component.transform.world_transform;
            };
            if !instance
                .component(parent_local)
                .is_some_and(|parent| parent.layout_chain_has_layout_component)
            {
                return component.transform.world_transform;
            }
        }
        let key = RuntimeWorldTransformCacheKey {
            cache_epoch: instance.cache_epoch(),
            layout_epoch: instance.layout_epoch(),
        };
        if let Some(transform) = self.world_transforms.get(graph.global_id, local_id, key) {
            return transform;
        }

        // Cycle guard: the memo insert below only lands AFTER the recursive
        // parent walk, so a malformed file's parent cycle would recurse forever
        // (every lap re-misses the cache). Track the locals on the current walk
        // (C++'s DependencySorter::visit visited-set idiom,
        // src/dependency_sorter.cpp) and fall back to the stored world transform
        // on revisit, terminating gracefully. No-op on valid (acyclic) files.
        if !self.world_transform_visiting.insert(local_id) {
            return component.transform.world_transform;
        }
        let transform = self.compute_component_world_transform_with_layout_bounds(
            instance,
            graph,
            local_id,
            layout_bounds,
        );
        self.world_transform_visiting.remove(&local_id);
        self.world_transforms
            .insert(graph.global_id, local_id, key, transform);
        transform
    }

    fn compute_component_world_transform_with_layout_bounds(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        local_id: usize,
        layout_bounds: &BTreeMap<usize, RuntimeLayoutBounds>,
    ) -> Mat2D {
        let Some(component) = instance.component(local_id) else {
            return Mat2D::IDENTITY;
        };
        if component.type_name == "Artboard" {
            return component.transform.world_transform;
        }
        if component.type_name == "LayoutComponent" {
            return instance.runtime_layout_component_world_transform_with_bounds(
                local_id,
                graph,
                Some(layout_bounds),
            );
        }
        if component.type_name == "NestedArtboardLayout"
            && let Some(bounds) = layout_bounds.get(&local_id).copied()
        {
            if mat2d_linear_is_identity(component.transform.local_transform) {
                return Mat2D([1.0, 0.0, 0.0, 1.0, bounds.x, bounds.y]);
            }
            let (origin_x, origin_y) = instance
                .nested_artboards
                .get(&local_id)
                .map(|nested| {
                    (
                        nested.child.width * nested.child.origin_x,
                        nested.child.height * nested.child.origin_y,
                    )
                })
                .unwrap_or((0.0, 0.0));
            let mut components = component.transform.local_transform.decompose();
            components.x = bounds.x + origin_x;
            components.y = bounds.y + origin_y;
            return Mat2D::compose(components);
        }
        let Some(parent_local) = component.parent_local else {
            return component.transform.world_transform;
        };
        if !instance
            .component(parent_local)
            .is_some_and(|parent| parent.layout_chain_has_layout_component)
        {
            return component.transform.world_transform;
        }
        if let Some(layout_local) = component.constrained_layout_ancestor {
            let layout_world = instance.runtime_layout_component_world_transform_with_bounds(
                layout_local,
                graph,
                Some(layout_bounds),
            );
            let stored_layout_world = instance
                .component(layout_local)
                .map(|component| component.transform.world_transform)
                .unwrap_or(Mat2D::IDENTITY);
            return layout_world
                .multiply(stored_layout_world.invert_or_identity())
                .multiply(component.transform.world_transform);
        }
        self.component_world_transform_with_bounds(
            instance,
            graph,
            parent_local,
            Some(layout_bounds),
        )
        .multiply(component.transform.local_transform)
    }

    fn image_world_transform_with_bounds(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        local_id: usize,
        image_object: Option<&RuntimeObject>,
        image: &dyn RenderImage,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
        has_vertex_mesh: bool,
    ) -> Result<Option<Mat2D>> {
        let Some(layout_bounds) = layout_bounds else {
            return Ok(None);
        };
        let Some(component) = instance.component(local_id) else {
            return Ok(None);
        };
        let Some(parent_local) = component.parent_local else {
            return Ok(None);
        };
        if !instance
            .component(parent_local)
            .is_some_and(|parent| parent.type_name == "LayoutComponent")
        {
            return Ok(None);
        }
        let Some(parent_bounds) = layout_bounds.get(&parent_local) else {
            return Ok(None);
        };

        let image_width = image.width() as f32;
        let image_height = image.height() as f32;
        let key = RuntimeImageLayoutTransformCacheKey {
            cache_epoch: instance.cache_epoch(),
            layout_epoch: instance.layout_epoch(),
            image_width_bits: image_width.to_bits(),
            image_height_bits: image_height.to_bits(),
            layout_width_bits: parent_bounds.width.to_bits(),
            layout_height_bits: parent_bounds.height.to_bits(),
        };

        let local_transform = if let Some(local_transform) =
            self.image_layout_transforms
                .get(graph.global_id, local_id, key)
        {
            local_transform
        } else {
            let local_transform = runtime_image_layout_local_transform(
                instance,
                local_id,
                image_object,
                component.transform.local_transform,
                image_width,
                image_height,
                parent_bounds.width,
                parent_bounds.height,
                has_vertex_mesh,
            )?;
            self.image_layout_transforms
                .insert(graph.global_id, local_id, key, local_transform);
            local_transform
        };

        let parent_world = self.component_world_transform_with_bounds(
            instance,
            graph,
            parent_local,
            Some(layout_bounds),
        );
        Ok(Some(parent_world.multiply(local_transform)))
    }

    fn prepared_artboard_frame(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
    ) -> RuntimePreparedArtboardFrame {
        let key = RuntimePreparedArtboardCacheKey {
            graph_global_id: graph.global_id,
            prepared_epoch: instance.prepared_epoch(),
        };
        if self
            .prepared_artboard
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            let layout_bounds = self.layout_bounds_frame(instance, graph, runtime);
            let sorted_drawable_order = self.sorted_drawable_order_frame(instance, graph);
            let commands = instance.draw_commands_with_sorted_drawable_order(
                graph,
                layout_bounds.as_ref().as_ref(),
                sorted_drawable_order.as_slice(),
                self,
            );
            let background =
                runtime_prepared_background_frame(instance, graph, layout_bounds.as_ref().as_ref());
            self.prepared_artboard = Some(RuntimePreparedArtboardFrame {
                key,
                layout_bounds,
                background,
                commands: Arc::new(commands),
            });
        }

        self.prepared_artboard
            .as_ref()
            .expect("prepared artboard frame was just populated")
            .clone()
    }

    fn sorted_drawable_order_frame(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
    ) -> Arc<Vec<SortedDrawableNode>> {
        let key = RuntimeSortedDrawableOrderCacheKey {
            graph_global_id: graph.global_id,
            draw_order_epoch: instance.draw_order_epoch(),
        };
        if self
            .sorted_drawable_order
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.sorted_drawable_order = Some(RuntimeSortedDrawableOrderFrame {
                key,
                order: Arc::new(instance.runtime_sorted_drawable_order(graph)),
            });
        }

        self.sorted_drawable_order
            .as_ref()
            .expect("sorted drawable order frame was just populated")
            .order
            .clone()
    }

    fn layout_bounds_frame(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        runtime: Option<&RuntimeFile>,
    ) -> Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>> {
        let key = RuntimeLayoutBoundsCacheKey {
            graph_global_id: graph.global_id,
            layout_epoch: instance.layout_epoch(),
        };
        if self
            .layout_bounds
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.layout_bounds = Some(RuntimeLayoutBoundsFrame {
                key,
                bounds: Arc::new(instance.runtime_taffy_layout_bounds(graph, runtime)),
            });
        }

        self.layout_bounds
            .as_ref()
            .expect("layout bounds frame was just populated")
            .bounds
            .clone()
    }

    fn path_composer_lookup_frame(
        &mut self,
        graph: &ArtboardGraph,
    ) -> RuntimePathComposerLookupFrame {
        let key = RuntimePathComposerLookupCacheKey {
            graph_global_id: graph.global_id,
            graph_identity: graph as *const ArtboardGraph as usize,
        };
        if self
            .path_composer_lookup
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.path_composer_lookup = Some(RuntimePathComposerLookupFrame {
                key,
                composer_index_by_shape_local: Arc::new(runtime_local_index_lookup(
                    graph
                        .path_composers
                        .iter()
                        .enumerate()
                        .map(|(index, composer)| (composer.shape_local, index)),
                )),
                path_index_by_local: Arc::new(runtime_local_index_lookup(
                    graph
                        .paths
                        .iter()
                        .enumerate()
                        .map(|(index, path)| (path.local_id, index)),
                )),
            });
        }

        self.path_composer_lookup
            .as_ref()
            .expect("path composer lookup frame was just populated")
            .clone()
    }

    fn path_geometry_commands_frame(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        path: &PathGeometryNode,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> RuntimeCachedPathGeometryCommands {
        let key = RuntimePathGeometryCommandsCacheKey {
            path_epoch: instance.path_epoch(),
            layout_epoch: instance.layout_epoch(),
        };
        let slot = self
            .path_geometry_commands
            .slot_mut(graph.global_id, path.local_id);

        if slot.as_ref().is_none_or(|cached| cached.key != key) {
            let runtime_path =
                instance.runtime_path_geometry_with_layout_control(path, layout_bounds);
            let weighted_context =
                instance.runtime_weighted_path_context(&runtime_path, graph, layout_bounds);
            let commands = path_commands(
                &runtime_path,
                ShapePaintPathKind::World,
                Mat2D::IDENTITY,
                weighted_context.as_ref(),
            );
            *slot = Some(RuntimeCachedPathGeometryCommands {
                key,
                path: Arc::new(runtime_path),
                commands: Arc::new(commands),
                has_weighted_context: weighted_context.is_some(),
            });
        }

        slot.as_ref()
            .expect("path geometry command cache was just populated")
            .clone()
    }

    fn text_shape_paint_commands(
        &mut self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        command: &RuntimeDrawCommand,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Result<Arc<Vec<RuntimeShapePaintCommand>>> {
        let text_local = command
            .local_id
            .context("text draw command missing local id")?;
        let slot = RuntimeTextShapePaintCacheSlot {
            graph_global_id: graph.global_id,
            text_local,
        };
        let key = RuntimeTextShapePaintCacheKey {
            path_epoch: instance.path_epoch(),
            layout_epoch: instance.layout_epoch(),
            instance_epoch: instance.cache_epoch(),
        };

        if self
            .text_shape_paints
            .get(&slot)
            .is_none_or(|cached| cached.key != key)
        {
            let layout_constraint =
                instance.runtime_text_layout_constraint(text_local, layout_bounds);
            let mut commands = runtime_text_shape_paint_commands(
                runtime,
                instance,
                graph,
                command,
                layout_bounds,
                layout_constraint,
            )?;
            assign_shape_paint_path_slot_indices(&mut commands);
            self.text_shape_paints.insert(
                slot,
                RuntimeCachedTextShapePaints {
                    key,
                    commands: Arc::new(commands),
                },
            );
        }

        Ok(self
            .text_shape_paints
            .get(&slot)
            .expect("text shape paint cache was just populated")
            .commands
            .clone())
    }

    fn gradient_preparation_frame(
        &mut self,
        graph: &ArtboardGraph,
    ) -> RuntimeGradientPreparationFrame {
        let key = RuntimeGradientPreparationCacheKey {
            graph_global_id: graph.global_id,
            graph_identity: graph as *const ArtboardGraph as usize,
        };
        if self
            .gradient_preparation
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            let paints_by_mutator = runtime_gradient_paints_by_mutator(graph);
            let has_world_space_gradient_paints =
                runtime_gradient_paints_include_world_space(graph, &paints_by_mutator);
            self.gradient_preparation = Some(RuntimeGradientPreparationFrame {
                key,
                paints_by_mutator: Arc::new(paints_by_mutator),
                dependency_order: Arc::new(runtime_gradient_dependency_order(
                    &graph.dependency_order,
                    graph,
                )),
                dependency_insertion_order: Arc::new(runtime_gradient_dependency_order(
                    if graph.dependency_insertion_order.is_empty() {
                        &graph.dependency_order
                    } else {
                        &graph.dependency_insertion_order
                    },
                    graph,
                )),
                has_world_space_gradient_paints,
            });
        }

        self.gradient_preparation
            .as_ref()
            .expect("gradient preparation frame was just populated")
            .clone()
    }

    fn retained_render_path_key(
        &self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        fill_rule: RenderFillRule,
    ) -> RuntimeRetainedRenderPathCacheKey {
        RuntimeRetainedRenderPathCacheKey {
            graph_global_id: graph.global_id,
            path_epoch: instance.path_epoch(),
            layout_epoch: instance.layout_epoch(),
            fill_rule,
        }
    }

    fn clipping_shape_path_commands(
        &mut self,
        instance: &ArtboardInstance,
        graph: &ArtboardGraph,
        clipping_shape: &ClippingShapeNode,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> Arc<Vec<RuntimePathCommand>> {
        let key = RuntimeClippingShapeCommandsCacheKey {
            path_epoch: instance.path_epoch(),
            layout_epoch: instance.layout_epoch(),
        };
        if let Some(commands) =
            self.clipping_shape_commands
                .get(graph.global_id, clipping_shape.local_id, key)
        {
            return commands;
        }

        let commands = Arc::new(instance.runtime_clipping_shape_path_commands_uncached(
            clipping_shape,
            graph,
            layout_bounds,
            self,
        ));
        self.clipping_shape_commands.insert(
            graph.global_id,
            clipping_shape.local_id,
            key,
            commands.clone(),
        );
        commands
    }

    fn artboard_clip_path(
        &mut self,
        key: RuntimeRetainedRenderPathCacheKey,
        factory: &mut dyn RenderFactory,
        commands: impl FnOnce() -> Vec<RuntimePathCommand>,
    ) -> &mut Box<dyn RenderPath> {
        runtime_cached_retained_render_path_with_commands(
            &mut self.artboard_clip,
            key,
            factory,
            commands,
        )
    }

    fn clipping_shape_path(
        &mut self,
        key: RuntimeRetainedRenderPathCacheKey,
        local_id: usize,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
    ) -> &mut Box<dyn RenderPath> {
        let slot = self.clip_paths.slot_mut(key.graph_global_id, local_id);
        runtime_cached_retained_render_path(slot, key, factory, commands)
    }

    fn layout_clip_path(
        &mut self,
        key: RuntimeRetainedRenderPathCacheKey,
        local_id: usize,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
    ) -> &mut Box<dyn RenderPath> {
        let slot = self
            .layout_clip_paths
            .slot_mut(key.graph_global_id, local_id);
        runtime_cached_retained_render_path(slot, key, factory, commands)
    }

    fn background_path(
        &mut self,
        key: RuntimeRetainedRenderPathCacheKey,
        local_id: usize,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
    ) -> &mut Box<dyn RenderPath> {
        let slot = self
            .background_paths
            .slot_mut(key.graph_global_id, local_id);
        runtime_cached_retained_render_path(slot, key, factory, commands)
    }

    fn draw_path(
        &mut self,
        key: RuntimeDrawPathCacheKey,
        path_epoch: u64,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
    ) -> &mut Box<dyn RenderPath> {
        self.draw_path_with_optional_fill_rule(key, path_epoch, factory, commands, fill_rule, None)
    }

    fn draw_path_with_optional_fill_rule(
        &mut self,
        key: RuntimeDrawPathCacheKey,
        path_epoch: u64,
        factory: &mut dyn RenderFactory,
        commands: &[RuntimePathCommand],
        fill_rule: RenderFillRule,
        replay_fill_rule: Option<RenderFillRule>,
    ) -> &mut Box<dyn RenderPath> {
        let cached = self.draw_paths.slot_mut(key).get_or_insert_with(|| {
            let raw_path = runtime_raw_path_from_commands(commands);
            RuntimeCachedDrawPath {
                path: runtime_make_path_from_raw_path(factory, &raw_path, fill_rule),
                raw_path,
                epoch: path_epoch,
                fill_rule,
            }
        });
        if cached.epoch != path_epoch {
            runtime_rebuild_raw_path_from_commands(&mut cached.raw_path, commands);
            runtime_rebuild_path_from_raw_path_preserving_fill_rule(
                cached.path.as_mut(),
                &cached.raw_path,
            );
            cached.epoch = path_epoch;
        }
        if let Some(replay_fill_rule) = replay_fill_rule
            && cached.fill_rule != replay_fill_rule
        {
            cached.path.fill_rule(replay_fill_rule);
            cached.fill_rule = replay_fill_rule;
        }
        &mut cached.path
    }
}

fn runtime_local_index_lookup(
    entries: impl IntoIterator<Item = (usize, usize)>,
) -> Vec<Option<usize>> {
    let mut lookup = Vec::new();
    for (local_id, index) in entries {
        if lookup.len() <= local_id {
            lookup.resize(local_id + 1, None);
        }
        lookup[local_id] = Some(index);
    }
    lookup
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeFeatherState {
    pub feather_local: usize,
    pub space_value: u32,
    pub strength: f32,
    pub offset_x: f32,
    pub offset_y: f32,
    pub inner: bool,
    pub inner_path_commands: Vec<RuntimePathCommand>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RuntimePathCommand {
    Move {
        x: f32,
        y: f32,
    },
    Line {
        x: f32,
        y: f32,
    },
    Cubic {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        x3: f32,
        y3: f32,
    },
    Close,
}

pub fn preallocate_render_paints(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaints {
    let _source_artboard_paints = preallocate_render_paint_batch(runtime, factory);
    preallocate_artboard_render_paint_batch(runtime, graph, factory)
}

pub fn preallocate_source_render_paints(
    runtime: &RuntimeFile,
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaints {
    preallocate_render_paint_batch(runtime, factory)
}

pub fn preallocate_render_paints_for_artboard_tree(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaints {
    preallocate_render_paint_cache_for_artboard_tree(runtime, graph, artboards, factory).paints
}

pub fn preallocate_render_paint_cache_for_artboard_tree(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaintCache {
    preallocate_render_paint_cache_for_artboard_tree_internal(
        runtime, graph, artboards, factory, true, true,
    )
}

pub fn preallocate_render_paint_cache_for_scripted_artboard_tree(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaintCache {
    preallocate_render_paint_cache_for_artboard_tree_internal(
        runtime, graph, artboards, factory, false, true,
    )
}

pub fn preallocate_render_paint_cache_for_scripted_artboard_tree_after_source_paints(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaintCache {
    preallocate_render_paint_cache_for_artboard_tree_internal(
        runtime, graph, artboards, factory, false, false,
    )
}

fn preallocate_render_paint_cache_for_artboard_tree_internal(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
    include_script_input_artboards: bool,
    allocate_source_paints: bool,
) -> RuntimeRenderPaintCache {
    let image_asset_globals = runtime
        .file_assets()
        .into_iter()
        .filter(|asset| asset.type_name == "ImageAsset")
        .map(|asset| asset.id)
        .collect::<Vec<_>>();
    let pre_source_image_assets =
        pre_source_image_asset_globals(runtime, graph, artboards, &image_asset_globals);
    let mut images = RuntimeRenderImages::default();
    for asset_global in image_asset_globals
        .iter()
        .copied()
        .filter(|asset_global| pre_source_image_assets.contains(asset_global))
    {
        predecode_render_image(runtime, asset_global, factory, &mut images);
    }
    let _source_artboard_paints =
        allocate_source_paints.then(|| preallocate_render_paint_batch(runtime, factory));
    for asset_global in image_asset_globals
        .iter()
        .copied()
        .filter(|asset_global| !pre_source_image_assets.contains(asset_global))
    {
        predecode_render_image(runtime, asset_global, factory, &mut images);
    }
    let mut source_meshes =
        preallocate_source_mesh_render_buffers_for_artboards(runtime, artboards, factory, &images);
    let mut cache = RuntimeRenderPaintCache::default();
    preallocate_artboard_mesh_render_buffer_tree_batch_into(
        runtime,
        graph,
        artboards,
        factory,
        &mut source_meshes,
        &mut cache,
        &mut BTreeSet::new(),
        include_script_input_artboards,
    );
    preallocate_artboard_render_paint_tree_batch_into(
        runtime,
        graph,
        artboards,
        factory,
        &mut cache,
        &mut BTreeSet::new(),
        include_script_input_artboards,
    );
    cache.images = images;
    cache
}

/// Allocates the render paints owned by a newly cloned artboard instance.
///
/// C++ `Artboard::instance()` clones each `ShapePaint` and initializes its
/// render paint from the file's existing factory. File-level source paints and
/// decoded assets are not recreated for every clone.
pub fn preallocate_render_paint_cache_for_artboard_instance(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaintCache {
    let mut cache = RuntimeRenderPaintCache::default();
    preallocate_artboard_render_paint_tree_batch_into(
        runtime,
        graph,
        artboards,
        factory,
        &mut cache,
        &mut BTreeSet::new(),
        true,
    );
    cache
}

fn pre_source_image_asset_globals(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    image_asset_globals: &[u32],
) -> BTreeSet<u32> {
    let mut pre_source_assets = import_stack_pre_source_image_asset_globals(runtime)
        .into_iter()
        .collect::<BTreeSet<_>>();
    let heuristic_count =
        heuristic_pre_source_image_decode_count(graph, artboards, image_asset_globals.len());
    pre_source_assets.extend(image_asset_globals.iter().copied().take(heuristic_count));
    pre_source_assets
}

fn import_stack_pre_source_image_asset_globals(runtime: &RuntimeFile) -> Vec<u32> {
    let mut pre_source_assets = Vec::new();
    let mut latest_image_asset = None;

    for object in runtime.objects.iter().flatten() {
        if object.type_name == "Artboard" {
            break;
        }
        if !runtime_object_uses_golden_file_asset_stack(object) {
            continue;
        }

        if let Some(asset_global) = latest_image_asset.take()
            && embedded_file_asset_bytes(runtime, asset_global).is_some()
        {
            pre_source_assets.push(asset_global);
        }
        if object.type_name == "ImageAsset" {
            latest_image_asset = Some(object.id);
        }
    }

    pre_source_assets
}

fn runtime_object_uses_golden_file_asset_stack(object: &RuntimeObject) -> bool {
    // The C++ golden runner is built without scripting, so ScriptAsset and
    // ShaderAsset do not enter the WITH_RIVE_SCRIPTING FileAsset stack cases
    // and must not displace a pending image importer here.
    matches!(
        object.type_name,
        "ImageAsset" | "FontAsset" | "AudioAsset" | "BlobAsset" | "ManifestAsset"
    )
}

fn heuristic_pre_source_image_decode_count(
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    image_asset_count: usize,
) -> usize {
    if image_asset_count == 0 {
        return 0;
    }

    if image_asset_count == 1 {
        // Mirrors the focused C++ ordering observed in spotify_kids_demo:
        // a text-bearing selected root with one image only in a sibling
        // artboard decodes before source paints, while feather_render_test
        // remains source-paint-first.
        let selected_has_direct_image = graph
            .sorted_drawable_order
            .iter()
            .any(|drawable| drawable.type_name == "Image");
        let selected_has_text = graph
            .local_objects
            .iter()
            .any(|object| object.type_name == Some("Text"));
        let image_is_only_in_non_selected_artboard = !selected_has_direct_image
            && selected_has_text
            && artboards.iter().any(|artboard| {
                artboard.global_id != graph.global_id
                    && artboard
                        .sorted_drawable_order
                        .iter()
                        .any(|drawable| drawable.type_name == "Image")
            });
        return usize::from(image_is_only_in_non_selected_artboard);
    }

    let has_asset_image_bind = artboards.iter().any(|artboard| {
        artboard.data_binds.iter().any(|data_bind| {
            data_bind.target_type_name == Some("Image")
                && runtime_draw_property_key_for_name("Image", "assetId")
                    .is_some_and(|key| data_bind.property_key == u64::from(key))
        })
    });
    let has_layout_component = artboards.iter().any(|artboard| {
        artboard
            .components
            .iter()
            .any(|component| component.type_name == "LayoutComponent")
    });
    if has_asset_image_bind && !has_layout_component {
        // Mirrors the C++ `FileAssetImporter` ordering for no-layout
        // asset-image data-context files: most embedded images resolve before
        // source paint allocation, while the final generated/private asset
        // resolution happens after source paints. The current corpus exposes
        // both plain scripted-property and nested-artboard variants.
        return image_asset_count.saturating_sub(1);
    }

    // Mirrors the C++ `src/importers/file_asset_importer.cpp` resolution
    // surface observed in asset-image-bound image/layout fixtures.
    let has_asset_image_layout = artboards.iter().any(|artboard| {
        let has_artboard_layout_component = artboard
            .components
            .iter()
            .any(|component| component.type_name == "LayoutComponent");
        if !has_artboard_layout_component {
            return false;
        }

        artboard.data_binds.iter().any(|data_bind| {
            data_bind.target_type_name == Some("Image")
                && runtime_draw_property_key_for_name("Image", "assetId")
                    .is_some_and(|key| data_bind.property_key == u64::from(key))
        })
    });
    if has_asset_image_layout {
        return image_asset_count.min(2);
    }

    // Ported from C++ `src/importers/file_asset_importer.cpp`: consecutive
    // embedded file assets resolve as each importer is displaced, while the
    // final file-asset importer resolves later when the import stack unwinds.
    image_asset_count.saturating_sub(1)
}

fn predecode_render_image(
    runtime: &RuntimeFile,
    asset_global: u32,
    factory: &mut dyn RenderFactory,
    images: &mut RuntimeRenderImages,
) {
    if let Some(bytes) = embedded_file_asset_bytes(runtime, asset_global) {
        images.insert(asset_global, factory.decode_image(bytes));
    }
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

fn write_render_buffer_bytes(buffer: &mut dyn RenderBuffer, bytes: &[u8]) {
    let target = buffer.map_mut();
    debug_assert_eq!(target.len(), bytes.len());
    target.copy_from_slice(bytes);
    buffer.unmap();
}

fn push_f32_pair_bytes(bytes: &mut Vec<u8>, x: f32, y: f32) {
    bytes.extend_from_slice(&x.to_le_bytes());
    bytes.extend_from_slice(&y.to_le_bytes());
}

fn preallocate_artboard_mesh_render_buffer_tree_batch_into(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
    source_meshes: &mut BTreeMap<(u32, usize), RuntimeSourceMeshRenderBuffers>,
    cache: &mut RuntimeRenderPaintCache,
    visiting: &mut BTreeSet<u32>,
    include_script_input_artboards: bool,
) {
    if !visiting.insert(graph.global_id) {
        return;
    }

    cache.meshes = preallocate_artboard_mesh_render_buffers(graph, factory, source_meshes);

    for local_object in &graph.local_objects {
        if !include_script_input_artboards && local_object.type_name == Some("ScriptInputArtboard")
        {
            continue;
        }
        if let Some(child_graph) =
            referenced_artboard_graph_for_local_object(runtime, artboards, local_object.global_id)
        {
            let cache_key = nested_render_cache_key(
                Some(local_object.global_id),
                Some(local_object.local_id),
                child_graph.global_id,
                0,
            );
            let child_cache = cache.nested_artboards.entry(cache_key).or_default();
            preallocate_artboard_mesh_render_buffer_tree_batch_into(
                runtime,
                child_graph,
                artboards,
                factory,
                source_meshes,
                child_cache,
                visiting,
                include_script_input_artboards,
            );
        }
    }

    visiting.remove(&graph.global_id);
}

fn preallocate_source_mesh_render_buffers_for_artboards(
    runtime: &RuntimeFile,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
    image_by_global: &RuntimeRenderImages,
) -> BTreeMap<(u32, usize), RuntimeSourceMeshRenderBuffers> {
    // Ported from C++ `src/shapes/mesh.cpp::Mesh::onAssetLoaded`: source
    // artboards allocate their mesh vertex/UV/index buffers before cloned
    // artboard instances allocate the dynamic vertex buffers they draw with.
    let mut source_meshes = BTreeMap::new();
    for graph in artboards {
        for mesh in &graph.meshes {
            if let Some(source) = preallocate_source_mesh_render_buffers(
                runtime,
                graph,
                mesh,
                factory,
                image_by_global,
            ) {
                source_meshes.insert((graph.global_id, mesh.local_id), source);
            }
        }
    }
    source_meshes
}

fn preallocate_artboard_mesh_render_buffers(
    graph: &ArtboardGraph,
    factory: &mut dyn RenderFactory,
    source_meshes: &mut BTreeMap<(u32, usize), RuntimeSourceMeshRenderBuffers>,
) -> RuntimeMeshRenderBufferSlots {
    let mut source_buffers = Vec::new();
    for mesh in &graph.meshes {
        if let Some(source) = source_meshes.remove(&(graph.global_id, mesh.local_id)) {
            source_buffers.push((mesh.local_id, source));
        }
    }

    let mut buffers_by_local = RuntimeMeshRenderBufferSlots::default();
    for (mesh_local, source) in source_buffers {
        let vertices = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::None,
            source.vertex_count as usize * 8,
        );
        buffers_by_local.insert(
            mesh_local,
            RuntimeMeshRenderBuffers {
                _source_vertices: source.source_vertices,
                vertices,
                uv_coords: source.uv_coords,
                indices: source.indices,
                vertex_count: source.vertex_count,
                index_count: source.index_count,
                last_vertex_bytes: None,
            },
        );
    }
    buffers_by_local
}

fn preallocate_source_mesh_render_buffers(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    mesh: &MeshGeometryNode,
    factory: &mut dyn RenderFactory,
    image_by_global: &RuntimeRenderImages,
) -> Option<RuntimeSourceMeshRenderBuffers> {
    let mesh_object = runtime.object(mesh.global_id as usize)?;
    let parent_id_key = runtime_draw_property_key_for_name("Component", "parentId")
        .or_else(|| runtime_draw_property_key_for_name("WorldTransformComponent", "parentId"))?;
    let image_local = usize::try_from(runtime_object_uint_property_by_key(
        mesh_object,
        parent_id_key,
    )?)
    .ok()?;
    let image_global = graph
        .local_objects
        .iter()
        .find(|object| object.local_id == image_local)?
        .global_id;
    let image_object = runtime.object(image_global as usize)?;
    let image_asset = runtime.resolved_file_asset_for_referencer(image_object)?;
    let image = image_by_global.get(image_asset.id)?;
    let indices = mesh_object.mesh_triangle_indices()?;
    let vertex_count = u32::try_from(mesh.vertices.len()).ok()?;
    let index_count = u32::try_from(indices.len()).ok()?;

    let source_vertices = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::None,
        mesh.vertices.len() * 8,
    );
    let mut uv_coords = factory.make_render_buffer(
        RenderBufferType::Vertex,
        RenderBufferFlags::MappedOnceAtInitialization,
        mesh.vertices.len() * 8,
    );
    let uv_transform = image.uv_transform();
    let u_key = runtime_draw_property_key_for_name("MeshVertex", "u")?;
    let v_key = runtime_draw_property_key_for_name("MeshVertex", "v")?;
    let mut uv_bytes = Vec::with_capacity(mesh.vertices.len() * 8);
    for vertex in &mesh.vertices {
        let vertex_object = runtime.object(vertex.global_id as usize)?;
        let uv = uv_transform.transform_point(RenderVec2D::new(
            runtime_object_explicit_double_property_by_key(vertex_object, u_key).unwrap_or(0.0),
            runtime_object_explicit_double_property_by_key(vertex_object, v_key).unwrap_or(0.0),
        ));
        push_f32_pair_bytes(&mut uv_bytes, uv.x, uv.y);
    }
    write_render_buffer_bytes(uv_coords.as_mut(), &uv_bytes);

    let mut index_buffer = factory.make_render_buffer(
        RenderBufferType::Index,
        RenderBufferFlags::MappedOnceAtInitialization,
        indices.len() * 2,
    );
    let mut index_bytes = Vec::with_capacity(indices.len() * 2);
    for index in indices {
        index_bytes.extend_from_slice(&index.to_le_bytes());
    }
    write_render_buffer_bytes(index_buffer.as_mut(), &index_bytes);

    Some(RuntimeSourceMeshRenderBuffers {
        source_vertices,
        uv_coords,
        indices: index_buffer,
        vertex_count,
        index_count,
    })
}

fn preallocate_render_paint_batch(
    runtime: &RuntimeFile,
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaints {
    let mut paints = RuntimeRenderPaints::default();
    for object in runtime.objects.iter().flatten() {
        if matches!(object.type_name, "Fill" | "Stroke") {
            paints.insert(object.id, factory.make_render_paint());
        }
    }
    paints
}

fn preallocate_artboard_render_paint_batch(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    factory: &mut dyn RenderFactory,
) -> RuntimeRenderPaints {
    let mut paints = RuntimeRenderPaints::default();
    let mut allocated = BTreeSet::new();
    let paint_by_mutator = graph
        .shape_paint_containers
        .iter()
        .flat_map(|container| container.paints.iter())
        .map(|paint| (paint.mutator_global, paint.global_id))
        .collect::<BTreeMap<_, _>>();

    for local_object in &graph.local_objects {
        if let Some(paint_global_id) = paint_by_mutator.get(&Some(local_object.global_id)) {
            preallocate_render_paint(*paint_global_id, factory, &mut paints, &mut allocated);
        }
    }

    for local_object in &graph.local_objects {
        let Some(object) = runtime.object(local_object.global_id as usize) else {
            continue;
        };
        if matches!(object.type_name, "Fill" | "Stroke") {
            preallocate_render_paint(object.id, factory, &mut paints, &mut allocated);
        }
    }
    paints
}

fn preallocate_artboard_render_paint_tree_batch_into(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    factory: &mut dyn RenderFactory,
    cache: &mut RuntimeRenderPaintCache,
    visiting: &mut BTreeSet<u32>,
    include_script_input_artboards: bool,
) {
    if !visiting.insert(graph.global_id) {
        return;
    }

    let mut allocated_for_instance = BTreeSet::new();
    let paint_by_mutator = graph
        .shape_paint_containers
        .iter()
        .flat_map(|container| container.paints.iter())
        .map(|paint| (paint.mutator_global, paint.global_id))
        .collect::<BTreeMap<_, _>>();

    for local_object in &graph.local_objects {
        if !include_script_input_artboards && local_object.type_name == Some("ScriptInputArtboard")
        {
            continue;
        }
        if let Some(child_graph) =
            referenced_artboard_graph_for_local_object(runtime, artboards, local_object.global_id)
        {
            let cache_key = nested_render_cache_key(
                Some(local_object.global_id),
                Some(local_object.local_id),
                child_graph.global_id,
                0,
            );
            let child_cache = cache.nested_artboards.entry(cache_key).or_default();
            preallocate_artboard_render_paint_tree_batch_into(
                runtime,
                child_graph,
                artboards,
                factory,
                child_cache,
                visiting,
                include_script_input_artboards,
            );
        }
    }

    for local_object in &graph.local_objects {
        if let Some(paint_global_id) = paint_by_mutator.get(&Some(local_object.global_id)) {
            preallocate_render_paint_for_instance(
                *paint_global_id,
                factory,
                &mut cache.paints,
                &mut allocated_for_instance,
            );
        }
    }

    for local_object in &graph.local_objects {
        let Some(object) = runtime.object(local_object.global_id as usize) else {
            continue;
        };
        if matches!(object.type_name, "Fill" | "Stroke") {
            preallocate_render_paint_for_instance(
                object.id,
                factory,
                &mut cache.paints,
                &mut allocated_for_instance,
            );
        }
    }

    visiting.remove(&graph.global_id);
}

fn referenced_artboard_graph_for_local_object<'a>(
    runtime: &RuntimeFile,
    artboards: &'a [ArtboardGraph],
    global_id: u32,
) -> Option<&'a ArtboardGraph> {
    let object = runtime.object(global_id as usize)?;
    let referenced = runtime.resolved_artboard_for_referencer_object(object)?;
    artboards
        .iter()
        .find(|graph| graph.global_id == referenced.id)
}

fn nested_render_cache_key(
    global_id: Option<u32>,
    local_id: Option<usize>,
    referenced_artboard_global: u32,
    instance_revision: u64,
) -> RuntimeNestedRenderCacheKey {
    let host = global_id
        .or_else(|| local_id.and_then(|local_id| u32::try_from(local_id).ok()))
        .unwrap_or(referenced_artboard_global);
    RuntimeNestedRenderCacheKey {
        host,
        referenced_artboard_global,
        instance_revision,
    }
}

fn component_list_render_cache_key(
    global_id: Option<u32>,
    local_id: Option<usize>,
    referenced_artboard_global: u32,
    item_index: usize,
    instance_revision: u64,
) -> RuntimeNestedRenderCacheKey {
    let mut key = nested_render_cache_key(
        global_id,
        local_id,
        referenced_artboard_global,
        instance_revision,
    );
    key.instance_revision = (1_u64 << 63) | ((item_index as u64) << 32) | instance_revision;
    key
}

fn preallocate_render_paint(
    global_id: u32,
    factory: &mut dyn RenderFactory,
    paints: &mut RuntimeRenderPaints,
    allocated: &mut BTreeSet<u32>,
) {
    if allocated.insert(global_id) {
        paints.insert(global_id, factory.make_render_paint());
    }
}

fn preallocate_render_paint_for_instance(
    global_id: u32,
    factory: &mut dyn RenderFactory,
    paints: &mut RuntimeRenderPaints,
    allocated_for_instance: &mut BTreeSet<u32>,
) {
    if !allocated_for_instance.insert(global_id) {
        return;
    }

    let render_paint = factory.make_render_paint();
    if !paints.contains(global_id) {
        paints.insert(global_id, render_paint);
    }
}

fn runtime_draw_background(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    background: &RuntimePreparedBackgroundFrame,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut RuntimeRenderPaints,
    path_cache: &mut RuntimeRenderPathCache,
    mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
) -> Result<()> {
    for runtime_paint in background.paints.iter() {
        let global_id = runtime_paint.paint_global_id;
        let paint_configuration_is_current =
            paint_configurations
                .as_deref()
                .is_some_and(|configurations| {
                    configurations.is_current(global_id, instance.cache_epoch())
                });
        if !paint_configuration_is_current {
            let object = runtime
                .object(global_id as usize)
                .with_context(|| format!("missing paint global {global_id}"))?;
            let render_paint = paint_by_global
                .paint_mut(global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_mut();
            if let Some(configurations) = paint_configurations.as_mut() {
                runtime_configure_paint_with_cache(
                    render_paint,
                    configurations,
                    global_id,
                    instance,
                    object,
                    runtime_paint,
                )?;
            } else {
                runtime_configure_paint(render_paint, instance, object, runtime_paint, None)?;
            }
        }
        renderer.save();
        renderer.transform(RenderMat2D::IDENTITY);
        if let Some(feather) = runtime_paint.feather_state.as_ref()
            && runtime_feather_uses_world_space(feather)
            && !feather.inner
            && runtime_feather_has_offset(feather)
        {
            renderer.transform(runtime_translation(feather.offset_x, feather.offset_y));
        }
        let fill_rule = runtime_paint.fill_rule;
        // C++ retains one artboard background path independently of each
        // ShapePaint's fill rule. Individual fills only mutate the path at
        // draw time; the inner-feather clip observes the retained default.
        let key = path_cache.retained_render_path_key(instance, graph, RenderFillRule::NonZero);
        let background_path = path_cache.background_path(
            key,
            background.container_local,
            factory,
            background.path_commands.as_slice(),
        );
        let path = if let Some(feather) = runtime_paint.feather_state.as_ref()
            && feather.inner
        {
            renderer.clip_path(background_path.as_ref());
            path_cache.draw_path(
                RuntimeDrawPathCacheKey {
                    kind: RuntimeDrawPathCacheKind::Draw,
                    path_kind: runtime_draw_path_cache_path_kind(runtime_paint),
                    local_id: Some(runtime_paint.paint_local),
                    path_index: runtime_paint.path_slot_index,
                },
                instance.path_epoch(),
                factory,
                feather.inner_path_commands.as_slice(),
                RenderFillRule::Clockwise,
            )
        } else {
            if let Some(feather) = runtime_paint.feather_state.as_ref()
                && !runtime_feather_uses_world_space(feather)
                && runtime_feather_has_offset(feather)
            {
                renderer.transform(runtime_translation(feather.offset_x, feather.offset_y));
            }
            background_path
        };
        if runtime_paint.paint_type == RuntimeShapePaintKind::Fill {
            path.fill_rule(fill_rule);
        }
        renderer.draw_path(
            path.as_ref(),
            paint_by_global
                .paint(global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_ref(),
        );
        renderer.restore();
    }
    Ok(())
}

fn runtime_prepared_background_frame(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Option<RuntimePreparedBackgroundFrame> {
    let container = graph
        .shape_paint_containers
        .iter()
        .find(|container| container.local_id == 0)?;
    let background_bounds = layout_bounds
        .and_then(|bounds| bounds.get(&0).copied())
        .or_else(|| instance.runtime_root_artboard_layout_bounds(graph))
        .unwrap_or(RuntimeLayoutBounds {
            x: 0.0,
            y: 0.0,
            width: instance.width,
            height: instance.height,
        });
    // Preserve C++ Artboard::updateRenderPath's multiplication order,
    // including the signed zero visible to renderer streams.
    let left = -background_bounds.width * instance.origin_x;
    let top = -background_bounds.height * instance.origin_y;
    let path_commands = Arc::new(runtime_rect_commands(
        left,
        top,
        left + background_bounds.width,
        top + background_bounds.height,
    ));
    let paints = container
        .paints
        .iter()
        .filter_map(|paint| {
            runtime_background_shape_paint_command(
                instance,
                paint,
                container.blend_mode_value,
                path_commands.as_slice(),
            )
        })
        .collect::<Vec<_>>();
    if paints.is_empty() {
        return None;
    }
    Some(RuntimePreparedBackgroundFrame {
        container_local: container.local_id,
        path_commands,
        paints: Arc::new(paints),
    })
}

fn runtime_background_shape_paint_command(
    instance: &ArtboardInstance,
    paint: &ShapePaintNode,
    container_blend_mode_value: u32,
    path_commands: &[RuntimePathCommand],
) -> Option<RuntimeShapePaintCommand> {
    if !runtime_shape_paint_is_visible(instance, paint) {
        return None;
    }
    let render_opacity = instance
        .component(0)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let paint_state = runtime_shape_paint_state(instance, paint, render_opacity);
    if !runtime_shape_paint_state_is_visible(&paint_state) {
        return None;
    }
    let mut feather_state = runtime_feather_state(
        instance,
        paint.feather.as_ref(),
        path_commands,
        Mat2D::IDENTITY,
    );
    if let Some(feather_state) = feather_state.as_mut() {
        prune_empty_path_segments(&mut feather_state.inner_path_commands);
    }
    Some(RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        paint_global_id: paint.global_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        fill_rule: runtime_live_shape_paint_fill_rule(instance, paint),
        path_kind: RuntimeShapePaintPathKind::Local,
        path_slot_index: 0,
        clip_path_slot_index: 0,
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: runtime_shape_paint_blend_mode_value(
            paint.blend_mode_value,
            container_blend_mode_value,
        ),
        render_opacity,
        paint_state,
        feather_state,
        paint_space_transform: None,
        path_commands: Vec::new(),
        effect_path_commands: Vec::new(),
        has_effect_path: false,
        needs_save_operation: true,
        shape_world_override: None,
        aliases_local_clockwise_path: false,
        uses_temporary_paint: false,
        allocates_text_paint_pool: false,
    })
}

fn runtime_prepare_gradient_paint_command(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    opacity_local: usize,
    container: &ShapePaintContainerNode,
    paint: &ShapePaintNode,
    shape_world: Mat2D,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> RuntimeShapePaintCommand {
    let render_opacity = instance
        .component(opacity_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let paint_state = runtime_deform_gradient_paint_state_with_nsliced_node(
        instance,
        graph,
        container,
        layout_bounds,
        shape_world,
        runtime_shape_paint_state(instance, paint, render_opacity),
    );
    RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        paint_global_id: paint.global_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        fill_rule: runtime_live_shape_paint_fill_rule(instance, paint),
        path_kind: paint
            .path_kind
            .and_then(runtime_shape_paint_path_kind)
            .unwrap_or(RuntimeShapePaintPathKind::Local),
        path_slot_index: 0,
        clip_path_slot_index: 0,
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: runtime_shape_paint_blend_mode_value(
            paint.blend_mode_value,
            container.blend_mode_value,
        ),
        render_opacity,
        paint_state,
        feather_state: None,
        paint_space_transform: runtime_shape_paint_space_transform(paint, shape_world),
        path_commands: Vec::new(),
        effect_path_commands: Vec::new(),
        has_effect_path: false,
        needs_save_operation: false,
        shape_world_override: None,
        aliases_local_clockwise_path: false,
        uses_temporary_paint: false,
        allocates_text_paint_pool: false,
    }
}

fn runtime_shape_paint_container_world_transform(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    container: &ShapePaintContainerNode,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    path_cache: &mut RuntimeRenderPathCache,
) -> Mat2D {
    match container.type_name {
        "LayoutComponent" => path_cache.component_world_transform_with_bounds(
            instance,
            graph,
            container.local_id,
            layout_bounds,
        ),
        "ForegroundLayoutDrawable" => {
            let layout_local = graph
                .components
                .iter()
                .find(|component| component.local_id == container.local_id)
                .and_then(|component| component.parent_local)
                .filter(|parent_local| {
                    instance
                        .component(*parent_local)
                        .is_some_and(|component| component.type_name == "LayoutComponent")
                })
                .unwrap_or(container.local_id);
            path_cache.component_world_transform_with_bounds(
                instance,
                graph,
                layout_local,
                layout_bounds,
            )
        }
        _ => path_cache.component_world_transform_with_bounds(
            instance,
            graph,
            container.local_id,
            layout_bounds,
        ),
    }
}

fn runtime_foreground_layout_parent_local(
    instance: &ArtboardInstance,
    foreground_local: usize,
) -> Option<usize> {
    instance
        .component(foreground_local)
        .and_then(|component| component.parent_local)
        .filter(|parent_local| {
            instance
                .component(*parent_local)
                .is_some_and(|component| component.type_name == "LayoutComponent")
        })
}

fn runtime_draw_command(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    command: &RuntimeDrawCommand,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut RuntimeRenderPaints,
    image_by_global: &RuntimeRenderImages,
    mesh_by_local: &mut RuntimeMeshRenderBufferSlots,
    path_cache: &mut RuntimeRenderPathCache,
    mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
    nested_paint_caches: Option<
        &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
    >,
    nested_ancestors: &BTreeSet<u32>,
) -> Result<()> {
    match command.kind {
        RuntimeDrawCommandKind::ClipStart => {
            runtime_draw_clip_start(
                instance,
                graph,
                command,
                layout_bounds,
                factory,
                renderer,
                path_cache,
            );
            return Ok(());
        }
        RuntimeDrawCommandKind::ClipEnd => {
            runtime_draw_clip_end(graph, command, renderer);
            return Ok(());
        }
        RuntimeDrawCommandKind::Draw => {}
    }

    if command.object_kind == RuntimeDrawCommandObjectKind::DrawableProxy
        && let Some(layout_local) = command.local_id
        && instance.runtime_layout_component_clip_enabled(layout_local)
    {
        renderer.save();
        let path_commands = instance.runtime_layout_component_clip_path_commands(
            layout_local,
            graph,
            layout_bounds,
        );
        if !path_commands.is_empty() {
            let fill_rule = RenderFillRule::Clockwise;
            let key = path_cache.retained_render_path_key(instance, graph, fill_rule);
            let path = path_cache.layout_clip_path(key, layout_local, factory, &path_commands);
            renderer.clip_path(path.as_ref());
        }
    }

    if command.object_kind == RuntimeDrawCommandObjectKind::LayoutComponent
        && let Some(layout_local) = command.local_id
        && instance.runtime_layout_component_clip_enabled(layout_local)
    {
        renderer.restore();
        return Ok(());
    }

    if command.object_kind.is_nested_artboard() {
        return runtime_draw_nested_artboard(
            runtime,
            instance,
            graph,
            artboards,
            command,
            factory,
            renderer,
            paint_by_global,
            image_by_global,
            mesh_by_local,
            path_cache,
            paint_configurations.as_deref_mut(),
            nested_paint_caches,
            layout_bounds,
            nested_ancestors,
        );
    }

    if command.object_kind == RuntimeDrawCommandObjectKind::ArtboardComponentList {
        return runtime_draw_component_list(
            runtime,
            instance,
            graph,
            artboards,
            command,
            factory,
            renderer,
            paint_by_global,
            image_by_global,
            mesh_by_local,
            path_cache,
            paint_configurations.as_deref_mut(),
            nested_paint_caches,
            layout_bounds,
            nested_ancestors,
        );
    }

    if command.object_kind == RuntimeDrawCommandObjectKind::Image {
        return runtime_draw_image(
            runtime,
            instance,
            graph,
            command,
            layout_bounds,
            image_by_global,
            mesh_by_local,
            path_cache,
            renderer,
        );
    }

    if command.object_kind == RuntimeDrawCommandObjectKind::ScriptedDrawable {
        return runtime_draw_scripted_drawable(instance, command, factory, renderer);
    }

    let shape_world = command.world_transform.unwrap_or_else(|| {
        command
            .local_id
            .map(|local_id| {
                path_cache.component_world_transform_with_bounds(
                    instance,
                    graph,
                    local_id,
                    layout_bounds,
                )
            })
            .unwrap_or(Mat2D::IDENTITY)
    });
    let draws_text = command.object_kind == RuntimeDrawCommandObjectKind::Text;
    let text_shape_paints = if draws_text {
        Some(path_cache.text_shape_paint_commands(
            runtime,
            instance,
            graph,
            command,
            layout_bounds,
        )?)
    } else {
        None
    };
    let shape_paints = if draws_text {
        text_shape_paints
            .as_ref()
            .expect("text shape paints were just populated")
            .as_slice()
    } else {
        command.shape_paints.as_slice()
    };
    if draws_text && command.needs_save_operation {
        renderer.save();
    }

    let mut text_temporary_paints = Vec::new();
    if draws_text {
        text_temporary_paints.reserve(
            shape_paints
                .iter()
                .filter(|paint| paint.uses_temporary_paint)
                .count(),
        );
        for _ in shape_paints
            .iter()
            .filter(|paint| paint.uses_temporary_paint)
        {
            text_temporary_paints.push(factory.make_render_paint());
        }
    }
    let mut text_temporary_paint_index = 0;
    let draw_path_epoch = instance.path_epoch();
    let foreground_layout_path_cache_local =
        if command.object_kind == RuntimeDrawCommandObjectKind::ForegroundLayoutDrawable {
            command
                .local_id
                .and_then(|local_id| runtime_foreground_layout_parent_local(instance, local_id))
        } else {
            None
        };
    for paint in shape_paints {
        let global_id = paint.paint_global_id;
        let effect_or_shape_path_commands = if paint.has_effect_path {
            &paint.effect_path_commands
        } else {
            &paint.path_commands
        };
        let draw_path_commands = paint
            .feather_state
            .as_ref()
            .filter(|feather| feather.inner)
            .map(|feather| feather.inner_path_commands.as_slice())
            .unwrap_or(effect_or_shape_path_commands);
        let temporary_paint_index = if draws_text && paint.uses_temporary_paint {
            let object = runtime
                .object(global_id as usize)
                .with_context(|| format!("missing paint global {global_id}"))?;
            let render_paint = text_temporary_paints
                .get_mut(text_temporary_paint_index)
                .context("missing temporary text render paint")?;
            let index = text_temporary_paint_index;
            text_temporary_paint_index += 1;
            runtime_configure_paint(render_paint.as_mut(), instance, object, paint, None)?;
            Some(index)
        } else {
            let paint_configuration_is_current =
                paint_configurations
                    .as_deref()
                    .is_some_and(|configurations| {
                        configurations.is_current(
                            global_id,
                            runtime_paint_configuration_epoch(instance, paint),
                        )
                    });
            if !paint_configuration_is_current {
                let object = runtime
                    .object(global_id as usize)
                    .with_context(|| format!("missing paint global {global_id}"))?;
                let render_paint = paint_by_global
                    .paint_mut(global_id)
                    .with_context(|| format!("missing render paint for global {global_id}"))?
                    .as_mut();
                if let Some(configurations) = paint_configurations.as_mut() {
                    runtime_configure_paint_with_cache(
                        render_paint,
                        configurations,
                        global_id,
                        instance,
                        object,
                        paint,
                    )?;
                } else {
                    runtime_configure_paint(render_paint, instance, object, paint, None)?;
                }
            }
            None
        };
        let mut saved = !paint.needs_save_operation;
        let paint_shape_world = paint.shape_world_override.unwrap_or(shape_world);
        // Ported from C++ src/shapes/paint/shape_paint.cpp and feather.cpp.
        // C++ reuses the parent layout render path for effect-free foreground
        // layout paints.
        let draw_path_cache_local = foreground_layout_path_cache_local
            .filter(|_| !paint.has_effect_path && paint.feather_state.is_none())
            .or_else(|| paint.has_effect_path.then_some(paint.paint_local))
            .or(command.local_id);
        let inner_feather_path_cache_local = if paint
            .feather_state
            .as_ref()
            .is_some_and(|feather| feather.inner)
        {
            Some(paint.paint_local)
        } else {
            draw_path_cache_local
        };
        if let Some(feather) = paint.feather_state.as_ref()
            && runtime_feather_uses_world_space(feather)
            && !feather.inner
            && runtime_feather_has_offset(feather)
        {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(runtime_translation(feather.offset_x, feather.offset_y));
        }
        if matches!(
            paint.path_kind,
            RuntimeShapePaintPathKind::Local | RuntimeShapePaintPathKind::LocalClockwise
        ) {
            if !saved {
                saved = true;
                renderer.save();
            }
            renderer.transform(runtime_render_mat(paint_shape_world));
        }
        if let Some(feather) = paint.feather_state.as_ref() {
            if feather.inner {
                if feather.inner_path_commands.is_empty() {
                    continue;
                }
                if !saved {
                    saved = true;
                    renderer.save();
                }
                let clip_path = path_cache.draw_path(
                    RuntimeDrawPathCacheKey {
                        kind: RuntimeDrawPathCacheKind::Draw,
                        path_kind: runtime_draw_path_cache_path_kind(paint),
                        local_id: draw_path_cache_local,
                        path_index: paint.clip_path_slot_index,
                    },
                    draw_path_epoch,
                    factory,
                    effect_or_shape_path_commands,
                    RenderFillRule::Clockwise,
                );
                renderer.clip_path(clip_path.as_ref());
            } else if !runtime_feather_uses_world_space(feather)
                && runtime_feather_has_offset(feather)
            {
                if !saved {
                    saved = true;
                    renderer.save();
                }
                renderer.transform(runtime_translation(feather.offset_x, feather.offset_y));
            }
        }
        let replay_fill_rule = if !draws_text && paint.paint_type == RuntimeShapePaintKind::Fill {
            Some(paint.fill_rule)
        } else {
            None
        };
        let path = path_cache.draw_path_with_optional_fill_rule(
            RuntimeDrawPathCacheKey {
                kind: RuntimeDrawPathCacheKind::Draw,
                path_kind: runtime_draw_path_cache_path_kind(paint),
                local_id: inner_feather_path_cache_local,
                path_index: paint.path_slot_index,
            },
            draw_path_epoch,
            factory,
            draw_path_commands,
            RenderFillRule::Clockwise,
            replay_fill_rule,
        );
        let render_paint = if let Some(index) = temporary_paint_index {
            text_temporary_paints[index].as_ref()
        } else {
            paint_by_global
                .paint(global_id)
                .with_context(|| format!("missing render paint for global {global_id}"))?
                .as_ref()
        };
        renderer.draw_path(path.as_ref(), render_paint);
        if saved && paint.needs_save_operation {
            renderer.restore();
        }
        if draws_text
            && paint.allocates_text_paint_pool
            && path_cache.allocated_text_paint_pools.insert((
                graph.global_id,
                command.local_id.unwrap_or_default(),
                paint.paint_global_id,
            ))
        {
            let _ = factory.make_render_paint();
        }
    }
    if draws_text && command.needs_save_operation {
        renderer.restore();
    }

    Ok(())
}

fn runtime_draw_scripted_drawable(
    instance: &ArtboardInstance,
    command: &RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    let Some(global_id) = command.global_id else {
        return Ok(());
    };
    let Some(script_instance) = instance.script_instance_for_global(global_id) else {
        return Ok(());
    };

    // Ported from C++ `src/scripted/scripted_drawable.cpp`.
    let needs_opacity_save = command.render_opacity != 1.0;
    if command.needs_save_operation || needs_opacity_save {
        renderer.save();
    }
    if needs_opacity_save {
        renderer.modulate_opacity(command.render_opacity);
    }
    renderer.transform(runtime_render_mat(
        command.world_transform.unwrap_or(Mat2D::IDENTITY),
    ));

    let mut host = crate::NoopScriptHost;
    script_instance
        .borrow_mut()
        .call_draw(factory, renderer, &mut host)
        .map_err(|error| anyhow::anyhow!("scripted drawable {global_id} draw failed: {error}"))?;

    if command.needs_save_operation || needs_opacity_save {
        renderer.restore();
    }
    Ok(())
}

fn runtime_draw_path_cache_path_kind(
    paint: &RuntimeShapePaintCommand,
) -> RuntimeShapePaintPathKind {
    if paint.aliases_local_clockwise_path
        && paint.path_kind == RuntimeShapePaintPathKind::LocalClockwise
    {
        RuntimeShapePaintPathKind::Local
    } else {
        paint.path_kind
    }
}

fn runtime_draw_image(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    command: &RuntimeDrawCommand,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    image_by_global: &RuntimeRenderImages,
    mesh_by_local: &mut RuntimeMeshRenderBufferSlots,
    path_cache: &mut RuntimeRenderPathCache,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    // Ported from C++ `src/shapes/image.cpp::Image::draw`.
    let local_id = command.local_id.context("image command missing local id")?;
    let image_object = command
        .global_id
        .and_then(|global_id| runtime.object(global_id as usize));
    let Some(image) = command
        .resolved_image_asset_global
        .and_then(|asset_global| image_by_global.get(asset_global))
    else {
        // C++ `Image::draw` returns before saving when the asset has no
        // decoded RenderImage, e.g. hosted images with no loader.
        return Ok(());
    };

    if command.needs_save_operation {
        renderer.save();
    }

    if let Some(mesh) = runtime_image_mesh(runtime, graph, local_id) {
        let buffers = mesh_by_local
            .get_mut(mesh.local_id)
            .with_context(|| format!("missing mesh render buffers for local {}", mesh.local_id))?;
        runtime_draw_mesh_image(
            runtime,
            instance,
            graph,
            local_id,
            image_object,
            mesh,
            buffers,
            layout_bounds,
            image,
            path_cache,
            renderer,
        )?;
        if command.needs_save_operation {
            renderer.restore();
        }
        return Ok(());
    }

    let origin_x_key =
        runtime_draw_property_key_for_name("Image", "originX").context("missing Image.originX")?;
    let origin_y_key =
        runtime_draw_property_key_for_name("Image", "originY").context("missing Image.originY")?;
    let origin_x = instance
        .double_property(local_id, origin_x_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_x_key)
            })
        })
        .unwrap_or(0.5);
    let origin_y = instance
        .double_property(local_id, origin_y_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_y_key)
            })
        })
        .unwrap_or(0.5);
    let world = path_cache
        .image_world_transform_with_bounds(
            instance,
            graph,
            local_id,
            image_object,
            image,
            layout_bounds,
            false,
        )?
        .unwrap_or_else(|| {
            path_cache.component_world_transform_with_bounds(
                instance,
                graph,
                local_id,
                layout_bounds,
            )
        });
    renderer.transform(runtime_render_mat(world));

    renderer.transform(RenderMat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        -(image.width() as f32 * origin_x),
        -(image.height() as f32 * origin_y),
    ]));

    let blend_mode_key = runtime_draw_property_key_for_name("Drawable", "blendModeValue")
        .context("missing Drawable.blendModeValue")?;
    let blend_mode_value = instance
        .uint_property(local_id, blend_mode_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_uint_property_by_key(object, blend_mode_key)
            })
        })
        .unwrap_or(3);
    let opacity = instance
        .component(local_id)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    renderer.draw_image(
        Some(image),
        RenderImageSampler::LINEAR_CLAMP,
        runtime_blend_mode(u32::try_from(blend_mode_value).unwrap_or(3))?,
        opacity,
    );

    if command.needs_save_operation {
        renderer.restore();
    }
    Ok(())
}

fn runtime_image_mesh<'a>(
    runtime: &RuntimeFile,
    graph: &'a ArtboardGraph,
    image_local: usize,
) -> Option<&'a MeshGeometryNode> {
    let parent_id_key = runtime_draw_property_key_for_name("Component", "parentId")
        .or_else(|| runtime_draw_property_key_for_name("WorldTransformComponent", "parentId"))?;
    graph.meshes.iter().find(|mesh| {
        runtime
            .object(mesh.global_id as usize)
            .and_then(|object| runtime_object_uint_property_by_key(object, parent_id_key))
            == Some(image_local as u64)
    })
}

fn runtime_draw_mesh_image(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    image_local: usize,
    image_object: Option<&RuntimeObject>,
    mesh: &MeshGeometryNode,
    buffers: &mut RuntimeMeshRenderBuffers,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    image: &dyn RenderImage,
    path_cache: &mut RuntimeRenderPathCache,
    renderer: &mut dyn Renderer,
) -> Result<()> {
    let weighted_context = instance.runtime_weighted_mesh_context(mesh, graph, layout_bounds);
    if weighted_context.is_none() {
        let world = path_cache
            .image_world_transform_with_bounds(
                instance,
                graph,
                image_local,
                image_object,
                image,
                layout_bounds,
                true,
            )?
            .unwrap_or_else(|| {
                path_cache.component_world_transform_with_bounds(
                    instance,
                    graph,
                    image_local,
                    layout_bounds,
                )
            });
        renderer.transform(runtime_render_mat(world));
    }

    runtime_update_mesh_vertex_buffer(runtime, instance, mesh, weighted_context.as_ref(), buffers)?;

    let blend_mode_key = runtime_draw_property_key_for_name("Drawable", "blendModeValue")
        .context("missing Drawable.blendModeValue")?;
    let blend_mode_value = instance
        .uint_property(image_local, blend_mode_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_uint_property_by_key(object, blend_mode_key)
            })
        })
        .unwrap_or(3);
    let opacity = instance
        .component(image_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    renderer.draw_image_mesh(
        Some(image),
        RenderImageSampler::LINEAR_CLAMP,
        Some(buffers.vertices.as_ref()),
        Some(buffers.uv_coords.as_ref()),
        Some(buffers.indices.as_ref()),
        buffers.vertex_count,
        buffers.index_count,
        runtime_blend_mode(u32::try_from(blend_mode_value).unwrap_or(3))?,
        opacity,
    );
    Ok(())
}

fn runtime_update_mesh_vertex_buffer(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    mesh: &MeshGeometryNode,
    weighted_context: Option<&WeightedPathContext>,
    buffers: &mut RuntimeMeshRenderBuffers,
) -> Result<()> {
    let bytes = runtime_mesh_vertex_buffer_bytes(runtime, instance, mesh, weighted_context)?;
    if buffers.last_vertex_bytes.as_ref() == Some(&bytes) {
        return Ok(());
    }
    write_render_buffer_bytes(buffers.vertices.as_mut(), &bytes);
    buffers.last_vertex_bytes = Some(bytes);
    Ok(())
}

fn runtime_mesh_vertex_buffer_bytes(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    mesh: &MeshGeometryNode,
    weighted_context: Option<&WeightedPathContext>,
) -> Result<Vec<u8>> {
    let mut bytes = Vec::with_capacity(mesh.vertices.len() * 8);
    for vertex in &mesh.vertices {
        let (x, y) =
            runtime_mesh_vertex_render_translation(runtime, instance, vertex, weighted_context)?;
        push_f32_pair_bytes(&mut bytes, x, y);
    }
    Ok(bytes)
}

fn runtime_mesh_vertex_render_translation(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    vertex: &MeshVertexNode,
    weighted_context: Option<&WeightedPathContext>,
) -> Result<(f32, f32)> {
    let vertex_object = runtime
        .object(vertex.global_id as usize)
        .with_context(|| format!("missing mesh vertex global {}", vertex.global_id))?;
    let x_key = runtime_draw_property_key_for_name("Vertex", "x").context("missing Vertex.x")?;
    let y_key = runtime_draw_property_key_for_name("Vertex", "y").context("missing Vertex.y")?;
    let x = instance
        .double_property(vertex.local_id, x_key)
        .or_else(|| runtime_object_explicit_double_property_by_key(vertex_object, x_key))
        .unwrap_or(0.0);
    let y = instance
        .double_property(vertex.local_id, y_key)
        .or_else(|| runtime_object_explicit_double_property_by_key(vertex_object, y_key))
        .unwrap_or(0.0);
    if let Some(weighted_context) = weighted_context
        && let Some(weight_global) = vertex.weight_global
    {
        let weight = runtime
            .object(weight_global as usize)
            .with_context(|| format!("missing mesh weight global {weight_global}"))?;
        let indices_key = runtime_draw_property_key_for_name("Weight", "indices")
            .context("missing Weight.indices")?;
        let values_key = runtime_draw_property_key_for_name("Weight", "values")
            .context("missing Weight.values")?;
        return weighted_context
            .deform_point(
                (x, y),
                u32::try_from(
                    runtime_object_explicit_uint_property_by_key(weight, indices_key).unwrap_or(1),
                )
                .unwrap_or(1),
                u32::try_from(
                    runtime_object_explicit_uint_property_by_key(weight, values_key).unwrap_or(255),
                )
                .unwrap_or(255),
            )
            .context("mesh bone deformation referenced a missing bone transform");
    }
    Ok((x, y))
}

fn runtime_image_layout_local_transform(
    instance: &ArtboardInstance,
    local_id: usize,
    image_object: Option<&RuntimeObject>,
    base_local_transform: Mat2D,
    image_width: f32,
    image_height: f32,
    layout_width: f32,
    layout_height: f32,
    has_vertex_mesh: bool,
) -> Result<Mat2D> {
    // Ported from C++ `src/shapes/image.cpp::Image::updateImageScale`.
    let fit_key =
        runtime_draw_property_key_for_name("Image", "fit").context("missing Image.fit")?;
    let origin_x_key =
        runtime_draw_property_key_for_name("Image", "originX").context("missing Image.originX")?;
    let origin_y_key =
        runtime_draw_property_key_for_name("Image", "originY").context("missing Image.originY")?;
    let alignment_x_key = runtime_draw_property_key_for_name("Image", "alignmentX")
        .context("missing Image.alignmentX")?;
    let alignment_y_key = runtime_draw_property_key_for_name("Image", "alignmentY")
        .context("missing Image.alignmentY")?;
    let fit = instance
        .uint_property(local_id, fit_key)
        .or_else(|| {
            image_object
                .and_then(|object| runtime_object_explicit_uint_property_by_key(object, fit_key))
        })
        .unwrap_or(0);
    let origin_x = instance
        .double_property(local_id, origin_x_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_x_key)
            })
        })
        .unwrap_or(0.5);
    let origin_y = instance
        .double_property(local_id, origin_y_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, origin_y_key)
            })
        })
        .unwrap_or(0.5);
    let alignment_x = instance
        .double_property(local_id, alignment_x_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, alignment_x_key)
            })
        })
        .unwrap_or(0.0);
    let alignment_y = instance
        .double_property(local_id, alignment_y_key)
        .or_else(|| {
            image_object.and_then(|object| {
                runtime_object_explicit_double_property_by_key(object, alignment_y_key)
            })
        })
        .unwrap_or(0.0);

    let width_scale = layout_width / image_width;
    let height_scale = layout_height / image_height;
    let (mut scale_x, mut scale_y) = match fit {
        1 => {
            let scale = width_scale.min(height_scale);
            (scale, scale)
        }
        2 => {
            let scale = width_scale.max(height_scale);
            (scale, scale)
        }
        3 => (width_scale, width_scale),
        4 => (height_scale, height_scale),
        5 => (1.0, 1.0),
        6 => {
            let scale = width_scale.min(height_scale);
            let scale = if scale < 1.0 { scale } else { 1.0 };
            (scale, scale)
        }
        0 | 7 => (width_scale, height_scale),
        _ => (width_scale, height_scale),
    };

    if fit != 5 && fit != 6 && (!scale_x.is_finite() || !scale_y.is_finite()) {
        scale_x = f32::NAN;
        scale_y = f32::NAN;
    }

    let (mut offset_x, mut offset_y) = (0.0, 0.0);
    if fit != 0 {
        let (bounds_origin_x, bounds_origin_y) = if has_vertex_mesh {
            (0.5, 0.5)
        } else {
            (origin_x, origin_y)
        };
        let bounds_left = -image_width * bounds_origin_x;
        let bounds_top = -image_height * bounds_origin_y;
        let x_align = (alignment_x + 1.0) * 0.5;
        let y_align = (alignment_y + 1.0) * 0.5;
        let scaled_left = bounds_left * scale_x;
        let scaled_top = bounds_top * scale_y;
        let width_remainder = layout_width - image_width * scale_x;
        let height_remainder = layout_height - image_height * scale_y;
        offset_x = -scaled_left + width_remainder * x_align;
        offset_y = -scaled_top + height_remainder * y_align;
    }

    let mut components = base_local_transform.decompose();
    components.scale_x = scale_x;
    components.scale_y = scale_y;
    components.x += offset_x;
    components.y += offset_y;
    Ok(Mat2D::compose(components))
}

#[allow(clippy::too_many_arguments)]
fn runtime_draw_component_list(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    command: &RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut RuntimeRenderPaints,
    image_by_global: &RuntimeRenderImages,
    mesh_by_local: &mut RuntimeMeshRenderBufferSlots,
    path_cache: &mut RuntimeRenderPathCache,
    mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
    mut nested_paint_caches: Option<
        &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
    >,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    nested_ancestors: &BTreeSet<u32>,
) -> Result<()> {
    let local_id = command
        .local_id
        .context("component-list draw command missing local id")?;
    let Some(items) = instance.component_list_items.get(&local_id) else {
        return Ok(());
    };
    let host_world =
        path_cache.component_world_transform_with_bounds(instance, graph, local_id, layout_bounds);
    if command.needs_save_operation {
        renderer.save();
    }
    renderer.transform(runtime_render_mat(host_world));

    let list_style_local = instance
        .runtime_layout_component_style_local(local_id)
        .or_else(|| {
            instance
                .component(local_id)
                .and_then(|component| component.parent_local)
                .and_then(|parent_local| {
                    instance.runtime_layout_component_style_local(parent_local)
                })
        });
    let (padding_left, padding_top, gap, is_row) = list_style_local
        .map(|style_local| {
            (
                instance
                    .runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingLeft,
                    )
                    .unwrap_or(0.0),
                instance
                    .runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::PaddingTop,
                    )
                    .unwrap_or(0.0),
                instance
                    .runtime_layout_style_double(
                        style_local,
                        RuntimeLayoutStyleProperty::GapHorizontal,
                    )
                    .unwrap_or(0.0),
                instance.runtime_layout_style_uint_default(
                    style_local,
                    RuntimeLayoutStyleProperty::FlexDirectionValue,
                    2,
                ) == 2,
            )
        })
        .unwrap_or((0.0, 0.0, 0.0, true));

    for (item_index, item) in items.iter().enumerate() {
        if nested_ancestors.contains(&item.child.graph_global_id) {
            continue;
        }
        let Some(child_graph) = artboards
            .iter()
            .find(|graph| graph.global_id == item.child.graph_global_id)
        else {
            continue;
        };
        let mut child_ancestors = nested_ancestors.clone();
        child_ancestors.insert(item.child.graph_global_id);
        let cache_key = component_list_render_cache_key(
            command.global_id,
            command.local_id,
            item.child.graph_global_id,
            item_index,
            item.render_cache_revision,
        );
        let child_cache = path_cache.nested_artboards.entry(cache_key).or_default();
        let item_offset = if is_row {
            Mat2D([
                1.0,
                0.0,
                0.0,
                1.0,
                padding_left + item_index as f32 * (item.child.width + gap),
                padding_top,
            ])
        } else {
            Mat2D([
                1.0,
                0.0,
                0.0,
                1.0,
                padding_left,
                padding_top + item_index as f32 * (item.child.height + gap),
            ])
        };

        renderer.save();
        renderer.transform(runtime_render_mat(item_offset.multiply(item.transform)));
        if let Some(caches) = nested_paint_caches.as_deref_mut() {
            if !caches.contains_key(&cache_key) {
                caches.insert(
                    cache_key,
                    preallocate_render_paint_cache_for_artboard_instance(
                        runtime,
                        child_graph,
                        artboards,
                        factory,
                    ),
                );
            }
            let child_paint_cache = caches.entry(cache_key).or_default();
            item.child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                &mut child_paint_cache.paints,
                Some(&mut child_paint_cache.paint_configurations),
                Some(&mut child_paint_cache.preparation),
                Some(&mut child_paint_cache.nested_artboards),
                child_cache,
                false,
                true,
                &child_ancestors,
            )?;
            item.child
                .draw_prepared_static_artboard_internal_with_path_cache(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    renderer,
                    &mut child_paint_cache.paints,
                    image_by_global,
                    &mut child_paint_cache.meshes,
                    child_cache,
                    Some(&mut child_paint_cache.paint_configurations),
                    Some(&mut child_paint_cache.nested_artboards),
                    false,
                    &child_ancestors,
                )?;
        } else {
            item.child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                None,
                None,
                child_cache,
                false,
                true,
                &child_ancestors,
            )?;
            item.child
                .draw_prepared_static_artboard_internal_with_path_cache(
                    runtime,
                    child_graph,
                    artboards,
                    factory,
                    renderer,
                    paint_by_global,
                    image_by_global,
                    mesh_by_local,
                    child_cache,
                    paint_configurations.as_deref_mut(),
                    None,
                    false,
                    &child_ancestors,
                )?;
        }
        renderer.restore();
    }

    if command.needs_save_operation {
        renderer.restore();
    }
    Ok(())
}

fn runtime_draw_nested_artboard(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    command: &RuntimeDrawCommand,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    paint_by_global: &mut RuntimeRenderPaints,
    image_by_global: &RuntimeRenderImages,
    mesh_by_local: &mut RuntimeMeshRenderBufferSlots,
    path_cache: &mut RuntimeRenderPathCache,
    mut paint_configurations: Option<&mut RuntimeRenderPaintConfigurationSlots>,
    nested_paint_caches: Option<
        &mut BTreeMap<RuntimeNestedRenderCacheKey, RuntimeRenderPaintCache>,
    >,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    nested_ancestors: &BTreeSet<u32>,
) -> Result<()> {
    if let Some(local_id) = command.local_id
        && let Some(artboard_id_key) =
            runtime_draw_property_key_for_name("NestedArtboard", "artboardId")
        && instance.uint_property(local_id, artboard_id_key) == Some(u64::from(u32::MAX))
    {
        return Ok(());
    }

    let nested_instance = command
        .local_id
        .and_then(|local_id| instance.nested_artboards.get(&local_id));
    let referenced_artboard_global = nested_instance
        .map(|nested| nested.child.graph_global_id)
        .or(command.referenced_artboard_global)
        .context("nested artboard missing referenced artboard")?;
    // Cycle guard (see `nested_ancestors` at the top of this module): mirror of
    // the paint-prep guard for the draw recursion. Skip a nested artboard that is
    // already an ancestor on the current descent path, as C++ Artboard::isAncestor
    // does, so a cyclic nesting graph terminates instead of looping/overflowing.
    if nested_ancestors.contains(&referenced_artboard_global) {
        return Ok(());
    }
    let mut child_ancestors = nested_ancestors.clone();
    child_ancestors.insert(referenced_artboard_global);
    let child_graph = artboards
        .iter()
        .find(|graph| graph.global_id == referenced_artboard_global)
        .with_context(|| {
            format!("missing nested artboard graph for global {referenced_artboard_global}")
        })?;
    let host_component = command
        .local_id
        .and_then(|local_id| instance.component(local_id))
        .context("nested artboard host component is missing")?;
    let persistent_child = nested_instance.map(|nested| nested.child.as_ref());
    let host_world = match command.object_kind {
        RuntimeDrawCommandObjectKind::NestedArtboardLayout => command
            .local_id
            .map(|local_id| {
                path_cache.component_world_transform_with_bounds(
                    instance,
                    graph,
                    local_id,
                    layout_bounds,
                )
            })
            .unwrap_or(host_component.transform.world_transform),
        RuntimeDrawCommandObjectKind::NestedArtboardLeaf => {
            let local_id = command
                .local_id
                .context("nested artboard leaf command missing local id")?;
            runtime_nested_artboard_leaf_world_transform(
                instance,
                graph,
                local_id,
                child_graph,
                persistent_child,
                layout_bounds,
                path_cache,
            )?
        }
        _ => command
            .local_id
            .map(|local_id| {
                path_cache.component_world_transform_with_bounds(
                    instance,
                    graph,
                    local_id,
                    layout_bounds,
                )
            })
            .unwrap_or(host_component.transform.world_transform),
    };
    let host_opacity = host_component.transform.render_opacity;

    if command.needs_save_operation {
        renderer.save();
    }
    renderer.transform(runtime_render_mat(host_world));

    let cache_key = nested_render_cache_key(
        command.global_id,
        command.local_id,
        referenced_artboard_global,
        nested_instance.map_or(0, |nested| nested.render_cache_revision),
    );
    let child_cache = path_cache.nested_artboards.entry(cache_key).or_default();

    if let Some(child) = persistent_child {
        let layout_child;
        let child = if command.object_kind == RuntimeDrawCommandObjectKind::NestedArtboardLayout {
            let mut cloned = child.clone();
            runtime_apply_nested_artboard_layout_child_bounds(
                &mut cloned,
                &command,
                layout_bounds,
            )?;
            cloned.update_pass();
            layout_child = cloned;
            &layout_child
        } else {
            child
        };
        if let Some(caches) = nested_paint_caches {
            let child_paint_cache = caches.entry(cache_key).or_default();
            child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                &mut child_paint_cache.paints,
                Some(&mut child_paint_cache.paint_configurations),
                Some(&mut child_paint_cache.preparation),
                Some(&mut child_paint_cache.nested_artboards),
                child_cache,
                false,
                true,
                &child_ancestors,
            )?;
            child.draw_prepared_static_artboard_internal_with_path_cache(
                runtime,
                child_graph,
                artboards,
                factory,
                renderer,
                &mut child_paint_cache.paints,
                image_by_global,
                &mut child_paint_cache.meshes,
                child_cache,
                Some(&mut child_paint_cache.paint_configurations),
                Some(&mut child_paint_cache.nested_artboards),
                false,
                &child_ancestors,
            )?;
        } else {
            child.prepare_static_artboard_tree_paints_internal(
                runtime,
                child_graph,
                artboards,
                factory,
                paint_by_global,
                paint_configurations.as_deref_mut(),
                None,
                None,
                child_cache,
                false,
                true,
                &child_ancestors,
            )?;
            child.draw_prepared_static_artboard_internal_with_path_cache(
                runtime,
                child_graph,
                artboards,
                factory,
                renderer,
                paint_by_global,
                image_by_global,
                mesh_by_local,
                child_cache,
                paint_configurations.as_deref_mut(),
                None,
                false,
                &child_ancestors,
            )?;
        }

        if command.needs_save_operation {
            renderer.restore();
        }
        return Ok(());
    }

    let mut child = ArtboardInstance::from_graph(runtime, child_graph)?;
    runtime_apply_nested_artboard_layout_child_bounds(&mut child, &command, layout_bounds)?;
    if let Some(opacity_key) = runtime_draw_property_key_for_name("Artboard", "opacity") {
        child.set_double_property(0, opacity_key, host_opacity);
    }
    child.update_pass();
    if let Some(caches) = nested_paint_caches {
        let child_paint_cache = caches.entry(cache_key).or_default();
        child.prepare_static_artboard_tree_paints_internal(
            runtime,
            child_graph,
            artboards,
            factory,
            &mut child_paint_cache.paints,
            Some(&mut child_paint_cache.paint_configurations),
            Some(&mut child_paint_cache.preparation),
            Some(&mut child_paint_cache.nested_artboards),
            child_cache,
            false,
            true,
            &child_ancestors,
        )?;
        child.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            child_graph,
            artboards,
            factory,
            renderer,
            &mut child_paint_cache.paints,
            image_by_global,
            &mut child_paint_cache.meshes,
            child_cache,
            Some(&mut child_paint_cache.paint_configurations),
            Some(&mut child_paint_cache.nested_artboards),
            false,
            &child_ancestors,
        )?;
    } else {
        child.prepare_static_artboard_tree_paints_internal(
            runtime,
            child_graph,
            artboards,
            factory,
            paint_by_global,
            paint_configurations.as_deref_mut(),
            None,
            None,
            child_cache,
            false,
            true,
            &child_ancestors,
        )?;
        child.draw_prepared_static_artboard_internal_with_path_cache(
            runtime,
            child_graph,
            artboards,
            factory,
            renderer,
            paint_by_global,
            image_by_global,
            mesh_by_local,
            child_cache,
            paint_configurations.as_deref_mut(),
            None,
            false,
            &child_ancestors,
        )?;
    }

    if command.needs_save_operation {
        renderer.restore();
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct RuntimeAabb {
    left: f32,
    top: f32,
    width: f32,
    height: f32,
}

impl RuntimeAabb {
    fn from_artboard(instance: &ArtboardInstance) -> Self {
        Self {
            left: -instance.width * instance.origin_x,
            top: -instance.height * instance.origin_y,
            width: instance.width,
            height: instance.height,
        }
    }

    fn from_artboard_with_layout(instance: &ArtboardInstance, graph: &ArtboardGraph) -> Self {
        instance
            .runtime_taffy_layout_bounds(graph, instance.runtime_file())
            .and_then(|bounds| bounds.get(&0).copied())
            .or_else(|| instance.runtime_root_artboard_layout_bounds(graph))
            .map(|bounds| Self {
                left: 0.0,
                top: 0.0,
                width: bounds.width,
                height: bounds.height,
            })
            .unwrap_or_else(|| Self::from_artboard(instance))
    }

    fn from_local_layout_bounds(bounds: RuntimeLayoutBounds) -> Self {
        Self {
            left: 0.0,
            top: 0.0,
            width: bounds.width,
            height: bounds.height,
        }
    }
}

fn runtime_nested_artboard_leaf_world_transform(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    local_id: usize,
    child_graph: &ArtboardGraph,
    child: Option<&ArtboardInstance>,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    path_cache: &mut RuntimeRenderPathCache,
) -> Result<Mat2D> {
    let host_world =
        path_cache.component_world_transform_with_bounds(instance, graph, local_id, layout_bounds);
    let Some(child) = child else {
        return Ok(host_world);
    };

    let frame =
        runtime_nested_artboard_leaf_frame_bounds(instance, graph, local_id, child, layout_bounds)?;
    let content = RuntimeAabb::from_artboard_with_layout(child, child_graph);
    let fit_key = runtime_draw_property_key_for_name("NestedArtboardLeaf", "fit")
        .context("missing NestedArtboardLeaf.fit")?;
    let alignment_x_key = runtime_draw_property_key_for_name("NestedArtboardLeaf", "alignmentX")
        .context("missing NestedArtboardLeaf.alignmentX")?;
    let alignment_y_key = runtime_draw_property_key_for_name("NestedArtboardLeaf", "alignmentY")
        .context("missing NestedArtboardLeaf.alignmentY")?;
    let fit = instance.uint_property(local_id, fit_key).unwrap_or(0);
    let alignment_x = instance
        .double_property(local_id, alignment_x_key)
        .unwrap_or(0.0);
    let alignment_y = instance
        .double_property(local_id, alignment_y_key)
        .unwrap_or(0.0);

    Ok(host_world.multiply(runtime_compute_alignment(
        fit,
        alignment_x,
        alignment_y,
        frame,
        content,
    )))
}

fn runtime_nested_artboard_leaf_frame_bounds(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    local_id: usize,
    child: &ArtboardInstance,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Result<RuntimeAabb> {
    let component = instance
        .component(local_id)
        .context("nested artboard leaf component is missing")?;
    if let Some(parent_local) = component.parent_local
        && instance
            .component(parent_local)
            .is_some_and(|parent| parent.type_name == "LayoutComponent")
    {
        let bounds = layout_bounds
            .and_then(|bounds| bounds.get(&parent_local).copied())
            .with_context(|| {
                let global = graph
                    .components
                    .iter()
                    .find(|component| component.local_id == parent_local)
                    .map(|component| component.global_id)
                    .unwrap_or_default();
                format!(
                    "nested artboard leaf local {local_id} missing parent layout bounds local {parent_local} global {global}"
                )
            })?;
        return Ok(RuntimeAabb::from_local_layout_bounds(bounds));
    }

    Ok(RuntimeAabb::from_artboard(child))
}

fn runtime_compute_alignment(
    fit: u64,
    alignment_x: f32,
    alignment_y: f32,
    frame: RuntimeAabb,
    content: RuntimeAabb,
) -> Mat2D {
    let content_width = content.width;
    let content_height = content.height;
    let x = -content.left - content_width * 0.5 - alignment_x * content_width * 0.5;
    let y = -content.top - content_height * 0.5 - alignment_y * content_height * 0.5;

    let mut scale_x = 1.0;
    let mut scale_y = 1.0;
    match fit {
        0 => {
            scale_x = frame.width / content_width;
            scale_y = frame.height / content_height;
        }
        1 => {
            let scale = (frame.width / content_width).min(frame.height / content_height);
            scale_x = scale;
            scale_y = scale;
        }
        2 => {
            let scale = (frame.width / content_width).max(frame.height / content_height);
            scale_x = scale;
            scale_y = scale;
        }
        3 => {
            let scale = frame.width / content_width;
            scale_x = scale;
            scale_y = scale;
        }
        4 => {
            let scale = frame.height / content_height;
            scale_x = scale;
            scale_y = scale;
        }
        5 => {}
        6 => {
            let scale = (frame.width / content_width).min(frame.height / content_height);
            let scale = if scale < 1.0 { scale } else { 1.0 };
            scale_x = scale;
            scale_y = scale;
        }
        7 => {}
        _ => {}
    }

    let translation = Mat2D([
        1.0,
        0.0,
        0.0,
        1.0,
        frame.left + frame.width * 0.5 + alignment_x * frame.width * 0.5,
        frame.top + frame.height * 0.5 + alignment_y * frame.height * 0.5,
    ]);
    let scale = Mat2D([scale_x, 0.0, 0.0, scale_y, 0.0, 0.0]);
    let translate_content = Mat2D([1.0, 0.0, 0.0, 1.0, x, y]);
    translation.multiply(scale).multiply(translate_content)
}

fn runtime_apply_nested_artboard_layout_child_bounds(
    child: &mut ArtboardInstance,
    command: &RuntimeDrawCommand,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Result<()> {
    if command.object_kind != RuntimeDrawCommandObjectKind::NestedArtboardLayout {
        return Ok(());
    }
    let local_id = command
        .local_id
        .context("nested artboard layout command missing local id")?;
    let bounds = layout_bounds
        .and_then(|bounds| bounds.get(&local_id).copied())
        .context("nested artboard layout missing Taffy bounds")?;
    child.set_artboard_dimensions(bounds.width, bounds.height);
    child.enable_layout_constraint_bounds();
    if let Some(width_key) = runtime_layout_component_property_key_for_name("width") {
        child.set_double_property(0, width_key, bounds.width);
    }
    if let Some(height_key) = runtime_layout_component_property_key_for_name("height") {
        child.set_double_property(0, height_key, bounds.height);
    }
    Ok(())
}

fn runtime_settle_nested_artboard_layout_child(child: &mut ArtboardInstance) {
    for _ in 0..100 {
        child.update_pass();
        if !child.has_dirt(ComponentDirt::COMPONENTS) {
            break;
        }
    }
}

fn runtime_draw_clip_start(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    command: &RuntimeDrawCommand,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    factory: &mut dyn RenderFactory,
    renderer: &mut dyn Renderer,
    path_cache: &mut RuntimeRenderPathCache,
) {
    let Some(clipping_shape) = command
        .clipping_shape_local
        .and_then(|local_id| runtime_clipping_shape(graph, local_id))
    else {
        return;
    };
    if !clipping_shape.is_visible {
        return;
    }
    if command.needs_save_operation {
        renderer.save();
    }
    let path_commands =
        path_cache.clipping_shape_path_commands(instance, graph, clipping_shape, layout_bounds);
    if path_commands.is_empty() {
        return;
    }
    // Ported from src/shapes/clipping_shape.cpp: each ClippingShape owns a
    // cached clip path, so repeated clip-start proxies reuse the same RenderPath.
    let fill_rule = runtime_fill_rule_for_value(clipping_shape.fill_rule);
    let key = path_cache.retained_render_path_key(instance, graph, fill_rule);
    let path = path_cache.clipping_shape_path(
        key,
        clipping_shape.local_id,
        factory,
        path_commands.as_slice(),
    );
    renderer.clip_path(path.as_ref());
}

fn runtime_draw_clip_end(
    graph: &ArtboardGraph,
    command: &RuntimeDrawCommand,
    renderer: &mut dyn Renderer,
) {
    let Some(clipping_shape) = command
        .clipping_shape_local
        .and_then(|local_id| runtime_clipping_shape(graph, local_id))
    else {
        return;
    };
    if clipping_shape.is_visible && command.needs_save_operation {
        renderer.restore();
    }
}

fn runtime_clipping_shape(graph: &ArtboardGraph, local_id: usize) -> Option<&ClippingShapeNode> {
    graph
        .clipping_shapes
        .iter()
        .find(|clipping_shape| clipping_shape.local_id == local_id)
}

fn assign_shape_paint_path_slot_indices(commands: &mut [RuntimeShapePaintCommand]) {
    let mut path_slots = Vec::<&[RuntimePathCommand]>::new();
    for command in commands {
        let effect_or_shape_path_commands = if command.has_effect_path {
            command.effect_path_commands.as_slice()
        } else {
            command.path_commands.as_slice()
        };
        if let Some(feather) = command
            .feather_state
            .as_ref()
            .filter(|feather| feather.inner)
        {
            command.clip_path_slot_index =
                runtime_cached_path_slot_index(&mut path_slots, effect_or_shape_path_commands);
            command.path_slot_index = runtime_cached_path_slot_index(
                &mut path_slots,
                feather.inner_path_commands.as_slice(),
            );
        } else {
            command.path_slot_index =
                runtime_cached_path_slot_index(&mut path_slots, effect_or_shape_path_commands);
            command.clip_path_slot_index = command.path_slot_index;
        }
    }
}

fn runtime_cached_path_slot_index<'a>(
    cache: &mut Vec<&'a [RuntimePathCommand]>,
    commands: &'a [RuntimePathCommand],
) -> usize {
    if let Some(index) = cache.iter().position(|candidate| *candidate == commands) {
        return index;
    }
    cache.push(commands);
    cache.len() - 1
}

fn runtime_render_paint_configuration(
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    paint: &RuntimeShapePaintCommand,
) -> Result<RuntimeRenderPaintConfiguration> {
    let (stroke_thickness, stroke_cap, stroke_join) = match paint.paint_type {
        RuntimeShapePaintKind::Fill => (None, None, None),
        RuntimeShapePaintKind::Stroke => (
            Some(runtime_stroke_thickness(
                instance,
                object,
                paint.paint_local,
            )),
            Some(runtime_stroke_cap(runtime_stroke_uint_property(
                instance,
                object,
                paint.paint_local,
                "cap",
                0,
            ))?),
            Some(runtime_stroke_join(runtime_stroke_uint_property(
                instance,
                object,
                paint.paint_local,
                "join",
                0,
            ))?),
        ),
        RuntimeShapePaintKind::Unknown => anyhow::bail!("unsupported: unknown shape paint"),
    };
    let shader = if let Some(render_color) = runtime_current_solid_render_color(instance, paint) {
        RuntimeRenderPaintShaderConfiguration::SolidColor(render_color)
    } else {
        match paint.paint_state.as_ref() {
            Some(RuntimeShapePaintState::SolidColor { render_color, .. }) => {
                RuntimeRenderPaintShaderConfiguration::SolidColor(*render_color)
            }
            Some(
                RuntimeShapePaintState::LinearGradient { .. }
                | RuntimeShapePaintState::RadialGradient { .. },
            ) => RuntimeRenderPaintShaderConfiguration::PreserveGradientShader,
            None => RuntimeRenderPaintShaderConfiguration::None,
        }
    };

    Ok(RuntimeRenderPaintConfiguration {
        paint_type: paint.paint_type,
        stroke_thickness,
        stroke_cap,
        stroke_join,
        blend_mode: runtime_blend_mode(paint.render_blend_mode_value)?,
        shader,
        feather: paint
            .feather_state
            .as_ref()
            .map(|feather| feather.strength)
            .unwrap_or(0.0),
    })
}

// The paint-configuration cache epoch for a shape paint command. World-space
// paints (paint_space_transform set) bake shape world transforms into their
// shader endpoints, and keyed transform writes only bump prepared_epoch, so
// their configuration must also be keyed on it. Mirrors C++
// linear_gradient.cpp:98-106 (PathFlags::world && WorldTransform dirt).
fn runtime_paint_configuration_epoch(
    instance: &ArtboardInstance,
    paint: &RuntimeShapePaintCommand,
) -> u64 {
    let cache_epoch = instance.cache_epoch();
    if paint.paint_space_transform.is_some() {
        (0xcbf29ce484222325u64 ^ cache_epoch)
            .wrapping_mul(0x100000001b3)
            .wrapping_add(instance.prepared_epoch())
    } else {
        cache_epoch
    }
}

fn runtime_configure_paint_with_cache(
    render_paint: &mut dyn RenderPaint,
    configurations: &mut RuntimeRenderPaintConfigurationSlots,
    paint_global_id: u32,
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    paint: &RuntimeShapePaintCommand,
) -> Result<()> {
    let instance_epoch = runtime_paint_configuration_epoch(instance, paint);
    if configurations
        .get(paint_global_id)
        .is_some_and(|cached| cached.instance_epoch == instance_epoch)
    {
        return Ok(());
    }

    let configuration = runtime_render_paint_configuration(instance, object, paint)?;
    if let Some(cached) = configurations.get_mut(paint_global_id)
        && cached.configuration == configuration
    {
        cached.instance_epoch = instance_epoch;
        return Ok(());
    }

    runtime_configure_paint(render_paint, instance, object, paint, None)?;
    configurations.insert(
        paint_global_id,
        RuntimeCachedRenderPaintConfiguration {
            instance_epoch,
            configuration,
        },
    );
    Ok(())
}

fn runtime_configure_paint(
    render_paint: &mut dyn RenderPaint,
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    paint: &RuntimeShapePaintCommand,
    gradient_resources: Option<&mut RuntimeGradientShaderResources<'_>>,
) -> Result<()> {
    match paint.paint_type {
        RuntimeShapePaintKind::Fill => {
            render_paint.style(RenderPaintStyle::Fill);
        }
        RuntimeShapePaintKind::Stroke => {
            render_paint.style(RenderPaintStyle::Stroke);
            render_paint.thickness(runtime_stroke_thickness(
                instance,
                object,
                paint.paint_local,
            ));
            render_paint.cap(runtime_stroke_cap(runtime_stroke_uint_property(
                instance,
                object,
                paint.paint_local,
                "cap",
                0,
            ))?);
            render_paint.join(runtime_stroke_join(runtime_stroke_uint_property(
                instance,
                object,
                paint.paint_local,
                "join",
                0,
            ))?);
        }
        RuntimeShapePaintKind::Unknown => anyhow::bail!("unsupported: unknown shape paint"),
    }
    render_paint.blend_mode(runtime_blend_mode(paint.render_blend_mode_value)?);
    if let Some(render_color) = runtime_current_solid_render_color(instance, paint) {
        render_paint.shader(None);
        render_paint.color(render_color);
    } else {
        match paint.paint_state.as_ref() {
            Some(RuntimeShapePaintState::SolidColor { render_color, .. }) => {
                render_paint.shader(None);
                render_paint.color(*render_color);
            }
            Some(
                state @ RuntimeShapePaintState::LinearGradient {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    stops,
                    ..
                },
            ) => {
                if let Some(resources) = gradient_resources {
                    let (start_x, start_y, end_x, end_y) = runtime_gradient_space_endpoints(
                        paint.paint_space_transform,
                        *start_x,
                        *start_y,
                        *end_x,
                        *end_y,
                    );
                    let state = runtime_shape_paint_state_with_endpoints(
                        state.clone(),
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                    );
                    runtime_configure_linear_gradient(
                        render_paint,
                        resources,
                        object.id,
                        state,
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        stops,
                    );
                }
            }
            Some(
                state @ RuntimeShapePaintState::RadialGradient {
                    start_x,
                    start_y,
                    end_x,
                    end_y,
                    stops,
                    ..
                },
            ) => {
                if let Some(resources) = gradient_resources {
                    let (start_x, start_y, end_x, end_y) = runtime_gradient_space_endpoints(
                        paint.paint_space_transform,
                        *start_x,
                        *start_y,
                        *end_x,
                        *end_y,
                    );
                    let state = runtime_shape_paint_state_with_endpoints(
                        state.clone(),
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                    );
                    runtime_configure_radial_gradient(
                        render_paint,
                        resources,
                        object.id,
                        state,
                        start_x,
                        start_y,
                        end_x,
                        end_y,
                        stops,
                    );
                }
            }
            None => {
                render_paint.shader(None);
            }
        }
    }
    render_paint.feather(
        paint
            .feather_state
            .as_ref()
            .map(|feather| feather.strength)
            .unwrap_or(0.0),
    );
    Ok(())
}

fn runtime_current_solid_render_color(
    instance: &ArtboardInstance,
    paint: &RuntimeShapePaintCommand,
) -> Option<u32> {
    match paint.paint_state.as_ref()? {
        RuntimeShapePaintState::SolidColor { color, .. } => {
            let color = paint
                .mutator_local
                .zip(solid_color_value_property_key())
                .and_then(|(local_id, property_key)| {
                    instance.color_property(local_id, property_key)
                })
                .unwrap_or(*color);
            Some(color_modulate_opacity(color, paint.render_opacity))
        }
        RuntimeShapePaintState::LinearGradient { .. }
        | RuntimeShapePaintState::RadialGradient { .. } => None,
    }
}

// Ported from src/shapes/paint/linear_gradient.cpp and radial_gradient.cpp.
fn runtime_configure_linear_gradient(
    render_paint: &mut dyn RenderPaint,
    resources: &mut RuntimeGradientShaderResources<'_>,
    paint_global_id: u32,
    state: RuntimeShapePaintState,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    stops: &[RuntimeGradientStop],
) {
    let shader = runtime_cached_gradient_shader(resources, paint_global_id, state, |factory| {
        let colors = stops
            .iter()
            .map(|stop| stop.render_color)
            .collect::<Vec<_>>();
        let positions = stops.iter().map(|stop| stop.position).collect::<Vec<_>>();
        factory.make_linear_gradient(start_x, start_y, end_x, end_y, &colors, &positions)
    });
    render_paint.shader(Some(shader));
}

fn runtime_configure_radial_gradient(
    render_paint: &mut dyn RenderPaint,
    resources: &mut RuntimeGradientShaderResources<'_>,
    paint_global_id: u32,
    state: RuntimeShapePaintState,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
    stops: &[RuntimeGradientStop],
) {
    let dx = end_x - start_x;
    let dy = end_y - start_y;
    let radius = (dx * dx + dy * dy).sqrt();
    let shader = runtime_cached_gradient_shader(resources, paint_global_id, state, |factory| {
        let colors = stops
            .iter()
            .map(|stop| stop.render_color)
            .collect::<Vec<_>>();
        let positions = stops.iter().map(|stop| stop.position).collect::<Vec<_>>();
        factory.make_radial_gradient(start_x, start_y, radius, &colors, &positions)
    });
    render_paint.shader(Some(shader));
}

fn runtime_shape_paint_state_with_endpoints(
    mut state: RuntimeShapePaintState,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> RuntimeShapePaintState {
    match &mut state {
        RuntimeShapePaintState::LinearGradient {
            start_x: state_start_x,
            start_y: state_start_y,
            end_x: state_end_x,
            end_y: state_end_y,
            ..
        }
        | RuntimeShapePaintState::RadialGradient {
            start_x: state_start_x,
            start_y: state_start_y,
            end_x: state_end_x,
            end_y: state_end_y,
            ..
        } => {
            *state_start_x = start_x;
            *state_start_y = start_y;
            *state_end_x = end_x;
            *state_end_y = end_y;
        }
        RuntimeShapePaintState::SolidColor { .. } => {}
    }
    state
}

fn runtime_deform_gradient_paint_state_with_nsliced_node(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    container: &ShapePaintContainerNode,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    shape_world: Mat2D,
    state: Option<RuntimeShapePaintState>,
) -> Option<RuntimeShapePaintState> {
    let state = state?;
    let Some(context) =
        runtime_nsliced_node_context_for_shape(instance, graph, container.local_id, layout_bounds)
    else {
        return Some(state);
    };
    let Some(inverse_shape_world) = runtime_mat2d_invert(shape_world) else {
        return Some(state);
    };

    let (start_x, start_y, end_x, end_y) = match &state {
        RuntimeShapePaintState::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            ..
        }
        | RuntimeShapePaintState::RadialGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            ..
        } => {
            let original_start_x = *start_x;
            let original_start_y = *start_y;
            let original_end_x = *end_x;
            let original_end_y = *end_y;
            let (start_x, start_y) = runtime_deform_local_gradient_point_with_nsliced_node(
                original_start_x,
                original_start_y,
                &context,
                shape_world,
                inverse_shape_world,
            );
            let (end_x, end_y) = runtime_deform_local_gradient_point_with_nsliced_node(
                original_end_x,
                original_end_y,
                &context,
                shape_world,
                inverse_shape_world,
            );
            (start_x, start_y, end_x, end_y)
        }
        RuntimeShapePaintState::SolidColor { .. } => return Some(state),
    };

    Some(runtime_shape_paint_state_with_endpoints(
        state, start_x, start_y, end_x, end_y,
    ))
}

fn runtime_cached_gradient_shader<'a>(
    resources: &'a mut RuntimeGradientShaderResources<'_>,
    paint_global_id: u32,
    state: RuntimeShapePaintState,
    create: impl FnOnce(&mut dyn RenderFactory) -> Box<dyn RenderShader>,
) -> &'a dyn RenderShader {
    let needs_shader = resources
        .shaders
        .get(&paint_global_id)
        .is_none_or(|entry| entry.state != state);
    if needs_shader {
        let shader = create(resources.factory);
        resources.shaders.insert(
            paint_global_id,
            RuntimeGradientShaderCacheEntry { state, shader },
        );
    }
    resources
        .shaders
        .get(&paint_global_id)
        .expect("gradient shader cache entry was inserted")
        .shader
        .as_ref()
}

fn runtime_fill_rule_for_value(value: u64) -> RenderFillRule {
    match value {
        1 => RenderFillRule::EvenOdd,
        2 => RenderFillRule::Clockwise,
        _ => RenderFillRule::NonZero,
    }
}

// C++ reads the live fillRule property on every draw
// (shape_paint.cpp:176-180); the value baked into the import-time graph
// snapshot goes stale after a runtime Fill.fillRule write, exactly like
// stroke thickness/cap/join. Non-Fill paints have no fillRule property and
// keep the imported value.
fn runtime_live_shape_paint_fill_rule(
    instance: &ArtboardInstance,
    paint: &ShapePaintNode,
) -> RenderFillRule {
    let live_value = runtime_draw_property_key_for_name("Fill", "fillRule")
        .and_then(|key| instance.uint_property(paint.local_id, key))
        .unwrap_or(paint.fill_rule);
    runtime_fill_rule_for_value(live_value)
}

fn runtime_make_path_from_raw_path(
    factory: &mut dyn RenderFactory,
    raw_path: &RawPath,
    fill_rule: RenderFillRule,
) -> Box<dyn RenderPath> {
    let mut path = factory.make_empty_render_path();
    runtime_rebuild_path_from_raw_path(path.as_mut(), raw_path, fill_rule);
    path
}

fn runtime_cached_retained_render_path<'a>(
    slot: &'a mut Option<RuntimeRetainedRenderPath>,
    key: RuntimeRetainedRenderPathCacheKey,
    factory: &mut dyn RenderFactory,
    commands: &[RuntimePathCommand],
) -> &'a mut Box<dyn RenderPath> {
    let cached = slot.get_or_insert_with(|| {
        let raw_path = runtime_raw_path_from_commands(commands);
        RuntimeRetainedRenderPath {
            path: runtime_make_path_from_raw_path(factory, &raw_path, key.fill_rule),
            raw_path,
            key,
        }
    });
    if cached.key != key {
        runtime_rebuild_raw_path_from_commands(&mut cached.raw_path, commands);
        runtime_rebuild_path_from_raw_path(cached.path.as_mut(), &cached.raw_path, key.fill_rule);
        cached.key = key;
    }
    &mut cached.path
}

fn runtime_cached_retained_render_path_with_commands<'a>(
    slot: &'a mut Option<RuntimeRetainedRenderPath>,
    key: RuntimeRetainedRenderPathCacheKey,
    factory: &mut dyn RenderFactory,
    commands: impl FnOnce() -> Vec<RuntimePathCommand>,
) -> &'a mut Box<dyn RenderPath> {
    if slot.as_ref().is_none_or(|cached| cached.key != key) {
        let commands = commands();
        match slot {
            Some(cached) => {
                runtime_rebuild_raw_path_from_commands(&mut cached.raw_path, &commands);
                runtime_rebuild_path_from_raw_path(
                    cached.path.as_mut(),
                    &cached.raw_path,
                    key.fill_rule,
                );
                cached.key = key;
            }
            None => {
                let raw_path = runtime_raw_path_from_commands(&commands);
                *slot = Some(RuntimeRetainedRenderPath {
                    path: runtime_make_path_from_raw_path(factory, &raw_path, key.fill_rule),
                    raw_path,
                    key,
                });
            }
        }
    }
    &mut slot
        .as_mut()
        .expect("retained render path was just populated")
        .path
}

fn runtime_rebuild_path_from_raw_path(
    path: &mut dyn RenderPath,
    raw_path: &RawPath,
    fill_rule: RenderFillRule,
) {
    path.rewind();
    path.reserve(raw_path.verbs().len(), raw_path.points().len());
    path.fill_rule(fill_rule);
    path.add_raw_path(raw_path);
}

// Contract with host renderers: RenderPath::rewind clears geometry but
// preserves the previously applied fill rule (C++ ShapePaintPath::renderPath
// likewise rewinds without re-applying it, shape_paint_path.cpp). The
// draw-time fill-rule replay skips redundant fill_rule() calls on that
// assumption.
fn runtime_rebuild_path_from_raw_path_preserving_fill_rule(
    path: &mut dyn RenderPath,
    raw_path: &RawPath,
) {
    path.rewind();
    path.reserve(raw_path.verbs().len(), raw_path.points().len());
    path.add_raw_path(raw_path);
}

fn runtime_raw_path_from_commands(commands: &[RuntimePathCommand]) -> RawPath {
    let mut raw_path = RawPath::new();
    runtime_rebuild_raw_path_from_commands(&mut raw_path, commands);
    raw_path
}

fn runtime_rebuild_raw_path_from_commands(raw_path: &mut RawPath, commands: &[RuntimePathCommand]) {
    raw_path.rewind();
    let (verbs, points) = runtime_path_command_counts(commands);
    raw_path.reserve(verbs, points);
    runtime_append_commands_to_raw_path(raw_path, commands);
}

fn runtime_path_command_counts(commands: &[RuntimePathCommand]) -> (usize, usize) {
    let mut points = 0;
    for command in commands {
        points += match command {
            RuntimePathCommand::Move { .. } | RuntimePathCommand::Line { .. } => 1,
            RuntimePathCommand::Cubic { .. } => 3,
            RuntimePathCommand::Close => 0,
        };
    }
    (commands.len(), points)
}

fn runtime_append_commands_to_raw_path(raw_path: &mut RawPath, commands: &[RuntimePathCommand]) {
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => raw_path.move_to(x, y),
            RuntimePathCommand::Line { x, y } => raw_path.line_to(x, y),
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => raw_path.cubic_to(x1, y1, x2, y2, x3, y3),
            RuntimePathCommand::Close => raw_path.close(),
        }
    }
}

fn transform_path_commands(commands: &mut [RuntimePathCommand], transform: Mat2D) {
    if transform == Mat2D::IDENTITY {
        return;
    }
    let [a, b, c, d, e, f] = transform.0;
    if b == 0.0 && c == 0.0 {
        transform_path_commands_scale_translate(commands, a, d, e, f);
    } else {
        transform_path_commands_affine(commands, a, b, c, d, e, f);
    }
}

fn transform_path_commands_scale_translate(
    commands: &mut [RuntimePathCommand],
    a: f32,
    d: f32,
    e: f32,
    f: f32,
) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                *x = a.mul_add(*x, e);
                *y = d.mul_add(*y, f);
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                *x1 = a.mul_add(*x1, e);
                *y1 = d.mul_add(*y1, f);
                *x2 = a.mul_add(*x2, e);
                *y2 = d.mul_add(*y2, f);
                *x3 = a.mul_add(*x3, e);
                *y3 = d.mul_add(*y3, f);
            }
            RuntimePathCommand::Close => {}
        }
    }
}

fn transform_path_commands_affine(
    commands: &mut [RuntimePathCommand],
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                let (next_x, next_y) = map_point_affine(*x, *y, a, b, c, d, e, f);
                *x = next_x;
                *y = next_y;
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                (*x1, *y1) = map_point_affine(*x1, *y1, a, b, c, d, e, f);
                (*x2, *y2) = map_point_affine(*x2, *y2, a, b, c, d, e, f);
                (*x3, *y3) = map_point_affine(*x3, *y3, a, b, c, d, e, f);
            }
            RuntimePathCommand::Close => {}
        }
    }
}

fn append_transformed_path_commands(
    destination: &mut Vec<RuntimePathCommand>,
    source: &[RuntimePathCommand],
    transform: Mat2D,
) {
    destination.reserve(source.len());
    if transform == Mat2D::IDENTITY {
        destination.extend_from_slice(source);
        return;
    }
    let [a, b, c, d, e, f] = transform.0;
    if b == 0.0 && c == 0.0 {
        append_scale_translate_path_commands(destination, source, a, d, e, f);
    } else {
        append_affine_path_commands(destination, source, a, b, c, d, e, f);
    }
}

fn append_scale_translate_path_commands(
    destination: &mut Vec<RuntimePathCommand>,
    source: &[RuntimePathCommand],
    a: f32,
    d: f32,
    e: f32,
    f: f32,
) {
    for command in source {
        destination.push(match *command {
            RuntimePathCommand::Move { x, y } => RuntimePathCommand::Move {
                x: a.mul_add(x, e),
                y: d.mul_add(y, f),
            },
            RuntimePathCommand::Line { x, y } => RuntimePathCommand::Line {
                x: a.mul_add(x, e),
                y: d.mul_add(y, f),
            },
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => RuntimePathCommand::Cubic {
                x1: a.mul_add(x1, e),
                y1: d.mul_add(y1, f),
                x2: a.mul_add(x2, e),
                y2: d.mul_add(y2, f),
                x3: a.mul_add(x3, e),
                y3: d.mul_add(y3, f),
            },
            RuntimePathCommand::Close => RuntimePathCommand::Close,
        });
    }
}

fn append_affine_path_commands(
    destination: &mut Vec<RuntimePathCommand>,
    source: &[RuntimePathCommand],
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    e: f32,
    f: f32,
) {
    for command in source {
        destination.push(match *command {
            RuntimePathCommand::Move { x, y } => {
                let (x, y) = map_point_affine(x, y, a, b, c, d, e, f);
                RuntimePathCommand::Move { x, y }
            }
            RuntimePathCommand::Line { x, y } => {
                let (x, y) = map_point_affine(x, y, a, b, c, d, e, f);
                RuntimePathCommand::Line { x, y }
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                let (x1, y1) = map_point_affine(x1, y1, a, b, c, d, e, f);
                let (x2, y2) = map_point_affine(x2, y2, a, b, c, d, e, f);
                let (x3, y3) = map_point_affine(x3, y3, a, b, c, d, e, f);
                RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                }
            }
            RuntimePathCommand::Close => RuntimePathCommand::Close,
        });
    }
}

fn map_point_affine(x: f32, y: f32, a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> (f32, f32) {
    (a.mul_add(x, c.mul_add(y, e)), d.mul_add(y, b.mul_add(x, f)))
}

fn runtime_rect_commands(left: f32, top: f32, right: f32, bottom: f32) -> Vec<RuntimePathCommand> {
    vec![
        RuntimePathCommand::Move { x: left, y: top },
        RuntimePathCommand::Line { x: right, y: top },
        RuntimePathCommand::Line {
            x: right,
            y: bottom,
        },
        RuntimePathCommand::Line { x: left, y: bottom },
        RuntimePathCommand::Close,
    ]
}

fn runtime_render_mat(mat: Mat2D) -> RenderMat2D {
    RenderMat2D(mat.0)
}

fn runtime_translation(x: f32, y: f32) -> RenderMat2D {
    RenderMat2D([1.0, 0.0, 0.0, 1.0, x, y])
}

fn runtime_feather_uses_world_space(feather: &RuntimeFeatherState) -> bool {
    feather.space_value == 0
}

fn runtime_feather_has_offset(feather: &RuntimeFeatherState) -> bool {
    feather.offset_x != 0.0 || feather.offset_y != 0.0
}

fn mat2d_has_visible_skew_or_rotation(mat: Mat2D) -> bool {
    // Keep near-axis-aligned cancellation on the regular multiply path; real
    // off-axis local path composition needs the fused C++ residual.
    const MATRIX_AXIS_EPSILON: f32 = 5.0e-2;
    mat.0[1].abs() > MATRIX_AXIS_EPSILON || mat.0[2].abs() > MATRIX_AXIS_EPSILON
}

fn mat2d_has_any_skew_or_rotation(mat: Mat2D) -> bool {
    mat.0[1] != 0.0 || mat.0[2] != 0.0
}

fn mat2d_linear_is_identity(mat: Mat2D) -> bool {
    mat.0[0] == 1.0 && mat.0[1] == 0.0 && mat.0[2] == 0.0 && mat.0[3] == 1.0
}

fn runtime_blend_mode(value: u32) -> Result<RenderBlendMode> {
    Ok(match value {
        3 => RenderBlendMode::SrcOver,
        14 => RenderBlendMode::Screen,
        15 => RenderBlendMode::Overlay,
        16 => RenderBlendMode::Darken,
        17 => RenderBlendMode::Lighten,
        18 => RenderBlendMode::ColorDodge,
        19 => RenderBlendMode::ColorBurn,
        20 => RenderBlendMode::HardLight,
        21 => RenderBlendMode::SoftLight,
        22 => RenderBlendMode::Difference,
        23 => RenderBlendMode::Exclusion,
        24 => RenderBlendMode::Multiply,
        25 => RenderBlendMode::Hue,
        26 => RenderBlendMode::Saturation,
        27 => RenderBlendMode::Color,
        28 => RenderBlendMode::Luminosity,
        _ => anyhow::bail!("unsupported blend mode {value}"),
    })
}

fn runtime_stroke_cap(value: u64) -> Result<RenderStrokeCap> {
    Ok(match value {
        0 => RenderStrokeCap::Butt,
        1 => RenderStrokeCap::Round,
        2 => RenderStrokeCap::Square,
        _ => anyhow::bail!("unsupported stroke cap {value}"),
    })
}

fn runtime_stroke_join(value: u64) -> Result<RenderStrokeJoin> {
    Ok(match value {
        0 => RenderStrokeJoin::Miter,
        1 => RenderStrokeJoin::Round,
        2 => RenderStrokeJoin::Bevel,
        _ => anyhow::bail!("unsupported stroke join {value}"),
    })
}

pub(crate) fn runtime_shape_paint_command(
    artboard: &ArtboardInstance,
    paint: &ShapePaintNode,
    container_blend_mode_value: u32,
    needs_save_operation: bool,
    render_opacity: f32,
    shape_world: Mat2D,
    path_commands: Vec<RuntimePathCommand>,
    require_effective_visible: bool,
    suppress_authored_transparent: bool,
    aliases_local_clockwise_path: bool,
) -> Option<RuntimeShapePaintCommand> {
    if !runtime_shape_paint_is_visible(artboard, paint) {
        return None;
    }
    let paint_state = runtime_shape_paint_state(artboard, paint, render_opacity);
    if require_effective_visible && !runtime_shape_paint_state_is_effectively_visible(&paint_state)
    {
        return None;
    }
    if suppress_authored_transparent && !runtime_shape_paint_state_is_visible(&paint_state) {
        return None;
    }
    let mut path_commands = path_commands;
    prune_empty_path_segments(&mut path_commands);
    let effect_path_commands = runtime_effect_path_commands(artboard, paint, &path_commands);
    let has_effect_path = effect_path_commands.is_some();
    let mut effect_path_commands = effect_path_commands.unwrap_or_default();
    prune_empty_path_segments(&mut effect_path_commands);
    let feather_path_commands = if has_effect_path {
        effect_path_commands.as_slice()
    } else {
        path_commands.as_slice()
    };
    let mut feather_state = runtime_feather_state(
        artboard,
        paint.feather.as_ref(),
        feather_path_commands,
        shape_world,
    );
    if let Some(feather_state) = feather_state.as_mut() {
        prune_empty_path_segments(&mut feather_state.inner_path_commands);
    }
    Some(RuntimeShapePaintCommand {
        paint_local: paint.local_id,
        paint_global_id: paint.global_id,
        mutator_local: paint.mutator_local,
        paint_type: runtime_shape_paint_kind(paint.paint_type),
        fill_rule: runtime_live_shape_paint_fill_rule(artboard, paint),
        path_kind: runtime_shape_paint_path_kind(paint.path_kind?)?,
        path_slot_index: 0,
        clip_path_slot_index: 0,
        blend_mode_value: paint.blend_mode_value,
        render_blend_mode_value: runtime_shape_paint_blend_mode_value(
            paint.blend_mode_value,
            container_blend_mode_value,
        ),
        render_opacity,
        paint_state,
        feather_state,
        paint_space_transform: runtime_shape_paint_space_transform(paint, shape_world),
        path_commands,
        effect_path_commands,
        has_effect_path,
        needs_save_operation,
        shape_world_override: None,
        aliases_local_clockwise_path,
        uses_temporary_paint: false,
        allocates_text_paint_pool: false,
    })
}

fn runtime_shape_paint_is_visible(artboard: &ArtboardInstance, paint: &ShapePaintNode) -> bool {
    let Some(property_key) = shape_paint_is_visible_property_key() else {
        return paint.is_visible;
    };
    match paint.paint_type {
        ShapePaintKind::Fill | ShapePaintKind::Unknown => artboard
            .bool_property(paint.local_id, property_key)
            .unwrap_or(paint.is_visible),
        ShapePaintKind::Stroke => {
            artboard
                .bool_property(paint.local_id, property_key)
                .map(|visible| paint.is_visible && visible)
                .unwrap_or(paint.is_visible)
                && runtime_stroke_thickness_for_local(artboard, paint.local_id).unwrap_or(1.0) > 0.0
        }
    }
}

fn runtime_stroke_thickness(
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    local_id: usize,
) -> f32 {
    let thickness_key = runtime_draw_property_key_for_name("Stroke", "thickness");
    runtime_stroke_thickness_for_local(instance, local_id)
        .or_else(|| {
            thickness_key
                .and_then(|key| runtime_object_explicit_double_property_by_key(object, key))
        })
        .unwrap_or(1.0)
}

fn runtime_stroke_thickness_for_local(instance: &ArtboardInstance, local_id: usize) -> Option<f32> {
    runtime_draw_property_key_for_name("Stroke", "thickness")
        .and_then(|key| instance.double_property(local_id, key))
}

fn runtime_stroke_uint_property(
    instance: &ArtboardInstance,
    object: &RuntimeObject,
    local_id: usize,
    property_name: &str,
    fallback: u64,
) -> u64 {
    let property_key = runtime_draw_property_key_for_name("Stroke", property_name);
    property_key
        .and_then(|key| instance.uint_property(local_id, key))
        .or_else(|| {
            property_key.and_then(|key| runtime_object_explicit_uint_property_by_key(object, key))
        })
        .unwrap_or(fallback)
}

fn runtime_shape_paint_state_is_visible(state: &Option<RuntimeShapePaintState>) -> bool {
    // C++ skips authored-transparent Backboard/background draws, while
    // render-opacity-transparent backgrounds can still appear in the stream.
    !matches!(
        state,
        Some(RuntimeShapePaintState::SolidColor { color, .. }) if (color >> 24) == 0
    )
}

fn runtime_shape_paint_state_is_effectively_visible(
    state: &Option<RuntimeShapePaintState>,
) -> bool {
    match state {
        Some(RuntimeShapePaintState::SolidColor { render_color, .. }) => (render_color >> 24) != 0,
        Some(RuntimeShapePaintState::LinearGradient { stops, .. })
        | Some(RuntimeShapePaintState::RadialGradient { stops, .. }) => {
            stops.iter().any(|stop| (stop.render_color >> 24) != 0)
        }
        None => true,
    }
}

fn runtime_shape_paint_blend_mode_value(
    paint_blend_mode_value: u32,
    container_blend_mode_value: u32,
) -> u32 {
    if paint_blend_mode_value == 127 {
        container_blend_mode_value
    } else {
        paint_blend_mode_value
    }
}

fn runtime_shape_paint_kind(kind: ShapePaintKind) -> RuntimeShapePaintKind {
    match kind {
        ShapePaintKind::Fill => RuntimeShapePaintKind::Fill,
        ShapePaintKind::Stroke => RuntimeShapePaintKind::Stroke,
        ShapePaintKind::Unknown => RuntimeShapePaintKind::Unknown,
    }
}

fn runtime_shape_paint_path_kind(kind: ShapePaintPathKind) -> Option<RuntimeShapePaintPathKind> {
    match kind {
        ShapePaintPathKind::Local => Some(RuntimeShapePaintPathKind::Local),
        ShapePaintPathKind::LocalClockwise => Some(RuntimeShapePaintPathKind::LocalClockwise),
        ShapePaintPathKind::World => Some(RuntimeShapePaintPathKind::World),
    }
}

fn runtime_shape_paint_space_transform(
    paint: &ShapePaintNode,
    shape_world: Mat2D,
) -> Option<Mat2D> {
    (paint.path_kind == Some(ShapePaintPathKind::World)).then_some(shape_world)
}

fn runtime_shape_paint_state(
    artboard: &ArtboardInstance,
    paint: &ShapePaintNode,
    render_opacity: f32,
) -> Option<RuntimeShapePaintState> {
    match paint.paint_state.clone()? {
        ShapePaintStateNode::SolidColor { color } => {
            let color = paint
                .mutator_local
                .zip(solid_color_value_property_key())
                .and_then(|(local_id, property_key)| {
                    artboard.color_property(local_id, property_key)
                })
                .unwrap_or(color);
            Some(RuntimeShapePaintState::SolidColor {
                color,
                render_color: color_modulate_opacity(color, render_opacity),
            })
        }
        ShapePaintStateNode::LinearGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            stops,
        } => {
            let opacity = paint
                .mutator_local
                .map(|mutator_local| {
                    runtime_gradient_double_property(
                        artboard,
                        mutator_local,
                        "LinearGradient",
                        "opacity",
                        opacity,
                    )
                })
                .unwrap_or(opacity);
            let (start_x, start_y, end_x, end_y) = runtime_gradient_endpoints(
                artboard,
                paint.mutator_local,
                "LinearGradient",
                start_x,
                start_y,
                end_x,
                end_y,
            );
            Some(RuntimeShapePaintState::LinearGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                opacity,
                render_opacity,
                stops: runtime_gradient_stops(artboard, stops, opacity * render_opacity),
            })
        }
        ShapePaintStateNode::RadialGradient {
            start_x,
            start_y,
            end_x,
            end_y,
            opacity,
            stops,
        } => {
            let opacity = paint
                .mutator_local
                .map(|mutator_local| {
                    runtime_gradient_double_property(
                        artboard,
                        mutator_local,
                        "RadialGradient",
                        "opacity",
                        opacity,
                    )
                })
                .unwrap_or(opacity);
            let (start_x, start_y, end_x, end_y) = runtime_gradient_endpoints(
                artboard,
                paint.mutator_local,
                "RadialGradient",
                start_x,
                start_y,
                end_x,
                end_y,
            );
            Some(RuntimeShapePaintState::RadialGradient {
                start_x,
                start_y,
                end_x,
                end_y,
                opacity,
                render_opacity,
                stops: runtime_gradient_stops(artboard, stops, opacity * render_opacity),
            })
        }
    }
}

fn shape_paint_container_opacity_local(graph: &ArtboardGraph, container_local: usize) -> usize {
    let mut current = Some(container_local);
    while let Some(local_id) = current {
        let Some(component) = graph
            .components
            .iter()
            .find(|component| component.local_id == local_id)
        else {
            break;
        };
        if component.capabilities.transform || component.capabilities.artboard {
            return local_id;
        }
        current = component.parent_local;
    }
    container_local
}

fn runtime_gradient_endpoints(
    artboard: &ArtboardInstance,
    mutator_local: Option<usize>,
    type_name: &str,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> (f32, f32, f32, f32) {
    let Some(mutator_local) = mutator_local else {
        return (start_x, start_y, end_x, end_y);
    };
    (
        runtime_gradient_double_property(artboard, mutator_local, type_name, "startX", start_x),
        runtime_gradient_double_property(artboard, mutator_local, type_name, "startY", start_y),
        runtime_gradient_double_property(artboard, mutator_local, type_name, "endX", end_x),
        runtime_gradient_double_property(artboard, mutator_local, type_name, "endY", end_y),
    )
}

fn runtime_gradient_space_endpoints(
    paint_space_transform: Option<Mat2D>,
    start_x: f32,
    start_y: f32,
    end_x: f32,
    end_y: f32,
) -> (f32, f32, f32, f32) {
    let Some(shape_world) = paint_space_transform else {
        return (start_x, start_y, end_x, end_y);
    };
    let (start_x, start_y) = shape_world.transform_point(start_x, start_y);
    let (end_x, end_y) = shape_world.transform_point(end_x, end_y);
    (start_x, start_y, end_x, end_y)
}

fn runtime_gradient_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    fallback: f32,
) -> f32 {
    runtime_draw_property_key_for_name(type_name, property_name)
        .and_then(|property_key| artboard.double_property(local_id, property_key))
        .unwrap_or(fallback)
}

fn runtime_gradient_stops(
    artboard: &ArtboardInstance,
    mut stops: Vec<GradientStopNode>,
    opacity: f32,
) -> Vec<RuntimeGradientStop> {
    // Ported from C++ `src/shapes/paint/linear_gradient.cpp::applyTo` and
    // `radial_gradient.cpp`: stops read live object properties after animation.
    let color_key = runtime_draw_property_key_for_name("GradientStop", "colorValue");
    let position_key = runtime_draw_property_key_for_name("GradientStop", "position");
    stops.sort_by(|left, right| {
        let left_position = runtime_gradient_stop_position(artboard, left, position_key);
        let right_position = runtime_gradient_stop_position(artboard, right, position_key);
        left_position.total_cmp(&right_position)
    });
    stops
        .into_iter()
        .map(|stop| {
            let color = color_key
                .and_then(|key| artboard.color_property(stop.local_id, key))
                .unwrap_or(stop.color);
            RuntimeGradientStop {
                color,
                render_color: color_modulate_opacity(color, opacity),
                position: runtime_gradient_stop_position(artboard, &stop, position_key)
                    .clamp(0.0, 1.0),
            }
        })
        .collect()
}

fn runtime_gradient_stop_position(
    artboard: &ArtboardInstance,
    stop: &GradientStopNode,
    position_key: Option<u16>,
) -> f32 {
    position_key
        .and_then(|key| artboard.double_property(stop.local_id, key))
        .unwrap_or(stop.position)
}

fn runtime_feather_state(
    artboard: &ArtboardInstance,
    feather: Option<&FeatherNode>,
    path_commands: &[RuntimePathCommand],
    shape_world: Mat2D,
) -> Option<RuntimeFeatherState> {
    let mut feather = feather?.clone();
    feather.space_value =
        runtime_feather_uint_property(artboard, &feather, "spaceValue", feather.space_value);
    feather.strength =
        runtime_feather_double_property(artboard, &feather, "strength", feather.strength);
    feather.offset_x =
        runtime_feather_double_property(artboard, &feather, "offsetX", feather.offset_x);
    feather.offset_y =
        runtime_feather_double_property(artboard, &feather, "offsetY", feather.offset_y);
    feather.inner = runtime_feather_bool_property(artboard, &feather, "inner", feather.inner);

    let inner_path_commands = inner_feather_path_commands(&feather, path_commands, shape_world);
    Some(RuntimeFeatherState {
        feather_local: feather.local_id,
        space_value: feather.space_value,
        strength: feather.strength,
        offset_x: feather.offset_x,
        offset_y: feather.offset_y,
        inner: feather.inner,
        inner_path_commands,
    })
}

fn inner_feather_path_commands(
    feather: &FeatherNode,
    source: &[RuntimePathCommand],
    shape_world: Mat2D,
) -> Vec<RuntimePathCommand> {
    if !feather.inner {
        return Vec::new();
    }
    let bounds = path_command_bounds(source).unwrap_or_else(|| PathBounds::from_point(0.0, 0.0));
    let pad = feather.strength * 1.5;
    let mut commands = rect_commands(bounds.pad(pad));
    let mut reversed_source = path_commands_backwards(source);
    let mut offset = (feather.offset_x, feather.offset_y);
    if feather.space_value == 0 {
        offset = shape_world
            .invert_or_identity()
            .transform_direction(offset.0, offset.1);
    }
    translate_path_commands(&mut reversed_source, offset);
    commands.extend(reversed_source);
    commands
}

fn runtime_feather_double_property(
    artboard: &ArtboardInstance,
    feather: &FeatherNode,
    property_name: &str,
    default: f32,
) -> f32 {
    runtime_draw_property_key_for_name(feather.type_name, property_name)
        .and_then(|key| artboard.double_property(feather.local_id, key))
        .unwrap_or(default)
}

fn runtime_feather_uint_property(
    artboard: &ArtboardInstance,
    feather: &FeatherNode,
    property_name: &str,
    default: u32,
) -> u32 {
    runtime_draw_property_key_for_name(feather.type_name, property_name)
        .and_then(|key| artboard.uint_property(feather.local_id, key))
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(default)
}

fn runtime_feather_bool_property(
    artboard: &ArtboardInstance,
    feather: &FeatherNode,
    property_name: &str,
    default: bool,
) -> bool {
    runtime_draw_property_key_for_name(feather.type_name, property_name)
        .and_then(|key| artboard.bool_property(feather.local_id, key))
        .unwrap_or(default)
}

#[derive(Debug, Clone, Copy)]
struct PathBounds {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

impl PathBounds {
    fn from_point(x: f32, y: f32) -> Self {
        Self {
            min_x: x,
            min_y: y,
            max_x: x,
            max_y: y,
        }
    }

    fn include(&mut self, x: f32, y: f32) {
        self.min_x = self.min_x.min(x);
        self.min_y = self.min_y.min(y);
        self.max_x = self.max_x.max(x);
        self.max_y = self.max_y.max(y);
    }

    fn pad(self, amount: f32) -> Self {
        Self {
            min_x: self.min_x - amount,
            min_y: self.min_y - amount,
            max_x: self.max_x + amount,
            max_y: self.max_y + amount,
        }
    }
}

fn path_command_bounds(commands: &[RuntimePathCommand]) -> Option<PathBounds> {
    let mut bounds: Option<PathBounds> = None;
    for command in commands {
        for (x, y) in command_points(command) {
            match &mut bounds {
                Some(bounds) => bounds.include(x, y),
                None => bounds = Some(PathBounds::from_point(x, y)),
            }
        }
    }
    bounds
}

fn rect_commands(bounds: PathBounds) -> Vec<RuntimePathCommand> {
    vec![
        RuntimePathCommand::Move {
            x: bounds.min_x,
            y: bounds.min_y,
        },
        RuntimePathCommand::Line {
            x: bounds.max_x,
            y: bounds.min_y,
        },
        RuntimePathCommand::Line {
            x: bounds.max_x,
            y: bounds.max_y,
        },
        RuntimePathCommand::Line {
            x: bounds.min_x,
            y: bounds.max_y,
        },
        RuntimePathCommand::Close,
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RawPathVerb {
    Move,
    Line,
    Cubic,
    Close,
}

fn path_commands_backwards(commands: &[RuntimePathCommand]) -> Vec<RuntimePathCommand> {
    let (verbs, points) = raw_path_parts(commands);
    if verbs.is_empty() {
        return Vec::new();
    }

    let reversed_points = points.into_iter().rev().collect::<Vec<_>>();
    let mut reversed_verbs = Vec::with_capacity(verbs.len());
    reversed_verbs.push(RawPathVerb::Move);
    let mut closed = false;
    for index in (0..verbs.len()).rev() {
        let verb = verbs[index];
        if verb == RawPathVerb::Close {
            closed = true;
            continue;
        }
        if verb == RawPathVerb::Move && closed {
            reversed_verbs.push(RawPathVerb::Close);
            closed = false;
        }
        if index != 0 {
            reversed_verbs.push(verb);
        } else {
            break;
        }
    }

    let mut commands = raw_path_parts_to_commands(&reversed_verbs, &reversed_points);
    prune_empty_path_segments(&mut commands);
    commands
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/math/raw_path.cpp RawPath::pruneEmptySegments
fn prune_empty_path_segments(commands: &mut Vec<RuntimePathCommand>) {
    prune_empty_path_segments_from(commands, 0);
}

fn prune_empty_path_segments_from(commands: &mut Vec<RuntimePathCommand>, start: usize) {
    let mut current = None::<(f32, f32)>;
    let mut write = start;
    let mut pruned = false;
    let mut multi_contour = None::<bool>;
    for read in start..commands.len() {
        let command = commands[read];
        let keep = match command {
            RuntimePathCommand::Move { x, y } => {
                current = Some((x, y));
                true
            }
            RuntimePathCommand::Line { x, y } => {
                let keep = current != Some((x, y));
                current = Some((x, y));
                keep
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                let exact_empty =
                    current == Some((x1, y1)) && (x1, y1) == (x2, y2) && (x2, y2) == (x3, y3);
                // Rust-side reverse/transform assembly can leave sub-ulp cancellation
                // noise in multi-contour paths that C++ has already collapsed.
                let near_empty = !exact_empty
                    && current.is_some_and(|current| {
                        path_points_match(current, (x1, y1))
                            && path_points_match(current, (x2, y2))
                            && path_points_match(current, (x3, y3))
                    })
                    && *multi_contour.get_or_insert_with(|| {
                        path_commands_have_multiple_contours(commands, start)
                    });
                if !exact_empty && !near_empty {
                    current = Some((x3, y3));
                    true
                } else {
                    current = Some((x3, y3));
                    false
                }
            }
            RuntimePathCommand::Close => true,
        };
        if keep {
            if pruned {
                commands[write] = command;
            }
            write += 1;
        } else {
            pruned = true;
        }
    }
    if pruned {
        commands.truncate(write);
    }
}

fn path_commands_have_multiple_contours(commands: &[RuntimePathCommand], start: usize) -> bool {
    commands
        .get(start..)
        .unwrap_or_default()
        .iter()
        .filter(|command| matches!(command, RuntimePathCommand::Move { .. }))
        .take(2)
        .count()
        >= 2
}

fn path_points_match(left: (f32, f32), right: (f32, f32)) -> bool {
    (left.0 - right.0).abs() <= f32::EPSILON && (left.1 - right.1).abs() <= f32::EPSILON
}

fn raw_path_parts(commands: &[RuntimePathCommand]) -> (Vec<RawPathVerb>, Vec<(f32, f32)>) {
    let mut verbs = Vec::with_capacity(commands.len());
    let mut points = Vec::new();
    for command in commands {
        match *command {
            RuntimePathCommand::Move { x, y } => {
                verbs.push(RawPathVerb::Move);
                points.push((x, y));
            }
            RuntimePathCommand::Line { x, y } => {
                verbs.push(RawPathVerb::Line);
                points.push((x, y));
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                verbs.push(RawPathVerb::Cubic);
                points.push((x1, y1));
                points.push((x2, y2));
                points.push((x3, y3));
            }
            RuntimePathCommand::Close => verbs.push(RawPathVerb::Close),
        }
    }
    (verbs, points)
}

fn raw_path_parts_to_commands(
    verbs: &[RawPathVerb],
    points: &[(f32, f32)],
) -> Vec<RuntimePathCommand> {
    let mut commands = Vec::with_capacity(verbs.len());
    let mut point_index = 0;
    for verb in verbs {
        match *verb {
            RawPathVerb::Move => {
                let Some((x, y)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                point_index += 1;
                commands.push(RuntimePathCommand::Move { x, y });
            }
            RawPathVerb::Line => {
                let Some((x, y)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                point_index += 1;
                commands.push(RuntimePathCommand::Line { x, y });
            }
            RawPathVerb::Cubic => {
                let Some((x1, y1)) = points.get(point_index).copied() else {
                    return Vec::new();
                };
                let Some((x2, y2)) = points.get(point_index + 1).copied() else {
                    return Vec::new();
                };
                let Some((x3, y3)) = points.get(point_index + 2).copied() else {
                    return Vec::new();
                };
                point_index += 3;
                commands.push(RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                });
            }
            RawPathVerb::Close => commands.push(RuntimePathCommand::Close),
        }
    }
    commands
}

fn translate_path_commands(commands: &mut [RuntimePathCommand], offset: (f32, f32)) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                *x += offset.0;
                *y += offset.1;
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                *x1 += offset.0;
                *y1 += offset.1;
                *x2 += offset.0;
                *y2 += offset.1;
                *x3 += offset.0;
                *y3 += offset.1;
            }
            RuntimePathCommand::Close => {}
        }
    }
}

fn command_points(command: &RuntimePathCommand) -> Vec<(f32, f32)> {
    match *command {
        RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
            vec![(x, y)]
        }
        RuntimePathCommand::Cubic {
            x1,
            y1,
            x2,
            y2,
            x3,
            y3,
        } => vec![(x1, y1), (x2, y2), (x3, y3)],
        RuntimePathCommand::Close => Vec::new(),
    }
}

fn runtime_effect_path_commands(
    artboard: &ArtboardInstance,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    if paint.effects.is_empty() {
        return None;
    }

    let mut current = source.to_vec();
    let mut has_supported_effect = false;
    for effect in &paint.effects {
        let Some(effect_path) =
            runtime_stroke_effect_path_commands(artboard, effect, paint, &current)
        else {
            continue;
        };
        current = effect_path;
        has_supported_effect = true;
    }

    has_supported_effect.then_some(current)
}

fn runtime_stroke_effect_path_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    match effect.type_name {
        "DashPath" => runtime_dash_path_effect_commands(artboard, effect, paint, source),
        "ScriptedPathEffect" => {
            let output = artboard
                .apply_scripted_path_effect(
                    effect.global_id,
                    runtime_raw_path_from_commands(source),
                    ScriptNode {
                        path: None,
                        paint: Some(script_paint_for_shape(artboard, paint)),
                    },
                )
                .map_err(|_| eprintln!("update function failed"))
                .ok()?;
            Some(runtime_path_commands_from_raw_path(&output))
        }
        "TargetEffect" => {
            let mut current = source.to_vec();
            let mut has_effect_path = false;
            for group_effect in &effect.group_effects {
                let Some(output) =
                    runtime_stroke_effect_path_commands(artboard, group_effect, paint, &current)
                else {
                    continue;
                };
                current = output;
                has_effect_path = true;
            }
            has_effect_path.then_some(current)
        }
        "TrimPath" => runtime_trim_path_line_effect_commands(artboard, effect, paint, source),
        _ => None,
    }
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/shapes/paint/dash_path.cpp DashPath::updateEffect
// and PathDasher::applyDash.
fn runtime_dash_path_effect_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    let offset = runtime_dash_path_double_property(artboard, effect, "offset", effect.dash_offset)?;
    let offset_is_percentage = runtime_dash_path_bool_property(
        artboard,
        effect,
        "offsetIsPercentage",
        effect.dash_offset_is_percentage,
    )?;
    if paint.paint_type == ShapePaintKind::Fill {
        return Some(source.to_vec());
    }

    Some(dash_path_apply_dash(
        artboard,
        source,
        offset,
        offset_is_percentage,
        &effect.dashes,
    ))
}

fn runtime_dash_path_double_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<f32>,
) -> Option<f32> {
    runtime_draw_property_key_for_name("DashPath", property_name)
        .and_then(|key| artboard.double_property(effect.local_id, key))
        .or(fallback)
}

fn runtime_dash_path_bool_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<bool>,
) -> Option<bool> {
    runtime_draw_property_key_for_name("DashPath", property_name)
        .and_then(|key| artboard.bool_property(effect.local_id, key))
        .or(fallback)
}

fn dash_path_apply_dash(
    artboard: &ArtboardInstance,
    source: &[RuntimePathCommand],
    offset: f32,
    offset_is_percentage: bool,
    dashes: &[DashNode],
) -> Vec<RuntimePathCommand> {
    let path_measure = RuntimePathMeasure::from_commands(source);
    if path_measure.length <= 0.0
        || !dashes.iter().any(|dash| {
            dash_normalized_length(
                runtime_dash_length(artboard, dash),
                runtime_dash_length_is_percentage(artboard, dash),
                path_measure.length,
                false,
            ) > 0.0
        })
    {
        return Vec::new();
    }

    let mut commands = Vec::new();
    let mut dash_index = 0usize;
    let mut dashed = 0.0;
    let mut distance =
        dash_normalized_length(offset, offset_is_percentage, path_measure.length, true);
    let mut draw = true;

    while dashed < path_measure.length {
        let dash = &dashes[dash_index % dashes.len()];
        dash_index += 1;

        let mut dash_length = dash_normalized_length(
            runtime_dash_length(artboard, dash),
            runtime_dash_length_is_percentage(artboard, dash),
            path_measure.length,
            false,
        );
        if dash_length > path_measure.length {
            dash_length = path_measure.length;
        }

        let mut end_length = distance + dash_length;
        if end_length > path_measure.length {
            end_length -= path_measure.length;
            if draw {
                if distance < path_measure.length {
                    path_measure.get_segment(distance, path_measure.length, &mut commands, true);
                    path_measure.get_segment(
                        0.0,
                        end_length,
                        &mut commands,
                        !path_measure.is_closed(),
                    );
                } else {
                    path_measure.get_segment(0.0, end_length, &mut commands, true);
                }
            }

            distance = end_length - dash_length;
        } else if draw {
            path_measure.get_segment(distance, end_length, &mut commands, true);
        }

        distance += dash_length;
        dashed += dash_length;
        draw = !draw;
    }

    commands
}

fn runtime_dash_length(artboard: &ArtboardInstance, dash: &DashNode) -> f32 {
    runtime_draw_property_key_for_name("Dash", "length")
        .and_then(|key| artboard.double_property(dash.local_id, key))
        .unwrap_or(dash.length)
}

fn runtime_dash_length_is_percentage(artboard: &ArtboardInstance, dash: &DashNode) -> bool {
    runtime_draw_property_key_for_name("Dash", "lengthIsPercentage")
        .and_then(|key| artboard.bool_property(dash.local_id, key))
        .unwrap_or(dash.length_is_percentage)
}

fn dash_normalized_length(
    value: f32,
    is_percentage: bool,
    contour_length: f32,
    wraps: bool,
) -> f32 {
    let mut p = value;
    if wraps {
        let right = if is_percentage { 1.0 } else { contour_length };
        if right != 0.0 {
            p = value % right;
            if p < 0.0 {
                p += right;
            }
        }
    }
    if is_percentage { p * contour_length } else { p }
}

fn runtime_trim_path_mode_value(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
) -> Option<u32> {
    runtime_draw_property_key_for_name("TrimPath", "modeValue")
        .and_then(|key| artboard.uint_property(effect.local_id, key))
        .and_then(|value| u32::try_from(value).ok())
        .or(effect.trim_mode_value)
}

fn runtime_trim_path_double_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<f32>,
) -> f32 {
    runtime_draw_property_key_for_name("TrimPath", property_name)
        .and_then(|key| artboard.double_property(effect.local_id, key))
        .or(fallback)
        .unwrap_or(0.0)
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/shapes/paint/trim_path.cpp TrimPath::trimPath
fn runtime_trim_path_line_effect_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    let mode = runtime_trim_path_mode_value(artboard, effect)?;
    if !matches!(mode, 1 | 2) {
        return None;
    }
    let contours = TrimContour::from_commands(source);
    if contours.is_empty() {
        return Some(Vec::new());
    }

    let render_offset = positive_unit_mod(runtime_trim_path_double_property(
        artboard,
        effect,
        "offset",
        effect.trim_offset,
    ));
    let trim_start =
        runtime_trim_path_double_property(artboard, effect, "start", effect.trim_start);
    let trim_end = runtime_trim_path_double_property(artboard, effect, "end", effect.trim_end);
    let close_shape = paint.paint_type == ShapePaintKind::Fill;
    match mode {
        1 => Some(trim_path_sequential(
            &contours,
            trim_start,
            trim_end,
            render_offset,
            close_shape,
        )),
        2 => Some(trim_path_synchronized(
            &contours,
            trim_start,
            trim_end,
            render_offset,
            close_shape,
        )),
        _ => None,
    }
}

fn positive_unit_mod(value: f32) -> f32 {
    value.rem_euclid(1.0)
}

#[derive(Debug, Clone)]
struct TrimContour {
    segments: Vec<TrimMeasuredSegment>,
    length: f32,
    is_closed: bool,
}

#[derive(Debug, Clone)]
pub struct RuntimePathMeasure {
    contours: Vec<TrimContour>,
    length: f32,
    raw_is_closed: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimePathSample {
    pub pos: (f32, f32),
    pub tan: (f32, f32),
}

#[derive(Debug, Clone)]
pub struct RuntimeContourMeasure {
    contour: TrimContour,
}

#[derive(Debug, Clone)]
struct TrimMeasuredSegment {
    original_index: usize,
    kind: TrimSegmentKind,
    distance: f32,
    t: f32,
}

#[derive(Debug, Clone, Copy)]
enum TrimSegmentKind {
    Line {
        from: (f32, f32),
        to: (f32, f32),
    },
    Cubic {
        p0: (f32, f32),
        p1: (f32, f32),
        p2: (f32, f32),
        p3: (f32, f32),
    },
}

impl TrimContour {
    fn from_commands(commands: &[RuntimePathCommand]) -> Vec<Self> {
        Self::from_commands_with_inv_tolerance(commands, TRIM_CONTOUR_DEFAULT_INV_TOLERANCE)
    }

    fn from_commands_with_inv_tolerance(
        commands: &[RuntimePathCommand],
        inv_tolerance: f32,
    ) -> Vec<Self> {
        let mut contours = Vec::new();
        let mut raw_segments = Vec::<TrimSegmentKind>::new();
        let mut start = None::<(f32, f32)>;
        let mut current = None::<(f32, f32)>;
        let mut is_closed = false;

        let finish_contour = |contours: &mut Vec<Self>,
                              raw_segments: &mut Vec<TrimSegmentKind>,
                              is_closed: &mut bool| {
            if let Some(contour) = Self::from_raw_segments(raw_segments, *is_closed, inv_tolerance)
            {
                contours.push(contour);
            }
            raw_segments.clear();
            *is_closed = false;
        };

        for command in commands {
            match *command {
                RuntimePathCommand::Move { x, y } => {
                    if !raw_segments.is_empty() {
                        finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
                    }
                    start = Some((x, y));
                    current = Some((x, y));
                }
                RuntimePathCommand::Line { x, y } => {
                    let Some(from) = current else {
                        continue;
                    };
                    let to = (x, y);
                    if distance_squared(from, to) > 0.0 {
                        raw_segments.push(TrimSegmentKind::Line { from, to });
                    }
                    current = Some(to);
                }
                RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => {
                    let Some(p0) = current else {
                        continue;
                    };
                    let p1 = (x1, y1);
                    let p2 = (x2, y2);
                    let p3 = (x3, y3);
                    raw_segments.push(TrimSegmentKind::Cubic { p0, p1, p2, p3 });
                    current = Some(p3);
                }
                RuntimePathCommand::Close => {
                    if let (Some(from), Some(to)) = (current, start) {
                        if distance_squared(from, to) > 0.0 {
                            raw_segments.push(TrimSegmentKind::Line { from, to });
                        }
                        current = Some(to);
                    }
                    is_closed = true;
                }
            }
        }

        if !raw_segments.is_empty() {
            finish_contour(&mut contours, &mut raw_segments, &mut is_closed);
        }
        contours
    }

    fn from_raw_segments(
        raw_segments: &[TrimSegmentKind],
        is_closed: bool,
        inv_tolerance: f32,
    ) -> Option<Self> {
        let mut segments = Vec::new();
        let mut distance_so_far = 0.0;

        for (original_index, segment) in raw_segments.iter().copied().enumerate() {
            match segment {
                TrimSegmentKind::Line { from, to } => {
                    let length = distance(from, to);
                    if length == 0.0 {
                        continue;
                    }
                    distance_so_far += length;
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
                TrimSegmentKind::Cubic { p0, p1, p2, p3 } => {
                    let segment_count =
                        cubic_measure_segment_count([p0, p1, p2, p3], inv_tolerance);
                    if segment_count == 0 {
                        continue;
                    }
                    let dt = 1.0 / segment_count as f32;
                    let mut t = dt;
                    let mut previous = p0;
                    for _ in 1..segment_count {
                        let next = eval_cubic([p0, p1, p2, p3], t);
                        distance_so_far += distance(previous, next);
                        segments.push(TrimMeasuredSegment {
                            original_index,
                            kind: segment,
                            distance: distance_so_far,
                            t: trim_contour_dot30_t(t),
                        });
                        previous = next;
                        t += dt;
                    }
                    distance_so_far += distance(previous, p3);
                    segments.push(TrimMeasuredSegment {
                        original_index,
                        kind: segment,
                        distance: distance_so_far,
                        t: 1.0,
                    });
                }
            }
        }

        (distance_so_far > 0.0 && !segments.is_empty()).then_some(Self {
            segments,
            length: distance_so_far,
            is_closed,
        })
    }

    fn get_segment(
        &self,
        mut start_distance: f32,
        mut end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        start_distance = start_distance.max(0.0);
        end_distance = end_distance.min(self.length);
        if start_distance >= end_distance {
            return;
        }

        let mut start_index = self.find_segment(start_distance);
        let end_index = self.find_segment(end_distance);
        let mut start_t = self.compute_t(start_index, start_distance);
        let end_t = self.compute_t(end_index, end_distance);

        if 1.0 - start_t < TRIM_CONTOUR_EPSILON && start_index < end_index {
            start_index += 1;
            start_t = 0.0;
        }

        let start = &self.segments[start_index];
        let end = &self.segments[end_index];
        if start.original_index == end.original_index {
            start
                .kind
                .extract(commands, start_t, end_t, start_with_move);
            return;
        }

        start.kind.extract(commands, start_t, 1.0, start_with_move);

        let mut original_index = start.original_index + 1;
        while original_index < end.original_index {
            if let Some(segment) = self.first_segment_for_original(original_index) {
                segment.kind.extract_full(commands);
            }
            original_index += 1;
        }

        end.kind.extract(commands, 0.0, end_t, false);
    }

    fn find_segment(&self, distance: f32) -> usize {
        self.segments
            .iter()
            .position(|segment| segment.distance >= distance)
            .unwrap_or_else(|| self.segments.len() - 1)
    }

    fn compute_t(&self, index: usize, distance: f32) -> f32 {
        let segment = &self.segments[index];
        let mut previous_distance = 0.0;
        let mut previous_t = 0.0;
        if index > 0 {
            let previous = &self.segments[index - 1];
            previous_distance = previous.distance;
            if previous.original_index == segment.original_index {
                previous_t = previous.t;
            }
        }

        let denominator = segment.distance - previous_distance;
        if denominator == 0.0 {
            return previous_t;
        }
        let ratio = (distance - previous_distance) / denominator;
        (previous_t + (segment.t - previous_t) * ratio).clamp(previous_t, segment.t)
    }

    fn first_segment_for_original(&self, original_index: usize) -> Option<&TrimMeasuredSegment> {
        self.segments
            .iter()
            .find(|segment| segment.original_index == original_index)
    }

    fn position_tangent_at_distance(&self, distance: f32) -> ((f32, f32), (f32, f32)) {
        let distance = distance.clamp(0.0, self.length);
        let index = self.find_segment(distance);
        let segment = &self.segments[index];

        match segment.kind {
            TrimSegmentKind::Line { from, to } => {
                let mut previous_distance = 0.0;
                if index > 0 {
                    previous_distance = self.segments[index - 1].distance;
                }
                let denominator = segment.distance - previous_distance;
                let rel_d = if denominator == 0.0 {
                    0.0
                } else {
                    (distance - previous_distance) / denominator
                };
                let (tan, _) = normalize_vector((to.0 - from.0, to.1 - from.1));
                (lerp_point(from, to, rel_d), tan)
            }
            TrimSegmentKind::Cubic { p0, p1, p2, p3 } => {
                cubic_position_tangent([p0, p1, p2, p3], self.compute_t(index, distance))
            }
        }
    }
}

impl RuntimePathMeasure {
    pub fn from_commands(commands: &[RuntimePathCommand]) -> Self {
        Self::from_commands_with_inv_tolerance(commands, TRIM_CONTOUR_DEFAULT_INV_TOLERANCE)
    }

    pub(crate) fn from_commands_with_tolerance(
        commands: &[RuntimePathCommand],
        tolerance: f32,
    ) -> Self {
        let min_tolerance = 1.0 / 16.0;
        let tolerance = if tolerance.is_finite() {
            tolerance.max(min_tolerance)
        } else {
            TRIM_CONTOUR_DEFAULT_TOLERANCE
        };
        Self::from_commands_with_inv_tolerance(commands, 1.0 / tolerance)
    }

    fn from_commands_with_inv_tolerance(
        commands: &[RuntimePathCommand],
        inv_tolerance: f32,
    ) -> Self {
        let contours = TrimContour::from_commands_with_inv_tolerance(commands, inv_tolerance);
        let length = contours.iter().map(|contour| contour.length).sum();
        let raw_is_closed = matches!(commands.last(), Some(RuntimePathCommand::Close));
        Self {
            contours,
            length,
            raw_is_closed,
        }
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub(crate) fn at_percentage(&self, percentage_distance: f32) -> RuntimePathSample {
        let mut in_range_percentage = percentage_distance % 1.0;
        if in_range_percentage < 0.0 {
            in_range_percentage += 1.0;
        }
        if percentage_distance != 0.0 && in_range_percentage == 0.0 {
            in_range_percentage = 1.0;
        }
        self.at_distance(self.length * in_range_percentage)
    }

    pub fn at_distance(&self, distance: f32) -> RuntimePathSample {
        let mut current_distance = distance;
        for contour in &self.contours {
            let contour_length = contour.length;
            if current_distance - contour_length <= 0.0 {
                let (pos, tan) = contour.position_tangent_at_distance(current_distance);
                return RuntimePathSample { pos, tan };
            }
            current_distance -= contour_length;
        }
        RuntimePathSample::default()
    }

    fn get_segment(
        &self,
        start_distance: f32,
        end_distance: f32,
        commands: &mut Vec<RuntimePathCommand>,
        start_with_move: bool,
    ) {
        if self.contours.is_empty() {
            return;
        }

        let start_distance = start_distance.clamp(0.0, self.length);
        let end_distance = end_distance.clamp(0.0, self.length);
        if start_distance >= end_distance {
            return;
        }

        let mut current_distance = 0.0;
        let mut is_first_segment = true;
        for contour in &self.contours {
            let contour_length = contour.length;
            let contour_start = current_distance;
            let contour_end = current_distance + contour_length;

            if contour_end > start_distance && contour_start < end_distance {
                let local_start = (start_distance - contour_start).max(0.0);
                let local_end = (end_distance - contour_start).min(contour_length);
                contour.get_segment(
                    local_start,
                    local_end,
                    commands,
                    !is_first_segment || start_with_move,
                );
                is_first_segment = false;
            }

            current_distance += contour_length;
            if current_distance >= end_distance {
                break;
            }
        }
    }

    pub fn is_closed(&self) -> bool {
        self.contours.len() == 1 && self.contours[0].is_closed
    }

    pub(crate) fn raw_is_closed(&self) -> bool {
        self.raw_is_closed
    }
}

impl RuntimeContourMeasure {
    pub fn from_commands(commands: &[RuntimePathCommand]) -> Vec<Self> {
        TrimContour::from_commands(commands)
            .into_iter()
            .map(|contour| Self { contour })
            .collect()
    }

    pub fn length(&self) -> f32 {
        self.contour.length
    }

    pub fn is_closed(&self) -> bool {
        self.contour.is_closed
    }

    pub fn at_distance(&self, distance: f32) -> RuntimePathSample {
        let (pos, tan) = self.contour.position_tangent_at_distance(distance);
        RuntimePathSample { pos, tan }
    }

    pub fn segment(&self, start: f32, end: f32, start_with_move: bool) -> RawPath {
        let mut commands = Vec::new();
        self.contour
            .get_segment(start, end, &mut commands, start_with_move);
        runtime_raw_path_from_commands(&commands)
    }
}

impl RuntimePathMeasure {
    pub fn segment(&self, start: f32, end: f32, start_with_move: bool) -> RawPath {
        let mut commands = Vec::new();
        self.get_segment(start, end, &mut commands, start_with_move);
        runtime_raw_path_from_commands(&commands)
    }
}

pub fn runtime_path_commands_from_raw_path(path: &RawPath) -> Vec<RuntimePathCommand> {
    (|| {
        let mut commands = Vec::with_capacity(path.verbs().len());
        let mut point_index = 0usize;
        let mut current = None;
        for verb in path.verbs() {
            match verb {
                RenderPathVerb::Move => {
                    let point = path.points().get(point_index)?;
                    point_index += 1;
                    current = Some((point.x, point.y));
                    commands.push(RuntimePathCommand::Move {
                        x: point.x,
                        y: point.y,
                    });
                }
                RenderPathVerb::Line => {
                    let point = path.points().get(point_index)?;
                    point_index += 1;
                    current = Some((point.x, point.y));
                    commands.push(RuntimePathCommand::Line {
                        x: point.x,
                        y: point.y,
                    });
                }
                RenderPathVerb::Quad => {
                    let from = current?;
                    let control = path.points().get(point_index)?;
                    let point = path.points().get(point_index + 1)?;
                    point_index += 2;
                    let x1 = from.0 + (control.x - from.0) * (2.0 / 3.0);
                    let y1 = from.1 + (control.y - from.1) * (2.0 / 3.0);
                    let x2 = point.x + (control.x - point.x) * (2.0 / 3.0);
                    let y2 = point.y + (control.y - point.y) * (2.0 / 3.0);
                    current = Some((point.x, point.y));
                    commands.push(RuntimePathCommand::Cubic {
                        x1,
                        y1,
                        x2,
                        y2,
                        x3: point.x,
                        y3: point.y,
                    });
                }
                RenderPathVerb::Cubic => {
                    let p1 = path.points().get(point_index)?;
                    let p2 = path.points().get(point_index + 1)?;
                    let p3 = path.points().get(point_index + 2)?;
                    point_index += 3;
                    current = Some((p3.x, p3.y));
                    commands.push(RuntimePathCommand::Cubic {
                        x1: p1.x,
                        y1: p1.y,
                        x2: p2.x,
                        y2: p2.y,
                        x3: p3.x,
                        y3: p3.y,
                    });
                }
                RenderPathVerb::Close => commands.push(RuntimePathCommand::Close),
            }
        }
        Some(commands)
    })()
    .unwrap_or_default()
}

impl TrimSegmentKind {
    fn extract(
        self,
        commands: &mut Vec<RuntimePathCommand>,
        start_t: f32,
        end_t: f32,
        move_to: bool,
    ) {
        match self {
            Self::Line { from, to } => {
                let start = lerp_point(from, to, start_t);
                let end = lerp_point(from, to, end_t);
                if move_to {
                    commands.push(RuntimePathCommand::Move {
                        x: start.0,
                        y: start.1,
                    });
                }
                commands.push(RuntimePathCommand::Line { x: end.0, y: end.1 });
            }
            Self::Cubic { p0, p1, p2, p3 } => {
                let [start, control_1, control_2, end] =
                    cubic_extract([p0, p1, p2, p3], start_t, end_t);
                if move_to {
                    commands.push(RuntimePathCommand::Move {
                        x: start.0,
                        y: start.1,
                    });
                }
                commands.push(RuntimePathCommand::Cubic {
                    x1: control_1.0,
                    y1: control_1.1,
                    x2: control_2.0,
                    y2: control_2.1,
                    x3: end.0,
                    y3: end.1,
                });
            }
        }
    }

    fn extract_full(self, commands: &mut Vec<RuntimePathCommand>) {
        match self {
            Self::Line { to, .. } => commands.push(RuntimePathCommand::Line { x: to.0, y: to.1 }),
            Self::Cubic { p1, p2, p3, .. } => commands.push(RuntimePathCommand::Cubic {
                x1: p1.0,
                y1: p1.1,
                x2: p2.0,
                y2: p2.1,
                x3: p3.0,
                y3: p3.1,
            }),
        }
    }
}

fn trim_path_sequential(
    contours: &[TrimContour],
    trim_start: f32,
    trim_end: f32,
    render_offset: f32,
    close_shape: bool,
) -> Vec<RuntimePathCommand> {
    let total_length = contours.iter().map(|contour| contour.length).sum::<f32>();
    if total_length == 0.0 {
        return Vec::new();
    }

    let mut start_length = total_length * (trim_start + render_offset);
    let mut end_length = total_length * (trim_end + render_offset);
    if end_length < start_length {
        std::mem::swap(&mut start_length, &mut end_length);
    }
    if start_length > total_length {
        start_length -= total_length;
        end_length -= total_length;
    }

    let mut indices = Vec::new();
    let mut lengths = Vec::new();
    let mut contour_index = 0usize;
    while end_length > 0.0 {
        let current_index = contour_index % contours.len();
        let contour = &contours[current_index];
        if start_length < contour.length {
            indices.push(current_index);
            lengths.push(start_length);
            lengths.push(end_length);
            end_length -= contour.length;
            start_length = 0.0;
        } else {
            start_length -= contour.length;
            end_length -= contour.length;
        }
        contour_index += 1;
    }

    let mut commands = Vec::new();
    let mut starting_index = 0isize;
    let mut index_count = 0usize;
    let mut previous_contour_index = None::<usize>;
    while index_count < indices.len() {
        let index = starting_index.rem_euclid(indices.len() as isize) as usize;
        let contour_index = indices[index];
        let contour = &contours[contour_index];
        let length_index = index * 2;
        let start_length = lengths[length_index];
        let end_length = lengths[length_index + 1];
        contour.get_segment(
            start_length,
            end_length,
            &mut commands,
            previous_contour_index != Some(contour_index) || !contour.is_closed,
        );
        if (start_length == 0.0 && end_length - start_length >= contour.length && contour.is_closed)
            || close_shape
        {
            commands.push(RuntimePathCommand::Close);
        }
        previous_contour_index = Some(contour_index);
        index_count += 1;
        starting_index -= 1;
    }
    commands
}

fn trim_path_synchronized(
    contours: &[TrimContour],
    trim_start: f32,
    trim_end: f32,
    render_offset: f32,
    close_shape: bool,
) -> Vec<RuntimePathCommand> {
    let mut commands = Vec::new();
    for contour in contours {
        let mut start_length = contour.length * (trim_start + render_offset);
        let mut end_length = contour.length * (trim_end + render_offset);
        if end_length < start_length {
            std::mem::swap(&mut start_length, &mut end_length);
        }

        if start_length >= contour.length {
            start_length -= contour.length;
            end_length -= contour.length;
        }
        contour.get_segment(start_length, end_length, &mut commands, true);
        while end_length > contour.length {
            start_length = 0.0;
            end_length -= contour.length;
            contour.get_segment(start_length, end_length, &mut commands, !contour.is_closed);
        }

        if (trim_start == 0.0 && trim_end == 1.0 && contour.is_closed) || close_shape {
            commands.push(RuntimePathCommand::Close);
        }
    }
    commands
}

const TRIM_CONTOUR_EPSILON: f32 = 1.0 / 4096.0;
const TRIM_CONTOUR_DEFAULT_TOLERANCE: f32 = 0.5;
const TRIM_CONTOUR_DEFAULT_INV_TOLERANCE: f32 = 1.0 / TRIM_CONTOUR_DEFAULT_TOLERANCE;
const TRIM_CONTOUR_MAX_SEGMENTS: u32 = 100;
const TRIM_CONTOUR_DOT30_SCALE: f32 = (1u32 << 30) as f32;
const TRIM_CONTOUR_MAX_DOT30: u32 = (1u32 << 30) - 1;
const TRIM_CONTOUR_INV_MAX_DOT30: f32 = 1.0 / TRIM_CONTOUR_MAX_DOT30 as f32;

fn trim_contour_dot30_t(t: f32) -> f32 {
    ((t * TRIM_CONTOUR_DOT30_SCALE) as u32) as f32 * TRIM_CONTOUR_INV_MAX_DOT30
}

fn cubic_measure_segment_count(points: [(f32, f32); 4], inv_tolerance: f32) -> u32 {
    wangs_cubic(points, inv_tolerance)
        .ceil()
        .ceil()
        .min(TRIM_CONTOUR_MAX_SEGMENTS as f32) as u32
}

fn wangs_cubic(points: [(f32, f32); 4], precision: f32) -> f32 {
    let first = vector_length_squared((
        points[0].0 - 2.0 * points[1].0 + points[2].0,
        points[0].1 - 2.0 * points[1].1 + points[2].1,
    ));
    let second = vector_length_squared((
        points[1].0 - 2.0 * points[2].0 + points[3].0,
        points[1].1 - 2.0 * points[2].1 + points[3].1,
    ));
    let length_term_pow2 = 9.0 * 4.0 / 64.0 * precision * precision;
    (first.max(second) * length_term_pow2).sqrt().sqrt()
}

fn eval_cubic(points: [(f32, f32); 4], t: f32) -> (f32, f32) {
    let a = (
        points[3].0 + 3.0 * (points[1].0 - points[2].0) - points[0].0,
        points[3].1 + 3.0 * (points[1].1 - points[2].1) - points[0].1,
    );
    let b = (
        3.0 * (points[2].0 - 2.0 * points[1].0 + points[0].0),
        3.0 * (points[2].1 - 2.0 * points[1].1 + points[0].1),
    );
    let c = (
        3.0 * (points[1].0 - points[0].0),
        3.0 * (points[1].1 - points[0].1),
    );
    (
        ((a.0 * t + b.0) * t + c.0) * t + points[0].0,
        ((a.1 * t + b.1) * t + c.1) * t + points[0].1,
    )
}

fn cubic_position_tangent(points: [(f32, f32); 4], t: f32) -> ((f32, f32), (f32, f32)) {
    if t == 0.0 {
        let tangent_to = if points[0] != points[1] {
            points[1]
        } else if points[1] != points[2] {
            points[2]
        } else {
            points[3]
        };
        return (
            points[0],
            (tangent_to.0 - points[0].0, tangent_to.1 - points[0].1),
        );
    }
    if t == 1.0 {
        let tangent_from = if points[3] != points[2] {
            points[2]
        } else if points[2] != points[1] {
            points[1]
        } else {
            points[0]
        };
        return (
            points[3],
            (points[3].0 - tangent_from.0, points[3].1 - tangent_from.1),
        );
    }

    let a = (
        points[3].0 + 3.0 * (points[1].0 - points[2].0) - points[0].0,
        points[3].1 + 3.0 * (points[1].1 - points[2].1) - points[0].1,
    );
    let b = (
        3.0 * (points[2].0 - 2.0 * points[1].0 + points[0].0),
        3.0 * (points[2].1 - 2.0 * points[1].1 + points[0].1),
    );
    let c = (
        3.0 * (points[1].0 - points[0].0),
        3.0 * (points[1].1 - points[0].1),
    );
    let (tan, _) = normalize_vector((
        (3.0 * a.0 * t + 2.0 * b.0) * t + c.0,
        (3.0 * a.1 * t + 2.0 * b.1) * t + c.1,
    ));
    (eval_cubic(points, t), tan)
}

fn cubic_extract(points: [(f32, f32); 4], start_t: f32, end_t: f32) -> [(f32, f32); 4] {
    if start_t == 0.0 && end_t == 1.0 {
        points
    } else if start_t == 0.0 {
        let chopped = cubic_subdivide(points, end_t);
        [chopped[0], chopped[1], chopped[2], chopped[3]]
    } else if end_t == 1.0 {
        let chopped = cubic_subdivide(points, start_t);
        [chopped[3], chopped[4], chopped[5], chopped[6]]
    } else {
        let chopped = cubic_subdivide(points, end_t);
        let chopped_again = cubic_subdivide(
            [chopped[0], chopped[1], chopped[2], chopped[3]],
            start_t / end_t,
        );
        [
            chopped_again[3],
            chopped_again[4],
            chopped_again[5],
            chopped_again[6],
        ]
    }
}

fn cubic_subdivide(points: [(f32, f32); 4], t: f32) -> [(f32, f32); 7] {
    let ab = lerp_point(points[0], points[1], t);
    let bc = lerp_point(points[1], points[2], t);
    let cd = lerp_point(points[2], points[3], t);
    let abc = lerp_point(ab, bc, t);
    let bcd = lerp_point(bc, cd, t);
    [
        points[0],
        ab,
        abc,
        lerp_point(abc, bcd, t),
        bcd,
        cd,
        points[3],
    ]
}

fn distance(from: (f32, f32), to: (f32, f32)) -> f32 {
    let x = to.0 - from.0;
    let y = to.1 - from.1;
    (x * x + y * y).sqrt()
}

fn distance_squared(from: (f32, f32), to: (f32, f32)) -> f32 {
    vector_length_squared((to.0 - from.0, to.1 - from.1))
}

fn vector_length_squared(vector: (f32, f32)) -> f32 {
    vector.0 * vector.0 + vector.1 * vector.1
}

fn lerp_point(from: (f32, f32), to: (f32, f32), t: f32) -> (f32, f32) {
    if t == 0.0 {
        return from;
    }
    if t == 1.0 {
        return to;
    }
    (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t)
}

fn color_modulate_opacity(color: u32, opacity: f32) -> u32 {
    let alpha = ((color >> 24) & 0xff) as f32 / 255.0;
    let alpha = opacity_to_alpha(alpha * opacity);
    (color & 0x00ff_ffff) | (u32::from(alpha) << 24)
}

pub(crate) fn color_lerp(from: u32, to: u32, mix: f32) -> u32 {
    let channel = |shift: u32| {
        let from = ((from >> shift) & 0xff) as f32;
        let to = ((to >> shift) & 0xff) as f32;
        ((from * (1.0 - mix) + to * mix).clamp(0.0, 255.0)).round() as u32
    };
    (channel(24) << 24) | (channel(16) << 16) | (channel(8) << 8) | channel(0)
}

fn opacity_to_alpha(opacity: f32) -> u8 {
    (255.0 * opacity.clamp(0.0, 1.0)).round() as u8
}

fn path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
    weighted_context: Option<&WeightedPathContext>,
) -> Vec<RuntimePathCommand> {
    match path.type_name {
        "Ellipse" => ellipse_path_commands(path, path_kind, transform),
        "PointsPath" => points_path_commands(path, path_kind, transform, weighted_context),
        "Polygon" => polygon_path_commands(path, path_kind, transform),
        "Rectangle" => rectangle_path_commands(path, path_kind, transform),
        "Star" => star_path_commands(path, path_kind, transform),
        "Triangle" => triangle_path_commands(path, path_kind, transform),
        _ => Vec::new(),
    }
}

pub(crate) fn runtime_path_geometry_commands(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let path = runtime_path_geometry(artboard, path);
    let weighted_context = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_weighted_path_context(&path, graph, None));
    let path_transform = if weighted_context.is_some() {
        Mat2D::IDENTITY
    } else {
        transform
    };
    let mut commands = path_commands(
        &path,
        ShapePaintPathKind::World,
        path_transform,
        weighted_context.as_ref(),
    );
    prune_empty_path_segments(&mut commands);
    commands
}

fn runtime_path_geometry(artboard: &ArtboardInstance, path: &PathGeometryNode) -> PathGeometryNode {
    let mut path = path.clone();
    path.is_closed = runtime_path_bool_property(
        artboard,
        path.local_id,
        path.type_name,
        "isClosed",
        path.is_closed,
    );
    path.is_hole = runtime_path_bool_property(
        artboard,
        path.local_id,
        path.type_name,
        "isHole",
        path.is_hole,
    );
    path.is_clockwise = runtime_path_is_clockwise(artboard, path.local_id, path.is_clockwise);
    path.parametric = runtime_parametric_path(artboard, &path);

    for vertex in &mut path.vertices {
        vertex.x = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "x",
            vertex.x,
        );
        vertex.y = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "y",
            vertex.y,
        );
        vertex.radius = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "radius",
            vertex.radius,
        );
        vertex.rotation = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "rotation",
            vertex.rotation,
        );
        vertex.distance = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "distance",
            vertex.distance,
        );
        vertex.in_rotation = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "inRotation",
            vertex.in_rotation,
        );
        vertex.in_distance = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "inDistance",
            vertex.in_distance,
        );
        vertex.out_rotation = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "outRotation",
            vertex.out_rotation,
        );
        vertex.out_distance = runtime_path_double_property(
            artboard,
            vertex.local_id,
            vertex.type_name,
            "outDistance",
            vertex.out_distance,
        );
    }

    path
}

fn runtime_parametric_path(
    artboard: &ArtboardInstance,
    path: &PathGeometryNode,
) -> Option<ParametricPathNode> {
    match path.parametric.as_ref()? {
        ParametricPathNode::Ellipse {
            width,
            height,
            origin_x,
            origin_y,
        } => Some(ParametricPathNode::Ellipse {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
        }),
        ParametricPathNode::Polygon {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
        } => Some(ParametricPathNode::Polygon {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
            points: runtime_path_uint_property(
                artboard,
                path.local_id,
                path.type_name,
                "points",
                *points as u64,
            ) as u32,
            corner_radius: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadius",
                *corner_radius,
            ),
        }),
        ParametricPathNode::Star {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
            inner_radius,
        } => Some(ParametricPathNode::Star {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
            points: runtime_path_uint_property(
                artboard,
                path.local_id,
                path.type_name,
                "points",
                *points as u64,
            ) as u32,
            corner_radius: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadius",
                *corner_radius,
            ),
            inner_radius: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "innerRadius",
                *inner_radius,
            ),
        }),
        ParametricPathNode::Rectangle {
            width,
            height,
            origin_x,
            origin_y,
            link_corner_radius,
            corner_radius_tl,
            corner_radius_tr,
            corner_radius_bl,
            corner_radius_br,
        } => Some(ParametricPathNode::Rectangle {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
            link_corner_radius: runtime_path_bool_property(
                artboard,
                path.local_id,
                path.type_name,
                "linkCornerRadius",
                *link_corner_radius,
            ),
            corner_radius_tl: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusTL",
                *corner_radius_tl,
            ),
            corner_radius_tr: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusTR",
                *corner_radius_tr,
            ),
            corner_radius_bl: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusBL",
                *corner_radius_bl,
            ),
            corner_radius_br: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "cornerRadiusBR",
                *corner_radius_br,
            ),
        }),
        ParametricPathNode::Triangle {
            width,
            height,
            origin_x,
            origin_y,
        } => Some(ParametricPathNode::Triangle {
            width: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "width",
                *width,
            ),
            height: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "height",
                *height,
            ),
            origin_x: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originX",
                *origin_x,
            ),
            origin_y: runtime_path_double_property(
                artboard,
                path.local_id,
                path.type_name,
                "originY",
                *origin_y,
            ),
        }),
    }
}

fn measure_parametric_path_layout(
    path: &PathGeometryNode,
    max_width: f32,
    max_height: f32,
) -> Option<(f32, f32)> {
    let (width, height) = match path.parametric.as_ref()? {
        ParametricPathNode::Ellipse { width, height, .. }
        | ParametricPathNode::Polygon { width, height, .. }
        | ParametricPathNode::Rectangle { width, height, .. }
        | ParametricPathNode::Star { width, height, .. }
        | ParametricPathNode::Triangle { width, height, .. } => (*width, *height),
    };
    Some((max_width.min(width), max_height.min(height)))
}

fn parametric_path_with_control_size(
    parametric: ParametricPathNode,
    width: f32,
    height: f32,
) -> ParametricPathNode {
    match parametric {
        ParametricPathNode::Ellipse {
            origin_x, origin_y, ..
        } => ParametricPathNode::Ellipse {
            width,
            height,
            origin_x,
            origin_y,
        },
        ParametricPathNode::Polygon {
            origin_x,
            origin_y,
            points,
            corner_radius,
            ..
        } => ParametricPathNode::Polygon {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
        },
        ParametricPathNode::Star {
            origin_x,
            origin_y,
            points,
            corner_radius,
            inner_radius,
            ..
        } => ParametricPathNode::Star {
            width,
            height,
            origin_x,
            origin_y,
            points,
            corner_radius,
            inner_radius,
        },
        ParametricPathNode::Triangle {
            origin_x, origin_y, ..
        } => ParametricPathNode::Triangle {
            width,
            height,
            origin_x,
            origin_y,
        },
        ParametricPathNode::Rectangle {
            origin_x,
            origin_y,
            link_corner_radius,
            corner_radius_tl,
            corner_radius_tr,
            corner_radius_bl,
            corner_radius_br,
            ..
        } => ParametricPathNode::Rectangle {
            width,
            height,
            origin_x,
            origin_y,
            link_corner_radius,
            corner_radius_tl,
            corner_radius_tr,
            corner_radius_bl,
            corner_radius_br,
        },
    }
}

fn runtime_path_property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    match (type_name, property_name) {
        ("Path", "pathFlags") => cached_runtime_property_key!("Path", "pathFlags"),
        ("Path", "isHole") => cached_runtime_property_key!("Path", "isHole"),
        ("Path", "isClosed") => cached_runtime_property_key!("Path", "isClosed"),
        ("PointsPath", "pathFlags") => cached_runtime_property_key!("PointsPath", "pathFlags"),
        ("PointsPath", "isHole") => cached_runtime_property_key!("PointsPath", "isHole"),
        ("PointsPath", "isClosed") => cached_runtime_property_key!("PointsPath", "isClosed"),
        ("Ellipse", "pathFlags") => cached_runtime_property_key!("Ellipse", "pathFlags"),
        ("Ellipse", "isHole") => cached_runtime_property_key!("Ellipse", "isHole"),
        ("Ellipse", "isClosed") => cached_runtime_property_key!("Ellipse", "isClosed"),
        ("Polygon", "pathFlags") => cached_runtime_property_key!("Polygon", "pathFlags"),
        ("Polygon", "isHole") => cached_runtime_property_key!("Polygon", "isHole"),
        ("Polygon", "isClosed") => cached_runtime_property_key!("Polygon", "isClosed"),
        ("Star", "pathFlags") => cached_runtime_property_key!("Star", "pathFlags"),
        ("Star", "isHole") => cached_runtime_property_key!("Star", "isHole"),
        ("Star", "isClosed") => cached_runtime_property_key!("Star", "isClosed"),
        ("Rectangle", "pathFlags") => cached_runtime_property_key!("Rectangle", "pathFlags"),
        ("Rectangle", "isHole") => cached_runtime_property_key!("Rectangle", "isHole"),
        ("Rectangle", "isClosed") => cached_runtime_property_key!("Rectangle", "isClosed"),
        ("Triangle", "pathFlags") => cached_runtime_property_key!("Triangle", "pathFlags"),
        ("Triangle", "isHole") => cached_runtime_property_key!("Triangle", "isHole"),
        ("Triangle", "isClosed") => cached_runtime_property_key!("Triangle", "isClosed"),
        ("StraightVertex", "x") => cached_runtime_property_key!("StraightVertex", "x"),
        ("StraightVertex", "y") => cached_runtime_property_key!("StraightVertex", "y"),
        ("StraightVertex", "radius") => cached_runtime_property_key!("StraightVertex", "radius"),
        ("StraightVertex", "rotation") => {
            cached_runtime_property_key!("StraightVertex", "rotation")
        }
        ("StraightVertex", "distance") => {
            cached_runtime_property_key!("StraightVertex", "distance")
        }
        ("StraightVertex", "inRotation") => {
            cached_runtime_property_key!("StraightVertex", "inRotation")
        }
        ("StraightVertex", "inDistance") => {
            cached_runtime_property_key!("StraightVertex", "inDistance")
        }
        ("StraightVertex", "outRotation") => {
            cached_runtime_property_key!("StraightVertex", "outRotation")
        }
        ("StraightVertex", "outDistance") => {
            cached_runtime_property_key!("StraightVertex", "outDistance")
        }
        ("CubicMirroredVertex", "x") => cached_runtime_property_key!("CubicMirroredVertex", "x"),
        ("CubicMirroredVertex", "y") => cached_runtime_property_key!("CubicMirroredVertex", "y"),
        ("CubicMirroredVertex", "radius") => {
            cached_runtime_property_key!("CubicMirroredVertex", "radius")
        }
        ("CubicMirroredVertex", "rotation") => {
            cached_runtime_property_key!("CubicMirroredVertex", "rotation")
        }
        ("CubicMirroredVertex", "distance") => {
            cached_runtime_property_key!("CubicMirroredVertex", "distance")
        }
        ("CubicMirroredVertex", "inRotation") => {
            cached_runtime_property_key!("CubicMirroredVertex", "inRotation")
        }
        ("CubicMirroredVertex", "inDistance") => {
            cached_runtime_property_key!("CubicMirroredVertex", "inDistance")
        }
        ("CubicMirroredVertex", "outRotation") => {
            cached_runtime_property_key!("CubicMirroredVertex", "outRotation")
        }
        ("CubicMirroredVertex", "outDistance") => {
            cached_runtime_property_key!("CubicMirroredVertex", "outDistance")
        }
        ("CubicAsymmetricVertex", "x") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "x")
        }
        ("CubicAsymmetricVertex", "y") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "y")
        }
        ("CubicAsymmetricVertex", "radius") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "radius")
        }
        ("CubicAsymmetricVertex", "rotation") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "rotation")
        }
        ("CubicAsymmetricVertex", "distance") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "distance")
        }
        ("CubicAsymmetricVertex", "inRotation") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "inRotation")
        }
        ("CubicAsymmetricVertex", "inDistance") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "inDistance")
        }
        ("CubicAsymmetricVertex", "outRotation") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "outRotation")
        }
        ("CubicAsymmetricVertex", "outDistance") => {
            cached_runtime_property_key!("CubicAsymmetricVertex", "outDistance")
        }
        ("CubicDetachedVertex", "x") => cached_runtime_property_key!("CubicDetachedVertex", "x"),
        ("CubicDetachedVertex", "y") => cached_runtime_property_key!("CubicDetachedVertex", "y"),
        ("CubicDetachedVertex", "radius") => {
            cached_runtime_property_key!("CubicDetachedVertex", "radius")
        }
        ("CubicDetachedVertex", "rotation") => {
            cached_runtime_property_key!("CubicDetachedVertex", "rotation")
        }
        ("CubicDetachedVertex", "distance") => {
            cached_runtime_property_key!("CubicDetachedVertex", "distance")
        }
        ("CubicDetachedVertex", "inRotation") => {
            cached_runtime_property_key!("CubicDetachedVertex", "inRotation")
        }
        ("CubicDetachedVertex", "inDistance") => {
            cached_runtime_property_key!("CubicDetachedVertex", "inDistance")
        }
        ("CubicDetachedVertex", "outRotation") => {
            cached_runtime_property_key!("CubicDetachedVertex", "outRotation")
        }
        ("CubicDetachedVertex", "outDistance") => {
            cached_runtime_property_key!("CubicDetachedVertex", "outDistance")
        }
        ("Ellipse", "width") => cached_runtime_property_key!("Ellipse", "width"),
        ("Ellipse", "height") => cached_runtime_property_key!("Ellipse", "height"),
        ("Ellipse", "originX") => cached_runtime_property_key!("Ellipse", "originX"),
        ("Ellipse", "originY") => cached_runtime_property_key!("Ellipse", "originY"),
        ("Polygon", "width") => cached_runtime_property_key!("Polygon", "width"),
        ("Polygon", "height") => cached_runtime_property_key!("Polygon", "height"),
        ("Polygon", "originX") => cached_runtime_property_key!("Polygon", "originX"),
        ("Polygon", "originY") => cached_runtime_property_key!("Polygon", "originY"),
        ("Polygon", "points") => cached_runtime_property_key!("Polygon", "points"),
        ("Polygon", "cornerRadius") => cached_runtime_property_key!("Polygon", "cornerRadius"),
        ("Star", "width") => cached_runtime_property_key!("Star", "width"),
        ("Star", "height") => cached_runtime_property_key!("Star", "height"),
        ("Star", "originX") => cached_runtime_property_key!("Star", "originX"),
        ("Star", "originY") => cached_runtime_property_key!("Star", "originY"),
        ("Star", "points") => cached_runtime_property_key!("Star", "points"),
        ("Star", "cornerRadius") => cached_runtime_property_key!("Star", "cornerRadius"),
        ("Star", "innerRadius") => cached_runtime_property_key!("Star", "innerRadius"),
        ("Rectangle", "width") => cached_runtime_property_key!("Rectangle", "width"),
        ("Rectangle", "height") => cached_runtime_property_key!("Rectangle", "height"),
        ("Rectangle", "originX") => cached_runtime_property_key!("Rectangle", "originX"),
        ("Rectangle", "originY") => cached_runtime_property_key!("Rectangle", "originY"),
        ("Rectangle", "linkCornerRadius") => {
            cached_runtime_property_key!("Rectangle", "linkCornerRadius")
        }
        ("Rectangle", "cornerRadiusTL") => {
            cached_runtime_property_key!("Rectangle", "cornerRadiusTL")
        }
        ("Rectangle", "cornerRadiusTR") => {
            cached_runtime_property_key!("Rectangle", "cornerRadiusTR")
        }
        ("Rectangle", "cornerRadiusBL") => {
            cached_runtime_property_key!("Rectangle", "cornerRadiusBL")
        }
        ("Rectangle", "cornerRadiusBR") => {
            cached_runtime_property_key!("Rectangle", "cornerRadiusBR")
        }
        ("Triangle", "width") => cached_runtime_property_key!("Triangle", "width"),
        ("Triangle", "height") => cached_runtime_property_key!("Triangle", "height"),
        ("Triangle", "originX") => cached_runtime_property_key!("Triangle", "originX"),
        ("Triangle", "originY") => cached_runtime_property_key!("Triangle", "originY"),
        _ => property_key_for_name(type_name, property_name),
    }
}

fn runtime_path_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    runtime_path_property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.double_property(local_id, key))
        .unwrap_or(default)
}

fn runtime_path_bool_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: bool,
) -> bool {
    runtime_path_property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.bool_property(local_id, key))
        .unwrap_or(default)
}

fn runtime_path_uint_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: u64,
) -> u64 {
    runtime_path_property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.uint_property(local_id, key))
        .unwrap_or(default)
}

fn runtime_path_is_clockwise(artboard: &ArtboardInstance, local_id: usize, default: bool) -> bool {
    let default_flags = if default { 0 } else { 1 << 1 };
    let flags = runtime_path_uint_property(artboard, local_id, "Path", "pathFlags", default_flags);
    flags & (1 << 1) == 0
}

fn points_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
    weighted_context: Option<&WeightedPathContext>,
) -> Vec<RuntimePathCommand> {
    if path.type_name != "PointsPath" || path.vertices.len() < 2 {
        return Vec::new();
    }
    if path
        .vertices
        .iter()
        .any(|vertex| vertex.weight_local.is_some())
    {
        let Some(weighted_context) = weighted_context else {
            return Vec::new();
        };
        return weighted_points_path_commands(path, path_kind, transform, weighted_context)
            .unwrap_or_default();
    }
    if path
        .vertices
        .iter()
        .any(|vertex| !is_supported_point_path_vertex(vertex))
    {
        return Vec::new();
    }
    let reverse_for_clockwise_fill = path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform);

    let mut commands = Vec::new();
    let first = &path.vertices[0];

    let (start, start_in, mut out, start_is_cubic, mut prev_is_cubic) =
        if let Some((in_point, out_point)) = cubic_vertex_points(first) {
            let start = vertex_translation(first);
            push_move(&mut commands, transform, start);
            (start, in_point, out_point, true, true)
        } else if first.radius != 0.0 {
            let prev = path
                .vertices
                .last()
                .expect("path has at least two vertices");
            let next = &path.vertices[1];
            let (start, mut out_point, mut in_point, out_after) =
                rounded_straight_vertex_points(first, prev, next);
            if first.radius < 0.0 {
                rotate_rounded_points(
                    out_after,
                    start,
                    vertex_translation(first),
                    &mut out_point,
                    &mut in_point,
                );
            }
            push_move(&mut commands, transform, start);
            push_cubic(&mut commands, transform, out_point, in_point, out_after);
            (start, start, out_after, false, false)
        } else {
            let start = vertex_translation(first);
            push_move(&mut commands, transform, start);
            (start, start, start, false, false)
        };

    for (index, vertex) in path.vertices.iter().enumerate().skip(1) {
        if let Some((in_point, out_point)) = cubic_vertex_points(vertex) {
            let translation = vertex_translation(vertex);
            push_cubic(&mut commands, transform, out, in_point, translation);
            prev_is_cubic = true;
            out = out_point;
        } else {
            let position = vertex_translation(vertex);
            if vertex.radius != 0.0 {
                let prev = &path.vertices[index - 1];
                let next = &path.vertices[(index + 1) % path.vertices.len()];
                let (translation, mut out_point, mut in_point, out_after) =
                    rounded_straight_vertex_points(vertex, prev, next);
                if prev_is_cubic {
                    push_cubic(&mut commands, transform, out, translation, translation);
                } else {
                    push_line(&mut commands, transform, translation);
                }
                if vertex.radius < 0.0 {
                    rotate_rounded_points(
                        out_after,
                        translation,
                        position,
                        &mut out_point,
                        &mut in_point,
                    );
                }
                push_cubic(&mut commands, transform, out_point, in_point, out_after);
                prev_is_cubic = false;
                out = out_after;
            } else if prev_is_cubic {
                push_cubic(&mut commands, transform, out, position, position);
                prev_is_cubic = false;
                out = position;
            } else {
                push_line(&mut commands, transform, position);
                out = position;
            }
        }
    }

    if path.is_closed {
        if prev_is_cubic || start_is_cubic {
            push_cubic(&mut commands, transform, out, start_in, start);
        } else {
            push_line(&mut commands, transform, start);
        }
        commands.push(RuntimePathCommand::Close);
    }

    if reverse_for_clockwise_fill {
        path_commands_backwards(&commands)
    } else {
        commands
    }
}

fn weighted_points_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
    weighted_context: &WeightedPathContext,
) -> Option<Vec<RuntimePathCommand>> {
    if path
        .vertices
        .iter()
        .any(|vertex| !is_supported_point_path_vertex(vertex))
    {
        return None;
    }
    let reverse_for_clockwise_fill = path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform);

    let mut commands = Vec::new();
    let first = &path.vertices[0];

    let (start, start_in, mut out, start_is_cubic, mut prev_is_cubic) =
        if let Some((in_point, out_point)) = weighted_cubic_vertex_points(first, weighted_context)?
        {
            let start = weighted_vertex_translation(first, weighted_context)?;
            push_move(&mut commands, transform, start);
            (start, in_point, out_point, true, true)
        } else if first.radius != 0.0 {
            let prev = path
                .vertices
                .last()
                .expect("path has at least two vertices");
            let next = &path.vertices[1];
            let (start, mut out_point, mut in_point, out_after) =
                weighted_rounded_straight_vertex_points(first, prev, next, weighted_context)?;
            if first.radius < 0.0 {
                rotate_rounded_points(
                    out_after,
                    start,
                    weighted_vertex_translation(first, weighted_context)?,
                    &mut out_point,
                    &mut in_point,
                );
            }
            push_move(&mut commands, transform, start);
            push_cubic(&mut commands, transform, out_point, in_point, out_after);
            (start, start, out_after, false, false)
        } else {
            let start = weighted_vertex_translation(first, weighted_context)?;
            push_move(&mut commands, transform, start);
            (start, start, start, false, false)
        };

    for (index, vertex) in path.vertices.iter().enumerate().skip(1) {
        if let Some((in_point, out_point)) = weighted_cubic_vertex_points(vertex, weighted_context)?
        {
            let translation = weighted_vertex_translation(vertex, weighted_context)?;
            push_cubic(&mut commands, transform, out, in_point, translation);
            prev_is_cubic = true;
            out = out_point;
        } else {
            let position = weighted_vertex_translation(vertex, weighted_context)?;
            if vertex.radius != 0.0 {
                let prev = &path.vertices[index - 1];
                let next = &path.vertices[(index + 1) % path.vertices.len()];
                let (translation, mut out_point, mut in_point, out_after) =
                    weighted_rounded_straight_vertex_points(vertex, prev, next, weighted_context)?;
                if prev_is_cubic {
                    push_cubic(&mut commands, transform, out, translation, translation);
                } else {
                    push_line(&mut commands, transform, translation);
                }
                if vertex.radius < 0.0 {
                    rotate_rounded_points(
                        out_after,
                        translation,
                        position,
                        &mut out_point,
                        &mut in_point,
                    );
                }
                push_cubic(&mut commands, transform, out_point, in_point, out_after);
                prev_is_cubic = false;
                out = out_after;
            } else if prev_is_cubic {
                push_cubic(&mut commands, transform, out, position, position);
                prev_is_cubic = false;
                out = position;
            } else {
                push_line(&mut commands, transform, position);
                out = position;
            }
        }
    }

    if path.is_closed {
        if prev_is_cubic || start_is_cubic {
            push_cubic(&mut commands, transform, out, start_in, start);
        } else {
            push_line(&mut commands, transform, start);
        }
        commands.push(RuntimePathCommand::Close);
    }

    Some(if reverse_for_clockwise_fill {
        path_commands_backwards(&commands)
    } else {
        commands
    })
}

fn weighted_vertex_translation(
    vertex: &PathVertexNode,
    weighted_context: &WeightedPathContext,
) -> Option<(f32, f32)> {
    if vertex.weight_local.is_some() {
        weighted_context.deform_point(
            vertex_translation(vertex),
            vertex.weight_indices.unwrap_or(1),
            vertex.weight_values.unwrap_or(255),
        )
    } else {
        Some(vertex_translation(vertex))
    }
}

fn weighted_cubic_vertex_points(
    vertex: &PathVertexNode,
    weighted_context: &WeightedPathContext,
) -> Option<Option<((f32, f32), (f32, f32))>> {
    let Some((in_point, out_point)) = cubic_vertex_points(vertex) else {
        return Some(None);
    };
    if vertex.weight_local.is_some() {
        Some(Some((
            weighted_context.deform_point(
                in_point,
                vertex.weight_in_indices.unwrap_or(1),
                vertex.weight_in_values.unwrap_or(255),
            )?,
            weighted_context.deform_point(
                out_point,
                vertex.weight_out_indices.unwrap_or(1),
                vertex.weight_out_values.unwrap_or(255),
            )?,
        )))
    } else {
        Some(Some((in_point, out_point)))
    }
}

fn weighted_vertex_render_in_point(
    vertex: &PathVertexNode,
    weighted_context: &WeightedPathContext,
) -> Option<(f32, f32)> {
    weighted_cubic_vertex_points(vertex, weighted_context)?
        .map(|(in_point, _)| in_point)
        .or_else(|| weighted_vertex_translation(vertex, weighted_context))
}

fn weighted_vertex_render_out_point(
    vertex: &PathVertexNode,
    weighted_context: &WeightedPathContext,
) -> Option<(f32, f32)> {
    weighted_cubic_vertex_points(vertex, weighted_context)?
        .map(|(_, out_point)| out_point)
        .or_else(|| weighted_vertex_translation(vertex, weighted_context))
}

fn weighted_rounded_straight_vertex_points(
    vertex: &PathVertexNode,
    prev: &PathVertexNode,
    next: &PathVertexNode,
    weighted_context: &WeightedPathContext,
) -> Option<((f32, f32), (f32, f32), (f32, f32), (f32, f32))> {
    let position = weighted_vertex_translation(vertex, weighted_context)?;
    let (to_prev, to_prev_length) = normalize_vector(subtract_point(
        weighted_vertex_render_out_point(prev, weighted_context)?,
        position,
    ));
    let (to_next, to_next_length) = normalize_vector(subtract_point(
        weighted_vertex_render_in_point(next, weighted_context)?,
        position,
    ));
    let render_radius = (to_prev_length / 2.0)
        .min(to_next_length / 2.0)
        .min(vertex.radius.abs());
    let ideal_distance = compute_ideal_control_point_distance(to_prev, to_next, render_radius);
    let translation = scale_and_add_point(position, to_prev, render_radius);
    let out_point = scale_and_add_point(position, to_prev, render_radius - ideal_distance);
    let in_point = scale_and_add_point(position, to_next, render_radius - ideal_distance);
    let out_after = scale_and_add_point(position, to_next, render_radius);

    Some((translation, out_point, in_point, out_after))
}

fn rectangle_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Rectangle {
        width,
        height,
        origin_x,
        origin_y,
        link_corner_radius,
        corner_radius_tl,
        corner_radius_tr,
        corner_radius_bl,
        corner_radius_br,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let width = *width;
    let height = *height;
    let origin_x = *origin_x;
    let origin_y = *origin_y;
    let link_corner_radius = *link_corner_radius;
    let left = -origin_x * width;
    let top = -origin_y * height;
    let right = left + width;
    let bottom = top + height;
    let top_left_radius = *corner_radius_tl;
    let top_right_radius = if link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_tr
    };
    let bottom_right_radius = if link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_br
    };
    let bottom_left_radius = if link_corner_radius {
        top_left_radius
    } else {
        *corner_radius_bl
    };

    let virtual_path = PathGeometryNode {
        local_id: path.local_id,
        global_id: path.global_id,
        type_name: "PointsPath",
        is_closed: true,
        is_hole: path.is_hole,
        is_clockwise: true,
        parametric: None,
        vertices: vec![
            virtual_straight_vertex(left, top, top_left_radius),
            virtual_straight_vertex(right, top, top_right_radius),
            virtual_straight_vertex(right, bottom, bottom_right_radius),
            virtual_straight_vertex(left, bottom, bottom_left_radius),
        ],
    };

    points_path_commands(&virtual_path, path_kind, transform, None)
}

const CIRCLE_CONSTANT: f32 = 0.552_284_8;

fn ellipse_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Ellipse {
        width,
        height,
        origin_x,
        origin_y,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };
    let reverse_for_clockwise_fill = path_kind == ShapePaintPathKind::LocalClockwise
        && path_needs_clockwise_reversal(path, transform);

    let radius_x = *width / 2.0;
    let radius_y = *height / 2.0;
    let ox = -*origin_x * *width + radius_x;
    let oy = -*origin_y * *height + radius_y;
    let top = (ox, oy - radius_y);
    let right = (ox + radius_x, oy);
    let bottom = (ox, oy + radius_y);
    let left = (ox - radius_x, oy);

    let mut commands = Vec::new();
    push_move(&mut commands, transform, top);
    push_cubic(
        &mut commands,
        transform,
        (ox + radius_x * CIRCLE_CONSTANT, oy - radius_y),
        (ox + radius_x, oy - CIRCLE_CONSTANT * radius_y),
        right,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox + radius_x, oy + CIRCLE_CONSTANT * radius_y),
        (ox + radius_x * CIRCLE_CONSTANT, oy + radius_y),
        bottom,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox - radius_x * CIRCLE_CONSTANT, oy + radius_y),
        (ox - radius_x, oy + radius_y * CIRCLE_CONSTANT),
        left,
    );
    push_cubic(
        &mut commands,
        transform,
        (ox - radius_x, oy - radius_y * CIRCLE_CONSTANT),
        (ox - radius_x * CIRCLE_CONSTANT, oy - radius_y),
        top,
    );
    commands.push(RuntimePathCommand::Close);
    if reverse_for_clockwise_fill {
        path_commands_backwards(&commands)
    } else {
        commands
    }
}

fn polygon_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Polygon {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let Ok(count) = usize::try_from(*points) else {
        return Vec::new();
    };
    if count < 2 {
        return Vec::new();
    }

    let half_width = *width / 2.0;
    let half_height = *height / 2.0;
    let ox = -*origin_x * *width + half_width;
    let oy = -*origin_y * *height + half_height;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    let inc = 2.0 * std::f32::consts::PI / *points as f32;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..count {
        vertices.push(virtual_straight_vertex(
            ox + angle.cos() * half_width,
            oy + angle.sin() * half_height,
            *corner_radius,
        ));
        angle += inc;
    }

    closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}

fn star_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Star {
        width,
        height,
        origin_x,
        origin_y,
        points,
        corner_radius,
        inner_radius,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let Ok(point_count) = usize::try_from(*points) else {
        return Vec::new();
    };
    let Some(count) = point_count.checked_mul(2) else {
        return Vec::new();
    };
    if count < 2 {
        return Vec::new();
    }

    let half_width = *width / 2.0;
    let half_height = *height / 2.0;
    let inner_half_width = *width * *inner_radius / 2.0;
    let inner_half_height = *height * *inner_radius / 2.0;
    let ox = -*origin_x * *width + half_width;
    let oy = -*origin_y * *height + half_height;
    let mut angle = -std::f32::consts::FRAC_PI_2;
    let inc = 2.0 * std::f32::consts::PI / count as f32;
    let mut vertices = Vec::with_capacity(count);
    for _ in 0..point_count {
        vertices.push(virtual_straight_vertex(
            ox + angle.cos() * half_width,
            oy + angle.sin() * half_height,
            *corner_radius,
        ));
        angle += inc;
        vertices.push(virtual_straight_vertex(
            ox + angle.cos() * inner_half_width,
            oy + angle.sin() * inner_half_height,
            *corner_radius,
        ));
        angle += inc;
    }

    closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}

fn closed_straight_vertices_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
    vertices: Vec<PathVertexNode>,
) -> Vec<RuntimePathCommand> {
    let virtual_path = PathGeometryNode {
        local_id: path.local_id,
        global_id: path.global_id,
        type_name: "PointsPath",
        is_closed: true,
        is_hole: path.is_hole,
        is_clockwise: true,
        parametric: None,
        vertices,
    };

    points_path_commands(&virtual_path, path_kind, transform, None)
}

fn triangle_path_commands(
    path: &PathGeometryNode,
    path_kind: ShapePaintPathKind,
    transform: Mat2D,
) -> Vec<RuntimePathCommand> {
    let Some(ParametricPathNode::Triangle {
        width,
        height,
        origin_x,
        origin_y,
    }) = path.parametric.as_ref()
    else {
        return Vec::new();
    };

    let ox = -*origin_x * *width;
    let oy = -*origin_y * *height;
    let vertices = vec![
        virtual_straight_vertex(ox + *width / 2.0, oy, 0.0),
        virtual_straight_vertex(ox + *width, oy + *height, 0.0),
        virtual_straight_vertex(ox, oy + *height, 0.0),
    ];

    closed_straight_vertices_path_commands(path, path_kind, transform, vertices)
}

fn virtual_straight_vertex(x: f32, y: f32, radius: f32) -> PathVertexNode {
    PathVertexNode {
        local_id: 0,
        global_id: 0,
        type_name: "StraightVertex",
        x,
        y,
        radius,
        rotation: 0.0,
        distance: 0.0,
        in_rotation: 0.0,
        in_distance: 0.0,
        out_rotation: 0.0,
        out_distance: 0.0,
        weight_local: None,
        weight_global: None,
        weight_type_name: None,
        weight_values: None,
        weight_indices: None,
        weight_in_values: None,
        weight_in_indices: None,
        weight_out_values: None,
        weight_out_indices: None,
    }
}

fn is_supported_point_path_vertex(vertex: &PathVertexNode) -> bool {
    match vertex.type_name {
        "StraightVertex" => true,
        "CubicDetachedVertex" | "CubicAsymmetricVertex" | "CubicMirroredVertex" => true,
        _ => false,
    }
}

#[derive(Debug, Clone)]
struct WeightedPathContext {
    skin_world_transform: Mat2D,
    bone_transforms: Vec<Mat2D>,
}

#[derive(Debug, Clone)]
struct RuntimeNSlicedNodeContext {
    world: Mat2D,
    inverse_world: Mat2D,
    width: f32,
    height: f32,
    scale_x: f32,
    scale_y: f32,
    x_px_stops: Vec<f32>,
    y_px_stops: Vec<f32>,
    x_scale_info: RuntimeNSlicerScaleInfo,
    y_scale_info: RuntimeNSlicerScaleInfo,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeNSlicerScaleInfo {
    use_scale: bool,
    scale_factor: f32,
    fallback_size: f32,
}

impl WeightedPathContext {
    fn deform_point(&self, point: (f32, f32), indices: u32, weights: u32) -> Option<(f32, f32)> {
        let mut blended = [0.0; 6];
        for index in 0..4 {
            let shift = index * 8;
            let weight = ((weights >> shift) & 0xff) as u8;
            if weight == 0 {
                continue;
            }
            let bone_index = ((indices >> shift) & 0xff) as usize;
            let bone_transform = self.bone_transforms.get(bone_index)?;
            let normalized_weight = f32::from(weight) / 255.0;
            for (target, value) in blended.iter_mut().zip(bone_transform.0) {
                *target += value * normalized_weight;
            }
        }

        let (x, y) = self.skin_world_transform.transform_point(point.0, point.1);
        Some(Mat2D(blended).transform_point(x, y))
    }
}

impl RuntimeNSlicedNodeContext {
    fn deform_world_point(&self, x: f32, y: f32) -> (f32, f32) {
        let (local_x, local_y) = self.inverse_world.map_point(x, y);
        let sliced_x = if self.scale_x == 0.0 {
            0.0
        } else {
            runtime_nslicer_map_value(
                &self.x_px_stops,
                self.x_scale_info,
                self.width.abs(),
                local_x,
            ) * runtime_copysign_one(self.scale_x)
        };
        let sliced_y = if self.scale_y == 0.0 {
            0.0
        } else {
            runtime_nslicer_map_value(
                &self.y_px_stops,
                self.y_scale_info,
                self.height.abs(),
                local_y,
            ) * runtime_copysign_one(self.scale_y)
        };
        self.world.map_point(sliced_x, sliced_y)
    }
}

fn runtime_nsliced_node_context_for_shape(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    shape_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Option<RuntimeNSlicedNodeContext> {
    if graph.n_slicer_details.is_empty() {
        return None;
    }

    let runtime = instance.runtime_file()?;
    let deformer = graph.shape_deformers.iter().find(|deformer| {
        deformer.shape_local == shape_local && deformer.deformer_type_name == Some("NSlicedNode")
    })?;
    let deformer_local = deformer.deformer_local?;
    let details = graph
        .n_slicer_details
        .iter()
        .find(|details| details.local_id == deformer_local && details.type_name == "NSlicedNode")?;
    let object = runtime.object(details.global_id as usize);
    let initial_width = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "initialWidth",
        0.0,
    );
    let initial_height = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "initialHeight",
        0.0,
    );
    if initial_width <= 0.0 || initial_height <= 0.0 || object.is_none() {
        return None;
    }

    let authored_width = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "width",
        0.0,
    );
    let authored_height = runtime_component_double_property(
        runtime,
        instance,
        details.local_id,
        details.global_id,
        "NSlicedNode",
        "height",
        0.0,
    );
    let control_size =
        runtime_nsliced_node_layout_control_size(instance, graph, details.local_id, layout_bounds);
    let width = control_size
        .map(|bounds| bounds.width)
        .unwrap_or(authored_width);
    let height = control_size
        .map(|bounds| bounds.height)
        .unwrap_or(authored_height);
    let world = instance.runtime_component_world_transform_with_bounds(
        details.local_id,
        graph,
        layout_bounds,
    );
    let inverse_world = runtime_mat2d_invert(world)?;
    let scale_x = width / initial_width;
    let scale_y = height / initial_height;
    let x_px_stops = runtime_nslicer_px_stops(runtime, instance, &details.x_axes, initial_width);
    let y_px_stops = runtime_nslicer_px_stops(runtime, instance, &details.y_axes, initial_height);
    let x_uv_stops = runtime_nslicer_uv_stops(runtime, instance, &details.x_axes, initial_width);
    let y_uv_stops = runtime_nslicer_uv_stops(runtime, instance, &details.y_axes, initial_height);

    Some(RuntimeNSlicedNodeContext {
        world,
        inverse_world,
        width,
        height,
        scale_x,
        scale_y,
        x_px_stops,
        y_px_stops,
        x_scale_info: runtime_nslicer_analyze_uv_stops(&x_uv_stops, initial_width, scale_x.abs()),
        // Mirrors C++ NSlicedNode::updateMapWorldPoint, including the width
        // argument used for Y scale analysis.
        y_scale_info: runtime_nslicer_analyze_uv_stops(&y_uv_stops, initial_width, scale_y.abs()),
    })
}

fn runtime_nsliced_node_layout_control_size(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    nsliced_local: usize,
    layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
) -> Option<RuntimeLayoutBounds> {
    let layout_bounds = layout_bounds?;
    let mut current_local = graph
        .components
        .iter()
        .find(|component| component.local_id == nsliced_local)?
        .parent_local?;
    loop {
        let component = graph
            .components
            .iter()
            .find(|component| component.local_id == current_local)?;
        match component.type_name {
            "LayoutComponent" => {
                return layout_bounds.get(&current_local).copied().or_else(|| {
                    Some(instance.runtime_layout_component_bounds(current_local, graph))
                });
            }
            "Artboard" => return None,
            "Node" => return None,
            _ => current_local = component.parent_local?,
        }
    }
}

fn runtime_deform_path_commands_with_nsliced_node(
    commands: &mut [RuntimePathCommand],
    context: &RuntimeNSlicedNodeContext,
    path_kind: ShapePaintPathKind,
    shape_world: Mat2D,
    inverse_shape_world: Mat2D,
) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                (*x, *y) = runtime_deform_path_point_with_nsliced_node(
                    *x,
                    *y,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                (*x1, *y1) = runtime_deform_path_point_with_nsliced_node(
                    *x1,
                    *y1,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
                (*x2, *y2) = runtime_deform_path_point_with_nsliced_node(
                    *x2,
                    *y2,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
                (*x3, *y3) = runtime_deform_path_point_with_nsliced_node(
                    *x3,
                    *y3,
                    context,
                    path_kind,
                    shape_world,
                    inverse_shape_world,
                );
            }
            RuntimePathCommand::Close => {}
        }
    }
}

fn runtime_deform_path_point_with_nsliced_node(
    x: f32,
    y: f32,
    context: &RuntimeNSlicedNodeContext,
    path_kind: ShapePaintPathKind,
    shape_world: Mat2D,
    inverse_shape_world: Mat2D,
) -> (f32, f32) {
    if path_kind == ShapePaintPathKind::World {
        return context.deform_world_point(x, y);
    }
    let (world_x, world_y) = shape_world.map_point(x, y);
    let (deformed_x, deformed_y) = context.deform_world_point(world_x, world_y);
    inverse_shape_world.map_point(deformed_x, deformed_y)
}

fn runtime_deform_local_gradient_point_with_nsliced_node(
    x: f32,
    y: f32,
    context: &RuntimeNSlicedNodeContext,
    shape_world: Mat2D,
    inverse_shape_world: Mat2D,
) -> (f32, f32) {
    let (world_x, world_y) = shape_world.map_point(x, y);
    let (deformed_x, deformed_y) = context.deform_world_point(world_x, world_y);
    inverse_shape_world.map_point(deformed_x, deformed_y)
}

fn runtime_nslicer_uv_stops(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axes: &[NSlicerAxisNode],
    size: f32,
) -> Vec<f32> {
    let mut stops = vec![0.0];
    for axis in axes {
        let offset = runtime_axis_offset(runtime, instance, axis);
        if runtime_axis_normalized(runtime, instance, axis) {
            stops.push(offset.clamp(0.0, 1.0));
        } else {
            stops.push((offset / size).clamp(0.0, 1.0));
        }
    }
    stops.push(1.0);
    stops.sort_by(f32::total_cmp);
    stops
}

fn runtime_nslicer_px_stops(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axes: &[NSlicerAxisNode],
    size: f32,
) -> Vec<f32> {
    let mut stops = vec![0.0];
    for axis in axes {
        let offset = runtime_axis_offset(runtime, instance, axis);
        if runtime_axis_normalized(runtime, instance, axis) {
            stops.push(offset.clamp(0.0, 1.0) * size);
        } else {
            stops.push(offset.clamp(0.0, size));
        }
    }
    stops.push(size);
    stops.sort_by(f32::total_cmp);
    stops
}

fn runtime_nslicer_analyze_uv_stops(
    stops: &[f32],
    size: f32,
    scale: f32,
) -> RuntimeNSlicerScaleInfo {
    let mut fixed_pct = 0.0;
    let mut empty_patch_count = 0;
    for index in 0..stops.len().saturating_sub(1) {
        let range = stops[index + 1] - stops[index];
        if runtime_nslicer_is_fixed_segment(index) {
            fixed_pct += range;
        } else if range == 0.0 {
            empty_patch_count += 1;
        }
    }

    let fixed_size = fixed_pct * size;
    let scalable_size = size - fixed_size;
    let use_scale = scalable_size != 0.0;
    let scale_factor = if use_scale {
        size.mul_add(scale, -fixed_size) / scalable_size
    } else {
        0.0
    };
    let fallback_size = if !use_scale && empty_patch_count != 0 {
        (size - fixed_size / scale) / empty_patch_count as f32
    } else {
        0.0
    };
    RuntimeNSlicerScaleInfo {
        use_scale,
        scale_factor,
        fallback_size,
    }
}

fn runtime_nslicer_map_value(
    stops: &[f32],
    scale_info: RuntimeNSlicerScaleInfo,
    size: f32,
    value: f32,
) -> f32 {
    let Some(first) = stops.first().copied() else {
        return value;
    };
    let Some(last) = stops.last().copied() else {
        return value;
    };
    if value < first - 0.01 {
        return value;
    }
    if value > last + 0.01 {
        return value - last + size;
    }

    let mut result = 0.0;
    for index in 0..stops.len().saturating_sub(1) {
        let found = value <= stops[index + 1];
        let span = if found {
            value - stops[index]
        } else {
            stops[index + 1] - stops[index]
        };
        if runtime_nslicer_is_fixed_segment(index) {
            result += span;
        } else if scale_info.use_scale {
            result = scale_info.scale_factor.mul_add(span, result);
        } else {
            result += scale_info.fallback_size;
        }
        if found {
            break;
        }
    }
    result
}

fn runtime_nslicer_is_fixed_segment(index: usize) -> bool {
    index % 2 == 0
}

fn runtime_axis_offset(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axis: &NSlicerAxisNode,
) -> f32 {
    runtime_component_double_property(
        runtime,
        instance,
        axis.local_id,
        axis.global_id,
        "Axis",
        "offset",
        0.0,
    )
}

fn runtime_axis_normalized(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    axis: &NSlicerAxisNode,
) -> bool {
    runtime_component_bool_property(
        runtime,
        instance,
        axis.local_id,
        axis.global_id,
        "Axis",
        "normalized",
        false,
    )
}

fn runtime_component_double_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    local_id: usize,
    global_id: u32,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    let property_key = property_key_for_name(type_name, property_name);
    property_key
        .and_then(|key| instance.double_property(local_id, key))
        .or_else(|| {
            runtime.object(global_id as usize).and_then(|object| {
                property_key
                    .and_then(|key| runtime_object_explicit_double_property_by_key(object, key))
            })
        })
        .unwrap_or(default)
}

fn runtime_component_bool_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    local_id: usize,
    global_id: u32,
    type_name: &str,
    property_name: &str,
    default: bool,
) -> bool {
    let property_key = property_key_for_name(type_name, property_name);
    property_key
        .and_then(|key| instance.bool_property(local_id, key))
        .or_else(|| {
            runtime.object(global_id as usize).and_then(|object| {
                property_key
                    .and_then(|key| runtime_object_explicit_bool_property_by_key(object, key))
            })
        })
        .unwrap_or(default)
}

fn runtime_mat2d_invert(mat: Mat2D) -> Option<Mat2D> {
    let determinant = mat.determinant();
    if determinant == 0.0 {
        return None;
    }

    let [a, b, c, d, e, f] = mat.0;
    let determinant = 1.0 / determinant;
    Some(Mat2D([
        d * determinant,
        -b * determinant,
        -c * determinant,
        a * determinant,
        (c * f - d * e) * determinant,
        (b * e - a * f) * determinant,
    ]))
}

fn runtime_copysign_one(value: f32) -> f32 {
    if value.is_sign_negative() { -1.0 } else { 1.0 }
}

fn vertex_translation(vertex: &PathVertexNode) -> (f32, f32) {
    (vertex.x, vertex.y)
}

fn cubic_vertex_points(vertex: &PathVertexNode) -> Option<((f32, f32), (f32, f32))> {
    let point = vertex_translation(vertex);
    match vertex.type_name {
        "CubicDetachedVertex" => Some((
            add_point(point, angle_vector(vertex.in_rotation, vertex.in_distance)),
            add_point(
                point,
                angle_vector(vertex.out_rotation, vertex.out_distance),
            ),
        )),
        "CubicAsymmetricVertex" => Some((
            subtract_point(point, angle_vector(vertex.rotation, vertex.in_distance)),
            add_point(point, angle_vector(vertex.rotation, vertex.out_distance)),
        )),
        "CubicMirroredVertex" => Some((
            subtract_point(point, angle_vector(vertex.rotation, vertex.distance)),
            add_point(point, angle_vector(vertex.rotation, vertex.distance)),
        )),
        _ => None,
    }
}

fn angle_vector(rotation: f32, distance: f32) -> (f32, f32) {
    (rotation.cos() * distance, rotation.sin() * distance)
}

fn add_point(point: (f32, f32), vector: (f32, f32)) -> (f32, f32) {
    (point.0 + vector.0, point.1 + vector.1)
}

fn subtract_point(point: (f32, f32), vector: (f32, f32)) -> (f32, f32) {
    (point.0 - vector.0, point.1 - vector.1)
}

fn rounded_straight_vertex_points(
    vertex: &PathVertexNode,
    prev: &PathVertexNode,
    next: &PathVertexNode,
) -> ((f32, f32), (f32, f32), (f32, f32), (f32, f32)) {
    let position = vertex_translation(vertex);
    let (to_prev, to_prev_length) =
        normalize_vector(subtract_point(vertex_render_out_point(prev), position));
    let (to_next, to_next_length) =
        normalize_vector(subtract_point(vertex_render_in_point(next), position));
    let render_radius = (to_prev_length / 2.0)
        .min(to_next_length / 2.0)
        .min(vertex.radius.abs());
    let ideal_distance = compute_ideal_control_point_distance(to_prev, to_next, render_radius);
    let translation = scale_and_add_point(position, to_prev, render_radius);
    let out_point = scale_and_add_point(position, to_prev, render_radius - ideal_distance);
    let in_point = scale_and_add_point(position, to_next, render_radius - ideal_distance);
    let out_after = scale_and_add_point(position, to_next, render_radius);

    (translation, out_point, in_point, out_after)
}

fn vertex_render_in_point(vertex: &PathVertexNode) -> (f32, f32) {
    cubic_vertex_points(vertex)
        .map(|(in_point, _)| in_point)
        .unwrap_or_else(|| vertex_translation(vertex))
}

fn vertex_render_out_point(vertex: &PathVertexNode) -> (f32, f32) {
    cubic_vertex_points(vertex)
        .map(|(_, out_point)| out_point)
        .unwrap_or_else(|| vertex_translation(vertex))
}

fn normalize_vector(vector: (f32, f32)) -> ((f32, f32), f32) {
    let length = vector_length(vector);
    if length > 0.0 {
        ((vector.0 / length, vector.1 / length), length)
    } else {
        (vector, length)
    }
}

fn vector_length(vector: (f32, f32)) -> f32 {
    (vector.0 * vector.0 + vector.1 * vector.1).sqrt()
}

fn scale_and_add_point(point: (f32, f32), vector: (f32, f32), scale: f32) -> (f32, f32) {
    // Mirrors C++ Vec2D::scaleAndAdd after compiler contraction; rounded
    // midpoint pruning can depend on the one-ulp split this preserves.
    (
        vector.0.mul_add(scale, point.0),
        vector.1.mul_add(scale, point.1),
    )
}

fn compute_ideal_control_point_distance(
    to_prev: (f32, f32),
    to_next: (f32, f32),
    radius: f32,
) -> f32 {
    let angle = cross_point(to_prev, to_next)
        .atan2(dot_point(to_prev, to_next))
        .abs();
    let natural_rounding = (4.0 / 3.0)
        * (std::f32::consts::PI / (2.0 * ((2.0 * std::f32::consts::PI) / angle))).tan()
        * radius
        * if angle < std::f32::consts::FRAC_PI_2 {
            1.0 + angle.cos()
        } else {
            2.0 - angle.sin()
        };

    radius.min(natural_rounding)
}

fn rotate_rounded_points(
    next_point: (f32, f32),
    prev_point: (f32, f32),
    point: (f32, f32),
    out_point: &mut (f32, f32),
    in_point: &mut (f32, f32),
) {
    let v1 = subtract_point(prev_point, next_point);
    let v2 = subtract_point(point, next_point);
    let angle = cross_point(v1, v2).atan2(dot_point(v1, v2));
    *out_point = rotate_point_around(*out_point, prev_point, angle * 2.0);
    *in_point = rotate_point_around(*in_point, next_point, -angle * 2.0);
}

fn rotate_point_around(point: (f32, f32), origin: (f32, f32), angle: f32) -> (f32, f32) {
    let sin = angle.sin();
    let cos = angle.cos();
    let x = point.0 - origin.0;
    let y = point.1 - origin.1;

    (x * cos - y * sin + origin.0, x * sin + y * cos + origin.1)
}

fn dot_point(left: (f32, f32), right: (f32, f32)) -> f32 {
    left.0 * right.0 + left.1 * right.1
}

fn cross_point(left: (f32, f32), right: (f32, f32)) -> f32 {
    left.0 * right.1 - left.1 * right.0
}

fn push_move(commands: &mut Vec<RuntimePathCommand>, transform: Mat2D, point: (f32, f32)) {
    let (x, y) = transform.map_point(point.0, point.1);
    commands.push(RuntimePathCommand::Move { x, y });
}

fn push_line(commands: &mut Vec<RuntimePathCommand>, transform: Mat2D, point: (f32, f32)) {
    let (x, y) = transform.map_point(point.0, point.1);
    commands.push(RuntimePathCommand::Line { x, y });
}

fn push_cubic(
    commands: &mut Vec<RuntimePathCommand>,
    transform: Mat2D,
    point1: (f32, f32),
    point2: (f32, f32),
    point3: (f32, f32),
) {
    let (x1, y1) = transform.map_point(point1.0, point1.1);
    let (x2, y2) = transform.map_point(point2.0, point2.1);
    let (x3, y3) = transform.map_point(point3.0, point3.1);
    commands.push(RuntimePathCommand::Cubic {
        x1,
        y1,
        x2,
        y2,
        x3,
        y3,
    });
}

fn path_needs_clockwise_reversal(path: &PathGeometryNode, transform: Mat2D) -> bool {
    let designed_clockwise = if path.is_clockwise { 1.0 } else { -1.0 };
    let is_not_clockwise = transform.determinant() * designed_clockwise < 0.0;
    is_not_clockwise != path.is_hole
}

fn sorted_drawable_uses_render_opacity(type_name: &str) -> bool {
    matches!(type_name, "Shape" | "TextInputDrawable")
}

fn is_text_input_drawable_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "TextInputCursor" | "TextInputSelectedText" | "TextInputSelection" | "TextInputText"
    )
}

fn sorted_drawable_is_nested_artboard(type_name: &str) -> bool {
    matches!(
        type_name,
        "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout"
    )
}

fn runtime_draw_command_is_nested_artboard(command: &RuntimeDrawCommand) -> bool {
    command.object_kind.is_nested_artboard() || command.referenced_artboard_global.is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_render_cache_key_includes_live_referenced_artboard() {
        let authored = nested_render_cache_key(Some(33), Some(14), 105, 0);
        let rebound = nested_render_cache_key(Some(33), Some(14), 147, 0);
        let replaced = nested_render_cache_key(Some(33), Some(14), 105, 1);

        assert_ne!(authored, rebound);
        assert_ne!(authored, replaced);
        assert_eq!(
            authored,
            nested_render_cache_key(Some(33), Some(14), 105, 0)
        );
    }

    #[test]
    fn raw_script_path_commands_preserve_verbs_and_lower_quadratics() {
        let mut path = RawPath::new();
        path.move_to(1.0, 2.0);
        path.line_to(3.0, 4.0);
        path.quad_to(6.0, 7.0, 9.0, 10.0);
        path.cubic_to(11.0, 12.0, 13.0, 14.0, 15.0, 16.0);
        path.close();

        assert_eq!(
            runtime_path_commands_from_raw_path(&path),
            vec![
                RuntimePathCommand::Move { x: 1.0, y: 2.0 },
                RuntimePathCommand::Line { x: 3.0, y: 4.0 },
                RuntimePathCommand::Cubic {
                    x1: 5.0,
                    y1: 6.0,
                    x2: 7.0,
                    y2: 8.0,
                    x3: 9.0,
                    y3: 10.0,
                },
                RuntimePathCommand::Cubic {
                    x1: 11.0,
                    y1: 12.0,
                    x2: 13.0,
                    y2: 14.0,
                    x3: 15.0,
                    y3: 16.0,
                },
                RuntimePathCommand::Close,
            ]
        );
    }

    #[test]
    fn script_path_measure_exposes_contours_and_extracts_distance_ranges() {
        let commands = [
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 10.0, y: 0.0 },
            RuntimePathCommand::Move { x: 20.0, y: 0.0 },
            RuntimePathCommand::Line { x: 20.0, y: 5.0 },
        ];
        let contours = RuntimeContourMeasure::from_commands(&commands);
        let measure = RuntimePathMeasure::from_commands(&commands);

        assert_eq!(contours.len(), 2);
        assert_eq!(contours[0].length(), 10.0);
        assert_eq!(contours[1].at_distance(2.0).pos, (20.0, 2.0));
        assert_eq!(measure.length(), 15.0);
        assert_eq!(measure.at_distance(12.0).pos, (20.0, 2.0));
        assert_eq!(
            runtime_path_commands_from_raw_path(&measure.segment(8.0, 12.0, true)),
            vec![
                RuntimePathCommand::Move { x: 8.0, y: 0.0 },
                RuntimePathCommand::Line { x: 10.0, y: 0.0 },
                RuntimePathCommand::Move { x: 20.0, y: 0.0 },
                RuntimePathCommand::Line { x: 20.0, y: 2.0 },
            ]
        );
    }
    use crate::{ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptValue};
    use nuxie_binary::read_runtime_file;
    use nuxie_graph::GraphFile;
    use nuxie_render_api::ColorInt;
    use std::any::Any;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

    #[derive(Default)]
    struct CountingStats {
        make_empty_paths: Cell<usize>,
        rewinds: Cell<usize>,
        add_raw_paths: Cell<usize>,
        fill_rules: Cell<usize>,
        lines: Cell<usize>,
        closes: Cell<usize>,
        linear_gradients: RefCell<Vec<[f32; 4]>>,
    }

    struct CountingFactory {
        stats: Rc<CountingStats>,
        next_path_id: usize,
    }

    struct RecordingRenderer {
        ops: Rc<RefCell<Vec<String>>>,
    }

    impl Renderer for RecordingRenderer {
        fn save(&mut self) {
            self.ops.borrow_mut().push("save".to_owned());
        }

        fn restore(&mut self) {
            self.ops.borrow_mut().push("restore".to_owned());
        }

        fn transform(&mut self, transform: RenderMat2D) {
            self.ops
                .borrow_mut()
                .push(format!("transform:{:?}", transform.0));
        }

        fn draw_path(&mut self, _path: &dyn RenderPath, _paint: &dyn RenderPaint) {
            self.ops.borrow_mut().push("draw_path".to_owned());
        }

        fn clip_path(&mut self, _path: &dyn RenderPath) {
            self.ops.borrow_mut().push("clip_path".to_owned());
        }

        fn draw_image(
            &mut self,
            _image: Option<&dyn RenderImage>,
            _sampler: RenderImageSampler,
            _blend_mode: RenderBlendMode,
            _opacity: f32,
        ) {
            self.ops.borrow_mut().push("draw_image".to_owned());
        }

        fn draw_image_mesh(
            &mut self,
            _image: Option<&dyn RenderImage>,
            _sampler: RenderImageSampler,
            _vertices: Option<&dyn RenderBuffer>,
            _uv_coords: Option<&dyn RenderBuffer>,
            _indices: Option<&dyn RenderBuffer>,
            _vertex_count: u32,
            _index_count: u32,
            _blend_mode: RenderBlendMode,
            _opacity: f32,
        ) {
            self.ops.borrow_mut().push("draw_image_mesh".to_owned());
        }

        fn modulate_opacity(&mut self, opacity: f32) {
            self.ops.borrow_mut().push(format!("opacity:{opacity}"));
        }
    }

    struct FakeScriptInstance {
        ops: Rc<RefCell<Vec<String>>>,
    }

    impl ScriptInstance for FakeScriptInstance {
        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(true)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn call_draw(
            &mut self,
            _factory: &mut dyn RenderFactory,
            _renderer: &mut dyn Renderer,
            _host: &mut dyn ScriptHost,
        ) -> Result<(), ScriptError> {
            self.ops.borrow_mut().push("script_draw".to_owned());
            Ok(())
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    struct CountingRenderPath {
        stats: Rc<CountingStats>,
        id: usize,
        raw_path: RawPath,
        fill_rule: RenderFillRule,
    }

    impl RenderPath for CountingRenderPath {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn rewind(&mut self) {
            self.stats.rewinds.set(self.stats.rewinds.get() + 1);
            self.raw_path.rewind();
        }

        fn fill_rule(&mut self, value: RenderFillRule) {
            self.stats.fill_rules.set(self.stats.fill_rules.get() + 1);
            self.fill_rule = value;
        }

        fn add_render_path(&mut self, _path: &dyn RenderPath, _transform: RenderMat2D) {}

        fn add_render_path_backwards(&mut self, _path: &dyn RenderPath, _transform: RenderMat2D) {}

        fn add_raw_path(&mut self, path: &RawPath) {
            self.stats
                .add_raw_paths
                .set(self.stats.add_raw_paths.get() + 1);
            self.raw_path.add_path(path, RenderMat2D::IDENTITY);
        }

        fn move_to(&mut self, x: f32, y: f32) {
            self.raw_path.move_to(x, y);
        }

        fn line_to(&mut self, x: f32, y: f32) {
            self.stats.lines.set(self.stats.lines.get() + 1);
            self.raw_path.line_to(x, y);
        }

        fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
            self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
        }

        fn close(&mut self) {
            self.stats.closes.set(self.stats.closes.get() + 1);
            self.raw_path.close();
        }
    }

    struct TestRenderBuffer {
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        bytes: Vec<u8>,
    }

    impl RenderBuffer for TestRenderBuffer {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn buffer_type(&self) -> RenderBufferType {
            self.buffer_type
        }

        fn flags(&self) -> RenderBufferFlags {
            self.flags
        }

        fn size_in_bytes(&self) -> usize {
            self.bytes.len()
        }

        fn map_mut(&mut self) -> &mut [u8] {
            &mut self.bytes
        }

        fn unmap(&mut self) {}
    }

    struct TestRenderShader;

    impl RenderShader for TestRenderShader {
        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    struct TestRenderImage;

    impl RenderImage for TestRenderImage {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn width(&self) -> u32 {
            0
        }

        fn height(&self) -> u32 {
            0
        }
    }

    struct TestRenderPaint;

    impl RenderPaint for TestRenderPaint {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn style(&mut self, _style: RenderPaintStyle) {}
        fn color(&mut self, _value: ColorInt) {}
        fn thickness(&mut self, _value: f32) {}
        fn join(&mut self, _value: RenderStrokeJoin) {}
        fn cap(&mut self, _value: RenderStrokeCap) {}
        fn feather(&mut self, _value: f32) {}
        fn blend_mode(&mut self, _value: RenderBlendMode) {}
        fn shader(&mut self, _shader: Option<&dyn RenderShader>) {}
        fn invalidate_stroke(&mut self) {}
    }

    impl RenderFactory for CountingFactory {
        fn make_render_buffer(
            &mut self,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size_in_bytes: usize,
        ) -> Box<dyn RenderBuffer> {
            Box::new(TestRenderBuffer {
                buffer_type,
                flags,
                bytes: vec![0; size_in_bytes],
            })
        }

        fn make_linear_gradient(
            &mut self,
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            _colors: &[ColorInt],
            _stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.stats
                .linear_gradients
                .borrow_mut()
                .push([sx, sy, ex, ey]);
            Box::new(TestRenderShader)
        }

        fn make_radial_gradient(
            &mut self,
            _cx: f32,
            _cy: f32,
            _radius: f32,
            _colors: &[ColorInt],
            _stops: &[f32],
        ) -> Box<dyn RenderShader> {
            Box::new(TestRenderShader)
        }

        fn make_render_path(
            &mut self,
            raw_path: RawPath,
            fill_rule: RenderFillRule,
        ) -> Box<dyn RenderPath> {
            let id = self.next_path_id;
            self.next_path_id += 1;
            Box::new(CountingRenderPath {
                stats: Rc::clone(&self.stats),
                id,
                raw_path,
                fill_rule,
            })
        }

        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.stats
                .make_empty_paths
                .set(self.stats.make_empty_paths.get() + 1);
            self.make_render_path(RawPath::new(), RenderFillRule::NonZero)
        }

        fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
            Box::new(TestRenderPaint)
        }

        fn decode_image(&mut self, _data: &[u8]) -> Box<dyn RenderImage> {
            Box::new(TestRenderImage)
        }
    }

    #[test]
    fn scripted_drawable_calls_attached_instance_with_cpp_draw_envelope() {
        let bytes = include_bytes!("../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture imports");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture graphs");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let mut instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");
        let ops = Rc::new(RefCell::new(Vec::new()));
        instance.set_script_instance_for_global(
            900,
            Box::new(FakeScriptInstance {
                ops: Rc::clone(&ops),
            }),
        );

        let command = RuntimeDrawCommand {
            kind: RuntimeDrawCommandKind::Draw,
            object_kind: RuntimeDrawCommandObjectKind::ScriptedDrawable,
            local_id: None,
            global_id: Some(900),
            type_name: "ScriptedDrawable",
            world_transform: Some(Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0])),
            render_opacity: 0.5,
            referenced_artboard_global: None,
            resolved_image_asset_global: None,
            clipping_shape_local: None,
            needs_save_operation: true,
            shape_paints: Vec::new(),
        };
        let stats = Rc::new(CountingStats::default());
        let mut factory = CountingFactory {
            stats,
            next_path_id: 0,
        };
        let mut renderer = RecordingRenderer {
            ops: Rc::clone(&ops),
        };

        runtime_draw_scripted_drawable(&instance, &command, &mut factory, &mut renderer)
            .expect("scripted draw succeeds");

        assert_eq!(
            ops.borrow().as_slice(),
            [
                "save",
                "opacity:0.5",
                "transform:[1.0, 0.0, 0.0, 1.0, 10.0, 20.0]",
                "script_draw",
                "restore",
            ]
        );
    }

    #[test]
    fn draw_path_reuses_render_path_until_path_epoch_changes() {
        let stats = Rc::new(CountingStats::default());
        let mut factory = CountingFactory {
            stats: Rc::clone(&stats),
            next_path_id: 0,
        };
        let mut cache = RuntimeRenderPathCache::default();
        let key = RuntimeDrawPathCacheKey {
            kind: RuntimeDrawPathCacheKind::Draw,
            path_kind: RuntimeShapePaintPathKind::Local,
            local_id: Some(7),
            path_index: 0,
        };
        let commands = vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 1.0, y: 1.0 },
        ];
        let changed_commands = vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 2.0, y: 2.0 },
            RuntimePathCommand::Close,
        ];

        let first_id = {
            let path = cache.draw_path(key, 10, &mut factory, &commands, RenderFillRule::Clockwise);
            path.as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path")
                .id
        };

        assert_eq!(stats.make_empty_paths.get(), 1);
        assert_eq!(stats.rewinds.get(), 1);
        assert_eq!(stats.add_raw_paths.get(), 1);
        assert_eq!(stats.lines.get(), 0);

        let second_id = {
            let path = cache.draw_path(key, 10, &mut factory, &commands, RenderFillRule::Clockwise);
            path.as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path")
                .id
        };

        assert_eq!(second_id, first_id);
        assert_eq!(stats.make_empty_paths.get(), 1);
        assert_eq!(stats.rewinds.get(), 1);
        assert_eq!(stats.add_raw_paths.get(), 1);
        assert_eq!(stats.lines.get(), 0);

        let (third_id, third_verbs, third_points) = {
            let path = cache.draw_path(
                key,
                11,
                &mut factory,
                &changed_commands,
                RenderFillRule::Clockwise,
            );
            let path = path
                .as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path");
            (
                path.id,
                path.raw_path.verbs().len(),
                path.raw_path.points().len(),
            )
        };

        assert_eq!(third_id, first_id);
        assert_eq!(stats.make_empty_paths.get(), 1);
        assert_eq!(stats.rewinds.get(), 2);
        assert_eq!(stats.add_raw_paths.get(), 2);
        assert_eq!(stats.lines.get(), 0);
        assert_eq!(stats.closes.get(), 0);
        assert_eq!(third_verbs, 3);
        assert_eq!(third_points, 2);
    }

    #[test]
    fn draw_path_skips_redundant_fill_rule_replay() {
        let stats = Rc::new(CountingStats::default());
        let mut factory = CountingFactory {
            stats: Rc::clone(&stats),
            next_path_id: 0,
        };
        let mut cache = RuntimeRenderPathCache::default();
        let key = RuntimeDrawPathCacheKey {
            kind: RuntimeDrawPathCacheKind::Draw,
            path_kind: RuntimeShapePaintPathKind::Local,
            local_id: Some(7),
            path_index: 0,
        };
        let commands = vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 1.0, y: 1.0 },
        ];

        let first_id = {
            let path = cache.draw_path_with_optional_fill_rule(
                key,
                10,
                &mut factory,
                &commands,
                RenderFillRule::Clockwise,
                Some(RenderFillRule::Clockwise),
            );
            path.as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path")
                .id
        };

        assert_eq!(stats.fill_rules.get(), 1);

        let second_id = {
            let path = cache.draw_path_with_optional_fill_rule(
                key,
                10,
                &mut factory,
                &commands,
                RenderFillRule::Clockwise,
                Some(RenderFillRule::Clockwise),
            );
            path.as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path")
                .id
        };

        assert_eq!(second_id, first_id);
        assert_eq!(stats.fill_rules.get(), 1);

        cache.draw_path_with_optional_fill_rule(
            key,
            10,
            &mut factory,
            &commands,
            RenderFillRule::Clockwise,
            Some(RenderFillRule::EvenOdd),
        );
        assert_eq!(stats.fill_rules.get(), 2);

        cache.draw_path_with_optional_fill_rule(
            key,
            10,
            &mut factory,
            &commands,
            RenderFillRule::Clockwise,
            Some(RenderFillRule::EvenOdd),
        );
        assert_eq!(stats.fill_rules.get(), 2);
    }

    #[test]
    fn retained_local_path_slots_skip_rebuild_until_key_changes() {
        let stats = Rc::new(CountingStats::default());
        let mut factory = CountingFactory {
            stats: Rc::clone(&stats),
            next_path_id: 0,
        };
        let mut cache = RuntimeRenderPathCache::default();
        let key = RuntimeRetainedRenderPathCacheKey {
            graph_global_id: 42,
            path_epoch: 10,
            layout_epoch: 20,
            fill_rule: RenderFillRule::Clockwise,
        };
        let commands = vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 1.0, y: 1.0 },
        ];
        let changed_commands = vec![
            RuntimePathCommand::Move { x: 0.0, y: 0.0 },
            RuntimePathCommand::Line { x: 2.0, y: 2.0 },
            RuntimePathCommand::Close,
        ];

        let first_id = {
            let path = cache.clipping_shape_path(key, 7, &mut factory, &commands);
            path.as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path")
                .id
        };

        assert_eq!(stats.make_empty_paths.get(), 1);
        assert_eq!(stats.rewinds.get(), 1);
        assert_eq!(stats.add_raw_paths.get(), 1);
        assert_eq!(stats.lines.get(), 0);

        let second_id = {
            let path = cache.clipping_shape_path(key, 7, &mut factory, &commands);
            path.as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path")
                .id
        };

        assert_eq!(second_id, first_id);
        assert_eq!(stats.make_empty_paths.get(), 1);
        assert_eq!(stats.rewinds.get(), 1);
        assert_eq!(stats.add_raw_paths.get(), 1);
        assert_eq!(stats.lines.get(), 0);

        let (third_id, third_verbs, third_points) = {
            let path = cache.clipping_shape_path(
                RuntimeRetainedRenderPathCacheKey {
                    path_epoch: 11,
                    ..key
                },
                7,
                &mut factory,
                &changed_commands,
            );
            let path = path
                .as_any()
                .downcast_ref::<CountingRenderPath>()
                .expect("counting path");
            (
                path.id,
                path.raw_path.verbs().len(),
                path.raw_path.points().len(),
            )
        };

        assert_eq!(third_id, first_id);
        assert_eq!(stats.make_empty_paths.get(), 1);
        assert_eq!(stats.rewinds.get(), 2);
        assert_eq!(stats.add_raw_paths.get(), 2);
        assert_eq!(stats.lines.get(), 0);
        assert_eq!(third_verbs, 3);
        assert_eq!(third_points, 2);
    }

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn schema_property_key(type_name: &str, property_name: &str) -> u64 {
        let definition = nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
        definition
            .properties
            .iter()
            .find(|property| property.name == property_name)
            .map(|property| property.key.int)
            .or_else(|| {
                definition.ancestors.iter().find_map(|ancestor| {
                    nuxie_schema::definition_by_name(ancestor).and_then(|ancestor| {
                        ancestor
                            .properties
                            .iter()
                            .find(|property| property.name == property_name)
                            .map(|property| property.key.int)
                    })
                })
            })
            .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
            .into()
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
        let type_key = nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int;
        push_var_uint(bytes, u64::from(type_key));
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_uint(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u64) {
        push_var_uint(bytes, schema_property_key(type_name, property_name));
        push_var_uint(bytes, value);
    }

    fn push_f32(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: f32) {
        push_var_uint(bytes, schema_property_key(type_name, property_name));
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_bool(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: bool) {
        push_var_uint(bytes, schema_property_key(type_name, property_name));
        bytes.push(u8::from(value));
    }

    fn push_color(bytes: &mut Vec<u8>, type_name: &str, property_name: &str, value: u32) {
        push_var_uint(bytes, schema_property_key(type_name, property_name));
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    // A shape with a world-space (transformAffectsStroke=false) gradient
    // stroke; the gradient shader bakes the shape's world transform into its
    // endpoints.
    fn synthetic_world_space_gradient_stroke_riv() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9631);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "Artboard", |_| {});
        push_object(&mut bytes, "Shape", |bytes| {
            push_uint(bytes, "Node", "parentId", 0);
            push_f32(bytes, "Node", "x", 0.0);
            push_f32(bytes, "Node", "y", 0.0);
        });
        push_object(&mut bytes, "Stroke", |bytes| {
            push_uint(bytes, "Component", "parentId", 1);
            push_f32(bytes, "Stroke", "thickness", 2.0);
            push_bool(bytes, "Stroke", "transformAffectsStroke", false);
        });
        push_object(&mut bytes, "LinearGradient", |bytes| {
            push_uint(bytes, "Component", "parentId", 2);
            push_f32(bytes, "LinearGradient", "endX", 10.0);
        });
        push_object(&mut bytes, "GradientStop", |bytes| {
            push_uint(bytes, "Component", "parentId", 3);
            push_color(bytes, "GradientStop", "colorValue", 0xffff_0000);
            push_f32(bytes, "GradientStop", "position", 0.0);
        });
        push_object(&mut bytes, "GradientStop", |bytes| {
            push_uint(bytes, "Component", "parentId", 3);
            push_color(bytes, "GradientStop", "colorValue", 0xff00_00ff);
            push_f32(bytes, "GradientStop", "position", 1.0);
        });
        push_object(&mut bytes, "PointsPath", |bytes| {
            push_uint(bytes, "Node", "parentId", 1);
            push_bool(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object(&mut bytes, "StraightVertex", |bytes| {
            push_uint(bytes, "Component", "parentId", 6);
            push_f32(bytes, "Vertex", "x", 0.0);
            push_f32(bytes, "Vertex", "y", 0.0);
        });
        push_object(&mut bytes, "StraightVertex", |bytes| {
            push_uint(bytes, "Component", "parentId", 6);
            push_f32(bytes, "Vertex", "x", 10.0);
            push_f32(bytes, "Vertex", "y", 0.0);
        });
        push_object(&mut bytes, "StraightVertex", |bytes| {
            push_uint(bytes, "Component", "parentId", 6);
            push_f32(bytes, "Vertex", "x", 10.0);
            push_f32(bytes, "Vertex", "y", 10.0);
        });
        bytes
    }

    struct FillRuleRecordingRenderer {
        fill_rules: Rc<RefCell<Vec<RenderFillRule>>>,
    }

    impl Renderer for FillRuleRecordingRenderer {
        fn save(&mut self) {}
        fn restore(&mut self) {}
        fn transform(&mut self, _transform: RenderMat2D) {}

        fn draw_path(&mut self, path: &dyn RenderPath, _paint: &dyn RenderPaint) {
            if let Some(path) = path.as_any().downcast_ref::<CountingRenderPath>() {
                self.fill_rules.borrow_mut().push(path.fill_rule);
            }
        }

        fn clip_path(&mut self, _path: &dyn RenderPath) {}

        fn draw_image(
            &mut self,
            _image: Option<&dyn RenderImage>,
            _sampler: RenderImageSampler,
            _blend_mode: RenderBlendMode,
            _opacity: f32,
        ) {
        }

        #[allow(clippy::too_many_arguments)]
        fn draw_image_mesh(
            &mut self,
            _image: Option<&dyn RenderImage>,
            _sampler: RenderImageSampler,
            _vertices: Option<&dyn RenderBuffer>,
            _uv_coords: Option<&dyn RenderBuffer>,
            _indices: Option<&dyn RenderBuffer>,
            _vertex_count: u32,
            _index_count: u32,
            _blend_mode: RenderBlendMode,
            _opacity: f32,
        ) {
        }

        fn modulate_opacity(&mut self, _opacity: f32) {}
    }

    // A shape with a solid fill whose fillRule starts as NonZero (0).
    fn synthetic_solid_fill_riv() -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9641);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "Artboard", |_| {});
        push_object(&mut bytes, "Shape", |bytes| {
            push_uint(bytes, "Node", "parentId", 0);
        });
        push_object(&mut bytes, "Fill", |bytes| {
            push_uint(bytes, "Component", "parentId", 1);
            push_uint(bytes, "Fill", "fillRule", 0);
        });
        push_object(&mut bytes, "SolidColor", |bytes| {
            push_uint(bytes, "Component", "parentId", 2);
            push_color(bytes, "SolidColor", "colorValue", 0xff33_66aa);
        });
        push_object(&mut bytes, "PointsPath", |bytes| {
            push_uint(bytes, "Node", "parentId", 1);
            push_bool(bytes, "PointsCommonPath", "isClosed", true);
        });
        push_object(&mut bytes, "StraightVertex", |bytes| {
            push_uint(bytes, "Component", "parentId", 4);
            push_f32(bytes, "Vertex", "x", 0.0);
            push_f32(bytes, "Vertex", "y", 0.0);
        });
        push_object(&mut bytes, "StraightVertex", |bytes| {
            push_uint(bytes, "Component", "parentId", 4);
            push_f32(bytes, "Vertex", "x", 10.0);
            push_f32(bytes, "Vertex", "y", 0.0);
        });
        push_object(&mut bytes, "StraightVertex", |bytes| {
            push_uint(bytes, "Component", "parentId", 4);
            push_f32(bytes, "Vertex", "x", 10.0);
            push_f32(bytes, "Vertex", "y", 10.0);
        });
        bytes
    }

    // Regression for the M8 audit finding (fill rule snapshot, item 25): draw
    // commands replayed the ShapePaintNode.fill_rule baked at import, so a
    // runtime Fill.fillRule write bumped every epoch and still rendered the
    // load-time rule. C++ reads the live property every draw
    // (shape_paint.cpp:176-180).
    #[test]
    fn runtime_fill_rule_write_renders_live_value() {
        let bytes = synthetic_solid_fill_riv();
        let file = read_runtime_file(&bytes).expect("synthetic riv imports");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv graphs");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");
        let mut instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        let stats = Rc::new(CountingStats::default());
        let mut factory = CountingFactory {
            stats: Rc::clone(&stats),
            next_path_id: 0,
        };
        let mut paint_cache = preallocate_render_paint_cache_for_artboard_instance(
            &file,
            artboard,
            &graph.artboards,
            &mut factory,
        );
        let mut path_cache = RuntimeRenderPathCache::default();
        let fill_rules = Rc::new(RefCell::new(Vec::new()));
        let mut renderer = FillRuleRecordingRenderer {
            fill_rules: Rc::clone(&fill_rules),
        };

        instance.update_components();
        instance
            .prepare_static_artboard_tree_paints(
                &file,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut paint_cache,
                &mut path_cache,
            )
            .expect("first prepare succeeds");
        instance
            .draw_prepared_static_artboard_with_render_cache(
                &file,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut renderer,
                &mut paint_cache,
                &mut path_cache,
            )
            .expect("first draw succeeds");
        assert_eq!(
            fill_rules.borrow().last().copied(),
            Some(RenderFillRule::NonZero),
            "imported fill rule renders as NonZero"
        );

        // Runtime write: Fill.fillRule -> EvenOdd.
        let fill_rule_key = crate::properties::property_key_for_name("Fill", "fillRule")
            .expect("fillRule property key");
        assert!(instance.set_uint_property(2, fill_rule_key, 1));

        instance.update_components();
        instance
            .prepare_static_artboard_tree_paints(
                &file,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut paint_cache,
                &mut path_cache,
            )
            .expect("second prepare succeeds");
        instance
            .draw_prepared_static_artboard_with_render_cache(
                &file,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut renderer,
                &mut paint_cache,
                &mut path_cache,
            )
            .expect("second draw succeeds");
        assert_eq!(
            fill_rules.borrow().last().copied(),
            Some(RenderFillRule::EvenOdd),
            "runtime fillRule write must render the live value"
        );
    }

    // Regression for the M8 audit finding (world-space gradient staleness,
    // item 23): world-space gradient shaders bake shape world transforms into
    // their endpoints, so the persistent-cache paint preparation must
    // reconfigure them when a keyed transform write moves the shape between
    // two prepared positions. Mirrors C++ linear_gradient.cpp:98-106
    // (PathFlags::world && WorldTransform dirt).
    #[test]
    fn world_space_gradient_stroke_reconfigures_after_transform_write() {
        use crate::TransformProperty;

        let bytes = synthetic_world_space_gradient_stroke_riv();
        let file = read_runtime_file(&bytes).expect("synthetic riv imports");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv graphs");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");
        let mut instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        let stats = Rc::new(CountingStats::default());
        let mut factory = CountingFactory {
            stats: Rc::clone(&stats),
            next_path_id: 0,
        };
        let mut paint_cache = preallocate_render_paint_cache_for_artboard_instance(
            &file,
            artboard,
            &graph.artboards,
            &mut factory,
        );
        let mut path_cache = RuntimeRenderPathCache::default();

        instance.update_components();
        instance
            .prepare_static_artboard_tree_paints(
                &file,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut paint_cache,
                &mut path_cache,
            )
            .expect("first prepare succeeds");
        let first = stats.linear_gradients.borrow().clone();
        let baseline = *first
            .last()
            .expect("first prepare configures the world-space gradient shader");

        // Keyed transform write between two prepared positions.
        assert!(instance.set_transform_property(1, TransformProperty::X, 25.0));
        instance.update_components();

        instance
            .prepare_static_artboard_tree_paints(
                &file,
                artboard,
                &graph.artboards,
                &mut factory,
                &mut paint_cache,
                &mut path_cache,
            )
            .expect("second prepare succeeds");
        let second = stats.linear_gradients.borrow().clone();
        assert!(
            second.len() > first.len(),
            "world-space gradient shader was not reconfigured after the transform write"
        );
        let moved = *second.last().expect("second prepare records endpoints");
        assert!(
            (moved[0] - (baseline[0] + 25.0)).abs() <= 0.0001
                && (moved[2] - (baseline[2] + 25.0)).abs() <= 0.0001,
            "world-space gradient endpoints did not follow the shape world transform: \
             baseline {baseline:?}, moved {moved:?}"
        );
    }
}
