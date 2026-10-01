//! Alignment of tracks and final positioning of items
use super::types::GridTrack;
use crate::compute::common::alignment::{
    apply_alignment_fallback, compute_alignment_offset, resolve_self_alignment_safety,
};
use crate::geometry::{InBothAbsAxis, Line, Point, Rect, Size};
use crate::style::{
    AlignContent, AlignItems, AlignItemsKeyword, AlignSelf, AvailableSpace, CoreStyle, GridItemStyle, Overflow,
    Position,
};
use crate::tree::{Layout, LayoutPartialTreeExt, NodeId, SizingMode};
use crate::util::sys::f32_max;
use crate::util::{MaybeMath, MaybeResolve, ResolveOrZero};

#[cfg(feature = "content_size")]
use crate::compute::common::content_size::compute_content_size_contribution;
use crate::{BoxSizing, Direction, LayoutGridContainer};

/// Align the grid tracks within the grid according to the align-content (rows) or
/// justify-content (columns) property. This only does anything if the size of the
/// grid is not equal to the size of the grid container in the axis being aligned.
pub(super) fn align_tracks(
    grid_container_content_box_size: f32,
    padding: Line<f32>,
    border: Line<f32>,
    tracks: &mut [GridTrack],
    track_alignment_style: AlignContent,
    axis_is_reversed: bool,
) {
    let used_size: f32 = tracks.iter().map(|track| track.base_size).sum();
    let free_space = grid_container_content_box_size - used_size;
    let origin = padding.start + border.start;

    // Count the number of non-collapsed tracks (not counting gutters)
    let num_tracks = tracks.iter().skip(1).step_by(2).filter(|track| !track.is_collapsed).count();

    // Grid layout treats gaps as full tracks rather than applying them at alignment so we
    // simply pass zero here. Grid layout is never reversed.
    let gap = 0.0;
    let layout_is_reversed = false;
    let track_alignment = apply_alignment_fallback(free_space, num_tracks, track_alignment_style);
    let track_alignment = if axis_is_reversed { track_alignment.reversed() } else { track_alignment };

    // Compute offsets
    let mut total_offset = origin;
    let mut seen_non_collapsed_track = false;
    tracks.iter_mut().enumerate().for_each(|(i, track)| {
        // Odd tracks are gutters (but slices are zero-indexed, so odd tracks have even indices)
        let is_gutter = i % 2 == 0;
        let is_non_collapsed_track = !is_gutter && !track.is_collapsed;

        // Alignment offsets should be applied only to non-collapsed tracks.
        let is_first = is_non_collapsed_track && !seen_non_collapsed_track;

        let offset = if is_non_collapsed_track {
            compute_alignment_offset(free_space, num_tracks, gap, track_alignment, layout_is_reversed, is_first)
        } else {
            0.0
        };

        track.offset = total_offset + offset;
        total_offset = total_offset + offset + track.base_size;
        if is_non_collapsed_track {
            seen_non_collapsed_track = true;
        }
    });
}

