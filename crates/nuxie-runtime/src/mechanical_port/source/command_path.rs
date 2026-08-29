use crate::mechanical_port::source::{
    math::{mat2d::Mat2D, path_types::FillRule, vec2d::Vec2D},
    renderer::RenderPath,
};

pub trait CommandPath {
    fn rewind(&mut self);
    fn set_fill_rule(&mut self, value: FillRule);
    fn add_path(&mut self, path: &mut dyn CommandPath, transform: &Mat2D);
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32);
    fn close(&mut self);
    fn render_path_mut(&mut self) -> &mut RenderPath;
    fn render_path(&self) -> &RenderPath;

    fn add_rect(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.move_to(x, y);
        self.line_to(x + width, y);
        self.line_to(x + width, y + height);
        self.line_to(x, y + height);
        self.close();
    }

    fn move_(&mut self, value: Vec2D) {
        self.move_to(value.x, value.y);
    }

    fn line(&mut self, value: Vec2D) {
        self.line_to(value.x, value.y);
    }

    fn cubic(&mut self, a: Vec2D, b: Vec2D, c: Vec2D) {
        self.cubic_to(a.x, a.y, b.x, b.y, c.x, c.y);
    }
}
