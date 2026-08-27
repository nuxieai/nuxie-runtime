use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::rectangle_base::RectangleBase,
    shapes::straight_vertex::StraightVertex,
};
pub struct Rectangle {
    pub base: RectangleBase,
    vertices: [Box<StraightVertex>; 4],
}
impl Rectangle {
    pub fn new(mut base: RectangleBase) -> Self {
        let mut vertices = std::array::from_fn(|_| Box::new(StraightVertex::default()));
        for vertex in &mut vertices {
            base.add_vertex(&mut **vertex);
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
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            let radius = self.base.corner_radius_tl();
            let link = self.base.link_corner_radius();
            let ox = -self.base.origin_x() * self.base.width();
            let oy = -self.base.origin_y() * self.base.height();
            let w = self.base.width();
            let h = self.base.height();
            let v = &mut self.vertices;
            v[0].base.set_x(ox);
            v[0].base.set_y(oy);
            v[0].base.set_radius(radius);
            v[1].base.set_x(ox + w);
            v[1].base.set_y(oy);
            v[1].base.set_radius(if link {
                radius
            } else {
                self.base.corner_radius_tr()
            });
            v[2].base.set_x(ox + w);
            v[2].base.set_y(oy + h);
            v[2].base.set_radius(if link {
                radius
            } else {
                self.base.corner_radius_br()
            });
            v[3].base.set_x(ox);
            v[3].base.set_y(oy + h);
            v[3].base.set_radius(if link {
                radius
            } else {
                self.base.corner_radius_bl()
            });
        }
        self.base.update(value);
    }
}
