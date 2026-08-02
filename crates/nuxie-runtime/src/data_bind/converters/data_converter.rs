//! Direct Rust owner for pinned C++
//! `src/data_bind/converters/data_converter.cpp`.
//!
//! Occurrence-owned `DataConverter::m_dataBinds`.
//!
//! Pinned C++ clones every converter-owned `DataBind` into the concrete
//! converter occurrence, in file order, and retargets the clone to that
//! occurrence (`src/data_bind/converters/data_converter.cpp:59-69`).
//! `DataConverterGroup` recursively owns a distinct clone for every authored
//! group item, while Formula and Scripted converters repair token/input
//! targets by the corresponding bind-list index
//! (`data_converter_group.cpp:48-61`;
//! `data_converter_formula.cpp:498-524`;
//! `scripted_data_converter.cpp:235-267`).
//!
//! This module is deliberately occurrence-local. Global object ids identify
//! authored definitions and therefore cannot identify duplicate GroupItem
//! clones or the same converter definition mounted in two live DataContexts.

use crate::artboard_data_bind::RuntimeOwnedDataContext;
use crate::data_bind_graph::{
    RuntimeDataBindGraphConverter, RuntimeDataBindGraphConverterState,
    data_bind_flags_apply_target_to_source,
};
use crate::retained_data_bind::{
    RuntimeConverterParentWake, RuntimeDataBindTarget, RuntimeRetainedDataBind,
};
use crate::scripting::{RuntimeScriptInstanceHandle, ScriptCoreString, ScriptError};
use crate::state_machine::RuntimeScriptedListenerBoundValue;
use crate::view_model::RuntimeOwnedViewModelInstance;
use crate::view_model_cell::{
    RuntimeCellNotificationQueue, RuntimeViewModelCell, RuntimeViewModelCellValue,
};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

const DATA_BIND_FLAG_ONCE: u64 = 1 << 2;

/// Immutable clone recipe for one converter occurrence.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeDataConverterDataBindDefinition {
    bindings: Vec<RuntimeDataConverterBindingDefinition>,
    children: Vec<Self>,
    /// Concrete C++ `ScriptedDataConverter` ownership survives even when the
    /// attached asset cannot be lowered to Rust's executable Scripted variant.
    detached_scripted_definition:
        Option<crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition>,
}

#[derive(Debug, Clone)]
enum RuntimeDataConverterBindingDefinition {
    Inert,
    Context {
        source_path: Option<Vec<u32>>,
        name_based: bool,
        property_key: u32,
        flags: u64,
        target: RuntimeDataConverterBindingTarget,
        initial_target: Option<RuntimeConverterPropertyValue>,
    },
}

/// The target repair performed after `DataConverter::copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RuntimeDataConverterBindingTarget {
    SelfProperty,
    FormulaToken {
        token_index: usize,
    },
    ScriptedInput {
        input_index: usize,
        data_bind_index: usize,
    },
}

/// Mutable owned-bind state for one concrete converter clone.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeDataConverterDataBindState {
    bindings: Vec<RuntimeDataConverterBindingState>,
    children: Vec<Self>,
    detached_scripted: Option<RuntimeDetachedScriptedDataConverterState>,
    dirty_queue: RuntimeCellNotificationQueue,
    processing: bool,
    active_dirty: BTreeSet<usize>,
    rejected_entries: usize,
    /// Occurrence-local equivalent of `DataConverter::m_parentDataBind`.
    parent_wake: Option<RuntimeConverterParentWake>,
    /// Generated fields belong to the converter occurrence, not to an
    /// individual DataBind. Keep fields that are not represented directly by
    /// `RuntimeDataBindGraphConverter` here so duplicate binds observe the
    /// same target identity.
    target_values:
        BTreeMap<(RuntimeDataConverterBindingTarget, u32), RuntimeConverterPropertyValue>,
}

/// Owner-neutral `DataConverter::bindFromContext` operation stream.
///
/// Script tables live above `nuxie-runtime`, so an occurrence cannot perform
/// the complete virtual call while holding a mutable graph borrow. Materialize
/// the immutable work list first, then let each owner tag it with its concrete
/// outer DataBind identity. The ordering is the C++ call stack: bind this
/// converter's own list, reinitialize Scripted and rebind its final custom
/// inputs, then visit Group children in authored order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeDataConverterBindStep {
    BindOwn {
        path: Vec<usize>,
    },
    Rehydrate {
        path: Vec<usize>,
        converter_global_id: u32,
        inits: bool,
    },
    RebindFinalInput {
        path: Vec<usize>,
        input_index: usize,
        data_bind_index: usize,
    },
}

pub(crate) fn runtime_data_converter_bind_steps(
    converter: &RuntimeDataBindGraphConverter,
) -> Vec<RuntimeDataConverterBindStep> {
    let mut steps = Vec::new();
    collect_runtime_data_converter_bind_steps(converter, &mut Vec::new(), &mut steps);
    steps
}

fn collect_runtime_data_converter_bind_steps(
    converter: &RuntimeDataBindGraphConverter,
    path: &mut Vec<usize>,
    steps: &mut Vec<RuntimeDataConverterBindStep>,
) {
    steps.push(RuntimeDataConverterBindStep::BindOwn { path: path.clone() });
    match converter {
        RuntimeDataBindGraphConverter::Scripted {
            global_id,
            serialized_implemented_methods,
            definition,
            ..
        } => {
            steps.push(RuntimeDataConverterBindStep::Rehydrate {
                path: path.clone(),
                converter_global_id: *global_id,
                inits: crate::scripted_data_converter::inits(*serialized_implemented_methods),
            });
            for (input_index, input) in definition.inputs.iter().enumerate() {
                let Some((data_bind_index, binding)) =
                    input.data_binds.iter().enumerate().next_back()
                else {
                    continue;
                };
                let crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                    ..
                } = binding
                else {
                    continue;
                };
                steps.push(RuntimeDataConverterBindStep::RebindFinalInput {
                    path: path.clone(),
                    input_index,
                    data_bind_index,
                });
            }
        }
        RuntimeDataBindGraphConverter::Group(children) => {
            for (index, child) in children.iter().enumerate() {
                path.push(index);
                collect_runtime_data_converter_bind_steps(child, path, steps);
                path.pop();
            }
        }
        _ => {}
    }
}

#[derive(Debug, Clone)]
struct RuntimeDetachedScriptedDataConverterState {
    definition: crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition,
    state: crate::scripted_data_converter::RuntimeScriptedDataConverterState,
}

#[derive(Debug, Clone, PartialEq)]
enum RuntimeConverterPropertyValue {
    Double(f32),
    Uint(u64),
    String(Arc<[u8]>),
    Bool(bool),
    Color(u32),
}

#[derive(Debug, Clone)]
enum RuntimeDataConverterBindingState {
    Inert,
    Context {
        source_path: Option<Vec<u32>>,
        name_based: bool,
        property_key: u32,
        flags: u64,
        target: RuntimeDataConverterBindingTarget,
        retained_bind: RuntimeRetainedDataBind,
    },
}

