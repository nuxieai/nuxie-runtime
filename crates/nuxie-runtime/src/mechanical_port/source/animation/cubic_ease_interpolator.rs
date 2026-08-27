use crate::mechanical_port::source::animation::cubic_interpolator_solver::CubicInterpolatorSolver;
use crate::mechanical_port::source::generated::animation::cubic_ease_interpolator_base::CubicEaseInterpolatorBase;

#[derive(Default)]
pub struct CubicEaseInterpolator {
    pub base: CubicEaseInterpolatorBase,
    solver: CubicInterpolatorSolver,
}

impl CubicEaseInterpolator {
    pub fn transform_value(&self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        value_from + (value_to - value_from) * self.transform(factor)
    }

    pub fn transform(&self, factor: f32) -> f32 {
        CubicInterpolatorSolver::calc_bezier(
            self.solver.get_t(factor),
            self.base.y1(),
            self.base.y2(),
        )
    }
}
