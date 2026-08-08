use super::ScriptListenerInvocation;
use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::data_bind_container::RuntimeDataBindContainerQueue;
use crate::data_bind_graph::{
    DATA_BIND_FLAG_DIRECTION_TO_SOURCE, RuntimeDataBindGraphConverter,
    RuntimeDataBindGraphConverterState, RuntimeDataBindGraphFormulaRandomSource,
    RuntimeDataBindGraphStatefulAdvance, RuntimeDataBindGraphValue,
    data_bind_flags_apply_source_to_target, data_bind_flags_apply_target_to_source,
    runtime_cell_value_from_graph_value,
    runtime_data_bind_graph_bind_owned_converter_operands_for_data_context,
    runtime_data_bind_graph_converter, runtime_data_bind_graph_converter_contains_formula,
    runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context,
    runtime_graph_value_from_bound_cell,
};
use crate::data_converter::{
    RuntimeDataConverterBindStep, RuntimeDataConverterDataBindDefinition,
    RuntimeDataConverterDataBindState, runtime_data_converter_bind_steps,
    runtime_data_converter_data_bind_definition,
};
use crate::retained_data_bind::RuntimeRetainedDataBind;
use crate::scripted_object::{
    RuntimeScriptInputProperties, RuntimeScriptInputTargetApply, RuntimeScriptInputTargetProperty,
};
use crate::scripting::{
    RuntimeScriptInstanceHandle, ScriptCoreString, ScriptError, ScriptHost,
    ScriptListenerActionDefinition, ScriptListenerInputKind, ScriptListenerInputSnapshot,
    ScriptListenerInputSnapshotValue, ScriptValue, ScriptedStateMachineObjectKind,
};
use crate::view_model::RuntimeOwnedViewModelInstance;
use crate::view_model_cell::{RuntimeCellDirt, RuntimeCellDirtSink, RuntimeViewModelCell};
use nuxie_binary::{RuntimeDataType, RuntimeFile, RuntimeObject};
use std::collections::BTreeMap;

const DATA_BIND_FLAG_ONCE: u64 = 1 << 2;
#[cfg(test)]
const DATA_BIND_ALL_AUTHORED_FLAGS: u64 = (1 << 5) - 1;

/// Immutable source-side recipe cloned with one state-machine definition.
///
/// Pinned C++ keeps the imported `DataBind` on the source `ScriptInput`, then
/// `ScriptedObject::cloneProperties` clones both that bind and its converter
/// into every concrete `StateMachineInstance`
/// (`scripted_object.cpp:558-586`). Mutable converter state therefore belongs
/// to [`RuntimeScriptedListenerActionBindingOccurrence`], never this recipe.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeScriptedListenerActionBindingDefinition {
    action_global_id: u32,
    inputs: Vec<RuntimeScriptedListenerInputBindingDefinition>,
}

#[derive(Debug, Clone)]
struct RuntimeScriptedListenerInputBindingDefinition {
    input_global_id: u32,
    kind: ScriptListenerInputKind,
    properties: RuntimeScriptInputProperties,
    binding: Option<RuntimeScriptedListenerInputDataBindDefinition>,
}

#[derive(Debug, Clone)]
struct RuntimeScriptedListenerInputDataBindDefinition {
    /// `DataBindContainer::bindDataBindsFromContext` only binds concrete
    /// `DataBindContext` occurrences. A later plain `DataBind` still replaces
    /// the ScriptInput's earlier context bind and is cloned/owned by the
    /// state-machine occurrence, but remains inert because it has no source
    /// (`data_bind.cpp:66-95`; `data_bind_container.cpp:25-35`;
    /// `scripted_object.cpp:558-586`).
    is_context: bool,
    source_path: Vec<u32>,
    name_based: bool,
    property_key: u32,
    target_property: RuntimeScriptInputTargetProperty,
    flags: u64,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_data_binds: RuntimeDataConverterDataBindDefinition,
    unresolved_converter: bool,
}

/// Mutable clone of every ScriptInput DataBind/converter owned by one
/// concrete ScriptedListenerAction occurrence.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeScriptedListenerActionBindingOccurrence {
    action_global_id: u32,
    inputs: Vec<RuntimeScriptedListenerInputBindingOccurrence>,
}

/// Ordered cross-crate work needed to reproduce the complete
/// `DataBindContainer::bindDataBindsFromContext` walk for one listener input.
///
/// Script tables live in the facade scripting VM, while cloned DataBinds live
/// in `nuxie-runtime`. Keeping the sequence explicit preserves the C++ order:
/// bind the outer input, bind one converter occurrence, rehydrate it before
/// the next occurrence binds, then retain the complete dependency set
/// (`data_bind_container.cpp:25-35`; `data_bind_context.cpp:56-89`;
/// `data_converter_group.cpp:63-75`;
/// `scripted_data_converter.cpp:170-188`).
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuntimeScriptedListenerDataConverterBindStep {
    BindListenerInput {
        action_global_id: u32,
        listener_input_global_id: u32,
    },
    BindConverter {
        action_global_id: u32,
        listener_input_global_id: u32,
        converter_path: Vec<usize>,
    },
    Rehydrate {
        action_global_id: u32,
        listener_input_global_id: u32,
        converter_path: Vec<usize>,
        converter_global_id: u32,
        inits: bool,
    },
    RebindFinalInput {
        action_global_id: u32,
        listener_input_global_id: u32,
        converter_path: Vec<usize>,
        converter_input_index: usize,
        data_bind_index: usize,
    },
    FinalizeListenerInput {
        action_global_id: u32,
        listener_input_global_id: u32,
    },
}

#[derive(Debug, Clone)]
struct RuntimeScriptedListenerInputBindingOccurrence {
    input_global_id: u32,
    kind: ScriptListenerInputKind,
    properties: RuntimeScriptInputProperties,
    binding: Option<RuntimeScriptedListenerInputDataBindOccurrence>,
}

#[derive(Debug, Clone)]
struct RuntimeScriptedListenerInputDataBindOccurrence {
    is_context: bool,
    source_path: Vec<u32>,
    name_based: bool,
    property_key: u32,
    target_property: RuntimeScriptInputTargetProperty,
    flags: u64,
    retained_bind: RuntimeRetainedDataBind,
    converter: Option<RuntimeDataBindGraphConverter>,
    converter_state: RuntimeDataBindGraphConverterState,
    converter_data_binds: RuntimeDataConverterDataBindState,
    formula_random_source: RuntimeDataBindGraphFormulaRandomSource,
    /// Formula is independently registered on the outer primary source even
    /// when the DataBind is `bindsOnce` or target-to-source-only. Keep that
    /// dependency separate so it clears sourceChange randoms without
    /// scheduling an otherwise sleeping outer DataBind.
    formula_source: Option<RuntimeViewModelCell>,
    formula_source_sink: RuntimeCellDirtSink,
    unresolved_converter: bool,
    last_source: Option<RuntimeDataBindGraphValue>,
    last_target: Option<RuntimeDataBindGraphValue>,
}

