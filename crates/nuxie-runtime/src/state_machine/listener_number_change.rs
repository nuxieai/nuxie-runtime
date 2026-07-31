use super::StateMachineInputInstance;
use super::listener_input_change::RuntimeListenerInputTarget;
use crate::ArtboardInstance;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerNumberChange {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeListenerNumberChange {
    #[cfg(test)]
    pub(crate) fn for_test(flags: u64, target: RuntimeListenerInputTarget, value: f32) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerNumberChange");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        target.write_to_owner(&action_owner);
        action_owner.set_double_imported_for_test(
            super::listener_action_owner::LISTENER_NUMBER_VALUE_KEY,
            value,
        );
        Self { action_owner }
    }

    pub(crate) fn perform(
        &self,
        artboard: &mut ArtboardInstance,
        inputs: &mut [StateMachineInputInstance],
    ) -> bool {
        let target = self.live_target(artboard);
        let value = self
            .action_owner
            .double(super::listener_action_owner::LISTENER_NUMBER_VALUE_KEY);
        if let Some(local_id) = target.nested_input_local_id {
            return artboard.set_nested_number_value(local_id, value);
        }
        target
            .direct_input_index
            .and_then(|index| inputs.get_mut(index))
            .is_some_and(|input| input.set_number(value))
    }

    pub(crate) fn targets_direct_input(&self, artboard: &ArtboardInstance) -> bool {
        self.live_target(artboard).nested_input_local_id.is_none()
    }

    fn live_target(&self, _artboard: &ArtboardInstance) -> RuntimeListenerInputTarget {
        RuntimeListenerInputTarget::resolve_live(&self.action_owner)
    }
}
