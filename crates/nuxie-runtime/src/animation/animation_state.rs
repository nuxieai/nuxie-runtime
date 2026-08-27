// Mirrors src/animation/animation_state.cpp and
// include/rive/animation/animation_state.hpp.
impl crate::state_machine::RuntimeLayerState {
    pub(crate) fn animation(&self) -> Option<RuntimeLinearAnimationHandle> {
        self.animation
    }

    pub(crate) fn make_animation_instance(
        &self,
        instance: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
    ) -> Option<AnimationStateInstance> {
        AnimationStateInstance::new(
            self,
            instance,
            animation_definitions,
            empty_animation_definition,
        )
    }

    #[cfg(test)]
    pub(crate) fn set_animation_for_testing(
        &mut self,
        animation: Option<RuntimeLinearAnimationHandle>,
    ) {
        self.animation = animation;
    }
}
