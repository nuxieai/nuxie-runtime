use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationResetTarget,
        animation_reset_factory::ResetArtboard,
        blend_animation_1d::BlendAnimation1D,
        blend_state_1d_instance::{
            BlendState1DDefinition, BlendState1DInstance, BlendState1DValueSource,
        },
        blend_state_instance::BlendStateDefinition,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
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
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> BlendState1DInstance<Self, BlendAnimation1D>
    where
        R: ResetArtboard + AnimationResetTarget,
    {
        let state = self
            .base
            .base
            .base
            .base
            .base
            .handle()
            .expect("an imported BlendState1D has arena identity before instancing");
        BlendState1DInstance::new(state, instance, artboard)
    }
}

impl BlendStateDefinition<BlendAnimation1D> for BlendState1D {
    fn animations(&self) -> Vec<CoreHandle> {
        self.base.base.animations().to_vec()
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
impl std::ops::Deref for BlendState1D {
    type Target = BlendState1DBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BlendState1D {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