impl RuntimeScriptedListenerInputDataBindOccurrence {
    fn attach_converter_parent(&mut self) {
        if self.converter.is_none() {
            return;
        }
        let wake = self.retained_bind.converter_parent_wake();
        self.converter_data_binds
            .set_parent_wake(wake, &mut self.converter_state);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeScriptedListenerBoundValue {
    Value(ScriptValue),
    Artboard(u64),
    Trigger(u64),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeScriptedListenerBindingUpdate {
    pub(crate) action_global_id: u32,
    pub(crate) input_name: ScriptCoreString,
    pub(crate) value: RuntimeScriptedListenerBoundValue,
}

/// Import one pinned C++ `ScriptedListenerAction` definition and its ordered
/// typed input referencers.
pub(crate) fn runtime_scripted_listener_action_definition(
    file: &RuntimeFile,
    action: &RuntimeObject,
    inputs: &[&RuntimeObject],
) -> Option<ScriptListenerActionDefinition> {
    if action.type_name != "ScriptedListenerAction" {
        return None;
    }
    runtime_scripted_object_definition(file, action, inputs)
}

/// Import one state-machine-owned C++ `ScriptedObject` occurrence. Both
/// listener actions and transition conditions clone through the same
/// `StateMachineInstance::m_scriptedObjectsMap` lifecycle.
pub(crate) fn runtime_scripted_object_definition(
    file: &RuntimeFile,
    object: &RuntimeObject,
    inputs: &[&RuntimeObject],
) -> Option<ScriptListenerActionDefinition> {
    let kind = match object.type_name {
        "ScriptedListenerAction" => ScriptedStateMachineObjectKind::ListenerAction,
        "ScriptedTransitionCondition" => ScriptedStateMachineObjectKind::TransitionCondition,
        _ => return None,
    };
    // Generated C++ initializes the uint32 field to -1. Import registers and
    // adds this ScriptedObject to the StateMachine before Backboard resolution;
    // a missing, out-of-range, or wrong-typed asset therefore leaves a
    // retained occurrence with a null ScriptAsset rather than deleting it
    // (`scripted_listener_action_base.hpp`,
    // `scripted_listener_action.cpp:120-139`,
    // `backboard_importer.cpp:84-101`,
    // `scripted_object.cpp:548-555`).
    let asset_ordinal = object
        .uint_property("scriptAssetId")
        .unwrap_or(u64::from(u32::MAX)) as u32 as usize;
    let asset = file.resolved_file_asset_for_referencer(object);
    let asset_name = asset
        .and_then(|asset| (asset.type_name == "ScriptAsset").then_some(asset))
        .and_then(|asset| asset.string_property("name"))
        .unwrap_or_default()
        .to_owned();
    let has_protocol_asset = asset.is_some_and(|asset| {
        asset.type_name == "ScriptAsset" && !asset.bool_property("isModule").unwrap_or(false)
    });
    // OptionalScriptedMethods starts at zero. ScriptAsset copies the
    // serialized bitfield only after the referencer resolves to an executable
    // protocol asset with a generator; old valid assets inherit the generated
    // all-bits property default. A missing, wrong-typed, or module asset never
    // performs that copy and therefore remains inert with mask zero
    // (`script_asset.hpp:97-108`; `script_asset.cpp:139-161`;
    // `scripted_object.cpp:548-555`).
    let serialized_implemented_methods = has_protocol_asset
        .then(|| {
            asset
                .and_then(|asset| asset.uint_property("serializedImplementedMethods"))
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK)
        })
        .unwrap_or(0);
    let inputs = inputs
        .iter()
        .filter_map(|input| {
            let kind = match input.type_name {
                "ScriptInputBoolean" => crate::ScriptListenerInputKind::Boolean,
                "ScriptInputNumber" => crate::ScriptListenerInputKind::Number,
                "ScriptInputColor" => crate::ScriptListenerInputKind::Color,
                "ScriptInputString" => crate::ScriptListenerInputKind::String,
                "ScriptInputTrigger" => crate::ScriptListenerInputKind::Trigger,
                "ScriptInputArtboard" => crate::ScriptListenerInputKind::Artboard,
                "ScriptInputViewModelProperty" => crate::ScriptListenerInputKind::ViewModelProperty,
                _ => return None,
            };
            Some(crate::ScriptListenerInputDefinition::new(input.id, kind))
        })
        .collect();
    Some(ScriptListenerActionDefinition::with_inputs_and_kind(
        object.id,
        kind,
        asset_ordinal,
        asset_name,
        has_protocol_asset,
        serialized_implemented_methods,
        inputs,
    ))
}

/// Capture the exact DataBind currently retained by every imported
/// ScriptInput.
///
/// `ScriptInput::dataBind` is a single pointer and a later authored bind
/// overwrites an earlier one (`data_bind.cpp:66-95`). Iterate the file in
/// authored order and deliberately keep the last matching occurrence.
pub(crate) fn runtime_scripted_listener_action_binding_definition(
    file: &RuntimeFile,
    action: &RuntimeObject,
    inputs: &[&RuntimeObject],
) -> Option<RuntimeScriptedListenerActionBindingDefinition> {
    if action.type_name != "ScriptedListenerAction" {
        return None;
    }
    runtime_scripted_object_binding_definition(file, action, inputs)
}

pub(crate) fn runtime_scripted_object_binding_definition(
    file: &RuntimeFile,
    object: &RuntimeObject,
    inputs: &[&RuntimeObject],
) -> Option<RuntimeScriptedListenerActionBindingDefinition> {
    if !matches!(
        object.type_name,
        "ScriptedListenerAction" | "ScriptedTransitionCondition" | "ScriptedInterpolator"
    ) {
        return None;
    }
    let inputs = inputs
        .iter()
        .filter_map(|input| {
            let kind = script_listener_input_kind(input)?;
            let data_bind = (0..file.object_count())
                .filter_map(|id| file.object(id))
                .filter(|data_bind| {
                    nuxie_schema::definition_by_name(data_bind.type_name)
                        .is_some_and(|definition| definition.is_a("DataBind"))
                })
                .filter(|data_bind| {
                    file.data_bind_target_for_object(data_bind)
                        .is_some_and(|target| target.id == input.id)
                })
                .last();
            let binding = data_bind.map(|data_bind| {
                let is_context = data_bind.type_name == "DataBindContext";
                let property_key = data_bind
                    .uint_property("propertyKey")
                    .and_then(|value| u32::try_from(value).ok())
                    .unwrap_or(u32::MAX);
                // Keep the authored path id here. C++ expands a name-based
                // manifest path exactly once when the occurrence binds, then
                // latches `m_isPathResolved`; storing the expanded name ids
                // and asking the context resolver to expand again can legally
                // select a different manifest path.
                let source_path = is_context
                    .then(|| file.data_bind_context_source_path_ids_for_object(data_bind))
                    .flatten()
                    .unwrap_or_default();
                let converter_object = file.resolved_data_converter_for_data_bind_object(data_bind);
                let converter = runtime_data_bind_graph_converter(file, data_bind);
                let converter_data_binds = converter_object
                    .zip(converter.as_ref())
                    .map(|(converter_object, converter)| {
                        runtime_data_converter_data_bind_definition(
                            file,
                            converter_object,
                            converter,
                        )
                    })
                    .unwrap_or_default();
                let unresolved_converter = converter.is_none() && converter_object.is_some();
                RuntimeScriptedListenerInputDataBindDefinition {
                    is_context,
                    source_path,
                    name_based: is_context
                        && file
                            .data_bind_is_name_based_for_object(data_bind)
                            .unwrap_or(false),
                    property_key,
                    target_property: RuntimeScriptInputProperties::property_for_key(
                        kind,
                        property_key,
                    ),
                    flags: data_bind.uint_property("flags").unwrap_or(0),
                    converter,
                    converter_data_binds,
                    unresolved_converter,
                }
            });
            Some(RuntimeScriptedListenerInputBindingDefinition {
                input_global_id: input.id,
                kind,
                properties: RuntimeScriptInputProperties::from_object(file, input, kind),
                binding,
            })
        })
        .collect();
    Some(RuntimeScriptedListenerActionBindingDefinition {
        action_global_id: object.id,
        inputs,
    })
}

fn script_listener_input_kind(input: &RuntimeObject) -> Option<ScriptListenerInputKind> {
    Some(match input.type_name {
        "ScriptInputBoolean" => ScriptListenerInputKind::Boolean,
        "ScriptInputNumber" => ScriptListenerInputKind::Number,
        "ScriptInputColor" => ScriptListenerInputKind::Color,
        "ScriptInputString" => ScriptListenerInputKind::String,
        "ScriptInputTrigger" => ScriptListenerInputKind::Trigger,
        "ScriptInputArtboard" => ScriptListenerInputKind::Artboard,
        "ScriptInputViewModelProperty" => ScriptListenerInputKind::ViewModelProperty,
        _ => return None,
    })
}

impl RuntimeScriptedListenerActionBindingDefinition {
    pub(crate) fn instantiate(&self) -> RuntimeScriptedListenerActionBindingOccurrence {
        RuntimeScriptedListenerActionBindingOccurrence {
            action_global_id: self.action_global_id,
            inputs: self
                .inputs
                .iter()
                .map(RuntimeScriptedListenerInputBindingOccurrence::from_definition)
                .collect(),
        }
    }
}

impl RuntimeScriptedListenerInputBindingOccurrence {
    fn from_definition(definition: &RuntimeScriptedListenerInputBindingDefinition) -> Self {
        Self {
            input_global_id: definition.input_global_id,
            kind: definition.kind,
            properties: definition.properties.clone_for_scripted_object(),
            binding: definition.binding.as_ref().map(|binding| {
                let retained_bind = RuntimeRetainedDataBind::new(
                    binding.flags,
                    binding.flags & DATA_BIND_FLAG_ONCE != 0,
                );
                let mut occurrence = RuntimeScriptedListenerInputDataBindOccurrence {
                    is_context: binding.is_context,
                    source_path: binding.source_path.clone(),
                    name_based: binding.name_based,
                    property_key: binding.property_key,
                    target_property: binding.target_property,
                    flags: binding.flags,
                    retained_bind,
                    converter: binding.converter.clone(),
                    converter_state: RuntimeDataBindGraphConverterState::for_converter(
                        binding.converter.as_ref(),
                    ),
                    converter_data_binds: binding.converter_data_binds.instantiate(),
                    formula_random_source: RuntimeDataBindGraphFormulaRandomSource::process_global(
                    ),
                    formula_source: None,
                    formula_source_sink: RuntimeCellDirtSink::new(),
                    unresolved_converter: binding.unresolved_converter,
                    last_source: None,
                    last_target: None,
                };
                occurrence.attach_converter_parent();
                occurrence
            }),
        }
    }
}

impl RuntimeScriptedListenerActionBindingOccurrence {
    pub(crate) fn action_global_id(&self) -> u32 {
        self.action_global_id
    }

    /// Clone-owned ScriptInput DataBinds join the same state-machine
    /// `DataBindContainer` after the machine's ordinary binds, preserving
    /// custom-property order (`scripted_object.cpp:558-586`).
    pub(crate) fn add_data_binds_to_container(
        &mut self,
        container: &mut RuntimeDataBindContainerQueue,
    ) -> Vec<(usize, usize)> {
        self.inputs
            .iter_mut()
            .enumerate()
            .filter_map(|(input_index, input)| {
                let binding = input.binding.as_mut()?;
                let occurrence = container.add_data_bind(&mut binding.retained_bind, false);
                Some((occurrence, input_index))
            })
            .collect()
    }

    pub(crate) fn data_bind_is_to_source(&self, input_index: usize) -> bool {
        self.inputs
            .get(input_index)
            .and_then(|input| input.binding.as_ref())
            .is_some_and(|binding| data_bind_flags_apply_target_to_source(binding.flags))
    }

    pub(crate) fn input_snapshots(&self) -> Vec<ScriptListenerInputSnapshot> {
        self.inputs
            .iter()
            .map(|input| {
                let value = if input.kind == ScriptListenerInputKind::Artboard {
                    input
                        .properties
                        .artboard_referenced_id()
                        .map(ScriptListenerInputSnapshotValue::Artboard)
                } else {
                    match (input.kind, input.properties.value()) {
                        (
                            ScriptListenerInputKind::Boolean,
                            Some(RuntimeDataBindGraphValue::Boolean(value)),
                        ) => Some(ScriptListenerInputSnapshotValue::Value(ScriptValue::Bool(
                            *value,
                        ))),
                        (
                            ScriptListenerInputKind::Number,
                            Some(RuntimeDataBindGraphValue::Number(value)),
                        ) => Some(ScriptListenerInputSnapshotValue::Value(
                            ScriptValue::Number(f64::from(*value)),
                        )),
                        (
                            ScriptListenerInputKind::Color,
                            Some(RuntimeDataBindGraphValue::Color(value)),
                        ) => Some(ScriptListenerInputSnapshotValue::Value(ScriptValue::Color(
                            *value,
                        ))),
                        (
                            ScriptListenerInputKind::String,
                            Some(RuntimeDataBindGraphValue::String(value)),
                        ) => Some(ScriptListenerInputSnapshotValue::Value(
                            ScriptValue::CoreString(ScriptCoreString::from_bytes(value.clone())),
                        )),
                        // ScriptInputTrigger has no hydration callback. Its
                        // nonzero edge is delivered only by DataBind update.
                        (ScriptListenerInputKind::Trigger, _)
                        | (ScriptListenerInputKind::ViewModelProperty, _)
                        | (ScriptListenerInputKind::Artboard, _) => None,
                        _ => None,
                    }
                };
                ScriptListenerInputSnapshot {
                    input_global_id: input.input_global_id,
                    kind: input.kind,
                    name: input.properties.name().clone(),
                    value,
                    view_model_path: input.properties.view_model_path().cloned(),
                }
            })
            .collect()
    }

    pub(crate) fn fresh_clone(&self) -> Self {
        Self {
            action_global_id: self.action_global_id,
            inputs: self
                .inputs
                .iter()
                .map(|input| RuntimeScriptedListenerInputBindingOccurrence {
                    input_global_id: input.input_global_id,
                    kind: input.kind,
                    properties: input.properties.clone_for_scripted_object(),
                    binding: input.binding.as_ref().map(|binding| {
                        let mut converter = binding.converter.clone();
                        if let Some(converter) = converter.as_mut() {
                            converter.detach_scripted_instance();
                            converter.clear_retained_owned_operands();
                        }
                        let mut occurrence = RuntimeScriptedListenerInputDataBindOccurrence {
                            is_context: binding.is_context,
                            source_path: binding.source_path.clone(),
                            name_based: binding.name_based,
                            property_key: binding.property_key,
                            target_property: binding.target_property,
                            flags: binding.flags,
                            retained_bind: RuntimeRetainedDataBind::new(
                                binding.flags,
                                binding.flags & DATA_BIND_FLAG_ONCE != 0,
                            ),
                            converter_state: RuntimeDataBindGraphConverterState::for_converter(
                                converter.as_ref(),
                            ),
                            converter_data_binds: binding.converter_data_binds.fresh_clone(),
                            converter,
                            formula_random_source:
                                RuntimeDataBindGraphFormulaRandomSource::process_global(),
                            formula_source: None,
                            formula_source_sink: RuntimeCellDirtSink::new(),
                            unresolved_converter: binding.unresolved_converter,
                            last_source: None,
                            last_target: None,
                        };
                        occurrence.attach_converter_parent();
                        occurrence
                    }),
                })
                .collect(),
        }
    }

    /// Transactional adoption preserves the live script/input values, but the
    /// candidate must own every nested converter notification queue.
    pub(crate) fn rehomed_clone(&self) -> Self {
        let mut cloned = self.clone();
        for input in &mut cloned.inputs {
            if let Some(binding) = input.binding.as_mut() {
                binding.converter_data_binds = binding.converter_data_binds.rehomed_clone();
                binding.attach_converter_parent();
            }
        }
        cloned
    }

    pub(crate) fn scripted_converter_targets(&self) -> Vec<(u32, Vec<usize>, u32, bool)> {
        self.scripted_converter_occurrences()
            .into_iter()
            .filter_map(|(input_global_id, path, global_id, inits, attached)| {
                (!attached).then_some((input_global_id, path, global_id, inits))
            })
            .collect()
    }

    pub(crate) fn scripted_converter_occurrences(&self) -> Vec<(u32, Vec<usize>, u32, bool, bool)> {
        let mut occurrences = Vec::new();
        for input in &self.inputs {
            let Some(converter) = input
                .binding
                .as_ref()
                .filter(|binding| binding.is_context)
                .and_then(|binding| binding.converter.as_ref())
            else {
                continue;
            };
            collect_scripted_converter_occurrences(
                converter,
                input.input_global_id,
                &mut Vec::new(),
                &mut occurrences,
            );
        }
        occurrences
    }

    pub(crate) fn scripted_converter_bind_steps(
        &self,
    ) -> Vec<RuntimeScriptedListenerDataConverterBindStep> {
        let mut steps = Vec::new();
        for input in &self.inputs {
            let Some(binding) = input.binding.as_ref().filter(|binding| binding.is_context) else {
                continue;
            };
            steps.push(
                RuntimeScriptedListenerDataConverterBindStep::BindListenerInput {
                    action_global_id: self.action_global_id,
                    listener_input_global_id: input.input_global_id,
                },
            );
            let Some(converter) = binding.converter.as_ref() else {
                continue;
            };
            collect_scripted_converter_bind_steps(
                converter,
                self.action_global_id,
                input.input_global_id,
                &mut steps,
            );
            steps.push(
                RuntimeScriptedListenerDataConverterBindStep::FinalizeListenerInput {
                    action_global_id: self.action_global_id,
                    listener_input_global_id: input.input_global_id,
                },
            );
        }
        steps
    }

    pub(crate) fn has_scripted_converter_instance_at_path(
        &self,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> bool {
        self.scripted_converter_instance_at_path(input_global_id, converter_path)
            .is_some()
    }

    pub(crate) fn attach_scripted_converter_instance_at_path(
        &mut self,
        input_global_id: u32,
        converter_path: &[usize],
        handle: &RuntimeScriptInstanceHandle,
    ) -> bool {
        let Some(converter) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
            .and_then(|binding| binding.converter.as_mut())
        else {
            return false;
        };
        converter.attach_scripted_instance_at_path(converter_path, handle)
    }

    pub(crate) fn scripted_converter_input_snapshots(
        &self,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> Option<Vec<ScriptListenerInputSnapshot>> {
        let binding = self
            .inputs
            .iter()
            .find(|input| input.input_global_id == input_global_id)?
            .binding
            .as_ref()
            .filter(|binding| binding.is_context)?;
        let converter = binding.converter.as_ref()?;
        binding
            .converter_state
            .scripted_converter_input_snapshots_at_path(converter, converter_path)
    }

    pub(crate) fn scripted_converter_instance_at_path(
        &self,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> Option<RuntimeScriptInstanceHandle> {
        self.inputs
            .iter()
            .find(|input| input.input_global_id == input_global_id)?
            .binding
            .as_ref()
            .filter(|binding| binding.is_context)?
            .converter
            .as_ref()?
            .scripted_instance_at_path(converter_path)
    }

    /// `ScriptedDataConverter::didHydrateScriptInputs` calls
    /// `DataConverter::markConverterDirty` after every complete successful
    /// hydrate/init and before the custom ScriptInput DataBinds are rebound
    /// (`scripted_object.cpp:399-437`;
    /// `scripted_data_converter.cpp:45-48,170-188`).
    pub(crate) fn mark_scripted_converter_hydrated(
        &mut self,
        input_global_id: u32,
        converter_path: &[usize],
    ) -> bool {
        if self
            .scripted_converter_instance_at_path(input_global_id, converter_path)
            .is_none()
        {
            return false;
        }
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        Self::mark_converter_dirty(binding);
        true
    }

    pub(crate) fn resolve(
        &mut self,
        file: &RuntimeFile,
        _context: &RuntimeOwnedViewModelInstance,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<RuntimeScriptedListenerBoundValue>, ScriptError> {
        let action_global_id = self.action_global_id;
        let input = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "ScriptedListenerAction global {action_global_id} has no input global {input_global_id}",
                ))
            })?;
        let Some(binding) = input.binding.as_mut() else {
            return Ok(None);
        };
        if !binding.is_context {
            return Ok(None);
        }
        if binding.unresolved_converter {
            return Err(ScriptError::new(format!(
                "ScriptedListenerAction global {} input global {input_global_id} references an unresolved data converter",
                action_global_id
            )));
        }
        // `ScriptedObject::bind` resolves and retains the source occurrence.
        // Steady updates read that retained object; C++ does not re-walk the
        // authored path until an explicit rebind.
        let Some(source) = binding
            .retained_bind
            .source()
            .and_then(|source| runtime_graph_value_from_bound_cell(&source.value()))
        else {
            return Ok(None);
        };
        Self::update_resolved_source(action_global_id, input, file, source, emit_unchanged)
    }

    pub(crate) fn resolve_from_data_context(
        &mut self,
        file: &RuntimeFile,
        _data_context: &RuntimeOwnedDataContext,
        input_global_id: u32,
        emit_unchanged: bool,
    ) -> Result<Option<RuntimeScriptedListenerBoundValue>, ScriptError> {
        let action_global_id = self.action_global_id;
        let input = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .ok_or_else(|| {
                ScriptError::new(format!(
                    "ScriptedListenerAction global {action_global_id} has no input global {input_global_id}",
                ))
            })?;
        let Some(binding) = input.binding.as_mut() else {
            return Ok(None);
        };
        if !binding.is_context {
            return Ok(None);
        }
        if binding.unresolved_converter {
            return Err(ScriptError::new(format!(
                "ScriptedListenerAction global {action_global_id} input global {input_global_id} references an unresolved data converter",
            )));
        }
        let Some(source) = binding
            .retained_bind
            .source()
            .and_then(|source| runtime_graph_value_from_bound_cell(&source.value()))
        else {
            return Ok(None);
        };
        Self::update_resolved_source(action_global_id, input, file, source, emit_unchanged)
    }

    fn bind_resolved_source(
        binding: &mut RuntimeScriptedListenerInputDataBindOccurrence,
        source_cell: Option<RuntimeViewModelCell>,
        force_reconcile: bool,
    ) {
        let source_resolved = source_cell.is_some();
        let source_rebound = match (binding.retained_bind.source(), source_cell.as_ref()) {
            (Some(current), Some(next)) => !current.ptr_eq(next),
            // DataBindContext::bindFromContext takes the unbind branch whenever
            // the freshly resolved source is null, including null -> null.
            // That recursive unbind clears converter-owned input sources
            // before the converter is rebound/reinitialized below.
            (None, None) => force_reconcile,
            _ => true,
        };
        if source_rebound {
            // `DataBind::unbind` always clears the outer source before the
            // converter's virtual unbind (`data_bind.cpp:354-369`).
            binding.retained_bind.clear_source();
            if source_resolved {
                if let Some(converter) = binding.converter.as_ref() {
                    binding
                        .converter_state
                        .reset_for_data_bind_rebind(converter);
                }
            } else if let Some(converter) = binding.converter.as_ref() {
                binding
                    .converter_data_binds
                    .unbind(converter, &mut binding.converter_state);
            }
            if let Some(source_cell) = source_cell {
                binding.retained_bind.set_source(source_cell);
            }
            Self::sync_formula_source(binding);
        }
        if source_resolved && (source_rebound || force_reconcile) {
            binding.retained_bind.mark_rebind_reconcile();
        }
    }

    pub(crate) fn bind_listener_input_source(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        input_global_id: u32,
        explicit_rebind: bool,
    ) -> bool {
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        let source_cell = context
            .property_path_for_context_source_path_with_persistent_resolver(
                file,
                &[],
                &binding.source_path,
                binding.name_based,
            )
            .and_then(|property_path| context.cell_by_property_path(&property_path));
        Self::bind_resolved_source(binding, source_cell, explicit_rebind);
        true
    }

    pub(crate) fn bind_listener_input_source_from_data_context(
        &mut self,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        input_global_id: u32,
        explicit_rebind: bool,
    ) -> bool {
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        let source_path = binding.source_path.clone();
        let name_based = binding.name_based;
        let source_cell = data_context.resolve_instance(&mut |_, context, scope_path| {
            let property_path = context
                .property_path_for_context_source_path_with_persistent_resolver(
                    file,
                    scope_path,
                    &source_path,
                    name_based,
                )?;
            context.cell_by_property_path(&property_path)
        });
        Self::bind_resolved_source(binding, source_cell, explicit_rebind);
        true
    }

    pub(crate) fn bind_converter_own_sources_at_path(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        input_global_id: u32,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        let Some(converter) = binding.converter.as_mut() else {
            return false;
        };
        binding.converter_data_binds.bind_own_sources_at_path(
            converter,
            &mut binding.converter_state,
            converter_path,
            file,
            context,
            explicit_rebind,
        )
    }

    pub(crate) fn bind_converter_own_sources_from_data_context_at_path(
        &mut self,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        input_global_id: u32,
        converter_path: &[usize],
        explicit_rebind: bool,
    ) -> bool {
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        let Some(converter) = binding.converter.as_mut() else {
            return false;
        };
        binding
            .converter_data_binds
            .bind_own_sources_from_data_context_at_path(
                converter,
                &mut binding.converter_state,
                converter_path,
                file,
                data_context,
                explicit_rebind,
            )
    }

    pub(crate) fn finalize_listener_input_sources(&mut self, input_global_id: u32) -> bool {
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        Self::refresh_additional_sources(binding);
        true
    }

    fn update_resolved_source(
        action_global_id: u32,
        input: &mut RuntimeScriptedListenerInputBindingOccurrence,
        file: &RuntimeFile,
        source: RuntimeDataBindGraphValue,
        _emit_unchanged: bool,
    ) -> Result<Option<RuntimeScriptedListenerBoundValue>, ScriptError> {
        let binding = input
            .binding
            .as_mut()
            .expect("source resolution requires an authored DataBind");
        binding.retained_bind.collect_source_dirt();
        if !binding.formula_source_sink.take_dirt().is_empty()
            && let Some(converter) = binding.converter.as_ref()
        {
            binding
                .converter_state
                .reset_source_change_formula_randoms(converter);
        }
        // `updateDataBinds(false)` clears target-origin reconciliation dirt
        // without pulling the target. Target→source runs only at the explicit
        // `updateDataBinds(true)` boundary
        // (`data_bind_container.cpp:115-147`).
        binding.retained_bind.take_target_dirt();
        Self::apply_resolved_source_to_target(action_global_id, input, file, source)
    }

    fn apply_resolved_source_to_target(
        action_global_id: u32,
        input: &mut RuntimeScriptedListenerInputBindingOccurrence,
        file: &RuntimeFile,
        source: RuntimeDataBindGraphValue,
    ) -> Result<Option<RuntimeScriptedListenerBoundValue>, ScriptError> {
        let input_global_id = input.input_global_id;
        let binding = input
            .binding
            .as_mut()
            .expect("source application requires an authored DataBind");
        if !binding.retained_bind.take_pending_source_dirt()
            || !data_bind_flags_apply_source_to_target(binding.flags)
        {
            return Ok(None);
        }
        let uses_artboard_referencer = input.kind == ScriptListenerInputKind::Artboard
            && matches!(source, RuntimeDataBindGraphValue::Artboard(_))
            && binding.converter.as_ref().is_none_or(|converter| {
                matches!(
                    converter.cpp_output_data_type(),
                    RuntimeDataType::None | RuntimeDataType::Input
                )
            });
        let target_apply = if uses_artboard_referencer {
            let RuntimeDataBindGraphValue::Artboard(artboard_id) = source else {
                unreachable!("Artboard referencer path was selected from an Artboard source")
            };
            input.properties.apply_artboard_source(file, artboard_id)
        } else {
            let target = match binding.converter.as_mut() {
                Some(converter) => {
                    let converted = if binding.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0 {
                        binding
                            .converter_state
                            .reverse_convert_value_with_formula_randoms_for_scripted_listener(
                                converter,
                                &source,
                                &mut binding.formula_random_source,
                            )?
                    } else {
                        binding
                            .converter_state
                            .convert_value_with_formula_randoms_for_scripted_listener(
                                converter,
                                &source,
                                &mut binding.formula_random_source,
                            )?
                    };
                    converted.ok_or_else(|| {
                        ScriptError::new(format!(
                            "ScriptedListenerAction global {action_global_id} input global {input_global_id} data converter produced no value for its target property",
                        ))
                    })?
                }
                None => source.clone(),
            };
            input
                .properties
                .apply_target(file, input.kind, binding.target_property, target)
        };
        binding.last_source = Some(source);
        binding.last_target = input.properties.value().cloned();
        if target_apply != RuntimeScriptInputTargetApply::ChangedWithTableProjection {
            return Ok(None);
        }
        let Some(projected_target) = input.properties.projection_value(input.kind) else {
            return Ok(None);
        };
        runtime_scripted_listener_bound_value(input.kind, projected_target).map(Some)
    }

    /// Register every cloned DataBind against a live root context without
    /// consuming source dirt or applying values. C++ performs this before
    /// `initScriptedObjects`; `updateDataBinds(false)` applies the source later.
    pub(crate) fn bind_sources(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        explicit_rebind: bool,
    ) {
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if !binding.is_context {
                continue;
            }
            let source_cell = context
                .property_path_for_context_source_path_with_persistent_resolver(
                    file,
                    &[],
                    &binding.source_path,
                    binding.name_based,
                )
                .and_then(|property_path| context.cell_by_property_path(&property_path));
            Self::bind_resolved_source(binding, source_cell, explicit_rebind);
            if let Some(converter) = binding.converter.as_mut() {
                binding.converter_data_binds.bind_sources(
                    converter,
                    &mut binding.converter_state,
                    file,
                    context,
                    explicit_rebind,
                );
                runtime_data_bind_graph_refresh_operation_view_model_converter_for_owned_context(
                    converter,
                    context,
                    &[&[]],
                );
                let mut operand_cells = Vec::new();
                converter.retained_operand_cells(&mut operand_cells);
                binding.retained_bind.set_additional_sources(operand_cells);
            }
        }
    }

    /// DataContext-scoped companion to [`Self::bind_sources`]. Converter
    /// operands resolve independently against the original outer DataContext,
    /// not the child instance selected by the primary source.
    pub(crate) fn bind_sources_from_data_context(
        &mut self,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        explicit_rebind: bool,
    ) {
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if !binding.is_context {
                continue;
            }
            let source_path = binding.source_path.clone();
            let name_based = binding.name_based;
            let source_cell = data_context.resolve_instance(&mut |_, context, scope_path| {
                let property_path = context
                    .property_path_for_context_source_path_with_persistent_resolver(
                        file,
                        scope_path,
                        &source_path,
                        name_based,
                    )?;
                context.cell_by_property_path(&property_path)
            });
            Self::bind_resolved_source(binding, source_cell, explicit_rebind);
            if let Some(converter) = binding.converter.as_mut() {
                binding.converter_data_binds.bind_sources_from_data_context(
                    converter,
                    &mut binding.converter_state,
                    file,
                    data_context,
                    explicit_rebind,
                );
                runtime_data_bind_graph_bind_owned_converter_operands_for_data_context(
                    converter,
                    data_context,
                );
                let mut operand_cells = Vec::new();
                converter.retained_operand_cells(&mut operand_cells);
                binding.retained_bind.set_additional_sources(operand_cells);
            }
        }
    }

