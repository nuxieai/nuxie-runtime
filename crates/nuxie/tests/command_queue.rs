//! Focused parity tests for Rive's pinned `command_queue_test.cpp`.
//!
//! These tests port the non-rendering command-loop invariants from
//! `tests/unit_tests/runtime/command_queue_test.cpp` at `d788e8ec`. The
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

const ARTBOARD_FIXTURE: &[u8] =
    include_bytes!("../../../fixtures/univ-1275/transform_live_write.riv");

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
    let one = queue.instantiate_default_artboard(file, None, 0);
    let missing = queue.instantiate_artboard_named(file, "missing", None, 0);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.file(file).is_some());
    assert!(server.artboard(one).is_some());
    assert!(server.artboard(missing).is_none());

    queue.delete_artboard(one, 0);
    assert!(server.process_commands());
    assert!(server.artboard(one).is_none());
}

#[test]
fn invalid_state_machine_management_is_typed_and_non_destructive() {
    let queue = CommandQueue::new();
    let (listener, events) = event_log();
    queue.set_global_artboard_listener(Some(&listener));
    let file = queue.load_file(ARTBOARD_FIXTURE.to_vec(), None, 0);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_state_machine_named(artboard, "missing", None, 9);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert!(server.artboard(artboard).is_some());
    assert!(server.state_machine(machine).is_none());
    queue.process_messages();
    assert!(
        events
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .iter()
            .any(|event| matches!(event, CommandEvent::ArtboardError { request_id: 9, .. }))
    );

    queue.delete_state_machine(machine, 0);
    assert!(server.process_commands());
    assert!(server.state_machine(machine).is_none());
}

#[test]
fn invalid_handles_emit_typed_errors() {
    let queue = CommandQueue::new();
    let (listener, events) = event_log();
    queue.set_global_file_listener(Some(&listener));
    let invalid_file = queue.load_file(Vec::new(), None, 10);
    queue.instantiate_default_artboard(invalid_file, None, 11);
    let mut server = server(&queue);
    assert!(server.process_commands());
    assert_eq!(queue.process_messages(), 2);
    let events = events
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CommandEvent::FileError { request_id: 10, .. }))
    );
    assert!(
        events
            .iter()
            .any(|event| matches!(event, CommandEvent::FileError { request_id: 11, .. }))
    );
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
