use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::polygon_base::PolygonBase,
    math::math_types,
    shapes::straight_vertex::StraightVertex,
};
#[derive(Default)]
pub struct PolygonState {
    pub vertices: Vec<Rc<RefCell<StraightVertex>>>,
}
impl std::ops::Deref for Polygon {
    type Target = PolygonBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Polygon {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Polygon {
    pub const TYPE_KEY: u16 = PolygonBase::TYPE_KEY;
}

pub struct Polygon {
    pub base: PolygonBase,
    pub polygon: PolygonState,
}
impl Polygon {
    pub fn new(base: PolygonBase) -> Self {
        Self {
            base,
            polygon: PolygonState::default(),
        }
    }
    pub fn corner_radius_changed(&mut self) {
        self.base.mark_path_dirty(true);
    }
    pub fn points_changed(&mut self) {
        self.base.mark_path_dirty(true);
    }
    pub fn vertex_count(&self) -> usize {
        self.base.points() as usize
    }
    pub fn build_polygon(&mut self) {
        let half_width = self.base.width() / 2.0;
        let half_height = self.base.height() / 2.0;
        let ox = -self.base.origin_x() * self.base.width() + half_width;
        let oy = -self.base.origin_y() * self.base.height() + half_height;
        let mut angle = -math_types::PI / 2.0;
        let increment = 2.0 * math_types::PI / self.base.points() as f32;
        for vertex in &self.polygon.vertices {
            let mut vertex = vertex.borrow_mut();
            vertex.set_x(ox + angle.cos() * half_width);
            vertex.set_y(oy + angle.sin() * half_height);
            vertex.set_radius(self.base.corner_radius());
            angle += increment;
        }
    }
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let count = self.vertex_count();
            if self.polygon.vertices.len() != count {
                self.polygon
                    .vertices
                    .resize_with(count, || Rc::new(RefCell::new(StraightVertex::default())));
                self.base.clear_vertices();
                for vertex in &self.polygon.vertices {
                    self.base.add_runtime_straight_vertex(vertex.clone());
                }
            }
            self.build_polygon();
        }
    }
}
use std::{cell::RefCell, rc::Rc};

impl Default for Polygon {
    fn default() -> Self {
        Self::new(PolygonBase::default())
    }
}
