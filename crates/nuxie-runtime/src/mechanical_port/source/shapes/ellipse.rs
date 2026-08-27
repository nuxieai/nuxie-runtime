use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::ellipse_base::EllipseBase,
    math::{circle_constant::CIRCLE_CONSTANT, vec2d::Vec2D},
    shapes::cubic_detached_vertex::CubicDetachedVertex,
};
pub struct Ellipse {
    pub base: EllipseBase,
    vertices: [Box<CubicDetachedVertex>; 4],
}
impl Ellipse {
    pub fn new(mut base: EllipseBase) -> Self {
        let mut vertices = std::array::from_fn(|_| Box::new(CubicDetachedVertex::default()));
        for vertex in &mut vertices {
            base.add_vertex(&mut **vertex);
        }
        Self { base, vertices }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let rx = self.base.width() / 2.0;
            let ry = self.base.height() / 2.0;
            let ox = -self.base.origin_x() * self.base.width() + rx;
            let oy = -self.base.origin_y() * self.base.height() + ry;
            let v = &mut self.vertices;
            v[0].base.set_x(ox);
            v[0].base.set_y(oy - ry);
            v[0].base
                .set_in_point(Vec2D::new(ox - rx * CIRCLE_CONSTANT, oy - ry));
            v[0].base
                .set_out_point(Vec2D::new(ox + rx * CIRCLE_CONSTANT, oy - ry));
            v[1].base.set_x(ox + rx);
            v[1].base.set_y(oy);
            v[1].base
                .set_in_point(Vec2D::new(ox + rx, oy + CIRCLE_CONSTANT * -ry));
            v[1].base
                .set_out_point(Vec2D::new(ox + rx, oy + CIRCLE_CONSTANT * ry));
            v[2].base.set_x(ox);
            v[2].base.set_y(oy + ry);
            v[2].base
                .set_in_point(Vec2D::new(ox + rx * CIRCLE_CONSTANT, oy + ry));
            v[2].base
                .set_out_point(Vec2D::new(ox - rx * CIRCLE_CONSTANT, oy + ry));
            v[3].base.set_x(ox - rx);
            v[3].base.set_y(oy);
            v[3].base
                .set_in_point(Vec2D::new(ox - rx, oy + ry * CIRCLE_CONSTANT));
            v[3].base
                .set_out_point(Vec2D::new(ox - rx, oy - ry * CIRCLE_CONSTANT));
        }
        self.base.update(value);
    }
}
