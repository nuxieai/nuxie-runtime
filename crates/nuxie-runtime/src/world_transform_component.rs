use super::ArtboardInstance;
use crate::components::{ComponentDirt, RuntimeComponent};

impl RuntimeComponent {
    pub(crate) fn child_opacity(&self, authored_opacity: f32) -> f32 {
        // `WorldTransformComponent::childOpacity` returns its authored
        // opacity, while `TransformComponent` overrides it with settled
        // render opacity (`src/world_transform_component.cpp:8`,
        // `include/rive/transform_component.hpp:42`).
        if self.capabilities.transform {
            self.transform.render_opacity
        } else {
            authored_opacity
        }
    }
}

impl ArtboardInstance {
    /// Direct `WorldTransformComponent::opacityChanged` dispatch.
    pub(super) fn mark_world_transform_opacity_dirty(&mut self, local_id: usize) -> bool {
        self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true)
    }

    /// Rust render-preparation observer for a settled
    /// `WorldTransformComponent::m_WorldTransform` change.
    pub(super) fn mark_world_transform_changed(&mut self) {
        self.prepared_epoch = self.prepared_epoch.wrapping_add(1);
        self.mark_tree_paint_preparation_changed();
    }
}
