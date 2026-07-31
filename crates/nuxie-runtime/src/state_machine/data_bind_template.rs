//! Authored-order state-machine DataBind occurrence templates.
//!
//! Pinned C++ clones every DataBind and converter occurrence before resolving
//! its source or selecting a concrete context-value subtype
//! (`backboard_importer.cpp:125-145`; `data_bind.cpp:251-299`). Target
//! families own target storage only; they must never filter, reorder, or
//! deduplicate the DataBind occurrence list.

use crate::RuntimeViewModelPointer;
use crate::data_bind_graph::{
    RuntimeDataBindGraphConverter, RuntimeDataBindGraphConverterBuildCache,
    RuntimeDataBindGraphTarget, RuntimeDataBindGraphValue,
    runtime_data_bind_graph_converter_with_cache,
};
use crate::data_converter::{
    RuntimeDataConverterDataBindDefinition, runtime_data_converter_data_bind_definition,
};
use crate::properties::property_key_for_name;
use crate::state_machine::bindables::runtime_unresolved_view_model_value_at_path;
use crate::state_machine::transition_duration_binding::{
    RuntimeTransitionDurationBinding, state_machine_transition_target_for_data_bind,
};
use nuxie_binary::{RuntimeDataValue, RuntimeFile, RuntimeObject};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineDataBindTemplate {
    pub(crate) data_bind_index: usize,
    pub(crate) authored_path: Vec<u32>,
    pub(crate) resolved_path: Vec<u32>,
    pub(crate) name_based: bool,
    pub(crate) context_bindable: bool,
    pub(crate) flags: u64,
    pub(crate) converter: Option<RuntimeDataBindGraphConverter>,
    pub(crate) converter_data_binds: RuntimeDataConverterDataBindDefinition,
    pub(crate) target: RuntimeDataBindGraphTarget,
    pub(crate) source_seed: RuntimeDataBindGraphValue,
    pub(crate) source_bound: bool,
    pub(crate) view_model_instance_ids: Vec<u32>,
}

pub(super) fn runtime_state_machine_data_bind_templates<'a>(
    file: &'a RuntimeFile,
    state_machine: &nuxie_binary::RuntimeStateMachine<'a>,
    default_instance: Option<&RuntimeObject>,
    transition_duration_bindings: &[RuntimeTransitionDurationBinding],
    converter_cache: &mut RuntimeDataBindGraphConverterBuildCache<'a>,
) -> Vec<RuntimeStateMachineDataBindTemplate> {
    state_machine
        .data_binds
        .iter()
        .enumerate()
        .map(|(data_bind_index, data_bind)| {
            // A base DataBind has no context path but still owns its cloned
            // converter occurrence and teardown identity in C++.
            let authored_source_path = file.data_bind_context_source_path_ids_for_object(data_bind);
            let context_bindable = authored_source_path.is_some();
            let authored_path = authored_source_path.unwrap_or_default();
            let resolved_path = file
                .data_bind_context_resolved_source_path_ids_for_object(data_bind)
                .unwrap_or_else(|| authored_path.clone());
            let name_based = file
                .data_bind_is_name_based_for_object(data_bind)
                .unwrap_or(false);
            let target =
                state_machine_transition_target_for_data_bind(file, state_machine, data_bind)
                    .and_then(|target_object| {
                        runtime_state_machine_data_bind_target(
                            data_bind_index,
                            data_bind,
                            target_object,
                            transition_duration_bindings,
                        )
                    })
                    .unwrap_or(RuntimeDataBindGraphTarget::Inert);
            let (source_seed, source_bound, view_model_instance_ids) =
                runtime_state_machine_data_bind_source(file, default_instance, &resolved_path);

            let converter_object = file.resolved_data_converter_for_data_bind_object(data_bind);
            let converter =
                runtime_data_bind_graph_converter_with_cache(file, data_bind, converter_cache);
            let converter_data_binds = converter_object
                .zip(converter.as_ref())
                .map(|(converter_object, converter)| {
                    runtime_data_converter_data_bind_definition(file, converter_object, converter)
                })
                .unwrap_or_default();

            RuntimeStateMachineDataBindTemplate {
                data_bind_index,
                authored_path,
                resolved_path,
                name_based,
                context_bindable,
                flags: data_bind.uint_property("flags").unwrap_or(0),
                converter,
                converter_data_binds,
                target,
                source_seed,
                source_bound,
                view_model_instance_ids,
            }
        })
        .collect()
}