impl RuntimeDataConverterDataBindDefinition {
    pub(crate) fn instantiate(&self) -> RuntimeDataConverterDataBindState {
        let dirty_queue = RuntimeCellNotificationQueue::default();
        let mut target_values = BTreeMap::new();
        let bindings = self
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| match binding {
                RuntimeDataConverterBindingDefinition::Inert => {
                    RuntimeDataConverterBindingState::Inert
                }
                RuntimeDataConverterBindingDefinition::Context {
                    source_path,
                    name_based,
                    property_key,
                    flags,
                    target,
                    initial_target,
                } => {
                    if let Some(initial_target) = initial_target {
                        target_values
                            .entry((*target, *property_key))
                            .or_insert_with(|| initial_target.clone());
                    }
                    let mut retained_bind =
                        RuntimeRetainedDataBind::new(*flags, *flags & DATA_BIND_FLAG_ONCE != 0);
                    retained_bind.report_source_dirt_to(&dirty_queue, index);
                    RuntimeDataConverterBindingState::Context {
                        source_path: source_path.clone(),
                        name_based: *name_based,
                        property_key: *property_key,
                        flags: *flags,
                        target: *target,
                        retained_bind,
                    }
                }
            })
            .collect();
        RuntimeDataConverterDataBindState {
            bindings,
            children: self.children.iter().map(Self::instantiate).collect(),
            detached_scripted: self.detached_scripted_definition.as_ref().map(|definition| {
                RuntimeDetachedScriptedDataConverterState {
                    state:
                        crate::scripted_data_converter::RuntimeScriptedDataConverterState::from_definition(
                            definition,
                        ),
                    definition: definition.clone(),
                }
            }),
            dirty_queue,
            processing: false,
            active_dirty: BTreeSet::new(),
            rejected_entries: 0,
            parent_wake: None,
            target_values,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_scripted_definition(
        definition: &crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition,
    ) -> Self {
        let bindings = definition
            .data_bind_order
            .iter()
            .filter_map(|(input_index, data_bind_index)| {
                let binding = definition
                    .inputs
                    .get(*input_index)?
                    .data_binds
                    .get(*data_bind_index)?;
                Some((*input_index, *data_bind_index, binding))
            })
            .map(
                |(input_index, data_bind_index, binding)| match binding {
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Inert {
                        ..
                    } => RuntimeDataConverterBindingDefinition::Inert,
                    crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Context {
                        source_path,
                        name_based,
                        property_key,
                        flags,
                        ..
                    } => RuntimeDataConverterBindingDefinition::Context {
                        source_path: source_path.clone(),
                        name_based: *name_based,
                        property_key: *property_key,
                        flags: *flags,
                        target: RuntimeDataConverterBindingTarget::ScriptedInput {
                            input_index,
                            data_bind_index,
                        },
                        initial_target: None,
                    },
                },
            )
            .collect();
        Self {
            bindings,
            children: Vec::new(),
            detached_scripted_definition: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_converter_shape(converter: &RuntimeDataBindGraphConverter) -> Self {
        match converter {
            RuntimeDataBindGraphConverter::Scripted { definition, .. } => {
                Self::for_scripted_definition(definition)
            }
            RuntimeDataBindGraphConverter::Group(children) => Self {
                children: children.iter().map(Self::for_converter_shape).collect(),
                ..Self::default()
            },
            _ => Self::default(),
        }
    }
}

impl RuntimeDataConverterDataBindState {
    /// Install the outer occurrence retained by this exact converter clone.
    ///
    /// C++ assigns `m_parentDataBind` before binding any inner DataBind, and
    /// Group children retain the same outer pointer
    /// (`data_converter.cpp:25-29`; `data_converter_group.cpp:63-75`).
    pub(crate) fn set_parent_wake(
        &mut self,
        wake: RuntimeConverterParentWake,
        converter_state: &mut RuntimeDataBindGraphConverterState,
    ) {
        self.parent_wake = Some(wake.clone());
        for binding in &mut self.bindings {
            if let RuntimeDataConverterBindingState::Context { retained_bind, .. } = binding {
                retained_bind.set_container_wake(Some(wake.clone()));
            }
        }
        if let RuntimeDataBindGraphConverterState::Scripted(state) = converter_state {
            state.set_container_wake(Some(wake.clone()));
        }
        if let Some(detached) = self.detached_scripted.as_mut() {
            detached.state.set_container_wake(Some(wake.clone()));
        }
        if let RuntimeDataBindGraphConverterState::Group(states) = converter_state
            && states.len() == self.children.len()
        {
            for (child, state) in self.children.iter_mut().zip(states) {
                child.set_parent_wake(wake.clone(), state);
            }
        }
    }

    /// Clone live converter state for transaction adoption without sharing the
    /// source occurrence's notification queue. `RuntimeRetainedDataBind`
    /// already duplicates its source sinks; this installs the corresponding
    /// queue endpoint for every cloned nested occurrence.
    pub(crate) fn rehomed_clone(&self) -> Self {
        let mut cloned = self.clone();
        cloned.parent_wake = None;
        cloned.rehome_notification_queues();
        cloned
    }

    fn rehome_notification_queues(&mut self) {
        self.dirty_queue = RuntimeCellNotificationQueue::default();
        for (index, binding) in self.bindings.iter_mut().enumerate() {
            if let RuntimeDataConverterBindingState::Context { retained_bind, .. } = binding {
                retained_bind.report_source_dirt_to(&self.dirty_queue, index);
            }
        }
        for child in &mut self.children {
            child.rehome_notification_queues();
        }
    }

    pub(crate) fn fresh_clone(&self) -> Self {
        let dirty_queue = RuntimeCellNotificationQueue::default();
        let bindings = self
            .bindings
            .iter()
            .enumerate()
            .map(|(index, binding)| match binding {
                RuntimeDataConverterBindingState::Inert => RuntimeDataConverterBindingState::Inert,
                RuntimeDataConverterBindingState::Context {
                    source_path,
                    name_based,
                    property_key,
                    flags,
                    target,
                    ..
                } => {
                    let mut retained_bind =
                        RuntimeRetainedDataBind::new(*flags, *flags & DATA_BIND_FLAG_ONCE != 0);
                    retained_bind.report_source_dirt_to(&dirty_queue, index);
                    RuntimeDataConverterBindingState::Context {
                        source_path: source_path.clone(),
                        name_based: *name_based,
                        property_key: *property_key,
                        flags: *flags,
                        target: *target,
                        retained_bind,
                    }
                }
            })
            .collect();
        Self {
            bindings,
            children: self.children.iter().map(Self::fresh_clone).collect(),
            detached_scripted: self.detached_scripted.as_ref().map(|detached| {
                RuntimeDetachedScriptedDataConverterState {
                    state:
                        crate::scripted_data_converter::RuntimeScriptedDataConverterState::from_definition(
                            &detached.definition,
                        ),
                    definition: detached.definition.clone(),
                }
            }),
            dirty_queue,
            processing: false,
            active_dirty: BTreeSet::new(),
            rejected_entries: 0,
            parent_wake: None,
            target_values: self.target_values.clone(),
        }
    }

    /// Mirror the concrete converter's virtual `unbind` implementation.
    ///
    /// This is intentionally not a blanket source clear:
    ///
    /// - base/ordinary converters clear their own `m_dataBinds`;
    /// - Scripted is a base converter, so its unified custom-input list clears;
    /// - Group clears only child converters and leaves the Group's own list;
    /// - Formula clears only its separately retained outer source, which the
    ///   owning listener occurrence handles, and leaves token/self binds live.
    ///
    /// (`data_converter.cpp:32`; `data_converter_group.cpp:77-88`;
    /// `data_converter_formula.cpp:545-553`).
    pub(crate) fn unbind(
        &mut self,
        converter: &RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
    ) {
        match (converter, converter_state) {
            (
                RuntimeDataBindGraphConverter::Group(converters),
                RuntimeDataBindGraphConverterState::Group(states),
            ) if converters.len() == states.len() && converters.len() == self.children.len() => {
                for ((child, child_state), child_binds) in
                    converters.iter().zip(states).zip(&mut self.children)
                {
                    child_binds.unbind(child, child_state);
                }
            }
            (RuntimeDataBindGraphConverter::Formula { .. }, _) => {
                // Formula intentionally does not call `DataConverter::unbind`.
            }
            (
                RuntimeDataBindGraphConverter::Scripted { .. },
                RuntimeDataBindGraphConverterState::Scripted(state),
            ) => {
                self.clear_own_sources();
                state.unbind_sources();
            }
            _ => self.clear_own_sources(),
        }
        if let Some(detached) = self.detached_scripted.as_mut() {
            detached.state.unbind_sources();
        }
    }

    fn clear_own_sources(&mut self) {
        for binding in &mut self.bindings {
            if let RuntimeDataConverterBindingState::Context { retained_bind, .. } = binding {
                retained_bind.clear_source();
            }
        }
    }

    /// Bind only the converter occurrence at `path`, never its Group children.
    ///
    /// Pinned `DataConverterGroup::bindFromContext` binds the group base, then
    /// calls each child in authored order. A Scripted child runs `reinit`
    /// before the next child binds, so recursive eager binding is observably
    /// wrong when that init mutates the live DataContext
    /// (`data_converter_group.cpp:63-75`;
    /// `scripted_data_converter.cpp:170-188`).
    pub(crate) fn bind_own_sources_at_path(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        path: &[usize],
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        explicit_rebind: bool,
    ) -> bool {
        let Some((&index, tail)) = path.split_first() else {
            self.bind_own_sources(
                converter,
                converter_state,
                file,
                &mut |source_path, name_based| {
                    context
                        .property_path_for_context_source_path_with_persistent_resolver(
                            file,
                            &[],
                            source_path,
                            name_based,
                        )
                        .and_then(|path| context.cell_by_property_path(&path))
                },
                &mut |state, inputs, input_index, data_bind_index| {
                    state.bind_input_source(inputs, input_index, data_bind_index, file, context)
                },
                explicit_rebind,
            );
            crate::data_bind_graph::runtime_data_bind_graph_refresh_own_operation_view_model_converter_for_owned_context(
                converter,
                context,
                &[&[]],
            );
            return true;
        };
        let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (converter, converter_state)
        else {
            return false;
        };
        let Some(converter) = converters.get_mut(index) else {
            return false;
        };
        let Some(state) = states.get_mut(index) else {
            return false;
        };
        let Some(child) = self.children.get_mut(index) else {
            return false;
        };
        child.bind_own_sources_at_path(converter, state, tail, file, context, explicit_rebind)
    }

    /// DataContext-scoped companion to [`Self::bind_own_sources_at_path`].
    pub(crate) fn bind_own_sources_from_data_context_at_path(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        path: &[usize],
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        explicit_rebind: bool,
    ) -> bool {
        let Some((&index, tail)) = path.split_first() else {
            self.bind_own_sources(
                converter,
                converter_state,
                file,
                &mut |source_path, name_based| {
                    data_context.resolve_instance(&mut |_, context, scope_path| {
                        let path = context
                            .property_path_for_context_source_path_with_persistent_resolver(
                                file,
                                scope_path,
                                source_path,
                                name_based,
                            )?;
                        context.cell_by_property_path(&path)
                    })
                },
                &mut |state, inputs, input_index, data_bind_index| {
                    state.bind_input_source_from_data_context(
                        inputs,
                        input_index,
                        data_bind_index,
                        file,
                        data_context,
                    )
                },
                explicit_rebind,
            );
            crate::data_bind_graph::runtime_data_bind_graph_bind_own_converter_operands_for_data_context(
                converter,
                data_context,
            );
            return true;
        };
        let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (converter, converter_state)
        else {
            return false;
        };
        let Some(converter) = converters.get_mut(index) else {
            return false;
        };
        let Some(state) = states.get_mut(index) else {
            return false;
        };
        let Some(child) = self.children.get_mut(index) else {
            return false;
        };
        child.bind_own_sources_from_data_context_at_path(
            converter,
            state,
            tail,
            file,
            data_context,
            explicit_rebind,
        )
    }

    /// `DataConverter::bindFromContext`: bind the one unified own list first;
    /// Group then recurses through child occurrences in item order.
    pub(crate) fn bind_sources(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        explicit_rebind: bool,
    ) {
        self.bind_own_sources(
            converter,
            converter_state,
            file,
            &mut |source_path, name_based| {
                context
                    .property_path_for_context_source_path_with_persistent_resolver(
                        file,
                        &[],
                        source_path,
                        name_based,
                    )
                    .and_then(|path| context.cell_by_property_path(&path))
            },
            &mut |state, inputs, input_index, data_bind_index| {
                state.bind_input_source(inputs, input_index, data_bind_index, file, context)
            },
            explicit_rebind,
        );
        if let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (&mut *converter, &mut *converter_state)
            && converters.len() == states.len()
            && converters.len() == self.children.len()
        {
            for ((child, child_state), child_binds) in
                converters.iter_mut().zip(states).zip(&mut self.children)
            {
                child_binds.bind_sources(child, child_state, file, context, explicit_rebind);
            }
        }
    }

    /// DataContext-scoped companion to [`Self::bind_sources`].
    pub(crate) fn bind_sources_from_data_context(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        explicit_rebind: bool,
    ) {
        self.bind_own_sources(
            converter,
            converter_state,
            file,
            &mut |source_path, name_based| {
                data_context.resolve_instance(&mut |_, context, scope_path| {
                    let path = context
                        .property_path_for_context_source_path_with_persistent_resolver(
                            file,
                            scope_path,
                            source_path,
                            name_based,
                        )?;
                    context.cell_by_property_path(&path)
                })
            },
            &mut |state, inputs, input_index, data_bind_index| {
                state.bind_input_source_from_data_context(
                    inputs,
                    input_index,
                    data_bind_index,
                    file,
                    data_context,
                )
            },
            explicit_rebind,
        );
        if let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (&mut *converter, &mut *converter_state)
            && converters.len() == states.len()
            && converters.len() == self.children.len()
        {
            for ((child, child_state), child_binds) in
                converters.iter_mut().zip(states).zip(&mut self.children)
            {
                child_binds.bind_sources_from_data_context(
                    child,
                    child_state,
                    file,
                    data_context,
                    explicit_rebind,
                );
            }
        }
    }

    fn bind_own_sources<F, G>(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        _file: &RuntimeFile,
        resolve: &mut F,
        bind_scripted_input: &mut G,
        explicit_rebind: bool,
    ) where
        F: FnMut(&[u32], bool) -> Option<RuntimeViewModelCell>,
        G: FnMut(
            &mut crate::scripted_data_converter::RuntimeScriptedDataConverterState,
            &mut [crate::scripted_data_converter::RuntimeScriptedDataConverterInputDefinition],
            usize,
            usize,
        ) -> bool,
    {
        for (binding_index, binding) in self.bindings.iter_mut().enumerate() {
            let RuntimeDataConverterBindingState::Context {
                source_path,
                name_based,
                target,
                retained_bind,
                ..
            } = binding
            else {
                continue;
            };
            if let RuntimeDataConverterBindingTarget::ScriptedInput {
                input_index,
                data_bind_index,
            } = target
            {
                if let (
                    RuntimeDataBindGraphConverter::Scripted { definition, .. },
                    RuntimeDataBindGraphConverterState::Scripted(state),
                ) = (&mut *converter, &mut *converter_state)
                {
                    state.report_input_source_dirt_to(
                        *input_index,
                        *data_bind_index,
                        &self.dirty_queue,
                        binding_index,
                    );
                    bind_scripted_input(
                        state,
                        &mut definition.inputs,
                        *input_index,
                        *data_bind_index,
                    );
                } else if let Some(detached) = self.detached_scripted.as_mut() {
                    detached.state.report_input_source_dirt_to(
                        *input_index,
                        *data_bind_index,
                        &self.dirty_queue,
                        binding_index,
                    );
                    bind_scripted_input(
                        &mut detached.state,
                        &mut detached.definition.inputs,
                        *input_index,
                        *data_bind_index,
                    );
                }
                continue;
            }
            let source_cell = source_path
                .as_deref()
                .and_then(|path| resolve(path, *name_based));
            bind_resolved_source(retained_bind, source_cell, explicit_rebind);
        }
    }

    /// Drain converter-owned binds before the outer DataBind applies its
    /// source. This is `DataBind::updateDependents` calling
    /// `DataConverter::update`, so a parameter/token update affects the same
    /// outer conversion (`data_bind.cpp:462-471`;
    /// `data_converter.cpp:57`).
    pub(crate) fn update<F>(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        apply: &mut F,
    ) -> Result<(), ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        // Pinned Group overrides update and intentionally does not drain its
        // own list; it updates only child occurrences in item order
        // (`data_converter_group.cpp:76-87`).
        if let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (&mut *converter, &mut *converter_state)
            && converters.len() == states.len()
            && converters.len() == self.children.len()
        {
            for ((child, child_state), child_binds) in
                converters.iter_mut().zip(states).zip(&mut self.children)
            {
                child_binds.update(child, child_state, file, context, owner_instance, apply)?;
            }
            return Ok(());
        }

        // Swap the active queue before applying. Notifications produced while
        // this pass runs append to the now-empty pending allocation and are
        // therefore deferred to the next converter update exactly like
        // `DataBindContainer::m_pendingDirty*`.
        let queued = self.take_dirty_binding_order();
        for (position, binding_index) in queued.iter().copied().enumerate() {
            self.begin_dirty_binding(binding_index);
            let result = (|| -> Result<_, ScriptError> {
                let mut changed_target = None;
                match &mut self.bindings[binding_index] {
                    RuntimeDataConverterBindingState::Inert => {}
                    RuntimeDataConverterBindingState::Context {
                        property_key,
                        target:
                            RuntimeDataConverterBindingTarget::ScriptedInput {
                                input_index,
                                data_bind_index,
                            },
                        ..
                    } => {
                        let target_changed = if let (
                            RuntimeDataBindGraphConverter::Scripted {
                                definition,
                                instance,
                                ..
                            },
                            RuntimeDataBindGraphConverterState::Scripted(state),
                        ) = (&mut *converter, &mut *converter_state)
                        {
                            state.update_input(
                                &mut definition.inputs,
                                *input_index,
                                *data_bind_index,
                                instance.as_ref().or(owner_instance),
                                file,
                                context,
                                apply,
                            )?
                        } else if let Some(detached) = self.detached_scripted.as_mut() {
                            detached.state.update_input(
                                &mut detached.definition.inputs,
                                *input_index,
                                *data_bind_index,
                                None,
                                file,
                                context,
                                apply,
                            )?
                        } else {
                            false
                        };
                        if target_changed {
                            changed_target = Some((
                                RuntimeDataConverterBindingTarget::ScriptedInput {
                                    input_index: *input_index,
                                    data_bind_index: *data_bind_index,
                                },
                                *property_key,
                            ));
                        }
                    }
                    RuntimeDataConverterBindingState::Context {
                        property_key,
                        target,
                        retained_bind,
                        ..
                    } => {
                        retained_bind.collect_source_dirt();
                        // Converter update is always `updateDataBinds(false)`.
                        retained_bind.take_target_dirt();
                        let source_kind = retained_bind.source().map(RuntimeViewModelCell::value);
                        let mut target = RuntimeConverterPropertyTarget {
                            converter,
                            target: *target,
                            property_key: *property_key,
                            source_kind,
                            target_values: &mut self.target_values,
                            changed: false,
                        };
                        retained_bind.update(&mut target);
                        if target.changed {
                            changed_target = Some((target.target, *property_key));
                        }
                    }
                }
                Ok(changed_target)
            })();
            match result {
                Ok(Some((target, property_key))) => {
                    self.notify_target_observers(
                        binding_index,
                        target,
                        property_key,
                        converter,
                        converter_state,
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    self.abort_dirty_bindings(queued[position..].iter().copied());
                    return Err(error);
                }
            }
        }
        self.finish_dirty_bindings();
        Ok(())
    }

    pub(crate) fn update_from_data_context<F>(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        file: &RuntimeFile,
        data_context: &RuntimeOwnedDataContext,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        apply: &mut F,
    ) -> Result<(), ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        if let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (&mut *converter, &mut *converter_state)
            && converters.len() == states.len()
            && converters.len() == self.children.len()
        {
            for ((child, child_state), child_binds) in
                converters.iter_mut().zip(states).zip(&mut self.children)
            {
                child_binds.update_from_data_context(
                    child,
                    child_state,
                    file,
                    data_context,
                    owner_instance,
                    apply,
                )?;
            }
            return Ok(());
        }

        let queued = self.take_dirty_binding_order();
        for (position, binding_index) in queued.iter().copied().enumerate() {
            self.begin_dirty_binding(binding_index);
            let result = (|| -> Result<_, ScriptError> {
                let mut changed_target = None;
                match &mut self.bindings[binding_index] {
                    RuntimeDataConverterBindingState::Inert => {}
                    RuntimeDataConverterBindingState::Context {
                        property_key,
                        target:
                            RuntimeDataConverterBindingTarget::ScriptedInput {
                                input_index,
                                data_bind_index,
                            },
                        ..
                    } => {
                        let target_changed = if let (
                            RuntimeDataBindGraphConverter::Scripted {
                                definition,
                                instance,
                                ..
                            },
                            RuntimeDataBindGraphConverterState::Scripted(state),
                        ) = (&mut *converter, &mut *converter_state)
                        {
                            state.update_input_from_data_context(
                                &mut definition.inputs,
                                *input_index,
                                *data_bind_index,
                                instance.as_ref().or(owner_instance),
                                file,
                                data_context,
                                apply,
                            )?
                        } else if let Some(detached) = self.detached_scripted.as_mut() {
                            detached.state.update_input_from_data_context(
                                &mut detached.definition.inputs,
                                *input_index,
                                *data_bind_index,
                                None,
                                file,
                                data_context,
                                apply,
                            )?
                        } else {
                            false
                        };
                        if target_changed {
                            changed_target = Some((
                                RuntimeDataConverterBindingTarget::ScriptedInput {
                                    input_index: *input_index,
                                    data_bind_index: *data_bind_index,
                                },
                                *property_key,
                            ));
                        }
                    }
                    RuntimeDataConverterBindingState::Context {
                        property_key,
                        target,
                        retained_bind,
                        ..
                    } => {
                        retained_bind.collect_source_dirt();
                        retained_bind.take_target_dirt();
                        let source_kind = retained_bind.source().map(RuntimeViewModelCell::value);
                        let mut target = RuntimeConverterPropertyTarget {
                            converter,
                            target: *target,
                            property_key: *property_key,
                            source_kind,
                            target_values: &mut self.target_values,
                            changed: false,
                        };
                        retained_bind.update(&mut target);
                        if target.changed {
                            changed_target = Some((target.target, *property_key));
                        }
                    }
                }
                Ok(changed_target)
            })();
            match result {
                Ok(Some((target, property_key))) => {
                    self.notify_target_observers(
                        binding_index,
                        target,
                        property_key,
                        converter,
                        converter_state,
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    self.abort_dirty_bindings(queued[position..].iter().copied());
                    return Err(error);
                }
            }
        }
        self.finish_dirty_bindings();
        Ok(())
    }

    /// Public `DataBindContainer::updateDataBinds(true)` for the concrete
    /// DataBinds cloned into one converter occurrence.
    ///
    /// Dependents run before their parent outer bind. Push-driven to-source
    /// occurrences remain ahead of to-target occurrences via
    /// `take_dirty_binding_order`, and each occurrence reconciles in its own
    /// authored favor order (`data_bind_container.cpp:115-203`;
    /// `data_converter.cpp:57-69`).
    pub(crate) fn public_update<F>(
        &mut self,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
        file: &RuntimeFile,
        owner_instance: Option<&RuntimeScriptInstanceHandle>,
        apply_target_to_source: bool,
        apply: &mut F,
    ) -> Result<(), ScriptError>
    where
        F: FnMut(
            &RuntimeScriptInstanceHandle,
            &ScriptCoreString,
            RuntimeScriptedListenerBoundValue,
        ) -> Result<(), ScriptError>,
    {
        if let (
            RuntimeDataBindGraphConverter::Group(converters),
            RuntimeDataBindGraphConverterState::Group(states),
        ) = (&mut *converter, &mut *converter_state)
            && converters.len() == states.len()
            && converters.len() == self.children.len()
        {
            for ((child, child_state), child_binds) in
                converters.iter_mut().zip(states).zip(&mut self.children)
            {
                child_binds.public_update(
                    child,
                    child_state,
                    file,
                    owner_instance,
                    apply_target_to_source,
                    apply,
                )?;
            }
            return Ok(());
        }

        let queued = self.take_dirty_binding_order();
        for (position, binding_index) in queued.iter().copied().enumerate() {
            self.begin_dirty_binding(binding_index);
            let result = (|| -> Result<_, ScriptError> {
                let mut changed_target = None;
                match &mut self.bindings[binding_index] {
                    RuntimeDataConverterBindingState::Inert => {}
                    RuntimeDataConverterBindingState::Context {
                        property_key,
                        target:
                            RuntimeDataConverterBindingTarget::ScriptedInput {
                                input_index,
                                data_bind_index,
                            },
                        ..
                    } => {
                        let target_changed = if let (
                            RuntimeDataBindGraphConverter::Scripted {
                                definition,
                                instance,
                                ..
                            },
                            RuntimeDataBindGraphConverterState::Scripted(state),
                        ) = (&mut *converter, &mut *converter_state)
                        {
                            state.public_update_input(
                                &mut definition.inputs,
                                *input_index,
                                *data_bind_index,
                                instance.as_ref().or(owner_instance),
                                file,
                                apply_target_to_source,
                                apply,
                            )?
                        } else if let Some(detached) = self.detached_scripted.as_mut() {
                            detached.state.public_update_input(
                                &mut detached.definition.inputs,
                                *input_index,
                                *data_bind_index,
                                None,
                                file,
                                apply_target_to_source,
                                apply,
                            )?
                        } else {
                            false
                        };
                        if target_changed {
                            changed_target = Some((
                                RuntimeDataConverterBindingTarget::ScriptedInput {
                                    input_index: *input_index,
                                    data_bind_index: *data_bind_index,
                                },
                                *property_key,
                            ));
                        }
                    }
                    RuntimeDataConverterBindingState::Context {
                        property_key,
                        target,
                        retained_bind,
                        ..
                    } => {
                        retained_bind.collect_source_dirt();
                        let wants_target_to_source = apply_target_to_source
                            && retained_bind.to_source()
                            && retained_bind
                                .pending_dirt()
                                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS_TARGET);
                        let source_runs_first = retained_bind.source_to_target_runs_first();
                        let source_kind = retained_bind.source().map(RuntimeViewModelCell::value);
                        let mut target_adapter = RuntimeConverterPropertyTarget {
                            converter,
                            target: *target,
                            property_key: *property_key,
                            source_kind,
                            target_values: &mut self.target_values,
                            changed: false,
                        };
                        if wants_target_to_source && !source_runs_first {
                            retained_bind.update_source_binding(&mut target_adapter);
                        }
                        retained_bind.update(&mut target_adapter);
                        if wants_target_to_source && source_runs_first {
                            retained_bind.update_source_binding(&mut target_adapter);
                        } else if !wants_target_to_source {
                            retained_bind.take_target_dirt();
                        }
                        if target_adapter.changed {
                            changed_target = Some((target_adapter.target, *property_key));
                        }
                    }
                }
                Ok(changed_target)
            })();
            match result {
                Ok(Some((target, property_key))) => {
                    self.notify_target_observers(
                        binding_index,
                        target,
                        property_key,
                        converter,
                        converter_state,
                    );
                }
                Ok(None) => {}
                Err(error) => {
                    self.abort_dirty_bindings(queued[position..].iter().copied());
                    return Err(error);
                }
            }
        }
        self.finish_dirty_bindings();
        Ok(())
    }

    fn take_dirty_binding_order(&mut self) -> Vec<usize> {
        if self.processing {
            self.rejected_entries += 1;
            return Vec::new();
        }
        self.processing = true;
        let mut reported = Vec::new();
        self.dirty_queue.swap_into(&mut reported);
        self.active_dirty.clear();
        let mut seen = vec![false; self.bindings.len()];
        reported.retain(|index| {
            let Some(slot) = seen.get_mut(*index) else {
                return false;
            };
            if *slot {
                return false;
            }
            *slot = true;
            true
        });
        self.active_dirty.extend(reported.iter().copied());
        let mut ordered = Vec::with_capacity(reported.len());
        for to_source_queue in [true, false] {
            ordered.extend(reported.iter().copied().filter(|index| {
                matches!(
                    self.bindings.get(*index),
                    Some(RuntimeDataConverterBindingState::Context { flags, .. })
                        if data_bind_flags_apply_target_to_source(*flags) == to_source_queue
                )
            }));
        }
        ordered
    }

    fn begin_dirty_binding(&mut self, binding_index: usize) {
        if self.active_dirty.remove(&binding_index) {
            self.dirty_queue.remove_data_bind(binding_index);
        }
    }

    fn finish_dirty_bindings(&mut self) {
        if self.rejected_entries > 0 {
            self.rejected_entries -= 1;
            return;
        }
        self.active_dirty.clear();
        self.processing = false;
    }

    fn abort_dirty_bindings(&mut self, remaining: impl IntoIterator<Item = usize>) {
        for index in remaining {
            if let Some(RuntimeDataConverterBindingState::Context { retained_bind, .. }) =
                self.bindings.get_mut(index)
            {
                retained_bind.requeue_source_dirt();
            }
        }
        self.finish_dirty_bindings();
    }

    fn notify_target_observers(
        &mut self,
        writer_index: usize,
        target: RuntimeDataConverterBindingTarget,
        property_key: u32,
        converter: &mut RuntimeDataBindGraphConverter,
        converter_state: &mut RuntimeDataBindGraphConverterState,
    ) {
        let candidates = self
            .bindings
            .iter()
            .enumerate()
            .filter_map(|(index, binding)| {
                if index == writer_index {
                    return None;
                }
                let RuntimeDataConverterBindingState::Context {
                    target: candidate_target,
                    property_key: candidate_key,
                    flags,
                    ..
                } = binding
                else {
                    return None;
                };
                let same_target = match (*candidate_target, target) {
                    (
                        RuntimeDataConverterBindingTarget::ScriptedInput {
                            input_index: candidate_input,
                            ..
                        },
                        RuntimeDataConverterBindingTarget::ScriptedInput {
                            input_index: writer_input,
                            ..
                        },
                    ) => candidate_input == writer_input,
                    _ => *candidate_target == target,
                };
                (same_target
                    && *candidate_key == property_key
                    && data_bind_flags_apply_target_to_source(*flags))
                .then_some((index, *candidate_target))
            })
            .collect::<Vec<_>>();

        for (index, candidate_target) in candidates {
            let marked = match candidate_target {
                RuntimeDataConverterBindingTarget::ScriptedInput {
                    input_index,
                    data_bind_index,
                } => {
                    if let (
                        RuntimeDataBindGraphConverter::Scripted { .. },
                        RuntimeDataBindGraphConverterState::Scripted(state),
                    ) = (&mut *converter, &mut *converter_state)
                    {
                        state.mark_input_target_changed(input_index, data_bind_index)
                    } else if let Some(detached) = self.detached_scripted.as_mut() {
                        detached
                            .state
                            .mark_input_target_changed(input_index, data_bind_index)
                    } else {
                        false
                    }
                }
                _ => {
                    let Some(RuntimeDataConverterBindingState::Context { retained_bind, .. }) =
                        self.bindings.get_mut(index)
                    else {
                        continue;
                    };
                    retained_bind.mark_target_changed();
                    true
                }
            };
            let _ = marked;
        }
    }
}

fn bind_resolved_source(
    retained_bind: &mut RuntimeRetainedDataBind,
    source_cell: Option<RuntimeViewModelCell>,
    force_reconcile: bool,
) {
    retained_bind.collect_source_dirt();
    let source_resolved = source_cell.is_some();
    let source_rebound = match (retained_bind.source(), source_cell.as_ref()) {
        (Some(current), Some(next)) => !current.ptr_eq(next),
        (None, None) => false,
        _ => true,
    };
    if source_rebound {
        retained_bind.clear_source();
        if let Some(source_cell) = source_cell {
            retained_bind.set_source(source_cell);
        }
    }
    // A missing source takes C++'s `unbind()` branch and does not enqueue a
    // fabricated reconcile. Only a resolved new/same source runs `bind()` or
    // the explicit same-pointer reconcile (`data_bind_context.cpp:56-89`).
    if source_resolved && (source_rebound || force_reconcile) {
        retained_bind.mark_rebind_reconcile();
    }
}

/// Build the exact own-list/Group-child clone recipe for one converter
/// definition. Attached converters on these DataBinds are intentionally not
/// inspected: `DataBindBase::copy` copies the serialized converter id but not
/// the resolved `m_dataConverter` pointer, so the cloned occurrence drops
/// that subordinate converter at this pin.
pub(crate) fn runtime_data_converter_data_bind_definition(
    file: &RuntimeFile,
    converter_object: &RuntimeObject,
    converter: &RuntimeDataBindGraphConverter,
) -> RuntimeDataConverterDataBindDefinition {
    runtime_data_converter_data_bind_definition_inner(
        file,
        converter_object,
        converter,
        &mut Vec::new(),
    )
}

fn runtime_data_converter_data_bind_definition_inner(
    file: &RuntimeFile,
    converter_object: &RuntimeObject,
    converter: &RuntimeDataBindGraphConverter,
    visiting: &mut Vec<u32>,
) -> RuntimeDataConverterDataBindDefinition {
    if visiting.contains(&converter_object.id) {
        return RuntimeDataConverterDataBindDefinition::default();
    }
    visiting.push(converter_object.id);

    let formula_tokens = if converter_object.type_name == "DataConverterFormula" {
        file.data_converter_formula_authored_tokens_for_object(converter_object)
    } else {
        Vec::new()
    };
    let formula_output_tokens = if converter_object.type_name == "DataConverterFormula" {
        file.data_converter_formula_output_tokens_for_object(converter_object)
    } else {
        Vec::new()
    };
    let authored_scripted_definition = if converter_object.type_name == "ScriptedDataConverter" {
        crate::data_bind_graph::runtime_scripted_data_converter_input_definitions(
            file,
            converter_object,
        )
    } else {
        crate::scripted_data_converter::RuntimeScriptedDataConverterDefinition::default()
    };
    let scripted_inputs = (!authored_scripted_definition.inputs.is_empty())
        .then_some(authored_scripted_definition.inputs.as_slice());
    let detached_scripted_definition = (converter_object.type_name == "ScriptedDataConverter"
        && !matches!(converter, RuntimeDataBindGraphConverter::Scripted { .. }))
    .then_some(authored_scripted_definition.clone());

    let bindings = (0..file.object_count())
        .filter_map(|id| file.object(id))
        .filter(|object| {
            nuxie_schema::definition_by_name(object.type_name)
                .is_some_and(|definition| definition.is_a("DataBind"))
        })
        .filter_map(|data_bind| {
            let target_object = file.data_bind_target_for_object(data_bind)?;
            let (target, initial_target_object) = if target_object.id == converter_object.id {
                (
                    RuntimeDataConverterBindingTarget::SelfProperty,
                    converter_object,
                )
            } else if formula_tokens
                .iter()
                .any(|token| token.id == target_object.id)
            {
                // `DataConverter::copy` initially targets every owned bind at
                // the cloned Formula itself. Formula repairs only output-queue
                // tokens by the original bind-list index; non-output tokens
                // stay self-targeted instead of disappearing.
                if matches!(converter, RuntimeDataBindGraphConverter::Formula { .. })
                    && let Some(token_index) = formula_output_tokens
                        .iter()
                        .position(|token| token.object.id == target_object.id)
                {
                    (
                        RuntimeDataConverterBindingTarget::FormulaToken { token_index },
                        target_object,
                    )
                } else {
                    (
                        RuntimeDataConverterBindingTarget::SelfProperty,
                        converter_object,
                    )
                }
            } else if let Some(inputs) = scripted_inputs {
                let input_index = inputs
                    .iter()
                    .position(|input| input.input_global_id == target_object.id)?;
                let data_bind_index = inputs[input_index]
                    .data_binds
                    .iter()
                    .position(|binding| binding.authored_order() == data_bind.id)?;
                (
                    RuntimeDataConverterBindingTarget::ScriptedInput {
                        input_index,
                        data_bind_index,
                    },
                    target_object,
                )
            } else {
                return None;
            };

            if data_bind.type_name != "DataBindContext" {
                return Some(RuntimeDataConverterBindingDefinition::Inert);
            }
            let property_key = data_bind
                .uint_property("propertyKey")
                .and_then(|value| u32::try_from(value).ok())
                .unwrap_or(u32::MAX);
            Some(RuntimeDataConverterBindingDefinition::Context {
                source_path: file.data_bind_context_source_path_ids_for_object(data_bind),
                name_based: file
                    .data_bind_is_name_based_for_object(data_bind)
                    .unwrap_or(false),
                property_key,
                flags: data_bind.uint_property("flags").unwrap_or(0),
                target,
                initial_target: runtime_object_target_value(initial_target_object, property_key),
            })
        })
        .collect();

    let children = match converter {
        RuntimeDataBindGraphConverter::Group(children) => file
            .data_converter_group_items_for_object(converter_object)
            .into_iter()
            .filter_map(|item| item.converter)
            .zip(children)
            .map(|(child_object, child)| {
                runtime_data_converter_data_bind_definition_inner(
                    file,
                    child_object,
                    child,
                    visiting,
                )
            })
            .collect(),
        _ => Vec::new(),
    };
    visiting.pop();
    RuntimeDataConverterDataBindDefinition {
        bindings,
        children,
        detached_scripted_definition,
    }
}

fn runtime_object_target_value(
    target: &RuntimeObject,
    property_key: u32,
) -> Option<RuntimeConverterPropertyValue> {
    let property_key = u16::try_from(property_key).ok()?;
    match nuxie_schema::core_registry_setter_field_kind_by_property_key(property_key)? {
        nuxie_schema::FieldKind::Double => {
            crate::properties::runtime_object_double_property_by_key(target, property_key)
                .map(RuntimeConverterPropertyValue::Double)
        }
        nuxie_schema::FieldKind::Int => None,
        nuxie_schema::FieldKind::Uint => {
            crate::properties::runtime_object_uint_property_by_key(target, property_key)
                .map(RuntimeConverterPropertyValue::Uint)
        }
        nuxie_schema::FieldKind::String | nuxie_schema::FieldKind::Bytes => {
            crate::properties::runtime_object_string_property_by_key(target, property_key)
                .map(|value| RuntimeConverterPropertyValue::String(Arc::from(value)))
        }
        nuxie_schema::FieldKind::Bool => {
            crate::properties::runtime_object_bool_property_by_key(target, property_key)
                .map(RuntimeConverterPropertyValue::Bool)
        }
        nuxie_schema::FieldKind::Color => {
            crate::properties::runtime_object_color_property_by_key(target, property_key)
                .map(RuntimeConverterPropertyValue::Color)
        }
        nuxie_schema::FieldKind::Callback => None,
    }
}

struct RuntimeConverterPropertyTarget<'a> {
    converter: &'a mut RuntimeDataBindGraphConverter,
    target: RuntimeDataConverterBindingTarget,
    property_key: u32,
    source_kind: Option<RuntimeViewModelCellValue>,
    target_values:
        &'a mut BTreeMap<(RuntimeDataConverterBindingTarget, u32), RuntimeConverterPropertyValue>,
    changed: bool,
}

impl RuntimeDataBindTarget for RuntimeConverterPropertyTarget<'_> {
    fn apply_to_target(&mut self, value: &RuntimeViewModelCellValue) {
        let key = (self.target, self.property_key);
        let before = read_converter_property_value(
            self.converter,
            self.target,
            self.property_key,
            self.target_values,
        );
        let accepted =
            apply_converter_property(self.converter, self.target, self.property_key, value);
        let normalized = normalize_converter_property_value(self.property_key, value);
        if accepted || normalized.is_some() {
            if let Some(value) =
                read_modeled_converter_property(self.converter, self.target, self.property_key)
                    .or(normalized)
            {
                self.changed |= before.as_ref() != Some(&value);
                self.target_values.insert(key, value);
            }
        }
    }

