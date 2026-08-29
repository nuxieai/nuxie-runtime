use crate::mechanical_port::source::{
    command_path::CommandPath,
    math::{aabb::IAabb, hit_test::HitTester, mat2d::Mat2D, path_types::FillRule, vec2d::Vec2D},
    renderer::RenderPath,
};

pub struct HitTestCommandPath {
    tester: HitTester,
    xform: Mat2D,
    area: IAabb,
    fill_rule: FillRule,
}

impl HitTestCommandPath {
    pub fn new(area: IAabb) -> Self {
        Self {
            tester: HitTester::from_area(area),
            xform: Mat2D::default(),
            area,
            fill_rule: FillRule::NonZero,
        }
    }

    pub fn set_xform(&mut self, xform: Mat2D) {
        self.xform = xform;
    }

    pub fn was_hit(&mut self) -> bool {
        self.tester.test(self.fill_rule)
    }
}

impl CommandPath for HitTestCommandPath {
    fn rewind(&mut self) {
        self.tester.reset_area(self.area);
    }

    fn set_fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_path(&mut self, _path: &mut dyn CommandPath, _transform: &Mat2D) {
        panic!("HitTestCommandPath does not support add_path");
    }

    fn render_path_mut(&mut self) -> &mut RenderPath {
        panic!("HitTestCommandPath does not expose a RenderPath");
    }

    fn render_path(&self) -> &RenderPath {
        panic!("HitTestCommandPath does not expose a RenderPath");
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.tester.move_to(self.xform * Vec2D::new(x, y));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.tester.line_to(self.xform * Vec2D::new(x, y));
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.tester.cubic_to(
            self.xform * Vec2D::new(ox, oy),
            self.xform * Vec2D::new(ix, iy),
            self.xform * Vec2D::new(x, y),
        );
    }

    fn close(&mut self) {
        self.tester.close();
    }
}
