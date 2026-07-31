//! Pinned `src/scripted/scripted_data_converter.cpp` occurrence behavior.

use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::data_bind_graph::{
    RuntimeDataBindGraphValue, data_bind_flags_apply_source_to_target,
    data_bind_flags_apply_target_to_source, runtime_cell_value_from_graph_value,
    runtime_graph_value_from_bound_cell,
};
use crate::retained_data_bind::{RuntimeConverterParentWake, RuntimeRetainedDataBind};
use crate::script_asset::RuntimeScriptImplementedMethods;
use crate::scripted_object::{
    RuntimeScriptInputProperties, RuntimeScriptInputTargetApply, RuntimeScriptInputTargetProperty,
};
use crate::scripting::{
    RuntimeScriptInstanceHandle, ScriptCoreString, ScriptDataConverterMethod, ScriptError,
    ScriptHost, ScriptListenerInputKind, ScriptListenerInputSnapshot,
    ScriptListenerInputSnapshotValue, ScriptValue,
};
use crate::state_machine::RuntimeScriptedListenerBoundValue;
use crate::view_model::RuntimeOwnedViewModelInstance;
use crate::view_model_cell::RuntimeCellNotificationQueue;
use crate::view_model_cell::RuntimeViewModelCell;

const DATA_BIND_FLAG_ONCE: u64 = 1 << 2;

/// Authored custom-input collection owned by one C++ ScriptedDataConverter.
///
/// Every DataBind occurrence stays in authored order. `ScriptInput::dataBind`
/// points at the last one, but `DataConverter::m_dataBinds` owns and updates
/// the complete occurrence list (`data_bind.cpp:66-95`;
/// `scripted_data_converter.cpp:242-269`).
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct RuntimeScriptedDataConverterInputDefinition {
    pub(crate) input_global_id: u32,
    pub(crate) kind: ScriptListenerInputKind,
    pub(crate) properties: RuntimeScriptInputProperties,
    pub(crate) data_binds: Vec<RuntimeScriptedDataConverterDataBindDefinition>,
}

/// Immutable clone recipe for one authored `ScriptedDataConverter`.
///
/// Inputs and owned DataBinds have two independent C++ insertion orders.
/// Keeping both sequences directly avoids reconstructing the converter's
/// `m_dataBinds` collection by sorting object ids after import.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct RuntimeScriptedDataConverterDefinition {
    pub(crate) inputs: Vec<RuntimeScriptedDataConverterInputDefinition>,
    pub(crate) data_bind_order: Vec<(usize, usize)>,
}

#[cfg(test)]
impl RuntimeScriptedDataConverterDefinition {
    pub(crate) fn with_grouped_test_bind_order(
        inputs: Vec<RuntimeScriptedDataConverterInputDefinition>,
    ) -> Self {
        let data_bind_order = inputs
            .iter()
            .enumerate()
            .flat_map(|(input_index, input)| {
                (0..input.data_binds.len())
                    .map(move |data_bind_index| (input_index, data_bind_index))
            })
            .collect();
        Self {
            inputs,
            data_bind_order,
        }
    }
}

/// Read-only proof surface for one cloned `ScriptedDataConverter` occurrence.
///
/// This is intentionally occurrence-keyed by the parent DataBind and Group
/// path. Authored converter ids identify definitions and therefore cannot
/// distinguish repeated uses of the same converter
/// (`data_converter_group.cpp:19-31`;
/// `scripted_data_converter.cpp:235-269`).
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeScriptedDataConverterOccurrenceSnapshot {
    pub parent_data_bind_index: usize,
    pub converter_path: Vec<usize>,
    pub converter_global_id: u32,
    pub serialized_implemented_methods: u32,
    pub attached: bool,
    pub inputs: Vec<ScriptListenerInputSnapshot>,
    pub data_binds: Vec<RuntimeScriptedDataConverterDataBindSnapshot>,
}

/// Read-only metadata for one cloned `ScriptedDataConverter::m_dataBinds`
/// occurrence in its exact collection order.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeScriptedDataConverterDataBindSnapshot {
    pub collection_index: usize,
    pub input_index: usize,
    pub input_data_bind_index: usize,
    pub context_bindable: bool,
    pub source_path: Option<Vec<u32>>,
    pub name_based: bool,
    pub property_key: u32,
    pub flags: u64,
    pub converter_id: u32,
    pub is_final_for_target: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum RuntimeScriptedDataConverterDataBindDefinition {
    Inert {
        authored_order: u32,
        property_key: u32,
        target_property: RuntimeScriptInputTargetProperty,
        flags: u64,
        converter_id: u32,
    },
    Context {
        authored_order: u32,
        source_path: Option<Vec<u32>>,
        name_based: bool,
        property_key: u32,
        target_property: RuntimeScriptInputTargetProperty,
        flags: u64,
        converter_id: u32,
    },
}