    fn read_target(&mut self) -> Option<RuntimeViewModelCellValue> {
        let value = read_converter_property_value(
            self.converter,
            self.target,
            self.property_key,
            self.target_values,
        )?;
        converter_property_value_for_source(value, self.source_kind.as_ref())
    }
}

fn normalize_converter_property_value(
    property_key: u32,
    value: &RuntimeViewModelCellValue,
) -> Option<RuntimeConverterPropertyValue> {
    let Some(property_key) = u16::try_from(property_key).ok() else {
        return None;
    };
    match nuxie_schema::core_registry_setter_field_kind_by_property_key(property_key) {
        Some(nuxie_schema::FieldKind::Double) => {
            number(value).map(RuntimeConverterPropertyValue::Double)
        }
        Some(nuxie_schema::FieldKind::Int) => None,
        Some(nuxie_schema::FieldKind::Uint) => uint(value).map(RuntimeConverterPropertyValue::Uint),
        // C++ `DataBindContextValueString` writes CoreString only. A Bytes
        // field such as OperationViewModel::sourcePathIds is not a compatible
        // runtime String target.
        Some(nuxie_schema::FieldKind::String) => {
            string(value).map(|value| RuntimeConverterPropertyValue::String(Arc::from(value)))
        }
        Some(nuxie_schema::FieldKind::Bool) => match value {
            RuntimeViewModelCellValue::Boolean(value) => {
                Some(RuntimeConverterPropertyValue::Bool(*value))
            }
            _ => None,
        },
        Some(nuxie_schema::FieldKind::Color) => match value {
            RuntimeViewModelCellValue::Color(value) => {
                Some(RuntimeConverterPropertyValue::Color(*value))
            }
            _ => None,
        },
        Some(nuxie_schema::FieldKind::Bytes | nuxie_schema::FieldKind::Callback) | None => None,
    }
}

