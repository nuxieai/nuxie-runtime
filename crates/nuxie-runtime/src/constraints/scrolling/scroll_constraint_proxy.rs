//! Direct owner for pinned `src/constraints/scrolling/scroll_constraint_proxy.cpp`.

use super::super::*;

pub(in crate::constraints) fn start(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
) {
    proxy.viewport_is_dragging = false;
    let local = artboard.component_at(proxy.constraint).local_id;
    if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
        return;
    }
    proxy.last_position = position;
    let direction = constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
    if let Some(scroll) = artboard
        .objects
        .component_mut(proxy.constraint)
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

pub(in crate::constraints) fn drag(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
    delta: (f32, f32),
    timestamp: f32,
) -> bool {
    let local = artboard.component_at(proxy.constraint).local_id;
    if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
        return false;
    }
    if !proxy.viewport_is_dragging {
        let direction =
            constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
        let threshold = constraint_double(artboard, local, "ScrollConstraint", "threshold", 0.0);
        let crossed = match direction {
            0 => delta.0.abs() > threshold,
            1 => delta.1.abs() > threshold,
            2 => delta.0.hypot(delta.1) > threshold,
            _ => false,
        };
        if !crossed {
            return false;
        }
        proxy.viewport_is_dragging = true;
    }
    drag_view(artboard, proxy.constraint, delta, timestamp);
    proxy.last_position = position;
    true
}

pub(in crate::constraints) fn end(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
) {
    let local = artboard.component_at(proxy.constraint).local_id;
    if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
        return;
    }
    run_physics(artboard, proxy.constraint);
}
fn drag_view(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    delta: (f32, f32),
    timestamp: f32,
) {
    let local = artboard.component_at(constraint).local_id;
    let multiplier = constraint_double(artboard, local, "ScrollConstraint", "dragMultiplier", 1.0);
    let scaled = (delta.0 * multiplier, delta.1 * multiplier);
    let Some((offset_x, offset_y)) = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
        .map(|scroll| {
            if let Some(physics) = scroll.physics.as_mut() {
                physics.accumulate(scaled, timestamp);
            }
            (scroll.offset_x + scaled.0, scroll.offset_y + scaled.1)
        })
    else {
        return;
    };
    set_scroll_offset(artboard, constraint, RuntimeScrollAxis::X, offset_x);
    set_scroll_offset(artboard, constraint, RuntimeScrollAxis::Y, offset_y);
}
fn run_physics(artboard: &mut ArtboardInstance, constraint: ComponentHandle) {
    let local = artboard.component_at(constraint).local_id;
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
        scroll.is_dragging = false;
        if let Some(physics) = scroll.physics.as_mut() {
            physics.run(
                range_min,
                (0.0, 0.0),
                (scroll.offset_x, scroll.offset_y),
                &snapping_points,
                content_size,
                viewport_size,
            );
        }
    }
}