impl RuntimeScriptedDataConverterDataBindDefinition {
    pub(crate) fn authored_order(&self) -> u32 {
        match self {
            Self::Inert { authored_order, .. } | Self::Context { authored_order, .. } => {
                *authored_order
            }
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeScriptedDataConverterInputOccurrence {
    input_global_id: u32,
    kind: ScriptListenerInputKind,
    properties: RuntimeScriptInputProperties,
    data_binds: Vec<RuntimeScriptedDataConverterDataBindOccurrence>,
}

#[derive(Debug, Clone)]
enum RuntimeScriptedDataConverterDataBindOccurrence {
    Inert {
        property_key: u32,
        target_property: RuntimeScriptInputTargetProperty,
        flags: u64,
        converter_id: u32,
    },
    Context {
        source_path: Option<Vec<u32>>,
        name_based: bool,
        property_key: u32,
        target_property: RuntimeScriptInputTargetProperty,
        flags: u64,
        converter_id: u32,
        retained_bind: RuntimeRetainedDataBind,
        last_target: Option<RuntimeDataBindGraphValue>,
    },
}

/// Mutable `ScriptedDataConverter::m_dataValue` owned by one cloned converter
/// occurrence. It survives DataContext rebind and script-table reinit; only a
/// fresh converter clone starts empty.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeScriptedDataConverterState {
    cached_value: Option<RuntimeDataBindGraphValue>,
    inputs: Vec<RuntimeScriptedDataConverterInputOccurrence>,
    /// `DataConverter::m_dataBinds` is one authored-order collection shared
    /// by all custom inputs. Keeping the input-local storage plus this exact
    /// traversal index preserves C++ interleaving without aliasing mutable
    /// input state.
    data_bind_order: Vec<(usize, usize)>,
}

impl RuntimeScriptedDataConverterState {
    pub(crate) fn from_definition(definition: &RuntimeScriptedDataConverterDefinition) -> Self {
        Self {
            cached_value: None,
            inputs: definition
                .inputs
                .iter()
                .map(|input| RuntimeScriptedDataConverterInputOccurrence {
                    input_global_id: input.input_global_id,
                    kind: input.kind,
                    properties: input.properties.clone_for_scripted_object(),
                    data_binds: input
                        .data_binds
                        .iter()
                        .map(|binding| match binding {
                            RuntimeScriptedDataConverterDataBindDefinition::Inert {
                                property_key,
                                target_property,
                                flags,
                                converter_id,
                                ..
                            } => RuntimeScriptedDataConverterDataBindOccurrence::Inert {
                                property_key: *property_key,
                                target_property: *target_property,
                                flags: *flags,
                                converter_id: *converter_id,
                            },
                            RuntimeScriptedDataConverterDataBindDefinition::Context {
                                source_path,
                                name_based,
                                property_key,
                                target_property,
                                flags,
                                converter_id,
                                ..
                            } => RuntimeScriptedDataConverterDataBindOccurrence::Context {
                                source_path: source_path.clone(),
                                name_based: *name_based,
                                property_key: *property_key,
                                target_property: *target_property,
                                flags: *flags,
                                converter_id: *converter_id,
                                retained_bind: RuntimeRetainedDataBind::new(
                                    *flags,
                                    *flags & DATA_BIND_FLAG_ONCE != 0,
                                ),
                                last_target: None,
                            },
                        })
                        .collect(),
                })
                .collect(),
            data_bind_order: definition.data_bind_order.clone(),
        }
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

    pub(crate) fn data_bind_snapshots(&self) -> Vec<RuntimeScriptedDataConverterDataBindSnapshot> {
        self.data_bind_order
            .iter()
            .enumerate()
            .filter_map(
                |(collection_index, &(input_index, input_data_bind_index))| {
                    let input = self.inputs.get(input_index)?;
                    let binding = input.data_binds.get(input_data_bind_index)?;
                    let (
                        context_bindable,
                        source_path,
                        name_based,
                        property_key,
                        flags,
                        converter_id,
                    ) = match binding {
                        RuntimeScriptedDataConverterDataBindOccurrence::Inert {
                            property_key,
                            flags,
                            converter_id,
                            ..
                        } => (false, None, false, *property_key, *flags, *converter_id),
                        RuntimeScriptedDataConverterDataBindOccurrence::Context {
                            source_path,
                            name_based,
                            property_key,
                            flags,
                            converter_id,
                            ..
                        } => (
                            true,
                            source_path.clone(),
                            *name_based,
                            *property_key,
                            *flags,
                            *converter_id,
                        ),
                    };
                    Some(RuntimeScriptedDataConverterDataBindSnapshot {
                        collection_index,
                        input_index,
                        input_data_bind_index,
                        context_bindable,
                        source_path,
                        name_based,
                        property_key,
                        flags,
                        converter_id,
                        is_final_for_target: input_data_bind_index + 1 == input.data_binds.len(),
                    })
                },
            )
            .collect()
    }

    pub(crate) fn set_container_wake(&mut self, wake: Option<RuntimeConverterParentWake>) {
        for input in &mut self.inputs {
            for binding in &mut input.data_binds {
                if let RuntimeScriptedDataConverterDataBindOccurrence::Context {
                    retained_bind,
                    ..
                } = binding
                {
                    retained_bind.set_container_wake(wake.clone());
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn bind_test_input_source(
        &mut self,
        input_index: usize,
        data_bind_index: usize,
        source: RuntimeViewModelCell,
    ) -> bool {
        let Some(RuntimeScriptedDataConverterDataBindOccurrence::Context { retained_bind, .. }) =
            self.inputs
                .get_mut(input_index)
                .and_then(|input| input.data_binds.get_mut(data_bind_index))
        else {
            return false;
        };
        retained_bind.set_source(source);
        true
    }

    #[cfg(test)]
    pub(crate) fn data_bind_order_for_test(&self) -> &[(usize, usize)] {
        &self.data_bind_order
    }

    #[cfg(test)]
    pub(crate) fn cached_value_for_test(&self) -> Option<&RuntimeDataBindGraphValue> {
        self.cached_value.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn data_bind_metadata_for_test(
        &self,
    ) -> Vec<Vec<(u32, RuntimeScriptInputTargetProperty, u64, u32)>> {
        self.inputs
            .iter()
            .map(|input| {
                input
                    .data_binds
                    .iter()
                    .map(|binding| match binding {
                        RuntimeScriptedDataConverterDataBindOccurrence::Inert {
                            property_key,
                            target_property,
                            flags,
                            converter_id,
                        } => (*property_key, *target_property, *flags, *converter_id),
                        RuntimeScriptedDataConverterDataBindOccurrence::Context {
                            property_key,
                            target_property,
                            flags,
                            converter_id,
                            ..
                        } => (*property_key, *target_property, *flags, *converter_id),
                    })
                    .collect()
            })
            .collect()
    }

    #[cfg(test)]
    pub(crate) fn set_cached_value_for_test(&mut self, value: RuntimeDataBindGraphValue) {
        self.cached_value = Some(value);
    }

    /// `ScriptedDataConverter` inherits base `DataConverter::unbind`, so its
    /// one authored-order `m_dataBinds` list releases every custom-input
    /// source. Attached converters on those cloned binds do not survive
    /// `DataBindBase::copy` at the pin and therefore have no recursive unbind
    /// here (`data_converter.cpp:32`; `scripted_data_converter.cpp:235-269`).
    pub(crate) fn unbind_sources(&mut self) {
        for input in &mut self.inputs {
            for binding in &mut input.data_binds {
                if let RuntimeScriptedDataConverterDataBindOccurrence::Context {
                    retained_bind,
                    ..
                } = binding
                {
                    retained_bind.clear_source();
                }
            }
        }
    }

    pub(crate) fn report_input_source_dirt_to(
        &mut self,
        input_index: usize,
        data_bind_index: usize,
        queue: &RuntimeCellNotificationQueue,
        occurrence_index: usize,
    ) -> bool {
        let Some(RuntimeScriptedDataConverterDataBindOccurrence::Context { retained_bind, .. }) =
            self.inputs
                .get_mut(input_index)
                .and_then(|input| input.data_binds.get_mut(data_bind_index))
        else {
            return false;
        };
        retained_bind.report_source_dirt_to(queue, occurrence_index);
        true
    }

    pub(crate) fn mark_input_target_changed(
        &mut self,
        input_index: usize,
        data_bind_index: usize,
    ) -> bool {
        let Some(RuntimeScriptedDataConverterDataBindOccurrence::Context { retained_bind, .. }) =
            self.inputs
                .get_mut(input_index)
                .and_then(|input| input.data_binds.get_mut(data_bind_index))
        else {
            return false;
        };
        retained_bind.mark_target_changed();
        true
    }

    pub(crate) fn bind_sources(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        file: &nuxie_binary::RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        explicit_rebind: bool,
    ) {
        for &(input_index, data_bind_index) in &self.data_bind_order {
            let Some(input) = self.inputs.get_mut(input_index) else {
                continue;
            };
            let Some(definition) = definitions.get_mut(input_index) else {
                continue;
            };
            let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
                continue;
            };
            let Some(definition) = definition.data_binds.get_mut(data_bind_index) else {
                continue;
            };
            let RuntimeScriptedDataConverterDataBindOccurrence::Context {
                source_path,
                name_based,
                retained_bind,
                ..
            } = binding
            else {
                continue;
            };
            if !matches!(
                definition,
                RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
            ) {
                continue;
            }
            let source_cell = source_path.as_deref().and_then(|source_path| {
                context
                    .property_path_for_context_source_path_with_persistent_resolver(
                        file,
                        &[],
                        source_path,
                        *name_based,
                    )
                    .and_then(|property_path| context.cell_by_property_path(&property_path))
            });
            bind_resolved_source(retained_bind, source_cell, explicit_rebind);
        }
    }

    pub(crate) fn bind_sources_from_data_context(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        file: &nuxie_binary::RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        explicit_rebind: bool,
    ) {
        for &(input_index, data_bind_index) in &self.data_bind_order {
            let Some(input) = self.inputs.get_mut(input_index) else {
                continue;
            };
            let Some(definition) = definitions.get_mut(input_index) else {
                continue;
            };
            let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
                continue;
            };
            let Some(definition) = definition.data_binds.get_mut(data_bind_index) else {
                continue;
            };
            let RuntimeScriptedDataConverterDataBindOccurrence::Context {
                source_path,
                name_based,
                retained_bind,
                ..
            } = binding
            else {
                continue;
            };
            if !matches!(
                definition,
                RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
            ) {
                continue;
            }
            let source_cell = source_path.as_deref().and_then(|source_path| {
                data_context.resolve_instance(&mut |_, context, scope_path| {
                    let property_path = context
                        .property_path_for_context_source_path_with_persistent_resolver(
                            file,
                            scope_path,
                            source_path,
                            *name_based,
                        )?;
                    context.cell_by_property_path(&property_path)
                })
            });
            bind_resolved_source(retained_bind, source_cell, explicit_rebind);
        }
    }

    pub(crate) fn bind_input_source(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        input_index: usize,
        data_bind_index: usize,
        file: &nuxie_binary::RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        let Some(input) = self.inputs.get_mut(input_index) else {
            return false;
        };
        let Some(definition) = definitions.get_mut(input_index) else {
            return false;
        };
        let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
            return false;
        };
        let Some(definition) = definition.data_binds.get_mut(data_bind_index) else {
            return false;
        };
        let RuntimeScriptedDataConverterDataBindOccurrence::Context {
            source_path,
            name_based,
            retained_bind,
            ..
        } = binding
        else {
            return false;
        };
        if !matches!(
            definition,
            RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
        ) {
            return false;
        }
        let source_cell = source_path.as_deref().and_then(|source_path| {
            context
                .property_path_for_context_source_path_with_persistent_resolver(
                    file,
                    &[],
                    source_path,
                    *name_based,
                )
                .and_then(|property_path| context.cell_by_property_path(&property_path))
        });
        bind_resolved_source(retained_bind, source_cell, true);
        true
    }

    pub(crate) fn bind_input_source_from_data_context(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        input_index: usize,
        data_bind_index: usize,
        file: &nuxie_binary::RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
    ) -> bool {
        let Some(input) = self.inputs.get_mut(input_index) else {
            return false;
        };
        let Some(definition) = definitions.get_mut(input_index) else {
            return false;
        };
        let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
            return false;
        };
        let Some(definition) = definition.data_binds.get_mut(data_bind_index) else {
            return false;
        };
        let RuntimeScriptedDataConverterDataBindOccurrence::Context {
            source_path,
            name_based,
            retained_bind,
            ..
        } = binding
        else {
            return false;
        };
        if !matches!(
            definition,
            RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
        ) {
            return false;
        }
        let source_cell = source_path.as_deref().and_then(|source_path| {
            data_context.resolve_instance(&mut |_, context, scope_path| {
                let property_path = context
                    .property_path_for_context_source_path_with_persistent_resolver(
                        file,
                        scope_path,
                        source_path,
                        *name_based,
                    )?;
                context.cell_by_property_path(&property_path)
            })
        });
        bind_resolved_source(retained_bind, source_cell, true);
        true
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_input<F>(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        input_index: usize,
        data_bind_index: usize,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        file: &nuxie_binary::RuntimeFile,
        _context: &RuntimeOwnedViewModelInstance,
        apply: &mut F,
    ) -> Result<bool, ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        let Some(input) = self.inputs.get_mut(input_index) else {
            return Ok(false);
        };
        let Some(definition) = definitions.get_mut(input_index) else {
            return Ok(false);
        };
        let kind = input.kind;
        let properties = &mut input.properties;
        let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
            return Ok(false);
        };
        let Some(definition) = definition.data_binds.get_mut(data_bind_index) else {
            return Ok(false);
        };
        let RuntimeScriptedDataConverterDataBindOccurrence::Context {
            target_property,
            flags,
            retained_bind,
            last_target,
            ..
        } = binding
        else {
            return Ok(false);
        };
        if !matches!(
            definition,
            RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
        ) {
            return Ok(false);
        }
        // Steady updates read the source object retained during bind. C++
        // resolves `sourcePath` only in `bindFromContext`; it does not relink
        // the authored path on every converter update.
        let Some(source) = retained_bind
            .source()
            .and_then(|source| runtime_graph_value_from_bound_cell(&source.value()))
        else {
            return Ok(false);
        };
        update_one_input_binding(
            kind,
            properties,
            *target_property,
            *flags,
            retained_bind,
            last_target,
            source,
            owner_instance,
            file,
            apply,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn update_input_from_data_context<F>(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        input_index: usize,
        data_bind_index: usize,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        file: &nuxie_binary::RuntimeFile,
        _data_context: &RuntimeOwnedDataContext,
        apply: &mut F,
    ) -> Result<bool, ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        let Some(input) = self.inputs.get_mut(input_index) else {
            return Ok(false);
        };
        let Some(definition) = definitions.get_mut(input_index) else {
            return Ok(false);
        };
        let kind = input.kind;
        let properties = &mut input.properties;
        let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
            return Ok(false);
        };
        let Some(definition) = definition.data_binds.get_mut(data_bind_index) else {
            return Ok(false);
        };
        let RuntimeScriptedDataConverterDataBindOccurrence::Context {
            target_property,
            flags,
            retained_bind,
            last_target,
            ..
        } = binding
        else {
            return Ok(false);
        };
        if !matches!(
            definition,
            RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
        ) {
            return Ok(false);
        }
        let Some(source) = retained_bind
            .source()
            .and_then(|source| runtime_graph_value_from_bound_cell(&source.value()))
        else {
            return Ok(false);
        };
        update_one_input_binding(
            kind,
            properties,
            *target_property,
            *flags,
            retained_bind,
            last_target,
            source,
            owner_instance,
            file,
            apply,
            false,
        )
    }

    /// Public `DataBindContainer::updateDataBinds(true)` for one cloned
    /// ScriptedDataConverter custom-input bind.
    ///
    /// `DataConverter::copy` deliberately leaves the clone's converter
    /// pointer null, so this owner reconciles the raw Core ScriptInput value
    /// in the authored favor order. The containing converter's
    /// `reverseConvert` applies only to its parent DataBind, not to these
    /// custom-input binds (`data_converter.cpp:59-69`;
    /// `scripted_data_converter.cpp:235-267`).
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn public_update_input<F>(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        input_index: usize,
        data_bind_index: usize,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        file: &nuxie_binary::RuntimeFile,
        apply_target_to_source: bool,
        apply: &mut F,
    ) -> Result<bool, ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        let Some(input) = self.inputs.get_mut(input_index) else {
            return Ok(false);
        };
        let Some(definition) = definitions.get(input_index) else {
            return Ok(false);
        };
        let Some(binding) = input.data_binds.get_mut(data_bind_index) else {
            return Ok(false);
        };
        let Some(definition) = definition.data_binds.get(data_bind_index) else {
            return Ok(false);
        };
        let RuntimeScriptedDataConverterDataBindOccurrence::Context {
            target_property,
            flags,
            retained_bind,
            last_target,
            ..
        } = binding
        else {
            return Ok(false);
        };
        if !matches!(
            definition,
            RuntimeScriptedDataConverterDataBindDefinition::Context { .. }
        ) {
            return Ok(false);
        }

        retained_bind.collect_source_dirt();
        let wants_target_to_source = apply_target_to_source
            && data_bind_flags_apply_target_to_source(*flags)
            && retained_bind
                .pending_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS_TARGET);
        let source_runs_first = retained_bind.source_to_target_runs_first();

        if wants_target_to_source && !source_runs_first {
            update_script_input_source_from_target(
                &input.properties,
                *target_property,
                retained_bind,
            );
        }

        let has_source_dirt = retained_bind
            .pending_dirt()
            .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS);
        let target_changed = if has_source_dirt
            && let Some(source) = retained_bind
                .source()
                .and_then(|source| runtime_graph_value_from_bound_cell(&source.value()))
        {
            update_one_input_binding(
                input.kind,
                &mut input.properties,
                *target_property,
                *flags,
                retained_bind,
                last_target,
                source,
                owner_instance,
                file,
                apply,
                true,
            )?
        } else {
            retained_bind.take_pending_source_dirt();
            false
        };

        if wants_target_to_source && source_runs_first {
            update_script_input_source_from_target(
                &input.properties,
                *target_property,
                retained_bind,
            );
        } else if !wants_target_to_source {
            // C++ clears the complete dirt mask after the update pass even
            // when public target-to-source application was not requested.
            retained_bind.take_target_dirt();
        }
        Ok(target_changed)
    }

