//! Direct Rust owner for pinned C++
//! `src/animation/listener_number_change.cpp`.
//!
//! The concrete validators retain C++'s null-is-valid compatibility rule;
//! perform gives a nonempty nested id precedence and otherwise mutates the
//! direct number input with the live authored value.

use super::StateMachineInputInstance;
use super::listener_input_change::RuntimeListenerInputTarget;
use crate::ArtboardInstance;
use nuxie_binary::RuntimeObject;
use nuxie_graph::ArtboardGraph;
use nuxie_schema::definition_by_name;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerNumberChange {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeListenerNumberChange {
    /// Mechanical translation of
    /// `ListenerNumberChange::validateInputType`. C++ accepts a null slot for
    /// forward compatibility and otherwise requires `StateMachineNumber`.
    fn validate_input_type(input_type: Option<&str>) -> bool {
        input_type.is_none_or(|input_type| {
            definition_by_name(input_type)
                .is_some_and(|definition| definition.is_a("StateMachineNumber"))
        })
    }

    /// Mechanical translation of
    /// `ListenerNumberChange::validateNestedInputType`. The base importer only
    /// calls this for a resolved `NestedInput`, while the virtual itself keeps
    /// the pinned null-is-valid contract.
    fn validate_nested_input_type(input_type: Option<&str>) -> bool {
        input_type.is_none_or(|input_type| {
            definition_by_name(input_type).is_some_and(|definition| definition.is_a("NestedNumber"))
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
