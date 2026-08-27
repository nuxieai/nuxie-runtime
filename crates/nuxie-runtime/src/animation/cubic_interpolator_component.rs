#[derive(Debug, Clone, Copy)]
/// Direct owner for pinned C++ `src/animation/cubic_interpolator_component.cpp`.
pub(crate) struct RuntimeCubicInterpolatorComponent {
    x1: f32,
    x2: f32,
}

impl RuntimeCubicInterpolatorComponent {
    /// C++ `onAddedDirty` builds the retained solver from the two x controls
    /// after the component superclass has accepted the object. Rust performs
    /// that superclass/graph admission before constructing this value.
    pub(crate) fn on_added_dirty(x1: f32, x2: f32) -> Self {
        Self { x1, x2 }
    }

    pub(crate) fn transform(self, factor: f32, y1: f32, y2: f32) -> f32 {
        cubic_interpolator_calc_bezier(
            cubic_interpolator_get_t(factor, self.x1, self.x2),
            y1,
            y2,
        )
    }
}
