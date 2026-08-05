//! Focused parity tests for Rive's pinned `command_queue_test.cpp`.
//!
//! These tests port the non-rendering command-loop invariants from
//! `tests/unit_tests/runtime/command_queue_test.cpp` at `4ac7b327`. The
//! case-by-case correspondence, including the remaining S4-45 WATCH residue,
//! is recorded in `docs/command-queue-test-ledger.md`.

use std::{
    any::Any,
    cell::RefCell,
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use nuxie::{
    AudioSource, RawTextFont, RecordingFactory, RenderImage, SemanticActionType, SemanticRole,
    SemanticState, SemanticTrait, SemanticsDiff, SemanticsDiffNode,
    command_queue::{
        ArtboardHandle, CommandDataType, CommandEvent, CommandQueue, CommandValue, Fit, Listener,
        PointerEvent, StateMachineHandle,
    },
    command_server::CommandServer,
    has_semantic_state, has_semantic_trait,
};

#[derive(Debug)]
struct ExternalImage(u8);

impl RenderImage for ExternalImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        u32::from(self.0) + 1
    }

    fn height(&self) -> u32 {
        1
    }
}

const ARTBOARD_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/two_artboards.riv");
const ENTRY_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/entry.riv");
const MULTI_MACHINE_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/multiple_state_machines.riv");
const DATA_BIND_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/data_bind_test_cmdq.riv");
const IMAGE_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/batdude.png");
const AUDIO_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/what.wav");
const FONT_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/OpenSans-Italic.ttf");
const POINTER_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/pointer_events.riv");
const RAPID_POINTER_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/rapid_pointer_events.riv");
const HOSTED_IMAGE_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/hosted_image_file.riv");
const HOSTED_FONT_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/hosted_font_file.riv");
const GLOBAL_VARIABLES_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/global_variables_test.riv");
const SEMANTIC_SIMPSONS_FIXTURE: &[u8] = include_bytes!("../../../fixtures/semantic/simpsons.riv");
const SEMANTIC_FOCUS_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv");

fn server(queue: &CommandQueue) -> CommandServer {
    CommandServer::new(queue.clone(), Box::new(RecordingFactory::new()))
}

fn semantic_fixture(
    listener: Option<&Listener>,
) -> (
    CommandQueue,
    CommandServer,
    ArtboardHandle,
    StateMachineHandle,
) {
    semantic_fixture_with(SEMANTIC_SIMPSONS_FIXTURE, listener)
}

fn semantic_fixture_with(
    fixture: &[u8],
    listener: Option<&Listener>,
) -> (
    CommandQueue,
    CommandServer,
    ArtboardHandle,
    StateMachineHandle,
) {
    let queue = CommandQueue::new();
    let file = queue.load_file(fixture.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let view_model =
        queue.instantiate_view_model_for_artboard(file, artboard, Some(String::new()), None, 0);
    let state_machine = queue.instantiate_default_state_machine(artboard, listener, 0);
    queue.bind_view_model(state_machine, view_model, 0);
    let server = server(&queue);
    (queue, server, artboard, state_machine)
}

fn warm_semantics(queue: &CommandQueue, state_machine: StateMachineHandle) {
    for _ in 0..10 {
        queue.advance_state_machine(state_machine, 0.1, 0);
    }
}

fn drain_semantics(queue: &CommandQueue, state_machine: StateMachineHandle, request_id: u64) {
    queue.drain_semantics_diff(
        state_machine,
        Fit::Contain,
        nuxie::command_queue::Alignment::CENTER,
        1.0,
        nuxie::Vec2D::new(500.0, 500.0),
        request_id,
    );
}

#[derive(Debug, Default)]
struct SemanticTestModel {
    nodes: BTreeMap<u32, SemanticsDiffNode>,
}

impl SemanticTestModel {
    fn apply(&mut self, diff: &SemanticsDiff) {
        for id in &diff.removed {
            self.nodes.remove(id);
        }
        for node in diff.added.iter().chain(&diff.moved) {
            self.nodes.insert(node.id, node.clone());
        }
        for node in &diff.updated_semantic {
            if let Some(existing) = self.nodes.get_mut(&node.id) {
                let bounds = existing.bounds();
                *existing = node.clone();
                existing.set_bounds(bounds);
            } else {
                self.nodes.insert(node.id, node.clone());
            }
        }
        for update in &diff.updated_geometry {
            if let Some(existing) = self.nodes.get_mut(&update.id) {
                existing.set_bounds(update.bounds());
            }
        }
    }
}

fn apply_semantic_events(model: &mut SemanticTestModel, captured: &[CommandEvent]) {
    for event in captured {
        if let CommandEvent::SemanticsDiffReceived { diff, .. } = event {
            model.apply(diff);
        }
    }
}

fn semantic_nodes_for_view(
    fit: Fit,
    scale_factor: f32,
    view_bounds: nuxie::Vec2D,
) -> SemanticTestModel {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);
    queue.drain_semantics_diff(
        state_machine,
        fit,
        nuxie::command_queue::Alignment::CENTER,
        scale_factor,
        view_bounds,
        0,
    );
    assert!(server.process_commands());
    queue.process_messages();
    let mut model = SemanticTestModel::default();
    apply_semantic_events(&mut model, &events(&log));
    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, CommandEvent::StateMachineError { .. }))
    );
    model
}

fn event_log() -> (Listener, Arc<Mutex<Vec<CommandEvent>>>) {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&events);
    let listener: Listener = Arc::new(move |event: &CommandEvent| {
        sink.lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event.clone());
    });
    (listener, events)
}

fn events(log: &Arc<Mutex<Vec<CommandEvent>>>) -> Vec<CommandEvent> {
    log.lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
}

#[test]
fn pod_stream_rcp() {
    const MAGIC_NUMBER: usize = 0x99;
    let queue = CommandQueue::new();
    let original = Arc::new(MAGIC_NUMBER);
    let captured = Arc::clone(&original);
    let null: Option<Arc<usize>> = None;
    let observed = Arc::new(Mutex::new(None));
    let observed_on_server = Arc::clone(&observed);
    queue.run_once(move |_| {
        assert!(Arc::ptr_eq(&captured, &original));
        assert_eq!(*captured, MAGIC_NUMBER);
        *observed_on_server
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(captured);
        assert!(null.is_none());
    });
    assert!(server(&queue).process_commands());
    assert_eq!(
        observed
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .as_deref(),
        Some(&MAGIC_NUMBER)
    );
}

#[test]
fn semantics_advance_does_not_auto_deliver_diff() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);

    assert!(server.process_commands());
    queue.process_messages();

    assert!(!events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::SemanticsDiffReceived { .. } | CommandEvent::StateMachineError { .. }
    )));
}

#[test]
fn semantics_enable_and_initial_diff_on_drain() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);

    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    let mut diff_count = 0;
    for event in events(&log) {
        if let CommandEvent::SemanticsDiffReceived { diff, .. } = event {
            diff_count += 1;
            model.apply(&diff);
        }
    }
    assert!(diff_count >= 1);
    assert!(!model.nodes.is_empty());
    assert!(
        model
            .nodes
            .values()
            .any(|node| node.role == SemanticRole::TabList as u32)
    );
    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, CommandEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_no_diff_when_not_enabled() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    warm_semantics(&queue, state_machine);

    assert!(server.process_commands());
    queue.process_messages();

    assert!(!events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::SemanticsDiffReceived { .. } | CommandEvent::StateMachineError { .. }
    )));
}

#[test]
fn semantics_drain_diff_errors_when_not_enabled() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    let request_id = 0x1234;
    drain_semantics(&queue, state_machine, request_id);

    assert!(server.process_commands());
    queue.process_messages();

    let captured = events(&log);
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::SemanticsDiffReceived { .. }))
    );
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, CommandEvent::StateMachineError { .. }))
            .count(),
        1
    );
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::StateMachineError { request_id: actual, .. } if *actual == request_id
    )));
}

#[test]
fn semantics_drain_diff_only_emits_for_non_empty_diff() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);
    let request_id = 0xABCD;
    drain_semantics(&queue, state_machine, request_id);

    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&log)
            .iter()
            .filter(|event| matches!(event, CommandEvent::SemanticsDiffReceived { .. }))
            .count(),
        1
    );
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::SemanticsDiffReceived { request_id: actual, .. } if *actual == request_id
    )));

    drain_semantics(&queue, state_machine, 0xBCDE);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, CommandEvent::SemanticsDiffReceived { .. }))
            .count(),
        1
    );
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_fire_tap_changes_selected_tab() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    let initial_events = events(&log);
    apply_semantic_events(&mut model, &initial_events);
    let selected_tab_id = model
        .nodes
        .values()
        .find(|node| {
            node.role == SemanticRole::Tab as u32
                && has_semantic_state(node.state_flags, SemanticState::SELECTED)
        })
        .map(|node| node.id)
        .expect("selected tab");
    let other_tab_id = model
        .nodes
        .values()
        .find(|node| {
            node.role == SemanticRole::Tab as u32
                && !has_semantic_state(node.state_flags, SemanticState::SELECTED)
        })
        .map(|node| node.id)
        .expect("other tab");

    queue.fire_semantic_action(state_machine, other_tab_id, SemanticActionType::Tap, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let captured = events(&log);
    apply_semantic_events(&mut model, &captured[initial_events.len()..]);
    assert!(has_semantic_state(
        model.nodes[&other_tab_id].state_flags,
        SemanticState::SELECTED
    ));
    assert!(!has_semantic_state(
        model.nodes[&selected_tab_id].state_flags,
        SemanticState::SELECTED
    ));
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_commands_on_invalid_state_machine_handle() {
    let (listener, log) = event_log();
    let (queue, mut server, artboard, _) = semantic_fixture(None);
    let bogus = queue.instantiate_state_machine_named(
        artboard,
        "this state machine does not exist",
        Some(&listener),
        0,
    );

    queue.enable_semantics(bogus, 0xE1);
    queue.drain_semantics_diff(
        bogus,
        Fit::Contain,
        nuxie::command_queue::Alignment::CENTER,
        1.0,
        nuxie::Vec2D::new(500.0, 500.0),
        0xE2,
    );
    queue.fire_semantic_action(bogus, 42, SemanticActionType::Tap, 0xE3);
    queue.request_semantic_focus(bogus, 42, 0xE4);
    queue.clear_semantic_focus(bogus, 0xE5);

    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    let error_request_ids = captured
        .iter()
        .filter_map(|event| match event {
            CommandEvent::StateMachineError { request_id, .. } => Some(*request_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(error_request_ids, [0xE1, 0xE2, 0xE3, 0xE4, 0xE5]);
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::SemanticsDiffReceived { .. }))
    );
}

#[test]
fn semantics_drain_diff_maps_bounds_into_view_space() {
    let small_view = nuxie::Vec2D::new(200.0, 200.0);
    let large_view = nuxie::Vec2D::new(800.0, 800.0);
    let small = semantic_nodes_for_view(Fit::Contain, 1.0, small_view);
    let large = semantic_nodes_for_view(Fit::Contain, 1.0, large_view);

    let (small_tab, large_tab) = small
        .nodes
        .values()
        .filter(|node| {
            node.role == SemanticRole::Tab as u32
                && node.max_x > node.min_x
                && node.max_y > node.min_y
        })
        .find_map(|small_tab| {
            large
                .nodes
                .get(&small_tab.id)
                .map(|large_tab| (small_tab, large_tab))
        })
        .expect("shared tab with non-empty bounds");
    let small_width = small_tab.max_x - small_tab.min_x;
    let small_height = small_tab.max_y - small_tab.min_y;
    let large_width = large_tab.max_x - large_tab.min_x;
    let large_height = large_tab.max_y - large_tab.min_y;
    let expected_scale = large_view.x / small_view.x;

    assert!(large_width > small_width);
    assert!(large_height > small_height);
    assert!(((large_width / small_width) / expected_scale - 1.0).abs() <= 0.01);
    assert!(((large_height / small_height) / expected_scale - 1.0).abs() <= 0.01);
}

#[test]
fn semantics_request_focus_errors_when_not_enabled() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.request_semantic_focus(state_machine, 1, 0x5151);

    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                CommandEvent::StateMachineError {
                    request_id: 0x5151,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::SemanticsDiffReceived { .. }))
    );
}