fn read_converter_property_value(
    converter: &RuntimeDataBindGraphConverter,
    target: RuntimeDataConverterBindingTarget,
    property_key: u32,
    target_values: &BTreeMap<
        (RuntimeDataConverterBindingTarget, u32),
        RuntimeConverterPropertyValue,
    >,
) -> Option<RuntimeConverterPropertyValue> {
    read_modeled_converter_property(converter, target, property_key)
        .or_else(|| target_values.get(&(target, property_key)).cloned())
}

fn converter_property_value_for_source(
    value: RuntimeConverterPropertyValue,
    source_kind: Option<&RuntimeViewModelCellValue>,
) -> Option<RuntimeViewModelCellValue> {
    match (value, source_kind?) {
        (RuntimeConverterPropertyValue::Double(value), RuntimeViewModelCellValue::Number(_)) => {
            Some(RuntimeViewModelCellValue::Number(value))
        }
        (RuntimeConverterPropertyValue::String(value), RuntimeViewModelCellValue::String(_)) => {
            Some(RuntimeViewModelCellValue::String(value))
        }
        (RuntimeConverterPropertyValue::Bool(value), RuntimeViewModelCellValue::Boolean(_)) => {
            Some(RuntimeViewModelCellValue::Boolean(value))
        }
        (RuntimeConverterPropertyValue::Color(value), RuntimeViewModelCellValue::Color(_)) => {
            Some(RuntimeViewModelCellValue::Color(value))
        }
        (RuntimeConverterPropertyValue::Uint(value), RuntimeViewModelCellValue::Enum(_)) => {
            Some(RuntimeViewModelCellValue::Enum(value as u32))
        }
        (RuntimeConverterPropertyValue::Uint(value), RuntimeViewModelCellValue::Trigger(_)) => {
            Some(RuntimeViewModelCellValue::Trigger(value))
        }
        (
            RuntimeConverterPropertyValue::Uint(value),
            RuntimeViewModelCellValue::SymbolListIndex(_),
        ) => Some(RuntimeViewModelCellValue::SymbolListIndex(value as u32)),
        (RuntimeConverterPropertyValue::Uint(value), RuntimeViewModelCellValue::AssetImage(_)) => {
            Some(RuntimeViewModelCellValue::AssetImage(value as u32))
        }
        (
            RuntimeConverterPropertyValue::Uint(value),
            RuntimeViewModelCellValue::AssetFont(current),
        ) => {
            let mut value_with_font = current.clone();
            value_with_font.set_file_asset_index(value);
            Some(RuntimeViewModelCellValue::AssetFont(value_with_font))
        }
        (RuntimeConverterPropertyValue::Uint(value), RuntimeViewModelCellValue::Artboard(_)) => {
            Some(RuntimeViewModelCellValue::Artboard(value as u32))
        }
        _ => None,
    }
}

