//! Direct owner for pinned `src/constraints/scrolling/scroll_constraint.cpp`.

use super::super::*;

pub(in crate::constraints) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint_handle: ComponentHandle,
) -> bool {
    // `ScrollConstraint::constrain` / `constrainChild`.
    let constraint_local = artboard.component_at(constraint_handle).local_id;
    let Some(content) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| scroll.content)
    else {
        return false;
    };
    if component_index != content {
        return false;
    }
    let retained_layout_bounds = artboard.retained_layout_bounds_arc();
    let layout_bounds = retained_layout_bounds.as_deref();
    artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("ScrollConstraint occurrence retains its concrete state")
        .layout_initialized = true;
    let intent_changed = {
        let scroll_constraint = artboard
            .objects
            .component(constraint_handle)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint occurrence retains its concrete state");
        if scroll_constraint.intent_x.is_some() || scroll_constraint.intent_y.is_some() {
            let include_item_bounds = scroll_constraint
                .intent_x
                .into_iter()
                .chain(scroll_constraint.intent_y)
                .any(|intent| intent.space == RuntimeScrollSpace::Index);
            let scroll_metrics = build_runtime_scroll_layout_metrics(
                artboard,
                constraint_handle,
                scroll_constraint,
                layout_bounds,
                include_item_bounds,
            );
            resolve_runtime_scroll_intents(artboard, constraint_local, &scroll_metrics)
        } else {
            false
        }
    };
    let scroll_constraint = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
        .expect("ScrollConstraint occurrence retains its concrete state");
    let metrics = build_runtime_scroll_layout_metrics(
        artboard,
        constraint_handle,
        scroll_constraint,
        layout_bounds,
        false,
    );
    let (clamped_x, clamped_y) =
        clamped_scroll_constraint_offsets(artboard, constraint_handle, &metrics);
    let offset_x = if metrics.constrains_horizontal() {
        clamped_x
    } else {
        0.0
    };
    let offset_y = if metrics.constrains_vertical() {
        clamped_y
    } else {
        0.0
    };
    let scroll_transform = Mat2D([1.0, 0.0, 0.0, 1.0, offset_x, offset_y]);
    let scroll = artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("ScrollConstraint occurrence retains its concrete state");
    scroll.scroll_transform = scroll_transform;
    scroll.child_constraint_applied_count = 0;
    intent_changed
}

pub(super) fn drag_view(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    delta: (f32, f32),
    timestamp: f32,
) {
    let local = artboard.component_at(constraint).local_id;
    let multiplier = constraint_double(artboard, local, "ScrollConstraint", "dragMultiplier", 1.0);
    let scaled = (delta.0 * multiplier, delta.1 * multiplier);
    let Some((mut offset_x, mut offset_y, has_physics)) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .map(|scroll| {
            if let Some(physics) = scroll.physics.as_mut() {
                physics.accumulate(scaled, timestamp);
            }
            (
                scroll.offset_x + scaled.0,
                scroll.offset_y + scaled.1,
                scroll.physics.is_some(),
            )
        })
    else {
        return;
    };
    if !has_physics {
        let metrics = artboard
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.scroll.as_ref())
            .map(|scroll| {
                runtime_scroll_layout_metrics(artboard, constraint, scroll, false).unwrap_or_else(
                    || {
                        build_runtime_scroll_layout_metrics(
                            artboard, constraint, scroll, None, false,
                        )
                    },
                )
            });
        if let Some(metrics) = metrics
            && !metrics.infinite
        {
            // Without physics no later owner pulls the stored offset back
            // into range. Clamp now so overscroll cannot eat the next drag.
            offset_x = rive_math_clamp(offset_x, metrics.max_offset(RuntimeScrollAxis::X), 0.0);
            offset_y = rive_math_clamp(offset_y, metrics.max_offset(RuntimeScrollAxis::Y), 0.0);
        }
    }
    set_scroll_offset(artboard, constraint, RuntimeScrollAxis::X, offset_x);
    set_scroll_offset(artboard, constraint, RuntimeScrollAxis::Y, offset_y);
}

pub(super) fn run_physics(artboard: &mut ArtboardInstance, constraint: ComponentHandle) {
    let local = artboard.component_at(constraint).local_id;
    if let Some(scroll) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
    {
        scroll.is_dragging = false;
    }
    let Some(scroll) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
    else {
        return;
    };
    let snap = constraint_bool(artboard, local, "ScrollConstraint", "snap", false);
    let metrics =
        runtime_scroll_layout_metrics(artboard, constraint, scroll, snap).unwrap_or_else(|| {
            build_runtime_scroll_layout_metrics(artboard, constraint, scroll, None, snap)
        });
    let snapping_points = if snap {
        metrics
            .item_bounds
            .iter()
            .filter(|bounds| !metrics.bounds_collapsed(**bounds))
            .map(|bounds| (bounds.x, bounds.y))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let range_min = (
        metrics.max_offset(RuntimeScrollAxis::X),
        metrics.max_offset(RuntimeScrollAxis::Y),
    );
    let range_max = (
        metrics.min_offset(RuntimeScrollAxis::X),
        metrics.min_offset(RuntimeScrollAxis::Y),
    );
    let content_size = if metrics.main_axis_horizontal {
        metrics.content_width
    } else {
        metrics.content_height
    };
    let viewport_size = if metrics.main_axis_horizontal {
        metrics.viewport_width
    } else {
        metrics.viewport_height
    };
    if let Some(scroll) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
    {
        if let Some(physics) = scroll.physics.as_mut() {
            physics.run(
                range_min,
                range_max,
                (scroll.offset_x, scroll.offset_y),
                &snapping_points,
                content_size,
                viewport_size,
            );
        }
    }
}

pub(super) fn init_physics(artboard: &mut ArtboardInstance, constraint: ComponentHandle) {
    let local = artboard.component_at(constraint).local_id;
    let direction = constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
    if let Some(scroll) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
    {
        scroll.is_dragging = true;
        scroll.intent_x = None;
        scroll.intent_y = None;
        scroll.last_frame_offset_x = scroll.offset_x;
        scroll.last_frame_offset_y = scroll.offset_y;
        if let Some(physics) = scroll.physics.as_mut() {
            physics.prepare(direction);
        }
    }
}
