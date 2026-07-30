use super::StateMachineInputInstance;
use super::listener_input_change::RuntimeListenerInputTarget;
use crate::ArtboardInstance;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerTriggerChange {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeListenerTriggerChange {
    #[cfg(test)]
    pub(crate) fn for_test(flags: u64, target: RuntimeListenerInputTarget) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerTriggerChange");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        target.write_to_owner(&action_owner);
        Self { action_owner }
    }

    pub(crate) fn perform(
        &self,
        artboard: &mut ArtboardInstance,
        inputs: &mut [StateMachineInputInstance],
    ) -> bool {
        let target = self.live_target(artboard);
        if let Some(local_id) = target.nested_input_local_id {
            // Pinned C++ passes `CallbackData(stateMachineInstance, 0)` here,
            // but `NestedTrigger::fire` discards that value and calls
            // `applyValue()` unconditionally
            // (`listener_trigger_change.cpp:47`;
            // `nested_trigger.cpp:9-20`). Invoke the retained callback owner
            // directly: callback fields are not stored uint properties, and
            // every repeated fire must reach the nested input occurrence.
            return artboard.fire_nested_trigger_input(local_id);
        }
        target
            .direct_input_index
            .and_then(|index| inputs.get_mut(index))
            .is_some_and(StateMachineInputInstance::fire_trigger)
    }

    pub(crate) fn targets_direct_input(&self, artboard: &ArtboardInstance) -> bool {
        self.live_target(artboard).nested_input_local_id.is_none()
    }

    fn live_target(&self, _artboard: &ArtboardInstance) -> RuntimeListenerInputTarget {
        RuntimeListenerInputTarget::resolve_live(&self.action_owner)
    }
}
