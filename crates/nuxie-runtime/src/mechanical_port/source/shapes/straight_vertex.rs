use crate::mechanical_port::source::generated::shapes::straight_vertex_base::StraightVertexBase;
#[derive(Default)]
pub struct StraightVertex {
    pub base: StraightVertexBase,
}
impl StraightVertex {
    pub fn radius_changed(&mut self) {
        self.base.mark_geometry_dirty();
    }
}
