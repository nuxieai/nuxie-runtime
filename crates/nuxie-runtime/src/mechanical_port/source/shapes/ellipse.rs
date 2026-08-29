use crate::mechanical_port::source::shapes::cubic_vertex::CubicVertexBehavior;
use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::ellipse_base::EllipseBase,
    math::{circle_constant::CIRCLE_CONSTANT, vec2d::Vec2D},
    shapes::cubic_detached_vertex::CubicDetachedVertex,
};
impl std::ops::Deref for Ellipse {
    type Target = EllipseBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Ellipse {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Ellipse {
    pub const TYPE_KEY: u16 = EllipseBase::TYPE_KEY;
}

pub struct Ellipse {
    pub base: EllipseBase,
    vertices: [Rc<RefCell<CubicDetachedVertex>>; 4],
}
impl Ellipse {
    pub fn new(mut base: EllipseBase) -> Self {
        let vertices =
            std::array::from_fn(|_| Rc::new(RefCell::new(CubicDetachedVertex::default())));
        for vertex in &vertices {
            base.add_runtime_cubic_vertex(vertex.clone());
        }
        Self { base, vertices }
    }
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let rx = self.base.width() / 2.0;
            let ry = self.base.height() / 2.0;
            let ox = -self.base.origin_x() * self.base.width() + rx;
            let oy = -self.base.origin_y() * self.base.height() + ry;
            {
                let mut vertex = self.vertices[0].borrow_mut();
                vertex.set_x(ox);
                vertex.set_y(oy - ry);
                vertex.set_in_point(Vec2D::new(ox - rx * CIRCLE_CONSTANT, oy - ry));
                vertex.set_out_point(Vec2D::new(ox + rx * CIRCLE_CONSTANT, oy - ry));
            }
            {
                let mut vertex = self.vertices[1].borrow_mut();
                vertex.set_x(ox + rx);
                vertex.set_y(oy);
                vertex.set_in_point(Vec2D::new(ox + rx, oy + CIRCLE_CONSTANT * -ry));
                vertex.set_out_point(Vec2D::new(ox + rx, oy + CIRCLE_CONSTANT * ry));
            }
            {
                let mut vertex = self.vertices[2].borrow_mut();
                vertex.set_x(ox);
                vertex.set_y(oy + ry);
                vertex.set_in_point(Vec2D::new(ox + rx * CIRCLE_CONSTANT, oy + ry));
                vertex.set_out_point(Vec2D::new(ox - rx * CIRCLE_CONSTANT, oy + ry));
            }
            {
                let mut vertex = self.vertices[3].borrow_mut();
                vertex.set_x(ox - rx);
                vertex.set_y(oy);
                vertex.set_in_point(Vec2D::new(ox - rx, oy + ry * CIRCLE_CONSTANT));
                vertex.set_out_point(Vec2D::new(ox - rx, oy - ry * CIRCLE_CONSTANT));
            }
        }
    }
}
use std::{cell::RefCell, rc::Rc};

impl Default for Ellipse {
    fn default() -> Self {
        Self::new(EllipseBase::default())
    }
}
