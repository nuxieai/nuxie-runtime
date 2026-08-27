// Mirrors src/animation/animation_state_instance.cpp and
// include/rive/animation/animation_state_instance.hpp.
#[derive(Debug, Clone)]
pub(crate) struct AnimationStateInstance {
    animation_instance: LinearAnimationInstance,
    keep_going: bool,
}

impl AnimationStateInstance {
    pub(crate) fn new(
        state: &crate::state_machine::RuntimeLayerState,
        _instance: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
    ) -> Option<Self> {
        let animation_instance = LinearAnimationInstance::new(
            state
                .animation()
                .unwrap_or_else(RuntimeLinearAnimationHandle::empty),
            Arc::clone(animation_definitions),
            Arc::clone(empty_animation_definition),
            state.speed,
        )?;
        Some(Self {
            animation_instance,
            keep_going: true,
        })
    }

    pub(crate) fn advance(
        &mut self,
        state: &crate::state_machine::RuntimeLayerState,
        artboard: &mut ArtboardInstance,
        seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        self.keep_going = artboard.advance_linear_animation_instance_with_events(
            &mut self.animation_instance,
            seconds * state.speed,
            reported_events,
        );
        self.keep_going
    }

    pub(crate) fn apply(&mut self, artboard: &mut ArtboardInstance, mix: f32) -> bool {
        self.animation_instance.apply(artboard, mix)
    }

    pub(crate) fn keep_going(&self) -> bool {
        self.keep_going
    }

    pub(crate) fn clear_spilled_time(&mut self) {
        self.animation_instance.clear_spilled_time();
    }

    pub(crate) fn animation_instance(&self) -> &LinearAnimationInstance {
        &self.animation_instance
    }

    pub(crate) fn animation_instance_mut(&mut self) -> &mut LinearAnimationInstance {
        &mut self.animation_instance
    }

    pub(crate) fn for_each_animation_instance_mut(
        &mut self,
        mut callback: impl FnMut(&mut LinearAnimationInstance),
    ) {
        callback(&mut self.animation_instance);
    }
}
