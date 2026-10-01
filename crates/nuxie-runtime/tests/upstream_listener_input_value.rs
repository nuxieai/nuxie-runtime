//! All seven tests from runtime/listener_input_value_test.cpp at abf676e7.
//! The authored fixture supplies focus, listener actions, and converters; no
//! synthetic dispatch or replacement converter bypasses the runtime path.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    CoreHandle, File, GAMEPAD_BATCH_WIRE_VERSION, RuntimeArtboardInstanceHandle,
    RuntimeFactoryHandle, RuntimeFileHandle, RuntimeStateMachineInstanceHandle,
    source::{
        input::{
            focus_manager::RuntimeFocusManagerHandle,
            focusable::{Key, KeyModifiers},
            gamepad_batch::GamepadRecordType,
            gamepad_snapshot::GamepadInputChangeKind,
        },
        math::vec2d::Vec2D,
        viewmodel::{
            viewmodel_instance::ViewModelInstance,
            viewmodel_instance_boolean::ViewModelInstanceBoolean,
            viewmodel_instance_number::ViewModelInstanceNumber,
            viewmodel_instance_string::ViewModelInstanceString,
        },
    },
};
use std::path::PathBuf;

struct GamepadWire {
    buf: Vec<u8>,
}
impl GamepadWire {
    fn new() -> Self {
        Self {
            buf: GAMEPAD_BATCH_WIRE_VERSION.to_le_bytes().to_vec(),
        }
    }
    fn connected(&mut self, device_id: i32, buttons: u8, axes: u8) {
        self.buf.push(GamepadRecordType::Connected as u8);
        self.buf.extend_from_slice(&device_id.to_le_bytes());
        self.buf.extend_from_slice(&[0, buttons, axes, 0]);
        for _ in 0..buttons + axes {
            self.buf.extend_from_slice(&0.0_f32.to_le_bytes());
        }
    }
    fn update(&mut self, device_id: i32, kind: GamepadInputChangeKind, index: u8, value: f32) {
        self.buf.push(GamepadRecordType::Update as u8);
        self.buf.extend_from_slice(&device_id.to_le_bytes());
        self.buf.extend_from_slice(&[1, kind as u8, index]);
        self.buf.extend_from_slice(&value.to_le_bytes());
    }
}

struct InputScene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    vmi: CoreHandle,
}
impl InputScene {
    fn new() -> Self {
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        let bytes = std::fs::read(
            PathBuf::from(root).join("tests/unit_tests/assets/listener_input_values.riv"),
        )
        .expect("pinned listener_input_values.riv");
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
        let file = File::import(&bytes, factory, None, None, None).expect("input scene imports");
        let artboard = file
            .with_file(File::artboard_default)
            .expect("default artboard");
        let machine = artboard.state_machine_at(0).expect("state machine 0");
        let vmi = file
            .with_file(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default view model instance");
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
        let scene = Self {
            _file: file,
            artboard,
            machine,
            vmi,
        };
        scene.advance();
        // Entry-state focus is required for keyboard, text, and gamepad input.
        assert!(
            scene
                .focus_manager()
                .with_focus_manager(|manager| manager.primary_focus().is_some())
        );
        scene
    }
    fn focus_manager(&self) -> RuntimeFocusManagerHandle {
        self.artboard
            .with_artboard(|artboard| artboard.focus_manager())
            .expect("artboard focus manager")
    }
    fn advance(&self) {
        self.machine.advance_and_apply(0.016);
    }
    fn property(&self, name: &str) -> CoreHandle {
        self.vmi
            .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named(name))
            .expect("view model instance")
            .unwrap_or_else(|| panic!("property {name}"))
    }
    fn number(&self, name: &str) -> f32 {
        self.property(name)
            .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
            .expect("number property")
    }
    fn boolean(&self, name: &str) -> bool {
        self.property(name)
            .with_downcast::<ViewModelInstanceBoolean, _>(ViewModelInstanceBoolean::value)
            .expect("boolean property")
    }
    fn string(&self, name: &str) -> String {
        self.property(name)
            .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value)
            .expect("string property")
    }
    fn submit(&self, wire: &GamepadWire) {
        assert!(self.machine.submit_gamepads_from_buffer(&wire.buf));
        self.advance();
    }
}

