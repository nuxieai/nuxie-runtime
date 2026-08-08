use super::focus_action_clear::RuntimeFocusActionClear;
use super::focus_action_target::RuntimeFocusActionTarget;
use super::focus_action_traversal::RuntimeFocusActionTraversal;
use super::listener_align_target::RuntimeListenerAlignTarget;
use super::listener_bool_change::RuntimeListenerBoolChange;
use super::listener_fire_event::RuntimeListenerFireEvent;
use super::listener_input_change::RuntimeListenerInputTarget;
use super::listener_number_change::RuntimeListenerNumberChange;
use super::listener_trigger_change::RuntimeListenerTriggerChange;
use super::listener_viewmodel_change::{
    RuntimeListenerViewModelChange, runtime_listener_view_model_change_action,
};
use super::scripted_listener_action::runtime_scripted_listener_action_definition;
use super::state_machine_fire_action::StateMachineFireOccurrence;
use super::state_machine_fire_trigger::RuntimeStateMachineFireTriggerPath;
use super::{
    StateMachineBindableArtboardInstance, StateMachineBindableAssetInstance,
    StateMachineBindableBooleanInstance, StateMachineBindableColorInstance,
    StateMachineBindableEnumInstance, StateMachineBindableIntegerInstance,
    StateMachineBindableListInstance, StateMachineBindableNumberInstance,
    StateMachineBindableStringInstance, StateMachineBindableTriggerInstance,
    StateMachineBindableViewModelInstance, StateMachineInputInstance, StateMachineReportedEvent,
    StateMachineTransitionDurationInstance, TransitionEvaluationContext,
};
use crate::ArtboardInstance;
use crate::scripting::{ScriptError, ScriptListenerActionDefinition};
use crate::view_model_cell::RuntimeViewModelCell;
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::ArtboardGraph;

#[derive(Debug, Clone)]
pub(crate) enum RuntimeScheduledListenerAction {
    FireEvent(RuntimeListenerFireEvent),
    BoolChange(RuntimeListenerBoolChange),
    NumberChange(RuntimeListenerNumberChange),
    TriggerChange(RuntimeListenerTriggerChange),
    AlignTarget(RuntimeListenerAlignTarget),
    ViewModelChange(RuntimeListenerViewModelChange),
    Scripted {
        action_owner: super::RuntimeActionCoreHandle,
        definition: Option<ScriptListenerActionDefinition>,
    },
    FocusTarget(RuntimeFocusActionTarget),
    FocusClear(RuntimeFocusActionClear),
    FocusTraversal(RuntimeFocusActionTraversal),
    Noop {
        action_owner: super::RuntimeActionCoreHandle,
    },
}

pub(crate) struct RuntimeScheduledListenerActionTargetsMut<'a> {
    pub(crate) inputs: &'a mut [StateMachineInputInstance],
    pub(crate) reported_events: &'a mut Vec<StateMachineReportedEvent>,
    pub(crate) bindable_numbers: &'a mut [StateMachineBindableNumberInstance],
    pub(crate) bindable_integers: &'a mut [StateMachineBindableIntegerInstance],
    pub(crate) bindable_colors: &'a mut [StateMachineBindableColorInstance],
    pub(crate) bindable_strings: &'a mut [StateMachineBindableStringInstance],
    pub(crate) bindable_enums: &'a mut [StateMachineBindableEnumInstance],
    pub(crate) bindable_assets: &'a mut [StateMachineBindableAssetInstance],
    pub(crate) bindable_artboards: &'a mut [StateMachineBindableArtboardInstance],
    pub(crate) bindable_lists: &'a mut [StateMachineBindableListInstance],
    pub(crate) bindable_triggers: &'a mut [StateMachineBindableTriggerInstance],
    pub(crate) bindable_view_models: &'a mut [StateMachineBindableViewModelInstance],
    pub(crate) bindable_booleans: &'a mut [StateMachineBindableBooleanInstance],
    pub(crate) transition_durations: &'a mut [StateMachineTransitionDurationInstance],
}