#[test]
fn semantics_fire_action_errors_when_not_enabled() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.fire_semantic_action(state_machine, 1, SemanticActionType::Tap, 0x5252);

    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                CommandEvent::StateMachineError {
                    request_id: 0x5252,
                    ..
                }
            ))
            .count(),
        1
    );
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::SemanticsDiffReceived { .. }))
    );
}

#[test]
fn semantics_request_focus_on_valid_node_routes_without_error() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) = semantic_fixture(Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    apply_semantic_events(&mut model, &events(&log));
    let target_id = model.nodes.keys().next().copied().expect("semantic node");
    queue.request_semantic_focus(state_machine, target_id, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, CommandEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_clear_focus_removes_focused_bit() {
    let (listener, log) = event_log();
    let (queue, mut server, _, state_machine) =
        semantic_fixture_with(SEMANTIC_FOCUS_FIXTURE, Some(&listener));
    queue.enable_semantics(state_machine, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();

    let mut model = SemanticTestModel::default();
    let initial = events(&log);
    apply_semantic_events(&mut model, &initial);
    let focusable_id = model
        .nodes
        .values()
        .find(|node| has_semantic_trait(node.trait_flags, SemanticTrait::FOCUSABLE))
        .map(|node| node.id)
        .expect("focusable semantic node");

    queue.request_semantic_focus(state_machine, focusable_id, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();
    let focused = events(&log);
    apply_semantic_events(&mut model, &focused[initial.len()..]);
    assert!(has_semantic_state(
        model.nodes[&focusable_id].state_flags,
        SemanticState::FOCUSED
    ));

    queue.clear_semantic_focus(state_machine, 0);
    warm_semantics(&queue, state_machine);
    drain_semantics(&queue, state_machine, 0);
    assert!(server.process_commands());
    queue.process_messages();
    let cleared = events(&log);
    apply_semantic_events(&mut model, &cleared[focused.len()..]);
    assert!(!has_semantic_state(
        model.nodes[&focusable_id].state_flags,
        SemanticState::FOCUSED
    ));
    assert!(
        !cleared
            .iter()
            .any(|event| matches!(event, CommandEvent::StateMachineError { .. }))
    );
}

#[test]
fn semantics_drain_diff_honors_scale_factor_for_matching_view() {
    let (queue, mut server, artboard, _) = semantic_fixture(None);
    let captured_bounds = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&captured_bounds);
    queue.run_once(move |server| {
        *sink.lock().unwrap_or_else(|poisoned| poisoned.into_inner()) = server
            .artboard(artboard)
            .map(|artboard| artboard.artboard_bounds());
    });
    assert!(server.process_commands());
    let (x, y, width, height) = captured_bounds
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .expect("artboard bounds");
    assert_eq!((x, y), (0.0, 0.0));
    assert!(width > 0.0 && height > 0.0);
    let view_bounds = nuxie::Vec2D::new(width, height);

    let at_scale_1 = semantic_nodes_for_view(Fit::Layout, 1.0, view_bounds);
    let at_scale_2 = semantic_nodes_for_view(Fit::Layout, 2.0, view_bounds);
    let mut compared_any = false;
    for node in at_scale_1
        .nodes
        .values()
        .filter(|node| node.max_x > node.min_x && node.max_y > node.min_y)
    {
        let Some(scaled) = at_scale_2.nodes.get(&node.id) else {
            continue;
        };
        let width_ratio = (scaled.max_x - scaled.min_x) / (node.max_x - node.min_x);
        let height_ratio = (scaled.max_y - scaled.min_y) / (node.max_y - node.min_y);
        assert!((width_ratio / 2.0 - 1.0).abs() <= 0.02);
        assert!((height_ratio / 2.0 - 1.0).abs() <= 0.02);
        compared_any = true;
    }
    assert!(compared_any);
}

#[test]
fn handles_are_typed_nonzero_and_monotonic() {
    let queue = CommandQueue::new();
    let first = queue.load_file(Vec::new(), None, 0);
    let second = queue.load_file(Vec::new(), None, 0);
    assert_eq!(first.get(), 1);
    assert_eq!(second.get(), 2);
}

#[test]
fn artboard_management() {
    let queue = CommandQueue::new();
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), None, 0);
    let one = queue.instantiate_artboard_named(file, "One", None, 0);
    let two = queue.instantiate_artboard_named(file, "Two", None, 0);
    let missing = queue.instantiate_artboard_named(file, "Three", None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.file(file).is_some());
    assert!(server.artboard(one).is_some());
    assert!(server.artboard(two).is_some());
    assert!(server.artboard(missing).is_none());

    queue.delete_artboard(missing, 0);
    queue.delete_artboard(two, 0);
    assert!(server.process_commands());
    assert!(server.artboard(one).is_some());
    assert!(server.artboard(two).is_none());
    assert!(server.artboard(missing).is_none());

    queue.delete_file(file, 0);
    assert!(server.process_commands());
    assert!(server.file(file).is_none());
    assert!(server.artboard(one).is_none());

    queue.delete_artboard(one, 0);
    assert!(server.process_commands());
    assert!(server.artboard(one).is_none());
}

#[test]
fn state_machine_management() {
    let queue = CommandQueue::new();
    let (listener, events) = event_log();
    queue.set_global_artboard_listener(Some(&listener));
    let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let one = queue.instantiate_state_machine_named(artboard, "one", None, 0);
    let two = queue.instantiate_state_machine_named(artboard, "two", None, 0);
    let missing = queue.instantiate_state_machine_named(artboard, "blahblah", None, 9);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.artboard(artboard).is_some());
    assert!(server.state_machine(one).is_some());
    assert!(server.state_machine(two).is_some());
    assert!(server.state_machine(missing).is_none());
    queue.process_messages();
    assert!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .any(|event| matches!(event, CommandEvent::ArtboardError { request_id: 9, .. }))
    );

    queue.delete_file(file, 0);
    queue.delete_artboard(artboard, 0);
    queue.delete_state_machine(one, 0);
    assert!(server.process_commands());
    assert!(server.file(file).is_none());
    assert!(server.artboard(artboard).is_none());
    assert!(server.state_machine(one).is_none());
    assert!(server.state_machine(two).is_none());

    queue.delete_state_machine(two, 0);
    assert!(server.process_commands());
    assert!(server.state_machine(two).is_none());
}

#[test]
fn default_artboard_and_state_machine() {
    let queue = CommandQueue::new();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let default_artboard = queue.instantiate_default_artboard(file, None, 0);
    let default_machine = queue.instantiate_default_state_machine(default_artboard, None, 0);
    let empty_artboard = queue.instantiate_artboard_named(file, "", None, 0);
    let empty_machine = queue.instantiate_state_machine_named(empty_artboard, "", None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());

    for (artboard, machine) in [
        (default_artboard, default_machine),
        (empty_artboard, empty_machine),
    ] {
        let artboard = server.artboard(artboard).expect("default artboard");
        assert_eq!(artboard.artboard().name(), Some("New Artboard"));
        let machine = server.state_machine(machine).expect("default machine");
        assert_eq!(
            artboard
                .artboard()
                .state_machine_name(machine.state_machine_index()),
            Some("State Machine 1")
        );
    }
}

#[test]
fn invalid_handles() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener));
    let good_file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let bad_file = queue.load_file(vec![0; 100 * 1024], None, 10);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.file(good_file).is_some());
    assert!(server.file(bad_file).is_none());

    let good_artboard = queue.instantiate_artboard_named(good_file, "New Artboard", None, 0);
    let bad_artboard_one = queue.instantiate_default_artboard(bad_file, None, 11);
    let bad_artboard_two = queue.instantiate_artboard_named(bad_file, "New Artboard", None, 12);
    let bad_artboard_three = queue.instantiate_artboard_named(good_file, "blahblahblah", None, 13);
    assert!(server.process_commands());
    assert!(server.artboard(good_artboard).is_some());
    for handle in [bad_artboard_one, bad_artboard_two, bad_artboard_three] {
        assert!(server.artboard(handle).is_none());
    }

    let good_machine =
        queue.instantiate_state_machine_named(good_artboard, "State Machine 1", None, 0);
    let bad_machine_one =
        queue.instantiate_state_machine_named(bad_artboard_two, "State Machine 1", None, 14);
    let bad_machine_two =
        queue.instantiate_state_machine_named(good_artboard, "blahblahblah", None, 15);
    let bad_machine_three = queue.instantiate_default_state_machine(bad_artboard_three, None, 16);
    assert!(server.process_commands());
    assert!(server.state_machine(good_machine).is_some());
    for handle in [bad_machine_one, bad_machine_two, bad_machine_three] {
        assert!(server.state_machine(handle).is_none());
    }

    for handle in [bad_machine_three, bad_machine_two, bad_machine_one] {
        queue.delete_state_machine(handle, 0);
    }
    for handle in [bad_artboard_three, bad_artboard_two, bad_artboard_one] {
        queue.delete_artboard(handle, 0);
    }
    queue.delete_file(bad_file, 0);
    assert!(server.process_commands());
    assert!(server.file(good_file).is_some());
    assert!(server.artboard(good_artboard).is_some());
    assert!(server.state_machine(good_machine).is_some());

    queue.delete_state_machine(good_machine, 0);
    queue.delete_artboard(good_artboard, 0);
    queue.delete_file(good_file, 0);
    assert!(server.process_commands());
    assert!(server.file(good_file).is_none());
    assert!(server.artboard(good_artboard).is_none());
    assert!(server.state_machine(good_machine).is_none());

    assert!(queue.process_messages() > 0);
    assert!(
        events(&log)
            .iter()
            .any(|event| { matches!(event, CommandEvent::FileError { request_id: 10, .. }) })
    );
}

#[test]
fn draw_loops() {
    let queue = CommandQueue::new();
    let first = queue.create_draw_key();
    let second = queue.create_draw_key();
    let counts = Arc::new(Mutex::new((0usize, 0usize)));
    let mut server = server(&queue);

    let first_counts = Arc::clone(&counts);
    queue.draw(first, move |_, _| {
        first_counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .0 += 1;
    });
    let second_counts = Arc::clone(&counts);
    queue.draw(second, move |_, _| {
        second_counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .1 += 1;
    });
    assert!(server.process_commands());
    assert_eq!(*counts.lock().unwrap(), (1, 1));

    let second_counts = Arc::clone(&counts);
    queue.draw(second, move |_, _| {
        second_counts
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .1 += 1;
    });
    assert!(server.process_commands());
    assert_eq!(*counts.lock().unwrap(), (1, 2));

    for _ in 0..10 {
        assert!(server.process_commands());
    }
    assert_eq!(*counts.lock().unwrap(), (1, 2));
}

#[test]
fn test_support_for_all_asset_types() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let attempted = Arc::new(Mutex::new(Vec::new()));
    let attempted_by_loader = Arc::clone(&attempted);
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_file_assets(file, 1);
    let loader = move |asset: &nuxie::RuntimeFileAsset,
                       _in_band: &[u8],
                       _factory: &mut dyn nuxie::Factory| {
        attempted_by_loader
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(asset.kind());
        false
    };
    let mut server = CommandServer::with_asset_loader(
        queue.clone(),
        Box::new(RecordingFactory::new()),
        Box::new(loader),
    );
    assert!(server.process_commands());
    assert!(queue.process_messages() >= 2);
    let assets = events(&log)
        .into_iter()
        .find_map(|event| match event {
            CommandEvent::FileAssetsListed { assets, .. } => Some(assets),
            _ => None,
        })
        .expect("file assets list callback");
    assert!(!assets.is_empty());
    assert!(assets.iter().all(|asset| asset.type_id != 0));
    let attempted = attempted
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(!attempted.is_empty());
    assert!(attempted.iter().all(|kind| matches!(
        kind,
        nuxie::RuntimeFileAssetKind::Image
            | nuxie::RuntimeFileAssetKind::Font
            | nuxie::RuntimeFileAssetKind::Audio
    )));
}

