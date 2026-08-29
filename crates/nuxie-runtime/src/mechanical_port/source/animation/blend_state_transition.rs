use crate::mechanical_port::source::{
    animation::{
        blend_animation_1d::BlendAnimation1D, blend_animation_direct::BlendAnimationDirect,
        blend_state_instance::BlendAnimationDefinition, state_instance::RuntimeStateInstanceHandle,
    },
    core::CoreHandle,
    generated::animation::blend_state_transition_base::BlendStateTransitionBase,
};
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ExitAnimationTiming {
    pub last_total_time: f32,
    pub total_time: f32,
    pub duration_seconds: f32,
    pub loop_value: i32,
}
#[derive(Default)]
pub struct BlendStateTransition {
    pub base: BlendStateTransitionBase,
    exit_blend_animation: Option<CoreHandle>,
}
impl BlendStateTransition {
    pub fn exit_blend_animation(&self) -> Option<CoreHandle> {
        self.exit_blend_animation.clone()
    }
    pub(crate) fn set_exit_blend_animation(&mut self, value: Option<CoreHandle>) {
        self.exit_blend_animation = value;
    }
    pub fn exit_time_animation_instance(
        &self,
        from: Option<&RuntimeStateInstanceHandle>,
    ) -> Option<ExitAnimationTiming> {
        let from = from?;
        let animation = self.exit_blend_animation.clone()?;
        match from.definition().with(|state| state.core_type())? {
            crate::mechanical_port::source::generated::animation::blend_state_1d_input_base::BlendState1DInputBase::TYPE_KEY
            | crate::mechanical_port::source::generated::animation::blend_state_1d_viewmodel_base::BlendState1DViewModelBase::TYPE_KEY
            | crate::mechanical_port::source::generated::animation::blend_state_direct_base::BlendStateDirectBase::TYPE_KEY => from.animation_for_blend(&animation, |instance| ExitAnimationTiming {
                last_total_time: instance.last_total_time(), total_time: instance.total_time(),
                duration_seconds: instance.duration_seconds(), loop_value: instance.loop_value(),
            }),
            _ => None,
        }
    }
    pub fn exit_time_animation(
        &self,
        _from: Option<&RuntimeStateInstanceHandle>,
    ) -> Option<CoreHandle> {
        self.exit_blend_animation
            .as_ref()
            .and_then(|animation| {
                animation
                    .with_downcast::<BlendAnimation1D, _>(BlendAnimationDefinition::animation)
                    .or_else(|| {
                        animation.with_downcast::<BlendAnimationDirect, _>(
                            BlendAnimationDefinition::animation,
                        )
                    })
            })
            .flatten()
    }
}

impl std::ops::Deref for BlendStateTransition {
    type Target = BlendStateTransitionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BlendStateTransition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