    pub(crate) fn rebind_scripted_converter_final_input(
        &mut self,
        file: &RuntimeFile,
        root_context: Option<&RuntimeOwnedViewModelInstance>,
        data_context: Option<&RuntimeOwnedDataContext>,
        listener_input_global_id: u32,
        converter_path: &[usize],
        converter_input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        let Some(binding) = self
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == listener_input_global_id)
            .and_then(|input| input.binding.as_mut())
            .filter(|binding| binding.is_context)
        else {
            return false;
        };
        let Some(converter) = binding.converter.as_mut() else {
            return false;
        };
        let rebound = if let Some(data_context) = data_context {
            binding
                .converter_state
                .bind_scripted_converter_input_source_from_data_context_at_path(
                    converter,
                    converter_path,
                    converter_input_index,
                    data_bind_index,
                    file,
                    data_context,
                )
        } else if let Some(root_context) = root_context {
            binding
                .converter_state
                .bind_scripted_converter_input_source_at_path(
                    converter,
                    converter_path,
                    converter_input_index,
                    data_bind_index,
                    file,
                    root_context,
                )
        } else {
            false
        };
        if !rebound {
            return false;
        }
        let mut operand_cells = Vec::new();
        converter.retained_operand_cells(&mut operand_cells);
        binding.retained_bind.set_additional_sources(operand_cells);
        true
    }

    fn unbind_source(binding: &mut RuntimeScriptedListenerInputDataBindOccurrence) {
        // C++ clears the outer DataBind source before dispatching the
        // converter-specific virtual unbind (`data_bind.cpp:354-369`).
        binding.retained_bind.clear_source();
        if let Some(converter) = binding.converter.as_ref() {
            binding
                .converter_data_binds
                .unbind(converter, &mut binding.converter_state);
        }
        Self::clear_formula_source(binding);
        Self::refresh_additional_sources(binding);
    }

    fn sync_formula_source(binding: &mut RuntimeScriptedListenerInputDataBindOccurrence) {
        let should_bind = binding
            .converter
            .as_ref()
            .is_some_and(runtime_data_bind_graph_converter_contains_formula);
        let next = should_bind
            .then(|| binding.retained_bind.source().cloned())
            .flatten();
        if binding
            .formula_source
            .as_ref()
            .zip(next.as_ref())
            .is_some_and(|(current, next)| current.ptr_eq(next))
            || (binding.formula_source.is_none() && next.is_none())
        {
            return;
        }
        Self::clear_formula_source(binding);
        if let Some(next) = next {
            next.add_dependent(&binding.formula_source_sink);
            binding.formula_source = Some(next);
            binding.formula_source_sink.take_dirt();
        }
    }

    fn clear_formula_source(binding: &mut RuntimeScriptedListenerInputDataBindOccurrence) {
        if let Some(source) = binding.formula_source.take() {
            source.remove_dependent(&binding.formula_source_sink);
        }
        binding.formula_source_sink.take_dirt();
    }

    fn refresh_additional_sources(binding: &mut RuntimeScriptedListenerInputDataBindOccurrence) {
        let Some(converter) = binding.converter.as_ref() else {
            binding.retained_bind.set_additional_sources(Vec::new());
            return;
        };
        let mut operand_cells = Vec::new();
        converter.retained_operand_cells(&mut operand_cells);
        binding.retained_bind.set_additional_sources(operand_cells);
    }

    fn mark_converter_dirty(binding: &mut RuntimeScriptedListenerInputDataBindOccurrence) {
        binding.retained_bind.mark_converter_changed();
    }

    pub(crate) fn update_scripted_converter_inputs<F>(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        apply: &mut F,
    ) -> Result<(), ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if !binding.is_context {
                continue;
            }
            let Some(converter) = binding.converter.as_mut() else {
                continue;
            };
            binding.converter_data_binds.update(
                converter,
                &mut binding.converter_state,
                file,
                context,
                None,
                apply,
            )?;
        }
        Ok(())
    }

    pub(crate) fn update_scripted_converter_inputs_from_data_context<F>(
        &mut self,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        apply: &mut F,
    ) -> Result<(), ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if !binding.is_context {
                continue;
            }
            let Some(converter) = binding.converter.as_mut() else {
                continue;
            };
            binding.converter_data_binds.update_from_data_context(
                converter,
                &mut binding.converter_state,
                file,
                data_context,
                None,
                apply,
            )?;
        }
        Ok(())
    }

    /// Public `DataBindContainer::updateDataBinds(true)` for every cloned
    /// DataBind owned by this concrete ScriptedListenerAction.
    ///
    /// Converter-owned dependents settle before their outer ScriptInput bind;
    /// each occurrence then applies target/source in its own authored favor
    /// order. The returned values are only the Lua-table projections produced
    /// by Core ScriptInput value callbacks—the cloned Core targets update even
    /// when no table is attached (`data_bind_container.cpp:115-203`;
    /// `scripted_object.cpp:558-586`).
    pub(crate) fn public_update_data_binds<F>(
        &mut self,
        file: &RuntimeFile,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        apply: &mut F,
    ) -> Result<Vec<RuntimeScriptedListenerBindingUpdate>, ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        let mut updates = Vec::new();
        for input_index in 0..self.inputs.len() {
            if let Some(update) =
                self.public_update_data_bind(input_index, file, owner_instance, true, apply)?
            {
                updates.push(update);
            }
        }
        Ok(updates)
    }

    /// One selected cloned ScriptInput occurrence from the owning state
    /// machine's outer `DataBindContainer` snapshot.
    pub(crate) fn public_update_data_bind<F>(
        &mut self,
        input_index: usize,
        file: &RuntimeFile,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        apply_target_to_source: bool,
        apply: &mut F,
    ) -> Result<Option<RuntimeScriptedListenerBindingUpdate>, ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        let action_global_id = self.action_global_id;
        let Some(input) = self.inputs.get_mut(input_index) else {
            return Ok(None);
        };
        let Some(binding) = input.binding.as_mut() else {
            return Ok(None);
        };
        if !binding.is_context {
            return Ok(None);
        }

        if let Some(converter) = binding.converter.as_mut() {
            binding.converter_data_binds.public_update(
                converter,
                &mut binding.converter_state,
                file,
                owner_instance,
                // Converter-owned DataBindContainers are reached through
                // DataBind::updateDependents. C++ always invokes them with
                // `updateDataBinds(false)`, even from the owning public pass.
                false,
                apply,
            )?;
        }

        binding.retained_bind.collect_source_dirt();
        if !binding.formula_source_sink.take_dirt().is_empty()
            && let Some(converter) = binding.converter.as_ref()
        {
            binding
                .converter_state
                .reset_source_change_formula_randoms(converter);
        }
        let wants_target_to_source = apply_target_to_source
            && data_bind_flags_apply_target_to_source(binding.flags)
            && binding
                .retained_bind
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS_TARGET);
        let source_runs_first = binding.retained_bind.source_to_target_runs_first();

        if wants_target_to_source && !source_runs_first {
            Self::update_outer_source_from_target(input)?;
        }

        let source_dirt = input.binding.as_ref().is_some_and(|binding| {
            binding
                .retained_bind
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
        });
        // `DataBindContainer` snapshots the occurrence dirt before calling
        // update. A target-only public occurrence must not project the stale
        // source back onto its just-edited ScriptInput.
        let has_source_dirt = source_dirt;
        let update = has_source_dirt
            .then(|| {
                input
                    .binding
                    .as_ref()
                    .and_then(|binding| binding.retained_bind.source())
                    .and_then(|source| runtime_graph_value_from_bound_cell(&source.value()))
                    .map(|source| {
                        Self::apply_resolved_source_to_target(action_global_id, input, file, source)
                    })
                    .transpose()
            })
            .transpose()?
            .flatten()
            .flatten();
        if wants_target_to_source && source_runs_first {
            Self::update_outer_source_from_target(input)?;
        } else if !wants_target_to_source && let Some(binding) = input.binding.as_mut() {
            binding.retained_bind.take_target_dirt();
        }

        Ok(update.map(|value| RuntimeScriptedListenerBindingUpdate {
            action_global_id,
            input_name: input.properties.name().clone(),
            value,
        }))
    }

    fn update_outer_source_from_target(
        input: &mut RuntimeScriptedListenerInputBindingOccurrence,
    ) -> Result<bool, ScriptError> {
        let binding = input
            .binding
            .as_mut()
            .expect("target-to-source application requires an authored DataBind");
        let source_value = binding
            .retained_bind
            .source()
            .map(RuntimeViewModelCell::value);
        let Some(mut target) = input
            .properties
            .target_value(binding.target_property, source_value.as_ref())
        else {
            binding.retained_bind.take_target_dirt();
            return Ok(false);
        };
        if let Some(converter) = binding.converter.as_ref() {
            let converted = if binding.flags & DATA_BIND_FLAG_DIRECTION_TO_SOURCE != 0 {
                binding
                    .converter_state
                    .convert_value_with_formula_randoms_for_scripted_listener(
                        converter,
                        &target,
                        &mut binding.formula_random_source,
                    )?
            } else {
                binding
                    .converter_state
                    .reverse_convert_value_with_formula_randoms_for_scripted_listener(
                        converter,
                        &target,
                        &mut binding.formula_random_source,
                    )?
            };
            let Some(converted) = converted else {
                binding.retained_bind.take_target_dirt();
                return Ok(false);
            };
            target = converted;
        }
        let Some(value) = runtime_cell_value_from_graph_value(&target, source_value.as_ref())
        else {
            binding.retained_bind.take_target_dirt();
            return Ok(false);
        };
        Ok(binding.retained_bind.update_source_binding_value(value))
    }

    pub(crate) fn resolve_runtime_table_updates_from_data_context(
        &mut self,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
    ) -> Result<Vec<RuntimeScriptedListenerBindingUpdate>, ScriptError> {
        let input_ids = self
            .inputs
            .iter()
            .map(|input| input.input_global_id)
            .collect::<Vec<_>>();
        let mut updates = Vec::new();
        for input_global_id in input_ids {
            let value =
                match self.resolve_from_data_context(file, data_context, input_global_id, false) {
                    Ok(Some(value)) => value,
                    Ok(None) => continue,
                    Err(error) if error.resource_code().is_some() => return Err(error),
                    // C++ updates every DataBind occurrence independently.
                    // A protected scripted conversion failure keeps its own
                    // retained/default result and cannot abort later inputs.
                    Err(_) => continue,
                };
            let input_name = self
                .inputs
                .iter()
                .find(|input| input.input_global_id == input_global_id)
                .expect("input id came from this occurrence")
                .properties
                .name()
                .to_owned();
            updates.push(RuntimeScriptedListenerBindingUpdate {
                action_global_id: self.action_global_id,
                input_name,
                value,
            });
        }
        Ok(updates)
    }

    pub(crate) fn resolve_runtime_table_updates(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
    ) -> Result<Vec<RuntimeScriptedListenerBindingUpdate>, ScriptError> {
        let input_ids = self
            .inputs
            .iter()
            .map(|input| input.input_global_id)
            .collect::<Vec<_>>();
        let mut updates = Vec::new();
        for input_global_id in input_ids {
            let value = match self.resolve(file, context, input_global_id, false) {
                Ok(Some(value)) => value,
                Ok(None) => continue,
                Err(error) if error.resource_code().is_some() => return Err(error),
                Err(_) => continue,
            };
            let input_name = self
                .inputs
                .iter()
                .find(|input| input.input_global_id == input_global_id)
                .expect("input id came from this occurrence")
                .properties
                .name()
                .to_owned();
            updates.push(RuntimeScriptedListenerBindingUpdate {
                action_global_id: self.action_global_id,
                input_name,
                value,
            });
        }
        Ok(updates)
    }

    pub(crate) fn advance_stateful_converters(
        &mut self,
        elapsed_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<RuntimeDataBindGraphStatefulAdvance, ScriptError> {
        let mut aggregate = RuntimeDataBindGraphStatefulAdvance::default();
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if !binding.is_context {
                continue;
            }
            // `DataBind::advance` requires a currently resolved source.
            // An unbound interpolator must not keep the machine awake forever
            // (`data_bind.cpp:582-588`).
            if binding.retained_bind.source().is_none() {
                continue;
            }
            if let Some(converter) = binding.converter.as_ref() {
                let advance = advance_converter_in_authored_order(
                    converter,
                    &mut binding.converter_state,
                    elapsed_seconds,
                    host,
                )?;
                aggregate.changed |= advance.changed;
                aggregate.keep_going |= advance.keep_going;
                if advance.changed {
                    Self::mark_converter_dirty(binding);
                }
            }
        }
        Ok(aggregate)
    }

    /// Advance one outer cloned DataBind in the shared StateMachine container
    /// traversal. Group internals remain authored inside that occurrence.
    pub(crate) fn advance_stateful_converter(
        &mut self,
        input_index: usize,
        elapsed_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<RuntimeDataBindGraphStatefulAdvance, ScriptError> {
        let Some(input) = self.inputs.get_mut(input_index) else {
            return Ok(RuntimeDataBindGraphStatefulAdvance::default());
        };
        let Some(binding) = input.binding.as_mut().filter(|binding| binding.is_context) else {
            return Ok(RuntimeDataBindGraphStatefulAdvance::default());
        };
        if binding.retained_bind.source().is_none() {
            return Ok(RuntimeDataBindGraphStatefulAdvance::default());
        }
        let Some(converter) = binding.converter.as_ref() else {
            return Ok(RuntimeDataBindGraphStatefulAdvance::default());
        };
        let advance = advance_converter_in_authored_order(
            converter,
            &mut binding.converter_state,
            elapsed_seconds,
            host,
        )?;
        if advance.changed {
            Self::mark_converter_dirty(binding);
        }
        Ok(advance)
    }

    pub(crate) fn collect_source_dirt(&mut self) -> bool {
        let mut dirty = false;
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if !binding.is_context {
                continue;
            }
            dirty |= binding.retained_bind.collect_source_dirt();
        }
        dirty
    }
}

