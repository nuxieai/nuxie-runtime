use super::{InstanceObjectArena, InstanceSlot};
use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
use nuxie_schema::definition_by_name;

fn node_arena() -> InstanceObjectArena {
    let backboard = definition_by_name("Backboard").expect("Backboard definition");
    let artboard = definition_by_name("Artboard").expect("Artboard definition");
    let node = definition_by_name("Node").expect("Node definition");
    let parent = crate::properties::property_key_for_name("Node", "parentId")
        .expect("Node.parentId property");
    let file = RuntimeFile::from_fixture_records(vec![
        FixtureRecord {
            type_key: backboard.type_key.int,
            properties: Vec::new(),
        },
        FixtureRecord {
            type_key: artboard.type_key.int,
            properties: Vec::new(),
        },
        FixtureRecord {
            type_key: node.type_key.int,
            properties: vec![FixtureProperty {
                key: parent,
                value: FixtureValue::Uint(0),
            }],
        },
    ])
    .expect("construct the exact default Node record");
    InstanceObjectArena::from_slots(
        &file,
        &[
            InstanceSlot {
                local_id: 0,
                source_global_id: 1,
                type_name: Some("Artboard"),
                name: None,
            },
            InstanceSlot {
                local_id: 1,
                source_global_id: 2,
                type_name: Some("Node"),
                name: None,
            },
        ],
    )
}

#[test]
fn wave_c3_node_001_instances_default_x() {
    let node = node_arena();
    assert_eq!(node.double_property_by_name(1, "x"), Some(0.0));
}

#[test]
fn wave_c3_node_002_x_setter_returns_x_value() {
    let mut node = node_arena();
    assert_eq!(node.double_property_by_name(1, "x"), Some(0.0));
    assert!(node.set_double_property_by_name(1, "x", 2.0));
    assert_eq!(node.double_property_by_name(1, "x"), Some(2.0));
}
