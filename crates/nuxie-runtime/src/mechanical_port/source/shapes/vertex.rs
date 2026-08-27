use crate::mechanical_port::source::{
    bones::weight::Weight,
    generated::shapes::vertex_base::VertexBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

pub struct VertexState {
    weight: Option<*mut Weight>,
}
impl Default for VertexState {
    fn default() -> Self {
        Self { weight: None }
    }
}

pub trait Vertex {
    fn vertex_base(&self) -> &VertexBase;
    fn vertex_state(&self) -> &VertexState;
    fn vertex_state_mut(&mut self) -> &mut VertexState;
    fn mark_geometry_dirty(&mut self);
    fn set_weight(&mut self, weight: &mut Weight) {
        self.vertex_state_mut().weight = Some(weight);
    }
    fn has_weight(&self) -> bool {
        self.vertex_state().weight.is_some()
    }
    fn render_translation(&self) -> Vec2D {
        self.vertex_state().weight.map_or(
            Vec2D::new(self.vertex_base().x(), self.vertex_base().y()),
            |weight| unsafe { (*weight).translation() },
        )
    }
    fn x_changed(&mut self) {
        self.mark_geometry_dirty();
    }
    fn y_changed(&mut self) {
        self.mark_geometry_dirty();
    }
    fn deform(&mut self, world: &Mat2D, bone_transforms: *const f32) {
        let weight = unsafe { &mut *self.vertex_state().weight.unwrap() };
        *weight.translation_mut() = Weight::deform(
            Vec2D::new(self.vertex_base().x(), self.vertex_base().y()),
            weight.indices(),
            weight.values(),
            world,
            bone_transforms,
        );
    }
}
