//! Runtime owner for pinned C++ `IntrinsicallySizeable` dispatch.

use crate::ArtboardInstance;
use crate::components::ComponentHandle;
use crate::draw::RuntimeLayoutBounds;
use nuxie_graph::ArtboardGraph;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeIntrinsicallySizeable {
    TransformComponent(ComponentHandle),
    Joystick(ComponentHandle),
}

impl RuntimeIntrinsicallySizeable {
    /// Literal `IntrinsicallySizeable::from`: TransformComponent wins before
    /// the sibling Joystick check (`src/intrinsically_sizeable.cpp:8-20`).
    pub(crate) fn from_component(
        artboard: &ArtboardInstance,
        component: ComponentHandle,
    ) -> Option<Self> {
        let owner = artboard.component_at(component);
        if owner.capabilities.transform {
            Some(Self::TransformComponent(component))
        } else if owner.type_name == "Joystick" {
            Some(Self::Joystick(component))
        } else {
            None
        }
    }

    pub(crate) fn is_joystick(self) -> bool {
        matches!(self, Self::Joystick(_))
    }
}

impl ArtboardInstance {
    pub(crate) fn control_runtime_layout_joysticks(
        &mut self,
        graph: &ArtboardGraph,
        layout_bounds: &BTreeMap<usize, RuntimeLayoutBounds>,
    ) -> bool {
        let mut controls = Vec::new();
        for layout in graph
            .components
            .iter()
            .filter(|component| component.type_name == "LayoutComponent")
        {
            let Some(bounds) = layout_bounds.get(&layout.local_id).copied() else {
                continue;
            };
            let Some(layout_handle) = self.component_handle(layout.local_id) else {
                continue;
            };
            let owner = self.component_at(layout_handle);
            if owner.is_collapsed()
                || owner
                    .concrete
                    .layout
                    .as_ref()
                    .and_then(|layout| layout.style)
                    .is_none()
            {
                continue;
            }
            self.collect_layout_joystick_controls(layout_handle, bounds, &mut controls);
        }

        controls
            .into_iter()
            .fold(false, |changed, (local_id, bounds)| {
                self.control_runtime_joystick_size(local_id, bounds.width, bounds.height) | changed
            })
    }

    fn collect_layout_joystick_controls(
        &self,
        container: ComponentHandle,
        bounds: RuntimeLayoutBounds,
        controls: &mut Vec<(usize, RuntimeLayoutBounds)>,
    ) {
        for index in 0..self.component_child_len(container) {
            let Some(child) = self.component_child_at(container, index) else {
                continue;
            };
            let owner = self.component_at(child);
            if owner.type_name == "LayoutComponent" || owner.type_name == "Node" {
                continue;
            }
            if let Some(sizeable) = RuntimeIntrinsicallySizeable::from_component(self, child) {
                if sizeable.is_joystick() {
                    controls.push((owner.local_id, bounds));
                    // Joystick::shouldPropagateSizeToChildren is false.
                    continue;
                }
                if owner.type_name == "NSlicedNode" {
                    // The other pinned false override is owned by FL-E2.
                    continue;
                }
            }
            if self.component_child_len(child) != 0 {
                self.collect_layout_joystick_controls(child, bounds, controls);
            }
        }
    }
}
