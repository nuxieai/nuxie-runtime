//! Direct owner for pinned `src/constraints/scrolling/scroll_bar_constraint.cpp`.

use super::super::*;

fn layout_size(artboard: &ArtboardInstance, component: ComponentHandle) -> (f32, f32) {
    artboard
        .objects
        .component(component)
        .and_then(|component| component.concrete.layout.as_ref())
        .map(|layout| {
            let (_, _, width, height) = layout.current_bounds();
            (width, height)
        })
        .unwrap_or_else(|| {
            let (_, _, width, height) = constraint_bounds(artboard, component);
            (width, height)
        })
}

fn force_thumb_width(artboard: &mut ArtboardInstance, thumb: ComponentHandle, width: f32) {
    let Some((local, changed)) = artboard.objects.component(thumb).and_then(|component| {
        let layout = component.concrete.layout.as_ref()?;
        let changed = layout
            .forced_size()
            .0
            .is_none_or(|current| current != width);
        if changed {
            layout.forced_width(width);
        }
        Some((component.local_id, changed))
    }) else {
        return;
    };
    if changed {
        crate::layout_component::mark_layout_style_dirty(artboard, local);
        crate::layout_component::mark_layout_node_dirty(artboard, local);
    }
}

fn force_thumb_height(artboard: &mut ArtboardInstance, thumb: ComponentHandle, height: f32) {
    let Some((local, changed)) = artboard.objects.component(thumb).and_then(|component| {
        let layout = component.concrete.layout.as_ref()?;
        let changed = layout
            .forced_size()
            .1
            .is_none_or(|current| current != height);
        if changed {
            layout.forced_height(height);
        }
        Some((component.local_id, changed))
    }) else {
        return;
    };
    if changed {
        crate::layout_component::mark_layout_style_dirty(artboard, local);
        crate::layout_component::mark_layout_node_dirty(artboard, local);
    }
}

pub(in crate::constraints) fn append_proxies(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    proxies: &mut Vec<RuntimeDraggableProxy>,
) {
    let Some(thumb) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.parent)
    else {
        return;
    };
    if artboard
        .objects
        .component(thumb)
        .is_some_and(|component| component.concrete.layout.is_some())
    {
        proxies.push(RuntimeDraggableProxy::new(
            constraint,
            thumb,
            RuntimeDraggableProxyKind::Thumb,
            true,
        ));
    }
    if let Some(track) = artboard
        .objects
        .component(thumb)
        .and_then(|component| component.parent)
        .filter(|track| {
            artboard
                .objects
                .component(*track)
                .is_some_and(|component| component.concrete.layout.is_some())
        })
    {
        proxies.push(RuntimeDraggableProxy::new(
            constraint,
            track,
            RuntimeDraggableProxyKind::Track,
            false,
        ));
    }
}

fn computed_thumb_width(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    scroll_constraint: ComponentHandle,
    thumb: ComponentHandle,
    track: ComponentHandle,
) -> f32 {
    let authored_width = layout_size(artboard, thumb).0;
    let constraint_local = artboard.component_at(constraint).local_id;
    if !constraint_bool(
        artboard,
        constraint_local,
        "ScrollBarConstraint",
        "autoSize",
        true,
    ) {
        return authored_width;
    }
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return authored_width;
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let track_width = layout_size(artboard, track).0;
    let style = layout_component_style_local(artboard, artboard.component_at(track).local_id);
    let inner_width = track_width
        - layout_style_axis_leading_padding(artboard, style, true)
        - layout_style_axis_trailing_padding(artboard, style, true);
    let visible_width_ratio = if metrics.content_width == 0.0 {
        1.0
    } else {
        (metrics.viewport_width / metrics.content_width).min(1.0)
    };
    inner_width * visible_width_ratio
}