#[test]
fn wait_for_server_race_condition() {
    let queue = CommandQueue::new();
    let worker_queue = queue.clone();
    let worker = thread::spawn(move || {
        let mut server = server(&worker_queue);
        server.serve_until_disconnect();
    });
    let completed = Arc::new(AtomicUsize::new(0));
    for _ in 0..100 {
        let completed_on_server = Arc::clone(&completed);
        queue.run_once(move |_| {
            completed_on_server.fetch_add(1, Ordering::SeqCst);
        });
        let key = queue.create_draw_key();
        queue.draw(key, |_, _| {});
    }
    let completed_on_server = Arc::clone(&completed);
    queue.run_once(move |_| {
        completed_on_server.fetch_add(1, Ordering::SeqCst);
    });
    queue.disconnect();
    worker.join().expect("command server thread panicked");
    assert_eq!(completed.load(Ordering::SeqCst), 101);
}

#[test]
fn stop_messages_command() {
    let queue = CommandQueue::new();
    let count = Arc::new(AtomicUsize::new(0));
    let mut server = server(&queue);
    let first = Arc::clone(&count);
    queue.run_once(move |_| {
        first.fetch_add(1, Ordering::SeqCst);
    });
    queue.testing_command_loop_break();
    for index in 0..10 {
        let count_on_server = Arc::clone(&count);
        queue.run_once(move |_| {
            count_on_server.fetch_add(1, Ordering::SeqCst);
        });
        if index == 5 {
            queue.testing_command_loop_break();
        }
    }

    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 7);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 11);
}

#[test]
fn global_asset_set_and_remove() {
    let queue = CommandQueue::new();
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    let bad_image = queue.decode_image(vec![0; 1024], None, 0);
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), None, 0);
    let bad_audio = queue.decode_audio(vec![0; 1024], None, 0);
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), None, 0);
    let bad_font = queue.decode_font(vec![0; 1024], None, 0);
    queue.add_global_image_asset("image", image);
    queue.add_global_image_asset("bad-image", bad_image);
    queue.add_global_audio_asset("audio", audio);
    queue.add_global_audio_asset("bad-audio", bad_audio);
    queue.add_global_font_asset("font", font);
    queue.add_global_font_asset("bad-font", bad_font);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(server.global_image_named("image"), Some(image));
    assert_eq!(server.global_audio_named("audio"), Some(audio));
    assert_eq!(server.global_font_named("font"), Some(font));
    assert_eq!(server.global_image_named("bad-image"), None);
    assert_eq!(server.global_audio_named("bad-audio"), None);
    assert_eq!(server.global_font_named("bad-font"), None);

    queue.remove_global_image_asset("image");
    queue.remove_global_audio_asset("audio");
    queue.remove_global_font_asset("font");
    queue.remove_global_image_asset("missing");
    queue.remove_global_audio_asset("missing");
    queue.remove_global_font_asset("missing");
    assert!(server.process_commands());
    assert_eq!(server.global_image_named("image"), None);
    assert_eq!(server.global_audio_named("audio"), None);
    assert_eq!(server.global_font_named("font"), None);

    queue.add_global_image_asset("image", image);
    queue.add_global_audio_asset("audio", audio);
    queue.add_global_font_asset("font", font);
    queue.delete_image(image, 0);
    queue.delete_audio(audio, 0);
    queue.delete_font(font, 0);
    assert!(server.process_commands());
    assert_eq!(server.global_image_named("image"), None);
    assert_eq!(server.global_audio_named("audio"), None);
    assert_eq!(server.global_font_named("font"), None);
}

#[test]
fn external_resources() {
    let queue = CommandQueue::new();
    let image: Box<dyn RenderImage + Send> = Box::new(ExternalImage(0));
    let image_identity = (&*image as *const dyn nuxie::RenderImage as *const ()) as usize;
    let audio = Arc::new(AudioSource::from_encoded(AUDIO_FIXTURE.to_vec()).expect("decode audio"));
    let audio_identity = Arc::as_ptr(&audio) as usize;
    let font = RawTextFont::decode(Arc::<[u8]>::from(FONT_FIXTURE)).expect("decode font");
    let image_handle = queue.add_external_image(image, None, 0);
    let audio_handle = queue.add_external_audio(audio, None, 0);
    let font_handle = queue.add_external_font(font, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    let retained_image = server.image(image_handle).expect("external image");
    assert_eq!(
        (retained_image as *const dyn nuxie::RenderImage as *const ()) as usize,
        image_identity
    );
    assert_eq!(
        Arc::as_ptr(server.audio_source(audio_handle).expect("external audio")) as usize,
        audio_identity
    );
    assert_eq!(
        server
            .font(font_handle)
            .expect("external font")
            .face_index(),
        0
    );

    queue.delete_image(image_handle, 0);
    queue.delete_audio(audio_handle, 0);
    queue.delete_font(font_handle, 0);
    assert!(server.process_commands());
    assert!(server.image(image_handle).is_none());
    assert!(server.audio_source(audio_handle).is_none());
    assert!(server.font(font_handle).is_none());
}

#[test]
fn render_image() {
    let queue = CommandQueue::new();
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    let bad_image = queue.decode_image(vec![0; 1024], None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.image(image).is_some());
    assert!(server.image(bad_image).is_none());
    queue.delete_image(image, 0);
    queue.delete_image(bad_image, 0);
    assert!(server.process_commands());
    assert!(server.image(image).is_none());
    assert!(server.image(bad_image).is_none());
}

#[test]
fn audio_source() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), Some(&listener), 10);
    let bad_audio = queue.decode_audio(vec![0; 1024], None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.audio_source(audio).is_some());
    assert!(server.audio_source(bad_audio).is_none());
    queue.delete_audio(audio, 0x10);
    queue.delete_audio(bad_audio, 0);
    assert!(server.process_commands());
    assert!(server.audio_source(audio).is_none());
    assert!(server.audio_source(bad_audio).is_none());
    assert!(queue.process_messages() >= 2);
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::AudioDeleted { handle, request_id: 0x10 } if *handle == audio
    )));
}

#[test]
fn font() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), Some(&listener), 10);
    let bad_font = queue.decode_font(vec![0; 1024], None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.font(font).is_some());
    assert!(server.font(bad_font).is_none());
    queue.delete_font(font, 0x10);
    queue.delete_font(bad_font, 0);
    assert!(server.process_commands());
    assert!(server.font(font).is_none());
    assert!(server.font(bad_font).is_none());
    assert!(queue.process_messages() >= 2);
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::FontDeleted { handle, request_id: 0x10 } if *handle == font
    )));
}

#[test]
fn view_models() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_view_model_listener(Some(&listener));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let blank = queue.instantiate_blank_view_model_named(file, "Test All", None, 0);
    let default = queue.instantiate_view_model_named(file, "Test All", "", None, 0);
    let named = queue.instantiate_view_model_named(file, "Test All", "Test Alternate", None, 0);
    let nested = queue.reference_nested_view_model(blank, "Test Nested", None, 0);
    queue.insert_view_model_list(blank, "Test List", nested, Some(0), 0);
    let listed = queue.reference_list_view_model(blank, "Test List", 0, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    for handle in [blank, default, named, nested, listed] {
        assert!(server.view_model(handle).is_some());
    }
    let nested_instance = server.view_model(nested).expect("nested view model");
    let listed_instance = server.view_model(listed).expect("listed view model");
    assert!(nested_instance.handle().ptr_eq(listed_instance.handle()));

    queue.remove_view_model_list(blank, "Test List", Some(nested), None, 0);
    queue.request_view_model_list_size(blank, "Test List", 2);
    assert!(server.process_commands());
    assert!(queue.process_messages() > 0);
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelListSize { handle, request_id: 2, size: 0, .. }
            if *handle == blank
    )));

    let bad_blank = queue.instantiate_blank_view_model_named(file, "Blah", None, 0);
    let bad_named = queue.instantiate_view_model_named(file, "Blah", "Blah", None, 0);
    let bad_instance = queue.instantiate_view_model_named(file, "Test All", "Blah", None, 0);
    let bad_nested = queue.reference_nested_view_model(blank, "Blah", None, 0);
    let bad_list = queue.reference_list_view_model(blank, "Test List", 5, None, 0);
    assert!(server.process_commands());
    for handle in [bad_blank, bad_named, bad_instance, bad_nested, bad_list] {
        assert!(server.view_model(handle).is_none());
    }

    queue.delete_view_model(blank, 0);
    assert!(server.process_commands());
    assert!(server.view_model(blank).is_none());
    assert!(server.view_model(nested).is_some());
    queue.delete_view_model(nested, 0);
    assert!(server.process_commands());
    assert!(server.view_model(nested).is_none());
}

#[test]
fn view_model_listed_listener() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_view_model_names(file, 2);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(queue.process_messages() >= 2);
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelsListed { handle, request_id: 2, names }
            if *handle == file && names == &["ListViewModel", "Empty VM", "Test All", "Nested VM", "State Transition", "Alternate VM"]
    )));

    let bad = queue.load_file(vec![0; 1024 * 1024], Some(&listener), 0);
    queue.request_view_model_names(bad, 2);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(!events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelsListed { handle, .. } if *handle == bad
    )));
}

#[test]
fn view_model_listener() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_view_model_instance_names(file, "Test All", 2);
    queue.request_view_model_properties(file, "Test All", 3);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(queue.process_messages() >= 3);
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelInstancesListed { handle, request_id: 2, view_model, names }
            if *handle == file && view_model == "Test All" && names == &["Test Default", "Test Alternate"]
    )));
    let properties = captured
        .iter()
        .find_map(|event| match event {
            CommandEvent::ViewModelPropertiesListed {
                handle,
                request_id: 3,
                view_model,
                properties,
            } if *handle == file && view_model == "Test All" => Some(properties),
            _ => None,
        })
        .expect("property list callback");
    let expected = [
        (CommandDataType::Artboard, "Test Artboard", ""),
        (CommandDataType::List, "Test List", ""),
        (CommandDataType::AssetImage, "Test Image", ""),
        (CommandDataType::Number, "Test Num", ""),
        (CommandDataType::String, "Test String", ""),
        (CommandDataType::Enum, "Test Enum", "Test Enum Values"),
        (CommandDataType::Boolean, "Test Bool", ""),
        (CommandDataType::Color, "Test Color", ""),
        (CommandDataType::Trigger, "Test Trigger", ""),
        (CommandDataType::ViewModel, "Test Nested", "Nested VM"),
    ];
    assert_eq!(properties.len(), expected.len());
    for (property, (data_type, name, metadata)) in properties.iter().zip(expected) {
        assert_eq!(property.data_type, data_type);
        assert_eq!(property.name, name);
        assert_eq!(property.metadata, metadata);
    }
}