impl Drop for RuntimeScriptedListenerActionBindingOccurrence {
    fn drop(&mut self) {
        // `StateMachineInstance::~StateMachineInstance` deletes its cloned
        // DataBinds, and each `DataBind::~DataBind` unbinds its source and
        // converter before deleting occurrence-owned state
        // (`state_machine_instance.cpp:2193-2199`;
        // `data_bind.cpp:239-249,354-369`). Rust drops the owned values
        // automatically, but must first unregister the same retained source
        // edges and preserve each converter subclass's virtual unbind rules.
        // The owning StateMachineInstance declares this binding collection
        // before its ScriptedObject table maps so every cloned DataBind drops
        // while its table occurrence is still alive
        // (`state_machine_instance.cpp:2169-2198`).
        for input in &mut self.inputs {
            let Some(binding) = input.binding.as_mut() else {
                continue;
            };
            if binding.is_context {
                Self::unbind_source(binding);
            }
        }
    }
}

fn advance_converter_in_authored_order(
    converter: &RuntimeDataBindGraphConverter,
    state: &mut RuntimeDataBindGraphConverterState,
    elapsed_seconds: f32,
    host: &mut dyn ScriptHost,
) -> Result<RuntimeDataBindGraphStatefulAdvance, ScriptError> {
    advance_converter_in_authored_order_with_observer(
        converter,
        state,
        elapsed_seconds,
        host,
        &mut |_| {},
    )
}

fn advance_converter_in_authored_order_with_observer(
    converter: &RuntimeDataBindGraphConverter,
    state: &mut RuntimeDataBindGraphConverterState,
    elapsed_seconds: f32,
    host: &mut dyn ScriptHost,
    observe_leaf: &mut impl FnMut(&RuntimeDataBindGraphConverter),
) -> Result<RuntimeDataBindGraphStatefulAdvance, ScriptError> {
    match (converter, state) {
        (
            RuntimeDataBindGraphConverter::Scripted {
                instance: Some(instance),
                serialized_implemented_methods,
                ..
            },
            RuntimeDataBindGraphConverterState::Scripted(_),
        ) => {
            observe_leaf(converter);
            let needs_advance = match crate::scripted_data_converter::advance(
                Some(instance),
                *serialized_implemented_methods,
                elapsed_seconds,
                host,
            ) {
                Ok(needs_advance) => needs_advance,
                Err(error)
                    if error.resource_code().is_some()
                        || host.requires_atomic_script_callbacks() =>
                {
                    return Err(error);
                }
                // C++'s protected call consumes an ordinary script error and
                // the enclosing DataConverterGroup continues to its next
                // authored item.
                Err(_) => false,
            };
            Ok(RuntimeDataBindGraphStatefulAdvance {
                changed: needs_advance,
                keep_going: needs_advance,
            })
        }
        (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) if converters.len() == states.len() => {
            let mut aggregate = RuntimeDataBindGraphStatefulAdvance::default();
            for (converter, state) in converters.iter().zip(states) {
                let advance = advance_converter_in_authored_order_with_observer(
                    converter,
                    state,
                    elapsed_seconds,
                    host,
                    observe_leaf,
                )?;
                aggregate.changed |= advance.changed;
                aggregate.keep_going |= advance.keep_going;
            }
            Ok(aggregate)
        }
        (converter, state) => {
            observe_leaf(converter);
            Ok(state.advance_converter(Some(converter), elapsed_seconds))
        }
    }
}

fn collect_scripted_converter_occurrences(
    converter: &RuntimeDataBindGraphConverter,
    input_global_id: u32,
    path: &mut Vec<usize>,
    occurrences: &mut Vec<(u32, Vec<usize>, u32, bool, bool)>,
) {
    match converter {
        RuntimeDataBindGraphConverter::Scripted {
            global_id,
            serialized_implemented_methods,
            instance,
            ..
        } => {
            occurrences.push((
                input_global_id,
                path.clone(),
                *global_id,
                crate::scripted_data_converter::inits(*serialized_implemented_methods),
                instance.is_some(),
            ));
        }
        RuntimeDataBindGraphConverter::Group(converters) => {
            for (index, converter) in converters.iter().enumerate() {
                path.push(index);
                collect_scripted_converter_occurrences(
                    converter,
                    input_global_id,
                    path,
                    occurrences,
                );
                path.pop();
            }
        }
        _ => {}
    }
}

fn collect_scripted_converter_bind_steps(
    converter: &RuntimeDataBindGraphConverter,
    action_global_id: u32,
    listener_input_global_id: u32,
    steps: &mut Vec<RuntimeScriptedListenerDataConverterBindStep>,
) {
    steps.extend(
        runtime_data_converter_bind_steps(converter)
            .into_iter()
            .map(|step| match step {
                RuntimeDataConverterBindStep::BindOwn { path } => {
                    RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                        action_global_id,
                        listener_input_global_id,
                        converter_path: path,
                    }
                }
                RuntimeDataConverterBindStep::Rehydrate {
                    path,
                    converter_global_id,
                    inits,
                } => RuntimeScriptedListenerDataConverterBindStep::Rehydrate {
                    action_global_id,
                    listener_input_global_id,
                    converter_path: path,
                    converter_global_id,
                    inits,
                },
                RuntimeDataConverterBindStep::RebindFinalInput {
                    path,
                    input_index,
                    data_bind_index,
                } => RuntimeScriptedListenerDataConverterBindStep::RebindFinalInput {
                    action_global_id,
                    listener_input_global_id,
                    converter_path: path,
                    converter_input_index: input_index,
                    data_bind_index,
                },
            }),
    );
}

fn runtime_scripted_listener_bound_value(
    kind: ScriptListenerInputKind,
    value: RuntimeDataBindGraphValue,
) -> Result<RuntimeScriptedListenerBoundValue, ScriptError> {
    let value = match (kind, value) {
        (ScriptListenerInputKind::Boolean, RuntimeDataBindGraphValue::Boolean(value)) => {
            RuntimeScriptedListenerBoundValue::Value(ScriptValue::Bool(value))
        }
        (ScriptListenerInputKind::Number, RuntimeDataBindGraphValue::Number(value)) => {
            RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(f64::from(value)))
        }
        (ScriptListenerInputKind::Color, RuntimeDataBindGraphValue::Color(value)) => {
            RuntimeScriptedListenerBoundValue::Value(ScriptValue::Color(value))
        }
        (ScriptListenerInputKind::String, RuntimeDataBindGraphValue::String(value)) => {
            RuntimeScriptedListenerBoundValue::Value(ScriptValue::CoreString(
                ScriptCoreString::from_bytes(value),
            ))
        }
        (ScriptListenerInputKind::Trigger, RuntimeDataBindGraphValue::Trigger(value)) => {
            RuntimeScriptedListenerBoundValue::Trigger(value)
        }
        (ScriptListenerInputKind::Artboard, RuntimeDataBindGraphValue::Artboard(value)) => {
            RuntimeScriptedListenerBoundValue::Artboard(value)
        }
        (kind, value) => {
            return Err(ScriptError::new(format!(
                "script listener input kind {kind:?} received incompatible bound value {value:?}",
            )));
        }
    };
    Ok(value)
}

