// Mirrors src/animation/blend_animation_1d.cpp and
// include/rive/animation/blend_animation_1d.hpp.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeBlendAnimation1D {
    pub(crate) animation: RuntimeLinearAnimationHandle,
    pub(crate) value: f32,
}

impl RuntimeBlendAnimation for RuntimeBlendAnimation1D {
    fn retained_animation(&self) -> RuntimeLinearAnimationHandle {
        self.animation
    }
}

impl Default for RuntimeBlendAnimation1D {
    fn default() -> Self {
        Self {
            animation: RuntimeLinearAnimationHandle::empty(),
            value: 0.0,
        }
    }
}

impl RuntimeBlendAnimation1D {
    #[allow(dead_code)]
    pub(crate) fn on_added_dirty(&self) -> bool {
        true
    }

    #[allow(dead_code)]
    pub(crate) fn on_added_clean(&self) -> bool {
        true
    }
}