#[test]
fn view_model_instance_listener() {
    let queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_artboard_named(file, "Test Artboard", None, 0);
    let bad_artboard = queue.instantiate_artboard_named(file, "Blah", None, 0);
    let mut listeners = Vec::new();
    let mut handles = Vec::new();
    for source in 0..8 {
        let (listener, log) = event_log();
        let handle = match source {
            0 => queue.instantiate_blank_view_model_named(file, "Test All", Some(&listener), 0),
            1 => queue.instantiate_view_model_named(file, "Test All", "", Some(&listener), 0),
            2 => queue.instantiate_view_model_named(
                file,
                "Test All",
                "Test Alternate",
                Some(&listener),
                0,
            ),
            3 => queue.instantiate_view_model_named(file, "Blah", "Blah", Some(&listener), 0),
            4 => {
                queue.instantiate_view_model_for_artboard(file, artboard, None, Some(&listener), 0)
            }
            5 => queue.instantiate_view_model_for_artboard(
                file,
                artboard,
                Some(String::new()),
                Some(&listener),
                0,
            ),
            6 => queue.instantiate_view_model_for_artboard(
                file,
                artboard,
                Some("Test Alternate".to_owned()),
                Some(&listener),
                0,
            ),
            _ => queue.instantiate_view_model_for_artboard(
                file,
                bad_artboard,
                Some("Test Alternate".to_owned()),
                Some(&listener),
                0,
            ),
        };
        queue.delete_view_model(handle, 0x10);
        listeners.push((listener, log));
        handles.push(handle);
    }
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(queue.process_messages() > 0);
    for ((_, log), handle) in listeners.iter().zip(handles) {
        assert!(events(log).iter().any(|event| matches!(
            event,
            CommandEvent::ViewModelDeleted { handle: deleted, request_id: 0x10 }
                if *deleted == handle
        )));
    }
}

#[test]
fn view_model_property_set_get() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_image_listener(Some(&listener));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    let blank = queue.instantiate_blank_view_model_named(file, "Nested VM", None, 0);
    let alternate =
        queue.instantiate_view_model_named(file, "Nested VM", "Alternate Nested", None, 0);

    let mut request = 1;
    let expected_values = RefCell::new(Vec::new());
    let mut set_and_get = |path: &str, value: CommandValue, data_type| {
        request += 1;
        expected_values
            .borrow_mut()
            .push((request, path.to_owned(), value.clone()));
        queue.set_view_model_value(root, path, value, request);
        queue.request_view_model_value(root, path, data_type, request);
    };
    set_and_get(
        "Test Bool",
        CommandValue::Boolean(true),
        CommandDataType::Boolean,
    );
    set_and_get(
        "Test Num",
        CommandValue::Number(10.0),
        CommandDataType::Number,
    );
    set_and_get(
        "Test Nested/Nested Number",
        CommandValue::Number(10.0),
        CommandDataType::Number,
    );
    set_and_get(
        "Test Nested",
        CommandValue::ViewModel(blank),
        CommandDataType::ViewModel,
    );
    set_and_get(
        "Test Nested/Nested Number",
        CommandValue::Number(10.0),
        CommandDataType::Number,
    );
    set_and_get(
        "Test Color",
        CommandValue::Color(0xffff_0000),
        CommandDataType::Color,
    );
    set_and_get(
        "Test Enum",
        CommandValue::Enum("Value 2".to_owned()),
        CommandDataType::Enum,
    );
    set_and_get(
        "Test String",
        CommandValue::String("Some String".to_owned()),
        CommandDataType::String,
    );

    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    queue.set_view_model_value(root, "Test Image", CommandValue::Image(Some(image)), 0);
    queue.run_once(move |server| {
        let expected = server.image(image).expect("decoded image");
        let actual = server
            .view_model(root)
            .expect("root view model")
            .raw()
            .runtime_image_by_property_name_path("Test Image")
            .and_then(|image| image.render_image())
            .expect("image property");
        assert!(std::ptr::eq(actual.as_any(), expected.as_any()));
    });

    let external: Box<dyn RenderImage + Send> = Box::new(ExternalImage(7));
    let external_image = queue.add_external_image(external, None, 0);
    queue.set_view_model_value(
        root,
        "Test Image",
        CommandValue::Image(Some(external_image)),
        0,
    );
    queue.request_view_model_value(root, "Test Image", CommandDataType::AssetImage, 70);
    expected_values.borrow_mut().push((
        70,
        "Test Image".to_owned(),
        CommandValue::Image(Some(external_image)),
    ));
    queue.run_once(move |server| {
        let expected = server.image(external_image).expect("external image");
        let actual = server
            .view_model(root)
            .expect("root view model")
            .raw()
            .runtime_image_by_property_name_path("Test Image")
            .and_then(|image| image.render_image())
            .expect("external image property");
        assert!(std::ptr::eq(actual.as_any(), expected.as_any()));
    });

    let bindable = queue.instantiate_default_artboard(file, None, 0);
    queue.set_view_model_value(
        root,
        "Test Artboard",
        CommandValue::Artboard(Some(bindable)),
        0,
    );
    queue.run_once(move |server| {
        let expected = server
            .bindable_artboard(bindable)
            .expect("bindable artboard");
        let actual = server
            .view_model(root)
            .expect("root view model")
            .raw()
            .runtime_artboard_by_property_name("Test Artboard")
            .expect("artboard property");
        assert!(actual.ptr_eq(expected));
    });
    queue.delete_artboard(bindable, 0);

    let bad_image = queue.decode_image(vec![0; 1024 * 1024], None, 0);
    queue.set_view_model_value(root, "Test Image", CommandValue::Image(Some(bad_image)), 20);
    queue.set_view_model_value(
        root,
        "Test Artboard",
        CommandValue::Artboard(Some(artboard)),
        0,
    );
    let bad_artboard = queue.instantiate_artboard_named(file, "Blah", None, 0);
    queue.set_view_model_value(
        root,
        "Test Artboard",
        CommandValue::Artboard(Some(bad_artboard)),
        21,
    );
    queue.set_view_model_value(root, "Blah", CommandValue::Image(Some(image)), 22);
    queue.set_view_model_value(root, "Blah", CommandValue::Artboard(Some(artboard)), 23);
    queue.run_once(move |server| {
        let expected_image = server.image(external_image).expect("external image");
        let retained_image = server
            .view_model(root)
            .expect("root view model")
            .raw()
            .runtime_image_by_property_name_path("Test Image")
            .and_then(|image| image.render_image())
            .expect("failed image set retains prior value");
        assert!(std::ptr::eq(
            retained_image.as_any(),
            expected_image.as_any()
        ));
        let expected_artboard = server.bindable_artboard(artboard).expect("main artboard");
        let root = server.view_model(root).expect("root view model");
        let actual_artboard = root
            .raw()
            .runtime_artboard_by_property_name("Test Artboard")
            .expect("failed artboard set retains prior value");
        assert!(actual_artboard.ptr_eq(expected_artboard));
    });
    queue.set_view_model_value(root, "Test Image", CommandValue::Image(None), 0);
    queue.set_view_model_value(root, "Test Artboard", CommandValue::Artboard(None), 0);
    queue.run_once(move |server| {
        let root = server.view_model(root).expect("root view model");
        let raw = root.raw();
        assert!(
            raw.runtime_image_by_property_name_path("Test Image")
                .and_then(|image| image.render_image())
                .is_none()
        );
        assert!(
            raw.runtime_artboard_by_property_name("Test Artboard")
                .is_none()
        );
    });

    for index in 0..10 {
        set_and_get(
            "Test Bool",
            CommandValue::Boolean(index % 2 != 0),
            CommandDataType::Boolean,
        );
        set_and_get(
            "Test Num",
            CommandValue::Number(index as f32),
            CommandDataType::Number,
        );
        set_and_get(
            "Test Nested",
            CommandValue::ViewModel(if index % 2 != 0 { blank } else { alternate }),
            CommandDataType::ViewModel,
        );
        set_and_get(
            "Test Color",
            CommandValue::Color(u32::from_ne_bytes([index; 4])),
            CommandDataType::Color,
        );
        set_and_get(
            "Test Enum",
            CommandValue::Enum(if index % 2 != 0 { "Value 2" } else { "Value 1" }.to_owned()),
            CommandDataType::Enum,
        );
        set_and_get(
            "Test String",
            CommandValue::String(index.to_string()),
            CommandDataType::String,
        );
    }
    drop(set_and_get);

    queue.delete_view_model(blank, 0);
    queue.delete_view_model(alternate, 0);
    queue.set_view_model_value(root, "Test Enum", CommandValue::Enum("Blah".to_owned()), 30);
    queue.set_view_model_value(root, "Test Nested", CommandValue::ViewModel(blank), 31);
    queue.request_view_model_value(root, "Test Enum", CommandDataType::Enum, 33);
    expected_values.borrow_mut().push((
        33,
        "Test Enum".to_owned(),
        CommandValue::Enum("Value 2".to_owned()),
    ));
    queue.request_view_model_value(
        root,
        "Test Nested/Nested Number",
        CommandDataType::Number,
        34,
    );
    expected_values.borrow_mut().push((
        34,
        "Test Nested/Nested Number".to_owned(),
        CommandValue::Number(10.0),
    ));
    for value in [
        CommandValue::Boolean(true),
        CommandValue::Number(10.0),
        CommandValue::ViewModel(alternate),
        CommandValue::Color(0xffff_0000),
        CommandValue::Enum("Value 2".to_owned()),
        CommandValue::String("Some String".to_owned()),
    ] {
        queue.set_view_model_value(root, "Blah", value, 32);
    }
    queue.delete_view_model(root, 40);
    for value in [
        CommandValue::Boolean(true),
        CommandValue::Number(10.0),
        CommandValue::Color(0xffff_0000),
        CommandValue::Enum("Value 2".to_owned()),
        CommandValue::String("Some String".to_owned()),
    ] {
        queue.set_view_model_value(root, "Test Bool", value, 41);
    }

    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(queue.process_messages() > 0);
    let captured = events(&log);
    let actual_values = captured
        .iter()
        .filter_map(|event| match event {
            CommandEvent::ViewModelValue {
                handle,
                request_id,
                path,
                value,
            } if *handle == root => Some((*request_id, path.clone(), value.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(actual_values, expected_values.into_inner());
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelDeleted { handle, request_id: 40 } if *handle == root
    )));
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, CommandEvent::ViewModelError { .. }))
            .count(),
        17,
        "{:?}",
        captured
            .iter()
            .filter(|event| matches!(event, CommandEvent::ViewModelError { .. }))
            .collect::<Vec<_>>()
    );
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::ImageError { handle, .. } if *handle == bad_image
    )));
}

#[test]
fn set_and_reset_artboard_size() {
    let queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    let original = server.artboard(artboard).unwrap().artboard_dimensions();
    queue.set_artboard_size(artboard, 1000.0, 1000.0, 1.0, 0);
    assert!(server.process_commands());
    assert_eq!(
        server.artboard(artboard).unwrap().artboard_dimensions(),
        (1000.0, 1000.0)
    );
    queue.set_artboard_size(artboard, 1000.0, 1000.0, 2.0, 0);
    assert!(server.process_commands());
    assert_eq!(
        server.artboard(artboard).unwrap().artboard_dimensions(),
        (500.0, 500.0)
    );
    queue.reset_artboard_size(artboard, 0);
    assert!(server.process_commands());
    assert_eq!(
        server.artboard(artboard).unwrap().artboard_dimensions(),
        original
    );
}

#[test]
fn set_and_get_artboard_volume() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, Some(&listener), 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.set_artboard_volume(artboard, 0.5, 0);
    assert!(server.process_commands());
    assert_eq!(server.artboard(artboard).unwrap().volume(), 0.5);
    queue.set_artboard_volume(artboard, 0.0, 0);
    assert!(server.process_commands());
    assert_eq!(server.artboard(artboard).unwrap().volume(), 0.0);
    queue.set_artboard_volume(artboard, 0.75, 0);
    queue.request_artboard_volume(artboard, 0x50);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::ArtboardVolume { handle, request_id: 0x50, volume }
            if *handle == artboard && *volume == 0.75
    )));
}

