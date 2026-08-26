use super::*;
use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue};
use nuxie_graph::GraphFile;

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

fn prefix() -> Vec<FixtureRecord> {
    vec![
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
    ]
}

fn routed(transition_owner: bool, flags: u64) -> RuntimeFile {
    let mut records = prefix();
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
    RuntimeFile::from_fixture_records(records).expect("listener action routes")
}

fn owner_counts(file: &RuntimeFile) -> (usize, usize, usize) {
    let machine = &file.artboard_state_machine_graphs(0)[0];
    let listener = machine.listeners[0].actions.len();
    let state = machine.layers[0].states[3].listener_actions.len();
    let transition = machine.layers[0].states[3]
        .transitions
        .first()
        .map_or(0, |transition| transition.listener_actions.len());
    (listener, transition, state)
}

#[derive(Clone, Copy)]
enum ExactOwner {
    Listener,
    Transition,
    State,
}

fn assert_exact_route(file: &RuntimeFile, expected: ExactOwner, flags: u64) {
    let machine = &file.artboard_state_machine_graphs(0)[0];
    let imported = file
        .objects
        .iter()
        .flatten()
        .find(|object| object.type_name == "FocusActionClear")
        .expect("exact imported ListenerAction");
    assert_eq!(imported.uint_property("flags"), Some(flags));
    let routed = match expected {
        ExactOwner::Listener => {
            assert!(machine.layers[0].states[3].listener_actions.is_empty());
            assert!(
                machine.layers[0].states[3]
                    .transitions
                    .first()
                    .is_none_or(|transition| transition.listener_actions.is_empty())
            );
            assert_eq!(machine.listeners[0].actions.len(), 1);
            &machine.listeners[0].actions[0]
        }
        ExactOwner::Transition => {
            assert!(machine.listeners[0].actions.is_empty());
            assert!(machine.layers[0].states[3].listener_actions.is_empty());
            let transition = machine.layers[0].states[3]
                .transitions
                .first()
                .expect("exact transition owner");
            assert_eq!(transition.listener_actions.len(), 1);
            &transition.listener_actions[0]
        }
        ExactOwner::State => {
            assert!(machine.listeners[0].actions.is_empty());
            assert!(machine.layers[0].states[3].transitions.is_empty());
            assert_eq!(machine.layers[0].states[3].listener_actions.len(), 1);
            &machine.layers[0].states[3].listener_actions[0]
        }
    };
    assert!(std::ptr::eq(routed.object, imported));
}

fn occurrence_runs(flags: u64, occurrence: StateMachineFireOccurrence) -> bool {
    struct CountingExecutor(usize);
    impl RuntimeScheduledListenerActionExecutor for CountingExecutor {
        fn perform_instance_action(
            &mut self,
            _artboard: &mut ArtboardInstance,
            _action: &RuntimeScheduledListenerAction,
            _targets: RuntimeScheduledListenerActionTargetsMut<'_>,
        ) -> Result<bool, ScriptError> {
            self.0 += 1;
            Ok(true)
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
    ])
    .expect("empty artboard fixture");
    let graphs = GraphFile::from_runtime_file(&file).expect("empty artboard graph");
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&file, &graphs.artboards[0], &graphs.artboards)
            .expect("empty artboard instance");
    let action = RuntimeScheduledListenerAction::scripted_for_test(flags, None);
    let mut executor = CountingExecutor(0);
    let mut reported_events = Vec::new();
    let mut inputs = Vec::new();
    let mut numbers = Vec::new();
    let mut integers = Vec::new();
    let mut colors = Vec::new();
    let mut strings = Vec::new();
    let mut enums = Vec::new();
    let mut assets = Vec::new();
    let mut artboards = Vec::new();
    let mut lists = Vec::new();
    let mut triggers = Vec::new();
    let mut view_models = Vec::new();
    let mut booleans = Vec::new();
    let mut durations = Vec::new();
    let changed = perform_scheduled_listener_actions(
        &[action],
        occurrence,
        &mut artboard,
        RuntimeScheduledListenerActionTargetsMut {
            inputs: &mut inputs,
            reported_events: &mut reported_events,
            bindable_numbers: &mut numbers,
            bindable_integers: &mut integers,
            bindable_colors: &mut colors,
            bindable_strings: &mut strings,
            bindable_enums: &mut enums,
            bindable_assets: &mut assets,
            bindable_artboards: &mut artboards,
            bindable_lists: &mut lists,
            bindable_triggers: &mut triggers,
            bindable_view_models: &mut view_models,
            bindable_booleans: &mut booleans,
            transition_durations: &mut durations,
        },
        &mut executor,
    )
    .expect("scheduled action evaluation");
    assert_eq!(changed, executor.0 == 1);
    executor.0 == 1
}

