//! All four keyboard/text input command_queue_test.cpp cases at ead02826.
use super::*;
use nuxie::runtime::input::{
    focus_node::FocusNode,
    focusable::{Focusable, Key, KeyModifiers},
};
use std::rc::Rc;

#[derive(Default)]
struct Results {
    handle: StateMachineHandle,
    keys: Vec<(u64, bool)>,
    texts: Vec<(u64, bool)>,
    errors: Vec<u64>,
}
struct InputListener {
    base: ListenerBase<StateMachineHandle>,
    results: Arc<Mutex<Results>>,
}
impl StateMachineListener for InputListener {
    fn listener_base(&mut self) -> &mut ListenerBase<StateMachineHandle> {
        &mut self.base
    }
    fn on_state_machine_error(&mut self, h: StateMachineHandle, id: u64, _: String) {
        let mut r = self.results.lock().unwrap();
        assert_eq!(h, r.handle);
        r.errors.push(id);
    }
    fn on_key_input_handled(&mut self, h: StateMachineHandle, id: u64, value: bool) {
        let mut r = self.results.lock().unwrap();
        assert_eq!(h, r.handle);
        r.keys.push((id, value));
    }
    fn on_text_input_handled(&mut self, h: StateMachineHandle, id: u64, value: bool) {
        let mut r = self.results.lock().unwrap();
        assert_eq!(h, r.handle);
        r.texts.push((id, value));
    }
}
fn listener() -> (StateMachineListenerHandle, Arc<Mutex<Results>>) {
    let results = Arc::new(Mutex::new(Results::default()));
    (
        ListenerHandle::new(Box::new(InputListener {
            base: ListenerBase::new(),
            results: results.clone(),
        })),
        results,
    )
}
struct Fixture {
    queue: CommandQueue,
    server: Box<CommandServer>,
    machine: StateMachineHandle,
}
impl Fixture {
    fn new(listener: Option<&StateMachineListenerHandle>) -> Self {
        let mut queue = CommandQueue::new();
        let server = server(&queue);
        let file = queue.load_file(MULTI_MACHINE_FIXTURE.to_vec(), None, 0, None);
        let artboard = queue.instantiate_default_artboard(file, None, 0);
        let machine = queue.instantiate_state_machine_named(artboard, "one".into(), listener, 0);
        let mut result = Self {
            queue,
            server,
            machine,
        };
        result.pump();
        result
    }
    fn pump(&mut self) {
        self.server.process_commands();
        self.queue.process_messages();
    }
}
#[derive(Default)]
struct InputRecords {
    handled: bool,
    keys: Vec<(Key, KeyModifiers, bool, bool)>,
    texts: Vec<String>,
}
struct RecordingInputFocusable(Arc<Mutex<InputRecords>>);
impl Focusable for RecordingInputFocusable {
    fn key_input(&mut self, k: Key, m: KeyModifiers, p: bool, r: bool) -> bool {
        let mut s = self.0.lock().unwrap();
        s.keys.push((k, m, p, r));
        s.handled
    }
    fn text_input(&mut self, t: &str) -> bool {
        let mut s = self.0.lock().unwrap();
        s.texts.push(t.into());
        s.handled
    }
    fn focused(&mut self) {}
    fn blurred(&mut self) {}
    fn accepts_keyboard_input(&self) -> bool {
        true
    }
}

#[test]
fn input_commands_report_invalid_and_deleted_handles() {
    for deleted in [false, true] {
        let mut fx = Fixture::new(None);
        let (listener, results) = listener();
        // Opaque Rust handles cannot be forged. Reserve an identity in a
        // separate, unprocessed queue so it has never existed on this server.
        let mut foreign = CommandQueue::new();
        let mut invalid = StateMachineHandle::default();
        for _ in 0..100 {
            invalid = foreign.instantiate_default_state_machine(ArtboardHandle::default(), None, 0);
        }
        let handle = if deleted { fx.machine } else { invalid };
        results.lock().unwrap().handle = handle;
        fx.queue.set_global_state_machine_listener(Some(&listener));
        fx.queue.delete_state_machine(fx.machine, 0);
        fx.queue
            .key_input(handle, Key::A, KeyModifiers::NONE, true, false, 1);
        fx.queue
            .text_input(handle, "discard this payload".into(), 2);
        fx.pump();
        let r = results.lock().unwrap();
        assert_eq!(r.errors, [1, 2]);
        assert!(r.keys.is_empty());
        assert!(r.texts.is_empty());
        fx.queue.set_global_state_machine_listener(None);
    }
}

