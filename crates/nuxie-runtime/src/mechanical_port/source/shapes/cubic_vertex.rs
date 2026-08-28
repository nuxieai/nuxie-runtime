use crate::mechanical_port::source::{
    bones::{cubic_weight::CubicWeight, weight::Weight},
    generated::shapes::cubic_vertex_base::CubicVertexBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::vertex::VertexBehavior,
};

#[derive(Default)]
pub struct CubicVertexState {
    pub in_valid: bool,
    pub out_valid: bool,
    pub in_point: Vec2D,
    pub out_point: Vec2D,
}

#[derive(Default)]
pub struct CubicVertex {
    pub base: CubicVertexBase,
    pub(crate) state: CubicVertexState,
}

impl VertexBehavior for CubicVertex {
    fn vertex(&self) -> &crate::mechanical_port::source::shapes::vertex::Vertex {
        &self.base.base.base
    }

    fn vertex_mut(&mut self) -> &mut crate::mechanical_port::source::shapes::vertex::Vertex {
        &mut self.base.base.base
    }

    fn mark_geometry_dirty(&mut self) {
        self.base.mark_geometry_dirty();
    }
}

pub trait CubicVertexBehavior: VertexBehavior {
    fn cubic_vertex(&self) -> &CubicVertex;
    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex;
    fn compute_in(&mut self);
    fn compute_out(&mut self);
    fn in_point(&mut self) -> Vec2D {
        if !self.cubic_vertex().state.in_valid {
            self.compute_in();
            self.cubic_vertex_mut().state.in_valid = true;
        }
        self.cubic_vertex().state.in_point
    }
    fn out_point(&mut self) -> Vec2D {
        if !self.cubic_vertex().state.out_valid {
            self.compute_out();
            self.cubic_vertex_mut().state.out_valid = true;
        }
        self.cubic_vertex().state.out_point
    }
    fn set_in_point(&mut self, value: Vec2D) {
        self.cubic_vertex_mut().state.in_point = value;
        self.cubic_vertex_mut().state.in_valid = true;
    }
    fn set_out_point(&mut self, value: Vec2D) {
        self.cubic_vertex_mut().state.out_point = value;
        self.cubic_vertex_mut().state.out_valid = true;
    }
    fn render_in(&mut self) -> Vec2D {
        self.cubic_vertex()
            .base
            .weight_handle()
            .and_then(|weight| {
                weight.with_downcast_mut::<CubicWeight, _>(|weight| *weight.in_translation())
            })
            .unwrap_or_else(|| self.in_point())
    }
    fn render_out(&mut self) -> Vec2D {
        self.cubic_vertex()
            .base
            .weight_handle()
            .and_then(|weight| {
                weight.with_downcast_mut::<CubicWeight, _>(|weight| *weight.out_translation())
            })
            .unwrap_or_else(|| self.out_point())
    }
    fn x_changed(&mut self) {
        VertexBehavior::x_changed(self);
        self.cubic_vertex_mut().state.in_valid = false;
        self.cubic_vertex_mut().state.out_valid = false;
    }
    fn y_changed(&mut self) {
        VertexBehavior::y_changed(self);
        self.cubic_vertex_mut().state.in_valid = false;
        self.cubic_vertex_mut().state.out_valid = false;
    }
    fn deform(&mut self, world: &Mat2D, bones: &[f32]) {
        VertexBehavior::deform(self, world, bones);
        let in_point = self.in_point();
        let out_point = self.out_point();
        if let Some(weight) = self.cubic_vertex().base.weight_handle() {
            weight.with_downcast_mut::<CubicWeight, _>(|weight| {
                *weight.in_translation() = Weight::deform(
                    in_point,
                    weight.in_indices(),
                    weight.in_values(),
                    world,
                    bones,
                );
                *weight.out_translation() = Weight::deform(
                    out_point,
                    weight.out_indices(),
                    weight.out_values(),
                    world,
                    bones,
                );
            });
        }
    }
}

impl CubicVertexBehavior for CubicVertex {
    fn cubic_vertex(&self) -> &CubicVertex {
        self
    }

    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
        self
    }

    fn compute_in(&mut self) {
        let point = Vec2D::new(self.base.x(), self.base.y());
        self.state.in_point = point;
    }

    fn compute_out(&mut self) {
        let point = Vec2D::new(self.base.x(), self.base.y());
        self.state.out_point = point;
    }
}

impl std::ops::Deref for CubicVertex {
    type Target = CubicVertexBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
