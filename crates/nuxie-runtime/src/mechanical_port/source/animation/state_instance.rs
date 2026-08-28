use crate::mechanical_port::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        state_machine_instance::StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
};
use std::{cell::RefCell, hash::Hash, rc::Rc};

/// Shared identity for one runtime-only `StateInstance` occurrence.
///
/// Authored `LayerState` definitions remain in the file's `CoreArena`; this
/// handle owns only the instance produced by `LayerState::makeInstance`, just
/// as the pinned state-machine layer owns the corresponding C++ pointer.
#[derive(Clone)]
pub struct RuntimeStateInstanceHandle {
    definition: CoreHandle,
    behavior: Rc<RefCell<Box<dyn StateInstanceBehavior>>>,
}

impl RuntimeStateInstanceHandle {
    pub fn new(definition: CoreHandle, behavior: Box<dyn StateInstanceBehavior>) -> Self {
        Self {
            definition,
            behavior: Rc::new(RefCell::new(behavior)),
        }
    }

    pub fn definition(&self) -> CoreHandle {
        self.definition.clone()
    }

    pub fn with_state<R>(&self, use_state: impl FnOnce(&dyn StateInstanceBehavior) -> R) -> R {
        use_state(self.behavior.borrow().as_ref())
    }

    pub fn with_state_mut<R>(
        &self,
        use_state: impl FnOnce(&mut dyn StateInstanceBehavior) -> R,
    ) -> R {
        use_state(self.behavior.borrow_mut().as_mut())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.behavior, &other.behavior)
    }

    fn identity(&self) -> *const () {
        Rc::as_ptr(&self.behavior).cast::<()>()
    }

    pub fn first_animation<R>(
        &self,
        use_animation: impl FnOnce(&mut LinearAnimationInstance) -> R,
    ) -> Option<R> {
        let mut use_animation = Some(use_animation);
        let mut result = None;
        self.with_state_mut(|state| {
            state.for_each_animation_instance(&mut |animation| {
                if let Some(use_animation) = use_animation.take() {
                    result = Some(use_animation(animation));
                }
            });
        });
        result
    }
}

impl PartialEq for RuntimeStateInstanceHandle {
    fn eq(&self, other: &Self) -> bool {
        self.ptr_eq(other)
    }
}

impl Eq for RuntimeStateInstanceHandle {}

impl Hash for RuntimeStateInstanceHandle {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.identity().hash(state);
    }
}

pub struct StateInstance {
    layer_state: CoreHandle,
}

impl StateInstance {
    pub fn new(layer_state: CoreHandle) -> Self {
        Self { layer_state }
    }

    pub fn clear_spilled_time(&mut self) {}

    pub fn for_each_animation_instance(
        &mut self,
        _callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
    }

    pub fn state(&self) -> CoreHandle {
        self.layer_state.clone()
    }
}

impl Drop for StateInstance {
    fn drop(&mut self) {}
}

pub trait StateInstanceBehavior {
    fn advance(&mut self, seconds: f32, state_machine_instance: &mut StateMachineInstance);
    fn apply(&mut self, artboard_instance: &RuntimeArtboardInstanceWeakHandle, mix: f32);
    fn keep_going(&self) -> bool;
    fn clear_spilled_time(&mut self) {}
    fn for_each_animation_instance(
        &mut self,
        _callback: &mut dyn FnMut(&mut LinearAnimationInstance),
    ) {
    }
}
