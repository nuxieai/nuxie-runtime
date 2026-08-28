use crate::mechanical_port::source::{
    component::ComponentDirt,
    generated::shapes::star_base::StarBase,
    math::math_types,
    shapes::{polygon::PolygonState, straight_vertex::StraightVertex},
};
pub struct Star {
    pub base: StarBase,
    pub polygon: PolygonState,
}
impl Star {
    pub fn new(base: StarBase) -> Self {
        Self {
            base,
            polygon: PolygonState::default(),
        }
    }
    pub fn inner_radius_changed(&mut self) {
        self.base.mark_path_dirty();
    }
    pub fn vertex_count(&self) -> usize {
        self.base.points() as usize * 2
    }
    pub fn build_polygon(&mut self) {
        let half_width = self.base.width() / 2.0;
        let half_height = self.base.height() / 2.0;
        let inner_half_width = self.base.width() * self.base.inner_radius() / 2.0;
        let inner_half_height = self.base.height() * self.base.inner_radius() / 2.0;
        let ox = -self.base.origin_x() * self.base.width() + half_width;
        let oy = -self.base.origin_y() * self.base.height() + half_height;
        let length = self.vertex_count();
        let mut angle = -math_types::PI / 2.0;
        let increment = 2.0 * math_types::PI / length as f32;
        for index in (0..length).step_by(2) {
            let mut outer = self.polygon.vertices[index].borrow_mut();
            outer.base.set_x(ox + angle.cos() * half_width);
            outer.base.set_y(oy + angle.sin() * half_height);
            outer.base.set_radius(self.base.corner_radius());
            drop(outer);
            angle += increment;
            let mut inner = self.polygon.vertices[index + 1].borrow_mut();
            inner.base.set_x(ox + angle.cos() * inner_half_width);
            inner.base.set_y(oy + angle.sin() * inner_half_height);
            inner.base.set_radius(self.base.corner_radius());
            angle += increment;
        }
    }
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if value.contains(ComponentDirt::PATH) {
            if self.polygon.vertices.len() != self.vertex_count() {
                self.polygon.vertices.resize_with(self.vertex_count(), || {
                    std::rc::Rc::new(std::cell::RefCell::new(StraightVertex::default()))
                });
                self.base.clear_vertices();
                for vertex in &self.polygon.vertices {
                    self.base.add_runtime_straight_vertex(vertex.clone());
                }
            }
            self.build_polygon();
        }
    }
}