fn computed_thumb_height(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    scroll_constraint: ComponentHandle,
    thumb: ComponentHandle,
    track: ComponentHandle,
) -> f32 {
    let authored_height = layout_size(artboard, thumb).1;
    let constraint_local = artboard.component_at(constraint).local_id;
    if !constraint_bool(
        artboard,
        constraint_local,
        "ScrollBarConstraint",
        "autoSize",
        true,
    ) {
        return authored_height;
    }
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return authored_height;
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let track_height = layout_size(artboard, track).1;
    let style = layout_component_style_local(artboard, artboard.component_at(track).local_id);
    let inner_height = track_height
        - layout_style_axis_leading_padding(artboard, style, false)
        - layout_style_axis_trailing_padding(artboard, style, false);
    let visible_height_ratio = if metrics.content_height == 0.0 {
        1.0
    } else {
        (metrics.viewport_height / metrics.content_height).min(1.0)
    };
    inner_height * visible_height_ratio
}

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
    let (track_width, track_height) = layout_size(artboard, track);
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
        let mut thumb_width =
            computed_thumb_width(artboard, constraint_handle, scroll_constraint, thumb, track);
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
        if auto_size {
            force_thumb_width(artboard, thumb, thumb_width);
        }
    }
    if constrains_vertical {
        let mut thumb_height =
            computed_thumb_height(artboard, constraint_handle, scroll_constraint, thumb, track);
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
        if auto_size {
            force_thumb_height(artboard, thumb, thumb_height);
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
    let constrained =
        transform_constraint::constrain_world(world, components_a, target, components_b, strength);
    write_world_transform(artboard, component_index, constrained)
}

pub(in crate::constraints) fn hit_track(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    world_position: (f32, f32),
) {
    let Some((scroll_constraint, thumb, track)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|bar| {
            let thumb = artboard.objects.component(constraint)?.parent?;
            let track = artboard.objects.component(thumb)?.parent?;
            Some((bar.scroll_constraint?, thumb, track))
        })
    else {
        return;
    };
    let local = artboard.component_at(constraint).local_id;
    let direction = constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return;
    };
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let Some(inverse) = artboard
        .component_at(track)
        .transform
        .world_transform
        .invert()
    else {
        return;
    };
    let local_position = inverse.transform_point(world_position.0, world_position.1);
    let style = layout_component_style_local(artboard, artboard.component_at(track).local_id);
    let (track_width, track_height) = layout_size(artboard, track);
    if matches!(direction, 0 | 2) {
        let local_x = local_position.0 - layout_style_axis_leading_padding(artboard, style, true);
        let thumb_width =
            computed_thumb_width(artboard, constraint, scroll_constraint, thumb, track);
        let track_range = track_width
            - layout_style_axis_leading_padding(artboard, style, true)
            - layout_style_axis_trailing_padding(artboard, style, true)
            - thumb_width;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::X);
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::X,
            rive_math_clamp(local_x / track_range * max_offset, max_offset, 0.0),
        );
    }
    if matches!(direction, 1 | 2) {
        let local_y = local_position.1 - layout_style_axis_leading_padding(artboard, style, false);
        let thumb_height =
            computed_thumb_height(artboard, constraint, scroll_constraint, thumb, track);
        let track_range = track_height
            - layout_style_axis_leading_padding(artboard, style, false)
            - layout_style_axis_trailing_padding(artboard, style, false)
            - thumb_height;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::Y);
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::Y,
            rive_math_clamp(local_y / track_range * max_offset, max_offset, 0.0),
        );
    }
}

pub(in crate::constraints) fn drag_thumb(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    delta: (f32, f32),
    timestamp: f32,
) {
    let Some((scroll_constraint, thumb, track)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|bar| {
            let thumb = artboard.objects.component(constraint)?.parent?;
            let track = artboard.objects.component(thumb)?.parent?;
            Some((bar.scroll_constraint?, thumb, track))
        })
    else {
        return;
    };
    let local = artboard.component_at(constraint).local_id;
    let direction = constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
    let Some(scroll) = artboard
        .objects
        .component(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return;
    };
    let previous = (scroll.offset_x, scroll.offset_y);
    let metrics = runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, false)
        .unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, scroll_constraint, scroll, None, false)
        });
    let (track_width, track_height) = layout_size(artboard, track);
    let auto_size = constraint_bool(artboard, local, "ScrollBarConstraint", "autoSize", true);
    let style = layout_component_style_local(artboard, artboard.component_at(track).local_id);
    if matches!(direction, 0 | 2) {
        let thumb_width =
            computed_thumb_width(artboard, constraint, scroll_constraint, thumb, track);
        if auto_size {
            force_thumb_width(artboard, thumb, thumb_width);
        }
        let track_range = track_width
            - layout_style_axis_leading_padding(artboard, style, true)
            - layout_style_axis_trailing_padding(artboard, style, true)
            - thumb_width;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::X);
        let thumb_offset = previous.0 / max_offset * track_range + delta.0;
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::X,
            rive_math_clamp(thumb_offset / track_range * max_offset, max_offset, 0.0),
        );
    }
    if matches!(direction, 1 | 2) {
        let thumb_height =
            computed_thumb_height(artboard, constraint, scroll_constraint, thumb, track);
        if auto_size {
            force_thumb_height(artboard, thumb, thumb_height);
        }
        let track_range = track_height
            - layout_style_axis_leading_padding(artboard, style, false)
            - layout_style_axis_trailing_padding(artboard, style, false)
            - thumb_height;
        let max_offset = metrics.max_offset(RuntimeScrollAxis::Y);
        let thumb_offset = previous.1 / max_offset * track_range + delta.1;
        set_scroll_offset(
            artboard,
            scroll_constraint,
            RuntimeScrollAxis::Y,
            rive_math_clamp(thumb_offset / track_range * max_offset, max_offset, 0.0),
        );
    }
    if let Some(scroll) = artboard
        .objects
        .component_mut(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        && let Some(physics) = scroll.physics.as_mut()
    {
        physics.accumulate(
            (scroll.offset_x - previous.0, scroll.offset_y - previous.1),
            timestamp,
        );
    }
}