#[test]
fn view_model_property_subscriptions() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    queue.set_view_model_value(root, "Test Bool", CommandValue::Boolean(false), 0);
    queue.set_view_model_value(root, "Test Color", CommandValue::Color(0), 0);
    for (path, data_type) in [
        ("Test Nested/Nested Number", CommandDataType::Number),
        ("Test Bool", CommandDataType::Boolean),
        ("Test Num", CommandDataType::Number),
        ("Test Color", CommandDataType::Color),
        ("Test Enum", CommandDataType::Enum),
        ("Test String", CommandDataType::String),
        ("Test Trigger", CommandDataType::Trigger),
        ("Test List", CommandDataType::List),
        ("Test Image", CommandDataType::AssetImage),
    ] {
        queue.subscribe_to_view_model_property(root, path, data_type, 0);
    }
    queue.subscribe_to_view_model_property(root, "Bad property", CommandDataType::AssetImage, 1);
    queue.subscribe_to_view_model_property(root, "Test Image", CommandDataType::Integer, 2);
    queue.run_once(|server| assert_eq!(server.testing_subscription_count(), 9));
    let blank = queue.instantiate_blank_view_model_named(file, "Nested VM", None, 0);
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    queue.set_view_model_value(root, "Test Bool", CommandValue::Boolean(true), 0);
    queue.set_view_model_value(root, "Test Num", CommandValue::Number(10.0), 0);
    queue.set_view_model_value(
        root,
        "Test Nested/Nested Number",
        CommandValue::Number(10.0),
        0,
    );
    queue.set_view_model_value(root, "Test Color", CommandValue::Color(0xffff_0000), 0);
    queue.set_view_model_value(root, "Test Enum", CommandValue::Enum("Value 2".into()), 0);
    queue.set_view_model_value(
        root,
        "Test String",
        CommandValue::String("Some String".into()),
        0,
    );
    queue.set_view_model_value(root, "Test Trigger", CommandValue::Trigger, 0);
    queue.set_view_model_value(root, "Test Image", CommandValue::Image(Some(image)), 0);
    queue.set_view_model_value(root, "Test Nested", CommandValue::ViewModel(blank), 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    for path in [
        "Test Nested/Nested Number",
        "Test Bool",
        "Test Num",
        "Test Color",
        "Test Enum",
        "Test String",
        "Test Trigger",
        "Test Image",
    ] {
        assert!(
            captured.iter().any(|event| matches!(
                event,
                CommandEvent::ViewModelValue { path: actual, .. } if actual == path
            )),
            "missing subscription callback for {path}"
        );
    }
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(
                event,
                CommandEvent::ViewModelError {
                    request_id: 1 | 2,
                    ..
                }
            ))
            .count(),
        2
    );
    for (path, data_type) in [
        ("Test Nested/Nested Number", CommandDataType::Number),
        ("Test Bool", CommandDataType::Boolean),
        ("Test Num", CommandDataType::Number),
        ("Test Color", CommandDataType::Color),
        ("Test Enum", CommandDataType::Enum),
        ("Test String", CommandDataType::String),
        ("Test Trigger", CommandDataType::Trigger),
        ("Test List", CommandDataType::List),
        ("Test Image", CommandDataType::AssetImage),
    ] {
        queue.unsubscribe_from_view_model_property(root, path, data_type);
    }
    queue.unsubscribe_from_view_model_property(root, "Blah", CommandDataType::Boolean);
    queue.run_once(|server| assert_eq!(server.testing_subscription_count(), 0));
    assert!(server.process_commands());
}

#[test]
fn view_model_property_async_subscriptions() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    queue.set_view_model_value(root, "Test Num", CommandValue::Number(0.0), 0);
    queue.subscribe_to_view_model_property(root, "Test Num", CommandDataType::Number, 0);
    queue.set_view_model_value(root, "Test Num", CommandValue::Number(10.0), 0);
    let ready = Arc::new(AtomicUsize::new(0));
    let ready_on_server = Arc::clone(&ready);
    queue.run_once(move |server| {
        assert_eq!(server.testing_subscription_count(), 1);
        ready_on_server.store(1, Ordering::Release);
    });
    queue.testing_command_loop_break();
    let worker_queue = queue.clone();
    let worker = thread::spawn(move || {
        let mut server = server(&worker_queue);
        while server.wait_commands() {}
    });
    while ready.load(Ordering::Acquire) == 0 {
        thread::yield_now();
    }
    for _ in 0..10_000 {
        queue.process_messages();
        if events(&log).iter().any(|event| {
            matches!(
                event,
                CommandEvent::ViewModelValue { handle, path, value: CommandValue::Number(10.0), .. }
                    if *handle == root && path == "Test Num"
            )
        }) {
            break;
        }
        thread::yield_now();
    }
    assert!(events(&log).iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelValue { handle, path, value: CommandValue::Number(10.0), .. }
            if *handle == root && path == "Test Num"
    )));
    queue.unsubscribe_from_view_model_property(root, "Test Num", CommandDataType::Number);
    queue.run_once(|server| assert_eq!(server.testing_subscription_count(), 0));
    queue.disconnect();
    worker.join().expect("server thread");
}

#[test]
fn list_view_model_property_set_get() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let root = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    let blank = queue.instantiate_blank_view_model_named(file, "Nested VM", None, 0);
    let alternate =
        queue.instantiate_view_model_named(file, "Nested VM", "Alternate Nested", None, 0);
    queue.insert_view_model_list(root, "Test List", blank, None, 0);
    queue.insert_view_model_list(root, "Test List", alternate, None, 0);
    queue.swap_view_model_list(root, "Test List", 2, 3, 0);
    queue.run_once(move |server| {
        let items = server
            .testing_view_model_list_handles(root, "Test List")
            .expect("list property");
        assert_eq!(items.len(), 4);
        assert_eq!(items[2], Some(alternate));
        assert_eq!(items[3], Some(blank));
    });
    queue.request_view_model_list_size(root, "Test List", 1);
    queue.insert_view_model_list(root, "Test List", blank, Some(0), 0);
    queue.insert_view_model_list(root, "Test List", alternate, Some(0), 0);
    queue.swap_view_model_list(root, "Test List", 0, 1, 0);
    queue.run_once(move |server| {
        let items = server
            .testing_view_model_list_handles(root, "Test List")
            .expect("list property");
        assert_eq!(items.len(), 6);
        assert_eq!(items[0], Some(blank));
        assert_eq!(items[1], Some(alternate));
        assert_eq!(items[4], Some(alternate));
        assert_eq!(items[5], Some(blank));
    });
    queue.request_view_model_list_size(root, "Test List", 2);
    let bad_blank = queue.instantiate_blank_view_model_named(file, "blah", None, 0);
    let bad_alternate = queue.instantiate_view_model_named(file, "Nested VM", "blah", None, 0);
    for (path, value, index) in [
        ("Test List", bad_blank, None),
        ("Test List", bad_alternate, None),
        ("Test List", bad_blank, Some(0)),
        ("Test List", bad_alternate, Some(0)),
        ("blah", blank, None),
        ("blah", alternate, None),
        ("blah", blank, Some(0)),
        ("blah", alternate, Some(0)),
    ] {
        queue.insert_view_model_list(root, path, value, index, 3);
    }
    for (path, a, b) in [
        ("Test List", 10, 1),
        ("Test List", 0, 10),
        ("Blah", 0, 1),
        ("Blah", 10, 1),
        ("Blah", 0, 10),
    ] {
        queue.swap_view_model_list(root, path, a, b, 4);
    }
    queue.run_once(move |server| {
        let items = server
            .testing_view_model_list_handles(root, "Test List")
            .expect("invalid operations retain list");
        assert_eq!(items.len(), 6);
        assert_eq!(items[0], Some(blank));
        assert_eq!(items[1], Some(alternate));
        assert_eq!(items[4], Some(alternate));
        assert_eq!(items[5], Some(blank));
    });
    queue.request_view_model_list_size(root, "Test List", 5);
    queue.request_view_model_list_size(root, "Blah", 6);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelListSize {
            request_id: 1,
            size: 4,
            ..
        }
    )));
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelListSize {
            request_id: 2 | 5,
            size: 6,
            ..
        }
    )));
    assert!(
        !captured
            .iter()
            .any(|event| matches!(event, CommandEvent::ViewModelListSize { request_id: 6, .. }))
    );
    assert_eq!(
        captured
            .iter()
            .filter(|event| matches!(event, CommandEvent::ViewModelError { .. }))
            .count(),
        14
    );
}

#[test]
fn file_error_messages() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    queue.instantiate_artboard_named(file, "Blah", None, 0);
    queue.instantiate_view_model_named(file, "Test All", "blah", None, 0);
    queue.instantiate_view_model_named(file, "blah", "blah", None, 0);
    queue.instantiate_view_model_named(file, "", "blah", None, 0);
    queue.instantiate_view_model_named(file, "Blah", "", None, 0);
    queue.instantiate_view_model_named(file, "", "", None, 0);
    queue.instantiate_blank_view_model_named(file, "Blah", None, 0);
    queue.instantiate_blank_view_model_named(file, "", None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&log)
            .iter()
            .filter(
                |event| matches!(event, CommandEvent::FileError { handle, .. } if *handle == file)
            )
            .count(),
        8
    );

    let (bad_listener, bad_log) = event_log();
    let bad = queue.load_file(vec![0; 100 * 1024], Some(&bad_listener), 0);
    queue.instantiate_default_artboard(bad, None, 0);
    queue.instantiate_blank_view_model_named(bad, "", None, 0);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&bad_log)
            .iter()
            .filter(
                |event| matches!(event, CommandEvent::FileError { handle, .. } if *handle == bad)
            )
            .count(),
        3
    );

    let (no_vm_listener, no_vm_log) = event_log();
    let no_vm = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&no_vm_listener), 0);
    let no_vm_artboard = queue.instantiate_default_artboard(no_vm, None, 0);
    for instance in [None, Some(String::new()), Some("Nonexistent".into())] {
        queue.instantiate_view_model_for_artboard(no_vm, no_vm_artboard, instance, None, 0);
    }
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(
        events(&no_vm_log)
            .iter()
            .filter(
                |event| matches!(event, CommandEvent::FileError { handle, .. } if *handle == no_vm)
            )
            .count(),
        3
    );
}

#[test]
fn list_artboard() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let good = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_artboard_names(good, 0x40);
    let bad = queue.load_file(vec![0; 100 * 1024], None, 0);
    queue.request_artboard_names(bad, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ArtboardsListed { handle, request_id: 0x40, names } if *handle == good && names == &["New Artboard", "New Artboard"])));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::ArtboardsListed { handle, .. } if *handle == bad)
    ));
}

#[test]
fn list_enums() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let good = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_view_model_enums(good, 0x40);
    let bad = queue.load_file(vec![0; 100 * 1024], None, 0);
    queue.request_view_model_enums(bad, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ViewModelEnumsListed { handle, request_id: 0x40, enums } if *handle == good && enums.len() == 1 && enums[0].name == "Test Enum Values" && enums[0].enumerants == ["Value 1", "Value 2"])), "{captured:?}");
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::ViewModelEnumsListed { handle, .. } if *handle == bad)
    ));
}

#[test]
fn request_view_model_and_instance_name() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let view_model = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    queue.request_view_model_name(view_model, 0x50);
    queue.request_view_model_instance_name(view_model, 0x50);
    let (bad_listener, bad_log) = event_log();
    let bad = queue.instantiate_view_model_named(file, "Blah", "Blah", Some(&bad_listener), 0);
    queue.request_view_model_name(bad, 0x51);
    queue.request_view_model_instance_name(bad, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ViewModelName { handle, request_id: 0x50, name } if *handle == view_model && name == "Test All")));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ViewModelInstanceName { handle, request_id: 0x50, name } if *handle == view_model && name == "Test Default")));
    assert!(!events(&bad_log).iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelName { .. } | CommandEvent::ViewModelInstanceName { .. }
    )));
    assert!(events(&bad_log).iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelError {
            request_id: 0x52,
            ..
        }
    )));
}

