use super::vec2d::Vec2D;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransformComponents {
    x: f32,
    y: f32,
    scale_x: f32,
    scale_y: f32,
    rotation: f32,
    skew: f32,
}
impl Default for TransformComponents {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            skew: 0.0,
        }
    }
}
impl TransformComponents {
    pub fn x(self) -> f32 {
        self.x
    }
    pub fn set_x(&mut self, value: f32) {
        self.x = value;
    }
    pub fn y(self) -> f32 {
        self.y
    }
    pub fn set_y(&mut self, value: f32) {
        self.y = value;
    }
    pub fn scale_x(self) -> f32 {
        self.scale_x
    }
    pub fn set_scale_x(&mut self, value: f32) {
        self.scale_x = value;
    }
    pub fn scale_y(self) -> f32 {
        self.scale_y
    }
    pub fn set_scale_y(&mut self, value: f32) {
        self.scale_y = value;
    }
    pub fn rotation(self) -> f32 {
        self.rotation
    }
    pub fn set_rotation(&mut self, value: f32) {
        self.rotation = value;
    }
    pub fn skew(self) -> f32 {
        self.skew
    }
    pub fn set_skew(&mut self, value: f32) {
        self.skew = value;
    }
    pub fn translation(self) -> Vec2D {
        Vec2D::new(self.x, self.y)
    }
    pub fn scale(self) -> Vec2D {
        Vec2D::new(self.scale_x, self.scale_y)
    }
}
