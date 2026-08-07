use nuxie_binary::RuntimeObject;
use nuxie_graph::ArtboardGraph;
use nuxie_schema::definition_by_name;

pub(crate) const EMPTY_INPUT_ID: u64 = u32::MAX as u64;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeListenerInputTarget {
    pub(crate) direct_input_index: Option<usize>,
    pub(crate) nested_input_local_id: Option<usize>,
}

impl RuntimeListenerInputTarget {
    pub(crate) fn from_object(object: &nuxie_binary::RuntimeObject) -> Self {
        let nested_input_local_id = object
            .uint_property("nestedInputId")
            .filter(|value| *value != EMPTY_INPUT_ID)
            .and_then(|value| usize::try_from(value).ok());
        // C++ retains both authored ids. Import first attempts to resolve the
        // nested id and falls back to validating the direct slot when that id
        // does not resolve to a NestedInput; perform still gives any nonempty
        // nested id precedence (`listener_input_change.cpp:24-47`).
        let direct_input_index = object
            .uint_property("inputId")
            .and_then(|value| usize::try_from(value).ok());
        Self {
            direct_input_index,
            nested_input_local_id,
        }
    }

    /// Mirrors pinned C++ `ListenerInputChange::import`: if the authored
    /// nested id resolves to any `NestedInput`, validate that concrete nested
    /// type. Otherwise validate the direct state-machine input slot. Missing
    /// slots remain forward-compatible null definitions.
    pub(crate) fn validates_for_import(
        self,
        graph: &ArtboardGraph,
        inputs: &[Option<&RuntimeObject>],
        direct_type: &str,
        nested_type: &str,
    ) -> bool {
        if let Some(local_id) = self.nested_input_local_id
            && let Some(component) = graph
                .components
                .iter()
                .find(|component| component.local_id == local_id)
            && definition_by_name(component.type_name)
                .is_some_and(|definition| definition.is_a("NestedInput"))
        {
            return definition_by_name(component.type_name)
                .is_some_and(|definition| definition.is_a(nested_type));
        }

        self.direct_input_index
            .and_then(|index| inputs.get(index))
            .copied()
            .flatten()
            .is_none_or(|input| {
                definition_by_name(input.type_name)
                    .is_some_and(|definition| definition.is_a(direct_type))
            })
    }

    pub(crate) fn resolve_live(action_owner: &super::RuntimeActionCoreHandle) -> Self {
        let input_id = action_owner.uint(super::listener_action_owner::LISTENER_INPUT_ID_KEY);
        let nested_input_id =
            action_owner.uint(super::listener_action_owner::LISTENER_NESTED_INPUT_ID_KEY);
        Self {
            direct_input_index: usize::try_from(input_id).ok(),
            nested_input_local_id: (nested_input_id != EMPTY_INPUT_ID)
                .then(|| usize::try_from(nested_input_id).ok())
                .flatten(),
        }
    }

    #[cfg(test)]
    pub(crate) fn write_to_owner(self, action_owner: &super::RuntimeActionCoreHandle) {
        action_owner.set_uint(
            super::listener_action_owner::LISTENER_INPUT_ID_KEY,
            self.direct_input_index
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(EMPTY_INPUT_ID),
        );
        action_owner.set_uint(
            super::listener_action_owner::LISTENER_NESTED_INPUT_ID_KEY,
            self.nested_input_local_id
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(EMPTY_INPUT_ID),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
    use nuxie_graph::GraphFile;

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .expect("schema definition")
                .type_key
                .int,
            properties,
        }
    }

    fn uint(type_name: &str, property_name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, property_name)
                .expect("schema property"),
            value: FixtureValue::Uint(value),
        }
    }

    #[test]
    fn import_validation_uses_nested_type_then_forward_compatible_direct_slot() {
        let file = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "NestedArtboard",
                vec![
                    uint("NestedArtboard", "parentId", 0),
                    uint("NestedArtboard", "artboardId", 1),
                ],
            ),
            record(
                "NestedStateMachine",
                vec![uint("NestedStateMachine", "parentId", 1)],
            ),
            record("NestedBool", vec![uint("NestedBool", "parentId", 2)]),
            record("StateMachine", Vec::new()),
            record("StateMachineBool", Vec::new()),
            record("StateMachineNumber", Vec::new()),
            record("Artboard", Vec::new()),
        ])
        .expect("validation fixture imports");
        let graphs = GraphFile::from_runtime_file(&file).expect("validation graph builds");
        let state_machine = file
            .artboard_state_machine_graphs(0)
            .into_iter()
            .next()
            .expect("state machine");
        let graph = &graphs.artboards[0];

        let nested = RuntimeListenerInputTarget {
            direct_input_index: Some(1),
            nested_input_local_id: Some(3),
        };
        assert!(nested.validates_for_import(
            graph,
            &state_machine.inputs,
            "StateMachineBool",
            "NestedBool",
        ));
        assert!(!nested.validates_for_import(
            graph,
            &state_machine.inputs,
            "StateMachineNumber",
            "NestedNumber",
        ));

        let direct = RuntimeListenerInputTarget {
            direct_input_index: Some(0),
            nested_input_local_id: None,
        };
        assert!(direct.validates_for_import(
            graph,
            &state_machine.inputs,
            "StateMachineBool",
            "NestedBool",
        ));
        assert!(!direct.validates_for_import(
            graph,
            &state_machine.inputs,
            "StateMachineNumber",
            "NestedNumber",
        ));

        let unresolved_nested = RuntimeListenerInputTarget {
            direct_input_index: Some(1),
            nested_input_local_id: Some(99),
        };
        assert!(
            unresolved_nested.validates_for_import(
                graph,
                &state_machine.inputs,
                "StateMachineNumber",
                "NestedNumber",
            ),
            "an unresolved nested id falls back to the authored direct slot for import validation"
        );
        assert!(
            !unresolved_nested.validates_for_import(
                graph,
                &state_machine.inputs,
                "StateMachineBool",
                "NestedBool",
            ),
            "the same unresolved nested id must not hide a known wrong direct type"
        );
        assert_eq!(
            unresolved_nested.nested_input_local_id,
            Some(99),
            "perform still gives the nonempty nested id precedence and becomes a no-op when it cannot resolve"
        );

        let missing = RuntimeListenerInputTarget {
            direct_input_index: Some(99),
            nested_input_local_id: None,
        };
        assert!(missing.validates_for_import(
            graph,
            &state_machine.inputs,
            "StateMachineTrigger",
            "NestedTrigger",
        ));
    }
}