#[test]
fn render_image_audio_source_font_error() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let image = queue.decode_image(vec![0; 1024], Some(&listener), 1);
    let audio = queue.decode_audio(vec![0; 1024], Some(&listener), 2);
    let font = queue.decode_font(vec![0; 1024], Some(&listener), 3);
    assert!(server(&queue).process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(
        captured.iter().any(
            |event| matches!(event, CommandEvent::ImageError { handle, .. } if *handle == image)
        )
    );
    assert!(
        captured.iter().any(
            |event| matches!(event, CommandEvent::AudioError { handle, .. } if *handle == audio)
        )
    );
    assert!(
        captured.iter().any(
            |event| matches!(event, CommandEvent::FontError { handle, .. } if *handle == font)
        )
    );
}

#[test]
fn state_machine_error() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let good = queue.instantiate_default_state_machine(artboard, Some(&listener), 0);
    let bad_vm = queue.instantiate_blank_view_model_named(file, "missing", None, 0);
    queue.bind_view_model(good, bad_vm, 1);
    let bad = queue.instantiate_default_state_machine(
        queue.instantiate_artboard_named(file, "missing", None, 0),
        Some(&listener),
        0,
    );
    let pointer = nuxie::command_queue::PointerEvent::default();
    queue.advance_state_machine(bad, 0.0, 2);
    queue.pointer_down(bad, pointer, 3);
    queue.pointer_exit(bad, pointer, 4);
    queue.pointer_up(bad, pointer, 5);
    queue.pointer_move(bad, pointer, 6);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(captured.iter().filter(|event| matches!(event, CommandEvent::StateMachineError { handle, .. } if *handle == good)).count(), 1);
    assert_eq!(captured.iter().filter(|event| matches!(event, CommandEvent::StateMachineError { handle, .. } if *handle == bad)).count(), 5);
}

#[test]
fn artboard_errors() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let good = queue.instantiate_artboard_named(file, "New Artboard", Some(&listener), 0);
    queue.instantiate_state_machine_named(good, "Blah", None, 1);
    let bad = queue.instantiate_artboard_named(file, "Blah", Some(&listener), 0);
    queue.request_state_machine_names(bad, 2);
    queue.instantiate_default_state_machine(bad, None, 3);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert_eq!(captured.iter().filter(|event| matches!(event, CommandEvent::ArtboardError { handle, .. } if *handle == good)).count(), 1);
    assert_eq!(captured.iter().filter(|event| matches!(event, CommandEvent::ArtboardError { handle, .. } if *handle == bad)).count(), 2);
}

#[test]
fn invalid_artboard_volume_errors() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let invalid = queue.instantiate_artboard_named(file, "missing", None, 0);
    queue.set_artboard_volume(invalid, 0.5, 0x51);
    queue.request_artboard_volume(invalid, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let ids = events(&log)
        .into_iter()
        .filter_map(|event| match event {
            CommandEvent::ArtboardError {
                handle, request_id, ..
            } if handle == invalid => Some(request_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, [0x51, 0x52]);
}

#[test]
fn invalid_artboard_size_errors() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener));
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let invalid = queue.instantiate_artboard_named(file, "missing", None, 0);
    queue.set_artboard_size(invalid, 10.0, 10.0, 1.0, 0x51);
    queue.reset_artboard_size(invalid, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let ids = events(&log)
        .into_iter()
        .filter_map(|event| match event {
            CommandEvent::ArtboardError {
                handle, request_id, ..
            } if handle == invalid => Some(request_id),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ids, [0x51, 0x52]);
}

#[test]
fn list_state_machine() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_artboard_named(file, "New Artboard", Some(&listener), 0);
    queue.request_state_machine_names(artboard, 0x40);
    let bad_file = queue.load_file(vec![0; 100 * 1024], None, 0);
    let bad = queue.instantiate_default_artboard(bad_file, None, 0);
    queue.request_state_machine_names(bad, 0x41);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::StateMachinesListed { handle, request_id: 0x40, names } if *handle == artboard && names == &["State Machine 1"])));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::StateMachinesListed { handle, .. } if *handle == bad)
    ));
}

#[test]
fn request_artboard_size() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, Some(&listener), 0);
    queue.request_artboard_size(artboard, 0x50);
    queue.set_artboard_size(artboard, 1000.0, 500.0, 1.0, 0);
    queue.request_artboard_size(artboard, 0x51);
    let invalid = queue.instantiate_artboard_named(file, "missing", None, 0);
    queue.request_artboard_size(invalid, 0x52);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ArtboardSize { handle, request_id: 0x50, .. } if *handle == artboard)));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ArtboardSize { handle, request_id: 0x51, width: 1000.0, height: 500.0 } if *handle == artboard)));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ArtboardError { handle, request_id: 0x52, .. } if *handle == invalid)));
}

#[test]
fn request_default_view_model() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_artboard_listener(Some(&listener));
    let good_file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let good = queue.instantiate_artboard_named(good_file, "Test Artboard", Some(&listener), 0);
    queue.request_default_view_model(good, good_file, 0x40);
    let bad_file = queue.load_file(vec![0; 100 * 1024], None, 0);
    let bad = queue.instantiate_default_artboard(bad_file, None, 0);
    queue.request_default_view_model(bad, good_file, 0x41);
    queue.request_default_view_model(good, bad_file, 0x42);
    queue.request_default_view_model(bad, bad_file, 0x43);
    let no_vm_file = queue.load_file(ENTRY_FIXTURE.to_vec(), None, 0);
    let no_vm = queue.instantiate_default_artboard(no_vm_file, Some(&listener), 0);
    queue.request_default_view_model(no_vm, no_vm_file, 0x44);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::DefaultViewModel { handle, request_id: 0x40, view_model, instance } if *handle == good && view_model == "Test All" && instance == "Test Default")));
    for request_id in [0x41, 0x42, 0x43, 0x44] {
        assert!(captured.iter().any(|event| matches!(event, CommandEvent::ArtboardError { request_id: actual, .. } if *actual == request_id)));
    }
}

#[test]
fn bind_view_model_instance() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_state_machine_listener(Some(&listener));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let view_model =
        queue.instantiate_view_model_named(file, "Test All", "Test Alternate", None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let state_machine = queue.instantiate_default_state_machine(artboard, None, 0);
    queue.bind_view_model(state_machine, view_model, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.state_machine(state_machine).is_some());
    assert!(server.view_model(view_model).is_some());
    assert!(server.testing_state_machine_view_model_is(state_machine, view_model));
    let bad_vm = queue.instantiate_view_model_named(file, "blah", "Test Alternate", None, 0);
    let bad_machine = queue.instantiate_state_machine_named(artboard, "blah", None, 0);
    queue.bind_view_model(state_machine, bad_vm, 1);
    queue.bind_view_model(bad_machine, view_model, 2);
    queue.bind_view_model(bad_machine, bad_vm, 3);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    for request_id in 1..=3 {
        assert!(captured.iter().any(|event| matches!(
            event,
            CommandEvent::StateMachineError { request_id: actual, .. }
                if *actual == request_id
        )));
    }
}

#[test]
fn advance_state_machine() {
    const SETTLER: &[u8] = include_bytes!("../../../fixtures/command_queue/settler.riv");
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(SETTLER.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, Some(&listener), 0);
    queue.advance_state_machine(machine, 10.0, 0);
    queue.advance_state_machine(machine, 10.0, 0);
    queue.advance_state_machine(machine, 10.0, 0x50);
    let bad = queue.instantiate_state_machine_named(artboard, "blah blah", Some(&listener), 0);
    queue.advance_state_machine(bad, 10.0, 0x51);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::StateMachineSettled { handle, request_id: 0x50 } if *handle == machine)));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::StateMachineSettled { handle, .. } if *handle == bad)
    ));
}

#[test]
fn listener_delete_callbacks() {
    let queue = CommandQueue::new();
    let (file_listener, file_log) = event_log();
    let (artboard_listener, artboard_log) = event_log();
    let (machine_listener, machine_log) = event_log();
    let (image_listener, image_log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&file_listener), 0);
    let artboard =
        queue.instantiate_artboard_named(file, "New Artboard", Some(&artboard_listener), 0);
    let machine = queue.instantiate_default_state_machine(artboard, Some(&machine_listener), 0);
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), Some(&image_listener), 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    for log in [&file_log, &artboard_log, &machine_log, &image_log] {
        assert!(!events(log).iter().any(|event| matches!(
            event,
            CommandEvent::FileDeleted { .. }
                | CommandEvent::ArtboardDeleted { .. }
                | CommandEvent::StateMachineDeleted { .. }
                | CommandEvent::ImageDeleted { .. }
        )));
    }
    queue.delete_state_machine(machine, 0x50);
    queue.delete_artboard(artboard, 0x51);
    queue.delete_file(file, 0x52);
    queue.delete_image(image, 0x53);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&file_log).iter().any(|event| matches!(event, CommandEvent::FileDeleted { handle, request_id: 0x52 } if *handle == file)));
    assert!(events(&artboard_log).iter().any(|event| matches!(event, CommandEvent::ArtboardDeleted { handle, request_id: 0x51 } if *handle == artboard)));
    assert!(events(&machine_log).iter().any(|event| matches!(event, CommandEvent::StateMachineDeleted { handle, request_id: 0x50 } if *handle == machine)));
    assert!(events(&image_log).iter().any(|event| matches!(event, CommandEvent::ImageDeleted { handle, request_id: 0x53 } if *handle == image)));
}

#[test]
fn file_loaded_callback() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let good = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&listener), 0x10);
    let (bad_listener, bad_log) = event_log();
    let bad = queue.load_file(vec![0; 1024], Some(&bad_listener), 0x10);
    assert!(server(&queue).process_commands());
    queue.process_messages();
    assert!(events(&log).iter().any(|event| matches!(event, CommandEvent::FileLoaded { handle, request_id: 0x10 } if *handle == good)));
    assert!(
        !events(&bad_log).iter().any(
            |event| matches!(event, CommandEvent::FileLoaded { handle, .. } if *handle == bad)
        )
    );
}

#[test]
fn artboard_instantiated_callback() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), Some(&listener), 0);
    let good = queue.instantiate_artboard_named(file, "One", None, 0x10);
    let bad = queue.instantiate_artboard_named(file, "Blah", None, 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ArtboardInstantiated { file: actual_file, handle, request_id: 0x10 } if *actual_file == file && *handle == good)));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::ArtboardInstantiated { handle, .. } if *handle == bad)
    ));
}

#[test]
fn state_machine_instantiated_callback() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, Some(&listener), 0);
    let good = queue.instantiate_state_machine_named(artboard, "one", None, 0x10);
    let bad = queue.instantiate_state_machine_named(artboard, "blahblahblah", None, 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::StateMachineInstantiated { artboard: actual_artboard, handle, request_id: 0x10 } if *actual_artboard == artboard && *handle == good)));
    assert!(!captured.iter().any(|event| matches!(event, CommandEvent::StateMachineInstantiated { handle, .. } if *handle == bad)));
}

#[test]
fn view_model_instance_instantiated_callback() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    let good = queue.instantiate_view_model_named(file, "Test All", "Test Alternate", None, 0x10);
    let bad = queue.instantiate_view_model_named(file, "Test All", "Blah", None, 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ViewModelInstantiated { file: actual_file, handle, request_id: 0x10 } if *actual_file == file && *handle == good)));
    assert!(!captured.iter().any(|event| matches!(event, CommandEvent::ViewModelInstantiated { handle, .. } if *handle == bad)));
}

#[test]
fn decoded_callbacks() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), Some(&listener), 0x10);
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), Some(&listener), 0x10);
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), Some(&listener), 0x10);
    let bad_image = queue.decode_image(vec![0; 1024], Some(&listener), 0x11);
    let bad_audio = queue.decode_audio(vec![0; 1024], Some(&listener), 0x11);
    let bad_font = queue.decode_font(vec![0; 1024], Some(&listener), 0x11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ImageDecoded { handle, request_id: 0x10 } if *handle == image)));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::AudioDecoded { handle, request_id: 0x10 } if *handle == audio)));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::FontDecoded { handle, request_id: 0x10 } if *handle == font)));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::ImageDecoded { handle, .. } if *handle == bad_image)
    ));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::AudioDecoded { handle, .. } if *handle == bad_audio)
    ));
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::FontDecoded { handle, .. } if *handle == bad_font)
    ));
}

