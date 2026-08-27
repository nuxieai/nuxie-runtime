// Mirrors src/animation/blend_state.cpp and
// include/rive/animation/blend_state.hpp.
pub(crate) trait RuntimeBlendState {
    type Animation;

    fn retained_animations(&self) -> &Vec<Self::Animation>;
    fn retained_animations_mut(&mut self) -> &mut Vec<Self::Animation>;

    fn add_animation(&mut self, animation: Self::Animation) {
        self.retained_animations_mut().push(animation);
    }

    fn animations(&self) -> &[Self::Animation] {
        self.retained_animations()
    }

    #[cfg(test)]
    fn animation_count(&self) -> usize {
        self.retained_animations().len()
    }

    #[cfg(test)]
    fn animation(&self, index: usize) -> Option<&Self::Animation> {
        self.retained_animations().get(index)
    }
}

impl RuntimeBlendState for crate::state_machine::RuntimeBlendState1D {
    type Animation = RuntimeBlendAnimation1D;

    fn retained_animations(&self) -> &Vec<Self::Animation> {
        &self.animations
    }

    fn retained_animations_mut(&mut self) -> &mut Vec<Self::Animation> {
        &mut self.animations
    }
}

impl Drop for crate::state_machine::RuntimeBlendState1D {
    fn drop(&mut self) {
        RuntimeBlendState1DViewModel::drop_source(&mut self.source);
        self.animations.clear();
    }
}

impl RuntimeBlendState for crate::state_machine::RuntimeBlendStateDirect {
    type Animation = RuntimeBlendAnimationDirect;

    fn retained_animations(&self) -> &Vec<Self::Animation> {
        &self.animations
    }

    fn retained_animations_mut(&mut self) -> &mut Vec<Self::Animation> {
        &mut self.animations
    }
}

impl Drop for crate::state_machine::RuntimeBlendStateDirect {
    fn drop(&mut self) {
        self.animations.clear();
    }
}
