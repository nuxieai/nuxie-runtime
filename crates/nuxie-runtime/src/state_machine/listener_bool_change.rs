//! Direct Rust owner for pinned C++
//! `src/animation/listener_bool_change.cpp`.
//!
//! The concrete validators retain C++'s null-is-valid compatibility rule;
//! perform gives a nonempty nested id precedence and applies the authored
//! false, true, or toggle operation to the resolved boolean input.

use super::StateMachineInputInstance;
use super::listener_input_change::RuntimeListenerInputTarget;
use crate::ArtboardInstance;
use nuxie_binary::RuntimeObject;
use nuxie_graph::ArtboardGraph;
use nuxie_schema::definition_by_name;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerBoolChange {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeListenerBoolChange {
    /// Mechanical translation of
    /// `ListenerBoolChange::validateInputType`. C++ accepts a null slot for
    /// forward compatibility and otherwise requires `StateMachineBool`.
    fn validate_input_type(input_type: Option<&str>) -> bool {
        input_type.is_none_or(|input_type| {
            definition_by_name(input_type)
                .is_some_and(|definition| definition.is_a("StateMachineBool"))
        })
    }

    /// Mechanical translation of
    /// `ListenerBoolChange::validateNestedInputType`. The base importer only
    /// calls this for a resolved `NestedInput`, while the virtual itself keeps
    /// the pinned null-is-valid contract.
    fn validate_nested_input_type(input_type: Option<&str>) -> bool {
        input_type.is_none_or(|input_type| {
            definition_by_name(input_type).is_some_and(|definition| definition.is_a("NestedBool"))
        })
    }

    pub(crate) fn validates_for_import(
        target: RuntimeListenerInputTarget,
        graph: &ArtboardGraph,
        inputs: &[Option<&RuntimeObject>],
    ) -> bool {
        target.validates_for_import_with(
            graph,
            inputs,
            Self::validate_input_type,
            Self::validate_nested_input_type,
        )
    }

    #[cfg(test)]
    pub(crate) fn for_test(flags: u64, target: RuntimeListenerInputTarget, value: u64) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerBoolChange");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        target.write_to_owner(&action_owner);
        action_owner.set_uint(super::listener_action_owner::LISTENER_BOOL_VALUE_KEY, value);
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
            .uint(super::listener_action_owner::LISTENER_BOOL_VALUE_KEY);
        if let Some(local_id) = target.nested_input_local_id {
            return artboard.apply_listener_nested_bool_change(local_id, value);
        }
        target
            .direct_input_index
            .and_then(|index| inputs.get_mut(index))
            .is_some_and(|input| input.apply_listener_bool_change(value))
    }

    pub(crate) fn targets_direct_input(&self, artboard: &ArtboardInstance) -> bool {
        self.live_target(artboard).nested_input_local_id.is_none()
    }

    fn live_target(&self, _artboard: &ArtboardInstance) -> RuntimeListenerInputTarget {
        RuntimeListenerInputTarget::resolve_live(&self.action_owner)
    }
}