    pub(crate) fn update_inputs<F>(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        file: &nuxie_binary::RuntimeFile,
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
        let order = self.data_bind_order.clone();
        for (input_index, data_bind_index) in order {
            let _ = self.update_input(
                definitions,
                input_index,
                data_bind_index,
                owner_instance,
                file,
                context,
                apply,
            )?;
        }
        Ok(())
    }

    pub(crate) fn update_inputs_from_data_context<F>(
        &mut self,
        definitions: &mut [RuntimeScriptedDataConverterInputDefinition],
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        file: &nuxie_binary::RuntimeFile,
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
        let order = self.data_bind_order.clone();
        for (input_index, data_bind_index) in order {
            let _ = self.update_input_from_data_context(
                definitions,
                input_index,
                data_bind_index,
                owner_instance,
                file,
                data_context,
                apply,
            )?;
        }
        Ok(())
    }

    pub(crate) fn apply_conversion(
        &mut self,
        instance: Option<&RuntimeScriptInstanceHandle>,
        serialized_implemented_methods: u32,
        method: ScriptDataConverterMethod,
        value: &RuntimeDataBindGraphValue,
    ) -> Result<Option<RuntimeDataBindGraphValue>, ScriptError> {
        let methods =
            RuntimeScriptImplementedMethods::from_serialized(serialized_implemented_methods);
        let implemented = match method {
            ScriptDataConverterMethod::Convert => methods.data_converts(),
            ScriptDataConverterMethod::ReverseConvert => methods.data_reverse_converts(),
        };
        if !implemented {
            return Ok(Some(value.clone()));
        }
        let Some(instance) = instance else {
            return Ok(Some(value.clone()));
        };
        if !instance.borrow_mut().script_lifetime_valid() {
            // C++ conversion is inert while `m_self == 0` after init
            // false/error/missing-requested-data. A hydration-prerequisite
            // miss does not invalidate that lifetime
            // (`scripted_object.cpp:277-303,399-417`;
            // `scripted_data_converter.cpp:96-167`).
            return Ok(Some(value.clone()));
        }
        let input = match value {
            RuntimeDataBindGraphValue::Number(value) => {
                Some(ScriptValue::Number(f64::from(*value)))
            }
            RuntimeDataBindGraphValue::Boolean(value) => Some(ScriptValue::Bool(*value)),
            RuntimeDataBindGraphValue::String(value) => Some(ScriptValue::CoreString(
                ScriptCoreString::from_bytes(value.clone()),
            )),
            RuntimeDataBindGraphValue::Color(value) => Some(ScriptValue::Color(*value)),
            // C++ cannot push unsupported DataValue subclasses and falls
            // through to its existing `m_dataValue`. The first such call
            // creates a concrete base `DataValue` sentinel rather than
            // returning null, so a containing converter group continues to
            // its later authored items (`scripted_data_converter.cpp:96-147`;
            // `data_converter_group.cpp:21-32`).
            _ => None,
        };
        let result = instance
            .borrow_mut()
            .call_optional_data_converter(method, input);
        let converted = match result {
            Ok(crate::ScriptDataConverterOptionalCall::Missing) => {
                // C++ tests the Lua field before attempting to push the
                // input. A missing/non-function method is pass-through even
                // for a DataValue subclass Lua cannot represent.
                return Ok(Some(value.clone()));
            }
            Ok(crate::ScriptDataConverterOptionalCall::UnsupportedInput) => None,
            Ok(crate::ScriptDataConverterOptionalCall::Returned(ScriptValue::Number(value))) => {
                Some(RuntimeDataBindGraphValue::Number(value as f32))
            }
            Ok(crate::ScriptDataConverterOptionalCall::Returned(ScriptValue::Bool(value))) => {
                Some(RuntimeDataBindGraphValue::Boolean(value))
            }
            Ok(crate::ScriptDataConverterOptionalCall::Returned(ScriptValue::String(value))) => {
                Some(RuntimeDataBindGraphValue::String(value.into_bytes()))
            }
            Ok(crate::ScriptDataConverterOptionalCall::Returned(ScriptValue::CoreString(
                value,
            ))) => Some(RuntimeDataBindGraphValue::String(value.into_bytes())),
            Ok(crate::ScriptDataConverterOptionalCall::Returned(ScriptValue::Color(value))) => {
                Some(RuntimeDataBindGraphValue::Color(value))
            }
            Ok(crate::ScriptDataConverterOptionalCall::Returned(_)) => None,
            Err(error) if error.resource_code().is_some() => return Err(error),
            Err(_) => None,
        };
        if let Some(converted) = converted {
            self.cached_value = Some(converted.clone());
            Ok(Some(converted))
        } else {
            let cached = self
                .cached_value
                .get_or_insert(RuntimeDataBindGraphValue::Untyped);
            Ok(Some(cached.clone()))
        }
    }
}

