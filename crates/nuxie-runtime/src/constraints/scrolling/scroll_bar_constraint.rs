//! Direct owner for pinned `src/constraints/scrolling/scroll_bar_constraint.cpp`.

use super::super::*;

pub(in crate::constraints) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint_handle: ComponentHandle,
) -> bool {
    let constraint_local = artboard.component_at(constraint_handle).local_id;
    let Some((scroll_constraint, thumb, track)) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|scroll_bar| {
            let thumb = artboard.objects.component(constraint_handle)?.parent?;
            let track = artboard.objects.component(thumb)?.parent?;
            Some((scroll_bar.scroll_constraint?, thumb, track))
        })
    else {
        return false;
    };
    if component_index != thumb {
        return false;
    }
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return false;
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let (_, _, track_width, track_height) = constraint_bounds(artboard, track);
    let (_, _, authored_thumb_width, authored_thumb_height) = constraint_bounds(artboard, thumb);
    let padding_left = layout_style_axis_leading_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        true,
    );
    let padding_right = layout_style_axis_trailing_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        true,
    );
    let padding_top = layout_style_axis_leading_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        false,
    );
    let padding_bottom = layout_style_axis_trailing_padding(
        artboard,
        layout_component_style_local(artboard, artboard.component_at(track).local_id),
        false,
    );
    let inner_width = track_width - padding_left - padding_right;
    let inner_height = track_height - padding_top - padding_bottom;
    let auto_size = constraint_bool(
        artboard,
        constraint_local,
        "ScrollBarConstraint",
        "autoSize",
        true,
    );
    let direction = constraint_uint(
        artboard,
        constraint_local,
        "DraggableConstraint",
        "directionValue",
        1,
    );
    let constrains_horizontal = matches!(direction, 0 | 2);
    let constrains_vertical = matches!(direction, 1 | 2);
    let mut thumb_offset_x = 0.0;
    let mut thumb_offset_y = 0.0;
    if constrains_horizontal {
        let mut thumb_width = if auto_size {
            inner_width
                * if metrics.content_width == 0.0 {
                    1.0
                } else {
                    (metrics.viewport_width / metrics.content_width).min(1.0)
                }
        } else {
            authored_thumb_width
        };
        let max_thumb_offset = inner_width - thumb_width;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::X);
        let clamped = clamped_scroll_constraint_offsets(artboard, scroll_constraint, &metrics).0;
        thumb_offset_x = if max_offset == 0.0 {
            0.0
        } else {
            clamped / max_offset * max_thumb_offset
        };
        if thumb_offset_x < 0.0 {
            thumb_width += thumb_offset_x;
            thumb_offset_x = 0.0;
        } else if thumb_offset_x > max_thumb_offset {
            thumb_width -= thumb_offset_x - max_thumb_offset;
            if !auto_size {
                thumb_offset_x = max_thumb_offset;
            }
        }
        if auto_size
            && let Some(layout) = artboard
                .objects
                .component(thumb)
                .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.forced_width(thumb_width);
        }
    }
    if constrains_vertical {
        let mut thumb_height = if auto_size {
            inner_height
                * if metrics.content_height == 0.0 {
                    1.0
                } else {
                    (metrics.viewport_height / metrics.content_height).min(1.0)
                }
        } else {
            authored_thumb_height
        };
        let max_thumb_offset = inner_height - thumb_height;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::Y);
        let clamped = clamped_scroll_constraint_offsets(artboard, scroll_constraint, &metrics).1;
        thumb_offset_y = if max_offset == 0.0 {
            0.0
        } else {
            clamped / max_offset * max_thumb_offset
        };
        if thumb_offset_y < 0.0 {
            thumb_height += thumb_offset_y;
            thumb_offset_y = 0.0;
        } else if thumb_offset_y > max_thumb_offset {
            thumb_height -= thumb_offset_y - max_thumb_offset;
            if !auto_size {
                thumb_offset_y = max_thumb_offset;
            }
        }
        if auto_size
            && let Some(layout) = artboard
                .objects
                .component(thumb)
                .and_then(|component| component.concrete.layout.as_ref())
        {
            layout.forced_height(thumb_height);
        }
    }
    let world = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let target = world.multiply(Mat2D([1.0, 0.0, 0.0, 1.0, thumb_offset_x, thumb_offset_y]));
    let strength = retained_constraint_double(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
        1.0,
    );
    let (components_a, components_b) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .map(|scroll_bar| (scroll_bar.components_a, scroll_bar.components_b))
        .unwrap_or_default();
    let constrained = transform_constraint::constrain_world(
        world,
        components_a,
        target,
        components_b,
        strength,
    );
    write_world_transform(artboard, component_index, constrained)
}
