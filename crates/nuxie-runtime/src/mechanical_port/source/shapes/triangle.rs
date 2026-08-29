use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::triangle_base::TriangleBase,
    shapes::straight_vertex::StraightVertex,
};
impl std::ops::Deref for Triangle {
    type Target = TriangleBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Triangle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Triangle {
    pub const TYPE_KEY: u16 = TriangleBase::TYPE_KEY;
}

pub struct Triangle {
    pub base: TriangleBase,
    vertices: [Rc<RefCell<StraightVertex>>; 3],
}
impl Triangle {
    pub fn new(mut base: TriangleBase) -> Self {
        let vertices = std::array::from_fn(|_| Rc::new(RefCell::new(StraightVertex::default())));
        for vertex in &vertices {
            base.add_runtime_straight_vertex(vertex.clone());
        }
        Self { base, vertices }
    }
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let ox = -self.base.origin_x() * self.base.width();
            let oy = -self.base.origin_y() * self.base.height();
            let w = self.base.width();
            let h = self.base.height();
            {
                let mut vertex = self.vertices[0].borrow_mut();
                vertex.set_x(ox + w / 2.0);
                vertex.set_y(oy);
            }
            {
                let mut vertex = self.vertices[1].borrow_mut();
                vertex.set_x(ox + w);
                vertex.set_y(oy + h);
            }
            {
                let mut vertex = self.vertices[2].borrow_mut();
                vertex.set_x(ox);
                vertex.set_y(oy + h);
            }
        }
    }
}
use std::{cell::RefCell, rc::Rc};

impl Default for Triangle {
    fn default() -> Self {
        Self::new(TriangleBase::default())
    }
}