#[test]
fn pointer_listeners_write_the_pointer_position() {
    let scene = InputScene::new();
    scene
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(120.0, 310.0), 0, nuxie_runtime::source::pointer_button::PointerButton::Primary));
    scene.advance();
    assert_eq!(scene.number("px"), 120.0);
    assert_eq!(scene.number("py"), 310.0);
    // Unavailable keyPressed skips the action, not writing its constant true.
    assert!(!scene.boolean("skipped"));
}

#[test]
fn gamepad_axes_read_the_full_snapshot() {
    let scene = InputScene::new();
    let mut wire = GamepadWire::new();
    wire.connected(0, 17, 4);
    wire.update(0, GamepadInputChangeKind::Axis, 0, 0.5);
    scene.submit(&wire);
    assert_eq!(scene.number("lx"), 0.5);
    assert_eq!(scene.number("ly"), 0.0);
    assert_eq!(scene.number("mapped"), 75.0);
    assert_eq!(scene.number("changed"), 0.5);
    let mut next = GamepadWire::new();
    next.update(0, GamepadInputChangeKind::Axis, 1, -0.25);
    scene.submit(&next);
    assert_eq!(scene.number("lx"), 0.5);
    assert_eq!(scene.number("ly"), -0.25);
    assert_eq!(scene.number("mapped"), 75.0);
    assert_eq!(scene.number("changed"), -0.25);
}

#[test]
fn gamepad_buttons_write_pressed_state_and_value() {
    let scene = InputScene::new();
    let mut press = GamepadWire::new();
    press.connected(0, 17, 4);
    press.update(0, GamepadInputChangeKind::Button, 0, 1.0);
    scene.submit(&press);
    assert!(scene.boolean("southPressed"));
    assert_eq!(scene.number("southValue"), 1.0);
    let mut release = GamepadWire::new();
    release.update(0, GamepadInputChangeKind::Button, 0, 0.0);
    scene.submit(&release);
    assert!(!scene.boolean("southPressed"));
    assert_eq!(scene.number("southValue"), 0.0);
}

#[test]
fn input_values_of_another_type_go_through_the_converter() {
    let scene = InputScene::new();
    scene
        .machine
        .with_instance_mut(|machine| machine.pointer_down(Vec2D::new(120.4, 310.0), 0, nuxie_runtime::source::pointer_button::PointerButton::Primary));
    scene.advance();
    assert_eq!(scene.string("pxText"), "120");
    let mut press = GamepadWire::new();
    press.connected(0, 17, 4);
    press.update(0, GamepadInputChangeKind::Button, 0, 1.0);
    scene.submit(&press);
    assert_eq!(scene.number("southNumber"), 1.0);
    assert_eq!(scene.number("southRaw"), 5.0);
    let mut release = GamepadWire::new();
    release.update(0, GamepadInputChangeKind::Button, 0, 0.0);
    scene.submit(&release);
    assert_eq!(scene.number("southNumber"), 0.0);
    assert_eq!(scene.number("southRaw"), 5.0);
}

#[test]
fn keyboard_listeners_write_whether_the_key_is_down() {
    let scene = InputScene::new();
    let manager = scene.focus_manager();
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::A, KeyModifiers::NONE, true, false)
    });
    scene.advance();
    assert!(scene.boolean("keyDown"));
    manager.with_focus_manager_mut(|manager| {
        manager.key_input(Key::A, KeyModifiers::NONE, false, false)
    });
    scene.advance();
    assert!(!scene.boolean("keyDown"));
}

#[test]
fn text_input_listeners_write_the_committed_text() {
    let scene = InputScene::new();
    scene
        .focus_manager()
        .with_focus_manager_mut(|manager| manager.text_input("hi"));
    scene.advance();
    assert_eq!(scene.string("typed"), "hi");
}

#[test]
fn focus_listeners_write_the_focus_state() {
    let scene = InputScene::new();
    // Entry focus events are delivered on the following advance.
    scene.advance();
    assert!(scene.boolean("focused"));
    scene
        .focus_manager()
        .with_focus_manager_mut(|manager| manager.clear_focus());
    scene.advance();
    assert!(!scene.boolean("focused"));
    scene
        .focus_manager()
        .with_focus_manager_mut(|manager| manager.focus_next());
    scene.advance();
    assert!(scene.boolean("focused"));
}