/// Align and size a grid item into it's final position
pub(super) fn align_and_position_item(
    tree: &mut impl LayoutGridContainer,
    node: NodeId,
    order: u32,
    grid_area: Rect<f32>,
    container_alignment_styles: InBothAbsAxis<Option<AlignItems>>,
    baseline_shim: f32,
    direction: Direction,
    rive_absolute_percentage_basis: Size<Option<f32>>,
    rive_container_is_row: bool,
) -> (Size<f32>, f32, f32) {
    let grid_area_size = Size { width: grid_area.right - grid_area.left, height: grid_area.bottom - grid_area.top };
    // Yoga's grid absolute pass retains finite Undefined offers for resolving
    // percentages, independently of the content-sized positioning rectangle.
    // Inflow items and ordinary CSS layouts pass Size::NONE.
    let percentage_basis = rive_absolute_percentage_basis.unwrap_or(grid_area_size);

    let style = tree.get_grid_child_style(node);

    let overflow = style.overflow();
    let scrollbar_width = style.scrollbar_width();
    let aspect_ratio = style.aspect_ratio();
    let justify_self = style.justify_self();
    let align_self = style.align_self();

    let position = style.position();
    let rive_absolute = position == Position::Absolute
        && (rive_absolute_percentage_basis.width.is_some() || rive_absolute_percentage_basis.height.is_some());
    let inset_horizontal = style
        .inset()
        .horizontal_components()
        .map(|size| size.resolve_to_option(percentage_basis.width, |val, basis| tree.calc(val, basis)));
    let inset_vertical = style
        .inset()
        .vertical_components()
        .map(|size| size.resolve_to_option(percentage_basis.height, |val, basis| tree.calc(val, basis)));
    let mut padding =
        style.padding().map(|p| p.resolve_or_zero(Some(percentage_basis.width), |val, basis| tree.calc(val, basis)));
    let mut border =
        style.border().map(|p| p.resolve_or_zero(Some(percentage_basis.width), |val, basis| tree.calc(val, basis)));
    let padding_border_size = (padding + border).sum_axes();

    let box_sizing_adjustment =
        if style.box_sizing() == BoxSizing::ContentBox { padding_border_size } else { Size::ZERO };

    let inherent_size = style
        .size()
        .maybe_resolve(percentage_basis, |val, basis| tree.calc(val, basis))
        .maybe_apply_aspect_ratio(aspect_ratio)
        .maybe_add(box_sizing_adjustment);
    let min_size = style
        .min_size()
        .maybe_resolve(percentage_basis, |val, basis| tree.calc(val, basis))
        .maybe_add(box_sizing_adjustment)
        .or(padding_border_size.map(Some))
        .maybe_max(padding_border_size)
        .maybe_apply_aspect_ratio(aspect_ratio);
    let max_size = style
        .max_size()
        .maybe_resolve(percentage_basis, |val, basis| tree.calc(val, basis))
        .maybe_apply_aspect_ratio(aspect_ratio)
        .maybe_add(box_sizing_adjustment);

    // Resolve default alignment styles if they are set on neither the parent or the node itself
    // Note: if the child has a preferred aspect ratio but neither width or height are set, then the width is stretched
    // and the then height is calculated from the width according the aspect ratio
    // See: https://www.w3.org/TR/css-grid-1/#grid-item-sizing
    let alignment_styles = InBothAbsAxis {
        horizontal: justify_self.or(container_alignment_styles.horizontal).unwrap_or_else(|| {
            if inherent_size.width.is_some() {
                AlignSelf::START
            } else {
                AlignSelf::STRETCH
            }
        }),
        vertical: align_self.or(container_alignment_styles.vertical).unwrap_or_else(|| {
            if inherent_size.height.is_some() || aspect_ratio.is_some() {
                AlignSelf::START
            } else {
                AlignSelf::STRETCH
            }
        }),
    };

    // Note: This is not a bug. It is part of the CSS spec that both horizontal and vertical margins
    // resolve against the WIDTH of the grid area.
    let margin = style
        .margin()
        .map(|margin| margin.resolve_to_option(percentage_basis.width, |val, basis| tree.calc(val, basis)));
    let margin = if rive_absolute { margin.map(|value| Some(value.unwrap_or(0.0))) } else { margin };

    let grid_area_minus_item_margins_size = Size {
        width: grid_area_size.width.maybe_sub(margin.left).maybe_sub(margin.right),
        height: grid_area_size.height.maybe_sub(margin.top).maybe_sub(margin.bottom) - baseline_shim,
    };

    let Size { width, height } = if rive_absolute {
        // YGNodeAbsoluteLayoutChild resolves both authored/inset axes before
        // deriving a missing axis from aspect ratio. Initial style dimensions
        // are not clamped here: the child applies its bounds with its own
        // requested dimensions as the owner basis during layout below.
        let mut requested = style
            .size()
            .maybe_resolve(percentage_basis, |val, basis| tree.calc(val, basis))
            .maybe_add(box_sizing_adjustment);
        let source_min = style.min_size().maybe_resolve(percentage_basis, |val, basis| tree.calc(val, basis));
        let source_max = style.max_size().maybe_resolve(percentage_basis, |val, basis| tree.calc(val, basis));
        if requested.width.is_none() {
            if let (Some(left), Some(right)) = (inset_horizontal.start, inset_horizontal.end) {
                requested.width = Some(
                    (grid_area_size.width - left - right)
                        .maybe_clamp(source_min.width, source_max.width)
                        .max(padding_border_size.width)
                        .maybe_sub(margin.left)
                        .maybe_sub(margin.right),
                );
            }
        }
        if requested.height.is_none() {
            if let (Some(top), Some(bottom)) = (inset_vertical.start, inset_vertical.end) {
                requested.height = Some(
                    (grid_area_size.height - top - bottom)
                        .maybe_clamp(source_min.height, source_max.height)
                        .max(padding_border_size.height)
                        .maybe_sub(margin.top)
                        .maybe_sub(margin.bottom),
                );
            }
        }
        requested.maybe_apply_aspect_ratio(aspect_ratio)
    } else {
        // If node is absolutely positioned and width is not set explicitly, then deduce it
        // from left, right and container_content_box if both are set.
        let width = inherent_size.width.or_else(|| {
            // Apply width derived from both the left and right properties of an absolutely
            // positioned element being set
            if position == Position::Absolute {
                if let (Some(left), Some(right)) = (inset_horizontal.start, inset_horizontal.end) {
                    return Some(f32_max(grid_area_minus_item_margins_size.width - left - right, 0.0));
                }
            }

            // Apply width based on stretch alignment if:
            //  - Alignment style is "stretch"
            //  - The node is not absolutely positioned
            //  - The node does not have auto margins in this axis.
            if margin.left.is_some()
                && margin.right.is_some()
                && alignment_styles.horizontal == AlignSelf::STRETCH
                && position != Position::Absolute
            {
                return Some(grid_area_minus_item_margins_size.width);
            }

            None
        });

        // Reapply aspect ratio after stretch and absolute position width adjustments
        let Size { width, height } =
            Size { width, height: inherent_size.height }.maybe_apply_aspect_ratio(aspect_ratio);

        let height = height.or_else(|| {
            if position == Position::Absolute {
                if let (Some(top), Some(bottom)) = (inset_vertical.start, inset_vertical.end) {
                    return Some(f32_max(grid_area_minus_item_margins_size.height - top - bottom, 0.0));
                }
            }

            // Apply height based on stretch alignment if:
            //  - Alignment style is "stretch"
            //  - The node is not absolutely positioned
            //  - The node does not have auto margins in this axis.
            if margin.top.is_some()
                && margin.bottom.is_some()
                && alignment_styles.vertical == AlignSelf::STRETCH
                && position != Position::Absolute
            {
                return Some(grid_area_minus_item_margins_size.height);
            }

            None
        });
        // Reapply aspect ratio after stretch and absolute position height adjustments
        let Size { width, height } = Size { width, height }.maybe_apply_aspect_ratio(aspect_ratio);

        // Clamp size by min and max width/height
        Size { width, height }.maybe_clamp(min_size, max_size)
    };

    // Layout node
    drop(style);

    let mut rive_layout_margins = None;
    let (layout_output, size) = if rive_absolute {
        // Yoga's request includes margins, and its owner basis is that same
        // request, not the grid's resolution basis. Re-resolve child margins
        // against it before mapping to Taffy's margin-excluded known sizes.
        let initial_margins = margin.map(|value| value.unwrap_or(0.0)).sum_axes();
        let mut request = Size { width, height }.maybe_add(initial_margins);
        if width.is_none() || height.is_none() {
            let constrain_width = !rive_container_is_row
                && width.is_none()
                && rive_absolute_percentage_basis.width.is_none()
                && percentage_basis.width > 0.0;
            if constrain_width {
                request.width = Some(percentage_basis.width);
            }
            let child_style = tree.get_grid_child_style(node);
            let child_margins =
                child_style.margin().resolve_or_zero(request.width, |val, basis| tree.calc(val, basis)).sum_axes();
            let mut known = request.maybe_sub(child_margins);
            if constrain_width {
                known.width = None;
            }
            drop(child_style);
            let measured = tree.measure_child_size_both(
                node,
                known,
                request,
                request.map(|value| value.map(AvailableSpace::Definite).unwrap_or(AvailableSpace::MaxContent)),
                SizingMode::InherentSize,
                Line::FALSE,
            );
            request = (measured + initial_margins).map(Some);
        }
        let child_style = tree.get_grid_child_style(node);
        let resolved_child_margins =
            child_style.margin().resolve_or_zero(request.width, |val, basis| tree.calc(val, basis));
        let child_margins = resolved_child_margins.sum_axes();
        rive_layout_margins = Some(resolved_child_margins);
        padding = child_style.padding().resolve_or_zero(request.width, |val, basis| tree.calc(val, basis));
        border = child_style.border().resolve_or_zero(request.width, |val, basis| tree.calc(val, basis));
        drop(child_style);
        let output = tree.perform_child_layout(
            node,
            request.maybe_sub(child_margins),
            request,
            request.map(|value| value.map(AvailableSpace::Definite).unwrap_or(AvailableSpace::MaxContent)),
            SizingMode::InherentSize,
            Line::FALSE,
        );
        (output, output.size)
    } else {
        let size = if position == Position::Absolute && (width.is_none() || height.is_none()) {
            tree.measure_child_size_both(
                node,
                Size { width, height },
                percentage_basis.map(Option::Some),
                grid_area_minus_item_margins_size.map(AvailableSpace::Definite),
                SizingMode::InherentSize,
                Line::FALSE,
            )
            .map(Some)
        } else {
            Size { width, height }
        };

        let layout_output = tree.perform_child_layout(
            node,
            size,
            percentage_basis.map(Option::Some),
            grid_area_minus_item_margins_size.map(AvailableSpace::Definite),
            SizingMode::InherentSize,
            Line::FALSE,
        );

        // Resolve final size
        (layout_output, size.unwrap_or(layout_output.size).maybe_clamp(min_size, max_size))
    };
    let Size { width, height } = size;

    let (x, x_margin) = align_item_within_area(
        Line { start: grid_area.left, end: grid_area.right },
        justify_self.unwrap_or(alignment_styles.horizontal),
        width,
        position,
        inset_horizontal,
        margin.horizontal_components(),
        0.0,
        direction,
    );
    let (y, y_margin) = align_item_within_area(
        Line { start: grid_area.top, end: grid_area.bottom },
        align_self.unwrap_or(alignment_styles.vertical),
        height,
        position,
        inset_vertical,
        margin.vertical_components(),
        baseline_shim,
        Direction::Ltr,
    );

    let scrollbar_size = Size {
        width: if overflow.y == Overflow::Scroll { scrollbar_width } else { 0.0 },
        height: if overflow.x == Overflow::Scroll { scrollbar_width } else { 0.0 },
    };

    let resolved_margin = Rect { left: x_margin.start, right: x_margin.end, top: y_margin.start, bottom: y_margin.end };

    tree.set_unrounded_layout(
        node,
        &Layout {
            order,
            location: Point { x, y },
            size: Size { width, height },
            #[cfg(feature = "content_size")]
            content_size: layout_output.content_size,
            scrollbar_size,
            padding,
            border,
            margin: rive_layout_margins.unwrap_or(resolved_margin),
        },
    );

    #[cfg(feature = "content_size")]
    let contribution = compute_content_size_contribution(
        Point { x: x - grid_area.left, y: y - grid_area.top },
        Size { width, height },
        layout_output.content_size,
        overflow,
    );
    #[cfg(not(feature = "content_size"))]
    let contribution = Size::ZERO;

    (contribution, y, height)
}

