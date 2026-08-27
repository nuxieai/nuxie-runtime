use crate::mechanical_port::source::{
    component::ComponentDirt,
    generated::world_transform_component_base::WorldTransformComponentBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

pub struct WorldTransformComponent {
    pub base: WorldTransformComponentBase,
    world_transform: Mat2D,
}

impl WorldTransformComponent {
    pub fn child_opacity(&self) -> f32 {
        self.base.opacity()
    }

    pub fn mark_world_transform_dirty(&mut self) {
        self.base.add_dirt(ComponentDirt::WORLD_TRANSFORM, true);
    }

    pub fn world_transform(&self) -> &Mat2D {
        &self.world_transform
    }

    pub fn mutable_world_transform(&mut self) -> &mut Mat2D {
        &mut self.world_transform
    }

    pub fn world_translation(&self) -> Vec2D {
        self.world_transform.translation()
    }

    pub fn opacity_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::RENDER_OPACITY, true);
    }
}
