//! All nine synchronized focus/input cases added by upstream 794f432a.
use super::*;
use nuxie::command_queue::FocusTraversalResult;
use nuxie::runtime::{
    input::{
        focus_manager::Direction,
        focus_node::{EdgeBehavior, FocusNode},
        focusable::{Focusable, Key, KeyModifiers},
    },
    semantic::semantic_snapshot::Bounds,
};
use std::rc::Rc;

struct Records {
    handled: bool,
    keys: Vec<(Key, KeyModifiers, bool, bool)>,
}
impl Default for Records {
    fn default() -> Self {
        Self {
            handled: true,
            keys: Vec::new(),
        }
    }
}
struct Recipient(Arc<Mutex<Records>>);
impl Focusable for Recipient {
    fn key_input(
        &mut self,
        key: Key,
        modifiers: KeyModifiers,
        pressed: bool,
        repeat: bool,
    ) -> bool {
        let mut records = self.0.lock().unwrap();
        records.keys.push((key, modifiers, pressed, repeat));
        records.handled
    }
    fn text_input(&mut self, _: &str) -> bool {
        self.0.lock().unwrap().handled
    }
    fn focused(&mut self) {}
    fn blurred(&mut self) {}
    fn accepts_keyboard_input(&self) -> bool {
        true
    }
}

struct Fixture {
    queue: CommandQueue,
    machine: StateMachineHandle,
    worker: Option<std::thread::JoinHandle<()>>,
}
impl Fixture {
    fn new() -> Self {
        let mut queue = CommandQueue::new();
        let server_queue = queue.clone();
        let worker = std::thread::spawn(move || {
            let mut server = server(&server_queue);
            server.serve_until_disconnect();
        });
        let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0, None);
        let artboard = queue.instantiate_default_artboard(file, None, 0);
        let machine = queue.instantiate_state_machine_named(artboard, "one".into(), None, 0);
        Self {
            queue,
            machine,
            worker: Some(worker),
        }
    }
    fn add_scope(&mut self, edge: EdgeBehavior, records: Arc<Mutex<Records>>) {
        let handle = self.machine;
        self.queue.run_once(Box::new(move |server| {
            server
                .with_state_machine_instance_mut(handle, |instance| {
                    instance
                        .focus_manager()
                        .expect("focus manager")
                        .with_focus_manager_mut(|manager| {
                            let scope = FocusNode::new(None);
                            scope.borrow_mut().set_edge_behavior(edge);
                            scope.borrow_mut().set_can_focus(false);
                            let child =
                                FocusNode::new(Some(Rc::new(RefCell::new(Recipient(records)))));
                            manager.add_child(None, scope.clone(), None);
                            manager.add_child(Some(scope), child, None);
                        });
                })
                .expect("state machine");
        }));
    }
    fn traverse(&mut self, previous: bool) -> FocusTraversalResult {
        if previous {
            self.queue
                .focus_previous_with_result_synchronized(self.machine)
        } else {
            self.queue.focus_next_with_result_synchronized(self.machine)
        }
    }
    fn key(&mut self, key: Key, modifiers: KeyModifiers, pressed: bool, repeat: bool) -> bool {
        self.queue
            .key_input_synchronized(self.machine, key, modifiers, pressed, repeat)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.queue.disconnect();
        self.worker
            .take()
            .unwrap()
            .join()
            .expect("command server thread");
    }
}
fn assert_result(result: FocusTraversalResult, moved: bool, has_focus: bool) {
    assert_eq!(result.moved, moved);
    assert_eq!(result.focus_state.has_focus, has_focus);
    assert_eq!(result.focus_state.expects_keyboard_input, has_focus);
}

#[test]
fn returns_traversal_and_resulting_focus_state() {
    for previous in [false, true] {
        let mut fx = Fixture::new();
        fx.add_scope(
            EdgeBehavior::ParentScope,
            Arc::new(Mutex::new(Records::default())),
        );
        let entry = fx.traverse(previous);
        let result = fx.traverse(previous);
        assert_result(entry, true, true);
        assert_result(result, false, false);
    }
}

#[test]
fn reports_retained_focus_when_traversal_does_not_move() {
    for previous in [false, true] {
        let mut fx = Fixture::new();
        fx.add_scope(EdgeBehavior::Stop, Arc::new(Mutex::new(Records::default())));
        let entry = fx.traverse(previous);
        let result = fx.traverse(previous);
        assert_result(entry, true, true);
        assert_result(result, false, true);
    }
}

#[test]
fn queued_clear_precedes_traversal() {
    for previous in [false, true] {
        let mut fx = Fixture::new();
        fx.add_scope(EdgeBehavior::Stop, Arc::new(Mutex::new(Records::default())));
        let entry = fx.traverse(previous);
        fx.queue.clear_focus(fx.machine, 0);
        let result = fx.traverse(previous);
        assert_result(entry, true, true);
        assert_result(result, true, true);
    }
}

#[test]
fn invalid_handles_return_empty_results() {
    for deleted in [false, true] {
        let mut fx = Fixture::new();
        let handle = if deleted {
            fx.queue.delete_state_machine(fx.machine, 0);
            fx.machine
        } else {
            StateMachineHandle::default()
        };
        assert_result(
            fx.queue.focus_next_with_result_synchronized(handle),
            false,
            false,
        );
        assert_result(
            fx.queue.focus_previous_with_result_synchronized(handle),
            false,
            false,
        );
    }
}