/// Execute one occurrence of pinned C++ `ScriptedListenerAction`.
///
/// `performStateful` prefers `performAction`, falls back to `perform`, pops an
/// ordinary protected-call failure, and lets the owning listener continue its
/// authored FIFO. Rust's typed resource-limit error is the explicit safety
/// fence and remains terminal.
pub(crate) fn perform_scripted_listener_action(
    instances: &BTreeMap<u32, RuntimeScriptInstanceHandle>,
    definition: &ScriptListenerActionDefinition,
    invocation: &ScriptListenerInvocation,
    host: &mut dyn ScriptHost,
) -> Result<bool, ScriptError> {
    let Some(instance) = instances.get(&definition.action_global_id()).cloned() else {
        // Visual-only imports deliberately leave script tables unattached.
        // Their ordinary listener actions keep working while embedded
        // bytecode remains inert.
        return Ok(false);
    };
    let result: Result<bool, ScriptError> = (|| {
        let mut instance = instance.borrow_mut();
        // A hydration-prerequisite miss leaves C++ `m_self` alive and
        // dispatchable even though init remains pending. Only init
        // false/error/missing-requested-data disposes that lifetime. Gate on
        // the table/context lifetime itself, never on the init-pending bit
        // (`scripted_object.cpp:277-303,399-435`;
        // `scripted_listener_action.cpp:107-118`).
        if !instance.script_lifetime_valid() {
            return Ok(false);
        }
        instance.call_preferred_listener_action(invocation, host)
    })();
    match result {
        Ok(changed) => Ok(changed),
        Err(error)
            if error.resource_code().is_some() || host.requires_atomic_script_callbacks() =>
        {
            Err(error)
        }
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_model_cell::RuntimeViewModelCellValue;
    use crate::{
        NoopScriptHost, ScriptDataConverterMethod, ScriptInstance, ScriptListenerActionMethod,
        ScriptMethod,
    };
    use nuxie_binary::read_runtime_file;
    use nuxie_schema::definition_by_name;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
        sync::Arc,
    };

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn type_key(type_name: &str) -> u16 {
        definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int
    }

    fn property_key(type_name: &str, property_name: &str) -> u16 {
        let definition = definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                definition_by_name(ancestor)
                    .unwrap_or_else(|| panic!("missing ancestor schema definition {ancestor}"))
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
            .key
            .int
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
        push_var_uint(bytes, u64::from(type_key(type_name)));
        properties(bytes);
        push_var_uint(bytes, 0);
    }

    fn push_uint(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: u64) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value);
    }

    fn push_f32(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: f32) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn push_blob(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &[u8]) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value);
    }

    fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
        push_blob(bytes, type_name, name, value.as_bytes());
    }

    fn number_context(initial: f32) -> (RuntimeFile, RuntimeOwnedViewModelInstance) {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9_520);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "ViewModel", |bytes| {
            push_string(bytes, "ViewModel", "name", "Root");
        });
        push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
            push_string(bytes, "ViewModelPropertyNumber", "name", "source");
        });
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ViewModelInstance", |bytes| {
            push_string(bytes, "ViewModelInstance", "name", "root-default");
            push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", initial);
        });
        let file = read_runtime_file(&bytes).expect("import number context");
        let context =
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("instantiate number context");
        (file, context)
    }

    fn scripted_listener_with_plain_bind_shadow() -> RuntimeFile {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9_521);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "ViewModel", |bytes| {
            push_string(bytes, "ViewModel", "name", "Root");
        });
        push_object(&mut bytes, "ViewModelPropertyNumber", |bytes| {
            push_string(bytes, "ViewModelPropertyNumber", "name", "source");
        });
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ViewModelInstance", |bytes| {
            push_string(bytes, "ViewModelInstance", "name", "root-default");
            push_uint(bytes, "ViewModelInstance", "viewModelId", 0);
        });
        push_object(&mut bytes, "ViewModelInstanceNumber", |bytes| {
            push_uint(bytes, "ViewModelInstanceNumber", "viewModelPropertyId", 0);
            push_f32(bytes, "ViewModelInstanceNumber", "propertyValue", 3.0);
        });
        push_object(&mut bytes, "Artboard", |bytes| {
            push_f32(bytes, "Artboard", "width", 100.0);
            push_f32(bytes, "Artboard", "height", 100.0);
            push_uint(bytes, "Artboard", "viewModelId", 0);
        });
        push_object(&mut bytes, "Shape", |bytes| {
            push_uint(bytes, "Node", "parentId", 0);
        });
        push_object(&mut bytes, "StateMachine", |_| {});
        push_object(&mut bytes, "StateMachineListenerSingle", |bytes| {
            push_uint(bytes, "StateMachineListener", "targetId", 1);
            push_uint(bytes, "StateMachineListenerSingle", "listenerTypeValue", 2);
        });
        push_object(&mut bytes, "ScriptedListenerAction", |_| {});
        push_object(&mut bytes, "ScriptInputNumber", |bytes| {
            push_string(bytes, "ScriptInputNumber", "name", "amount");
            push_f32(bytes, "ScriptInputNumber", "propertyValue", 7.0);
        });
        let mut source_path = Vec::new();
        push_var_uint(&mut source_path, 0);
        push_var_uint(&mut source_path, 0);
        push_object(&mut bytes, "DataBindContext", |bytes| {
            push_uint(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(property_key("ScriptInputNumber", "propertyValue")),
            );
            push_blob(bytes, "DataBindContext", "sourcePathIds", &source_path);
        });
        push_object(&mut bytes, "DataBind", |bytes| {
            push_uint(bytes, "DataBind", "flags", DATA_BIND_ALL_AUTHORED_FLAGS);
        });
        read_runtime_file(&bytes).expect("import scripted listener shadow fixture")
    }

    #[derive(Clone, Copy)]
    enum ScriptAssetFixture {
        Missing,
        WrongType,
        Module,
        ProtocolDefault,
        ProtocolMask(u32),
    }

    fn scripted_listener_asset_fixture(asset: ScriptAssetFixture) -> RuntimeFile {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9_522);
        push_var_uint(&mut bytes, 0);
        push_object(&mut bytes, "Backboard", |_| {});
        match asset {
            ScriptAssetFixture::Missing => {}
            ScriptAssetFixture::WrongType => {
                push_object(&mut bytes, "ImageAsset", |bytes| {
                    push_uint(bytes, "FileAsset", "assetId", 0);
                });
            }
            ScriptAssetFixture::Module
            | ScriptAssetFixture::ProtocolDefault
            | ScriptAssetFixture::ProtocolMask(_) => {
                push_object(&mut bytes, "ScriptAsset", |bytes| {
                    push_uint(bytes, "FileAsset", "assetId", 0);
                    if matches!(asset, ScriptAssetFixture::Module) {
                        push_uint(bytes, "ScriptAsset", "isModule", 1);
                    }
                    if let ScriptAssetFixture::ProtocolMask(mask) = asset {
                        push_uint(
                            bytes,
                            "ScriptAsset",
                            "serializedImplementedMethods",
                            u64::from(mask),
                        );
                    }
                });
            }
        }
        push_object(&mut bytes, "Artboard", |bytes| {
            push_f32(bytes, "Artboard", "width", 100.0);
            push_f32(bytes, "Artboard", "height", 100.0);
        });
        push_object(&mut bytes, "StateMachine", |_| {});
        push_object(&mut bytes, "StateMachineListenerSingle", |_| {});
        push_object(&mut bytes, "ScriptedListenerAction", |bytes| {
            push_uint(bytes, "ScriptedListenerAction", "scriptAssetId", 0);
        });
        read_runtime_file(&bytes).expect("import scripted-listener asset fixture")
    }

    #[test]
    fn only_an_executable_protocol_asset_copies_optional_method_bits() {
        const EXPLICIT_MASK: u32 = 1 << 14;
        for (asset, expected_protocol, expected_mask) in [
            (ScriptAssetFixture::Missing, false, 0),
            (ScriptAssetFixture::WrongType, false, 0),
            (ScriptAssetFixture::Module, false, 0),
            (
                ScriptAssetFixture::ProtocolDefault,
                true,
                crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
            ),
            (
                ScriptAssetFixture::ProtocolMask(EXPLICIT_MASK),
                true,
                EXPLICIT_MASK,
            ),
        ] {
            let file = scripted_listener_asset_fixture(asset);
            let action = (0..file.object_count())
                .filter_map(|id| file.object(id))
                .find(|object| object.type_name == "ScriptedListenerAction")
                .expect("fixture scripted listener");
            let definition = runtime_scripted_listener_action_definition(&file, action, &[])
                .expect("retained scripted-listener definition");
            assert_eq!(definition.has_protocol_asset(), expected_protocol);
            assert_eq!(
                definition.serialized_implemented_methods(),
                expected_mask,
                "OptionalScriptedMethods remains zero until a valid protocol ScriptAsset copies its serialized bitfield (`script_asset.hpp:97-108`; `script_asset.cpp:139-161`)"
            );
        }
    }

    #[derive(Debug)]
    struct ListenerLifetimeProbe {
        lifetime_valid: bool,
        init_pending: bool,
        calls: Rc<Cell<usize>>,
    }

    impl ScriptInstance for ListenerLifetimeProbe {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::PerformAction)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn call_listener_action(
            &mut self,
            _method: ScriptListenerActionMethod,
            _invocation: &ScriptListenerInvocation,
            _host: &mut dyn ScriptHost,
        ) -> Result<(), ScriptError> {
            self.calls.set(self.calls.get() + 1);
            Ok(())
        }

        fn call_preferred_listener_action(
            &mut self,
            _invocation: &ScriptListenerInvocation,
            _host: &mut dyn ScriptHost,
        ) -> Result<bool, ScriptError> {
            self.calls.set(self.calls.get() + 1);
            Ok(true)
        }

        fn user_init_pending(&mut self) -> Result<bool, ScriptError> {
            Ok(self.init_pending)
        }

        fn script_lifetime_valid(&self) -> bool {
            self.lifetime_valid
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn listener_dispatch_gates_on_cpp_table_lifetime_not_pending_init() {
        let definition = ScriptListenerActionDefinition::new(7, 0, "listener".to_owned());
        for (label, lifetime_valid, init_pending, expected_calls) in [
            ("hydration-prerequisite-pending", true, true, 1),
            ("init-failed-lifetime", false, true, 0),
        ] {
            let calls = Rc::new(Cell::new(0));
            let mut instances = BTreeMap::new();
            instances.insert(
                definition.action_global_id(),
                RuntimeScriptInstanceHandle::new(Box::new(ListenerLifetimeProbe {
                    lifetime_valid,
                    init_pending,
                    calls: Rc::clone(&calls),
                })),
            );
            assert_eq!(
                perform_scripted_listener_action(
                    &instances,
                    &definition,
                    &ScriptListenerInvocation::None,
                    &mut NoopScriptHost,
                )
                .expect(label),
                lifetime_valid,
                "{label}"
            );
            assert_eq!(calls.get(), expected_calls, "{label}");
        }
    }

    #[test]
    fn later_plain_bind_replaces_context_bind_but_remains_occurrence_owned_and_inert() {
        let file = scripted_listener_with_plain_bind_shadow();
        let action = (0..file.object_count())
            .filter_map(|id| file.object(id))
            .find(|object| object.type_name == "ScriptedListenerAction")
            .expect("fixture scripted listener");
        let input = (0..file.object_count())
            .filter_map(|id| file.object(id))
            .find(|object| object.type_name == "ScriptInputNumber")
            .expect("fixture scripted input");
        let definition =
            runtime_scripted_listener_action_binding_definition(&file, action, &[input])
                .expect("scripted listener binding definition");
        let definition_binding = definition.inputs[0]
            .binding
            .as_ref()
            .expect("last plain bind remains owned");
        assert!(!definition_binding.is_context);
        assert_eq!(
            definition_binding.flags, DATA_BIND_ALL_AUTHORED_FLAGS,
            "Direction, TwoWay, Once, favored-order, and NameBased bits survive import exactly",
        );
        assert_eq!(
            definition.inputs[0].properties.value(),
            Some(&RuntimeDataBindGraphValue::Number(7.0)),
            "the cloned DataBind target keeps the imported ScriptInput Core value",
        );

        let mut occurrence = definition.instantiate();
        let occurrence_binding = occurrence.inputs[0]
            .binding
            .as_ref()
            .expect("plain bind is cloned into the occurrence");
        assert!(!occurrence_binding.is_context);
        assert_eq!(occurrence_binding.flags, DATA_BIND_ALL_AUTHORED_FLAGS,);
        let cold = occurrence.fresh_clone();
        let cold_binding = cold.inputs[0]
            .binding
            .as_ref()
            .expect("cold clone retains the inert DataBind occurrence");
        assert_eq!(cold_binding.flags, DATA_BIND_ALL_AUTHORED_FLAGS);
        assert_eq!(
            cold.inputs[0].properties.value(),
            Some(&RuntimeDataBindGraphValue::Number(7.0)),
            "every fresh occurrence clones the target's current Core value",
        );

        let context =
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("fixture default context");
        occurrence.bind_sources(&file, &context, true);
        assert_eq!(
            occurrence.resolve(&file, &context, input.id, true).unwrap(),
            None,
            "DataBindContainer binds only DataBindContext; the later plain occurrence stays inert"
        );
    }

    #[test]
    fn scripted_string_core_state_preserves_raw_bytes_and_embedded_nul() {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, 9_529);
        push_var_uint(&mut bytes, 0);
        let authored_name = [0xff, b'k', 0, b't'];
        let authored_value = [0xfe, b'v', 0, b'x'];
        push_object(&mut bytes, "ScriptInputString", |bytes| {
            push_blob(bytes, "ScriptInputString", "name", &authored_name);
            push_blob(bytes, "ScriptInputString", "propertyValue", &authored_value);
        });
        let file = read_runtime_file(&bytes).expect("import raw CoreString fixture");
        let input = file.object(0).expect("ScriptInputString");
        let mut properties = RuntimeScriptInputProperties::from_object(
            &file,
            input,
            ScriptListenerInputKind::String,
        );
        assert_eq!(properties.name().as_bytes(), authored_name);
        assert_eq!(
            properties.value(),
            Some(&RuntimeDataBindGraphValue::String(authored_value.to_vec()))
        );

        let rebound_name = [0xfd, b'n', 0, b'z'];
        assert_eq!(
            properties.apply_target(
                &file,
                ScriptListenerInputKind::String,
                RuntimeScriptInputTargetProperty::Name,
                RuntimeDataBindGraphValue::String(rebound_name.to_vec()),
            ),
            RuntimeScriptInputTargetApply::ChangedWithoutTableProjection,
        );
        assert_eq!(properties.name().as_bytes(), rebound_name);
        assert_eq!(properties.name().as_c_str_bytes(), &rebound_name[..2]);

        let binding = occurrence(vec![RuntimeScriptedListenerInputBindingOccurrence {
            input_global_id: input.id,
            kind: ScriptListenerInputKind::String,
            properties,
            binding: None,
        }]);
        let snapshot = binding
            .input_snapshots()
            .into_iter()
            .next()
            .expect("one raw string snapshot");
        assert_eq!(snapshot.name.as_bytes(), rebound_name);
        assert_eq!(
            snapshot.value,
            Some(ScriptListenerInputSnapshotValue::Value(
                ScriptValue::CoreString(ScriptCoreString::from_bytes(authored_value.to_vec()))
            )),
            "the cloned Core target keeps the full byte suffix; Lua truncation happens later"
        );
    }

    fn number_input(
        input_global_id: u32,
        converter: RuntimeDataBindGraphConverter,
    ) -> RuntimeScriptedListenerInputBindingOccurrence {
        let converter_state = RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
        let converter_data_binds =
            RuntimeDataConverterDataBindDefinition::for_converter_shape(&converter).instantiate();
        let mut input = RuntimeScriptedListenerInputBindingOccurrence {
            input_global_id,
            kind: ScriptListenerInputKind::Number,
            properties: RuntimeScriptInputProperties::for_test(
                format!("value{input_global_id}"),
                u32::MAX,
                Some(RuntimeDataBindGraphValue::Number(0.0)),
            ),
            binding: Some(RuntimeScriptedListenerInputDataBindOccurrence {
                is_context: true,
                source_path: vec![0, 0],
                name_based: false,
                property_key: crate::properties::property_key_for_name(
                    "ScriptInputNumber",
                    "propertyValue",
                )
                .map(u32::from)
                .expect("number property key"),
                flags: 0,
                target_property: RuntimeScriptInputTargetProperty::Value,
                retained_bind: RuntimeRetainedDataBind::new(0, false),
                converter: Some(converter),
                converter_state,
                converter_data_binds,
                formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
                formula_source: None,
                formula_source_sink: RuntimeCellDirtSink::new(),
                unresolved_converter: false,
                last_source: None,
                last_target: None,
            }),
        };
        input
            .binding
            .as_mut()
            .expect("number input binding")
            .attach_converter_parent();
        input
    }

    fn typed_input(
        input_global_id: u32,
        kind: ScriptListenerInputKind,
        type_name: &str,
        property_name: &str,
        initial: RuntimeDataBindGraphValue,
    ) -> RuntimeScriptedListenerInputBindingOccurrence {
        let properties = match initial {
            RuntimeDataBindGraphValue::Artboard(artboard_id) => {
                RuntimeScriptInputProperties::for_test_artboard(
                    format!("value{input_global_id}"),
                    u32::MAX,
                    artboard_id,
                    (artboard_id != u64::from(u32::MAX)).then_some(artboard_id),
                    true,
                )
            }
            initial => RuntimeScriptInputProperties::for_test(
                format!("value{input_global_id}"),
                u32::MAX,
                Some(initial),
            ),
        };
        RuntimeScriptedListenerInputBindingOccurrence {
            input_global_id,
            kind,
            properties,
            binding: Some(RuntimeScriptedListenerInputDataBindOccurrence {
                is_context: true,
                source_path: vec![0, 0],
                name_based: false,
                property_key: crate::properties::property_key_for_name(type_name, property_name)
                    .map(u32::from)
                    .unwrap_or_else(|| panic!("missing {type_name}.{property_name} key")),
                flags: 0,
                target_property: RuntimeScriptInputTargetProperty::Value,
                retained_bind: RuntimeRetainedDataBind::new(0, false),
                converter: None,
                converter_state: RuntimeDataBindGraphConverterState::for_converter(None),
                converter_data_binds: RuntimeDataConverterDataBindState::default(),
                formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
                formula_source: None,
                formula_source_sink: RuntimeCellDirtSink::new(),
                unresolved_converter: false,
                last_source: None,
                last_target: None,
            }),
        }
    }

    fn bind_direct_cell(
        occurrence: &mut RuntimeScriptedListenerActionBindingOccurrence,
        input_global_id: u32,
        source: &RuntimeViewModelCell,
    ) {
        let binding = occurrence
            .inputs
            .iter_mut()
            .find(|input| input.input_global_id == input_global_id)
            .and_then(|input| input.binding.as_mut())
            .expect("typed input binding");
        RuntimeScriptedListenerActionBindingOccurrence::bind_resolved_source(
            binding,
            Some(source.clone()),
            true,
        );
    }

    fn occurrence(
        inputs: Vec<RuntimeScriptedListenerInputBindingOccurrence>,
    ) -> RuntimeScriptedListenerActionBindingOccurrence {
        RuntimeScriptedListenerActionBindingOccurrence {
            action_global_id: 42,
            inputs,
        }
    }

    fn bind(
        occurrence: &mut RuntimeScriptedListenerActionBindingOccurrence,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
    ) {
        occurrence.bind_sources(file, context, true);
    }

    fn number(update: Option<RuntimeScriptedListenerBoundValue>) -> Option<f64> {
        match update {
            Some(RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(value))) => {
                Some(value)
            }
            None => None,
            value => panic!("expected number update, got {value:?}"),
        }
    }

    #[test]
    fn typed_script_inputs_apply_live_cpp_core_values_and_keep_occurrences_isolated() {
        let file = scripted_listener_with_plain_bind_shadow();
        let context =
            RuntimeOwnedViewModelInstance::new(&file, 0).expect("fixture default context");
        let template = occurrence(vec![
            typed_input(
                20,
                ScriptListenerInputKind::Boolean,
                "ScriptInputBoolean",
                "propertyValue",
                RuntimeDataBindGraphValue::Boolean(false),
            ),
            typed_input(
                21,
                ScriptListenerInputKind::Number,
                "ScriptInputNumber",
                "propertyValue",
                RuntimeDataBindGraphValue::Number(0.0),
            ),
            typed_input(
                22,
                ScriptListenerInputKind::Color,
                "ScriptInputColor",
                "propertyValue",
                RuntimeDataBindGraphValue::Color(0),
            ),
            typed_input(
                23,
                ScriptListenerInputKind::String,
                "ScriptInputString",
                "propertyValue",
                RuntimeDataBindGraphValue::String(Vec::new()),
            ),
            typed_input(
                24,
                ScriptListenerInputKind::Trigger,
                "ScriptInputTrigger",
                "propertyValue",
                RuntimeDataBindGraphValue::Trigger(0),
            ),
            typed_input(
                25,
                ScriptListenerInputKind::Artboard,
                "ScriptInputArtboard",
                "artboardId",
                RuntimeDataBindGraphValue::Artboard(0),
            ),
        ]);
        let mut live = template.fresh_clone();
        let cold = template.fresh_clone();
        let boolean = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Boolean(true));
        let number =
            RuntimeViewModelCell::new(RuntimeViewModelCellValue::SymbolListIndex(16_777_217));
        let color = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Color(0x1122_3344));
        let string =
            RuntimeViewModelCell::new(RuntimeViewModelCellValue::String(Arc::from(&b"ready"[..])));
        let trigger = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(0));
        let artboard = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Artboard(0));

        for (input_global_id, source) in [
            (20, &boolean),
            (21, &number),
            (22, &color),
            (23, &string),
            (24, &trigger),
            (25, &artboard),
        ] {
            bind_direct_cell(&mut live, input_global_id, source);
        }

        assert_eq!(
            live.resolve(&file, &context, 20, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Value(ScriptValue::Bool(
                true
            ))),
        );
        assert_eq!(
            live.resolve(&file, &context, 21, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Value(
                ScriptValue::Number(f64::from(16_777_217_u32 as f32))
            )),
            "ContextValueSymbolListIndex writes the generated CoreDouble field before projection",
        );
        assert_eq!(
            live.resolve(&file, &context, 22, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Value(
                ScriptValue::Color(0x1122_3344)
            )),
        );
        assert_eq!(
            live.resolve(&file, &context, 23, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Value(
                ScriptValue::CoreString(ScriptCoreString::from("ready"))
            )),
        );
        assert_eq!(
            live.resolve(&file, &context, 24, false).unwrap(),
            None,
            "ScriptInputTrigger does not call the script for its zero value",
        );
        assert_eq!(
            live.resolve(&file, &context, 25, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Artboard(0)),
        );

        assert!(trigger.set_value(RuntimeViewModelCellValue::Trigger(1)));
        assert_eq!(
            live.resolve(&file, &context, 24, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Trigger(1)),
        );
        assert!(trigger.set_value(RuntimeViewModelCellValue::Trigger(2)));
        assert_eq!(
            live.resolve(&file, &context, 24, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Trigger(2)),
            "every nonzero generated-property change fires the scripted trigger callback",
        );

        assert!(artboard.set_value(RuntimeViewModelCellValue::Artboard(99)));
        assert_eq!(
            live.resolve(&file, &context, 25, false).unwrap(),
            None,
            "a missing live artboard leaves the last resolved referencer untouched",
        );
        assert_eq!(
            live.inputs
                .iter()
                .find(|input| input.input_global_id == 25)
                .and_then(|input| input.properties.value()),
            Some(&RuntimeDataBindGraphValue::Artboard(0)),
        );
        assert!(artboard.set_value(RuntimeViewModelCellValue::Artboard(0)));
        assert_eq!(
            live.resolve(&file, &context, 25, false).unwrap(),
            Some(RuntimeScriptedListenerBoundValue::Artboard(0)),
            "a successful update projects fresh artboard userdata even when the numeric id repeats",
        );

        assert!(
            cold.inputs.iter().all(|input| input
                .binding
                .as_ref()
                .is_some_and(|binding| binding.retained_bind.source().is_none())),
            "the second cold occurrence owns no source identity from the live occurrence",
        );
        assert_eq!(
            cold.input_snapshots()
                .into_iter()
                .map(|snapshot| snapshot.value)
                .collect::<Vec<_>>(),
            vec![
                Some(ScriptListenerInputSnapshotValue::Value(ScriptValue::Bool(
                    false
                ))),
                Some(ScriptListenerInputSnapshotValue::Value(
                    ScriptValue::Number(0.0)
                )),
                Some(ScriptListenerInputSnapshotValue::Value(ScriptValue::Color(
                    0
                ))),
                Some(ScriptListenerInputSnapshotValue::Value(
                    ScriptValue::CoreString(ScriptCoreString::default())
                )),
                None,
                Some(ScriptListenerInputSnapshotValue::Artboard(0)),
            ],
        );
    }

    #[test]
    fn occurrence_drop_unbinds_outer_and_scripted_converter_custom_sources() {
        let custom_input =
            crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
                input_global_id: 50,
                kind: ScriptListenerInputKind::Number,
                properties: RuntimeScriptInputProperties::for_test(
                    "custom",
                    u32::MAX,
                    Some(RuntimeDataBindGraphValue::Number(1.0)),
                ),
                data_binds: vec![
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                        authored_order: 20,
                        source_path: Some(vec![0, 0]),
                        name_based: false,
                        property_key: crate::properties::property_key_for_name(
                            "ScriptInputNumber",
                            "propertyValue",
                        )
                        .map(u32::from)
                        .expect("number property key"),
                        target_property: RuntimeScriptInputTargetProperty::Value,
                        flags: 0,
                        converter_id: u32::MAX,
                    },
                ],
            };
        let mut live = occurrence(vec![number_input(
            10,
            scripted_converter(7, vec![custom_input]),
        )]);
        let outer_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(3.0));
        let custom_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(4.0));
        bind_direct_cell(&mut live, 10, &outer_source);
        let binding = live.inputs[0]
            .binding
            .as_mut()
            .expect("outer listener bind");
        let RuntimeDataBindGraphConverterState::Scripted(converter_state) =
            &mut binding.converter_state
        else {
            panic!("scripted converter occurrence");
        };
        assert!(converter_state.bind_test_input_source(0, 0, custom_source.clone(),));
        assert_eq!(outer_source.dependent_count(), 1);
        assert_eq!(custom_source.dependent_count(), 1);

        drop(live);

        assert_eq!(
            outer_source.dependent_count(),
            0,
            "StateMachineInstance destruction deletes the outer cloned DataBind and unregisters its exact source (`state_machine_instance.cpp:2193-2199`; `data_bind.cpp:239-249,354-369`)"
        );
        assert_eq!(
            custom_source.dependent_count(),
            0,
            "ScriptedDataConverter inherits DataConverter::unbind and releases every occurrence-owned custom-input source before destruction (`data_converter.cpp:32`; `scripted_data_converter.cpp:235-280`)"
        );
    }

    #[test]
    fn explicit_unresolved_outer_rebind_still_unbinds_converter_owned_sources() {
        let custom_input =
            crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
                input_global_id: 50,
                kind: ScriptListenerInputKind::Number,
                properties: RuntimeScriptInputProperties::for_test(
                    "custom",
                    u32::MAX,
                    Some(RuntimeDataBindGraphValue::Number(1.0)),
                ),
                data_binds: vec![
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                        authored_order: 20,
                        source_path: Some(vec![0, 0]),
                        name_based: false,
                        property_key: crate::properties::property_key_for_name(
                            "ScriptInputNumber",
                            "propertyValue",
                        )
                        .map(u32::from)
                        .expect("number property key"),
                        target_property: RuntimeScriptInputTargetProperty::Value,
                        flags: 0,
                        converter_id: u32::MAX,
                    },
                ],
            };
        let mut occurrence = occurrence(vec![number_input(
            10,
            scripted_converter(7, vec![custom_input]),
        )]);
        let converter_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(4.0));
        let binding = occurrence.inputs[0]
            .binding
            .as_mut()
            .expect("outer listener bind");
        assert!(
            binding.retained_bind.source().is_none(),
            "the outer source begins unresolved"
        );
        let RuntimeDataBindGraphConverterState::Scripted(converter_state) =
            &mut binding.converter_state
        else {
            panic!("scripted converter occurrence");
        };
        assert!(converter_state.bind_test_input_source(0, 0, converter_source.clone()));
        assert_eq!(converter_source.dependent_count(), 1);

        RuntimeScriptedListenerActionBindingOccurrence::bind_resolved_source(binding, None, true);

        assert_eq!(
            converter_source.dependent_count(),
            0,
            "DataBindContext::bindFromContext takes DataBind::unbind even for null -> null, recursively releasing converter-owned sources before rebind (`data_bind_context.cpp:56-89`; `data_bind.cpp:354-369`)"
        );
    }

    #[test]
    fn interpolator_updates_on_the_next_bind_pass_and_is_occurrence_local() {
        let (file, mut context) = number_context(0.0);
        let converter = RuntimeDataBindGraphConverter::Interpolator {
            global_id: 7,
            duration: 1.0,
            interpolator: None,
        };
        let template = occurrence(vec![number_input(10, converter)]);
        let mut first = template.fresh_clone();
        let mut second = template.fresh_clone();
        let mut host = NoopScriptHost;
        bind(&mut first, &file, &context);
        bind(&mut second, &file, &context);

        assert_eq!(
            number(first.resolve(&file, &context, 10, true).unwrap()),
            None,
            "binding an unchanged cloned target does not invoke its generated changed callback"
        );
        assert_eq!(
            number(second.resolve(&file, &context, 10, true).unwrap()),
            None,
            "table hydration is a separate ScriptedObject phase before updateDataBinds(false)"
        );
        for binding in [&mut first, &mut second] {
            binding.advance_stateful_converters(0.1, &mut host).unwrap();
            binding.advance_stateful_converters(0.1, &mut host).unwrap();
        }

        assert!(context.set_number_by_property_name("source", 10.0));
        assert_eq!(
            number(first.resolve(&file, &context, 10, false).unwrap()),
            None
        );
        let advance = first.advance_stateful_converters(0.5, &mut host).unwrap();
        assert!(advance.changed);
        assert_eq!(
            number(first.resolve(&file, &context, 10, false).unwrap()),
            Some(5.0)
        );
        assert_eq!(
            number(second.resolve(&file, &context, 10, false).unwrap()),
            None,
            "the second occurrence retains its own untouched converter state"
        );
    }

    #[test]
    fn source_cell_rebind_resets_interpolator_state_even_when_the_value_matches() {
        let (file, mut first_context) = number_context(0.0);
        let (_, mut second_context) = number_context(0.0);
        let converter = RuntimeDataBindGraphConverter::Interpolator {
            global_id: 7,
            duration: 1.0,
            interpolator: None,
        };
        let mut binding = occurrence(vec![number_input(10, converter)]);
        let mut host = NoopScriptHost;

        bind(&mut binding, &file, &first_context);
        binding.resolve(&file, &first_context, 10, true).unwrap();
        binding.advance_stateful_converters(0.1, &mut host).unwrap();
        binding.advance_stateful_converters(0.1, &mut host).unwrap();
        assert!(first_context.set_number_by_property_name("source", 10.0));
        binding.resolve(&file, &first_context, 10, false).unwrap();
        binding.advance_stateful_converters(0.5, &mut host).unwrap();
        assert_eq!(
            number(binding.resolve(&file, &first_context, 10, false).unwrap()),
            Some(5.0)
        );

        assert!(second_context.set_number_by_property_name("source", 10.0));
        bind(&mut binding, &file, &second_context);
        assert_eq!(
            number(binding.resolve(&file, &second_context, 10, false).unwrap()),
            Some(10.0),
            "a different retained cell rebinds and resets the cloned converter"
        );
    }

    #[test]
    fn formula_source_change_cache_survives_rebind_and_clears_only_on_live_notification() {
        let (file, first_context) = number_context(0.0);
        let (_, mut second_context) = number_context(0.0);
        let converter = RuntimeDataBindGraphConverter::Formula {
            tokens: vec![
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Function {
                    function_type: 16,
                    arguments_count: 0,
                    random_mode: 2,
                },
            ],
        };
        let mut occurrence = occurrence(vec![number_input(10, converter)]);
        {
            let binding = occurrence.inputs[0]
                .binding
                .as_mut()
                .expect("formula binding");
            binding.flags = DATA_BIND_FLAG_ONCE;
            binding.retained_bind = RuntimeRetainedDataBind::new(DATA_BIND_FLAG_ONCE, true);
            binding.formula_random_source.set_values(&[0.25, 0.75]);
        }

        bind(&mut occurrence, &file, &first_context);
        assert_eq!(
            number(
                occurrence
                    .resolve(&file, &first_context, 10, false)
                    .unwrap()
            ),
            Some(0.25)
        );
        assert_eq!(
            occurrence.inputs[0]
                .binding
                .as_ref()
                .expect("formula binding")
                .formula_random_source
                .call_count(),
            1
        );

        // A fresh retained source object carrying the same value is a DataBind
        // rebind, not a source-change notification. Formula has no reset()
        // override, so the cached sourceChange random survives.
        bind(&mut occurrence, &file, &second_context);
        assert_eq!(
            occurrence
                .resolve(&file, &second_context, 10, false)
                .unwrap(),
            None,
            "the cached 0.25 result leaves the already-0.25 target unchanged"
        );
        assert_eq!(
            occurrence.inputs[0]
                .binding
                .as_ref()
                .expect("formula binding")
                .formula_random_source
                .call_count(),
            1,
            "DataConverterFormula inherits no reset and preserves m_randoms across an equal-value DataContext rebind"
        );

        assert!(second_context.set_number_by_property_name("source", 1.0));
        assert_eq!(
            occurrence
                .resolve(&file, &second_context, 10, false)
                .unwrap(),
            None,
            "Once does not subscribe the outer DataBind, but Formula still observes the primary source and clears sourceChange state"
        );
        {
            let binding = occurrence.inputs[0]
                .binding
                .as_mut()
                .expect("formula binding");
            binding.retained_bind.mark_source_changed();
        }
        assert_eq!(
            number(
                occurrence
                    .resolve(&file, &second_context, 10, false)
                    .unwrap()
            ),
            Some(0.75)
        );
        assert_eq!(
            occurrence.inputs[0]
                .binding
                .as_ref()
                .expect("formula binding")
                .formula_random_source
                .call_count(),
            2,
            "only the live source notification clears Formula's sourceChange cache (`data_converter_formula.cpp:526-553`)"
        );
    }

    #[test]
    fn unbound_interpolator_does_not_keep_the_state_machine_awake() {
        let converter = RuntimeDataBindGraphConverter::Interpolator {
            global_id: 7,
            duration: 1.0,
            interpolator: None,
        };
        let mut binding = occurrence(vec![number_input(10, converter)]);
        let advance = binding
            .advance_stateful_converters(0.5, &mut NoopScriptHost)
            .unwrap();
        assert_eq!(advance.changed, false);
        assert_eq!(advance.keep_going, false);
    }

    #[test]
    fn mixed_converter_group_advances_each_leaf_in_authored_order() {
        fn scripted(
            global_id: u32,
            instance: &RuntimeScriptInstanceHandle,
        ) -> RuntimeDataBindGraphConverter {
            RuntimeDataBindGraphConverter::Scripted {
                global_id,
                serialized_implemented_methods:
                    crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
                definition:
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default(
                    ),
                instance: Some(instance.clone()),
            }
        }

        fn interpolator(global_id: u32) -> RuntimeDataBindGraphConverter {
            RuntimeDataBindGraphConverter::Interpolator {
                global_id,
                duration: 1.0,
                interpolator: None,
            }
        }

        fn run(
            converter: RuntimeDataBindGraphConverter,
            calls: &Rc<RefCell<Vec<&'static str>>>,
        ) -> Vec<&'static str> {
            let mut state = RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
            let mut order = Vec::new();
            advance_converter_in_authored_order_with_observer(
                &converter,
                &mut state,
                0.25,
                &mut NoopScriptHost,
                &mut |leaf| {
                    order.push(match leaf {
                        RuntimeDataBindGraphConverter::Scripted { .. } => "scripted",
                        RuntimeDataBindGraphConverter::Interpolator { .. } => "interpolator",
                        _ => "other",
                    });
                },
            )
            .expect("mixed authored-order advance");
            assert_eq!(
                calls
                    .borrow()
                    .iter()
                    .filter(|call| **call == "advance")
                    .count(),
                1
            );
            order
        }

        let first_calls = Rc::new(RefCell::new(Vec::new()));
        let first_handle =
            RuntimeScriptInstanceHandle::new(converter(0.0, false, false, Rc::clone(&first_calls)));
        assert_eq!(
            run(
                RuntimeDataBindGraphConverter::Group(vec![
                    scripted(7, &first_handle),
                    interpolator(8),
                ]),
                &first_calls,
            ),
            ["scripted", "interpolator"],
        );

        let second_calls = Rc::new(RefCell::new(Vec::new()));
        let second_handle = RuntimeScriptInstanceHandle::new(converter(
            0.0,
            false,
            false,
            Rc::clone(&second_calls),
        ));
        assert_eq!(
            run(
                RuntimeDataBindGraphConverter::Group(vec![
                    interpolator(8),
                    scripted(7, &second_handle),
                ]),
                &second_calls,
            ),
            ["interpolator", "scripted"],
        );
    }

    #[derive(Debug)]
    struct OffsetConverter {
        offset: f64,
        ordinary_failure: Rc<Cell<bool>>,
        resource_failure: bool,
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl ScriptInstance for OffsetConverter {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            if method == ScriptMethod::Advance {
                self.calls.borrow_mut().push("advance");
                return Ok(ScriptValue::Bool(true));
            }
            Ok(ScriptValue::Nil)
        }

        fn call_data_converter(
            &mut self,
            method: ScriptDataConverterMethod,
            value: ScriptValue,
        ) -> Result<ScriptValue, ScriptError> {
            self.calls.borrow_mut().push(match method {
                ScriptDataConverterMethod::Convert => "convert",
                ScriptDataConverterMethod::ReverseConvert => "reverse",
            });
            if self.resource_failure {
                return Err(ScriptError::with_resource_code(
                    "terminal converter resource failure",
                    "script.resource.test",
                ));
            }
            if self.ordinary_failure.get() {
                return Err(ScriptError::new("ordinary converter failure"));
            }
            let ScriptValue::Number(value) = value else {
                return Ok(ScriptValue::Nil);
            };
            Ok(ScriptValue::Number(match method {
                ScriptDataConverterMethod::Convert => value + self.offset,
                ScriptDataConverterMethod::ReverseConvert => value - self.offset,
            }))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    fn converter(
        offset: f64,
        ordinary_failure: bool,
        resource_failure: bool,
        calls: Rc<RefCell<Vec<&'static str>>>,
    ) -> Box<dyn ScriptInstance> {
        converter_with_failure_toggle(
            offset,
            Rc::new(Cell::new(ordinary_failure)),
            resource_failure,
            calls,
        )
    }

    fn converter_with_failure_toggle(
        offset: f64,
        ordinary_failure: Rc<Cell<bool>>,
        resource_failure: bool,
        calls: Rc<RefCell<Vec<&'static str>>>,
    ) -> Box<dyn ScriptInstance> {
        Box::new(OffsetConverter {
            offset,
            ordinary_failure,
            resource_failure,
            calls,
        })
    }

    fn scripted_converter(
        global_id: u32,
        inputs: Vec<crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition>,
    ) -> RuntimeDataBindGraphConverter {
        let definition =
            crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(inputs);
        scripted_converter_with_definition(global_id, definition)
    }

    fn scripted_converter_with_definition(
        global_id: u32,
        definition: crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition,
    ) -> RuntimeDataBindGraphConverter {
        RuntimeDataBindGraphConverter::Scripted {
            global_id,
            serialized_implemented_methods:
                crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
            definition,
            instance: None,
        }
    }

    #[derive(Debug)]
    struct DropProbe {
        drops: Rc<Cell<usize>>,
    }

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops.set(self.drops.get() + 1);
        }
    }

    impl ScriptInstance for DropProbe {
        fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(false)
        }

        fn call_method(
            &mut self,
            _method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn repeated_group_items_keep_distinct_occurrence_paths_tables_and_caches() {
        fn custom_input()
        -> crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
            crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
                input_global_id: 50,
                kind: ScriptListenerInputKind::Number,
                properties: RuntimeScriptInputProperties::for_test(
                    "custom",
                    u32::MAX,
                    Some(RuntimeDataBindGraphValue::Number(1.0)),
                ),
                data_binds: vec![
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                        authored_order: 0,
                        source_path: Some(vec![0, 0]),
                        name_based: false,
                        property_key: crate::properties::property_key_for_name(
                            "ScriptInputNumber",
                            "propertyValue",
                        )
                        .map(u32::from)
                        .expect("number property key"),
                        target_property: RuntimeScriptInputTargetProperty::Value,
                        flags: 0,
                        converter_id: u32::MAX,
                    },
                ],
            }
        }

        let (file, mut context) = number_context(3.0);
        assert!(context.set_number_by_property_name("source", 3.0));
        let template = occurrence(vec![number_input(
            10,
            RuntimeDataBindGraphConverter::Group(vec![
                scripted_converter(7, vec![custom_input()]),
                scripted_converter(7, vec![custom_input()]),
            ]),
        )]);
        assert_eq!(
            template.scripted_converter_targets(),
            vec![(10, vec![0], 7, true), (10, vec![1], 7, true),],
            "authored group occurrences are not deduplicated by their shared global id"
        );

        let first_calls = Rc::new(RefCell::new(Vec::new()));
        let second_calls = Rc::new(RefCell::new(Vec::new()));
        let first_failure = Rc::new(Cell::new(false));
        let second_failure = Rc::new(Cell::new(false));
        let first = RuntimeScriptInstanceHandle::new(converter_with_failure_toggle(
            1.0,
            Rc::clone(&first_failure),
            false,
            Rc::clone(&first_calls),
        ));
        let second = RuntimeScriptInstanceHandle::new(converter_with_failure_toggle(
            2.0,
            Rc::clone(&second_failure),
            false,
            Rc::clone(&second_calls),
        ));
        let mut live = template.fresh_clone();
        assert!(live.attach_scripted_converter_instance_at_path(10, &[0], &first));
        assert!(live.attach_scripted_converter_instance_at_path(10, &[1], &second));
        bind(&mut live, &file, &context);
        let mut custom_updates = Vec::new();
        live.update_scripted_converter_inputs(&file, &context, &mut |instance, name, value| {
            let RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(value)) = value else {
                panic!("expected number custom-input update");
            };
            custom_updates.push((
                if instance == &first {
                    "first"
                } else if instance == &second {
                    "second"
                } else {
                    "unknown"
                },
                name.to_owned(),
                value,
            ));
            Ok(())
        })
        .expect("update both repeated converter-owned bind collections");
        assert_eq!(
            custom_updates,
            vec![
                ("first", ScriptCoreString::from("custom"), 3.0),
                ("second", ScriptCoreString::from("custom"), 3.0),
            ],
            "the same converter definition clones a distinct custom-input DataBind and table target for every group occurrence"
        );
        for path in [&[0][..], &[1][..]] {
            let snapshots = live
                .scripted_converter_input_snapshots(10, path)
                .expect("path-specific custom-input snapshots");
            assert!(matches!(
                snapshots.as_slice(),
                [ScriptListenerInputSnapshot {
                    input_global_id: 50,
                    value: Some(ScriptListenerInputSnapshotValue::Value(
                        ScriptValue::Number(3.0)
                    )),
                    ..
                }]
            ));
        }
        assert_eq!(
            number(live.resolve(&file, &context, 10, true).unwrap()),
            Some(6.0),
            "each occurrence contributes its own live conversion"
        );
        assert_eq!(&*first_calls.borrow(), &["convert"]);
        assert_eq!(&*second_calls.borrow(), &["convert"]);
        first_failure.set(true);
        second_failure.set(true);
        {
            let binding = live.inputs[0]
                .binding
                .as_mut()
                .expect("outer group binding");
            let Some(RuntimeDataBindGraphConverter::Group(converters)) = binding.converter.as_ref()
            else {
                panic!("expected converter group");
            };
            let RuntimeDataBindGraphConverterState::Group(states) = &mut binding.converter_state
            else {
                panic!("expected converter group state");
            };
            let mut random_source = RuntimeDataBindGraphFormulaRandomSource::default();
            assert_eq!(
                states[0]
                    .convert_value_with_formula_randoms_for_scripted_listener(
                        &converters[0],
                        &RuntimeDataBindGraphValue::Number(99.0),
                        &mut random_source,
                    )
                    .unwrap(),
                Some(RuntimeDataBindGraphValue::Number(4.0)),
                "the first failed call replays only its own prior m_dataValue"
            );
            assert_eq!(
                states[1]
                    .convert_value_with_formula_randoms_for_scripted_listener(
                        &converters[1],
                        &RuntimeDataBindGraphValue::Number(99.0),
                        &mut random_source,
                    )
                    .unwrap(),
                Some(RuntimeDataBindGraphValue::Number(6.0)),
                "the second same-id occurrence retains a distinct prior m_dataValue"
            );
        }

        let cold = live.fresh_clone();
        assert_eq!(
            cold.scripted_converter_targets(),
            vec![(10, vec![0], 7, true), (10, vec![1], 7, true),],
            "a cold clone detaches both occurrence-local tables without collapsing either path"
        );
        for path in [&[0][..], &[1][..]] {
            let snapshots = cold
                .scripted_converter_input_snapshots(10, path)
                .expect("cold path-specific custom-input snapshots");
            assert!(matches!(
                snapshots.as_slice(),
                [ScriptListenerInputSnapshot {
                    input_global_id: 50,
                    value: Some(ScriptListenerInputSnapshotValue::Value(
                        ScriptValue::Number(1.0)
                    )),
                    ..
                }]
            ));
        }
        // C++ clones every Group item, its ScriptedDataConverter table, and
        // its complete custom-input/DataBind collection by occurrence rather
        // than by shared definition id (`data_converter_group.cpp:48-75`;
        // `scripted_data_converter.cpp:235-273`).
    }

    #[test]
    fn bind_steps_interleave_each_converter_occurrence_before_the_next_input() {
        let binding = occurrence(vec![
            number_input(
                10,
                RuntimeDataBindGraphConverter::Group(vec![
                    scripted_converter(7, Vec::new()),
                    RuntimeDataBindGraphConverter::PassThrough,
                ]),
            ),
            number_input(11, scripted_converter(8, Vec::new())),
        ]);

        assert_eq!(
            binding.scripted_converter_bind_steps(),
            vec![
                RuntimeScriptedListenerDataConverterBindStep::BindListenerInput {
                    action_global_id: 42,
                    listener_input_global_id: 10,
                },
                RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                    action_global_id: 42,
                    listener_input_global_id: 10,
                    converter_path: Vec::new(),
                },
                RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                    action_global_id: 42,
                    listener_input_global_id: 10,
                    converter_path: vec![0],
                },
                RuntimeScriptedListenerDataConverterBindStep::Rehydrate {
                    action_global_id: 42,
                    listener_input_global_id: 10,
                    converter_path: vec![0],
                    converter_global_id: 7,
                    inits: true,
                },
                RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                    action_global_id: 42,
                    listener_input_global_id: 10,
                    converter_path: vec![1],
                },
                RuntimeScriptedListenerDataConverterBindStep::FinalizeListenerInput {
                    action_global_id: 42,
                    listener_input_global_id: 10,
                },
                RuntimeScriptedListenerDataConverterBindStep::BindListenerInput {
                    action_global_id: 42,
                    listener_input_global_id: 11,
                },
                RuntimeScriptedListenerDataConverterBindStep::BindConverter {
                    action_global_id: 42,
                    listener_input_global_id: 11,
                    converter_path: Vec::new(),
                },
                RuntimeScriptedListenerDataConverterBindStep::Rehydrate {
                    action_global_id: 42,
                    listener_input_global_id: 11,
                    converter_path: Vec::new(),
                    converter_global_id: 8,
                    inits: true,
                },
                RuntimeScriptedListenerDataConverterBindStep::FinalizeListenerInput {
                    action_global_id: 42,
                    listener_input_global_id: 11,
                },
            ],
            "C++ binds, reinitializes, and final-rebinds each occurrence before binding the next one"
        );
    }

    #[test]
    fn successful_scripted_converter_hydration_wakes_only_its_idle_outer_occurrence() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let first_handle =
            RuntimeScriptInstanceHandle::new(converter(0.0, false, false, Rc::clone(&calls)));
        let second_handle =
            RuntimeScriptInstanceHandle::new(converter(0.0, false, false, Rc::clone(&calls)));
        let mut live = occurrence(vec![
            number_input(10, scripted_converter(7, Vec::new())),
            number_input(11, scripted_converter(8, Vec::new())),
        ]);
        assert!(live.attach_scripted_converter_instance_at_path(10, &[], &first_handle));
        assert!(live.attach_scripted_converter_instance_at_path(11, &[], &second_handle));

        for input in &mut live.inputs {
            let binding = input.binding.as_mut().expect("scripted converter binding");
            binding.retained_bind =
                RuntimeRetainedDataBind::new(crate::data_bind_graph::DATA_BIND_FLAG_TWO_WAY, false);
            binding.attach_converter_parent();
            binding.retained_bind.mark_rebind_reconcile();
            assert!(binding.retained_bind.take_target_dirt());
            assert!(binding.retained_bind.take_pending_source_dirt());
            assert!(
                binding.retained_bind.target_origin(),
                "target-first reconcile establishes the direction hydration must preserve"
            );
        }

        assert!(
            !live.mark_scripted_converter_hydrated(10, &[0]),
            "a missing converter occurrence cannot wake any outer bind"
        );
        assert!(live.inputs.iter().all(|input| {
            input
                .binding
                .as_ref()
                .is_some_and(|binding| binding.retained_bind.pending_dirt().is_empty())
        }));

        assert!(live.mark_scripted_converter_hydrated(10, &[]));
        let first = live.inputs[0].binding.as_ref().unwrap();
        let second = live.inputs[1].binding.as_ref().unwrap();
        assert!(first.retained_bind.target_origin());
        assert!(
            first
                .retained_bind
                .pending_dirt()
                .contains(RuntimeCellDirt::BINDINGS_TARGET)
        );
        assert!(
            second.retained_bind.pending_dirt().is_empty(),
            "didHydrateScriptInputs wakes only the converter's exact parent DataBind occurrence"
        );
    }

    #[test]
    fn fresh_clone_does_not_retain_the_live_converter_table() {
        let drops = Rc::new(Cell::new(0));
        let handle = RuntimeScriptInstanceHandle::new(Box::new(DropProbe {
            drops: Rc::clone(&drops),
        }));
        let mut live = occurrence(vec![number_input(10, scripted_converter(7, Vec::new()))]);
        assert!(live.attach_scripted_converter_instance_at_path(10, &[], &handle));
        let cold = live.fresh_clone();

        drop(handle);
        drop(live);
        assert_eq!(
            drops.get(),
            1,
            "the cold clone must not retain the live occurrence's Lua table"
        );
        assert_eq!(
            cold.scripted_converter_targets(),
            vec![(10, Vec::new(), 7, true)],
            "the cold clone retains the authored converter occurrence for a later reinit"
        );
    }

    #[test]
    fn scripted_converter_custom_data_binds_update_in_global_authored_order() {
        fn binding(
            authored_order: u32,
        ) -> crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition
        {
            crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                authored_order,
                source_path: Some(vec![0, 0]),
                name_based: false,
                property_key: crate::properties::property_key_for_name(
                    "ScriptInputNumber",
                    "propertyValue",
                )
                .map(u32::from)
                .expect("number property key"),
                target_property: RuntimeScriptInputTargetProperty::Value,
                flags: 0,
                converter_id: u32::MAX,
            }
        }

        fn custom_input(
            input_global_id: u32,
            name: &str,
            data_binds: Vec<
                crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition,
            >,
        ) -> crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
            crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
                input_global_id,
                kind: ScriptListenerInputKind::Number,
                properties: RuntimeScriptInputProperties::for_test(
                    name,
                    u32::MAX,
                    Some(RuntimeDataBindGraphValue::Number(0.0)),
                ),
                data_binds,
            }
        }

        let (file, mut context) = number_context(3.0);
        assert!(context.set_number_by_property_name("source", 3.0));
        let custom_inputs = vec![
            custom_input(50, "a", vec![binding(0), binding(2)]),
            custom_input(51, "b", vec![binding(1)]),
        ];
        let mut occurrence = occurrence(vec![number_input(
            10,
            scripted_converter_with_definition(
                7,
                crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition {
                    inputs: custom_inputs,
                    data_bind_order: vec![(0, 0), (1, 0), (0, 1)],
                },
            ),
        )]);
        let outer = RuntimeScriptInstanceHandle::new(converter(
            0.0,
            false,
            false,
            Rc::new(RefCell::new(Vec::new())),
        ));
        assert!(occurrence.attach_scripted_converter_instance_at_path(10, &[], &outer));
        bind(&mut occurrence, &file, &context);
        let outer_binding = occurrence.inputs[0]
            .binding
            .as_ref()
            .expect("outer DataBind");
        let RuntimeDataBindGraphConverterState::Scripted(state) = &outer_binding.converter_state
        else {
            panic!("scripted state");
        };
        assert_eq!(
            state.data_bind_order_for_test(),
            &[(0, 0), (1, 0), (0, 1)],
            "the cloned DataConverter owns one file-order collection rather than regrouping by ScriptInput"
        );

        let mut applied = Vec::new();
        let outer_binding = occurrence.inputs[0]
            .binding
            .as_mut()
            .expect("outer DataBind");
        outer_binding
            .converter_state
            .update_scripted_converter_inputs(
                outer_binding
                    .converter
                    .as_mut()
                    .expect("scripted converter"),
                &file,
                &context,
                &mut |instance, name, value| {
                    let RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(value)) =
                        value
                    else {
                        panic!("expected number update");
                    };
                    applied.push((instance == &outer, name.to_owned(), value));
                    Ok(())
                },
            )
            .unwrap();

        assert_eq!(
            applied,
            vec![
                (true, ScriptCoreString::from("a"), 3.0),
                (true, ScriptCoreString::from("b"), 3.0),
            ],
            "the final A occurrence still executes in authored order but produces no table callback because it writes the same raw value; subordinate converter pointers are absent after DataBindBase::copy"
        );
    }

    #[test]
    fn scripted_converter_custom_input_directions_match_false_update_boundary() {
        const DATA_BIND_TO_SOURCE: u64 = 1 << 0;
        const DATA_BIND_TWO_WAY: u64 = 1 << 1;
        const DATA_BIND_ONCE: u64 = 1 << 2;

        fn custom_input(
            flags: u64,
        ) -> crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
            crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition {
                input_global_id: 50,
                kind: ScriptListenerInputKind::Number,
                properties: RuntimeScriptInputProperties::for_test(
                    "custom",
                    u32::MAX,
                    Some(RuntimeDataBindGraphValue::Number(1.0)),
                ),
                data_binds: vec![
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                        authored_order: 0,
                        source_path: Some(vec![0, 0]),
                        name_based: false,
                        property_key: crate::properties::property_key_for_name(
                            "ScriptInputNumber",
                            "propertyValue",
                        )
                        .map(u32::from)
                        .expect("number property key"),
                        target_property: RuntimeScriptInputTargetProperty::Value,
                        flags,
                        converter_id: u32::MAX,
                    },
                ],
            }
        }

        fn custom_snapshot(binding: &RuntimeScriptedListenerActionBindingOccurrence) -> f64 {
            let snapshots = binding
                .scripted_converter_input_snapshots(10, &[])
                .expect("outer converter snapshots");
            let Some(ScriptListenerInputSnapshotValue::Value(ScriptValue::Number(value))) =
                snapshots[0].value
            else {
                panic!("custom input is not a number");
            };
            value
        }

        fn binding_with_flags(
            flags: u64,
            handle: &RuntimeScriptInstanceHandle,
        ) -> RuntimeScriptedListenerActionBindingOccurrence {
            let mut binding = occurrence(vec![number_input(
                10,
                scripted_converter(7, vec![custom_input(flags)]),
            )]);
            assert!(binding.attach_scripted_converter_instance_at_path(10, &[], handle));
            binding
        }

        let (file, mut context) = number_context(3.0);
        assert!(context.set_number_by_property_name("source", 3.0));
        let handle = RuntimeScriptInstanceHandle::new(converter(
            0.0,
            false,
            false,
            Rc::new(RefCell::new(Vec::new())),
        ));

        let mut to_source = binding_with_flags(DATA_BIND_TO_SOURCE, &handle);
        bind(&mut to_source, &file, &context);
        let mut to_source_updates = Vec::new();
        let binding = to_source.inputs[0].binding.as_mut().expect("outer bind");
        binding
            .converter_state
            .update_scripted_converter_inputs(
                binding.converter.as_mut().expect("scripted converter"),
                &file,
                &context,
                &mut |_, name, value| {
                    to_source_updates.push((name.to_owned(), value));
                    Ok(())
                },
            )
            .unwrap();
        assert!(to_source_updates.is_empty());
        assert_eq!(
            custom_snapshot(&to_source),
            1.0,
            "pure ToSource is retained but inert in updateDataBinds(false)"
        );

        let mut target_first_two_way =
            binding_with_flags(DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY, &handle);
        bind(&mut target_first_two_way, &file, &context);
        let mut two_way_updates = Vec::new();
        let binding = target_first_two_way.inputs[0]
            .binding
            .as_mut()
            .expect("outer bind");
        binding
            .converter_state
            .update_scripted_converter_inputs(
                binding.converter.as_mut().expect("scripted converter"),
                &file,
                &context,
                &mut |_, name, value| {
                    two_way_updates.push((name.to_owned(), value));
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(two_way_updates.len(), 1);
        assert_eq!(
            custom_snapshot(&target_first_two_way),
            3.0,
            "TwoWay still applies source-to-target at this false-update boundary even when target-first is authored"
        );

        let mut once = binding_with_flags(DATA_BIND_ONCE, &handle);
        bind(&mut once, &file, &context);
        let mut once_updates = Vec::new();
        {
            let binding = once.inputs[0].binding.as_mut().expect("outer bind");
            binding
                .converter_state
                .update_scripted_converter_inputs(
                    binding.converter.as_mut().expect("scripted converter"),
                    &file,
                    &context,
                    &mut |_, name, value| {
                        once_updates.push((name.to_owned(), value));
                        Ok(())
                    },
                )
                .unwrap();
        }
        assert_eq!(custom_snapshot(&once), 3.0);
        assert!(context.set_number_by_property_name("source", 4.0));
        {
            let binding = once.inputs[0].binding.as_mut().expect("outer bind");
            binding
                .converter_state
                .update_scripted_converter_inputs(
                    binding.converter.as_mut().expect("scripted converter"),
                    &file,
                    &context,
                    &mut |_, name, value| {
                        once_updates.push((name.to_owned(), value));
                        Ok(())
                    },
                )
                .unwrap();
        }
        assert_eq!(once_updates.len(), 1);
        assert_eq!(
            custom_snapshot(&once),
            3.0,
            "Once performs the initial reconcile without subscribing to later source changes"
        );

        assert!(context.set_number_by_property_name("source", 3.0));
        let mut public_to_source = binding_with_flags(DATA_BIND_TO_SOURCE, &handle);
        bind(&mut public_to_source, &file, &context);
        public_to_source
            .public_update_data_binds(&file, Some(&handle), &mut |_, _, _| Ok(()))
            .unwrap();
        assert_eq!(
            context.number_value_by_property_name("source"),
            Some(3.0),
            "the outer public pass reaches converter-owned binds with false semantics"
        );
        assert_eq!(custom_snapshot(&public_to_source), 1.0);

        assert!(context.set_number_by_property_name("source", 4.0));
        let mut public_target_first =
            binding_with_flags(DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY, &handle);
        bind(&mut public_target_first, &file, &context);
        public_target_first
            .public_update_data_binds(&file, Some(&handle), &mut |_, _, _| Ok(()))
            .unwrap();
        assert_eq!(
            context.number_value_by_property_name("source"),
            Some(4.0),
            "target-first converter-owned TwoWay cannot reverse during an outer public pass"
        );

        assert!(context.set_number_by_property_name("source", 3.0));
        let mut public_source_first =
            binding_with_flags(DATA_BIND_TO_SOURCE | DATA_BIND_TWO_WAY | (1 << 3), &handle);
        bind(&mut public_source_first, &file, &context);
        let mut public_updates = Vec::new();
        public_source_first
            .public_update_data_binds(&file, Some(&handle), &mut |_, name, value| {
                public_updates.push((name.to_owned(), value));
                Ok(())
            })
            .unwrap();
        assert_eq!(
            context.number_value_by_property_name("source"),
            Some(3.0),
            "source-first converter-owned TwoWay applies source before reversing the normalized target"
        );
        assert_eq!(custom_snapshot(&public_source_first), 3.0);
        assert!(
            public_updates
                .iter()
                .any(|(name, _)| name == &ScriptCoreString::from("custom")),
            "the cloned custom ScriptInput callback projects its source-first value into the owning converter table"
        );
    }

    #[test]
    fn public_update_reconciles_cloned_script_inputs_in_cpp_favor_order_and_reverse_converts() {
        const TO_SOURCE: u64 = 1 << 0;
        const TWO_WAY: u64 = 1 << 1;
        const SOURCE_FIRST: u64 = 1 << 3;

        fn configured(
            flags: u64,
            converter: Option<(RuntimeDataBindGraphConverter, RuntimeScriptInstanceHandle)>,
        ) -> RuntimeScriptedListenerActionBindingOccurrence {
            let (converter, handle) = match converter {
                Some((converter, handle)) => (converter, Some(handle)),
                None => (RuntimeDataBindGraphConverter::PassThrough, None),
            };
            let mut occurrence = occurrence(vec![number_input(10, converter)]);
            let binding = occurrence.inputs[0]
                .binding
                .as_mut()
                .expect("number binding");
            binding.flags = flags;
            binding.retained_bind = RuntimeRetainedDataBind::new(flags, false);
            if let Some(handle) = handle {
                assert!(occurrence.attach_scripted_converter_instance_at_path(10, &[], &handle));
            }
            occurrence
        }

        fn run(
            flags: u64,
            target: f32,
            converter: Option<(RuntimeDataBindGraphConverter, RuntimeScriptInstanceHandle)>,
        ) -> (f32, f32, Vec<RuntimeScriptedListenerBindingUpdate>) {
            let (file, mut context) = number_context(3.0);
            assert!(context.set_number_by_property_name("source", 3.0));
            let mut occurrence = configured(flags, converter);
            assert_eq!(
                occurrence.inputs[0].properties.apply_target(
                    &file,
                    ScriptListenerInputKind::Number,
                    RuntimeScriptInputTargetProperty::Value,
                    RuntimeDataBindGraphValue::Number(target),
                ),
                if target == 0.0 {
                    RuntimeScriptInputTargetApply::Unchanged
                } else {
                    RuntimeScriptInputTargetApply::ChangedWithTableProjection
                }
            );
            bind(&mut occurrence, &file, &context);
            let updates = occurrence
                .public_update_data_binds(&file, None, &mut |_, _, _| Ok(()))
                .expect("public scripted-input reconciliation");
            let source = context
                .number_value_by_property_name("source")
                .expect("number source");
            let target = occurrence.inputs[0]
                .properties
                .value()
                .and_then(|value| match value {
                    RuntimeDataBindGraphValue::Number(value) => Some(*value),
                    _ => None,
                })
                .expect("number target");
            (source, target, updates)
        }

        assert_eq!(
            run(TO_SOURCE, 1.0, None),
            (1.0, 1.0, Vec::new()),
            "pure ToSource seeds the retained source from the cloned Core target"
        );
        assert_eq!(
            run(TO_SOURCE | TWO_WAY, 1.0, None),
            (1.0, 1.0, Vec::new()),
            "target-first TwoWay pulls the target before the source apply"
        );
        let (source, target, updates) = run(TO_SOURCE | TWO_WAY | SOURCE_FIRST, 1.0, None);
        assert_eq!((source, target), (3.0, 3.0));
        assert_eq!(
            updates.len(),
            1,
            "source-first TwoWay projects the source into the cloned ScriptInput before reversing it"
        );

        let calls = Rc::new(RefCell::new(Vec::new()));
        let handle =
            RuntimeScriptInstanceHandle::new(converter(2.0, false, false, Rc::clone(&calls)));
        let scripted = scripted_converter(7, Vec::new());
        assert_eq!(
            run(TO_SOURCE, 1.0, Some((scripted, handle))),
            (3.0, 1.0, Vec::new()),
            "reverseConvert owns the target-to-source value before the retained source write"
        );
        assert_eq!(&*calls.borrow(), &["convert"]);
    }

    #[test]
    fn converter_method_follows_authored_main_direction_not_physical_flow() {
        const TO_SOURCE: u64 = 1 << 0;
        const TWO_WAY: u64 = 1 << 1;

        fn run(flags: u64) -> (f32, f32, Vec<&'static str>) {
            let (file, mut context) = number_context(3.0);
            assert!(context.set_number_by_property_name("source", 3.0));
            let calls = Rc::new(RefCell::new(Vec::new()));
            let handle =
                RuntimeScriptInstanceHandle::new(converter(2.0, false, false, Rc::clone(&calls)));
            let mut occurrence =
                occurrence(vec![number_input(10, scripted_converter(7, Vec::new()))]);
            {
                let input = &mut occurrence.inputs[0];
                assert_eq!(
                    input.properties.apply_target(
                        &file,
                        ScriptListenerInputKind::Number,
                        RuntimeScriptInputTargetProperty::Value,
                        RuntimeDataBindGraphValue::Number(1.0),
                    ),
                    RuntimeScriptInputTargetApply::ChangedWithTableProjection,
                );
                let binding = input.binding.as_mut().expect("number binding");
                binding.flags = flags;
                binding.retained_bind = RuntimeRetainedDataBind::new(flags, false);
            }
            assert!(occurrence.attach_scripted_converter_instance_at_path(10, &[], &handle));
            bind(&mut occurrence, &file, &context);
            occurrence
                .public_update_data_binds(&file, None, &mut |_, _, _| Ok(()))
                .expect("four-direction public reconciliation");
            let source = context
                .number_value_by_property_name("source")
                .expect("source");
            let target = occurrence.inputs[0]
                .properties
                .value()
                .and_then(|value| match value {
                    RuntimeDataBindGraphValue::Number(value) => Some(*value),
                    _ => None,
                })
                .expect("target");
            let calls = calls.borrow().clone();
            (source, target, calls)
        }

        assert_eq!(run(0), (3.0, 5.0, vec!["convert"]));
        assert_eq!(
            run(TWO_WAY),
            (-1.0, 1.0, vec!["reverse", "convert"]),
            "ToTarget-main uses reverseConvert for the physical target-to-source leg"
        );
        assert_eq!(
            run(TO_SOURCE),
            (3.0, 1.0, vec!["convert"]),
            "ToSource-main uses convert for the physical target-to-source leg"
        );
        assert_eq!(
            run(TO_SOURCE | TWO_WAY),
            (3.0, 1.0, vec!["convert", "reverse"]),
            "ToSource-main uses reverseConvert for the physical source-to-target leg"
        );
    }

    #[test]
    fn scripted_converter_tables_are_distinct_per_bind_occurrence_and_advance() {
        let (file, mut context) = number_context(3.0);
        assert!(context.set_number_by_property_name("source", 3.0));
        let template = occurrence(vec![number_input(
            10,
            RuntimeDataBindGraphConverter::Scripted {
                global_id: 7,
                serialized_implemented_methods:
                    crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
                definition:
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default(
                    ),
                instance: None,
            },
        )]);
        let mut first = template.fresh_clone();
        let mut second = template.fresh_clone();
        let first_calls = Rc::new(RefCell::new(Vec::new()));
        let second_calls = Rc::new(RefCell::new(Vec::new()));
        let first_handle =
            RuntimeScriptInstanceHandle::new(converter(1.0, false, false, Rc::clone(&first_calls)));
        let second_handle = RuntimeScriptInstanceHandle::new(converter(
            2.0,
            false,
            false,
            Rc::clone(&second_calls),
        ));
        assert!(first.attach_scripted_converter_instance_at_path(10, &[], &first_handle));
        assert!(second.attach_scripted_converter_instance_at_path(10, &[], &second_handle));
        bind(&mut first, &file, &context);
        bind(&mut second, &file, &context);

        assert_eq!(
            number(first.resolve(&file, &context, 10, true).unwrap()),
            Some(4.0)
        );
        assert_eq!(
            number(second.resolve(&file, &context, 10, true).unwrap()),
            Some(5.0)
        );
        first
            .advance_stateful_converters(0.25, &mut NoopScriptHost)
            .unwrap();
        second
            .advance_stateful_converters(0.25, &mut NoopScriptHost)
            .unwrap();
        assert_eq!(&*first_calls.borrow(), &["convert", "advance"]);
        assert_eq!(&*second_calls.borrow(), &["convert", "advance"]);
    }

    #[test]
    fn ordinary_converter_failure_does_not_abort_later_bind_but_resource_failure_does() {
        let (file, mut context) = number_context(3.0);
        assert!(context.set_number_by_property_name("source", 3.0));
        let mut binding = occurrence(vec![
            number_input(
                10,
                RuntimeDataBindGraphConverter::Scripted {
                    global_id: 7,
                    serialized_implemented_methods:
                        crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
                    definition:
                        crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default(),
                    instance: None,
                },
            ),
            number_input(
                11,
                RuntimeDataBindGraphConverter::Scripted {
                    global_id: 8,
                    serialized_implemented_methods:
                        crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
                    definition:
                        crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default(),
                    instance: None,
                },
            ),
        ]);
        let failed_calls = Rc::new(RefCell::new(Vec::new()));
        let later_calls = Rc::new(RefCell::new(Vec::new()));
        let failed =
            RuntimeScriptInstanceHandle::new(converter(0.0, true, false, Rc::clone(&failed_calls)));
        let later =
            RuntimeScriptInstanceHandle::new(converter(2.0, false, false, Rc::clone(&later_calls)));
        assert!(binding.attach_scripted_converter_instance_at_path(10, &[], &failed));
        assert!(binding.attach_scripted_converter_instance_at_path(11, &[], &later));
        bind(&mut binding, &file, &context);
        let updates = binding
            .resolve_runtime_table_updates(&file, &context)
            .unwrap();
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].input_name, "value11");
        assert_eq!(
            updates[0].value,
            RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(5.0))
        );

        let terminal = RuntimeScriptInstanceHandle::new(converter(
            0.0,
            false,
            true,
            Rc::new(RefCell::new(Vec::new())),
        ));
        let mut terminal_binding = occurrence(vec![number_input(
            12,
            RuntimeDataBindGraphConverter::Scripted {
                global_id: 9,
                serialized_implemented_methods:
                    crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
                definition:
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default(
                    ),
                instance: None,
            },
        )]);
        assert!(terminal_binding.attach_scripted_converter_instance_at_path(12, &[], &terminal,));
        bind(&mut terminal_binding, &file, &context);
        let error = terminal_binding
            .resolve_runtime_table_updates(&file, &context)
            .expect_err("resource errors remain terminal");
        assert_eq!(error.resource_code(), Some("script.resource.test"));
    }

    #[test]
    fn failed_scripted_converter_keeps_group_order_and_runs_later_items() {
        let (file, mut context) = number_context(3.0);
        assert!(context.set_number_by_property_name("source", 3.0));
        let mut binding = occurrence(vec![number_input(
            10,
            RuntimeDataBindGraphConverter::Group(vec![
                RuntimeDataBindGraphConverter::Scripted {
                    global_id: 7,
                    serialized_implemented_methods:
                        crate::script_asset::RuntimeScriptImplementedMethods::METHOD_MASK,
                    definition:
                        crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default(),
                    instance: None,
                },
                RuntimeDataBindGraphConverter::Formula { tokens: Vec::new() },
            ]),
        )]);
        let failed_calls = Rc::new(RefCell::new(Vec::new()));
        let failed =
            RuntimeScriptInstanceHandle::new(converter(0.0, true, false, Rc::clone(&failed_calls)));
        assert!(binding.attach_scripted_converter_instance_at_path(10, &[0], &failed));
        assert_eq!(
            binding.inputs[0].properties.apply_target(
                &file,
                ScriptListenerInputKind::Number,
                RuntimeScriptInputTargetProperty::Value,
                RuntimeDataBindGraphValue::Number(9.0),
            ),
            RuntimeScriptInputTargetApply::ChangedWithTableProjection,
        );
        bind(&mut binding, &file, &context);

        let updates = binding
            .resolve_runtime_table_updates(&file, &context)
            .expect("an ordinary protected-call failure is not terminal");
        assert_eq!(
            updates,
            vec![RuntimeScriptedListenerBindingUpdate {
                action_global_id: 42,
                input_name: "value10".into(),
                value: RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(0.0)),
            }],
            "the later Formula consumes the untyped sentinel and supplies its C++ default"
        );
        assert_eq!(&*failed_calls.borrow(), &["convert"]);
    }
}
