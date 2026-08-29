use crate::mechanical_port::source::{
    animation::animation_state_instance::AnimationStateInstance,
    artboard::RuntimeArtboardInstanceWeakHandle, core::CoreHandle,
    generated::animation::animation_state_base::AnimationStateBase,
};

pub struct AnimationState {
    pub base: AnimationStateBase,
    animation: Option<CoreHandle>,
}

impl Default for AnimationState {
    fn default() -> Self {
        Self {
            base: AnimationStateBase::default(),
            animation: None,
        }
    }
}

impl AnimationState {
    pub fn animation(&self) -> Option<CoreHandle> {
        self.animation.clone()
    }

    pub(crate) fn set_animation(&mut self, animation: Option<CoreHandle>) {
        self.animation = animation;
    }

    #[cfg(test)]
    pub fn animation_for_testing(&mut self, animation: CoreHandle) {
        self.animation = Some(animation);
    }

    pub fn speed(&self) -> f32 {
        self.base.base.base.speed()
    }

    pub fn make_instance(
        &self,
        instance: RuntimeArtboardInstanceWeakHandle,
    ) -> AnimationStateInstance {
        let state = self
            .base
            .base
            .base
            .base
            .handle()
            .expect("an imported AnimationState has arena identity before instancing");
        AnimationStateInstance::new(state, instance)
    }
}

impl std::ops::Deref for AnimationState {
    type Target = AnimationStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for AnimationState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
