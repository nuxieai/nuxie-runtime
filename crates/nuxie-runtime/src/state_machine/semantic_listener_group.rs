use super::listener_types::RuntimeListenerType;
use super::{RuntimeStateMachineListener, ScriptListenerInvocation};

/// One occurrence of pinned C++ `SemanticListenerGroup`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeSemanticListenerGroup {
    pub(crate) listener_index: usize,
    pub(crate) target_local_id: usize,
    pub(crate) semantic_data_local_id: usize,
}

impl RuntimeSemanticListenerGroup {
    pub(crate) fn new(
        listener_index: usize,
        semantic_data_local_id: usize,
        listener: &RuntimeStateMachineListener,
    ) -> Option<Self> {
        listener
            .has_listener(RuntimeListenerType::SemanticAction)
            .then_some(Self {
                listener_index,
                target_local_id: listener.target_local_id,
                semantic_data_local_id,
            })
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