fn runtime_state_machine_data_bind_target(
    data_bind_index: usize,
    data_bind: &RuntimeObject,
    target: &RuntimeObject,
    transition_duration_bindings: &[RuntimeTransitionDurationBinding],
) -> Option<RuntimeDataBindGraphTarget> {
    let property_key = data_bind
        .uint_property("propertyKey")
        .and_then(|value| u16::try_from(value).ok());
    let property_matches = |type_name: &str, property_name: &str| {
        property_key_for_name(type_name, property_name) == property_key
    };
    let trigger_property_matches = || {
        property_key.is_none_or(|property_key| property_key == 0)
            || property_matches("BindablePropertyTrigger", "propertyValue")
    };

    Some(match target.type_name {
        "BindablePropertyNumber" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Number {
                global_id: target.id,
            }
        }
        "BindablePropertyInteger" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Integer {
                global_id: target.id,
            }
        }
        "BindablePropertyBoolean" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Boolean {
                global_id: target.id,
            }
        }
        "BindablePropertyString" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::String {
                global_id: target.id,
            }
        }
        "BindablePropertyColor" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Color {
                global_id: target.id,
            }
        }
        "BindablePropertyEnum" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Enum {
                global_id: target.id,
            }
        }
        "BindablePropertyAsset" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Asset {
                global_id: target.id,
            }
        }
        "BindablePropertyArtboard" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::Artboard {
                global_id: target.id,
            }
        }
        "BindablePropertyList" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::List {
                global_id: target.id,
            }
        }
        "BindablePropertyTrigger" if trigger_property_matches() => {
            RuntimeDataBindGraphTarget::Trigger {
                global_id: target.id,
            }
        }
        "BindablePropertyViewModel" if property_matches(target.type_name, "propertyValue") => {
            RuntimeDataBindGraphTarget::ViewModel {
                global_id: target.id,
            }
        }
        "StateTransition" if property_matches(target.type_name, "duration") => {
            let occurrence_index = transition_duration_bindings
                .iter()
                .position(|binding| binding.data_bind_index == data_bind_index)?;
            RuntimeDataBindGraphTarget::TransitionDuration {
                transition_global_id: target.id,
                occurrence_index,
            }
        }
        _ => return None,
    })
}

