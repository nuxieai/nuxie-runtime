#[derive(Debug, Clone, Copy)]
/// Direct owner for pinned C++ `src/animation/cubic_interpolator_component.cpp`.
pub(crate) struct RuntimeCubicInterpolatorComponent {
    solver: RuntimeCubicInterpolatorSolver,
}

impl RuntimeCubicInterpolatorComponent {
    /// C++ `onAddedDirty` builds the retained solver from the two x controls
    /// after the component superclass has accepted the object. Rust performs
    /// that superclass/graph admission before constructing this value.
    pub(crate) fn on_added_dirty(x1: f32, x2: f32) -> Self {
        Self {
            solver: RuntimeCubicInterpolatorSolver::build(x1, x2),
        }
    }

    pub(crate) fn transform(self, factor: f32, y1: f32, y2: f32) -> f32 {
        RuntimeCubicInterpolatorSolver::calc_bezier(self.solver.get_t(factor), y1, y2)
    }
}
