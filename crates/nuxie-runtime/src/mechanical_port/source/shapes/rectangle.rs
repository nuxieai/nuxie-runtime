use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::rectangle_base::RectangleBase,
    shapes::straight_vertex::StraightVertex,
};
impl std::ops::Deref for Rectangle {
    type Target = RectangleBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Rectangle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Rectangle {
    pub const TYPE_KEY: u16 = RectangleBase::TYPE_KEY;
}

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
        self.base.mark_path_dirty(true);
    }
    pub fn corner_radius_tr_changed(&mut self) {
        self.base.mark_path_dirty(true);
    }
    pub fn corner_radius_bl_changed(&mut self) {
        self.base.mark_path_dirty(true);
    }
    pub fn corner_radius_br_changed(&mut self) {
        self.base.mark_path_dirty(true);
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
                vertex.set_x(ox);
                vertex.set_y(oy);
                vertex.set_radius(radius);
            }
            {
                let mut vertex = self.vertices[1].borrow_mut();
                vertex.set_x(ox + w);
                vertex.set_y(oy);
                vertex.set_radius(if link {
                    radius
                } else {
                    self.base.corner_radius_tr()
                });
            }
            {
                let mut vertex = self.vertices[2].borrow_mut();
                vertex.set_x(ox + w);
                vertex.set_y(oy + h);
                vertex.set_radius(if link {
                    radius
                } else {
                    self.base.corner_radius_br()
                });
            }
            {
                let mut vertex = self.vertices[3].borrow_mut();
                vertex.set_x(ox);
                vertex.set_y(oy + h);
                vertex.set_radius(if link {
                    radius
                } else {
                    self.base.corner_radius_bl()
                });
            }
        }
    }
}
use std::{cell::RefCell, rc::Rc};

impl Default for Rectangle {
    fn default() -> Self {
        Self::new(RectangleBase::default())
    }
}
