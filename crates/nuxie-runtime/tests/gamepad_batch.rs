use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::{ArtboardInstance, GAMEPAD_BATCH_WIRE_VERSION, StateMachineInstance};
use nuxie_runtime::{GAMEPAD_BATCH_MAX_AXES, GAMEPAD_BATCH_MAX_BUTTONS};
use std::path::{Path, PathBuf};

fn fixture_path() -> PathBuf {
    let runtime_dir = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime_dir).join("tests/unit_tests/assets/gamepad_test.riv")
}

fn ready_gamepad_fixture() -> (ArtboardInstance, StateMachineInstance) {
    let bytes = std::fs::read(fixture_path()).expect("gamepad fixture bytes");
    let runtime = read_runtime_file(&bytes).expect("gamepad fixture imports");
    let graph = GraphFile::from_runtime_file(&runtime).expect("gamepad fixture graph");
    let artboard = graph.artboards.first().expect("gamepad fixture artboard");
    let mut instance =
        ArtboardInstance::from_graph_with_artboards(&runtime, artboard, &graph.artboards)
            .expect("gamepad fixture instantiates");
    let mut machine = instance
        .state_machine_instance(0)
        .expect("gamepad fixture state machine");
    machine.bind_default_view_model_context();
    let _ = instance.advance_state_machine_instance(&mut machine, 0.0);
    (instance, machine)
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

    fn connected(&mut self, device_id: i32, button_count: u8, axis_count: u8) {
        self.connected_with_mapping(device_id, button_count, axis_count, 0);
    }

    fn connected_with_mapping(
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
        self.update_many(device_id, &[(kind, index, value)]);
    }

    fn update_many(&mut self, device_id: i32, changes: &[(u8, u8, f32)]) {
        self.bytes.push(1);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
        self.bytes
            .push(u8::try_from(changes.len()).expect("test change count fits u8"));
        for (kind, index, value) in changes {
            self.bytes.extend_from_slice(&[*kind, *index]);
            self.bytes.extend_from_slice(&value.to_le_bytes());
        }
    }

    fn disconnected(&mut self, device_id: i32) {
        self.bytes.push(2);
        self.bytes.extend_from_slice(&device_id.to_le_bytes());
    }
}

#[test]
fn accepts_a_single_connected_record_with_the_pinned_wire_size() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut wire = WireBuilder::new();
    wire.connected(0, 17, 4);

    assert_eq!(wire.bytes.len(), 4 + 1 + 4 + 4 + 17 * 4 + 4 * 4);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));
}

#[test]
fn tracks_multiple_device_ids_independently() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut connected = WireBuilder::new();
    connected.connected(1, 17, 4);
    connected.connected(7, 17, 4);
    connected.connected(42, 17, 4);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &connected.bytes));

    let mut updates = WireBuilder::new();
    updates.update(1, 0, 0, 1.0);
    updates.update(7, 1, 2, -0.5);
    updates.update(42, 0, 4, 1.0);
    updates.update(42, 1, 0, 0.75);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &updates.bytes));
}

#[test]
fn rejects_an_update_for_an_unknown_device_after_retaining_prior_records() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut wire = WireBuilder::new();
    wire.connected(3, 17, 4);
    wire.update(99, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));

    let mut known_update = WireBuilder::new();
    known_update.update(3, 0, 0, 1.0);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &known_update.bytes));
}

#[test]
fn disconnects_only_the_target_device() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut wire = WireBuilder::new();
    for device_id in [10, 20, 30] {
        wire.connected(device_id, 17, 4);
    }
    wire.update(10, 0, 0, 1.0);
    wire.update(20, 1, 1, 0.25);
    wire.update(30, 0, 2, 1.0);
    wire.disconnected(20);
    wire.update(10, 1, 0, -1.0);
    wire.update(30, 0, 3, 0.0);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));

    let mut disconnected_update = WireBuilder::new();
    disconnected_update.update(20, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &disconnected_update.bytes));
}

#[test]
fn allows_reconnecting_the_same_device_id() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut first = WireBuilder::new();
    first.connected(5, 17, 4);
    first.update(5, 0, 0, 1.0);
    first.disconnected(5);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &first.bytes));

    let mut stray = WireBuilder::new();
    stray.update(5, 0, 0, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &stray.bytes));

    let mut reconnect = WireBuilder::new();
    reconnect.connected(5, 17, 4);
    reconnect.update(5, 1, 0, 0.5);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &reconnect.bytes));
}

#[test]
fn tolerates_disconnect_of_an_unknown_device_id() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut wire = WireBuilder::new();
    wire.disconnected(1234);

    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &wire.bytes));
}

#[test]
fn validates_versions_record_shapes_caps_and_indices() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &[]));
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &1_u32.to_le_bytes()));

    let mut truncated = GAMEPAD_BATCH_WIRE_VERSION.to_le_bytes().to_vec();
    truncated.extend_from_slice(&[0, 1, 0]);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &truncated));

    let mut unknown = GAMEPAD_BATCH_WIRE_VERSION.to_le_bytes().to_vec();
    unknown.push(99);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &unknown));

    let mut oversized = WireBuilder::new();
    oversized.connected(1, GAMEPAD_BATCH_MAX_BUTTONS + 1, 0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &oversized.bytes));

    let mut connect = WireBuilder::new();
    connect.connected_with_mapping(2, 0, 0, 255);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &connect.bytes));

    let mut valid_edges = WireBuilder::new();
    valid_edges.update(2, 0, GAMEPAD_BATCH_MAX_BUTTONS - 1, f32::NAN);
    valid_edges.update(2, 255, GAMEPAD_BATCH_MAX_AXES - 1, -0.0);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &valid_edges.bytes));

    let mut invalid_button = WireBuilder::new();
    invalid_button.update(2, 0, GAMEPAD_BATCH_MAX_BUTTONS, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &invalid_button.bytes));

    let mut invalid_axis = WireBuilder::new();
    invalid_axis.update(2, 1, GAMEPAD_BATCH_MAX_AXES, 1.0);
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &invalid_axis.bytes));
}

#[test]
fn a_later_invalid_change_keeps_the_device_snapshot_live() {
    let (mut artboard, mut machine) = ready_gamepad_fixture();
    let mut connect = WireBuilder::new();
    connect.connected(9, 0, 0);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &connect.bytes));

    let mut partially_applied = WireBuilder::new();
    partially_applied.update_many(
        9,
        &[
            (0, GAMEPAD_BATCH_MAX_BUTTONS - 1, 1.0),
            (0, GAMEPAD_BATCH_MAX_BUTTONS, 1.0),
        ],
    );
    assert!(!machine.submit_gamepads_from_buffer(&mut artboard, &partially_applied.bytes));

    let mut later = WireBuilder::new();
    later.update(9, 1, 0, 0.25);
    assert!(machine.submit_gamepads_from_buffer(&mut artboard, &later.bytes));
}