impl RuntimeScheduledListenerActionTargetsMut<'_> {
    pub(crate) fn reborrow(&mut self) -> RuntimeScheduledListenerActionTargetsMut<'_> {
        RuntimeScheduledListenerActionTargetsMut {
            inputs: &mut *self.inputs,
            reported_events: &mut *self.reported_events,
            bindable_numbers: &mut *self.bindable_numbers,
            bindable_integers: &mut *self.bindable_integers,
            bindable_colors: &mut *self.bindable_colors,
            bindable_strings: &mut *self.bindable_strings,
            bindable_enums: &mut *self.bindable_enums,
            bindable_assets: &mut *self.bindable_assets,
            bindable_artboards: &mut *self.bindable_artboards,
            bindable_lists: &mut *self.bindable_lists,
            bindable_triggers: &mut *self.bindable_triggers,
            bindable_view_models: &mut *self.bindable_view_models,
            bindable_booleans: &mut *self.bindable_booleans,
            transition_durations: &mut *self.transition_durations,
        }
    }

    pub(crate) fn evaluation_context(
        &self,
        data_context_present: bool,
        layer_index: usize,
        view_model_trigger_layer_id: u64,
    ) -> TransitionEvaluationContext<'_> {
        TransitionEvaluationContext {
            bindable_numbers: self.bindable_numbers,
            bindable_integers: self.bindable_integers,
            bindable_colors: self.bindable_colors,
            bindable_strings: self.bindable_strings,
            bindable_enums: self.bindable_enums,
            bindable_assets: self.bindable_assets,
            bindable_artboards: self.bindable_artboards,
            bindable_triggers: self.bindable_triggers,
            bindable_view_models: self.bindable_view_models,
            bindable_booleans: self.bindable_booleans,
            data_context_present,
            layer_index,
            view_model_trigger_layer_id,
        }
    }
}

pub(crate) trait RuntimeScheduledListenerActionExecutor {
    /// Mirror `SMIInput::valueChanged()`: a genuine direct input mutation
    /// marks the owning StateMachineInstance, including actions performed
    /// while entering or resetting a state. Nested inputs mark their child
    /// occurrence instead and never call this hook.
    fn mark_direct_input_changed(&mut self) {}

    fn target_has_focus(&self, _target_local_id: usize) -> bool {
        false
    }

    fn evaluate_scripted_condition(&self, _global_id: u32) -> bool {
        false
    }

    fn retained_view_model_source(&self, _bindable_global_id: u32) -> Option<RuntimeViewModelCell> {
        None
    }

    fn fire_view_model_trigger(&mut self, _path: &RuntimeStateMachineFireTriggerPath) -> bool {
        false
    }

    fn requires_atomic_script_callbacks(&self) -> bool {
        false
    }

    fn perform_instance_action(
        &mut self,
        artboard: &mut ArtboardInstance,
        action: &RuntimeScheduledListenerAction,
        targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    ) -> Result<bool, ScriptError>;
}

