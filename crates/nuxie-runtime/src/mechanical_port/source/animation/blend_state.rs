use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::blend_animation::BlendAnimation,
    generated::animation::blend_state_base::BlendStateBase, importers::import_stack::ImportStack,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct BlendState {
    pub base: BlendStateBase,
    animations: Vec<NonNull<BlendAnimation>>,
}

impl BlendState {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.base.base.import(import_stack)
    }

    pub fn animations(&self) -> &[NonNull<BlendAnimation>] {
        &self.animations
    }

    pub(crate) fn add_animation(&mut self, animation: NonNull<BlendAnimation>) {
        assert!(!self.animations.contains(&animation));
        self.animations.push(animation);
    }

    #[cfg(feature = "testing")]
    pub fn animation_count(&self) -> usize {
        self.animations.len()
    }

    #[cfg(feature = "testing")]
    pub fn animation(&self, index: usize) -> NonNull<BlendAnimation> {
        self.animations[index]
    }
}

impl Drop for BlendState {
    fn drop(&mut self) {
        for animation in self.animations.drain(..) {
            unsafe { drop(Box::from_raw(animation.as_ptr())) };
        }
    }
}
