//! Computes size using styles and measure functions

use crate::geometry::{Point, Size};
use crate::style::{AvailableSpace, Overflow, Position};
use crate::tree::{CollapsibleMarginSet, RunMode};
use crate::tree::{LayoutInput, LayoutOutput, SizingMode};
use crate::util::debug::debug_log;
use crate::util::sys::f32_max;
use crate::util::MaybeMath;
use crate::util::{MaybeResolve, ResolveOrZero};
use crate::{BoxSizing, CoreStyle};
use core::unreachable;
use crate::tree::rive_measure_cache::{MeasureGeometry, MeasureRequest, RiveMeasureCache};
use crate::tree::RiveMeasureMetadata;

/// Compute the size of a leaf node (node with no children)
pub fn compute_leaf_layout<MeasureFunction>(
    inputs: LayoutInput,
    style: &impl CoreStyle,
    resolve_calc_value: impl Fn(*const (), f32) -> f32,
    measure_function: MeasureFunction,
) -> LayoutOutput
where
    MeasureFunction: FnOnce(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>,
{
    compute_leaf_layout_with_rive_measurement(inputs, style, resolve_calc_value, measure_function, false)
}

/// Rive's Yoga boundary may omit a measured leaf callback when both axes are
/// exact. Generic Taffy still measures to obtain CSS intrinsic content size.
pub(crate) fn compute_leaf_layout_with_rive_measurement<MeasureFunction>(
    inputs: LayoutInput,
    style: &impl CoreStyle,
    resolve_calc_value: impl Fn(*const (), f32) -> f32,
    measure_function: MeasureFunction,
    rive_measured_leaf: bool,
) -> LayoutOutput
where
    MeasureFunction: FnOnce(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>,
{
    compute_leaf_layout_cached(inputs, style, resolve_calc_value, measure_function, rive_measured_leaf, None)
}

pub(crate) fn compute_leaf_layout_cached<MeasureFunction>(
    inputs: LayoutInput,
    style: &impl CoreStyle,
    resolve_calc_value: impl Fn(*const (), f32) -> f32,
    measure_function: MeasureFunction,
    rive_measured_leaf: bool,
    mut cache: Option<(&mut RiveMeasureCache, RiveMeasureMetadata, u64)>,
) -> LayoutOutput
where
    MeasureFunction: FnOnce(Size<Option<f32>>, Size<AvailableSpace>) -> Size<f32>,
{
    let LayoutInput { known_dimensions, parent_size, available_space, sizing_mode, run_mode, .. } = inputs;

    // Note: both horizontal and vertical percentage padding/borders are resolved against the container's inline size (i.e. width).
    // This is not a bug, but is how CSS is specified (see: https://developer.mozilla.org/en-US/docs/Web/CSS/padding#values)
    let margin = style.margin().resolve_or_zero(parent_size.width, &resolve_calc_value);
    let padding = style.padding().resolve_or_zero(parent_size.width, &resolve_calc_value);
    let border = style.border().resolve_or_zero(parent_size.width, &resolve_calc_value);
    let padding_border = padding + border;
    let pb_sum = padding_border.sum_axes();
    let box_sizing_adjustment = if style.box_sizing() == BoxSizing::ContentBox { pb_sum } else { Size::ZERO };

    // Resolve node's preferred/min/max sizes (width/heights) against the available space (percentages resolve to pixel values)
    // For ContentSize mode, we pretend that the node has no size styles as these should be ignored.
    let (mut node_size, node_min_size, node_max_size, aspect_ratio) = match sizing_mode {
        SizingMode::ContentSize => {
            let node_size = known_dimensions;
            let node_min_size = Size::NONE;
            let node_max_size = Size::NONE;
            (node_size, node_min_size, node_max_size, None)
        }
        SizingMode::InherentSize => {
            let aspect_ratio = style.aspect_ratio();
            let style_size = style
                .size()
                .maybe_resolve(parent_size, &resolve_calc_value)
                .maybe_apply_aspect_ratio(aspect_ratio)
                .maybe_add(box_sizing_adjustment);
            let style_min_size = style
                .min_size()
                .maybe_resolve(parent_size, &resolve_calc_value)
                .maybe_apply_aspect_ratio(aspect_ratio)
                .maybe_add(box_sizing_adjustment);
            let style_max_size =
                style.max_size().maybe_resolve(parent_size, &resolve_calc_value).maybe_add(box_sizing_adjustment);

            let node_size = known_dimensions.or(style_size);
            (node_size, style_min_size, style_max_size, aspect_ratio)
        }
    };

    if inputs.rive_undefined_available.width.is_some() { node_size.width = None; }
    if inputs.rive_undefined_available.height.is_some() { node_size.height = None; }

    // Scrollbar gutters are reserved when the `overflow` property is set to `Overflow::Scroll`.
    // However, the axis are switched (transposed) because a node that scrolls vertically needs
    // *horizontal* space to be reserved for a scrollbar
    let scrollbar_gutter = style.overflow().transpose().map(|overflow| match overflow {
        Overflow::Scroll => style.scrollbar_width(),
        _ => 0.0,
    });
    // TODO: make side configurable based on the `direction` property
    let mut content_box_inset = padding_border;
    content_box_inset.right += scrollbar_gutter.x;
    content_box_inset.bottom += scrollbar_gutter.y;

    let has_styles_preventing_being_collapsed_through = !style.is_block()
        || style.overflow().x.is_scroll_container()
        || style.overflow().y.is_scroll_container()
        || style.position() == Position::Absolute
        || padding.top > 0.0
        || padding.bottom > 0.0
        || border.top > 0.0
        || border.bottom > 0.0
        || matches!(node_size.height, Some(h) if h > 0.0)
        || matches!(node_min_size.height, Some(h) if h > 0.0);

    debug_log!("LEAF");
    debug_log!("node_size", dbg:node_size);
    debug_log!("min_size ", dbg:node_min_size);
    debug_log!("max_size ", dbg:node_max_size);

    // The cache sees Yoga's outer available dimensions, not the inset-stripped
    // callback offers. Exact dimensions include margins in that coordinate.
    let axis = |known: Option<f32>, available: AvailableSpace, margin: f32| {
        if let Some(size) = known { (1, size + margin) } else {
            match available {
                AvailableSpace::MaxContent => (0, f32::NAN),
                AvailableSpace::MinContent => (2, 0.0),
                AvailableSpace::Definite(size) => (2, size),
            }
        }
    };
    let margins = margin.sum_axes();
    let (wm, w) = inputs.rive_undefined_available.width.map(|v| (0, v))
        .unwrap_or_else(|| axis(node_size.width, available_space.width, margins.width));
    let (hm, h) = inputs.rive_undefined_available.height.map(|v| (0, v))
        .unwrap_or_else(|| axis(node_size.height, available_space.height, margins.height));
    let request = MeasureRequest {
        modes: Size { width: wm, height: hm }, available: Size { width: w, height: h }, margin: margins,
        geometry: MeasureGeometry { min: node_min_size, max: node_max_size, inset: content_box_inset, padding, border, aspect: aspect_ratio },
    };
    if let Some((cache, metadata, generation)) = cache.as_mut() {
        if let Some(output) = cache.get(request, *metadata, *generation) {
            cache.finish(request, output, run_mode, *generation, true);
            return output;
        }
    }

    // Return early if both width and height are known
    if run_mode == RunMode::ComputeSize && has_styles_preventing_being_collapsed_through {
        if let Size { width: Some(width), height: Some(height) } = node_size {
            let size = Size { width, height }
                .maybe_clamp(node_min_size, node_max_size)
                .maybe_max(padding_border.sum_axes().map(Some));
            let output = LayoutOutput {
                size,
                #[cfg(feature = "content_size")]
                content_size: Size::ZERO,
                first_baselines: Point::NONE,
                top_margin: CollapsibleMarginSet::ZERO,
                bottom_margin: CollapsibleMarginSet::ZERO,
                margins_can_collapse_through: false,
            };
            if let Some((cache, _, generation)) = cache.as_mut() {
                cache.finish(request, output, run_mode, *generation, false);
            }
            return output;
        };
    }

    // Compute available space
    let available_space = Size {
        width: known_dimensions
            .width
            .map(AvailableSpace::from)
            .unwrap_or(available_space.width)
            .maybe_sub(margin.horizontal_axis_sum())
            .maybe_set(known_dimensions.width)
            .maybe_set(node_size.width)
            .map_definite_value(|size| {
                size.maybe_clamp(node_min_size.width, node_max_size.width) - content_box_inset.horizontal_axis_sum()
            }),
        height: known_dimensions
            .height
            .map(AvailableSpace::from)
            .unwrap_or(available_space.height)
            .maybe_sub(margin.vertical_axis_sum())
            .maybe_set(known_dimensions.height)
            .maybe_set(node_size.height)
            .map_definite_value(|size| {
                size.maybe_clamp(node_min_size.height, node_max_size.height) - content_box_inset.vertical_axis_sum()
            }),
    };

    // Measure node
    let measured_size = if rive_measured_leaf
        && run_mode == RunMode::PerformLayout
        && !style.is_block()
        && node_size.width.is_some()
        && node_size.height.is_some()
    {
        // Yoga::YGNodeWithMeasureFuncSetMeasuredDimensions skips measurement
        // for Exactly/Exactly. Keep the normal final sizing path below; only
        // intrinsic content size is omitted at this non-CSS runtime boundary.
        Size::ZERO
    } else {
        measure_function(
            if rive_measured_leaf && !style.is_block() {
                // The completed cache and Yoga callback must agree about
                // Exactly axes, including dimensions resolved from style.
                // These remain outer dimensions; available_space below has
                // already had padding/border removed for the intrinsic offer.
                node_size
            } else {
                match run_mode {
                    RunMode::ComputeSize => known_dimensions,
                    RunMode::PerformLayout => Size::NONE,
                    RunMode::PerformHiddenLayout => unreachable!(),
                }
            },
            available_space,
        )
    };
    let clamped_size = known_dimensions
        .or(node_size)
        .unwrap_or(measured_size + content_box_inset.sum_axes())
        .maybe_clamp(node_min_size, node_max_size);
    let size = Size {
        width: clamped_size.width,
        height: f32_max(clamped_size.height, aspect_ratio.map(|ratio| clamped_size.width / ratio).unwrap_or(0.0)),
    };
    let size = size.maybe_max(padding_border.sum_axes().map(Some));

    let output = LayoutOutput {
        size,
        #[cfg(feature = "content_size")]
        content_size: measured_size + padding.sum_axes(),
        first_baselines: Point::NONE,
        top_margin: CollapsibleMarginSet::ZERO,
        bottom_margin: CollapsibleMarginSet::ZERO,
        margins_can_collapse_through: !has_styles_preventing_being_collapsed_through
            && size.height == 0.0
            && measured_size.height == 0.0,
    };
    if let Some((cache, _, generation)) = cache.as_mut() {
        cache.finish(request, output, run_mode, *generation, false);
    }
    output
}
