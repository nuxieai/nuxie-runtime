//! Exact executable ports of the remaining five wire-contract cases in pinned
//! `tests/unit_tests/runtime/gamepad_test.cpp`.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, GAMEPAD_BATCH_WIRE_VERSION, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeStateMachineInstanceHandle,
};
use std::path::{Path, PathBuf};

fn fixture_path() -> PathBuf {
    let runtime = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime).join("tests/unit_tests/assets/gamepad_test.riv")
}

fn open_ready_state_machine() -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let bytes = std::fs::read(fixture_path()).expect("gamepad fixture bytes");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, factory, None, None, None).expect("gamepad fixture imports");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("gamepad fixture artboard");
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("gamepad fixture state machine");
    let view_model = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    });
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model));
    machine.advance_and_apply(0.0);
    (file, artboard, machine)
}

struct WireBuilder {
    bytes: Vec<u8>,
}

impl WireBuilder {
    fn new() -> Self {
        Self {
            bytes: GAMEPAD_BATCH_WIRE_VERSION.to_le_bytes().to_vec(),
        }
    }

    fn connected(&mut self, device_id: i32) {
        self.connected_with_shape(device_id, 17, 4, 0);
    }

    fn connected_with_shape(
        &mut self,
        device_id: i32,
        button_count: u8,
        axis_count: u8,
        mapping: u8,
    ) {
        self.bytes.push(0);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
        self.bytes
            .extend_from_slice(&[mapping, button_count, axis_count, 0]);
        self.bytes
            .resize(self.bytes.len() + usize::from(button_count) * 4, 0);
        self.bytes
            .resize(self.bytes.len() + usize::from(axis_count) * 4, 0);
    }

    fn update(&mut self, device_id: i32, kind: u8, index: u8, value: f32) {
        self.bytes.push(1);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
        self.bytes.push(1);
        self.bytes.extend_from_slice(&[kind, index]);
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn disconnected(&mut self, device_id: i32) {
        self.bytes.push(2);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
    }
}

#[test]
fn wave_b4_gamepad_case_002_tracks_multiple_device_ids_independently() {
    let (_file, _artboard, machine) = open_ready_state_machine();
    let mut connected = WireBuilder::new();
    connected.connected(1);
    connected.connected(7);
    connected.connected(42);
    assert!(machine.submit_gamepads_from_buffer(&connected.bytes));

    let mut updates = WireBuilder::new();
    updates.update(1, 0, 0, 1.0);
    updates.update(7, 1, 2, -0.5);
    updates.update(42, 0, 4, 1.0);
    updates.update(42, 1, 0, 0.75);
    assert!(machine.submit_gamepads_from_buffer(&updates.bytes));
}

#[test]
fn wave_b4_gamepad_case_003_rejects_unknown_device_update() {
    let (_file, _artboard, machine) = open_ready_state_machine();
    let mut wire = WireBuilder::new();
    wire.connected(3);
    wire.update(99, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&wire.bytes));
}

#[test]
fn wave_b4_gamepad_case_004_disconnects_only_selected_device() {
    let (_file, _artboard, machine) = open_ready_state_machine();
    let mut wire = WireBuilder::new();
    wire.connected(10);
    wire.connected(20);
    wire.connected(30);
    wire.update(10, 0, 0, 1.0);
    wire.update(20, 1, 1, 0.25);
    wire.update(30, 0, 2, 1.0);
    wire.disconnected(20);
    wire.update(10, 1, 0, -1.0);
    wire.update(30, 0, 3, 0.0);
    assert!(machine.submit_gamepads_from_buffer(&wire.bytes));

    let mut after_disconnect = WireBuilder::new();
    after_disconnect.update(20, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&after_disconnect.bytes));
}

#[test]
fn wave_b4_gamepad_case_005_reconnects_same_device_id() {
    let (_file, _artboard, machine) = open_ready_state_machine();
    let mut first = WireBuilder::new();
    first.connected(5);
    first.update(5, 0, 0, 1.0);
    first.disconnected(5);
    assert!(machine.submit_gamepads_from_buffer(&first.bytes));

    let mut stray = WireBuilder::new();
    stray.update(5, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&stray.bytes));

    let mut reconnect = WireBuilder::new();
    reconnect.connected(5);
    reconnect.update(5, 1, 0, 0.5);
    assert!(machine.submit_gamepads_from_buffer(&reconnect.bytes));
}

#[test]
fn wave_b4_gamepad_case_006_unknown_disconnect_is_noop() {
    let (_file, _artboard, machine) = open_ready_state_machine();
    let mut wire = WireBuilder::new();
    wire.disconnected(1234);
    assert!(machine.submit_gamepads_from_buffer(&wire.bytes));
}
