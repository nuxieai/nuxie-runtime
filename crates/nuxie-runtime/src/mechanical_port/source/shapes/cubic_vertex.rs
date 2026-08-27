use crate::mechanical_port::source::{
    bones::{cubic_weight::CubicWeight, weight::Weight},
    generated::shapes::cubic_vertex_base::CubicVertexBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

#[derive(Default)]
pub struct CubicVertexState {
    pub in_valid: bool,
    pub out_valid: bool,
    pub in_point: Vec2D,
    pub out_point: Vec2D,
}
pub trait CubicVertex {
    fn cubic_base(&self) -> &CubicVertexBase;
    fn cubic_base_mut(&mut self) -> &mut CubicVertexBase;
    fn cubic_state(&self) -> &CubicVertexState;
    fn cubic_state_mut(&mut self) -> &mut CubicVertexState;
    fn compute_in(&mut self);
    fn compute_out(&mut self);
    fn in_point(&mut self) -> Vec2D {
        if !self.cubic_state().in_valid {
            self.compute_in();
            self.cubic_state_mut().in_valid = true;
        }
        self.cubic_state().in_point
    }
    fn out_point(&mut self) -> Vec2D {
        if !self.cubic_state().out_valid {
            self.compute_out();
            self.cubic_state_mut().out_valid = true;
        }
        self.cubic_state().out_point
    }
    fn set_in_point(&mut self, value: Vec2D) {
        self.cubic_state_mut().in_point = value;
        self.cubic_state_mut().in_valid = true;
    }
    fn set_out_point(&mut self, value: Vec2D) {
        self.cubic_state_mut().out_point = value;
        self.cubic_state_mut().out_valid = true;
    }
    fn render_in(&mut self) -> Vec2D {
        if self.cubic_base().has_weight() {
            self.cubic_base().weight::<CubicWeight>().in_translation()
        } else {
            self.in_point()
        }
    }
    fn render_out(&mut self) -> Vec2D {
        if self.cubic_base().has_weight() {
            self.cubic_base().weight::<CubicWeight>().out_translation()
        } else {
            self.out_point()
        }
    }
    fn x_changed(&mut self) {
        self.cubic_base_mut().super_x_changed();
        self.cubic_state_mut().in_valid = false;
        self.cubic_state_mut().out_valid = false;
    }
    fn y_changed(&mut self) {
        self.cubic_base_mut().super_y_changed();
        self.cubic_state_mut().in_valid = false;
        self.cubic_state_mut().out_valid = false;
    }
    fn deform(&mut self, world: &Mat2D, bones: *const f32) {
        self.cubic_base_mut().super_deform(world, bones);
        let in_point = self.in_point();
        let out_point = self.out_point();
        let weight = self.cubic_base_mut().weight_mut::<CubicWeight>();
        *weight.in_translation_mut() = Weight::deform(
            in_point,
            weight.in_indices(),
            weight.in_values(),
            world,
            bones,
        );
        *weight.out_translation_mut() = Weight::deform(
            out_point,
            weight.out_indices(),
            weight.out_values(),
            world,
            bones,
        );
    }
}