#[test]
fn listener_lifetimes() {
    let queue = CommandQueue::new();
    let (file_listener, file_log) = event_log();
    let (artboard_listener, artboard_log) = event_log();
    let (machine_listener, machine_log) = event_log();
    let file = queue.load_file(ENTRY_FIXTURE.to_vec(), Some(&file_listener), 0);
    let artboard = queue.instantiate_default_artboard(file, Some(&artboard_listener), 0);
    let machine = queue.instantiate_default_state_machine(artboard, Some(&machine_listener), 0);
    queue.request_artboard_names(file, 1);
    queue.request_state_machine_names(artboard, 2);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(!events(&file_log).is_empty());
    assert!(!events(&artboard_log).is_empty());
    let moved_file_listener = Arc::clone(&file_listener);
    let moved_artboard_listener = Arc::clone(&artboard_listener);
    let moved_machine_listener = Arc::clone(&machine_listener);
    drop(file_listener);
    drop(artboard_listener);
    drop(machine_listener);
    queue.delete_state_machine(machine, 3);
    queue.delete_artboard(artboard, 4);
    queue.delete_file(file, 5);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&machine_log).iter().any(|event| matches!(event, CommandEvent::StateMachineDeleted { handle, .. } if *handle == machine)));
    assert!(events(&artboard_log).iter().any(
        |event| matches!(event, CommandEvent::ArtboardDeleted { handle, .. } if *handle == artboard)
    ));
    assert!(
        events(&file_log).iter().any(
            |event| matches!(event, CommandEvent::FileDeleted { handle, .. } if *handle == file)
        )
    );
    drop((
        moved_file_listener,
        moved_artboard_listener,
        moved_machine_listener,
    ));
    let (ephemeral, ephemeral_log) = event_log();
    queue.load_file(vec![0; 1024], Some(&ephemeral), 6);
    drop(ephemeral);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&ephemeral_log).is_empty());
}

#[test]
fn empty_listener_code_coverage() {
    let queue = CommandQueue::new();
    let listener: Listener = Arc::new(|_: &CommandEvent| {});
    let file = queue.load_file(Vec::new(), Some(&listener), 0);
    queue.delete_file(file, 0);
    assert!(server(&queue).process_commands());
    assert_eq!(queue.process_messages(), 2);
}

fn pointer_at(x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        position: nuxie::Vec2D::new(x, y),
        ..PointerEvent::default()
    }
}

