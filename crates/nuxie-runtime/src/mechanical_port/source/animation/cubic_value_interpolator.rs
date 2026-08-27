use crate::mechanical_port::source::{
    animation::cubic_interpolator_solver::CubicInterpolatorSolver,
    generated::animation::cubic_value_interpolator_base::CubicValueInterpolatorBase,
    status_code::StatusCode,
};
pub struct CubicValueInterpolator {
    pub base: CubicValueInterpolatorBase,
    a: f32,
    b: f32,
    c: f32,
    d: f32,
    value_to: f32,
    solver: CubicInterpolatorSolver,
}
impl Default for CubicValueInterpolator {
    fn default() -> Self {
        let mut value = Self {
            base: CubicValueInterpolatorBase::default(),
            a: 0.0,
            b: 0.0,
            c: 0.0,
            d: 0.0,
            value_to: 0.0,
            solver: CubicInterpolatorSolver::default(),
        };
        value.compute_parameters();
        value
    }
}
impl CubicValueInterpolator {
    pub fn new() -> Self {
        Self::default()
    }
    fn compute_parameters(&mut self) {
        let y1 = self.d;
        let y2 = self.base.base.base.y1();
        let y3 = self.base.base.base.y2();
        let y4 = self.value_to;
        self.a = y4 + 3.0 * (y2 - y3) - y1;
        self.b = 3.0 * (y3 - y2 * 2.0 + y1);
        self.c = 3.0 * (y2 - y1);
    }
    pub fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32 {
        if self.d != from || self.value_to != to {
            self.d = from;
            self.value_to = to;
            self.compute_parameters();
        }
        let t = self.solver.get_t(factor);
        ((self.a * t + self.b) * t + self.c) * t + self.d
    }
    pub fn transform(&self, factor: f32) -> f32 {
        debug_assert!(false);
        factor
    }
    pub fn on_added_dirty(&mut self) -> StatusCode {
        self.compute_parameters();
        self.solver
            .build(self.base.base.base.x1(), self.base.base.base.x2());
        StatusCode::Ok
    }
}