/// Align and size a grid item along a single axis
#[allow(clippy::too_many_arguments)]
pub(super) fn align_item_within_area(
    grid_area: Line<f32>,
    alignment_style: AlignSelf,
    resolved_size: f32,
    position: Position,
    inset: Line<Option<f32>>,
    margin: Line<Option<f32>>,
    baseline_shim: f32,
    direction: Direction,
) -> (f32, Line<f32>) {
    // Calculate grid area dimension in the axis
    let non_auto_margin = Line { start: margin.start.unwrap_or(0.0) + baseline_shim, end: margin.end.unwrap_or(0.0) };
    let grid_area_size = f32_max(grid_area.end - grid_area.start, 0.0);
    let free_space = f32_max(grid_area_size - resolved_size - non_auto_margin.sum(), 0.0);

    // Expand auto margins to fill available space
    let auto_margin_count = margin.start.is_none() as u8 + margin.end.is_none() as u8;
    let auto_margin_size = if auto_margin_count > 0 { free_space / auto_margin_count as f32 } else { 0.0 };
    let resolved_margin = Line {
        start: margin.start.unwrap_or(auto_margin_size) + baseline_shim,
        end: margin.end.unwrap_or(auto_margin_size),
    };

    let overflows = resolved_size + non_auto_margin.sum() > grid_area_size;
    let alignment_keyword = resolve_self_alignment_safety(alignment_style, overflows);

    // Compute offset in the axis
    let alignment_based_offset = match alignment_keyword {
        // TODO: Add support for baseline alignment. For now we treat it as "start".
        AlignItemsKeyword::Start
        | AlignItemsKeyword::FlexStart
        | AlignItemsKeyword::Baseline
        | AlignItemsKeyword::Stretch => {
            if direction.is_rtl() {
                grid_area_size - resolved_size - resolved_margin.end
            } else {
                resolved_margin.start
            }
        }
        AlignItemsKeyword::End | AlignItemsKeyword::FlexEnd => {
            if direction.is_rtl() {
                resolved_margin.start
            } else {
                grid_area_size - resolved_size - resolved_margin.end
            }
        }
        AlignItemsKeyword::Center => {
            (grid_area_size - resolved_size + resolved_margin.start - resolved_margin.end) / 2.0
        }
    };

    let offset_within_area = if position == Position::Absolute {
        match (inset.start, inset.end) {
            (Some(start), Some(end)) => {
                if direction.is_rtl() {
                    grid_area_size - end - resolved_size - non_auto_margin.end
                } else {
                    start + non_auto_margin.start
                }
            }
            (Some(start), None) => start + non_auto_margin.start,
            (None, Some(end)) => grid_area_size - end - resolved_size - non_auto_margin.end,
            (None, None) => alignment_based_offset,
        }
    } else {
        alignment_based_offset
    };

    let mut start = grid_area.start + offset_within_area;
    if position == Position::Relative {
        let relative_inset = if direction.is_rtl() {
            inset.end.map(|pos| -pos).or(inset.start)
        } else {
            inset.start.or(inset.end.map(|pos| -pos))
        };
        start += relative_inset.unwrap_or(0.0);
    }

    (start, resolved_margin)
}