fn runtime_state_machine_data_bind_source(
    file: &RuntimeFile,
    default_instance: Option<&RuntimeObject>,
    path: &[u32],
) -> (RuntimeDataBindGraphValue, bool, Vec<u32>) {
    let resolved = default_instance
        .and_then(|instance| file.data_context_view_model_property_for_instance(instance, path))
        .and_then(|source| file.view_model_instance_source_data_value_for_object(source));

    match resolved {
        Some(RuntimeDataValue::Number(value)) => {
            (RuntimeDataBindGraphValue::Number(value), true, Vec::new())
        }
        Some(RuntimeDataValue::String(value)) => (
            RuntimeDataBindGraphValue::String(value.to_vec()),
            true,
            Vec::new(),
        ),
        Some(RuntimeDataValue::Boolean(value)) => {
            (RuntimeDataBindGraphValue::Boolean(value), true, Vec::new())
        }
        Some(RuntimeDataValue::Color(value)) => {
            (RuntimeDataBindGraphValue::Color(value), true, Vec::new())
        }
        Some(RuntimeDataValue::Enum { value, .. }) => {
            (RuntimeDataBindGraphValue::Enum(value), true, Vec::new())
        }
        Some(RuntimeDataValue::Trigger(value)) => {
            (RuntimeDataBindGraphValue::Trigger(value), true, Vec::new())
        }
        Some(RuntimeDataValue::List(items)) => (
            RuntimeDataBindGraphValue::List {
                item_count: items.len(),
            },
            true,
            Vec::new(),
        ),
        Some(RuntimeDataValue::SymbolListIndex(value)) => (
            RuntimeDataBindGraphValue::SymbolListIndex(value),
            true,
            Vec::new(),
        ),
        Some(RuntimeDataValue::AssetImage(value) | RuntimeDataValue::AssetFont(value)) => {
            (RuntimeDataBindGraphValue::Asset(value), true, Vec::new())
        }
        Some(RuntimeDataValue::Artboard(value)) => {
            (RuntimeDataBindGraphValue::Artboard(value), true, Vec::new())
        }
        Some(RuntimeDataValue::ViewModel(reference)) => {
            let pointer = reference
                .as_ref()
                .map(|reference| RuntimeViewModelPointer::Imported {
                    object_id: reference.object.id,
                })
                .unwrap_or(RuntimeViewModelPointer::Null);
            let instance_ids = reference
                .and_then(|reference| file.view_model(reference.view_model_index))
                .map(|view_model| {
                    view_model
                        .instances
                        .into_iter()
                        .map(|instance| instance.object.id)
                        .collect()
                })
                .unwrap_or_default();
            (
                RuntimeDataBindGraphValue::ViewModel(pointer),
                true,
                instance_ids,
            )
        }
        Some(RuntimeDataValue::None) | None => (
            runtime_unresolved_view_model_value_at_path(file, path)
                .unwrap_or(RuntimeDataBindGraphValue::Untyped),
            false,
            Vec::new(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn templates_retain_base_and_unsupported_targets_in_authored_order() {
        let number_property_key = property_key_for_name("BindablePropertyNumber", "propertyValue")
            .expect("number property key");
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record("BindablePropertyNumber", Vec::new()),
            // A base DataBind has no context path and its default invalid
            // property key has no concrete target adapter.
            record("DataBind", Vec::new()),
            record("BindablePropertyNumber", Vec::new()),
            // A context occurrence with a known target family but a wrong
            // property key is likewise retained and inert.
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(number_property_key + 1)),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![0, 0]),
                    ),
                ],
            ),
            record("BindablePropertyNumber", Vec::new()),
            record(
                "DataBindContext",
                vec![
                    property(
                        "DataBindContext",
                        "propertyKey",
                        AuthoringValue::Uint(u64::from(number_property_key)),
                    ),
                    property(
                        "DataBindContext",
                        "sourcePathIds",
                        AuthoringValue::Bytes(vec![0, 0]),
                    ),
                ],
            ),
        ])
        .expect("state-machine DataBind fixture imports");
        let state_machine = file
            .artboard_state_machine_graphs(0)
            .into_iter()
            .next()
            .expect("fixture state machine");
        assert_eq!(state_machine.data_binds.len(), 3);

        let mut converter_cache = RuntimeDataBindGraphConverterBuildCache::default();
        let templates = runtime_state_machine_data_bind_templates(
            &file,
            &state_machine,
            None,
            &[],
            &mut converter_cache,
        );

        // Pinned C++ appends every accepted DataBind clone in insertion order
        // before binding (`state_machine_instance.cpp:1754-1824`;
        // `data_bind.cpp:251-299`). A missing context path or unsupported
        // property adapter cannot compact that occurrence list.
        assert_eq!(
            templates
                .iter()
                .map(|template| template.data_bind_index)
                .collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!(
            templates
                .iter()
                .map(|template| template.context_bindable)
                .collect::<Vec<_>>(),
            [false, true, true]
        );
        assert!(matches!(
            templates[0].target,
            RuntimeDataBindGraphTarget::Inert
        ));
        assert!(matches!(
            templates[1].target,
            RuntimeDataBindGraphTarget::Inert
        ));
        assert!(matches!(
            templates[2].target,
            RuntimeDataBindGraphTarget::Number { .. }
        ));
    }
}