#[test]
fn input_commands_return_false_without_a_focused_recipient() {
    let (listener, results) = listener();
    let mut fx = Fixture::new(Some(&listener));
    results.lock().unwrap().handle = fx.machine;
    fx.queue
        .key_input(fx.machine, Key::A, KeyModifiers::NONE, true, false, 3);
    fx.queue.text_input(fx.machine, "hello".into(), 4);
    fx.pump();
    let r = results.lock().unwrap();
    assert_eq!(r.keys, [(3, false)]);
    assert_eq!(r.texts, [(4, false)]);
}

#[test]
fn input_commands_preserve_arguments_and_deliver_both_listeners() {
    let (listener, results) = listener();
    let (global, global_results) = self::listener();
    let mut fx = Fixture::new(Some(&listener));
    let machine = fx.machine;
    results.lock().unwrap().handle = machine;
    global_results.lock().unwrap().handle = machine;
    fx.queue.set_global_state_machine_listener(Some(&global));
    let records = Arc::new(Mutex::new(InputRecords {
        handled: true,
        ..Default::default()
    }));
    let captured = records.clone();
    fx.queue.run_once(Box::new(move |server| {
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance
                    .focus_manager()
                    .unwrap()
                    .with_focus_manager_mut(|manager| {
                        let node = FocusNode::new(Some(Rc::new(RefCell::new(
                            RecordingInputFocusable(captured),
                        ))));
                        manager.add_child(None, node.clone(), None);
                        manager.set_focus(node);
                    });
            })
            .unwrap();
    }));
    let modifiers = KeyModifiers::CTRL | KeyModifiers::SHIFT;
    fx.queue
        .key_input(machine, Key::LEFT, modifiers, true, true, 5);
    fx.queue
        .key_input(machine, Key::LEFT, KeyModifiers::NONE, false, false, 6);
    let mut text = String::from("héllo 日本 😀");
    fx.queue.text_input(machine, text.clone(), 7);
    text.clear();
    text.push_str("caller changed its buffer");
    fx.queue.text_input(machine, String::new(), 8);
    fx.pump();
    {
        let r = records.lock().unwrap();
        assert_eq!(
            r.keys,
            [
                (Key::LEFT, modifiers, true, true),
                (Key::LEFT, KeyModifiers::NONE, false, false)
            ]
        );
        assert_eq!(r.texts, ["héllo 日本 😀", ""]);
    }
    {
        let r = results.lock().unwrap();
        assert_eq!(r.keys, [(5, true), (6, true)]);
        assert_eq!(r.texts, [(7, true), (8, true)]);
    }
    {
        let r = global_results.lock().unwrap();
        assert_eq!(r.keys.len(), 2);
        assert!(r.keys.iter().all(|(_, v)| *v));
        assert_eq!(r.texts.len(), 2);
        assert!(r.texts.iter().all(|(_, v)| *v));
    }
    fx.queue.set_global_state_machine_listener(None);
    fx.queue
        .text_input(StateMachineHandle::default(), "discard".into(), 9);
    fx.pump();
    fx.queue.set_global_state_machine_listener(Some(&global));
    records.lock().unwrap().handled = false;
    fx.queue
        .key_input(machine, Key::ENTER, KeyModifiers::ALT, true, false, 10);
    fx.queue.text_input(machine, "after invalid".into(), 11);
    fx.pump();
    {
        let r = global_results.lock().unwrap();
        assert_eq!(r.keys.len(), 3);
        assert_eq!(r.keys.last(), Some(&(10, false)));
        assert_eq!(r.texts.len(), 3);
        assert_eq!(r.texts.last(), Some(&(11, false)));
    }
    {
        let r = results.lock().unwrap();
        assert!(!r.keys.last().unwrap().1);
        assert!(!r.texts.last().unwrap().1);
    }
    assert_eq!(
        records.lock().unwrap().texts.last().unwrap(),
        "after invalid"
    );
    fx.queue.set_global_state_machine_listener(None);
}