#[test]
fn key_input_forwards_arguments_and_consumption() {
    let records = Arc::new(Mutex::new(Records::default()));
    let mut fx = Fixture::new();
    fx.add_scope(EdgeBehavior::Stop, records.clone());
    assert!(fx.queue.focus_next_synchronized(fx.machine));
    let modifiers = KeyModifiers::CTRL | KeyModifiers::SHIFT;
    assert!(fx.key(Key::LEFT, modifiers, true, true));
    records.lock().unwrap().handled = false;
    assert!(!fx.key(Key::LEFT, KeyModifiers::NONE, false, false));
    {
        let records = records.lock().unwrap();
        assert_eq!(records.keys.len(), 2);
        assert_eq!(records.keys[0], (Key::LEFT, modifiers, true, true));
        assert_eq!(
            records.keys[1],
            (Key::LEFT, KeyModifiers::NONE, false, false)
        );
    }
    records.lock().unwrap().handled = true;
    assert!(fx.key(Key::RIGHT, KeyModifiers::ALT, true, false));
    let records = records.lock().unwrap();
    assert_eq!(records.keys.len(), 3);
    assert_eq!(
        records.keys[2],
        (Key::RIGHT, KeyModifiers::ALT, true, false)
    );
}

#[test]
fn key_input_observes_queued_clear() {
    let records = Arc::new(Mutex::new(Records::default()));
    let mut fx = Fixture::new();
    fx.add_scope(EdgeBehavior::Stop, records.clone());
    assert!(fx.queue.focus_next_synchronized(fx.machine));
    assert!(fx.key(Key::RIGHT, KeyModifiers::NONE, true, false));
    fx.queue.clear_focus(fx.machine, 0);
    assert!(!fx.key(Key::RIGHT, KeyModifiers::NONE, true, false));
    assert_eq!(records.lock().unwrap().keys.len(), 1);
}

fn bounds(x: f32, y: f32) -> Bounds {
    Bounds {
        min_x: x,
        min_y: y,
        max_x: x + 10.0,
        max_y: y + 10.0,
    }
}

#[test]
fn directional_focus_forwards_direction_and_movement() {
    for direction in [
        Direction::Left,
        Direction::Right,
        Direction::Up,
        Direction::Down,
    ] {
        let mut fx = Fixture::new();
        let handle = fx.machine;
        fx.queue.run_once(Box::new(move |server| {
            server
                .with_state_machine_instance_mut(handle, |instance| {
                    instance
                        .focus_manager()
                        .unwrap()
                        .with_focus_manager_mut(|manager| {
                            let origin = FocusNode::new(None);
                            let target = FocusNode::new(None);
                            origin.borrow_mut().world_bounds = bounds(0.0, 0.0);
                            target.borrow_mut().world_bounds = match direction {
                                Direction::Left => bounds(-20.0, 0.0),
                                Direction::Right => bounds(20.0, 0.0),
                                Direction::Up => bounds(0.0, -20.0),
                                Direction::Down => bounds(0.0, 20.0),
                            };
                            manager.add_child(None, origin.clone(), None);
                            manager.add_child(None, target, None);
                            manager.set_focus(origin);
                        });
                })
                .expect("state machine");
        }));
        assert!(
            fx.queue
                .focus_in_direction_synchronized(fx.machine, direction)
        );
        assert!(
            !fx.queue
                .focus_in_direction_synchronized(fx.machine, direction)
        );
    }
}

#[test]
fn keyboard_and_directional_input_reject_missing_targets() {
    for without_nodes in [false, true] {
        let mut fx = Fixture::new();
        let handle = if without_nodes {
            fx.machine
        } else {
            StateMachineHandle::default()
        };
        assert!(!fx.queue.key_input_synchronized(
            handle,
            Key::RIGHT,
            KeyModifiers::NONE,
            true,
            false
        ));
        assert!(
            !fx.queue
                .focus_in_direction_synchronized(handle, Direction::Right)
        );
    }
}

#[test]
fn input_observes_queued_state_machine_deletion() {
    for directional in [false, true] {
        let records = Arc::new(Mutex::new(Records::default()));
        let mut fx = Fixture::new();
        let handle = fx.machine;
        let captured = records.clone();
        fx.queue.run_once(Box::new(move |server| {
            server
                .with_state_machine_instance_mut(handle, |instance| {
                    instance
                        .focus_manager()
                        .unwrap()
                        .with_focus_manager_mut(|manager| {
                            let recipient = Rc::new(RefCell::new(Recipient(captured)));
                            let left = FocusNode::new(Some(recipient.clone()));
                            let right = FocusNode::new(Some(recipient));
                            left.borrow_mut().world_bounds = bounds(0.0, 0.0);
                            right.borrow_mut().world_bounds = bounds(20.0, 0.0);
                            manager.add_child(None, left.clone(), None);
                            manager.add_child(None, right, None);
                            manager.set_focus(left);
                        });
                })
                .expect("state machine");
        }));
        if directional {
            assert!(
                fx.queue
                    .focus_in_direction_synchronized(handle, Direction::Right)
            );
            assert!(
                fx.queue
                    .focus_in_direction_synchronized(handle, Direction::Left)
            );
            fx.queue.delete_state_machine(handle, 0);
            assert!(
                !fx.queue
                    .focus_in_direction_synchronized(handle, Direction::Right)
            );
        } else {
            assert!(fx.key(Key::RIGHT, KeyModifiers::NONE, true, false));
            fx.queue.delete_state_machine(handle, 0);
            assert!(!fx.key(Key::RIGHT, KeyModifiers::NONE, true, false));
            assert_eq!(records.lock().unwrap().keys.len(), 1);
        }
    }
}