impl RuntimeScheduledListenerAction {
    #[cfg(test)]
    pub(crate) fn scripted_for_test(
        flags: u64,
        definition: Option<ScriptListenerActionDefinition>,
    ) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ScriptedListenerAction");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        Self::Scripted {
            action_owner,
            definition,
        }
    }

    #[cfg(test)]
    pub(crate) fn noop_for_test(flags: u64) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerAction");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        Self::Noop { action_owner }
    }

    pub(crate) fn flags(&self) -> u64 {
        let action_owner = match self {
            Self::FireEvent(action) => &action.action_owner,
            Self::BoolChange(action) => &action.action_owner,
            Self::NumberChange(action) => &action.action_owner,
            Self::TriggerChange(action) => &action.action_owner,
            Self::AlignTarget(action) => &action.action_owner,
            Self::ViewModelChange(action) => &action.action_owner,
            Self::Scripted { action_owner, .. } | Self::Noop { action_owner } => action_owner,
            Self::FocusTarget(action) => &action.action_owner,
            Self::FocusClear(action) => &action.action_owner,
            Self::FocusTraversal(action) => &action.action_owner,
        };
        action_owner.uint(super::listener_action_owner::LISTENER_FLAGS_KEY)
    }

    pub(crate) fn validates_for_import(
        graph: &ArtboardGraph,
        state_machine_inputs: &[Option<&RuntimeObject>],
        action: &nuxie_binary::RuntimeListenerAction<'_>,
    ) -> bool {
        match action.object.type_name {
            "ListenerBoolChange" => RuntimeListenerInputTarget::from_object(action.object)
                .validates_for_import(
                    graph,
                    state_machine_inputs,
                    "StateMachineBool",
                    "NestedBool",
                ),
            "ListenerNumberChange" => RuntimeListenerInputTarget::from_object(action.object)
                .validates_for_import(
                    graph,
                    state_machine_inputs,
                    "StateMachineNumber",
                    "NestedNumber",
                ),
            "ListenerTriggerChange" => RuntimeListenerInputTarget::from_object(action.object)
                .validates_for_import(
                    graph,
                    state_machine_inputs,
                    "StateMachineTrigger",
                    "NestedTrigger",
                ),
            // `nuxie-binary` exposes only successfully imported actions.
            // C++ requires the BindableProperty importer to exist, but its
            // transferred pointer may already be null after an earlier
            // consumer; that later action still imports and is retained as a
            // no-op occurrence.
            "ListenerViewModelChange" => true,
            _ => true,
        }
    }

    pub(crate) fn from_imported(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        state_machine_inputs: &[Option<&RuntimeObject>],
        state_machine_data_binds: &[&RuntimeObject],
        action: &nuxie_binary::RuntimeListenerAction<'_>,
        action_owner: super::RuntimeActionCoreHandle,
    ) -> Self {
        debug_assert!(
            Self::validates_for_import(graph, state_machine_inputs, action),
            "nuxie-binary must project only listener actions accepted by the pinned C++ importer"
        );
        let imported = match action.object.type_name {
            "ListenerFireEvent" => Self::FireEvent(RuntimeListenerFireEvent { action_owner }),
            "ListenerBoolChange" => {
                let target = RuntimeListenerInputTarget::from_object(action.object);
                debug_assert!(target.validates_for_import(
                    graph,
                    state_machine_inputs,
                    "StateMachineBool",
                    "NestedBool",
                ));
                Self::BoolChange(RuntimeListenerBoolChange { action_owner })
            }
            "ListenerNumberChange" => {
                let target = RuntimeListenerInputTarget::from_object(action.object);
                debug_assert!(target.validates_for_import(
                    graph,
                    state_machine_inputs,
                    "StateMachineNumber",
                    "NestedNumber",
                ));
                Self::NumberChange(RuntimeListenerNumberChange { action_owner })
            }
            "ListenerTriggerChange" => {
                let target = RuntimeListenerInputTarget::from_object(action.object);
                debug_assert!(target.validates_for_import(
                    graph,
                    state_machine_inputs,
                    "StateMachineTrigger",
                    "NestedTrigger",
                ));
                Self::TriggerChange(RuntimeListenerTriggerChange { action_owner })
            }
            "ListenerAlignTarget" => Self::AlignTarget(RuntimeListenerAlignTarget { action_owner }),
            "ListenerViewModelChange" => runtime_listener_view_model_change_action(
                file,
                state_machine_data_binds,
                action,
                action_owner,
            ),
            "ScriptedListenerAction" => Self::Scripted {
                action_owner,
                definition: runtime_scripted_listener_action_definition(file, action.object, &[]),
            },
            "FocusActionTarget" => Self::FocusTarget(RuntimeFocusActionTarget { action_owner }),
            "FocusActionClear" => Self::FocusClear(RuntimeFocusActionClear { action_owner }),
            "FocusActionTraversal" => {
                Self::FocusTraversal(RuntimeFocusActionTraversal { action_owner })
            }
            _ => Self::Noop { action_owner },
        };
        imported
    }
}

