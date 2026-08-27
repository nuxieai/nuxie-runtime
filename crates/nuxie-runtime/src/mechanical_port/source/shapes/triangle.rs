use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::triangle_base::TriangleBase,
    shapes::straight_vertex::StraightVertex,
};
pub struct Triangle {
    pub base: TriangleBase,
    vertices: [Box<StraightVertex>; 3],
}
impl Triangle {
    pub fn new(mut base: TriangleBase) -> Self {
        let mut vertices = std::array::from_fn(|_| Box::new(StraightVertex::default()));
        for vertex in &mut vertices {
            base.add_vertex(&mut **vertex);
        }
        Self { base, vertices }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let ox = -self.base.origin_x() * self.base.width();
            let oy = -self.base.origin_y() * self.base.height();
            let w = self.base.width();
            let h = self.base.height();
            self.vertices[0].base.set_x(ox + w / 2.0);
            self.vertices[0].base.set_y(oy);
            self.vertices[1].base.set_x(ox + w);
            self.vertices[1].base.set_y(oy + h);
            self.vertices[2].base.set_x(ox);
            self.vertices[2].base.set_y(oy + h);
        }
        self.base.update(value);
    }
}
