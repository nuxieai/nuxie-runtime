use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::rectangle_base::RectangleBase,
    shapes::straight_vertex::StraightVertex,
};
pub struct Rectangle {
    pub base: RectangleBase,
    vertices: [Rc<RefCell<StraightVertex>>; 4],
}
impl Rectangle {
    pub fn new(mut base: RectangleBase) -> Self {
        let vertices = std::array::from_fn(|_| Rc::new(RefCell::new(StraightVertex::default())));
        for vertex in &vertices {
            base.add_runtime_straight_vertex(vertex.clone());
        }
        Self { base, vertices }
    }
    pub fn corner_radius_tl_changed(&mut self) {
        self.base.mark_path_dirty();
    }
    pub fn corner_radius_tr_changed(&mut self) {
        self.base.mark_path_dirty();
    }
    pub fn corner_radius_bl_changed(&mut self) {
        self.base.mark_path_dirty();
    }
    pub fn corner_radius_br_changed(&mut self) {
        self.base.mark_path_dirty();
    }
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let radius = self.base.corner_radius_tl();
            let link = self.base.link_corner_radius();
            let ox = -self.base.origin_x() * self.base.width();
            let oy = -self.base.origin_y() * self.base.height();
            let w = self.base.width();
            let h = self.base.height();
            {
                let mut vertex = self.vertices[0].borrow_mut();
                vertex.base.set_x(ox);
                vertex.base.set_y(oy);
                vertex.base.set_radius(radius);
            }
            {
                let mut vertex = self.vertices[1].borrow_mut();
                vertex.base.set_x(ox + w);
                vertex.base.set_y(oy);
                vertex.base.set_radius(if link {
                    radius
                } else {
                    self.base.corner_radius_tr()
                });
            }
            {
                let mut vertex = self.vertices[2].borrow_mut();
                vertex.base.set_x(ox + w);
                vertex.base.set_y(oy + h);
                vertex.base.set_radius(if link {
                    radius
                } else {
                    self.base.corner_radius_br()
                });
            }
            {
                let mut vertex = self.vertices[3].borrow_mut();
                vertex.base.set_x(ox);
                vertex.base.set_y(oy + h);
                vertex.base.set_radius(if link {
                    radius
                } else {
                    self.base.corner_radius_bl()
                });
            }
        }
    }
}
use std::{cell::RefCell, rc::Rc};
