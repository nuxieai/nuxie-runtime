fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct RuntimeCubicInterpolator {
    x1: f32,
    x2: f32,
    solver: RuntimeCubicInterpolatorSolver,
}

impl RuntimeCubicInterpolator {
    fn on_added_dirty(x1: f32, x2: f32) -> Self {
        let mut interpolator = Self {
            x1,
            x2,
            solver: RuntimeCubicInterpolatorSolver::build(0.0, 0.0),
        };
        interpolator.initialize();
        interpolator
    }

    fn initialize(&mut self) {
        self.solver = RuntimeCubicInterpolatorSolver::build(self.x1, self.x2);
    }

    fn get_t(self, x: f32) -> f32 {
        self.solver.get_t(x)
    }
}
