//! Regression coverage for the two runtime branches changed in upstream
//! 1fac60f8. These are local regressions, not upstream-authored test cases.
#![cfg(feature = "tools")]

use nuxie_binary::{BinaryWriter, SUPPORTED_MAJOR_VERSION, SUPPORTED_MINOR_VERSION};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::animation::{
    state_machine::StateMachine, state_machine_layer::StateMachineLayer,
};
use nuxie_runtime::{File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle};

fn fixture(entry: bool, any: bool, exit: bool) -> Vec<u8> {
    let mut owners = vec!["Backboard", "Artboard", "StateMachine", "StateMachineLayer"];
    if any {
        owners.push("AnyState");
    }
    if entry {
        owners.push("EntryState");
    }
    if exit {
        owners.push("ExitState");
    }
    // Write raw records so intentionally missing Entry reaches File::import,
    // rather than being rejected by the binary descriptor's own validator.
    let mut bytes = Vec::new();
    {
        let mut writer = BinaryWriter::new(&mut bytes);
        writer.write_bytes(b"RIVE");
        writer.write_var_uint64(SUPPORTED_MAJOR_VERSION);
        writer.write_var_uint64(SUPPORTED_MINOR_VERSION);
        writer.write_var_uint64(0); // file ID
        writer.write_var_uint64(0); // no extra property field IDs
        for owner in owners {
            let key = nuxie_schema::definition_by_name(owner)
                .unwrap()
                .type_key
                .int;
            writer.write_var_uint64(u64::from(key));
            writer.write_var_uint64(0); // end of this object's properties
        }
    }
    bytes
}

fn import(bytes: &[u8]) -> (Option<RuntimeFileHandle>, ImportResult) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let mut result = ImportResult::Success;
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        Some(&mut result),
        None,
        None,
    );
    (file, result)
}

#[test]
fn entry_layer_imports_instances_and_advances_with_optional_any_and_exit() {
    for any in [false, true] {
        for exit in [false, true] {
            let label = format!("any={any}, exit={exit}");
            let (file, result) = import(&fixture(true, any, exit));
            assert_eq!(result, ImportResult::Success, "{label}");
            let file = file.unwrap_or_else(|| panic!("{label}: import rejected"));
            let artboard = file
                .with_file(|file| file.artboard_default())
                .expect("instance");
            let machine = artboard
                .state_machine_instance_handle(0)
                .expect("state machine instance");
            let source = machine.with_instance(|m| m.state_machine());
            let layer = source
                .with_downcast::<StateMachine, _>(|m| {
                    assert_eq!(m.layer_count(), 1, "{label}");
                    m.layer(0).unwrap()
                })
                .unwrap();
            let entry = layer
                .with_downcast::<StateMachineLayer, _>(|layer| {
                    assert_eq!(layer.any_state().is_some(), any, "{label}");
                    assert_eq!(layer.exit_state().is_some(), exit, "{label}");
                    assert_eq!(
                        layer.state_count(),
                        1 + usize::from(any) + usize::from(exit),
                        "{label}"
                    );
                    layer.entry_state().expect("Entry remains mandatory")
                })
                .unwrap();
            assert_eq!(
                machine.with_instance_mut(|m| m.layer_state(0)),
                Some(entry.clone()),
                "{label}"
            );
            // Exercise initial and settled transition searches: no AnyState
            // instance is valid, and no authored transition leaves Entry.
            for seconds in [0.0, 1.0 / 60.0, 0.0] {
                machine.advance_and_apply(seconds);
                assert_eq!(
                    machine.with_instance_mut(|m| m.layer_state(0)),
                    Some(entry.clone()),
                    "{label}"
                );
            }
        }
    }
}

#[test]
fn missing_entry_is_still_rejected_for_every_any_exit_combination() {
    for any in [false, true] {
        for exit in [false, true] {
            let (file, result) = import(&fixture(false, any, exit));
            assert!(
                file.is_none(),
                "missing Entry accepted: any={any}, exit={exit}"
            );
            assert_eq!(result, ImportResult::Malformed, "any={any}, exit={exit}");
        }
    }
}