fn number(value: &RuntimeViewModelCellValue) -> Option<f32> {
    match value {
        RuntimeViewModelCellValue::Number(value) => Some(*value),
        RuntimeViewModelCellValue::SymbolListIndex(value) => Some(*value as f32),
        _ => None,
    }
}

fn uint(value: &RuntimeViewModelCellValue) -> Option<u64> {
    match value {
        RuntimeViewModelCellValue::Number(value) => {
            if value.is_nan() || *value < 0.0 {
                return Some(0);
            }
            Some(value.round() as u64)
        }
        RuntimeViewModelCellValue::SymbolListIndex(value)
        | RuntimeViewModelCellValue::Enum(value)
        | RuntimeViewModelCellValue::AssetImage(value)
        | RuntimeViewModelCellValue::Artboard(value) => Some(u64::from(*value)),
        RuntimeViewModelCellValue::Trigger(value) => Some(*value),
        RuntimeViewModelCellValue::AssetFont(value) => Some(value.file_asset_index()),
        _ => None,
    }
}

fn string(value: &RuntimeViewModelCellValue) -> Option<Vec<u8>> {
    match value {
        RuntimeViewModelCellValue::String(value) => Some(value.to_vec()),
        _ => None,
    }
}

fn apply_converter_property(
    converter: &mut RuntimeDataBindGraphConverter,
    target: RuntimeDataConverterBindingTarget,
    property_key: u32,
    value: &RuntimeViewModelCellValue,
) -> bool {
    if let RuntimeDataConverterBindingTarget::FormulaToken { token_index } = target {
        let RuntimeDataBindGraphConverter::Formula { tokens } = converter else {
            return false;
        };
        let Some(token) = tokens.get_mut(token_index) else {
            return false;
        };
        return match (token, property_key) {
            (
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Value {
                    value: target,
                    ..
                },
                777,
            ) => number(value).is_some_and(|value| {
                *target = value;
                true
            }),
            (
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Operation {
                    operation_type,
                },
                775,
            ) => uint(value).is_some_and(|value| {
                *operation_type = value;
                true
            }),
            (
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Function {
                    function_type,
                    ..
                },
                776,
            ) => uint(value).is_some_and(|value| {
                *function_type = value;
                true
            }),
            _ => false,
        };
    }
    if target != RuntimeDataConverterBindingTarget::SelfProperty {
        return false;
    }
    match (converter, property_key) {
        (RuntimeDataBindGraphConverter::NumberToList { view_model_id, .. }, 816) => uint(value)
            .is_some_and(|value| {
                *view_model_id = value;
                true
            }),
        (RuntimeDataBindGraphConverter::ToString { flags, .. }, 764) => {
            uint(value).is_some_and(|value| {
                *flags = value;
                true
            })
        }
        (RuntimeDataBindGraphConverter::ToString { decimals, .. }, 765) => {
            uint(value).is_some_and(|value| {
                *decimals = value;
                true
            })
        }
        (RuntimeDataBindGraphConverter::ToString { color_format, .. }, 766) => string(value)
            .is_some_and(|value| {
                *color_format = value;
                true
            }),
        (
            RuntimeDataBindGraphConverter::OperationValue { operation_type, .. }
            | RuntimeDataBindGraphConverter::SystemOperationValue { operation_type, .. }
            | RuntimeDataBindGraphConverter::OperationViewModel { operation_type, .. },
            682,
        ) => uint(value).is_some_and(|value| {
            *operation_type = value;
            true
        }),
        (
            RuntimeDataBindGraphConverter::OperationValue {
                operation_value, ..
            }
            | RuntimeDataBindGraphConverter::SystemOperationValue {
                operation_value, ..
            },
            681,
        ) => number(value).is_some_and(|value| {
            *operation_value = value;
            true
        }),
        (RuntimeDataBindGraphConverter::Rounder { decimals }, 669) => {
            uint(value).is_some_and(|value| {
                *decimals = value;
                true
            })
        }
        (
            RuntimeDataBindGraphConverter::RangeMapper {
                interpolation_type, ..
            },
            713,
        ) => uint(value).is_some_and(|value| {
            *interpolation_type = value;
            true
        }),
        (RuntimeDataBindGraphConverter::RangeMapper { flags, .. }, 715) => {
            uint(value).is_some_and(|value| {
                *flags = value;
                true
            })
        }
        (RuntimeDataBindGraphConverter::RangeMapper { min_input, .. }, 716) => number(value)
            .is_some_and(|value| {
                *min_input = value;
                true
            }),
        (RuntimeDataBindGraphConverter::RangeMapper { max_input, .. }, 717) => number(value)
            .is_some_and(|value| {
                *max_input = value;
                true
            }),
        (RuntimeDataBindGraphConverter::RangeMapper { min_output, .. }, 718) => number(value)
            .is_some_and(|value| {
                *min_output = value;
                true
            }),
        (RuntimeDataBindGraphConverter::RangeMapper { max_output, .. }, 719) => number(value)
            .is_some_and(|value| {
                *max_output = value;
                true
            }),
        (RuntimeDataBindGraphConverter::StringTrim { trim_type, .. }, 746) => uint(value)
            .is_some_and(|value| {
                *trim_type = value;
                true
            }),
        (RuntimeDataBindGraphConverter::StringPad { length, .. }, 743) => {
            uint(value).is_some_and(|value| {
                *length = value;
                true
            })
        }
        (RuntimeDataBindGraphConverter::StringPad { text, .. }, 744) => {
            string(value).is_some_and(|value| {
                *text = value;
                true
            })
        }
        (RuntimeDataBindGraphConverter::StringPad { pad_type, .. }, 745) => uint(value)
            .is_some_and(|value| {
                *pad_type = value;
                true
            }),
        (RuntimeDataBindGraphConverter::Formula { tokens }, 887) => {
            uint(value).is_some_and(|value| {
                for token in tokens {
                    if let crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Function {
                        random_mode,
                        ..
                    } = token
                    {
                        *random_mode = value;
                    }
                }
                true
            })
        }
        (RuntimeDataBindGraphConverter::Interpolator { duration, .. }, 756) => number(value)
            .is_some_and(|value| {
                *duration = value;
                true
            }),
        _ => false,
    }
}

