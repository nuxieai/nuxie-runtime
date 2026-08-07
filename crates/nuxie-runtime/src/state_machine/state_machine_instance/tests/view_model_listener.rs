use super::*;
use crate::properties::property_key_for_name;
use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
use nuxie_graph::GraphFile;

#[test]
fn one_listener_occurrence_binds_every_authored_view_model_source() {
    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record("StateMachine", Vec::new()),
        record(
            "StateMachineListener",
            vec![uint_property("StateMachineListener", "targetId", 0)],
        ),
        record(
            "ListenerInputTypeViewModel",
            vec![
                uint_property(
                    "ListenerInputTypeViewModel",
                    "listenerTypeValue",
                    RuntimeListenerType::ViewModel as u64,
                ),
                bytes_property("ListenerInputTypeViewModel", "viewModelPathIds", vec![0, 0]),
            ],
        ),
        record(
            "ListenerInputTypeViewModel",
            vec![
                uint_property(
                    "ListenerInputTypeViewModel",
                    "listenerTypeValue",
                    RuntimeListenerType::ViewModel as u64,
                ),
                bytes_property("ListenerInputTypeViewModel", "viewModelPathIds", vec![0, 1]),
            ],
        ),
    ])
    .expect("view-model listener records import");
    let graph = GraphFile::from_runtime_file(&file).expect("listener graph builds");
    let authored = file.artboard_state_machine_graphs(0);
    let action_catalog = RuntimeFileStateMachineActionCatalog::new(&file);
    let action_owners = action_catalog
        .arena(authored[0].object.id)
        .expect("state-machine action owners");
    let definition = runtime_state_machine_listener(
        &file,
        graph.artboards.first().expect("artboard graph"),
        &authored[0].inputs,
        &[],
        &authored[0].listeners[0],
        &action_owners,
    )
    .expect("listener definition");
    let definitions = Arc::new(vec![definition]);
    let mut occurrence = RuntimeViewModelListenerInstance::new(Arc::clone(&definitions), 0)
        .expect("view-model listener occurrence");

    assert!(std::ptr::eq(occurrence.listener(), &definitions[0]));
    assert_eq!(occurrence.property_bindings.len(), 2);
    assert!(matches!(
        occurrence.property_bindings[0].source,
        RuntimeViewModelListenerSource::Input(0)
    ));
    assert!(matches!(
        occurrence.property_bindings[1].source,
        RuntimeViewModelListenerSource::Input(1)
    ));

    let queue = RuntimeCellNotificationQueue::default();
    let first = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
    let second = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(0.0));
    relink_view_model_listener_cell(
        &mut occurrence.property_bindings[0],
        Some(first.clone()),
        &queue,
        0,
    );
    relink_view_model_listener_cell(
        &mut occurrence.property_bindings[1],
        Some(second.clone()),
        &queue,
        0,
    );

    assert!(first.set_value(RuntimeViewModelCellValue::Number(1.0)));
    assert!(second.set_value(RuntimeViewModelCellValue::Number(2.0)));
    let mut reporting = Vec::new();
    queue.swap_into(&mut reporting);

    // C++ has one ListenerViewModel parent and one property binding per
    // authored input. Either binding reports that same parent, preserving
    // mutation/FIFO order (`state_machine_instance.cpp:1324-1375,
    // 1377-1382,1454-1489,3021-3025`).
    assert_eq!(reporting, [0, 0]);
}

fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int,
        properties,
    }
}

fn uint_property(type_name: &str, name: &str, value: u64) -> FixtureProperty {
    FixtureProperty {
        key: property_key_for_name(type_name, name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
        value: FixtureValue::Uint(value),
    }
}

fn bytes_property(type_name: &str, name: &str, value: Vec<u8>) -> FixtureProperty {
    FixtureProperty {
        key: property_key_for_name(type_name, name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
        value: FixtureValue::Bytes(value),
    }
}