#[test]
fn input_commands_edit_a_real_text_field_in_queue_order() {
    use nuxie::runtime::{
        generated::{core_registry::CoreRegistry, text::text_input_base::TextInputBase},
        text::text_input::TextInput,
    };
    struct InputFocusable(nuxie::CoreHandle);
    impl Focusable for InputFocusable {
        fn key_input(&mut self, k: Key, m: KeyModifiers, p: bool, r: bool) -> bool {
            self.0
                .with_downcast_mut::<TextInput, _>(|i| i.key_input(k, m, p, r))
                .unwrap()
        }
        fn text_input(&mut self, t: &str) -> bool {
            self.0
                .with_downcast_mut::<TextInput, _>(|i| i.text_input(t))
                .unwrap()
        }
        fn focused(&mut self) {
            self.0
                .with_downcast_mut::<TextInput, _>(TextInput::focused)
                .unwrap();
        }
        fn blurred(&mut self) {
            self.0
                .with_downcast_mut::<TextInput, _>(TextInput::blurred)
                .unwrap();
        }
        fn accepts_keyboard_input(&self) -> bool {
            true
        }
    }
    let (listener, results) = listener();
    let mut fx = Fixture::new(None);
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let file = fx.queue.load_file(
        std::fs::read(root.join("tests/unit_tests/assets/text_input.riv")).unwrap(),
        None,
        0,
        None,
    );
    let artboard =
        fx.queue
            .instantiate_artboard_named(file, "Text Input - Multiline".into(), None, 0);
    let machine = fx
        .queue
        .instantiate_default_state_machine(artboard, Some(&listener), 0);
    results.lock().unwrap().handle = machine;
    fx.queue.run_once(Box::new(move |server| {
        let input = server
            .with_artboard_instance(artboard, |a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.is_type_of(TextInputBase::TYPE_KEY))
                    .cloned()
            })
            .flatten()
            .unwrap();
        assert!(CoreRegistry::set_string_handle(
            &input,
            TextInputBase::TEXT_PROPERTY_KEY.into(),
            String::new()
        ));
    }));
    // The queue owns the runtime machine handle. Advance between the same
    // source operations without retaining a mutable machine across callbacks.
    fx.queue.advance_state_machine(machine, 0.0, 0);
    fx.queue.run_once(Box::new(move |server| {
        let input = server
            .with_artboard_instance(artboard, |a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.is_type_of(TextInputBase::TYPE_KEY))
                    .cloned()
            })
            .flatten()
            .unwrap();
        server
            .with_state_machine_instance_mut(machine, |instance| {
                instance
                    .focus_manager()
                    .unwrap()
                    .with_focus_manager_mut(|manager| {
                        let node =
                            FocusNode::new(Some(Rc::new(RefCell::new(InputFocusable(input)))));
                        manager.add_child(None, node.clone(), None);
                        manager.set_focus(node);
                    });
            })
            .unwrap();
    }));
    fx.queue.text_input(machine, "hello".into(), 12);
    fx.queue
        .key_input(machine, Key::LEFT, KeyModifiers::SHIFT, true, false, 13);
    fx.queue.text_input(machine, "!".into(), 14);
    fx.queue
        .key_input(machine, Key::BACKSPACE, KeyModifiers::NONE, true, false, 15);
    fx.queue.text_input(machine, "é".into(), 16);
    fx.queue.advance_state_machine(machine, 0.0, 0);
    fx.pump();
    fx.server
        .with_artboard_instance(artboard, |a| {
            let input = a
                .objects()
                .iter()
                .flatten()
                .find(|o| o.is_type_of(TextInputBase::TYPE_KEY))
                .unwrap();
            input
                .with_downcast_mut::<TextInput, _>(|input| {
                    assert_eq!(input.text(), "hellé");
                    assert_eq!(input.raw_text_input().text(), "hellé");
                })
                .unwrap();
        })
        .unwrap();
    let r = results.lock().unwrap();
    assert_eq!(r.keys.len(), 2);
    assert!(r.keys.iter().all(|(_, v)| *v));
    assert_eq!(r.texts.len(), 3);
    assert!(r.texts.iter().all(|(_, v)| *v));
    assert!(r.errors.is_empty());
}
