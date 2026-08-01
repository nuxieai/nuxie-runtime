use nuxie_render_api::FillRule;

pub use crate::math::hit_test::HitTestArea;
use crate::math::hit_test::HitTester;
use crate::{Mat2D, RuntimePathCommand};

/// Direct command-path owner for the pinned C++ `HitTestCommandPath`.
///
/// Geometry is transformed as commands arrive and accumulated with the same
/// integer-cell delta-winding raster used by C++ `HitTester`.
#[derive(Debug)]
pub struct HitTestCommandPath {
    tester: HitTester,
    transform: Mat2D,
    area: HitTestArea,
    fill_rule: FillRule,
}

impl HitTestCommandPath {
    pub fn new(area: HitTestArea) -> Self {
        Self {
            tester: HitTester::new(area),
            transform: Mat2D::IDENTITY,
            area,
            fill_rule: FillRule::NonZero,
        }
    }

    pub fn set_transform(&mut self, transform: Mat2D) {
        self.transform = transform;
    }

    pub fn was_hit(&mut self) -> bool {
        self.tester.test(self.fill_rule)
    }

    pub fn rewind(&mut self) {
        self.tester.reset(self.area);
    }

    pub fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.tester.move_to(self.transform.transform_point(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.tester.line_to(self.transform.transform_point(x, y));
    }

    pub fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.tester.cubic_to(
            self.transform.transform_point(ox, oy),
            self.transform.transform_point(ix, iy),
            self.transform.transform_point(x, y),
        );
    }

    pub fn close(&mut self) {
        self.tester.close();
    }

    pub(crate) fn add_runtime_commands(&mut self, commands: &[RuntimePathCommand]) {
        for command in commands {
            match *command {
                RuntimePathCommand::Move { x, y } => self.move_to(x, y),
                RuntimePathCommand::Line { x, y } => self.line_to(x, y),
                RuntimePathCommand::Cubic {
                    x1,
                    y1,
                    x2,
                    y2,
                    x3,
                    y3,
                } => self.cubic_to(x1, y1, x2, y2, x3, y3),
                RuntimePathCommand::Close => self.close(),
            }
        }
    }
}
