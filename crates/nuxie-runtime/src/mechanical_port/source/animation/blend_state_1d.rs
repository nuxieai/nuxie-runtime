use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationResetTarget,
        animation_reset_factory::{ResetArtboard, ResetLinearAnimation},
        blend_animation_1d::BlendAnimation1D,
        blend_state_1d_instance::{
            BlendState1DDefinition, BlendState1DInstance, BlendState1DValueSource,
        },
        blend_state_instance::BlendStateDefinition,
    },
    generated::animation::blend_state_1d_base::BlendState1DBase,
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct BlendState1D {
    pub base: BlendState1DBase,
}

impl BlendState1D {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.base.base.import(import_stack)
    }

    pub fn make_instance<R>(
        &self,
        instance: &mut R,
    ) -> BlendState1DInstance<'_, Self, BlendAnimation1D>
    where
        R: ResetArtboard + AnimationResetTarget,
        <BlendAnimation1D as crate::mechanical_port::source::animation::blend_state_instance::BlendAnimationDefinition>::Animation: ResetLinearAnimation,
    {
        BlendState1DInstance::new(self, instance)
    }
}

impl BlendStateDefinition<BlendAnimation1D> for BlendState1D {
    fn animations(&self) -> Vec<&BlendAnimation1D> {
        self.base
            .base
            .animations()
            .iter()
            .map(|animation| unsafe { &*animation.as_ptr().cast::<BlendAnimation1D>() })
            .collect()
    }

    fn flags(&self) -> u8 {
        self.base.base.base.base.base.flags() as u8
    }
}

impl BlendState1DDefinition<BlendAnimation1D> for BlendState1D {
    fn value_source(&self) -> BlendState1DValueSource {
        BlendState1DValueSource::Default
    }
}
