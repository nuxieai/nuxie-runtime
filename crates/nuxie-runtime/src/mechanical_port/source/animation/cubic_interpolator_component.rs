use crate::mechanical_port::source::generated::animation::cubic_interpolator_component_base::CubicInterpolatorComponentBase;
use crate::mechanical_port::source::{
    animation::cubic_interpolator_solver::CubicInterpolatorSolver, core_context::CoreContext,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct CubicInterpolatorComponent {
    pub base: CubicInterpolatorComponentBase,
    solver: CubicInterpolatorSolver,
}

impl CubicInterpolatorComponent {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.solver.build(self.base.x1(), self.base.x2());
        StatusCode::Ok
    }

    pub fn transform(&self, factor: f32) -> f32 {
        CubicInterpolatorSolver::calc_bezier(
            self.solver.get_t(factor),
            self.base.y1(),
            self.base.y2(),
        )
    }
}
