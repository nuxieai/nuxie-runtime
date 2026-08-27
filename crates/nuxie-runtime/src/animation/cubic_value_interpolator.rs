// Mirrors src/animation/cubic_value_interpolator.cpp and
// include/rive/animation/cubic_value_interpolator.hpp.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct RuntimeCubicValueInterpolator {
    cubic: RuntimeCubicInterpolator,
    y1: f32,
    y2: f32,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    value_to: f32,
}

impl RuntimeCubicValueInterpolator {
    fn new() -> Self {
        let mut interpolator = Self {
            cubic: RuntimeCubicInterpolator {
                x1: 0.42,
                x2: 0.58,
                solver: RuntimeCubicInterpolatorSolver::build(0.0, 0.0),
            },
            y1: 0.0,
            y2: 1.0,
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            value_to: 0.0,
        };
        interpolator.compute_parameters();
        interpolator
    }

    fn on_added_dirty(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        let mut interpolator = Self::new();
        interpolator.cubic.x1 = x1;
        interpolator.y1 = y1;
        interpolator.cubic.x2 = x2;
        interpolator.y2 = y2;
        interpolator.compute_parameters();
        interpolator.cubic.initialize();
        interpolator
    }

    fn compute_parameters(&mut self) {
        let y1 = self.d;
        let y2 = self.y1;
        let y3 = self.y2;
        let y4 = self.value_to;

        self.a = y4 + 3.0 * (y2 - y3) - y1;
        self.b = 3.0 * (y3 - y2 * 2.0 + y1);
        self.c = 3.0 * (y2 - y1);
    }

    fn transform_value(&mut self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        if self.d != value_from || self.value_to != value_to {
            self.d = value_from;
            self.value_to = value_to;
            self.compute_parameters();
        }
        let t = self.cubic.get_t(factor);
        ((self.a * t + self.b) * t + self.c) * t + self.d
    }

    fn transform(&self, factor: f32) -> f32 {
        debug_assert!(false, "CubicValueInterpolator::transform is invalid");
        factor
    }
}
