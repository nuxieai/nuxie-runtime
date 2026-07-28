use super::instance::RuntimeStateMachineListenerActionExecutor;
use super::{RuntimeScheduledListenerAction, RuntimeScheduledListenerActionTargetsMut};
use crate::ArtboardInstance;
use crate::state_machine::RuntimeBindableAssetValue;
use crate::view_model::RuntimeViewModelPointer;
use nuxie_binary::{RuntimeFile, RuntimeObject};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerViewModelChange {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
    /// Exact BindableProperty occurrence acquired by the C++ importer.
    pub(crate) bindable_global_id: Option<u32>,
    pub(crate) value: Option<RuntimeListenerViewModelChangeValue>,
}

impl RuntimeListenerViewModelChange {
    #[cfg(test)]
    pub(crate) fn for_test(
        flags: u64,
        bindable_global_id: Option<u32>,
        value: Option<RuntimeListenerViewModelChangeValue>,
    ) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerViewModelChange");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        Self {
            action_owner,
            bindable_global_id,
            value,
        }
    }

    /// Return the value held by this state-machine occurrence's cloned
    /// BindableProperty. Pinned C++ calls `bindablePropertyInstance` at
    /// perform time; reading the authored definition value here would miss
    /// animation or DataBind writes made after construction.
    pub(crate) fn occurrence_value(
        &self,
        targets: &RuntimeScheduledListenerActionTargetsMut<'_>,
        data_context_present: bool,
    ) -> Option<RuntimeListenerViewModelChangeValue> {
        let global_id = self.bindable_global_id?;
        match self.value.as_ref()? {
            RuntimeListenerViewModelChangeValue::Number(_) => targets
                .bindable_numbers
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Number(value.value)),
            RuntimeListenerViewModelChangeValue::Integer(_) => targets
                .bindable_integers
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Integer(value.value)),
            RuntimeListenerViewModelChangeValue::Color(_) => targets
                .bindable_colors
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Color(value.value)),
            RuntimeListenerViewModelChangeValue::String(_) => targets
                .bindable_strings
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::String(value.value.clone())),
            RuntimeListenerViewModelChangeValue::Enum(_) => targets
                .bindable_enums
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Enum(value.value)),
            RuntimeListenerViewModelChangeValue::Asset(_) => targets
                .bindable_assets
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Asset(value.value.clone())),
            RuntimeListenerViewModelChangeValue::Artboard(_) => targets
                .bindable_artboards
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Artboard(value.value)),
            RuntimeListenerViewModelChangeValue::Trigger(_) => targets
                .bindable_triggers
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Trigger(value.value)),
            RuntimeListenerViewModelChangeValue::Boolean(_) => targets
                .bindable_booleans
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| RuntimeListenerViewModelChangeValue::Boolean(value.value)),
            RuntimeListenerViewModelChangeValue::List(_) => targets
                .bindable_lists
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| {
                    RuntimeListenerViewModelChangeValue::List(
                        u64::try_from(value.property_value).unwrap_or(u64::MAX),
                    )
                }),
            RuntimeListenerViewModelChangeValue::ViewModel(_) => targets
                .bindable_view_models
                .iter()
                .find(|value| value.global_id == global_id)
                .map(|value| {
                    RuntimeListenerViewModelChangeValue::ViewModel(if data_context_present {
                        RuntimeViewModelPointer::DataContextRoot
                    } else {
                        value.value
                    })
                }),
        }
    }

    /// Execute the exact retained BindableProperty occurrence against the
    /// live state-machine DataContext.
    ///
    /// C++ resolves `bindablePropertyInstance()` at perform time, locates the
    /// paired DataBind, then pushes the changed value back to its source
    /// (`listener_viewmodel_change.cpp:42-80`). Keeping this method on the
    /// corresponding owner prevents the instance orchestrator from growing
    /// another target-specific action branch.
    pub(super) fn perform(
        &self,
        executor: &mut RuntimeStateMachineListenerActionExecutor<'_>,
        artboard: &mut ArtboardInstance,
        targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> bool {
        let data_context_present = executor.data_bind_graph.data_context_present();
        let Some(bindable_global_id) = self.bindable_global_id else {
            return false;
        };
        let Some(value) = self.occurrence_value(&targets, data_context_present) else {
            return false;
        };
        executor.perform_scheduled_view_model_change(artboard, bindable_global_id, &value, targets)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeListenerViewModelChangeValue {
    Number(f32),
    Integer(u64),
    Color(u32),
    String(Vec<u8>),
    Enum(u64),
    Asset(RuntimeBindableAssetValue),
    Artboard(u64),
    Trigger(u64),
    Boolean(bool),
    List(u64),
    ViewModel(RuntimeViewModelPointer),
}

pub(super) fn runtime_listener_view_model_change_action(
    file: &RuntimeFile,
    state_machine_data_binds: &[&RuntimeObject],
    action: &nuxie_binary::RuntimeListenerAction<'_>,
    action_owner: super::RuntimeActionCoreHandle,
) -> RuntimeScheduledListenerAction {
    // `ListenerViewModelChange::import` takes the exact current
    // BindableProperty occurrence. The binary projection exposes its stable
    // object id; each state-machine occurrence later resolves the paired binds
    // from that retained identity.
    let _ = (file, state_machine_data_binds);
    let (bindable_global_id, value) = action
        .bindable_property
        .map(|bindable_property| {
            (
                Some(bindable_property.id),
                runtime_listener_view_model_change_value(bindable_property),
            )
        })
        .unwrap_or((None, None));
    RuntimeScheduledListenerAction::ViewModelChange(RuntimeListenerViewModelChange {
        action_owner,
        bindable_global_id,
        value,
    })
}

fn runtime_listener_view_model_change_value(
    target: &RuntimeObject,
) -> Option<RuntimeListenerViewModelChangeValue> {
    match target.type_name {
        "BindablePropertyNumber" => Some(RuntimeListenerViewModelChangeValue::Number(
            target.double_property("propertyValue").unwrap_or(0.0),
        )),
        "BindablePropertyInteger" => Some(RuntimeListenerViewModelChangeValue::Integer(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyColor" => Some(RuntimeListenerViewModelChangeValue::Color(
            target.color_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyString" => Some(RuntimeListenerViewModelChangeValue::String(
            target
                .string_property_bytes("propertyValue")
                .unwrap_or_default()
                .to_vec(),
        )),
        "BindablePropertyEnum" => Some(RuntimeListenerViewModelChangeValue::Enum(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyAsset" => Some(RuntimeListenerViewModelChangeValue::Asset(
            RuntimeBindableAssetValue::from_asset_index(
                target
                    .uint_property("propertyValue")
                    .unwrap_or(u64::from(u32::MAX)),
            ),
        )),
        "BindablePropertyArtboard" => Some(RuntimeListenerViewModelChangeValue::Artboard(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyList" => Some(RuntimeListenerViewModelChangeValue::List(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyTrigger" => Some(RuntimeListenerViewModelChangeValue::Trigger(
            target.uint_property("propertyValue").unwrap_or(0),
        )),
        "BindablePropertyBoolean" => Some(RuntimeListenerViewModelChangeValue::Boolean(
            target.bool_property("propertyValue").unwrap_or(false),
        )),
        "BindablePropertyViewModel" => Some(RuntimeListenerViewModelChangeValue::ViewModel(
            RuntimeViewModelPointer::DataContextRoot,
        )),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_machine::bindables::RuntimeBindableViewModelSource;
    use crate::state_machine::{
        RuntimeBindableViewModel, StateMachineBindableArtboardInstance,
        StateMachineBindableAssetInstance, StateMachineBindableBooleanInstance,
        StateMachineBindableColorInstance, StateMachineBindableEnumInstance,
        StateMachineBindableIntegerInstance, StateMachineBindableListInstance,
        StateMachineBindableNumberInstance, StateMachineBindableStringInstance,
        StateMachineBindableTriggerInstance, StateMachineBindableViewModelInstance,
    };

    #[test]
    fn perform_reads_the_mutable_bindable_occurrence_not_the_authored_default() {
        let action = RuntimeListenerViewModelChange::for_test(
            0,
            Some(7),
            Some(RuntimeListenerViewModelChangeValue::Number(1.0)),
        );
        let mut inputs = Vec::new();
        let mut reported_events = Vec::new();
        let mut bindable_numbers = vec![StateMachineBindableNumberInstance {
            global_id: 7,
            data_bind_indices: Vec::new(),
            value: 9.0,
        }];
        let mut bindable_integers = Vec::new();
        let mut bindable_colors = Vec::new();
        let mut bindable_strings = Vec::new();
        let mut bindable_enums = Vec::new();
        let mut bindable_assets = Vec::new();
        let mut bindable_artboards = Vec::new();
        let mut bindable_lists = Vec::new();
        let mut bindable_triggers = Vec::new();
        let mut bindable_view_models = Vec::new();
        let mut bindable_booleans = Vec::new();
        let mut transition_durations = Vec::new();
        let targets = RuntimeScheduledListenerActionTargetsMut {
            inputs: &mut inputs,
            reported_events: &mut reported_events,
            bindable_numbers: &mut bindable_numbers,
            bindable_integers: &mut bindable_integers,
            bindable_colors: &mut bindable_colors,
            bindable_strings: &mut bindable_strings,
            bindable_enums: &mut bindable_enums,
            bindable_assets: &mut bindable_assets,
            bindable_artboards: &mut bindable_artboards,
            bindable_lists: &mut bindable_lists,
            bindable_triggers: &mut bindable_triggers,
            bindable_view_models: &mut bindable_view_models,
            bindable_booleans: &mut bindable_booleans,
            transition_durations: &mut transition_durations,
        };

        assert!(matches!(
            action.occurrence_value(&targets, false),
            Some(RuntimeListenerViewModelChangeValue::Number(9.0))
        ));
    }

    #[test]
    fn perform_reads_every_mutable_bindable_value_family() {
        let mut inputs = Vec::new();
        let mut reported_events = Vec::new();
        let mut bindable_numbers = vec![StateMachineBindableNumberInstance {
            global_id: 1,
            data_bind_indices: Vec::new(),
            value: 1.5,
        }];
        let mut bindable_integers = vec![StateMachineBindableIntegerInstance {
            global_id: 2,
            data_bind_indices: Vec::new(),
            value: 2,
        }];
        let mut bindable_colors = vec![StateMachineBindableColorInstance {
            global_id: 3,
            data_bind_indices: Vec::new(),
            value: 0x03040506,
        }];
        let mut bindable_strings = vec![StateMachineBindableStringInstance {
            global_id: 4,
            data_bind_indices: Vec::new(),
            value: b"live".to_vec(),
        }];
        let mut bindable_enums = vec![StateMachineBindableEnumInstance {
            global_id: 5,
            data_bind_indices: Vec::new(),
            value: 5,
        }];
        let mut bindable_assets = vec![StateMachineBindableAssetInstance {
            global_id: 6,
            data_bind_indices: Vec::new(),
            default_view_model_sources: Vec::new(),
            value: RuntimeBindableAssetValue::from_asset_index(6),
        }];
        let mut bindable_artboards = vec![StateMachineBindableArtboardInstance {
            global_id: 7,
            data_bind_indices: Vec::new(),
            value: 7,
        }];
        let mut bindable_lists = vec![StateMachineBindableListInstance {
            global_id: 8,
            data_bind_indices: Vec::new(),
            property_value: 8,
        }];
        let mut bindable_triggers = vec![StateMachineBindableTriggerInstance {
            global_id: 9,
            data_bind_indices: Vec::new(),
            value: 9,
        }];
        let mut bindable_view_models = vec![StateMachineBindableViewModelInstance::new(
            &RuntimeBindableViewModel {
                global_id: 10,
                data_bind_indices: Vec::new(),
                default_view_model_sources: Vec::new(),
                source: RuntimeBindableViewModelSource::Null,
                property_value: u64::from(u32::MAX),
            },
        )];
        bindable_view_models[0].set_value(RuntimeViewModelPointer::Imported { object_id: 10 });
        let mut bindable_booleans = vec![StateMachineBindableBooleanInstance {
            global_id: 11,
            data_bind_indices: Vec::new(),
            value: true,
        }];
        let mut transition_durations = Vec::new();
        let targets = RuntimeScheduledListenerActionTargetsMut {
            inputs: &mut inputs,
            reported_events: &mut reported_events,
            bindable_numbers: &mut bindable_numbers,
            bindable_integers: &mut bindable_integers,
            bindable_colors: &mut bindable_colors,
            bindable_strings: &mut bindable_strings,
            bindable_enums: &mut bindable_enums,
            bindable_assets: &mut bindable_assets,
            bindable_artboards: &mut bindable_artboards,
            bindable_lists: &mut bindable_lists,
            bindable_triggers: &mut bindable_triggers,
            bindable_view_models: &mut bindable_view_models,
            bindable_booleans: &mut bindable_booleans,
            transition_durations: &mut transition_durations,
        };

        let cases = [
            (
                1,
                RuntimeListenerViewModelChangeValue::Number(0.0),
                RuntimeListenerViewModelChangeValue::Number(1.5),
            ),
            (
                2,
                RuntimeListenerViewModelChangeValue::Integer(0),
                RuntimeListenerViewModelChangeValue::Integer(2),
            ),
            (
                3,
                RuntimeListenerViewModelChangeValue::Color(0),
                RuntimeListenerViewModelChangeValue::Color(0x03040506),
            ),
            (
                4,
                RuntimeListenerViewModelChangeValue::String(Vec::new()),
                RuntimeListenerViewModelChangeValue::String(b"live".to_vec()),
            ),
            (
                5,
                RuntimeListenerViewModelChangeValue::Enum(0),
                RuntimeListenerViewModelChangeValue::Enum(5),
            ),
            (
                6,
                RuntimeListenerViewModelChangeValue::Asset(
                    RuntimeBindableAssetValue::from_asset_index(0),
                ),
                RuntimeListenerViewModelChangeValue::Asset(
                    RuntimeBindableAssetValue::from_asset_index(6),
                ),
            ),
            (
                7,
                RuntimeListenerViewModelChangeValue::Artboard(0),
                RuntimeListenerViewModelChangeValue::Artboard(7),
            ),
            (
                8,
                RuntimeListenerViewModelChangeValue::List(0),
                RuntimeListenerViewModelChangeValue::List(8),
            ),
            (
                9,
                RuntimeListenerViewModelChangeValue::Trigger(0),
                RuntimeListenerViewModelChangeValue::Trigger(9),
            ),
            (
                10,
                RuntimeListenerViewModelChangeValue::ViewModel(RuntimeViewModelPointer::Null),
                RuntimeListenerViewModelChangeValue::ViewModel(RuntimeViewModelPointer::Imported {
                    object_id: 10,
                }),
            ),
            (
                11,
                RuntimeListenerViewModelChangeValue::Boolean(false),
                RuntimeListenerViewModelChangeValue::Boolean(true),
            ),
        ];

        for (global_id, authored, expected) in cases {
            let action =
                RuntimeListenerViewModelChange::for_test(0, Some(global_id), Some(authored));
            let actual = action
                .occurrence_value(&targets, false)
                .expect("the exact occurrence must resolve");
            assert_listener_view_model_change_value_eq(&actual, &expected);
        }

        let root_action = RuntimeListenerViewModelChange::for_test(
            0,
            Some(10),
            Some(RuntimeListenerViewModelChangeValue::ViewModel(
                RuntimeViewModelPointer::Null,
            )),
        );
        assert!(matches!(
            root_action.occurrence_value(&targets, true),
            Some(RuntimeListenerViewModelChangeValue::ViewModel(
                RuntimeViewModelPointer::DataContextRoot
            ))
        ));
    }

    fn assert_listener_view_model_change_value_eq(
        actual: &RuntimeListenerViewModelChangeValue,
        expected: &RuntimeListenerViewModelChangeValue,
    ) {
        match (actual, expected) {
            (
                RuntimeListenerViewModelChangeValue::Number(actual),
                RuntimeListenerViewModelChangeValue::Number(expected),
            ) => assert_eq!(actual.to_bits(), expected.to_bits()),
            (
                RuntimeListenerViewModelChangeValue::Integer(actual),
                RuntimeListenerViewModelChangeValue::Integer(expected),
            )
            | (
                RuntimeListenerViewModelChangeValue::Enum(actual),
                RuntimeListenerViewModelChangeValue::Enum(expected),
            )
            | (
                RuntimeListenerViewModelChangeValue::Artboard(actual),
                RuntimeListenerViewModelChangeValue::Artboard(expected),
            )
            | (
                RuntimeListenerViewModelChangeValue::Trigger(actual),
                RuntimeListenerViewModelChangeValue::Trigger(expected),
            )
            | (
                RuntimeListenerViewModelChangeValue::List(actual),
                RuntimeListenerViewModelChangeValue::List(expected),
            ) => assert_eq!(actual, expected),
            (
                RuntimeListenerViewModelChangeValue::Color(actual),
                RuntimeListenerViewModelChangeValue::Color(expected),
            ) => assert_eq!(actual, expected),
            (
                RuntimeListenerViewModelChangeValue::String(actual),
                RuntimeListenerViewModelChangeValue::String(expected),
            ) => assert_eq!(actual, expected),
            (
                RuntimeListenerViewModelChangeValue::Asset(actual),
                RuntimeListenerViewModelChangeValue::Asset(expected),
            ) => assert_eq!(actual.asset_index(), expected.asset_index()),
            (
                RuntimeListenerViewModelChangeValue::ViewModel(actual),
                RuntimeListenerViewModelChangeValue::ViewModel(expected),
            ) => assert_eq!(actual, expected),
            (
                RuntimeListenerViewModelChangeValue::Boolean(actual),
                RuntimeListenerViewModelChangeValue::Boolean(expected),
            ) => assert_eq!(actual, expected),
            _ => panic!("mismatched listener ViewModel value families"),
        }
    }
}
