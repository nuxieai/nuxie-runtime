//! Exact executable ports of the six wire-contract cases in pinned
//! `tests/unit_tests/runtime/gamepad_test.cpp`.

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, GAMEPAD_BATCH_WIRE_VERSION, StateMachineInstance};
use std::path::{Path, PathBuf};

fn fixture_path() -> PathBuf {
    let runtime = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime).join("tests/unit_tests/assets/gamepad_test.riv")
}

fn open_ready_state_machine() -> (ArtboardInstance, StateMachineInstance) {
    let bytes = std::fs::read(fixture_path()).expect("gamepad fixture bytes");
    let runtime = read_runtime_file(&bytes).expect("gamepad fixture imports");
    let graphs = GraphFile::from_runtime_file(&runtime).expect("gamepad fixture graph");
    let graph = graphs.artboards.first().expect("gamepad fixture artboard");
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .expect("gamepad fixture instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("gamepad fixture state machine");
    machine.bind_default_view_model_context();
    let _ = artboard.advance_state_machine_instance(&mut machine, 0.0);
    (artboard, machine)
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
fn wave_b4_gamepad_case_001_accepts_single_connected_record() {
    let (mut artboard, mut machine) = open_ready_state_machine();
    let mut wire = WireBuilder::new();
    wire.connected_with_shape(0, 17, 4, 0);
    assert_eq!(wire.bytes.len(), 4 + 1 + 4 + 4 + 17 * 4 + 4 * 4);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));
}

#[test]
fn wave_b4_gamepad_case_002_tracks_multiple_device_ids_independently() {
    let (mut artboard, mut machine) = open_ready_state_machine();
    let mut connected = WireBuilder::new();
    connected.connected(1);
    connected.connected(7);
    connected.connected(42);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &connected.bytes));

    let mut updates = WireBuilder::new();
    updates.update(1, 0, 0, 1.0);
    updates.update(7, 1, 2, -0.5);
    updates.update(42, 0, 4, 1.0);
    updates.update(42, 1, 0, 0.75);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &updates.bytes));
}

#[test]
fn wave_b4_gamepad_case_003_rejects_unknown_device_update() {
    let (mut artboard, mut machine) = open_ready_state_machine();
    let mut wire = WireBuilder::new();
    wire.connected(3);
    wire.update(99, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));
}

#[test]
fn wave_b4_gamepad_case_004_disconnects_only_selected_device() {
    let (mut artboard, mut machine) = open_ready_state_machine();
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
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));

    let mut after_disconnect = WireBuilder::new();
    after_disconnect.update(20, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &after_disconnect.bytes));
}

#[test]
fn wave_b4_gamepad_case_005_reconnects_same_device_id() {
    let (mut artboard, mut machine) = open_ready_state_machine();
    let mut first = WireBuilder::new();
    first.connected(5);
    first.update(5, 0, 0, 1.0);
    first.disconnected(5);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &first.bytes));

    let mut stray = WireBuilder::new();
    stray.update(5, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &stray.bytes));

    let mut reconnect = WireBuilder::new();
    reconnect.connected(5);
    reconnect.update(5, 1, 0, 0.5);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &reconnect.bytes));
}

#[test]
fn wave_b4_gamepad_case_006_unknown_disconnect_is_noop() {
    let (mut artboard, mut machine) = open_ready_state_machine();
    let mut wire = WireBuilder::new();
    wire.disconnected(1234);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));
}
