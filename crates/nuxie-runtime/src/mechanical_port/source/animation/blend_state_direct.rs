use crate::mechanical_port::source::{
    animation::{
        blend_animation_direct::BlendAnimationDirect,
        blend_state_direct_instance::BlendStateDirectInstance,
        blend_state_instance::BlendStateDefinition,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
    generated::animation::blend_state_direct_base::BlendStateDirectBase,
};

#[derive(Default)]
pub struct BlendStateDirect {
    pub base: BlendStateDirectBase,
}

impl BlendStateDirect {
    pub fn make_instance(
        &self,
        instance: RuntimeArtboardInstanceWeakHandle,
    ) -> BlendStateDirectInstance<Self, BlendAnimationDirect> {
        let state = self
            .base
            .base
            .base
            .base
            .base
            .handle()
            .expect("an imported BlendStateDirect has arena identity before instancing");
        BlendStateDirectInstance::new(state, instance)
    }
}

impl BlendStateDefinition<BlendAnimationDirect> for BlendStateDirect {
    fn animations(&self) -> Vec<CoreHandle> {
        self.base.base.animations().to_vec()
    }

    fn flags(&self) -> u8 {
        self.base.base.base.base.base.flags() as u8
    }
}
