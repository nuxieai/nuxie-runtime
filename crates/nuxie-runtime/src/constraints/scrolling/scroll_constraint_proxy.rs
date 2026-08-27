//! Direct owner for pinned `src/constraints/scrolling/scroll_constraint_proxy.cpp`
//! and its handwritten `scroll_constraint_proxy.hpp` bodies.

use super::{super::*, scroll_constraint};

// The unified Rust draggable proxy retains the C++ owner's `m_constraint`,
// `m_lastPosition`, and `m_isDragging` state. Rust's drop is the empty
// destructor.
pub(in crate::constraints) fn new(
    constraint: ComponentHandle,
    hittable: ComponentHandle,
) -> RuntimeDraggableProxy {
    RuntimeDraggableProxy::new(
        constraint,
        hittable,
        RuntimeDraggableProxyKind::Viewport,
        false,
    )
}

pub(in crate::constraints) fn drag(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
    _dispatcher_delta: (f32, f32),
    timestamp: f32,
) -> bool {
    let local = artboard.component_at(proxy.constraint).local_id;
    if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
        return false;
    }
    // The shared Rust dispatcher precomputes a delta for every proxy kind;
    // this owner deliberately derives it again at the pinned ownership site.
    let delta = (
        position.0 - proxy.last_position.0,
        position.1 - proxy.last_position.1,
    );
    if !proxy.viewport_is_dragging {
        let direction =
            constraint_uint(artboard, local, "DraggableConstraint", "directionValue", 1);
        let threshold = constraint_double(artboard, local, "ScrollConstraint", "threshold", 0.0);
        match direction {
            0 => {
                if delta.0.abs() > threshold {
                    proxy.viewport_is_dragging = true;
                } else {
                    return false;
                }
            }
            1 => {
                if delta.1.abs() > threshold {
                    proxy.viewport_is_dragging = true;
                } else {
                    return false;
                }
            }
            2 => {
                if (delta.0 * delta.0 + delta.1 * delta.1).sqrt() > threshold {
                    proxy.viewport_is_dragging = true;
                } else {
                    return false;
                }
            }
            // The pinned enum conversion does not validate the authored byte;
            // an unknown value simply takes no switch case and falls through.
            _ => {}
        }
    }
    scroll_constraint::drag_view(artboard, proxy.constraint, delta, timestamp);
    proxy.last_position = position;
    true
}

pub(in crate::constraints) fn start(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
) -> bool {
    // The upstream timestamp is unused; the shared dispatcher elides it for
    // this proxy kind.
    let local = artboard.component_at(proxy.constraint).local_id;
    if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
        return false;
    }
    proxy.viewport_is_dragging = false;
    scroll_constraint::init_physics(artboard, proxy.constraint);
    proxy.last_position = position;
    true
}

pub(in crate::constraints) fn end(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
) -> bool {
    // Both upstream arguments are unused; the shared dispatcher elides them.
    let local = artboard.component_at(proxy.constraint).local_id;
    if !constraint_bool(artboard, local, "ScrollConstraint", "interactive", true) {
        return false;
    }
    scroll_constraint::run_physics(artboard, proxy.constraint);
    true
}
