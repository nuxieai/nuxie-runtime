// Pinned C++ correspondence (d788e8ec):
// src/animation/semantic_listener_group.cpp:1-54.

use std::cell::RefCell;
use std::rc::Rc;

use super::listener_types::RuntimeListenerType;
use super::{RuntimeStateMachineListener, ScriptListenerInvocation};
use crate::{RuntimeSemanticData, SemanticActionType, SemanticListener};

#[derive(Debug)]
struct RuntimeSemanticListenerCallback {
    queued_actions: Rc<RefCell<Vec<u32>>>,
}

impl SemanticListener for RuntimeSemanticListenerCallback {
    fn on_semantic_tap(&self) {
        self.queued_actions
            .borrow_mut()
            .push(SemanticActionType::Tap as u32);
    }

    fn on_semantic_increase(&self) {
        self.queued_actions
            .borrow_mut()
            .push(SemanticActionType::Increase as u32);
    }

    fn on_semantic_decrease(&self) {
        self.queued_actions
            .borrow_mut()
            .push(SemanticActionType::Decrease as u32);
    }
}

/// One occurrence of pinned C++ `SemanticListenerGroup`.
#[derive(Debug)]
pub(crate) struct RuntimeSemanticListenerGroup {
    pub(crate) listener_index: usize,
    pub(crate) target_local_id: usize,
    pub(crate) semantic_data_local_id: usize,
    queued_actions: Rc<RefCell<Vec<u32>>>,
    semantic_listener: Rc<dyn SemanticListener>,
}

fn semantic_listener_callback() -> (Rc<RefCell<Vec<u32>>>, Rc<dyn SemanticListener>) {
    let queued_actions = Rc::new(RefCell::new(Vec::new()));
    let semantic_listener: Rc<dyn SemanticListener> = Rc::new(RuntimeSemanticListenerCallback {
        queued_actions: queued_actions.clone(),
    });
    (queued_actions, semantic_listener)
}

impl Clone for RuntimeSemanticListenerGroup {
    fn clone(&self) -> Self {
        let (queued_actions, semantic_listener) = semantic_listener_callback();
        Self {
            listener_index: self.listener_index,
            target_local_id: self.target_local_id,
            semantic_data_local_id: self.semantic_data_local_id,
            queued_actions,
            semantic_listener,
        }
    }
}

impl RuntimeSemanticListenerGroup {
    pub(crate) fn new(
        listener_index: usize,
        semantic_data_local_id: usize,
        listener: &RuntimeStateMachineListener,
    ) -> Option<Self> {
        listener
            .has_listener(RuntimeListenerType::SemanticAction)
            .then(|| {
                let (queued_actions, semantic_listener) = semantic_listener_callback();
                Self {
                    listener_index,
                    target_local_id: listener.target_local_id,
                    semantic_data_local_id,
                    queued_actions,
                    semantic_listener,
                }
            })
    }

    /// Register the retained callback with its SemanticData owner. The shared
    /// StateMachineInstance lifecycle splice calls this after resolving the
    /// authored data occurrence.
    pub(crate) fn register(&self, semantic_data: &mut RuntimeSemanticData) {
        semantic_data.add_semantic_listener(self.semantic_listener.clone());
    }

    /// Mirror the pinned destructor's first-matching-pointer removal.
    pub(crate) fn unregister(&self, semantic_data: &mut RuntimeSemanticData) {
        semantic_data.remove_semantic_listener(&self.semantic_listener);
    }

    /// Convert registered callbacks into the same constrained invocation that
    /// the instance queues for deferred listener processing.
    pub(crate) fn drain_registered_invocations(
        &self,
        listener: &RuntimeStateMachineListener,
    ) -> Vec<ScriptListenerInvocation> {
        std::mem::take(&mut *self.queued_actions.borrow_mut())
            .into_iter()
            .filter_map(|action_type| self.invocation(listener, action_type))
            .collect()
    }

    pub(crate) fn invocation(
        &self,
        listener: &RuntimeStateMachineListener,
        action_type: u32,
    ) -> Option<ScriptListenerInvocation> {
        listener.semantic_constraints_met(action_type).then_some(
            ScriptListenerInvocation::Semantic {
                listener_index: self.listener_index,
                action_type,
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semantic_listener() -> RuntimeStateMachineListener {
        RuntimeStateMachineListener {
            name: None,
            target_local_id: 9,
            is_single: false,
            listener_types: vec![RuntimeListenerType::SemanticAction],
            event_local_indices: Vec::new(),
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: Vec::new(),
        }
    }

    #[test]
    fn registration_queues_callbacks_and_unregisters_the_same_retained_listener() {
        let listener = semantic_listener();
        let group = RuntimeSemanticListenerGroup::new(4, 7, &listener).expect("semantic group");
        let cloned_group = group.clone();
        let mut data = RuntimeSemanticData::new(7, Some(9));
        group.register(&mut data);
        data.fire(SemanticActionType::Tap);
        data.fire(SemanticActionType::Increase);
        assert_eq!(
            group.queued_actions.borrow().as_slice(),
            [
                SemanticActionType::Tap as u32,
                SemanticActionType::Increase as u32
            ]
        );
        assert!(cloned_group.queued_actions.borrow().is_empty());
        group.queued_actions.borrow_mut().clear();

        group.unregister(&mut data);
        data.fire(SemanticActionType::Decrease);
        assert!(group.queued_actions.borrow().is_empty());
    }
}
