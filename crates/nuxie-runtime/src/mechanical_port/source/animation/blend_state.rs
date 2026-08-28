use crate::mechanical_port::source::{
    core::CoreHandle, generated::animation::blend_state_base::BlendStateBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
};

#[derive(Default)]
pub struct BlendState {
    pub base: BlendStateBase,
    animations: Vec<CoreHandle>,
}

impl BlendState {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.base.base.import(import_stack)
    }

    pub fn animations(&self) -> &[CoreHandle] {
        &self.animations
    }

    pub(crate) fn add_animation(&mut self, animation: CoreHandle) {
        assert!(!self.animations.contains(&animation));
        self.animations.push(animation);
    }

    #[cfg(test)]
    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }

    #[cfg(test)]
    pub fn animation(&self, index: usize) -> CoreHandle {
        self.animations[index].clone()
    }
}
impl std::ops::Deref for BlendState {
    type Target = BlendStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for BlendState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