pub(crate) fn perform_scheduled_listener_actions(
    listener_actions: &[RuntimeScheduledListenerAction],
    occurrence: StateMachineFireOccurrence,
    artboard: &mut ArtboardInstance,
    mut targets: RuntimeScheduledListenerActionTargetsMut<'_>,
    executor: &mut dyn RuntimeScheduledListenerActionExecutor,
) -> Result<bool, ScriptError> {
    let mut changed = false;
    for action in listener_actions {
        if action.flags() & 1 != occurrence.value() {
            continue;
        }
        match action {
            RuntimeScheduledListenerAction::FireEvent(action) => {
                if let Some(event) = action.perform(artboard) {
                    targets.reported_events.push(event);
                    changed = true;
                }
            }
            RuntimeScheduledListenerAction::BoolChange(action) => {
                let action_changed = action.perform(artboard, targets.inputs);
                if action_changed && action.targets_direct_input(artboard) {
                    executor.mark_direct_input_changed();
                }
                changed |= action_changed;
            }
            RuntimeScheduledListenerAction::NumberChange(action) => {
                let action_changed = action.perform(artboard, targets.inputs);
                if action_changed && action.targets_direct_input(artboard) {
                    executor.mark_direct_input_changed();
                }
                changed |= action_changed;
            }
            RuntimeScheduledListenerAction::TriggerChange(action) => {
                let action_changed = action.perform(artboard, targets.inputs);
                if action_changed && action.targets_direct_input(artboard) {
                    executor.mark_direct_input_changed();
                }
                changed |= action_changed;
            }
            RuntimeScheduledListenerAction::AlignTarget(_)
            | RuntimeScheduledListenerAction::ViewModelChange(_)
            | RuntimeScheduledListenerAction::Scripted { .. }
            | RuntimeScheduledListenerAction::FocusTarget(_)
            | RuntimeScheduledListenerAction::FocusClear(_)
            | RuntimeScheduledListenerAction::FocusTraversal(_) => {
                // C++ listener actions have a void perform boundary. Script
                // protected-call failures are consumed by the action and do
                // not truncate the authored FIFO. Rust's typed terminal
                // resource fence is the only error allowed to escape.
                match executor.perform_instance_action(artboard, action, targets.reborrow()) {
                    Ok(action_changed) => changed |= action_changed,
                    Err(error)
                        if error.resource_code().is_some()
                            || executor.requires_atomic_script_callbacks() =>
                    {
                        return Err(error);
                    }
                    Err(_) => {}
                }
            }
            RuntimeScheduledListenerAction::Noop { .. } => {}
        }
    }
    Ok(changed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue};
    use nuxie_graph::GraphFile;
    use std::sync::Arc;

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value,
        }
    }

    fn instantiate(records: Vec<FixtureRecord>) -> anyhow::Result<ArtboardInstance> {
        let file = RuntimeFile::from_fixture_records(records)?;
        let graph = GraphFile::from_runtime_file(&file)?;
        ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("action validation artboard"),
            &graph.artboards,
        )
    }

    fn prefix_with_input(input_type: &str) -> Vec<FixtureRecord> {
        vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Node",
                vec![property("Node", "parentId", FixtureValue::Uint(0))],
            ),
            record("StateMachine", Vec::new()),
            record(input_type, Vec::new()),
            record(
                "StateMachineListenerSingle",
                vec![
                    property(
                        "StateMachineListenerSingle",
                        "targetId",
                        FixtureValue::Uint(1),
                    ),
                    property(
                        "StateMachineListenerSingle",
                        "listenerTypeValue",
                        FixtureValue::Uint(15),
                    ),
                ],
            ),
        ]
    }

    fn prefix() -> Vec<FixtureRecord> {
        prefix_with_input("StateMachineNumber")
    }

    #[test]
    fn every_wrong_typed_listener_input_action_rejects_import() {
        for (input_type, action_type) in [
            ("StateMachineNumber", "ListenerBoolChange"),
            ("StateMachineBool", "ListenerNumberChange"),
            ("StateMachineBool", "ListenerTriggerChange"),
        ] {
            let mut records = prefix_with_input(input_type);
            records.push(record(
                action_type,
                vec![property(action_type, "inputId", FixtureValue::Uint(0))],
            ));
            let error =
                instantiate(records).expect_err("known wrong direct input type must reject");
            assert!(error.to_string().contains(action_type));
        }
    }

    #[test]
    fn every_out_of_range_listener_input_slot_is_retained_as_a_nullable_noop() {
        let mut records = prefix();
        for action_type in [
            "ListenerBoolChange",
            "ListenerNumberChange",
            "ListenerTriggerChange",
        ] {
            records.push(record(
                action_type,
                vec![property(action_type, "inputId", FixtureValue::Uint(99))],
            ));
        }
        let file = RuntimeFile::from_fixture_records(records)
            .expect("pinned C++ accepts forward-compatible null input slots");
        let state_machine = file
            .artboard_state_machine_graphs(0)
            .into_iter()
            .next()
            .expect("state machine");
        assert_eq!(
            state_machine.listeners[0].actions.len(),
            3,
            "none of the three nullable action occurrences may be compacted"
        );
    }

    #[test]
    fn listener_view_model_change_without_bindable_importer_rejects_import() {
        let mut records = prefix();
        records.push(record("ListenerViewModelChange", Vec::new()));
        let error =
            instantiate(records).expect_err("missing BindableProperty importer must reject");
        assert!(error.to_string().contains("ListenerViewModelChange"));
    }

    #[test]
    fn duplicate_view_model_actions_retain_the_consumed_null_occurrence() {
        let mut records = prefix();
        records.push(record(
            "BindablePropertyNumber",
            vec![property(
                "BindablePropertyNumber",
                "propertyValue",
                FixtureValue::Double(3.0),
            )],
        ));
        records.push(record("ListenerViewModelChange", Vec::new()));
        records.push(record("ListenerViewModelChange", Vec::new()));
        let file = RuntimeFile::from_fixture_records(records)
            .expect("the importer remains present after its pointer is consumed");
        let state_machine = file
            .artboard_state_machine_graphs(0)
            .into_iter()
            .next()
            .expect("state machine");
        let actions = &state_machine.listeners[0].actions;

        assert_eq!(actions.len(), 2);
        assert_eq!(
            actions[0]
                .bindable_property
                .map(|property| property.type_name),
            Some("BindablePropertyNumber")
        );
        assert!(
            actions[1].bindable_property.is_none(),
            "C++ retains the second action with the consumed importer pointer set to null"
        );
    }

    #[test]
    fn listener_parent_kind_requires_owner_and_raw_three_falls_back_to_listener() {
        let error = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record("FocusActionClear", Vec::new()),
        ])
        .expect_err("listener-kind action without current listener must reject");
        assert!(error.to_string().contains("FocusActionClear"));

        let mut records = prefix();
        records.push(record(
            "FocusActionClear",
            vec![property(
                "FocusActionClear",
                "flags",
                // `(flags >> 1) & 3 == 3` canonicalizes to Listener in
                // pinned `ListenerAction::parentKind`.
                FixtureValue::Uint(3 << 1),
            )],
        ));
        let file = RuntimeFile::from_fixture_records(records)
            .expect("reserved raw parent kind attaches to current listener");
        let state_machine = file
            .artboard_state_machine_graphs(0)
            .into_iter()
            .next()
            .expect("state machine");
        assert_eq!(state_machine.listeners.len(), 1);
        assert_eq!(state_machine.listeners[0].actions.len(), 1);
        assert_eq!(
            state_machine.listeners[0].actions[0]
                .object
                .uint_property("flags"),
            Some(3 << 1)
        );
    }

    #[test]
    fn upstream_listener_action_flag_decode_occurrence_and_import_routing_matrix() {
        // Assertion-for-assertion coverage for listener_action_flags_test.cpp.
        // The runtime's observable parent-kind result is which importer owns
        // the action; occurrence matching is exercised through the same bit
        // predicate used by perform_scheduled_listener_actions.
        let parent_kind = |flags: u64| match (flags >> 1) & 0x3 {
            1 => "transition",
            2 => "state",
            _ => "listener",
        };
        assert_eq!(parent_kind(0), "listener");
        assert_eq!(parent_kind(1 << 1), "transition");
        assert_eq!(parent_kind(2 << 1), "state");
        assert_eq!(parent_kind(3 << 1), "listener");

        let matches_occurrence =
            |flags: u64, occurrence: StateMachineFireOccurrence| flags & 1 == occurrence.value();
        assert_eq!(parent_kind(1), "listener");
        assert!(matches_occurrence(1, StateMachineFireOccurrence::AtEnd));
        assert!(!matches_occurrence(1, StateMachineFireOccurrence::AtStart));
        assert_eq!(parent_kind(2 << 1), "state");
        assert!(matches_occurrence(
            2 << 1,
            StateMachineFireOccurrence::AtStart
        ));
        assert!(!matches_occurrence(
            2 << 1,
            StateMachineFireOccurrence::AtEnd
        ));
        assert_eq!(parent_kind(1 | (1 << 1)), "transition");
        assert!(matches_occurrence(
            1 | (1 << 1),
            StateMachineFireOccurrence::AtEnd
        ));

        assert!(matches_occurrence(0, StateMachineFireOccurrence::AtStart));
        assert!(!matches_occurrence(0, StateMachineFireOccurrence::AtEnd));
        assert!(matches_occurrence(1, StateMachineFireOccurrence::AtEnd));
        assert!(!matches_occurrence(1, StateMachineFireOccurrence::AtStart));

        let routed = |transition_owner: bool, flags: u64| {
            let mut records = vec![
                record("Backboard", Vec::new()),
                record("Artboard", Vec::new()),
                record(
                    "Node",
                    vec![property("Node", "parentId", FixtureValue::Uint(0))],
                ),
                record("StateMachine", Vec::new()),
                record(
                    "StateMachineListenerSingle",
                    vec![
                        property(
                            "StateMachineListenerSingle",
                            "targetId",
                            FixtureValue::Uint(1),
                        ),
                        property(
                            "StateMachineListenerSingle",
                            "listenerTypeValue",
                            FixtureValue::Uint(15),
                        ),
                    ],
                ),
                record("StateMachineLayer", Vec::new()),
                record("AnyState", Vec::new()),
                record("EntryState", Vec::new()),
                record("ExitState", Vec::new()),
                record("AnimationState", Vec::new()),
            ];
            if transition_owner {
                records.push(record(
                    "StateTransition",
                    vec![property(
                        "StateTransition",
                        "stateToId",
                        FixtureValue::Uint(3),
                    )],
                ));
            }
            records.push(record(
                "FocusActionClear",
                vec![property(
                    "FocusActionClear",
                    "flags",
                    FixtureValue::Uint(flags),
                )],
            ));
            RuntimeFile::from_fixture_records(records).expect("routed listener action")
        };

        let transition_file = routed(true, 1 << 1);
        let transition_machine = &transition_file.artboard_state_machine_graphs(0)[0];
        assert_eq!(transition_machine.layers[0].states[3].transitions.len(), 1);
        assert_eq!(
            transition_machine.layers[0].states[3].transitions[0]
                .listener_actions
                .len(),
            1
        );
        assert_eq!(
            transition_machine.layers[0].states[3].transitions[0].listener_actions[0]
                .object
                .type_name,
            "FocusActionClear"
        );
        assert_eq!(transition_machine.listeners[0].actions.len(), 0);

        let state_file = routed(false, 2 << 1);
        let state_machine = &state_file.artboard_state_machine_graphs(0)[0];
        assert_eq!(state_machine.layers[0].states[3].listener_actions.len(), 1);
        assert_eq!(
            state_machine.layers[0].states[3].listener_actions[0]
                .object
                .type_name,
            "FocusActionClear"
        );
        assert!(state_machine.layers[0].states[3].transitions.is_empty());
        assert_eq!(state_machine.listeners[0].actions.len(), 0);

        let listener_file = routed(true, 0);
        let listener_machine = &listener_file.artboard_state_machine_graphs(0)[0];
        assert_eq!(listener_machine.listeners[0].actions.len(), 1);
        assert!(
            listener_machine.layers[0].states[3].transitions[0]
                .listener_actions
                .is_empty()
        );

        let missing_listener = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record("StateMachineLayer", Vec::new()),
            record("AnyState", Vec::new()),
            record("EntryState", Vec::new()),
            record("ExitState", Vec::new()),
            record("AnimationState", Vec::new()),
            record(
                "StateTransition",
                vec![property(
                    "StateTransition",
                    "stateToId",
                    FixtureValue::Uint(3),
                )],
            ),
            record(
                "FocusActionClear",
                vec![property("FocusActionClear", "flags", FixtureValue::Uint(0))],
            ),
        ]);
        assert!(missing_listener.is_err());
        // The failed import did not attach the action to the layer component:
        // constructing the same prefix without it leaves the transition empty.
        let control = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record("StateMachineLayer", Vec::new()),
            record("AnyState", Vec::new()),
            record("EntryState", Vec::new()),
            record("ExitState", Vec::new()),
            record("AnimationState", Vec::new()),
            record(
                "StateTransition",
                vec![property(
                    "StateTransition",
                    "stateToId",
                    FixtureValue::Uint(3),
                )],
            ),
        ])
        .expect("control file");
        assert!(
            control.artboard_state_machine_graphs(0)[0].layers[0].states[3].transitions[0]
                .listener_actions
                .is_empty()
        );
    }

    #[test]
    fn ordinary_listener_actions_read_live_core_fields_at_perform_time() {
        struct NoopExecutor;

        impl RuntimeScheduledListenerActionExecutor for NoopExecutor {
            fn perform_instance_action(
                &mut self,
                _artboard: &mut ArtboardInstance,
                _action: &RuntimeScheduledListenerAction,
                _targets: RuntimeScheduledListenerActionTargetsMut<'_>,
            ) -> Result<bool, ScriptError> {
                Ok(false)
            }
        }

        let records = vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Node",
                vec![property("Node", "parentId", FixtureValue::Uint(0))],
            ),
            record(
                "Event",
                vec![
                    property("Event", "parentId", FixtureValue::Uint(0)),
                    property("Event", "name", FixtureValue::String("first".to_owned())),
                ],
            ),
            record(
                "Event",
                vec![
                    property("Event", "parentId", FixtureValue::Uint(0)),
                    property("Event", "name", FixtureValue::String("second".to_owned())),
                ],
            ),
            record("StateMachine", Vec::new()),
            record("StateMachineBool", Vec::new()),
            record("StateMachineBool", Vec::new()),
            record(
                "StateMachineListenerSingle",
                vec![
                    property(
                        "StateMachineListenerSingle",
                        "targetId",
                        FixtureValue::Uint(1),
                    ),
                    property(
                        "StateMachineListenerSingle",
                        "listenerTypeValue",
                        FixtureValue::Uint(15),
                    ),
                ],
            ),
            record(
                "ListenerFireEvent",
                vec![
                    property("ListenerFireEvent", "eventId", FixtureValue::Uint(2)),
                    property(
                        "ListenerFireEvent",
                        "flags",
                        FixtureValue::Uint(StateMachineFireOccurrence::AtStart.value()),
                    ),
                ],
            ),
            record(
                "ListenerBoolChange",
                vec![
                    property("ListenerBoolChange", "inputId", FixtureValue::Uint(0)),
                    property("ListenerBoolChange", "value", FixtureValue::Uint(0)),
                ],
            ),
        ];
        let file = RuntimeFile::from_fixture_records(records).expect("live action fixture");
        let graph = GraphFile::from_runtime_file(&file).expect("live action graph");
        let action_catalog = super::super::RuntimeFileStateMachineActionCatalog::new(&file);
        let file_view_models = crate::RuntimeFileViewModelInstanceCatalog::new(&file);
        let instantiate = || {
            ArtboardInstance::from_graph_with_artboards_external_fonts_and_file_catalogs(
                &file,
                graph.artboards.first().expect("live action artboard"),
                &graph.artboards,
                &std::collections::BTreeMap::new(),
                file_view_models.clone(),
                action_catalog.clone(),
            )
            .expect("live action instance")
        };
        let mut artboard = instantiate();
        let mut second_existing_artboard = instantiate();
        let state_machine = artboard.state_machines[0].clone();
        let fire = state_machine.listeners[0].listener_actions[0].clone();
        let bool_change = state_machine.listeners[0].listener_actions[1].clone();
        let imported = file
            .artboard_state_machine_graphs(0)
            .into_iter()
            .next()
            .expect("imported state machine");
        let state_machine_global_id = imported.object.id;
        let fire_global_id = imported.listeners[0].actions[0].object.id;
        let bool_global_id = imported.listeners[0].actions[1].object.id;

        assert!(
            action_catalog.set_uint(
                state_machine_global_id,
                fire_global_id,
                crate::properties::property_key_for_name("ListenerFireEvent", "eventId")
                    .expect("ListenerFireEvent.eventId"),
                3,
            )
        );
        assert!(
            action_catalog.set_uint(
                state_machine_global_id,
                fire_global_id,
                crate::properties::property_key_for_name("ListenerFireEvent", "flags")
                    .expect("ListenerFireEvent.flags"),
                StateMachineFireOccurrence::AtEnd.value(),
            )
        );
        assert!(
            action_catalog.set_uint(
                state_machine_global_id,
                bool_global_id,
                crate::properties::property_key_for_name("ListenerBoolChange", "inputId")
                    .expect("ListenerBoolChange.inputId"),
                1,
            )
        );
        assert!(
            action_catalog.set_uint(
                state_machine_global_id,
                bool_global_id,
                crate::properties::property_key_for_name("ListenerBoolChange", "value")
                    .expect("ListenerBoolChange.value"),
                1,
            )
        );
        let mut future_artboard = instantiate();

        let mut inputs = (0..state_machine.inputs.len())
            .map(|index| StateMachineInputInstance::new(index, Arc::clone(&state_machine.inputs)))
            .collect::<Vec<_>>();
        let mut reported_events = Vec::new();
        let mut bindable_numbers = Vec::new();
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
        macro_rules! targets {
            () => {
                RuntimeScheduledListenerActionTargetsMut {
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
                }
            };
        }

        assert!(
            !perform_scheduled_listener_actions(
                std::slice::from_ref(&fire),
                StateMachineFireOccurrence::AtStart,
                &mut artboard,
                targets!(),
                &mut NoopExecutor,
            )
            .expect("at-start live flag"),
            "the live flags write moves this action away from at-start"
        );
        assert!(
            perform_scheduled_listener_actions(
                std::slice::from_ref(&fire),
                StateMachineFireOccurrence::AtEnd,
                &mut artboard,
                targets!(),
                &mut NoopExecutor,
            )
            .expect("at-end live flag")
        );
        assert_eq!(reported_events[0].name(), Some("second"));

        let second_bool_change =
            second_existing_artboard.state_machines[0].listeners[0].listener_actions[1].clone();
        assert!(
            perform_scheduled_listener_actions(
                std::slice::from_ref(&second_bool_change),
                StateMachineFireOccurrence::AtStart,
                &mut second_existing_artboard,
                targets!(),
                &mut NoopExecutor,
            )
            .expect("live bool fields")
        );
        assert_eq!(inputs[0].bool_value(), Some(false));
        assert_eq!(inputs[1].bool_value(), Some(true));

        let future_fire =
            future_artboard.state_machines[0].listeners[0].listener_actions[0].clone();
        assert!(
            perform_scheduled_listener_actions(
                std::slice::from_ref(&future_fire),
                StateMachineFireOccurrence::AtEnd,
                &mut future_artboard,
                targets!(),
                &mut NoopExecutor,
            )
            .expect("future instance observes the file-owned action fields")
        );
        assert_eq!(
            reported_events
                .last()
                .and_then(StateMachineReportedEvent::name),
            Some("second")
        );

        let _ = bool_change;
    }

    #[test]
    fn scheduled_action_keeps_terminal_resource_failure_typed() {
        struct TerminalExecutor;

        impl RuntimeScheduledListenerActionExecutor for TerminalExecutor {
            fn perform_instance_action(
                &mut self,
                _artboard: &mut ArtboardInstance,
                _action: &RuntimeScheduledListenerAction,
                _targets: RuntimeScheduledListenerActionTargetsMut<'_>,
            ) -> Result<bool, ScriptError> {
                Err(ScriptError::with_resource_code(
                    "script cycle exceeds 256 host commands",
                    "script.resource.host_commands",
                ))
            }
        }

        let mut artboard = instantiate(prefix()).expect("listener action artboard");
        let actions = [RuntimeScheduledListenerAction::scripted_for_test(
            StateMachineFireOccurrence::AtStart.value(),
            None,
        )];
        let mut inputs = Vec::new();
        let mut reported_events = Vec::new();
        let mut bindable_numbers = Vec::new();
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
        let error = perform_scheduled_listener_actions(
            &actions,
            StateMachineFireOccurrence::AtStart,
            &mut artboard,
            RuntimeScheduledListenerActionTargetsMut {
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
            },
            &mut TerminalExecutor,
        )
        .expect_err("terminal resource failures must escape scheduled actions");

        assert_eq!(error.resource_code(), Some("script.resource.host_commands"));
    }

    #[test]
    fn scheduled_direct_input_actions_mark_only_genuine_owner_changes() {
        struct RecordingExecutor {
            direct_input_changes: usize,
        }

        impl RuntimeScheduledListenerActionExecutor for RecordingExecutor {
            fn mark_direct_input_changed(&mut self) {
                self.direct_input_changes += 1;
            }

            fn perform_instance_action(
                &mut self,
                _artboard: &mut ArtboardInstance,
                _action: &RuntimeScheduledListenerAction,
                _targets: RuntimeScheduledListenerActionTargetsMut<'_>,
            ) -> Result<bool, ScriptError> {
                unreachable!("the fixture contains only direct input actions")
            }
        }

        let definitions = std::sync::Arc::new(vec![
            Some(crate::state_machine::RuntimeStateMachineInput::new_bool(
                1, None, false,
            )),
            Some(crate::state_machine::RuntimeStateMachineInput::new_number(
                2, None, 0.0,
            )),
            Some(crate::state_machine::RuntimeStateMachineInput::new_trigger(
                3, None,
            )),
        ]);
        let mut inputs = (0..definitions.len())
            .map(|index| StateMachineInputInstance::new(index, std::sync::Arc::clone(&definitions)))
            .collect::<Vec<_>>();
        let direct = |index| RuntimeListenerInputTarget {
            direct_input_index: Some(index),
            nested_input_local_id: None,
        };
        let actions = [
            RuntimeScheduledListenerAction::BoolChange(RuntimeListenerBoolChange::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                direct(0),
                1,
            )),
            RuntimeScheduledListenerAction::NumberChange(RuntimeListenerNumberChange::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                direct(1),
                4.0,
            )),
            RuntimeScheduledListenerAction::TriggerChange(RuntimeListenerTriggerChange::for_test(
                StateMachineFireOccurrence::AtStart.value(),
                direct(2),
            )),
        ];
        let mut artboard = instantiate(prefix()).expect("listener action artboard");
        let mut executor = RecordingExecutor {
            direct_input_changes: 0,
        };
        let mut reported_events = Vec::new();

        let run = |inputs: &mut [StateMachineInputInstance],
                   executor: &mut RecordingExecutor,
                   artboard: &mut ArtboardInstance,
                   reported_events: &mut Vec<StateMachineReportedEvent>| {
            perform_scheduled_listener_actions(
                &actions,
                StateMachineFireOccurrence::AtStart,
                artboard,
                RuntimeScheduledListenerActionTargetsMut {
                    inputs,
                    reported_events,
                    bindable_numbers: &mut [],
                    bindable_integers: &mut [],
                    bindable_colors: &mut [],
                    bindable_strings: &mut [],
                    bindable_enums: &mut [],
                    bindable_assets: &mut [],
                    bindable_artboards: &mut [],
                    bindable_lists: &mut [],
                    bindable_triggers: &mut [],
                    bindable_view_models: &mut [],
                    bindable_booleans: &mut [],
                    transition_durations: &mut [],
                },
                executor,
            )
        };

        assert!(
            run(
                &mut inputs,
                &mut executor,
                &mut artboard,
                &mut reported_events
            )
            .expect("first action batch")
        );
        assert_eq!(
            executor.direct_input_changes, 3,
            "SMIBool/SMINumber/SMITrigger each call valueChanged after their first genuine mutation"
        );
        assert!(
            !run(
                &mut inputs,
                &mut executor,
                &mut artboard,
                &mut reported_events
            )
            .expect("same-value action batch"),
            "equal bool/number values and an already-fired trigger are all no-ops in pinned C++"
        );
        assert_eq!(executor.direct_input_changes, 3);
    }
}
