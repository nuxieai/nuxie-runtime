//! Focus cases added by upstream 503eab633367d00ad3627770e7898d22379cc2e2.
use super::*;
use nuxie::command_queue::FocusState;
use nuxie::runtime::input::{
    focus_node::{EdgeBehavior, FocusNode},
    focusable::{Focusable, Key, KeyModifiers},
};
use std::rc::Rc;

#[derive(Default)]
struct FocusResults {
    handle: StateMachineHandle,
    availability: Vec<(u64, bool)>,
    states: Vec<(u64, FocusState)>,
    errors: Vec<u64>,
}

struct FocusCommandListener {
    base: ListenerBase<StateMachineHandle>,
    results: Arc<Mutex<FocusResults>>,
}

impl StateMachineListener for FocusCommandListener {
    fn listener_base(&mut self) -> &mut ListenerBase<StateMachineHandle> {
        &mut self.base
    }
    fn on_state_machine_error(&mut self, handle: StateMachineHandle, id: u64, _: String) {
        let mut results = self.results.lock().unwrap();
        assert_eq!(handle, results.handle);
        results.errors.push(id);
    }
    fn on_has_focus_nodes_received(&mut self, handle: StateMachineHandle, id: u64, value: bool) {
        let mut results = self.results.lock().unwrap();
        assert_eq!(handle, results.handle);
        results.availability.push((id, value));
    }
    fn on_focus_state_received(&mut self, handle: StateMachineHandle, id: u64, state: FocusState) {
        let mut results = self.results.lock().unwrap();
        assert_eq!(handle, results.handle);
        results.states.push((id, state));
    }
}

fn listener() -> (StateMachineListenerHandle, Arc<Mutex<FocusResults>>) {
    let results = Arc::new(Mutex::new(FocusResults::default()));
    (
        ListenerHandle::new(Box::new(FocusCommandListener {
            base: ListenerBase::new(),
            results: results.clone(),
        })),
        results,
    )
}

struct FocusCommandFixture {
    queue: CommandQueue,
    server: Box<CommandServer>,
    artboard: ArtboardHandle,
    machine: StateMachineHandle,
}

impl FocusCommandFixture {
    fn new(listener: Option<&StateMachineListenerHandle>) -> Self {
        let mut queue = CommandQueue::new();
        let server = server(&queue);
        let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0, None);
        let artboard = queue.instantiate_default_artboard(file, None, 0);
        let machine = queue.instantiate_state_machine_named(artboard, "one".into(), listener, 0);
        let mut fixture = Self {
            queue,
            server,
            artboard,
            machine,
        };
        fixture.pump();
        fixture
    }

    fn pump(&mut self) {
        self.server.process_commands();
        self.queue.process_messages();
    }
}

// Store only addresses for equality, never dereference them. Rc focus nodes stay
// on the server thread while Send command callbacks carry the observed identity.
fn check_focus(queue: &mut CommandQueue, machine: StateMachineHandle, expected: Option<usize>) {
    queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance.focus_manager().with_focus_manager(|manager| {
                    assert_eq!(
                        manager
                            .primary_focus()
                            .map(|node| Rc::as_ptr(&node) as usize),
                        expected
                    );
                });
            })
            .expect("state machine");
    }));
}

#[test]
fn focus_commands_mirror_state_machine_instance_traversal_and_queries() {
    let (listener, results) = listener();
    let mut fx = FocusCommandFixture::new(Some(&listener));
    results.lock().unwrap().handle = fx.machine;
    fx.queue.request_has_focus_nodes(fx.machine, 0xF1);
    fx.pump();
    assert_eq!(results.lock().unwrap().availability, [(0xF1, false)]);

    let nodes = Arc::new(Mutex::new([0usize; 2]));
    let captured = nodes.clone();
    let machine = fx.machine;
    fx.queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance.focus_manager().with_focus_manager_mut(|manager| {
                    let first = FocusNode::new(None);
                    let second = FocusNode::new(None);
                    *captured.lock().unwrap() =
                        [Rc::as_ptr(&first) as usize, Rc::as_ptr(&second) as usize];
                    manager.add_child(None, first, None);
                    manager.add_child(None, second, None);
                });
            })
            .expect("state machine");
    }));
    fx.pump();
    let [first, second] = *nodes.lock().unwrap();
    fx.queue.request_has_focus_nodes(machine, 0xF2);
    fx.queue.focus_next(machine, 0xF3);
    check_focus(&mut fx.queue, machine, Some(first));
    fx.queue.focus_next(machine, 0xF4);
    check_focus(&mut fx.queue, machine, Some(second));
    fx.queue.focus_next(machine, 0xF5);
    check_focus(&mut fx.queue, machine, None);
    fx.queue.focus_previous(machine, 0xF6);
    check_focus(&mut fx.queue, machine, Some(second));
    fx.pump();
    assert_eq!(
        results.lock().unwrap().availability,
        [(0xF1, false), (0xF2, true)]
    );
    fx.queue.clear_focus(machine, 0xF7);
    check_focus(&mut fx.queue, machine, None);
    fx.pump();
    assert!(results.lock().unwrap().errors.is_empty());
}

