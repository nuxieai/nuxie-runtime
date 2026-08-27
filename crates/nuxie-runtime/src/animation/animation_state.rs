// Mirrors src/animation/animation_state.cpp and
// include/rive/animation/animation_state.hpp.
impl crate::state_machine::RuntimeLayerState {
    pub(crate) fn animation(&self) -> Option<RuntimeLinearAnimationHandle> {
        self.animation
    }

    pub(crate) fn make_animation_instance(
        &self,
        _instance: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
    ) -> Option<(LinearAnimationInstance, bool)> {
        let mut animation = LinearAnimationInstance::new(
            self.animation()?,
            Arc::clone(animation_definitions),
            Arc::clone(empty_animation_definition),
            self.speed,
        )?;
        let keep_going = animation.advance(0.0);
        Some((animation, keep_going))
    }

    #[cfg(test)]
    pub(crate) fn set_animation_for_testing(
        &mut self,
        animation: Option<RuntimeLinearAnimationHandle>,
    ) {
        self.animation = animation;
    }
}