fn bind_resolved_source(
    retained_bind: &mut RuntimeRetainedDataBind,
    source_cell: Option<RuntimeViewModelCell>,
    force_reconcile: bool,
) {
    let source_resolved = source_cell.is_some();
    let source_rebound = match (retained_bind.source(), source_cell.as_ref()) {
        (Some(current), Some(next)) => !current.ptr_eq(next),
        // An explicit DataBindContext::bindFromContext call with no resolved
        // source still executes unbind(), including null -> null. Preserve
        // that branch so converter-owned subscriptions cannot survive an
        // unresolved rebind (`data_bind_context.cpp:56-89`;
        // `data_bind.cpp:354-369`).
        (None, None) => force_reconcile,
        _ => true,
    };
    if source_rebound {
        retained_bind.clear_source();
        if let Some(source_cell) = source_cell {
            retained_bind.set_source(source_cell);
        }
    }
    // Pinned DataBindContext::bindFromContext reconciles only a resolved
    // source: a new/non-null source calls bind(), and the same non-null source
    // adds reconcile dirt directly. A missing source takes unbind(), including
    // null -> null, but does not enqueue fabricated reconcile dirt
    // (`data_bind_context.cpp:56-89`).
    if source_resolved && (source_rebound || force_reconcile) {
        retained_bind.mark_rebind_reconcile();
    }
}