fn read_modeled_converter_property(
    converter: &RuntimeDataBindGraphConverter,
    target: RuntimeDataConverterBindingTarget,
    property_key: u32,
) -> Option<RuntimeConverterPropertyValue> {
    let as_number = |value: f32| Some(RuntimeConverterPropertyValue::Double(value));
    let as_uint = |value: u64| Some(RuntimeConverterPropertyValue::Uint(value));
    if let RuntimeDataConverterBindingTarget::FormulaToken { token_index } = target {
        let RuntimeDataBindGraphConverter::Formula { tokens } = converter else {
            return None;
        };
        return match (tokens.get(token_index)?, property_key) {
            (
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Value { value, .. },
                777,
            ) => as_number(*value),
            (
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Operation {
                    operation_type,
                },
                775,
            ) => as_uint(*operation_type),
            (
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Function {
                    function_type,
                    ..
                },
                776,
            ) => as_uint(*function_type),
            _ => None,
        };
    }
    if target != RuntimeDataConverterBindingTarget::SelfProperty {
        return None;
    }
    match (converter, property_key) {
        (RuntimeDataBindGraphConverter::NumberToList { view_model_id, .. }, 816) => {
            as_uint(*view_model_id)
        }
        (RuntimeDataBindGraphConverter::ToString { flags, .. }, 764) => as_uint(*flags),
        (RuntimeDataBindGraphConverter::ToString { decimals, .. }, 765) => as_uint(*decimals),
        (RuntimeDataBindGraphConverter::ToString { color_format, .. }, 766) => Some(
            RuntimeConverterPropertyValue::String(Arc::from(color_format.as_slice())),
        ),
        (
            RuntimeDataBindGraphConverter::OperationValue { operation_type, .. }
            | RuntimeDataBindGraphConverter::SystemOperationValue { operation_type, .. }
            | RuntimeDataBindGraphConverter::OperationViewModel { operation_type, .. },
            682,
        ) => as_uint(*operation_type),
        (
            RuntimeDataBindGraphConverter::OperationValue {
                operation_value, ..
            }
            | RuntimeDataBindGraphConverter::SystemOperationValue {
                operation_value, ..
            },
            681,
        ) => as_number(*operation_value),
        (RuntimeDataBindGraphConverter::Rounder { decimals }, 669) => as_uint(*decimals),
        (
            RuntimeDataBindGraphConverter::RangeMapper {
                interpolation_type, ..
            },
            713,
        ) => as_uint(*interpolation_type),
        (RuntimeDataBindGraphConverter::RangeMapper { flags, .. }, 715) => as_uint(*flags),
        (RuntimeDataBindGraphConverter::RangeMapper { min_input, .. }, 716) => {
            as_number(*min_input)
        }
        (RuntimeDataBindGraphConverter::RangeMapper { max_input, .. }, 717) => {
            as_number(*max_input)
        }
        (RuntimeDataBindGraphConverter::RangeMapper { min_output, .. }, 718) => {
            as_number(*min_output)
        }
        (RuntimeDataBindGraphConverter::RangeMapper { max_output, .. }, 719) => {
            as_number(*max_output)
        }
        (RuntimeDataBindGraphConverter::StringTrim { trim_type, .. }, 746) => as_uint(*trim_type),
        (RuntimeDataBindGraphConverter::StringPad { length, .. }, 743) => as_uint(*length),
        (RuntimeDataBindGraphConverter::StringPad { text, .. }, 744) => Some(
            RuntimeConverterPropertyValue::String(Arc::from(text.as_slice())),
        ),
        (RuntimeDataBindGraphConverter::StringPad { pad_type, .. }, 745) => as_uint(*pad_type),
        // Formula::randomMode is an independent generated field. The
        // occurrence-level target map owns it even when there are no function
        // tokens; applying the field still propagates the value into every
        // random token used by conversion.
        (RuntimeDataBindGraphConverter::Formula { .. }, 887) => None,
        (RuntimeDataBindGraphConverter::Interpolator { duration, .. }, 756) => as_number(*duration),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_bind_graph::{DATA_BIND_FLAG_DIRECTION_TO_SOURCE, DATA_BIND_FLAG_TWO_WAY};
    use crate::script_asset::RuntimeScriptImplementedMethods;
    use crate::scripted_data_converter::{
        RuntimeScriptedDataConverterDataBindDefinition, RuntimeScriptedDataConverterDefinition,
        RuntimeScriptedDataConverterInputDefinition,
    };
    use crate::scripted_object::{RuntimeScriptInputProperties, RuntimeScriptInputTargetProperty};
    use crate::scripting::{
        ScriptHost, ScriptInstance, ScriptListenerInputKind, ScriptMethod, ScriptValue,
    };
    use nuxie_binary::read_runtime_file;
    use nuxie_schema::definition_by_name;
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;

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

    fn property_key(type_name: &str, property_name: &str) -> u16 {
        let definition = definition_by_name(type_name).expect("test schema definition");
        definition
            .properties
            .iter()
            .chain(definition.ancestors.iter().flat_map(|ancestor| {
                definition_by_name(ancestor)
                    .expect("test ancestor definition")
                    .properties
                    .iter()
            }))
            .find(|property| property.name == property_name)
            .expect("test schema property")
            .key
            .int
    }

    fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
        push_var_uint(
            bytes,
            u64::from(
                definition_by_name(type_name)
                    .expect("test schema definition")
                    .type_key
                    .int,
            ),
        );
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

    fn push_string(bytes: &mut Vec<u8>, type_name: &str, name: &str, value: &str) {
        push_var_uint(bytes, u64::from(property_key(type_name, name)));
        push_var_uint(bytes, value.len() as u64);
        bytes.extend_from_slice(value.as_bytes());
    }

    fn file_header(id: u64) -> Vec<u8> {
        let mut bytes = b"RIVE".to_vec();
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, id);
        push_var_uint(&mut bytes, 0);
        bytes
    }

    fn push_context_bind(bytes: &mut Vec<u8>, property_key: u16) {
        push_object(bytes, "DataBindContext", |bytes| {
            push_uint(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(property_key),
            );
        });
    }

    fn number_binding_definition(
        flags: u64,
        target: RuntimeDataConverterBindingTarget,
        property_key: u32,
    ) -> RuntimeDataConverterBindingDefinition {
        RuntimeDataConverterBindingDefinition::Context {
            source_path: Some(vec![0, 0]),
            name_based: false,
            property_key,
            flags,
            target,
            initial_target: Some(RuntimeConverterPropertyValue::Double(0.0)),
        }
    }

    fn bind_number_source(
        state: &mut RuntimeDataConverterDataBindState,
        binding_index: usize,
        source: &RuntimeViewModelCell,
    ) {
        bind_source(state, binding_index, source);
    }

    fn bind_source(
        state: &mut RuntimeDataConverterDataBindState,
        binding_index: usize,
        source: &RuntimeViewModelCell,
    ) {
        let RuntimeDataConverterBindingState::Context { retained_bind, .. } =
            &mut state.bindings[binding_index]
        else {
            panic!("expected context binding at {binding_index}");
        };
        retained_bind.set_source(source.clone());
    }

    fn scripted_number_input(
        input_global_id: u32,
        name: &str,
        authored_order: u32,
    ) -> RuntimeScriptedDataConverterInputDefinition {
        RuntimeScriptedDataConverterInputDefinition {
            input_global_id,
            kind: ScriptListenerInputKind::Number,
            properties: RuntimeScriptInputProperties::for_test(
                name,
                u32::MAX,
                Some(crate::data_bind_graph::RuntimeDataBindGraphValue::Number(
                    0.0,
                )),
            ),
            data_binds: vec![RuntimeScriptedDataConverterDataBindDefinition::Context {
                authored_order,
                source_path: Some(vec![0, input_global_id]),
                name_based: false,
                property_key: u32::from(property_key("ScriptInputNumber", "propertyValue")),
                target_property: RuntimeScriptInputTargetProperty::Value,
                flags: 0,
                converter_id: u32::MAX,
            }],
        }
    }

    #[test]
    fn bind_plan_preserves_base_then_group_then_scripted_virtual_call_order() {
        let scripted = RuntimeDataBindGraphConverter::Scripted {
            global_id: 70,
            serialized_implemented_methods: RuntimeScriptImplementedMethods::METHOD_MASK,
            definition: RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(vec![
                scripted_number_input(80, "amount", 91),
            ]),
            instance: None,
        };
        let converter = RuntimeDataBindGraphConverter::Group(vec![
            RuntimeDataBindGraphConverter::PassThrough,
            scripted,
        ]);

        assert_eq!(
            runtime_data_converter_bind_steps(&converter),
            vec![
                RuntimeDataConverterBindStep::BindOwn { path: vec![] },
                RuntimeDataConverterBindStep::BindOwn { path: vec![0] },
                RuntimeDataConverterBindStep::BindOwn { path: vec![1] },
                RuntimeDataConverterBindStep::Rehydrate {
                    path: vec![1],
                    converter_global_id: 70,
                    inits: true,
                },
                RuntimeDataConverterBindStep::RebindFinalInput {
                    path: vec![1],
                    input_index: 0,
                    data_bind_index: 0,
                },
            ]
        );
    }

    struct NoopInputInstance;

    impl ScriptInstance for NoopInputInstance {
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

    struct DropSeesDependentCount {
        source: RuntimeViewModelCell,
        observed: Rc<Cell<Option<usize>>>,
    }

    impl Drop for DropSeesDependentCount {
        fn drop(&mut self) {
            self.observed.set(Some(self.source.dependent_count()));
        }
    }

    impl ScriptInstance for DropSeesDependentCount {
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

    fn retained_source(
        state: &RuntimeDataConverterDataBindState,
        binding_index: usize,
    ) -> Option<&RuntimeViewModelCell> {
        let RuntimeDataConverterBindingState::Context { retained_bind, .. } =
            &state.bindings[binding_index]
        else {
            panic!("expected context binding at {binding_index}");
        };
        retained_bind.source()
    }

    #[test]
    fn source_owner_unbinds_scripted_custom_inputs_before_destroying_table() {
        let inner_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(3.0));
        let observed = Rc::new(Cell::new(None));
        let handle = RuntimeScriptInstanceHandle::new(Box::new(DropSeesDependentCount {
            source: inner_source.clone(),
            observed: Rc::clone(&observed),
        }));
        let definition =
            RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(vec![
                scripted_number_input(80, "amount", 91),
            ]);
        let converter = RuntimeDataBindGraphConverter::Scripted {
            global_id: 70,
            serialized_implemented_methods: RuntimeScriptImplementedMethods::METHOD_MASK,
            definition: definition.clone(),
            instance: Some(handle.clone()),
        };
        let mut converter_data_binds =
            RuntimeDataConverterDataBindDefinition::for_scripted_definition(&definition)
                .instantiate();
        bind_number_source(&mut converter_data_binds, 0, &inner_source);
        assert_eq!(inner_source.dependent_count(), 1);

        let mut sources = Vec::new();
        let mut targets = Vec::new();
        let mut bindings = Vec::new();
        crate::data_bind_graph::RuntimeDataBindGraph::push_default_view_model_binding(
            &mut sources,
            &mut targets,
            &mut bindings,
            0,
            &[],
            0,
            Some(converter),
            crate::data_bind_graph::RuntimeDataBindGraphTarget::Number { global_id: 7 },
            crate::data_bind_graph::RuntimeDataBindGraphValue::Number(0.0),
        );
        sources[0].converter_data_binds = converter_data_binds;
        drop(handle);
        drop(sources);

        assert_eq!(
            observed.get(),
            Some(0),
            "DataBind::~DataBind clears ScriptedDataConverter custom-input sources before deleting the live table (`data_bind.cpp:239-249,354-369`)"
        );
        assert_eq!(inner_source.dependent_count(), 0);
    }

    #[test]
    fn formula_clone_keeps_non_output_token_binds_self_targeted_in_file_order() {
        let mut bytes = file_header(95_101);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "DataConverterFormula", |bytes| {
            push_uint(bytes, "DataConverterFormula", "randomModeValue", 1);
        });
        push_context_bind(
            &mut bytes,
            property_key("DataConverterFormula", "randomModeValue"),
        );
        push_object(&mut bytes, "FormulaTokenValue", |bytes| {
            push_f32(bytes, "FormulaTokenValue", "operationValue", 2.0);
        });
        push_context_bind(
            &mut bytes,
            property_key("FormulaTokenValue", "operationValue"),
        );
        for token_type in [
            "FormulaTokenParenthesisOpen",
            "FormulaTokenArgumentSeparator",
            "FormulaTokenParenthesisClose",
        ] {
            push_object(&mut bytes, token_type, |_| {});
            push_context_bind(
                &mut bytes,
                property_key("DataConverterFormula", "randomModeValue"),
            );
        }
        let file = read_runtime_file(&bytes).expect("formula ownership fixture");
        let converter_object = file.data_converter(0).expect("formula converter");
        let value_token = file
            .data_converter_formula_tokens_for_object(converter_object)
            .into_iter()
            .find(|token| token.type_name == "FormulaTokenValue")
            .expect("value token");
        let converter = RuntimeDataBindGraphConverter::Formula {
            tokens: vec![
                crate::data_bind_graph::RuntimeDataBindGraphFormulaToken::Value {
                    token_id: value_token.id,
                    value: 2.0,
                },
            ],
        };
        let definition =
            runtime_data_converter_data_bind_definition(&file, converter_object, &converter);

        assert_eq!(definition.bindings.len(), 5);
        assert!(matches!(
            definition.bindings[0],
            RuntimeDataConverterBindingDefinition::Context {
                target: RuntimeDataConverterBindingTarget::SelfProperty,
                ..
            }
        ));
        assert!(matches!(
            definition.bindings[1],
            RuntimeDataConverterBindingDefinition::Context {
                target: RuntimeDataConverterBindingTarget::FormulaToken { token_index: 0 },
                ..
            }
        ));
        assert!(definition.bindings[2..].iter().all(|binding| matches!(
            binding,
            RuntimeDataConverterBindingDefinition::Context {
                target: RuntimeDataConverterBindingTarget::SelfProperty,
                initial_target: Some(RuntimeConverterPropertyValue::Uint(1)),
                ..
            }
        )));
    }

    #[test]
    fn unsupported_scripted_converter_keeps_all_input_binds_and_inert_final_occurrence() {
        let mut bytes = file_header(95_102);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ScriptedDataConverter", |_| {});
        push_object(&mut bytes, "ScriptInputNumber", |bytes| {
            push_string(bytes, "ScriptInputNumber", "name", "amount");
            push_f32(bytes, "ScriptInputNumber", "propertyValue", 4.0);
        });
        push_context_bind(
            &mut bytes,
            property_key("ScriptInputNumber", "propertyValue"),
        );
        push_object(&mut bytes, "DataBind", |_| {});

        let file = read_runtime_file(&bytes).expect("scripted ownership fixture");
        let converter_object = file.data_converter(0).expect("scripted converter");
        let definition = runtime_data_converter_data_bind_definition(
            &file,
            converter_object,
            &RuntimeDataBindGraphConverter::Unsupported,
        );

        assert_eq!(definition.bindings.len(), 2);
        assert!(matches!(
            definition.bindings[0],
            RuntimeDataConverterBindingDefinition::Context {
                target: RuntimeDataConverterBindingTarget::ScriptedInput {
                    input_index: 0,
                    data_bind_index: 0,
                },
                ..
            }
        ));
        assert!(matches!(
            definition.bindings[1],
            RuntimeDataConverterBindingDefinition::Inert
        ));
        let detached = definition
            .detached_scripted_definition
            .as_ref()
            .expect("unsupported lowering retains concrete ScriptedDataConverter ownership");
        assert_eq!(detached.inputs.len(), 1);
        assert_eq!(detached.inputs[0].data_binds.len(), 2);
        assert!(matches!(
            detached.inputs[0].data_binds.last(),
            Some(
                crate::scripted_data_converter::RuntimeScriptedDataConverterDataBindDefinition::Inert {
                    ..
                }
            )
        ));
        assert!(definition.instantiate().detached_scripted.is_some());
    }

    #[test]
    fn scripted_custom_bind_clones_complete_flags_target_and_value() {
        const ALL_AUTHORED_FLAGS: u64 = (1 << 5) - 1;
        const AUTHORED_CONVERTER_ID: u64 = 77;
        let authored_property_key = u32::from(property_key("ScriptInputNumber", "propertyValue"));

        let mut bytes = file_header(95_103);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ScriptedDataConverter", |_| {});
        push_object(&mut bytes, "ScriptInputNumber", |bytes| {
            push_string(bytes, "ScriptInputNumber", "name", "amount");
            push_f32(bytes, "ScriptInputNumber", "propertyValue", 4.25);
        });
        push_object(&mut bytes, "DataBindContext", |bytes| {
            push_uint(
                bytes,
                "DataBindContext",
                "propertyKey",
                u64::from(authored_property_key),
            );
            push_uint(bytes, "DataBindContext", "flags", ALL_AUTHORED_FLAGS);
            push_uint(
                bytes,
                "DataBindContext",
                "converterId",
                AUTHORED_CONVERTER_ID,
            );
        });

        let file = read_runtime_file(&bytes).expect("scripted clone fixture");
        let converter_object = file.data_converter(0).expect("scripted converter");
        let scripted_definition =
            crate::data_bind_graph::runtime_scripted_data_converter_input_definitions(
                &file,
                converter_object,
            );
        assert_eq!(scripted_definition.inputs.len(), 1);
        assert_eq!(
            scripted_definition.inputs[0].properties.value(),
            Some(&crate::data_bind_graph::RuntimeDataBindGraphValue::Number(
                4.25
            )),
            "the immutable converter definition retains the authored custom-input target"
        );
        assert!(matches!(
            scripted_definition.inputs[0].data_binds.as_slice(),
            [RuntimeScriptedDataConverterDataBindDefinition::Context {
                property_key: bound_property_key,
                flags: ALL_AUTHORED_FLAGS,
                target_property: RuntimeScriptInputTargetProperty::Value,
                converter_id: bound_converter_id,
                ..
            }] if *bound_property_key == authored_property_key
                && *bound_converter_id == AUTHORED_CONVERTER_ID as u32
        ));
        let converter = RuntimeDataBindGraphConverter::Scripted {
            global_id: converter_object.id,
            serialized_implemented_methods: RuntimeScriptImplementedMethods::METHOD_MASK,
            definition: scripted_definition,
            instance: None,
        };
        let RuntimeDataBindGraphConverterState::Scripted(live_input_state) =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter))
        else {
            panic!("expected scripted converter occurrence state");
        };
        assert!(matches!(
            live_input_state.input_snapshots().as_slice(),
            [crate::ScriptListenerInputSnapshot {
                value: Some(crate::ScriptListenerInputSnapshotValue::Value(
                    ScriptValue::Number(4.25)
                )),
                ..
            }]
        ));
        assert_eq!(
            live_input_state.data_bind_metadata_for_test(),
            vec![vec![(
                authored_property_key,
                RuntimeScriptInputTargetProperty::Value,
                ALL_AUTHORED_FLAGS,
                AUTHORED_CONVERTER_ID as u32,
            )]],
            "the concrete converter occurrence retains every serialized DataBindBase field even though DataBindBase::copy intentionally does not retain the resolved converter pointer"
        );
        let definition =
            runtime_data_converter_data_bind_definition(&file, converter_object, &converter);
        assert!(matches!(
            definition.bindings.as_slice(),
            [RuntimeDataConverterBindingDefinition::Context {
                flags: ALL_AUTHORED_FLAGS,
                target: RuntimeDataConverterBindingTarget::ScriptedInput {
                    input_index: 0,
                    data_bind_index: 0,
                },
                initial_target: Some(RuntimeConverterPropertyValue::Double(4.25)),
                ..
            }]
        ));

        let occurrence = definition.instantiate();
        assert!(matches!(
            occurrence.bindings.as_slice(),
            [RuntimeDataConverterBindingState::Context {
                flags: ALL_AUTHORED_FLAGS,
                target: RuntimeDataConverterBindingTarget::ScriptedInput {
                    input_index: 0,
                    data_bind_index: 0,
                },
                ..
            }]
        ));
        assert_eq!(
            occurrence.target_values.get(&(
                RuntimeDataConverterBindingTarget::ScriptedInput {
                    input_index: 0,
                    data_bind_index: 0,
                },
                u32::from(property_key("ScriptInputNumber", "propertyValue")),
            )),
            Some(&RuntimeConverterPropertyValue::Double(4.25)),
        );

        let cold = occurrence.fresh_clone();
        assert!(matches!(
            cold.bindings.as_slice(),
            [RuntimeDataConverterBindingState::Context {
                flags: ALL_AUTHORED_FLAGS,
                target: RuntimeDataConverterBindingTarget::ScriptedInput {
                    input_index: 0,
                    data_bind_index: 0,
                },
                retained_bind,
                ..
            }] if !retained_bind.has_sources()
        ));
        assert_eq!(
            cold.target_values, occurrence.target_values,
            "a fresh converter occurrence clones the authored target value but no live source"
        );
        let cold_converter = converter.clone();
        let RuntimeDataBindGraphConverterState::Scripted(cold_input_state) =
            RuntimeDataBindGraphConverterState::for_converter(Some(&cold_converter))
        else {
            panic!("expected cold scripted converter occurrence state");
        };
        assert!(matches!(
            cold_input_state.input_snapshots().as_slice(),
            [crate::ScriptListenerInputSnapshot {
                value: Some(crate::ScriptListenerInputSnapshotValue::Value(
                    ScriptValue::Number(4.25)
                )),
                ..
            }]
        ));
        assert_eq!(
            cold_input_state.data_bind_metadata_for_test(),
            live_input_state.data_bind_metadata_for_test(),
            "every fresh ScriptedDataConverter occurrence clones propertyKey, flags, target identity, and serialized converterId"
        );
        // `DataBindBase::copy` preserves generated fields and repairs the
        // clone's target to the copied ScriptInput, while each occurrence
        // starts with an unbound retained source (`data_bind.cpp:57-97`;
        // `data_converter.cpp:59-69`;
        // `scripted_data_converter.cpp:235-273`).
    }

    #[test]
    fn scripted_converter_self_binds_keep_order_fields_and_live_table_identity() {
        let mut bytes = file_header(95_105);
        push_object(&mut bytes, "Backboard", |_| {});
        push_object(&mut bytes, "ScriptedDataConverter", |bytes| {
            push_string(bytes, "ScriptedDataConverter", "name", "authored");
            push_uint(bytes, "ScriptedDataConverter", "scriptAssetId", 7);
        });
        push_context_bind(&mut bytes, property_key("ScriptedDataConverter", "name"));
        push_context_bind(
            &mut bytes,
            property_key("ScriptedDataConverter", "scriptAssetId"),
        );

        let file = read_runtime_file(&bytes).expect("scripted self-bind fixture");
        let converter_object = file.data_converter(0).expect("scripted converter");
        let handle = RuntimeScriptInstanceHandle::new(Box::new(NoopInputInstance));
        let mut converter = RuntimeDataBindGraphConverter::Scripted {
            global_id: converter_object.id,
            serialized_implemented_methods: RuntimeScriptImplementedMethods::METHOD_MASK,
            definition: RuntimeScriptedDataConverterDefinition::default(),
            instance: Some(handle.clone()),
        };
        let definition =
            runtime_data_converter_data_bind_definition(&file, converter_object, &converter);

        assert!(matches!(
            definition.bindings.as_slice(),
            [
                RuntimeDataConverterBindingDefinition::Context {
                    property_key: 662,
                    target: RuntimeDataConverterBindingTarget::SelfProperty,
                    initial_target: Some(RuntimeConverterPropertyValue::String(name)),
                    ..
                },
                RuntimeDataConverterBindingDefinition::Context {
                    property_key: 892,
                    target: RuntimeDataConverterBindingTarget::SelfProperty,
                    initial_target: Some(RuntimeConverterPropertyValue::Uint(7)),
                    ..
                },
            ] if name.as_ref() == b"authored"
        ));

        let mut occurrence = definition.instantiate();
        let name_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::String(Arc::from(
            b"before".as_slice(),
        )));
        let asset_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        bind_source(&mut occurrence, 0, &name_source);
        bind_source(&mut occurrence, 1, &asset_source);
        assert!(
            name_source.set_value(RuntimeViewModelCellValue::String(Arc::from(
                b"live".as_slice(),
            )))
        );
        assert!(asset_source.set_value(RuntimeViewModelCellValue::Number(3.6)));

        let mut converter_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
        let mut apply =
            |_instance: &RuntimeScriptInstanceHandle,
             _name: &ScriptCoreString,
             _value: RuntimeScriptedListenerBoundValue| { Ok(()) };
        occurrence
            .public_update(
                &mut converter,
                &mut converter_state,
                &file,
                None,
                false,
                &mut apply,
            )
            .expect("scripted converter self-bind update");

        assert_eq!(
            occurrence.target_values.get(&(
                RuntimeDataConverterBindingTarget::SelfProperty,
                u32::from(property_key("ScriptedDataConverter", "name")),
            )),
            Some(&RuntimeConverterPropertyValue::String(Arc::from(
                b"live".as_slice()
            ))),
        );
        assert_eq!(
            occurrence.target_values.get(&(
                RuntimeDataConverterBindingTarget::SelfProperty,
                u32::from(property_key("ScriptedDataConverter", "scriptAssetId")),
            )),
            Some(&RuntimeConverterPropertyValue::Uint(4)),
            "DataBindContextValueNumber clamps negatives and rounds before CoreRegistry::setUint (`context_value_number.cpp:18-37`)"
        );
        assert!(
            matches!(
                &converter,
                RuntimeDataBindGraphConverter::Scripted {
                    instance: Some(current),
                    ..
                } if current == &handle
            ),
            "changing generated scriptAssetId updates the occurrence field but does not relink or recreate the already initialized ScriptedDataConverter table"
        );

        let cold = occurrence.fresh_clone();
        assert!(cold.bindings.iter().all(|binding| matches!(
            binding,
            RuntimeDataConverterBindingState::Context { retained_bind, .. }
                if !retained_bind.has_sources()
        )));
        assert_eq!(
            cold.target_values, occurrence.target_values,
            "DataConverter::copy clones the occurrence's current generated fields while every cloned DataBind starts unbound"
        );

        let second = definition.instantiate();
        assert_eq!(
            second.target_values.get(&(
                RuntimeDataConverterBindingTarget::SelfProperty,
                u32::from(property_key("ScriptedDataConverter", "name")),
            )),
            Some(&RuntimeConverterPropertyValue::String(Arc::from(
                b"authored".as_slice()
            ))),
        );
        assert_eq!(
            second.target_values.get(&(
                RuntimeDataConverterBindingTarget::SelfProperty,
                u32::from(property_key("ScriptedDataConverter", "scriptAssetId")),
            )),
            Some(&RuntimeConverterPropertyValue::Uint(7)),
            "a separate occurrence instantiates from the immutable authored definition rather than aliasing the first occurrence's generated fields"
        );
    }

    #[test]
    fn formula_random_mode_is_an_occurrence_field_even_without_random_tokens() {
        let mut converter = RuntimeDataBindGraphConverter::Formula { tokens: Vec::new() };
        let mut target_values = BTreeMap::from([(
            (RuntimeDataConverterBindingTarget::SelfProperty, 887),
            RuntimeConverterPropertyValue::Uint(0),
        )]);
        let mut target = RuntimeConverterPropertyTarget {
            converter: &mut converter,
            target: RuntimeDataConverterBindingTarget::SelfProperty,
            property_key: 887,
            source_kind: Some(RuntimeViewModelCellValue::Enum(0)),
            target_values: &mut target_values,
            changed: false,
        };

        target.apply_to_target(&RuntimeViewModelCellValue::Enum(2));
        assert!(target.changed);
        assert_eq!(
            target.read_target(),
            Some(RuntimeViewModelCellValue::Enum(2)),
        );
        assert!(matches!(
            converter,
            RuntimeDataBindGraphConverter::Formula { ref tokens } if tokens.is_empty()
        ));
    }

    #[test]
    fn virtual_unbind_preserves_each_cpp_converter_subclass_boundary() {
        let ordinary_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let ordinary_definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![number_binding_definition(
                0,
                RuntimeDataConverterBindingTarget::SelfProperty,
                681,
            )],
            children: Vec::new(),
            detached_scripted_definition: None,
        };
        let mut ordinary_binds = ordinary_definition.instantiate();
        bind_number_source(&mut ordinary_binds, 0, &ordinary_source);
        let ordinary = RuntimeDataBindGraphConverter::PassThrough;
        let mut ordinary_state = RuntimeDataBindGraphConverterState::for_converter(Some(&ordinary));
        ordinary_binds.unbind(&ordinary, &mut ordinary_state);
        assert!(
            retained_source(&ordinary_binds, 0).is_none(),
            "DataConverter::unbind clears its own m_dataBinds"
        );

        let formula_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));
        let mut formula_binds = ordinary_definition.instantiate();
        bind_number_source(&mut formula_binds, 0, &formula_source);
        let formula = RuntimeDataBindGraphConverter::Formula { tokens: Vec::new() };
        let mut formula_state = RuntimeDataBindGraphConverterState::for_converter(Some(&formula));
        formula_binds.unbind(&formula, &mut formula_state);
        assert!(
            retained_source(&formula_binds, 0).is_some_and(|source| source.ptr_eq(&formula_source)),
            "DataConverterFormula::unbind removes only its separate outer-source dependency and deliberately leaves inherited token/self DataBinds bound (`data_converter_formula.cpp:545-553`)"
        );

        let group_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(3.0));
        let child_source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(4.0));
        let group_definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![number_binding_definition(
                0,
                RuntimeDataConverterBindingTarget::SelfProperty,
                681,
            )],
            children: vec![ordinary_definition.clone()],
            detached_scripted_definition: None,
        };
        let mut group_binds = group_definition.instantiate();
        bind_number_source(&mut group_binds, 0, &group_source);
        bind_number_source(&mut group_binds.children[0], 0, &child_source);
        let group =
            RuntimeDataBindGraphConverter::Group(vec![RuntimeDataBindGraphConverter::PassThrough]);
        let mut group_state = RuntimeDataBindGraphConverterState::for_converter(Some(&group));
        group_binds.unbind(&group, &mut group_state);
        assert!(
            retained_source(&group_binds, 0).is_some_and(|source| source.ptr_eq(&group_source)),
            "DataConverterGroup::unbind skips its inherited own list"
        );
        assert!(
            retained_source(&group_binds.children[0], 0).is_none(),
            "DataConverterGroup::unbind visits child occurrences in item order"
        );

        let operation_operand = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(5.0));
        let operation = RuntimeDataBindGraphConverter::OperationViewModel {
            operation_type: 0,
            operation_value: 5.0,
            default_operation_value: 0.0,
            source_path: Some(vec![0, 0]),
            retained_operation_value: Some(operation_operand.clone()),
        };
        let mut operation_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&operation));
        let mut operation_binds = RuntimeDataConverterDataBindState::default();
        operation_binds.unbind(&operation, &mut operation_state);
        let RuntimeDataBindGraphConverter::OperationViewModel {
            retained_operation_value,
            ..
        } = operation
        else {
            unreachable!();
        };
        assert!(
            retained_operation_value
                .as_ref()
                .is_some_and(|source| source.ptr_eq(&operation_operand)),
            "OperationViewModel has no unbind override; its retained operand remains attached to the converter occurrence (`data_converter_operation_viewmodel.cpp:48-59`)"
        );
    }

    #[test]
    fn converter_dirty_queue_preserves_arrival_order_with_to_source_first() {
        let definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![
                number_binding_definition(0, RuntimeDataConverterBindingTarget::SelfProperty, 716),
                number_binding_definition(
                    DATA_BIND_FLAG_DIRECTION_TO_SOURCE,
                    RuntimeDataConverterBindingTarget::SelfProperty,
                    717,
                ),
                number_binding_definition(0, RuntimeDataConverterBindingTarget::SelfProperty, 718),
                number_binding_definition(
                    DATA_BIND_FLAG_DIRECTION_TO_SOURCE,
                    RuntimeDataConverterBindingTarget::SelfProperty,
                    719,
                ),
            ],
            children: Vec::new(),
            detached_scripted_definition: None,
        };
        let mut state = definition.instantiate();
        let sources = (0..4)
            .map(|index| RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(index as f32)))
            .collect::<Vec<_>>();
        for (index, source) in sources.iter().enumerate() {
            bind_number_source(&mut state, index, source);
        }

        for index in [2, 0, 3, 1] {
            assert!(
                sources[index].set_value(RuntimeViewModelCellValue::Number(10.0 + index as f32))
            );
        }
        // A second mutation before the sink is drained is coalesced by the
        // same DataBind occurrence, just like `DataBind::addDirt`.
        assert!(sources[2].set_value(RuntimeViewModelCellValue::Number(20.0)));

        assert_eq!(
            state.take_dirty_binding_order(),
            [3, 1, 2, 0],
            "DataBindContainer drains target-to-source before target binds while retaining arrival order within each queue (`data_bind_container.cpp:156-203`)"
        );
        state.finish_dirty_bindings();
        assert!(state.take_dirty_binding_order().is_empty());
        state.finish_dirty_bindings();
    }

    #[test]
    fn converter_source_wakes_the_exact_outer_occurrence_before_inner_processing() {
        let definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![number_binding_definition(
                0,
                RuntimeDataConverterBindingTarget::SelfProperty,
                716,
            )],
            children: Vec::new(),
            detached_scripted_definition: None,
        };
        let mut converter_binds = definition.instantiate();
        let converter = RuntimeDataBindGraphConverter::PassThrough;
        let mut converter_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter));

        let outer_queue = RuntimeCellNotificationQueue::default();
        let mut outer = RuntimeRetainedDataBind::new(DATA_BIND_FLAG_TWO_WAY, false);
        outer.report_source_dirt_to(&outer_queue, 41);
        outer.mark_rebind_reconcile();
        let mut outer_reports = Vec::new();
        outer_queue.swap_into(&mut outer_reports);
        assert_eq!(outer_reports, [41]);
        assert!(outer.take_target_dirt());
        assert!(outer.take_pending_source_dirt());
        assert!(outer.target_origin());

        converter_binds.set_parent_wake(outer.converter_parent_wake(), &mut converter_state);
        let source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        bind_number_source(&mut converter_binds, 0, &source);

        assert!(source.set_value(RuntimeViewModelCellValue::Number(2.0)));
        outer_queue.swap_into(&mut outer_reports);
        assert_eq!(
            outer_reports,
            [41],
            "the parent DataBind is synchronously reported before the converter queue is drained"
        );
        assert!(outer.target_origin());
        assert!(
            outer
                .pending_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS_TARGET),
            "DataConverter::markConverterDirty preserves the outer direction"
        );
        assert_eq!(
            converter_binds.take_dirty_binding_order(),
            [0],
            "the same source mutation then enters the inner converter queue"
        );
        converter_binds.finish_dirty_bindings();
    }

    #[test]
    fn nested_converter_clone_rehomes_every_parent_wake_to_its_outer_occurrence() {
        let child_definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![number_binding_definition(
                0,
                RuntimeDataConverterBindingTarget::SelfProperty,
                716,
            )],
            children: Vec::new(),
            detached_scripted_definition: None,
        };
        let definition = RuntimeDataConverterDataBindDefinition {
            bindings: Vec::new(),
            children: vec![child_definition],
            detached_scripted_definition: None,
        };
        let converter =
            RuntimeDataBindGraphConverter::Group(vec![RuntimeDataBindGraphConverter::PassThrough]);
        let mut converter_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
        let mut first = definition.instantiate();
        let source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        bind_number_source(&mut first.children[0], 0, &source);

        let first_queue = RuntimeCellNotificationQueue::default();
        let mut first_outer = RuntimeRetainedDataBind::new(0, false);
        first_outer.report_source_dirt_to(&first_queue, 10);
        first.set_parent_wake(first_outer.converter_parent_wake(), &mut converter_state);

        let mut second = first.rehomed_clone();
        let mut second_converter_state = converter_state.clone();
        let second_queue = RuntimeCellNotificationQueue::default();
        let mut second_outer = RuntimeRetainedDataBind::new(0, false);
        second_outer.report_source_dirt_to(&second_queue, 20);
        second.set_parent_wake(
            second_outer.converter_parent_wake(),
            &mut second_converter_state,
        );

        assert!(source.set_value(RuntimeViewModelCellValue::Number(2.0)));
        let mut reports = Vec::new();
        first_queue.swap_into(&mut reports);
        assert_eq!(reports, [10]);
        second_queue.swap_into(&mut reports);
        assert_eq!(reports, [20]);
        assert_eq!(first.children[0].take_dirty_binding_order(), [0]);
        first.children[0].finish_dirty_bindings();
        assert_eq!(second.children[0].take_dirty_binding_order(), [0]);
        second.children[0].finish_dirty_bindings();
        assert!(
            first_outer
                .pending_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS)
        );
        assert!(
            second_outer
                .pending_dirt()
                .contains(crate::view_model_cell::RuntimeCellDirt::BINDINGS)
        );
    }

    #[test]
    fn converter_notifications_created_during_update_wait_for_the_next_pass() {
        let target = RuntimeDataConverterBindingTarget::SelfProperty;
        let definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![
                number_binding_definition(0, target, 716),
                number_binding_definition(DATA_BIND_FLAG_DIRECTION_TO_SOURCE, target, 716),
            ],
            children: Vec::new(),
            detached_scripted_definition: None,
        };
        let mut state = definition.instantiate();
        let writer = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        bind_number_source(&mut state, 0, &writer);

        assert!(writer.set_value(RuntimeViewModelCellValue::Number(2.0)));
        assert_eq!(state.take_dirty_binding_order(), [0]);
        state.begin_dirty_binding(0);

        // The active queue has already been swapped out. The writer's target
        // notification therefore enters the pending allocation and cannot
        // join the pass currently being processed.
        let mut converter = RuntimeDataBindGraphConverter::PassThrough;
        let mut converter_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
        state.notify_target_observers(0, target, 716, &mut converter, &mut converter_state);
        state.finish_dirty_bindings();
        assert_eq!(state.take_dirty_binding_order(), [1]);
        state.finish_dirty_bindings();
        assert!(state.take_dirty_binding_order().is_empty());
    }

    #[test]
    fn nested_entry_does_not_finish_the_outer_converter_pass() {
        let definition = RuntimeDataConverterDataBindDefinition {
            bindings: vec![number_binding_definition(
                0,
                RuntimeDataConverterBindingTarget::SelfProperty,
                716,
            )],
            children: Vec::new(),
            detached_scripted_definition: None,
        };
        let mut state = definition.instantiate();
        let source = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        bind_number_source(&mut state, 0, &source);
        assert!(source.set_value(RuntimeViewModelCellValue::Number(2.0)));
        assert_eq!(state.take_dirty_binding_order(), [0]);
        assert!(state.take_dirty_binding_order().is_empty());
        state.finish_dirty_bindings();
        assert!(state.processing, "inner rejection leaves outer pass active");
        state.begin_dirty_binding(0);
        state.finish_dirty_bindings();
        assert!(!state.processing);
    }

    #[test]
    fn scripted_input_setter_dirties_a_sibling_to_source_bind_for_the_next_pass() {
        let mut input = scripted_number_input(10, "value", 20);
        input
            .data_binds
            .push(RuntimeScriptedDataConverterDataBindDefinition::Context {
                authored_order: 21,
                source_path: Some(vec![0, 11]),
                name_based: false,
                property_key: u32::from(property_key("ScriptInputNumber", "propertyValue")),
                target_property: RuntimeScriptInputTargetProperty::Value,
                flags: DATA_BIND_FLAG_DIRECTION_TO_SOURCE,
                converter_id: u32::MAX,
            });
        let inputs = vec![input];
        let scripted_definition =
            RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(inputs);
        let definition =
            RuntimeDataConverterDataBindDefinition::for_scripted_definition(&scripted_definition);
        let mut state = definition.instantiate();
        let writer = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0));
        let sibling = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0));

        let handle = RuntimeScriptInstanceHandle::new(Box::new(NoopInputInstance));
        let mut converter = RuntimeDataBindGraphConverter::Scripted {
            global_id: 7,
            serialized_implemented_methods: RuntimeScriptImplementedMethods::METHOD_MASK,
            definition: scripted_definition,
            instance: Some(handle),
        };
        let mut converter_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
        let RuntimeDataBindGraphConverterState::Scripted(scripted_state) = &mut converter_state
        else {
            panic!("scripted converter state");
        };
        // C++ has one cloned DataBind occurrence: DataConverter owns it and
        // ScriptedDataConverter retargets that same pointer to the ScriptInput.
        // Rust keeps the container driver metadata separate from the retained
        // input bind, so construction must join the retained occurrence to the
        // driver's queue before binding. This is the same join performed by
        // `bind_own_sources` in production (`data_converter.cpp:59-68`;
        // `scripted_data_converter.cpp:235-269`).
        assert!(scripted_state.report_input_source_dirt_to(0, 0, &state.dirty_queue, 0));
        assert!(scripted_state.report_input_source_dirt_to(0, 1, &state.dirty_queue, 1));
        assert!(scripted_state.bind_test_input_source(0, 0, writer.clone()));
        assert!(scripted_state.bind_test_input_source(0, 1, sibling.clone()));
        assert!(writer.set_value(RuntimeViewModelCellValue::Number(10.0)));

        let mut bytes = file_header(95_104);
        push_object(&mut bytes, "Backboard", |_| {});
        let file = read_runtime_file(&bytes).expect("scripted sibling fixture");
        let mut apply =
            |_instance: &RuntimeScriptInstanceHandle,
             _name: &ScriptCoreString,
             _value: RuntimeScriptedListenerBoundValue| { Ok(()) };

        state
            .public_update(
                &mut converter,
                &mut converter_state,
                &file,
                None,
                true,
                &mut apply,
            )
            .expect("source-to-target setter pass");
        assert_eq!(
            sibling.value(),
            RuntimeViewModelCellValue::Number(2.0),
            "a sibling dirtied during the active traversal waits for the next snapshot"
        );
        state
            .public_update(
                &mut converter,
                &mut converter_state,
                &file,
                None,
                true,
                &mut apply,
            )
            .expect("deferred target-to-source sibling pass");
        assert_eq!(
            sibling.value(),
            RuntimeViewModelCellValue::Number(10.0),
            "the generated ScriptInput setter dirties the sibling DataBind exactly like C++ notifyPropertyChanged"
        );
    }

    #[test]
    fn error_requeues_the_unvisited_converter_tail_at_the_real_entry_point() {
        let inputs = vec![
            scripted_number_input(10, "first", 20),
            scripted_number_input(11, "second", 21),
        ];
        let scripted_definition =
            RuntimeScriptedDataConverterDefinition::with_grouped_test_bind_order(inputs);
        let definition =
            RuntimeDataConverterDataBindDefinition::for_scripted_definition(&scripted_definition);
        let mut state = definition.instantiate();
        let sources = [
            RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(1.0)),
            RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(2.0)),
        ];
        for (index, source) in sources.iter().enumerate() {
            bind_number_source(&mut state, index, source);
        }

        let handle = RuntimeScriptInstanceHandle::new(Box::new(NoopInputInstance));
        let mut converter = RuntimeDataBindGraphConverter::Scripted {
            global_id: 7,
            serialized_implemented_methods: RuntimeScriptImplementedMethods::METHOD_MASK,
            definition: scripted_definition,
            instance: Some(handle),
        };
        let mut converter_state =
            RuntimeDataBindGraphConverterState::for_converter(Some(&converter));
        let RuntimeDataBindGraphConverterState::Scripted(scripted_state) = &mut converter_state
        else {
            panic!("scripted converter state");
        };
        for (index, source) in sources.iter().enumerate() {
            assert!(scripted_state.bind_test_input_source(index, 0, source.clone()));
            assert!(source.set_value(RuntimeViewModelCellValue::Number(10.0 + index as f32)));
        }

        let mut bytes = file_header(95_103);
        push_object(&mut bytes, "Backboard", |_| {});
        let file = read_runtime_file(&bytes).expect("converter retry fixture");
        let fail_first = Rc::new(Cell::new(true));
        let applied = Rc::new(RefCell::new(Vec::new()));
        let mut apply = {
            let fail_first = Rc::clone(&fail_first);
            let applied = Rc::clone(&applied);
            move |_instance: &RuntimeScriptInstanceHandle,
                  name: &ScriptCoreString,
                  _value: RuntimeScriptedListenerBoundValue| {
                if fail_first.replace(false) {
                    return Err(ScriptError::new("synthetic converter failure"));
                }
                applied.borrow_mut().push(name.as_c_str_bytes().to_vec());
                Ok(())
            }
        };

        assert!(
            state
                .public_update(
                    &mut converter,
                    &mut converter_state,
                    &file,
                    None,
                    false,
                    &mut apply,
                )
                .is_err()
        );
        assert!(
            state
                .public_update(
                    &mut converter,
                    &mut converter_state,
                    &file,
                    None,
                    false,
                    &mut apply,
                )
                .is_ok(),
            "the failed occurrence and untouched authored tail stay eligible after cleanup"
        );
        assert_eq!(
            applied.borrow().as_slice(),
            [b"second".as_slice()],
            "the completed Core write is not replayed, while the untouched second occurrence applies"
        );
    }
}
