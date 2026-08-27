use crate::mechanical_port::source::{
    animation::{
        blend_animation_direct::BlendAnimationDirect,
        blend_state_direct_instance::BlendStateDirectInstance,
        blend_state_instance::BlendStateDefinition,
    },
    generated::animation::blend_state_direct_base::BlendStateDirectBase,
};

#[derive(Default)]
pub struct BlendStateDirect {
    pub base: BlendStateDirectBase,
}

impl BlendStateDirect {
    pub fn make_instance(
        &self,
        instance: *mut (),
    ) -> BlendStateDirectInstance<'_, Self, BlendAnimationDirect> {
        BlendStateDirectInstance::new(self, instance)
    }
}

impl BlendStateDefinition<BlendAnimationDirect> for BlendStateDirect {
    fn animations(&self) -> Vec<&BlendAnimationDirect> {
        self.base
            .base
            .animations()
            .iter()
            .map(|animation| unsafe { &*animation.as_ptr().cast::<BlendAnimationDirect>() })
            .collect()
    }

    fn flags(&self) -> u8 {
        self.base.base.base.base.base.flags() as u8
    }
}