#[test]
fn synchronized_focus_traversal_mirrors_state_machine_instance() {
    fn assert_send<T: Send>() {}
    assert_send::<CommandQueue>();

    let mut queue = CommandQueue::new();
    let server_queue = queue.clone();
    let server_thread = std::thread::spawn(move || {
        let mut server = server(&server_queue);
        server.serve_until_disconnect();
    });
    let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0, None);
    let artboard = queue.instantiate_default_artboard(file, None, 0);
    let machine = queue.instantiate_state_machine_named(artboard, "one".into(), None, 0);
    queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance.focus_manager().with_focus_manager_mut(|manager| {
                    manager.add_child(None, FocusNode::new(None), None);
                    manager.add_child(None, FocusNode::new(None), None);
                });
            })
            .expect("state machine");
    }));

    assert!(queue.focus_next_synchronized(machine));
    assert!(queue.focus_next_synchronized(machine));
    assert!(!queue.focus_next_synchronized(machine));
    assert!(queue.focus_previous_synchronized(machine));
    assert!(!queue.focus_next_synchronized(StateMachineHandle::default()));
    assert!(!queue.focus_previous_synchronized(StateMachineHandle::default()));

    queue.disconnect();
    server_thread.join().expect("command server thread");
}

struct KeyboardAcceptingFocusable;
impl Focusable for KeyboardAcceptingFocusable {
    fn key_input(&mut self, _: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
        false
    }
    fn text_input(&mut self, _: &str) -> bool {
        false
    }
    fn focused(&mut self) {}
    fn blurred(&mut self) {}
    fn accepts_keyboard_input(&self) -> bool {
        true
    }
}

#[test]
fn focus_state_query_reports_internal_focus_changes() {
    let (listener, results) = listener();
    let mut fx = FocusCommandFixture::new(Some(&listener));
    let machine = fx.machine;
    results.lock().unwrap().handle = machine;
    fx.queue.request_focus_state(machine, 0xF8);
    fx.queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance.focus_manager().with_focus_manager_mut(|manager| {
                    let node =
                        FocusNode::new(Some(Rc::new(RefCell::new(KeyboardAcceptingFocusable))));
                    manager.add_child(None, node.clone(), None);
                    manager.set_focus(node);
                });
            })
            .expect("state machine");
    }));
    fx.queue.request_focus_state(machine, 0xF9);
    fx.queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| instance.clear_focus())
            .expect("state machine");
    }));
    fx.queue.request_focus_state(machine, 0xFB);
    fx.pump();
    let results = results.lock().unwrap();
    assert_eq!(results.states.len(), 3);
    for ((id, state), (expected_id, has_focus, expects_keyboard_input)) in
        results.states.iter().zip([
            (0xF8, false, false),
            (0xF9, true, true),
            (0xFB, false, false),
        ])
    {
        assert_eq!(*id, expected_id);
        assert_eq!(state.has_focus, has_focus);
        assert_eq!(state.expects_keyboard_input, expects_keyboard_input);
    }
    assert!(results.errors.is_empty());
}

#[test]
fn focus_command_preserves_current_stop_traversal_result() {
    let (listener, results) = listener();
    let mut fx = FocusCommandFixture::new(Some(&listener));
    let machine = fx.machine;
    results.lock().unwrap().handle = machine;
    let stopped = Arc::new(Mutex::new(0usize));
    let captured = stopped.clone();
    fx.queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance.focus_manager().with_focus_manager_mut(|manager| {
                    let scope = FocusNode::new(None);
                    let first = FocusNode::new(None);
                    let second = FocusNode::new(None);
                    scope.borrow_mut().set_edge_behavior(EdgeBehavior::Stop);
                    *captured.lock().unwrap() = Rc::as_ptr(&second) as usize;
                    manager.add_child(None, scope.clone(), None);
                    manager.add_child(Some(scope.clone()), first, None);
                    manager.add_child(Some(scope), second.clone(), None);
                    manager.set_focus(second);
                });
            })
            .expect("state machine");
    }));
    fx.pump();
    fx.queue.focus_next(machine, 0xFA);
    check_focus(&mut fx.queue, machine, Some(*stopped.lock().unwrap()));
    fx.pump();
    assert!(results.lock().unwrap().errors.is_empty());
}

#[test]
fn focus_commands_report_invalid_state_machine_handles() {
    let mut fx = FocusCommandFixture::new(None);
    let (listener, results) = listener();
    let invalid = fx.queue.instantiate_state_machine_named(
        fx.artboard,
        "does not exist".into(),
        Some(&listener),
        0,
    );
    results.lock().unwrap().handle = invalid;
    fx.queue.focus_next(invalid, 0xFB);
    fx.queue.focus_previous(invalid, 0xFC);
    fx.queue.request_has_focus_nodes(invalid, 0xFD);
    fx.queue.clear_focus(invalid, 0xFE);
    fx.queue.request_focus_state(invalid, 0xFF);
    fx.pump();
    let results = results.lock().unwrap();
    assert_eq!(results.errors, [0xFB, 0xFC, 0xFD, 0xFE, 0xFF]);
    assert!(results.availability.is_empty());
    assert!(results.states.is_empty());
}
