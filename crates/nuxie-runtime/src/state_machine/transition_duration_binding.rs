use super::{
    RuntimeBindableNumberDefaultViewModelSource,
    runtime_number_default_view_model_source_for_instance,
};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTransitionDurationBinding {
    pub(crate) transition_global_id: u32,
    pub(crate) source: RuntimeBindableNumberDefaultViewModelSource,
}

#[derive(Debug, Clone)]
pub(crate) struct StateMachineTransitionDurationInstance {
    pub(crate) transition_global_id: u32,
    value: f32,
}

impl StateMachineTransitionDurationInstance {
    pub(crate) fn new(binding: &RuntimeTransitionDurationBinding) -> Self {
        Self {
            transition_global_id: binding.transition_global_id,
            value: 0.0,
        }
    }

    pub(crate) fn set_value(&mut self, value: f32) {
        self.value = value;
    }

    pub(crate) fn value(&self) -> f32 {
        self.value
    }
}

pub(super) fn runtime_transition_duration_bindings(
    file: &RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'_>,
    default_instance: Option<&RuntimeObject>,
) -> Vec<RuntimeTransitionDurationBinding> {
    // C++ constructs every state-machine DataBind occurrence before a live
    // data context is attached, then DataBindContainer resolves its retained
    // DataBindContext path from StateMachineInstance::internalDataContext
    // (`state_machine_instance.cpp:1742-1766,2901-2905`;
    // `data_bind_container.cpp:25-33`). A missing authored default therefore
    // cannot erase this definition.
    let mut bindings = BTreeMap::<u32, RuntimeTransitionDurationBinding>::new();
    for (data_bind_index, data_bind) in state_machine.data_binds.iter().enumerate() {
        let Some(target) =
            state_machine_transition_target_for_data_bind(file, state_machine, data_bind)
        else {
            continue;
        };
        if target.type_name != "StateTransition" {
            continue;
        }
        let Some(source) = runtime_number_default_view_model_source_for_instance(
            file,
            data_bind_index,
            data_bind,
            "StateTransition",
            "duration",
            default_instance,
            0.0,
        ) else {
            continue;
        };
        bindings.insert(
            target.id,
            RuntimeTransitionDurationBinding {
                transition_global_id: target.id,
                source,
            },
        );
    }
    bindings.into_values().collect()
}

fn state_machine_transition_target_for_data_bind<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'a>,
    data_bind: &RuntimeObject,
) -> Option<&'a RuntimeObject> {
    if let Some(target) = file.data_bind_target_for_object(data_bind) {
        return Some(target);
    }

    state_machine
        .layers
        .iter()
        .flat_map(|layer| &layer.states)
        .flat_map(|state| &state.transitions)
        .map(|transition| transition.object)
        .filter(|transition| transition.id < data_bind.id)
        .max_by_key(|transition| transition.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::properties::property_key_for_name;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue};

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    #[test]
    fn unresolved_nested_duration_source_survives_until_live_context_binding() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(
                            property_key_for_name("StateTransition", "duration")
                                .expect("transition duration property key"),
                        )),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![1, 2, 0]),
                    ),
                    property("DataBindContext", "flags", AuthoringValue::Uint(4)),
                ],
            ),
        ])
        .expect("unresolved nested transition duration fixture imports");
        let data_bind = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "DataBindContext")
            .expect("fixture has a data bind");

        let source = runtime_number_default_view_model_source_for_instance(
            &file,
            0,
            data_bind,
            "StateTransition",
            "duration",
            None,
            0.0,
        )
        .expect("C++ retains the path before the live data context exists");

        assert_eq!(source.path, [1, 2, 0]);
        assert_eq!(source.flags, 4);
        assert!(matches!(
            source.value,
            crate::RuntimeDataBindGraphValue::Number(0.0)
        ));
    }
}
