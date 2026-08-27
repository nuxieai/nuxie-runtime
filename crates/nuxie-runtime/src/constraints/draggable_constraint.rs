//! Direct owner for pinned `src/constraints/draggable_constraint.cpp`.

use super::*;

/// One proxy/listener-group pair constructed by a concrete C++
/// `DraggableConstraint::listenerGroups` call. Every StateMachineInstance owns
/// a fresh set; the constraint and hittable remain non-owning occurrence
/// handles (`draggable_constraint.cpp:8-28`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeDraggableProxyKind {
    Viewport,
    Thumb,
    Track,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeDraggableProxy {
    pub(crate) constraint: ComponentHandle,
    pub(crate) hittable: ComponentHandle,
    pub(crate) kind: RuntimeDraggableProxyKind,
    pub(crate) opaque: bool,
    pub(crate) last_position: (f32, f32),
    pub(crate) viewport_is_dragging: bool,
    pub(crate) active_pointers: Vec<i32>,
    pub(crate) has_scrolled: bool,
}

impl RuntimeDraggableProxy {
    pub(in crate::constraints) fn new(
        constraint: ComponentHandle,
        hittable: ComponentHandle,
        kind: RuntimeDraggableProxyKind,
        opaque: bool,
    ) -> Self {
        Self {
            constraint,
            hittable,
            kind,
            opaque,
            last_position: (0.0, 0.0),
            viewport_is_dragging: false,
            active_pointers: Vec::new(),
            has_scrolled: false,
        }
    }

    pub(crate) fn clone_cold(&self) -> Self {
        Self::new(self.constraint, self.hittable, self.kind, self.opaque)
    }
}

/// Construct the exact component-provided draggable groups in authored
/// occurrence order for one StateMachineInstance
/// (`state_machine_instance.cpp:1969-2013`;
/// `draggable_constraint.cpp:8-28`).
pub(crate) fn runtime_draggable_proxies(artboard: &ArtboardInstance) -> Vec<RuntimeDraggableProxy> {
    let mut proxies = Vec::new();
    for component in artboard.components().iter() {
        let Some(handle) = artboard.component_handle(component.local_id) else {
            continue;
        };
        if let Some(scroll) = component.concrete.scroll.as_ref()
            && let Some(viewport) = scroll
                .content
                .and_then(|content| artboard.objects.component(content)?.parent)
        {
            proxies.push(scrolling::scroll_constraint_proxy::new(handle, viewport));
        }
        if component.concrete.scroll_bar.is_some() {
            scrolling::scroll_bar_constraint::append_proxies(artboard, handle, &mut proxies);
        }
    }
    proxies
}