#[allow(clippy::too_many_arguments)]
fn update_one_input_binding<F>(
    kind: ScriptListenerInputKind,
    properties: &mut RuntimeScriptInputProperties,
    target_property: RuntimeScriptInputTargetProperty,
    flags: u64,
    retained_bind: &mut RuntimeRetainedDataBind,
    last_target: &mut Option<RuntimeDataBindGraphValue>,
    source: RuntimeDataBindGraphValue,
    owner_instance: Option<&RuntimeScriptInstanceHandle>,
    file: &nuxie_binary::RuntimeFile,
    apply: &mut F,
    preserve_target_dirt: bool,
) -> Result<bool, ScriptError>
where
    F: FnMut(
        &RuntimeScriptInstanceHandle,
        &ScriptCoreString,
        RuntimeScriptedListenerBoundValue,
    ) -> Result<(), ScriptError>,
{
    retained_bind.collect_source_dirt();
    // The ordinary updateDataBinds(false) boundary discards target dirt.
    // Public updateDataBinds(true) preserves it until the favored reverse
    // direction has run.
    if !preserve_target_dirt {
        retained_bind.take_target_dirt();
    }
    if !retained_bind.take_pending_source_dirt() || !data_bind_flags_apply_source_to_target(flags) {
        return Ok(false);
    }

    // `DataBindBase::copy` does not copy the resolved `m_dataConverter`
    // pointer. The cloned custom-input occurrence therefore derives its
    // ContextValue subclass from the live raw source type. An Artboard source
    // takes the referencer path and leaves generated `artboardId` untouched.
    let target_apply = if kind == ScriptListenerInputKind::Artboard
        && let RuntimeDataBindGraphValue::Artboard(artboard_id) = &source
    {
        properties.apply_artboard_source(file, *artboard_id)
    } else {
        properties.apply_target(file, kind, target_property, source)
    };
    let target_changed = matches!(
        target_apply,
        RuntimeScriptInputTargetApply::ChangedWithoutTableProjection
            | RuntimeScriptInputTargetApply::ChangedWithTableProjection
    );
    *last_target = properties.value().cloned();
    if target_apply != RuntimeScriptInputTargetApply::ChangedWithTableProjection {
        return Ok(target_changed);
    }
    let Some(owner_instance) = owner_instance else {
        return Ok(target_changed);
    };
    let Some(projected_target) = properties.projection_value(kind) else {
        return Ok(target_changed);
    };
    let value = scripted_input_bound_value(kind, projected_target)?;
    apply(owner_instance, properties.name(), value)?;
    Ok(target_changed)
}

fn update_script_input_source_from_target(
    properties: &RuntimeScriptInputProperties,
    target_property: RuntimeScriptInputTargetProperty,
    retained_bind: &mut RuntimeRetainedDataBind,
) -> bool {
    let source_value = retained_bind.source().map(RuntimeViewModelCell::value);
    let Some(target) = properties.target_value(target_property, source_value.as_ref()) else {
        retained_bind.take_target_dirt();
        return false;
    };
    let Some(value) = runtime_cell_value_from_graph_value(&target, source_value.as_ref()) else {
        retained_bind.take_target_dirt();
        return false;
    };
    retained_bind.update_source_binding_value(value)
}

fn scripted_input_bound_value(
    kind: ScriptListenerInputKind,
    value: RuntimeDataBindGraphValue,
) -> Result<RuntimeScriptedListenerBoundValue, ScriptError> {
    Ok(match (kind, value) {
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
                "scripted converter input kind {kind:?} received incompatible bound value {value:?}",
            )));
        }
    })
}

pub(crate) fn inits(serialized_implemented_methods: u32) -> bool {
    RuntimeScriptImplementedMethods::from_serialized(serialized_implemented_methods).inits()
}