#[test]
fn pointer_input() {
    let queue = CommandQueue::new();
    let file = queue.load_file(POINTER_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    queue.run_once(move |server| {
        assert!(server.file(file).is_some());
        assert!(server.artboard(artboard).is_some());
        assert!(server.state_machine(machine).is_some());
    });
    queue.advance_state_machine(machine, 0.0, 0);
    let assert_bool = |expected| {
        queue.run_once(move |server| {
            assert_eq!(
                server
                    .state_machine(machine)
                    .and_then(|machine| machine.get_bool("isDown"))
                    .and_then(|input| input.bool_value()),
                Some(expected)
            );
        });
    };
    queue.pointer_down(machine, pointer_at(425.0, 425.0), 0);
    assert_bool(true);
    queue.pointer_up(machine, pointer_at(425.0, 425.0), 0);
    assert_bool(true);
    queue.pointer_down(machine, pointer_at(425.0, 425.0), 0);
    assert_bool(false);
    queue.pointer_down(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(true);
    queue.pointer_up(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(false);
    queue.pointer_move(machine, pointer_at(250.0, 250.0), 0);
    assert_bool(false);
    queue.pointer_move(machine, pointer_at(425.0, 75.0), 0);
    assert_bool(true);
    queue.pointer_move(machine, pointer_at(250.0, 250.0), 0);
    assert_bool(true);
    queue.pointer_move(machine, pointer_at(425.0, 75.0), 0);
    assert_bool(false);
    queue.pointer_down(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(true);
    queue.pointer_exit(machine, pointer_at(-25.0, -25.0), 0);
    assert_bool(true);
    queue.pointer_up(machine, pointer_at(-25.0, -25.0), 0);
    assert_bool(true);
    queue.pointer_up(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(false);
    queue.pointer_down(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(true);
    queue.pointer_exit(machine, pointer_at(-25.0, -25.0), 0);
    assert_bool(true);
    queue.pointer_move(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(true);
    queue.pointer_up(machine, pointer_at(75.0, 75.0), 0);
    assert_bool(false);
    assert!(server(&queue).process_commands());
}

#[test]
fn pointer_down_advances_before_rapid_pointer_up() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(RAPID_POINTER_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let view_model = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    queue.bind_view_model(machine, view_model, 0);
    queue.subscribe_to_view_model_property(view_model, "hasReached", CommandDataType::Boolean, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(
        !events(&log)
            .iter()
            .any(|event| matches!(event, CommandEvent::ViewModelValue { .. }))
    );
    queue.advance_state_machine(machine, 0.0, 0);
    queue.pointer_down(machine, pointer_at(250.0, 250.0), 0);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(events(&log).iter().filter(|event| matches!(event, CommandEvent::ViewModelValue { handle, path, value: CommandValue::Boolean(true), .. } if *handle == view_model && path == "hasReached")).count(), 1);
    queue.pointer_up(machine, pointer_at(250.0, 250.0), 0);
    assert!(server.process_commands());
    queue.process_messages();
    assert_eq!(events(&log).iter().filter(|event| matches!(event, CommandEvent::ViewModelValue { handle, path, .. } if *handle == view_model && path == "hasReached")).count(), 1);
}

#[test]
fn pointer_input_translation() {
    let queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let checks = [
        ((50.0, 50.0), (250.0, 250.0)),
        ((25.0, 25.0), (125.0, 125.0)),
        ((75.0, 75.0), (375.0, 375.0)),
        ((75.0, 25.0), (375.0, 125.0)),
        ((25.0, 75.0), (125.0, 375.0)),
    ];
    for ((x, y), (expected_x, expected_y)) in checks {
        queue.run_once(move |server| {
            let translated = server
                .testing_cursor_position(
                    machine,
                    PointerEvent {
                        fit: Fit::Contain,
                        screen_bounds: nuxie::Vec2D::new(100.0, 100.0),
                        position: nuxie::Vec2D::new(x, y),
                        ..PointerEvent::default()
                    },
                )
                .expect("state machine cursor translation");
            assert!((translated.x - expected_x).abs() < 0.0001);
            assert!((translated.y - expected_y).abs() < 0.0001);
        });
    }
    assert!(server(&queue).process_commands());
}

#[test]
fn global_listener() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener));
    queue.set_global_artboard_listener(Some(&listener));
    queue.set_global_state_machine_listener(Some(&listener));
    queue.set_global_view_model_listener(Some(&listener));
    queue.set_global_image_listener(Some(&listener));
    queue.set_global_audio_listener(Some(&listener));
    queue.set_global_font_listener(Some(&listener));
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 1);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let view_model =
        queue.instantiate_view_model_for_artboard(file, artboard, Some(String::new()), None, 0);
    let image = queue.decode_image(IMAGE_FIXTURE.to_vec(), None, 0);
    let audio = queue.decode_audio(AUDIO_FIXTURE.to_vec(), None, 0);
    let font = queue.decode_font(FONT_FIXTURE.to_vec(), None, 0);
    queue.request_artboard_names(file, 2);
    queue.request_view_model_names(file, 3);
    queue.request_view_model_instance_names(file, "Test All", 4);
    queue.request_view_model_properties(file, "Test All", 5);
    queue.request_view_model_enums(file, 6);
    queue.request_state_machine_names(artboard, 11);
    queue.request_default_view_model(artboard, file, 20);
    queue.request_view_model_value(view_model, "Test Bool", CommandDataType::Boolean, 13);
    queue.request_view_model_list_size(view_model, "Test List", 14);
    queue.request_view_model_name(view_model, 18);
    queue.request_view_model_instance_name(view_model, 19);
    for _ in 0..3 {
        queue.advance_state_machine(machine, 1.0, 16);
    }
    queue.delete_font(font, 10);
    queue.delete_state_machine(machine, 17);
    queue.delete_artboard(artboard, 12);
    queue.delete_view_model(view_model, 15);
    queue.delete_image(image, 8);
    queue.delete_file(file, 7);
    queue.delete_audio(audio, 9);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    for predicate in [
        |event: &CommandEvent| matches!(event, CommandEvent::FileLoaded { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ArtboardInstantiated { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::StateMachineInstantiated { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelInstantiated { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ImageDecoded { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::AudioDecoded { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::FontDecoded { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::StateMachineSettled { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ArtboardsListed { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::StateMachinesListed { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelsListed { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelInstancesListed { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelPropertiesListed { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelEnumsListed { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelValue { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelListSize { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelName { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelInstanceName { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::FontDeleted { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::StateMachineDeleted { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ArtboardDeleted { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ViewModelDeleted { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::ImageDeleted { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::FileDeleted { .. }),
        |event: &CommandEvent| matches!(event, CommandEvent::AudioDeleted { .. }),
    ] {
        assert!(captured.iter().any(predicate));
    }
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::StateMachineSettled { handle, request_id: 16 }
            if *handle == machine
    )));
}

#[test]
fn sync_pointer_events() {
    let queue = CommandQueue::new();
    let file = queue.load_file(POINTER_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_artboard_named(file, "art-1", None, 0);
    let machine = queue.instantiate_state_machine_named(artboard, "sm-1", None, 0);
    queue.advance_state_machine(machine, 0.0, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    for index in 0..20 {
        let position = 50.0 + index as f32 * 10.0;
        let event = pointer_at(position, position);
        queue.pointer_down(machine, event, 0);
        queue.pointer_up(machine, event, 0);
        queue.pointer_move(machine, event, 0);
        queue.advance_state_machine(machine, 0.1, 0);
        server.pointer_down_synchronized(machine, event);
        server.pointer_up_synchronized(machine, event);
        server.pointer_move_synchronized(machine, event);
        assert!(server.process_commands());
    }
    queue.delete_state_machine(machine, 0);
    assert!(server.process_commands());
    server.pointer_down_synchronized(machine, PointerEvent::default());
    server.pointer_up_synchronized(machine, PointerEvent::default());
    server.pointer_move_synchronized(machine, PointerEvent::default());
}

#[test]
fn request_view_model_instance_list_clear() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let view_model = queue.instantiate_view_model_for_artboard(
        file,
        artboard,
        Some(String::new()),
        Some(&listener),
        0,
    );
    let nested = queue.instantiate_blank_view_model_named(file, "ListViewModel", None, 0);
    queue.insert_view_model_list(view_model, "Test List", nested, None, 0);
    queue.request_view_model_list_size(view_model, "Test List", 1);
    queue.request_view_model_list_clear(view_model, "Test List", 0x42);
    queue.request_view_model_list_size(view_model, "Test List", 2);
    let bad = queue.instantiate_blank_view_model_named(file, "Bad", None, 0);
    queue.request_view_model_list_clear(bad, "Test List", 3);
    queue.request_view_model_list_clear(view_model, "Bad", 4);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ViewModelListSize { request_id: 1, size, .. } if *size >= 1)));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::ViewModelListCleared { handle, request_id: 0x42, path } if *handle == view_model && path == "Test List")));
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::ViewModelListSize {
            request_id: 2,
            size: 0,
            ..
        }
    )));
}

#[test]
fn dependency_lifetime_management() {
    let queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 1);
    let artboards = [
        queue.instantiate_default_artboard(file, None, 0),
        queue.instantiate_default_artboard(file, None, 0),
        queue.instantiate_default_artboard(file, None, 0),
    ];
    let machines = [
        queue.instantiate_default_state_machine(artboards[0], None, 0),
        queue.instantiate_default_state_machine(artboards[0], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
        queue.instantiate_default_state_machine(artboards[1], None, 0),
    ];
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(
        artboards
            .iter()
            .all(|handle| server.artboard(*handle).is_some())
    );
    assert!(
        machines
            .iter()
            .all(|handle| server.state_machine(*handle).is_some())
    );
    queue.delete_artboard(artboards[0], 0);
    assert!(server.process_commands());
    assert!(server.artboard(artboards[0]).is_none());
    assert!(server.artboard(artboards[1]).is_some());
    assert!(server.artboard(artboards[2]).is_some());
    assert!(server.state_machine(machines[0]).is_none());
    assert!(server.state_machine(machines[1]).is_none());
    assert!(
        machines[2..]
            .iter()
            .all(|handle| server.state_machine(*handle).is_some())
    );
    queue.delete_state_machine(machines[2], 0);
    assert!(server.process_commands());
    assert!(server.state_machine(machines[2]).is_none());
    assert!(
        machines[3..]
            .iter()
            .all(|handle| server.state_machine(*handle).is_some())
    );
}

fn listed_assets(bytes: &[u8], request_id: u64) -> Vec<nuxie::command_queue::FileAssetData> {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(bytes.to_vec(), Some(&listener), 0);
    queue.request_file_assets(file, request_id);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    events(&log)
        .into_iter()
        .find_map(|event| match event {
            CommandEvent::FileAssetsListed {
                handle,
                request_id: actual,
                assets,
            } if handle == file && actual == request_id => Some(assets),
            _ => None,
        })
        .expect("file assets callback")
}

#[test]
fn file_assets_listed_image_asset() {
    let assets = listed_assets(HOSTED_IMAGE_FIXTURE, 42);
    assert_eq!(assets.len(), 1);
    let asset = &assets[0];
    assert_eq!(asset.name, "one.png");
    assert_eq!(asset.asset_id, 45008);
    assert_eq!(asset.cdn_uuid, "edcb1816-8405-4983-acd2-16db48d85df4");
    assert_eq!(asset.cdn_base_url, "https://public.uat.rive.app/cdn/uuid");
    assert_eq!(asset.file_extension, "png");
    assert_eq!(asset.type_id, 105);
}

#[test]
fn file_assets_listed_font_asset() {
    let assets = listed_assets(HOSTED_FONT_FIXTURE, 43);
    assert_eq!(assets.len(), 1);
    let asset = &assets[0];
    assert_eq!(asset.name, "Inter");
    assert_eq!(asset.asset_id, 43276);
    assert_eq!(asset.cdn_base_url, "https://public.uat.rive.app/cdn/uuid");
    assert_eq!(asset.file_extension, "ttf");
    assert_eq!(asset.type_id, 141);
}

#[test]
fn file_assets_listed_type_ids_match_runtime() {
    assert_eq!(
        nuxie_schema::definition_by_name("ImageAsset")
            .unwrap()
            .type_key
            .int,
        105
    );
    assert_eq!(
        nuxie_schema::definition_by_name("FontAsset")
            .unwrap()
            .type_key
            .int,
        141
    );
    assert_eq!(
        nuxie_schema::definition_by_name("AudioAsset")
            .unwrap()
            .type_key
            .int,
        406
    );
}

#[test]
fn file_assets_listed_empty_file() {
    assert!(listed_assets(ARTBOARD_FIXTURE, 44).is_empty());
}

#[test]
fn file_assets_listed_invalid_handle() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    queue.set_global_file_listener(Some(&listener));
    let bad = queue.load_file(vec![0; 1024], None, 0);
    queue.request_file_assets(bad, 45);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(!captured.iter().any(
        |event| matches!(event, CommandEvent::FileAssetsListed { handle, .. } if *handle == bad)
    ));
    assert!(captured.iter().any(|event| matches!(event, CommandEvent::FileError { handle, request_id: 45, .. } if *handle == bad)));
}

#[test]
fn file_assets_listed_all_assets_returned() {
    let assets = listed_assets(DATA_BIND_FIXTURE, 46);
    let file = nuxie::File::import(DATA_BIND_FIXTURE).expect("fixture import");
    assert_eq!(assets.len(), file.assets().len());
}

#[test]
fn global_view_model_names_listed() {
    let queue = CommandQueue::new();
    let (listener, log) = event_log();
    let file = queue.load_file(GLOBAL_VARIABLES_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_global_view_model_names(file, 7);
    let bad = queue.load_file(vec![0; 1024 * 1024], Some(&listener), 0);
    queue.request_global_view_model_names(bad, 8);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let captured = events(&log);
    assert!(captured.iter().any(|event| matches!(
        event,
        CommandEvent::GlobalViewModelsListed { handle, request_id: 7, names }
            if *handle == file
                && !names.is_empty()
                && names.iter().all(|name| !name.is_empty())
    )));
    assert!(!captured.iter().any(|event| matches!(event, CommandEvent::GlobalViewModelsListed { handle, .. } if *handle == bad)));
}

#[test]
fn set_bind_get_global_view_model_instance() {
    let queue = CommandQueue::new();
    let (file_listener, file_log) = event_log();
    let file = queue.load_file(GLOBAL_VARIABLES_FIXTURE.to_vec(), Some(&file_listener), 0);
    queue.request_global_view_model_names(file, 1);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.process_messages();
    let global_name = events(&file_log)
        .into_iter()
        .find_map(|event| match event {
            CommandEvent::GlobalViewModelsListed { names, .. } => names.into_iter().next(),
            _ => None,
        })
        .expect("global view model name");
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_default_state_machine(artboard, None, 0);
    let global = queue.instantiate_view_model_named(file, &global_name, "", None, 0);
    queue.set_global_view_model_instance(machine, &global_name, global, 0);
    queue.bind_state_machine(machine, 0);
    let (ok_listener, ok_log) = event_log();
    let fetched = queue.global_view_model_instance(machine, &global_name, Some(&ok_listener), 0);
    queue.request_view_model_name(fetched, 2);
    let (error_listener, error_log) = event_log();
    let missing =
        queue.global_view_model_instance(machine, "not-a-global", Some(&error_listener), 3);
    assert!(server.process_commands());
    queue.process_messages();
    assert!(events(&ok_log).iter().any(|event| matches!(event, CommandEvent::ViewModelName { handle, request_id: 2, name } if *handle == fetched && name == &global_name)));
    assert!(events(&error_log).iter().any(|event| matches!(event, CommandEvent::ViewModelError { handle, request_id: 3, .. } if *handle == missing)));
}

#[test]
fn command_server_get_handle_for_instance() {
    let queue = CommandQueue::new();
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), None, 0);
    let handle = queue.instantiate_view_model_named(file, "Test All", "", None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    let instance = server.view_model(handle).expect("view model instance");
    assert_eq!(server.handle_for_view_model(instance), Some(handle));
}

#[test]
fn run_once_preserves_command_order() {
    let queue = CommandQueue::new();
    let order = Arc::new(Mutex::new(Vec::new()));
    for value in 0..3 {
        let order = Arc::clone(&order);
        queue.run_once(move |_| {
            order
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(value);
        });
    }
    assert!(server(&queue).process_commands());
    assert_eq!(
        *order
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()),
        vec![0, 1, 2]
    );
}

#[test]
fn draw_is_coalesced_by_key_within_one_poll() {
    let queue = CommandQueue::new();
    let key = queue.create_draw_key();
    let count = Arc::new(AtomicUsize::new(0));
    for value in [1, 10] {
        let count = Arc::clone(&count);
        queue.draw(key, move |_, _| {
            count.fetch_add(value, Ordering::SeqCst);
        });
    }
    assert!(server(&queue).process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 10);
}

#[test]
fn disconnect_stops_a_non_waiting_server() {
    let queue = CommandQueue::new();
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(!server.was_disconnected());
    queue.disconnect();
    assert!(!server.process_commands());
    assert!(server.was_disconnected());
}

#[test]
fn draw_happens_once_per_poll() {
    let queue = CommandQueue::new();
    let key = queue.create_draw_key();
    let count = Arc::new(AtomicUsize::new(0));
    let mut server = server(&queue);
    for expected in 1..=2 {
        let count_on_draw = Arc::clone(&count);
        queue.draw(key, move |_, _| {
            count_on_draw.fetch_add(1, Ordering::SeqCst);
        });
        assert!(server.process_commands());
        assert_eq!(count.load(Ordering::SeqCst), expected);
    }
}

#[test]
fn cancel_draw_only_cancels_matching_pending_key() {
    let queue = CommandQueue::new();
    let cancelled = queue.create_draw_key();
    let retained = queue.create_draw_key();
    let count = Arc::new(AtomicUsize::new(0));
    let first = Arc::clone(&count);
    queue.draw(cancelled, move |_, _| {
        first.fetch_add(1, Ordering::SeqCst);
    });
    let second = Arc::clone(&count);
    queue.draw(retained, move |_, _| {
        second.fetch_add(10, Ordering::SeqCst);
    });
    queue.cancel_draw(cancelled);
    assert!(server(&queue).process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 10);

    let count_after_cancel = Arc::clone(&count);
    queue.draw(cancelled, move |_, _| {
        count_after_cancel.fetch_add(1, Ordering::SeqCst);
    });
    assert!(server(&queue).process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 11);
}

#[test]
fn command_poll_is_entry_bounded() {
    let queue = CommandQueue::new();
    let count = Arc::new(AtomicUsize::new(0));
    let nested_queue = queue.clone();
    let count_for_outer = Arc::clone(&count);
    queue.run_once(move |_| {
        count_for_outer.fetch_add(1, Ordering::SeqCst);
        let count_for_inner = Arc::clone(&count_for_outer);
        nested_queue.run_once(move |_| {
            count_for_inner.fetch_add(1, Ordering::SeqCst);
        });
    });
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 1);
    assert!(server.process_commands());
    assert_eq!(count.load(Ordering::SeqCst), 2);
}

#[test]
fn message_poll_is_entry_bounded() {
    let queue = CommandQueue::new();
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_on_message = Arc::clone(&events);
    let queue_on_message = queue.clone();
    let listener: Listener = Arc::new(move |event: &CommandEvent| {
        events_on_message
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(event.clone());
        queue_on_message.load_file(Vec::new(), None, 2);
    });
    queue.set_global_file_listener(Some(&listener));
    queue.load_file(Vec::new(), None, 1);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(queue.process_messages(), 1);
    assert_eq!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .len(),
        1
    );
    assert!(server.process_commands());
    assert_eq!(queue.process_messages(), 1);
}

#[test]
fn listener_lifetime_is_weak() {
    let queue = CommandQueue::new();
    let (listener, events) = event_log();
    queue.load_file(Vec::new(), Some(&listener), 4);
    drop(listener);
    assert!(server(&queue).process_commands());
    assert_eq!(queue.process_messages(), 1);
    assert!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .is_empty()
    );
}

#[test]
fn wait_commands_wakes_for_work_and_disconnects() {
    let queue = CommandQueue::new();
    let worker_queue = queue.clone();
    let worker = thread::spawn(move || {
        let mut server = server(&worker_queue);
        server.serve_until_disconnect();
        server.was_disconnected()
    });
    let completed = Arc::new(AtomicUsize::new(0));
    let completed_on_server = Arc::clone(&completed);
    queue.run_once(move |_| {
        completed_on_server.store(1, Ordering::SeqCst);
    });
    queue.disconnect();
    assert!(worker.join().expect("command server thread panicked"));
    assert_eq!(completed.load(Ordering::SeqCst), 1);
}

#[test]
fn deleting_file_releases_dependent_artboards_and_machines() {
    let queue = CommandQueue::new();
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    queue.delete_file(file, 0);
    assert!(server.process_commands());
    assert!(server.file(file).is_none());
    assert!(server.artboard(artboard).is_none());
}
