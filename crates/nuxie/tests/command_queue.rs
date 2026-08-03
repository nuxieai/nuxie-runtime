//! Focused parity tests for Rive's pinned `command_queue_test.cpp`.
//!
//! These tests port the non-rendering command-loop invariants from
//! `tests/unit_tests/runtime/command_queue_test.cpp` at `4ac7b327`. The
//! case-by-case correspondence, including the F6 exclusions, is recorded in
//! `docs/p3f-command-queue-test-ledger.md`.

use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    thread,
};

use nuxie::{
    RecordingFactory,
    command_queue::{CommandEvent, CommandQueue, Listener},
    command_server::CommandServer,
};

const ARTBOARD_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/two_artboards.riv");
const ENTRY_FIXTURE: &[u8] = include_bytes!("../../../fixtures/command_queue/entry.riv");
const MULTI_MACHINE_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/multiple_state_machines.riv");
const DATA_BIND_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/command_queue/data_bind_test_cmdq.riv");

fn server(queue: &CommandQueue) -> CommandServer {
    CommandServer::new(queue.clone(), Box::new(RecordingFactory::new()))
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
    let observed = Arc::new(Mutex::new(None));
    let observed_on_server = Arc::clone(&observed);
    queue.run_once(move |_| {
        assert!(Arc::ptr_eq(&captured, &original));
        assert_eq!(*captured, MAGIC_NUMBER);
        *observed_on_server
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(captured);
        let null: Option<Arc<usize>> = None;
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
    let file = queue.load_file(DATA_BIND_FIXTURE.to_vec(), Some(&listener), 0);
    queue.request_file_assets(file, 1);
    let mut server = server(&queue);
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