#[test]
fn wave_c2_listener_flags_001_parent_kind_decodes_bits_one_and_two() {
    assert_exact_route(&routed(true, 0), ExactOwner::Listener, 0);
    assert_exact_route(&routed(true, 1 << 1), ExactOwner::Transition, 1 << 1);
    assert_exact_route(&routed(false, 2 << 1), ExactOwner::State, 2 << 1);
    assert_exact_route(&routed(true, 3 << 1), ExactOwner::Listener, 3 << 1);
}

#[test]
fn wave_c2_listener_flags_002_fields_are_independent() {
    assert_exact_route(&routed(true, 1), ExactOwner::Listener, 1);
    assert!(occurrence_runs(1, StateMachineFireOccurrence::AtEnd));
    assert!(!occurrence_runs(1, StateMachineFireOccurrence::AtStart));
    assert_exact_route(&routed(false, 2 << 1), ExactOwner::State, 2 << 1);
    assert!(occurrence_runs(2 << 1, StateMachineFireOccurrence::AtStart));
    assert!(!occurrence_runs(2 << 1, StateMachineFireOccurrence::AtEnd));
    assert_exact_route(
        &routed(true, 1 | (1 << 1)),
        ExactOwner::Transition,
        1 | (1 << 1),
    );
    assert!(occurrence_runs(
        1 | (1 << 1),
        StateMachineFireOccurrence::AtEnd
    ));
}

#[test]
fn wave_c2_listener_flags_003_matches_both_occurrences() {
    assert!(occurrence_runs(0, StateMachineFireOccurrence::AtStart));
    assert!(!occurrence_runs(0, StateMachineFireOccurrence::AtEnd));
    assert!(occurrence_runs(1, StateMachineFireOccurrence::AtEnd));
    assert!(!occurrence_runs(1, StateMachineFireOccurrence::AtStart));
}

#[test]
fn wave_c2_listener_flags_004_transition_routes_to_layer_importer() {
    assert_exact_route(&routed(true, 1 << 1), ExactOwner::Transition, 1 << 1);
}

#[test]
fn wave_c2_listener_flags_005_state_routes_to_layer_importer() {
    assert_exact_route(&routed(false, 2 << 1), ExactOwner::State, 2 << 1);
}

#[test]
fn wave_c2_listener_flags_006_listener_routes_to_listener_importer() {
    assert_exact_route(&routed(true, 0), ExactOwner::Listener, 0);
}

#[test]
fn wave_c2_listener_flags_007_missing_listener_importer_fails() {
    let error = RuntimeFile::from_fixture_records(vec![
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
    ])
    .expect_err("Listener parent without listener importer must reject");
    assert_eq!(
        error.to_string(),
        "fixture object 9 (FocusActionClear) would be dropped during import: MissingObject"
    );

    // Rust fixture import is transactional: the failed RuntimeFile is never
    // published. Re-run the exact owner prefix without the failed action and
    // prove the transition's concrete retained collection is empty.
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
    .expect("exact transition owner prefix");
    let transition =
        &control.artboard_state_machine_graphs(0)[0].layers[0].states[3].transitions[0];
    assert!(transition.listener_actions.is_empty());
}