pub(crate) fn advance(
    instance: Option<&RuntimeScriptInstanceHandle>,
    serialized_implemented_methods: u32,
    elapsed_seconds: f32,
    host: &mut dyn ScriptHost,
) -> Result<bool, ScriptError> {
    if elapsed_seconds == 0.0
        || !RuntimeScriptImplementedMethods::from_serialized(serialized_implemented_methods)
            .advances()
    {
        return Ok(false);
    }
    let Some(instance) = instance else {
        return Ok(false);
    };
    if !instance.borrow_mut().script_lifetime_valid() {
        return Ok(false);
    }
    match instance
        .borrow_mut()
        .call_advance_truthy(elapsed_seconds, host)
    {
        // Lua truthiness: only nil and false are false; zero, empty strings,
        // tables, functions, userdata, and threads all request another
        // advance.
        Ok(needs_advance) => Ok(needs_advance),
        Err(error) if error.resource_code().is_some() => Err(error),
        Err(_) => Ok(false),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScriptMethod;
    use crate::data_bind_graph::DATA_BIND_FLAG_DIRECTION_TO_SOURCE;
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    #[derive(Debug)]
    struct ConverterProbe {
        fail: Rc<Cell<bool>>,
        has_convert: Rc<Cell<bool>>,
        lifetime_valid: bool,
        advance_result: ScriptValue,
        calls: Rc<RefCell<Vec<&'static str>>>,
    }

    impl crate::ScriptInstance for ConverterProbe {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance && self.has_convert.get())
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            self.calls.borrow_mut().push("advance");
            Ok(self.advance_result.clone())
        }

        fn has_data_converter_method(
            &self,
            _method: ScriptDataConverterMethod,
        ) -> Result<bool, ScriptError> {
            Ok(self.has_convert.get())
        }

        fn call_data_converter(
            &mut self,
            method: ScriptDataConverterMethod,
            value: ScriptValue,
        ) -> Result<ScriptValue, ScriptError> {
            self.calls.borrow_mut().push(match method {
                ScriptDataConverterMethod::Convert => "convert",
                ScriptDataConverterMethod::ReverseConvert => "reverseConvert",
            });
            if self.fail.get() {
                return Err(ScriptError::new("ordinary conversion failure"));
            }
            let ScriptValue::Number(value) = value else {
                return Ok(ScriptValue::Nil);
            };
            Ok(ScriptValue::Number(value + 1.0))
        }

        fn call_data_converter_if_present(
            &mut self,
            method: ScriptDataConverterMethod,
            value: ScriptValue,
        ) -> Result<Option<ScriptValue>, ScriptError> {
            if !self.has_convert.get() {
                return Ok(None);
            }
            self.call_data_converter(method, value).map(Some)
        }

        fn call_advance_truthy(
            &mut self,
            _elapsed_seconds: f32,
            _host: &mut dyn ScriptHost,
        ) -> Result<bool, ScriptError> {
            if !self.has_convert.get() {
                return Ok(false);
            }
            self.calls.borrow_mut().push("advance");
            Ok(!matches!(
                self.advance_result,
                ScriptValue::Nil | ScriptValue::Bool(false)
            ))
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

    fn probe_handle(
        fail: Rc<Cell<bool>>,
        has_convert: bool,
        advance_result: ScriptValue,
        calls: Rc<RefCell<Vec<&'static str>>>,
    ) -> RuntimeScriptInstanceHandle {
        RuntimeScriptInstanceHandle::new(Box::new(ConverterProbe {
            fail,
            has_convert: Rc::new(Cell::new(has_convert)),
            lifetime_valid: true,
            advance_result,
            calls,
        }))
    }

    #[derive(Debug)]
    struct AtomicOptionalConverterProbe {
        calls: Rc<RefCell<Vec<(ScriptDataConverterMethod, bool)>>>,
    }

    impl crate::ScriptInstance for AtomicOptionalConverterProbe {
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

        fn call_data_converter(
            &mut self,
            _method: ScriptDataConverterMethod,
            _value: ScriptValue,
        ) -> Result<ScriptValue, ScriptError> {
            panic!("the converter runtime must use the atomic optional operation")
        }

        fn call_data_converter_if_present(
            &mut self,
            _method: ScriptDataConverterMethod,
            _value: ScriptValue,
        ) -> Result<Option<ScriptValue>, ScriptError> {
            panic!("the converter runtime must not perform a second method lookup")
        }

        fn has_data_converter_method(
            &self,
            _method: ScriptDataConverterMethod,
        ) -> Result<bool, ScriptError> {
            panic!("the converter runtime must not preflight with a separate lookup")
        }

        fn call_optional_data_converter(
            &mut self,
            method: ScriptDataConverterMethod,
            value: Option<ScriptValue>,
        ) -> Result<crate::ScriptDataConverterOptionalCall, ScriptError> {
            self.calls.borrow_mut().push((method, value.is_some()));
            Ok(match value {
                Some(value) => crate::ScriptDataConverterOptionalCall::Returned(value),
                None => crate::ScriptDataConverterOptionalCall::UnsupportedInput,
            })
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    #[test]
    fn conversion_uses_one_optional_lookup_before_input_push() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let handle = RuntimeScriptInstanceHandle::new(Box::new(AtomicOptionalConverterProbe {
            calls: Rc::clone(&calls),
        }));
        let mut state = RuntimeScriptedDataConverterState::default();

        for (method, mask) in [
            (
                ScriptDataConverterMethod::Convert,
                RuntimeScriptImplementedMethods::DATA_CONVERT,
            ),
            (
                ScriptDataConverterMethod::ReverseConvert,
                RuntimeScriptImplementedMethods::DATA_REVERSE_CONVERT,
            ),
        ] {
            assert_eq!(
                state
                    .apply_conversion(
                        Some(&handle),
                        mask,
                        method,
                        &RuntimeDataBindGraphValue::Number(3.0),
                    )
                    .unwrap(),
                Some(RuntimeDataBindGraphValue::Number(3.0))
            );
            assert_eq!(
                state
                    .apply_conversion(
                        Some(&handle),
                        mask,
                        method,
                        &RuntimeDataBindGraphValue::Asset(7),
                    )
                    .unwrap(),
                Some(RuntimeDataBindGraphValue::Number(3.0)),
                "a present method with an unsupported input retains the occurrence cache"
            );
        }

        assert_eq!(
            &*calls.borrow(),
            &[
                (ScriptDataConverterMethod::Convert, true),
                (ScriptDataConverterMethod::Convert, false),
                (ScriptDataConverterMethod::ReverseConvert, true),
                (ScriptDataConverterMethod::ReverseConvert, false),
            ]
        );
    }

    #[test]
    fn conversion_and_advance_are_inert_after_cpp_table_disposal() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let handle = RuntimeScriptInstanceHandle::new(Box::new(ConverterProbe {
            fail: Rc::new(Cell::new(false)),
            has_convert: Rc::new(Cell::new(true)),
            lifetime_valid: false,
            advance_result: ScriptValue::Bool(true),
            calls: Rc::clone(&calls),
        }));
        let input = RuntimeDataBindGraphValue::Number(3.0);
        assert_eq!(
            RuntimeScriptedDataConverterState::default()
                .apply_conversion(
                    Some(&handle),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::Convert,
                    &input,
                )
                .unwrap(),
            Some(input),
        );
        assert!(
            !advance(
                Some(&handle),
                RuntimeScriptImplementedMethods::ADVANCE,
                0.25,
                &mut crate::NoopScriptHost,
            )
            .unwrap()
        );
        assert!(
            calls.borrow().is_empty(),
            "a disposed C++ m_self cannot receive convert or advance"
        );
    }

    fn number_input_definition(flags: u64) -> RuntimeScriptedDataConverterInputDefinition {
        RuntimeScriptedDataConverterInputDefinition {
            input_global_id: 10,
            kind: ScriptListenerInputKind::Number,
            properties: RuntimeScriptInputProperties::for_test(
                "value",
                u32::MAX,
                Some(RuntimeDataBindGraphValue::Number(0.0)),
            ),
            data_binds: vec![RuntimeScriptedDataConverterDataBindDefinition::Context {
                authored_order: 20,
                source_path: Some(vec![0, 0]),
                name_based: false,
                property_key: crate::properties::property_key_for_name(
                    "ScriptInputNumber",
                    "propertyValue",
                )
                .map(u32::from)
                .expect("ScriptInputNumber.propertyValue"),
                target_property: RuntimeScriptInputTargetProperty::Value,
                flags,
                converter_id: u32::MAX,
            }],
        }
    }

    #[test]
    fn conversion_mask_missing_method_and_occurrence_cache_match_cpp() {
        let fail = Rc::new(Cell::new(true));
        let calls = Rc::new(RefCell::new(Vec::new()));
        let handle = probe_handle(
            Rc::clone(&fail),
            true,
            ScriptValue::Bool(false),
            Rc::clone(&calls),
        );
        let mut state = RuntimeScriptedDataConverterState::default();
        let input = RuntimeDataBindGraphValue::Number(3.0);

        assert_eq!(
            state
                .apply_conversion(Some(&handle), 0, ScriptDataConverterMethod::Convert, &input)
                .unwrap(),
            Some(input.clone()),
            "an absent optional-method bit passes through without consulting the live table"
        );
        assert!(calls.borrow().is_empty());

        assert_eq!(
            state
                .apply_conversion(
                    Some(&handle),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::Convert,
                    &input,
                )
                .unwrap(),
            Some(RuntimeDataBindGraphValue::Untyped),
            "the first protected-call failure creates C++'s non-null base DataValue sentinel"
        );
        fail.set(false);
        assert_eq!(
            state
                .apply_conversion(
                    Some(&handle),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::Convert,
                    &RuntimeDataBindGraphValue::Number(4.0),
                )
                .unwrap(),
            Some(RuntimeDataBindGraphValue::Number(5.0))
        );
        fail.set(true);
        assert_eq!(
            state
                .apply_conversion(
                    Some(&handle),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::Convert,
                    &RuntimeDataBindGraphValue::Number(9.0),
                )
                .unwrap(),
            Some(RuntimeDataBindGraphValue::Number(5.0)),
            "a later protected-call failure replays this occurrence's prior typed cache"
        );
        assert_eq!(
            state
                .apply_conversion(
                    Some(&handle),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::Convert,
                    &RuntimeDataBindGraphValue::Asset(7),
                )
                .unwrap(),
            Some(RuntimeDataBindGraphValue::Number(5.0)),
            "an unsupported input kind also retains the prior C++ m_dataValue"
        );

        let missing_calls = Rc::new(RefCell::new(Vec::new()));
        let missing = probe_handle(
            Rc::new(Cell::new(false)),
            false,
            ScriptValue::Bool(false),
            Rc::clone(&missing_calls),
        );
        assert_eq!(
            RuntimeScriptedDataConverterState::default()
                .apply_conversion(
                    Some(&missing),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::Convert,
                    &input,
                )
                .unwrap(),
            Some(input),
            "legacy all-bits files still pass through a non-function field"
        );
        assert!(missing_calls.borrow().is_empty());

        for (method, method_mask) in [
            (
                ScriptDataConverterMethod::Convert,
                RuntimeScriptImplementedMethods::DATA_CONVERT,
            ),
            (
                ScriptDataConverterMethod::ReverseConvert,
                RuntimeScriptImplementedMethods::DATA_REVERSE_CONVERT,
            ),
        ] {
            let available = Rc::new(Cell::new(false));
            let fail_after_install = Rc::new(Cell::new(false));
            let optional_calls = Rc::new(RefCell::new(Vec::new()));
            let optional = RuntimeScriptInstanceHandle::new(Box::new(ConverterProbe {
                fail: Rc::clone(&fail_after_install),
                has_convert: Rc::clone(&available),
                lifetime_valid: true,
                advance_result: ScriptValue::Bool(false),
                calls: Rc::clone(&optional_calls),
            }));
            let mut optional_state = RuntimeScriptedDataConverterState::default();
            let first = RuntimeDataBindGraphValue::Number(3.0);
            assert_eq!(
                optional_state
                    .apply_conversion(Some(&optional), method_mask, method, &first)
                    .unwrap(),
                Some(first),
                "a missing/non-function callback passes through without populating C++ m_dataValue"
            );
            assert!(optional_calls.borrow().is_empty());

            available.set(true);
            fail_after_install.set(true);
            assert_eq!(
                optional_state
                    .apply_conversion(
                        Some(&optional),
                        method_mask,
                        method,
                        &RuntimeDataBindGraphValue::Number(9.0),
                    )
                    .unwrap(),
                Some(RuntimeDataBindGraphValue::Untyped),
                "the later failing callback creates the base sentinel instead of replaying the earlier pass-through input"
            );
        }

        let reverse_calls = Rc::new(RefCell::new(Vec::new()));
        let reverse = probe_handle(
            Rc::new(Cell::new(false)),
            true,
            ScriptValue::Bool(false),
            Rc::clone(&reverse_calls),
        );
        let mut reverse_state = RuntimeScriptedDataConverterState::default();
        assert_eq!(
            reverse_state
                .apply_conversion(
                    Some(&reverse),
                    RuntimeScriptImplementedMethods::DATA_CONVERT,
                    ScriptDataConverterMethod::ReverseConvert,
                    &RuntimeDataBindGraphValue::Number(3.0),
                )
                .unwrap(),
            Some(RuntimeDataBindGraphValue::Number(3.0)),
            "the convert bit does not enable the independent reverseConvert method"
        );
        assert!(reverse_calls.borrow().is_empty());
        assert_eq!(
            reverse_state
                .apply_conversion(
                    Some(&reverse),
                    RuntimeScriptImplementedMethods::DATA_REVERSE_CONVERT,
                    ScriptDataConverterMethod::ReverseConvert,
                    &RuntimeDataBindGraphValue::Number(3.0),
                )
                .unwrap(),
            Some(RuntimeDataBindGraphValue::Number(4.0))
        );
        assert_eq!(&*reverse_calls.borrow(), &["reverseConvert"]);

        assert!(!inits(0));
        assert!(!inits(RuntimeScriptImplementedMethods::DATA_CONVERT));
        assert!(inits(RuntimeScriptImplementedMethods::INIT));
    }

    #[test]
    fn advance_mask_and_lua_truthiness_match_cpp() {
        fn run(mask: u32, result: ScriptValue, elapsed: f32) -> (bool, usize) {
            let calls = Rc::new(RefCell::new(Vec::new()));
            let handle = probe_handle(Rc::new(Cell::new(false)), true, result, Rc::clone(&calls));
            let keep_going =
                advance(Some(&handle), mask, elapsed, &mut crate::NoopScriptHost).unwrap();
            let call_count = calls.borrow().len();
            (keep_going, call_count)
        }

        assert_eq!(run(0, ScriptValue::Bool(true), 1.0), (false, 0));
        assert_eq!(
            run(
                RuntimeScriptImplementedMethods::ADVANCE,
                ScriptValue::Bool(true),
                0.0,
            ),
            (false, 0)
        );
        assert_eq!(
            run(
                RuntimeScriptImplementedMethods::ADVANCE,
                ScriptValue::Number(0.0),
                1.0,
            ),
            (true, 1),
            "Lua treats numeric zero as truthy"
        );
        assert_eq!(
            run(
                RuntimeScriptImplementedMethods::ADVANCE,
                ScriptValue::String(String::new()),
                1.0,
            ),
            (true, 1),
            "Lua treats an empty string as truthy"
        );
        assert_eq!(
            run(
                RuntimeScriptImplementedMethods::ADVANCE,
                ScriptValue::Bool(false),
                1.0,
            ),
            (false, 1)
        );

        let missing_calls = Rc::new(RefCell::new(Vec::new()));
        let missing = probe_handle(
            Rc::new(Cell::new(false)),
            false,
            ScriptValue::Bool(true),
            Rc::clone(&missing_calls),
        );
        assert!(
            !advance(
                Some(&missing),
                RuntimeScriptImplementedMethods::ADVANCE,
                1.0,
                &mut crate::NoopScriptHost,
            )
            .unwrap(),
            "an authored advance bit with no live function is inert"
        );
        assert!(
            missing_calls.borrow().is_empty(),
            "a missing/non-function field never reaches the protected call"
        );
    }

    #[test]
    fn inherited_unbind_releases_every_scripted_custom_input_source() {
        let definitions = [
            number_input_definition(0),
            number_input_definition(DATA_BIND_FLAG_DIRECTION_TO_SOURCE),
            number_input_definition(DATA_BIND_FLAG_ONCE),
        ];
        let definition = RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(
            definitions.to_vec(),
        );
        let mut state = RuntimeScriptedDataConverterState::from_definition(&definition);
        let sources = (0..definitions.len())
            .map(|index| {
                RuntimeViewModelCell::new(
                    crate::view_model_cell::RuntimeViewModelCellValue::Number(index as f32),
                )
            })
            .collect::<Vec<_>>();
        for (input, source) in state.inputs.iter_mut().zip(&sources) {
            let RuntimeScriptedDataConverterDataBindOccurrence::Context { retained_bind, .. } =
                &mut input.data_binds[0]
            else {
                panic!("expected context bind");
            };
            retained_bind.set_source(source.clone());
            assert!(
                retained_bind
                    .source()
                    .is_some_and(|bound| bound.ptr_eq(source))
            );
        }

        state.unbind_sources();

        for input in &state.inputs {
            let RuntimeScriptedDataConverterDataBindOccurrence::Context { retained_bind, .. } =
                &input.data_binds[0]
            else {
                panic!("expected context bind");
            };
            assert!(
                retained_bind.source().is_none(),
                "ScriptedDataConverter inherits DataConverter::unbind, so ToTarget, ToSource, and Once custom-input occurrences all clear their retained source (`data_converter.cpp:32`; `scripted_data_converter.cpp:235-269`)"
            );
        }
    }

    #[test]
    fn same_value_rebind_rehomes_the_converter_input_source_cell() {
        let definitions = [number_input_definition(0)];
        let definition = RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(
            definitions.to_vec(),
        );
        let mut state = RuntimeScriptedDataConverterState::from_definition(&definition);
        let first = RuntimeViewModelCell::new(
            crate::view_model_cell::RuntimeViewModelCellValue::Number(3.0),
        );
        let second = RuntimeViewModelCell::new(
            crate::view_model_cell::RuntimeViewModelCellValue::Number(3.0),
        );
        let RuntimeScriptedDataConverterDataBindOccurrence::Context { retained_bind, .. } =
            &mut state.inputs[0].data_binds[0]
        else {
            panic!("expected converter-owned context bind");
        };

        bind_resolved_source(retained_bind, Some(first.clone()), false);
        assert!(
            retained_bind.take_pending_source_dirt(),
            "the first bind schedules the cloned custom input"
        );
        bind_resolved_source(retained_bind, Some(second.clone()), false);
        assert!(
            retained_bind
                .source()
                .is_some_and(|source| source.ptr_eq(&second)),
            "C++ retains the newly resolved cell identity even when its value is unchanged"
        );
        assert!(
            retained_bind.take_pending_source_dirt(),
            "a different source cell is a bind boundary, not an equal-value no-op"
        );

        assert!(
            first.set_value(crate::view_model_cell::RuntimeViewModelCellValue::Number(
                4.0
            ))
        );
        assert!(
            !retained_bind.collect_source_dirt(),
            "DataBind::clearSource unregisters the departed cell before the replacement bind"
        );
        assert!(
            second.set_value(crate::view_model_cell::RuntimeViewModelCellValue::Number(
                4.0
            ))
        );
        assert!(
            retained_bind.collect_source_dirt(),
            "only the replacement cell wakes this ScriptedDataConverter occurrence"
        );
        assert!(
            retained_bind.take_pending_source_dirt(),
            "the replacement notification is retained for the next authored update"
        );
        // `DataBindContext::bindFromContext` resolves and assigns a source
        // pointer at each explicit bind boundary; equality belongs to the
        // later target apply, not source ownership
        // (`data_bind_context.cpp:56-89`; `data_bind.cpp:245-280`).
    }

    #[test]
    fn unresolved_rebind_clears_source_without_inventing_reconcile_dirt() {
        let mut retained_bind = RuntimeRetainedDataBind::new(0, false);
        let source = RuntimeViewModelCell::new(
            crate::view_model_cell::RuntimeViewModelCellValue::Number(3.0),
        );

        bind_resolved_source(&mut retained_bind, Some(source), true);
        assert!(retained_bind.take_pending_source_dirt());
        retained_bind.take_target_dirt();

        bind_resolved_source(&mut retained_bind, None, true);
        assert!(retained_bind.source().is_none());
        assert!(
            retained_bind.pending_dirt().is_empty(),
            "Some->None is C++ DataBind::unbind(), not a reconcile"
        );

        bind_resolved_source(&mut retained_bind, None, true);
        assert!(
            retained_bind.pending_dirt().is_empty(),
            "an explicit unresolved None->None bind remains clean"
        );
    }
}
