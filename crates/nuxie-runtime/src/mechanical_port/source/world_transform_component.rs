use crate::mechanical_port::source::{
    generated::core_registry::CoreCapabilities,
    generated::world_transform_component_base::{
        WorldTransformComponentBase, WorldTransformComponentBaseCallbacks,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

#[derive(Default)]
pub struct WorldTransformComponent {
    pub base: WorldTransformComponentBase,
    world_transform: Mat2D,
}

impl WorldTransformComponentBaseCallbacks for WorldTransformComponent {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn opacity_changed(&mut self) {
        WorldTransformComponent::opacity_changed(self);
    }
}

impl WorldTransformComponent {
    pub fn child_opacity(&self) -> f32 {
        self.base.opacity()
    }

    pub fn mark_world_transform_dirty(&mut self) {
        CoreCapabilities::world_transform_mark_dirty(self);
    }

    pub fn world_transform(&self) -> &Mat2D {
        &self.world_transform
    }

    pub fn mutable_world_transform(&mut self) -> &mut Mat2D {
        &mut self.world_transform
    }

    pub fn set_world_transform(&mut self, transform: Mat2D) {
        self.world_transform = transform;
    }

    pub fn world_translation(&self) -> Vec2D {
        self.world_transform.translation()
    }

    pub fn opacity_changed(&mut self) {
        CoreCapabilities::world_transform_opacity_changed(self);
    }
}

impl std::ops::Deref for WorldTransformComponent {
    type Target = WorldTransformComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for WorldTransformComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
