use crate::mechanical_port::source::generated::animation::cubic_interpolator_base::CubicInterpolatorBase;
use crate::mechanical_port::source::{
    animation::cubic_interpolator_solver::CubicInterpolatorSolver, core_context::CoreContext,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct CubicInterpolator {
    pub base: CubicInterpolatorBase,
    solver: CubicInterpolatorSolver,
}

impl CubicInterpolator {
    pub fn on_added_dirty(&mut self, _context: &mut CoreContext) -> StatusCode {
        self.solver.build(self.base.x1(), self.base.x2());
        StatusCode::Ok
    }

    pub fn initialize(&mut self) {
        self.solver.build(self.base.x1(), self.base.x2());
    }
}
