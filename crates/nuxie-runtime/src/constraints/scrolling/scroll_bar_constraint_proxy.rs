//! Direct owner for pinned `src/constraints/scrolling/scroll_bar_constraint_proxy.cpp`.

use super::super::*;
use super::scroll_bar_constraint;

pub(in crate::constraints) fn start(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    position: (f32, f32),
    timestamp: f32,
) {
    match proxy.kind {
        RuntimeDraggableProxyKind::Thumb => {
            proxy.last_position = position;
            let Some(scroll_constraint) = artboard
                .objects
                .component(proxy.constraint)
                .and_then(|component| component.concrete.scroll_bar.as_ref())
                .and_then(|bar| bar.scroll_constraint)
            else {
                return;
            };
            if let Some(scroll) = artboard
                .objects
                .component_mut(scroll_constraint)
                .and_then(|component| component.concrete.scroll.as_mut())
            {
                if !scroll.is_scroll_bar_dragging {
                    scroll.intent_x = None;
                    scroll.intent_y = None;
                    scroll.last_frame_offset_x = scroll.offset_x;
                    scroll.last_frame_offset_y = scroll.offset_y;
                }
                scroll.is_scroll_bar_dragging = true;
                if let Some(physics) = scroll.physics.as_mut() {
                    physics.accumulate((0.0, 0.0), timestamp);
                }
            }
        }
        RuntimeDraggableProxyKind::Track => {
            scroll_bar_constraint::hit_track(artboard, proxy.constraint, position);
        }
        RuntimeDraggableProxyKind::Viewport => {}
    }
}

pub(in crate::constraints) fn drag(
    artboard: &mut ArtboardInstance,
    proxy: &mut RuntimeDraggableProxy,
    delta: (f32, f32),
    position: (f32, f32),
    timestamp: f32,
) -> bool {
    match proxy.kind {
        RuntimeDraggableProxyKind::Thumb => {
            scroll_bar_constraint::drag_thumb(artboard, proxy.constraint, delta, timestamp);
            proxy.last_position = position;
            true
        }
        RuntimeDraggableProxyKind::Track => true,
        RuntimeDraggableProxyKind::Viewport => false,
    }
}

pub(in crate::constraints) fn end(artboard: &mut ArtboardInstance, proxy: &RuntimeDraggableProxy) {
    if proxy.kind != RuntimeDraggableProxyKind::Thumb {
        return;
    }
    let Some(scroll_constraint) = artboard
        .objects
        .component(proxy.constraint)
        .and_then(|component| component.concrete.scroll_bar.as_ref())
        .and_then(|bar| bar.scroll_constraint)
    else {
        return;
    };
    if let Some(scroll) = artboard
        .objects
        .component_mut(scroll_constraint)
        .and_then(|component| component.concrete.scroll.as_mut())
    {
        scroll.is_scroll_bar_dragging = false;
        if let Some(physics) = scroll.physics.as_mut() {
            physics.clear_velocity();
        }
    }
}
