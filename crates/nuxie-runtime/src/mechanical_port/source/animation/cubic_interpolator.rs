use crate::mechanical_port::source::generated::animation::cubic_interpolator_base::CubicInterpolatorBase;
use crate::mechanical_port::source::{
    animation::cubic_interpolator_solver::CubicInterpolatorSolver, core_context::CoreContext,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct CubicInterpolator {
    pub base: CubicInterpolatorBase,
    pub(super) solver: CubicInterpolatorSolver,
}

impl CubicInterpolator {
    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        self.solver.build(self.base.x1(), self.base.x2());
        StatusCode::Ok
    }

    pub fn initialize(&mut self) {
        self.solver.build(self.base.x1(), self.base.x2());
    }
}
impl std::ops::Deref for CubicInterpolator {
    type Target = CubicInterpolatorBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for CubicInterpolator {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::cubic_interpolator_base::CubicInterpolatorBaseCallbacks for CubicInterpolator { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
