//! Direct safe-Rust ports of pinned `tests/unit_tests/runtime/node_test.cpp`.

use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
use nuxie_graph::GraphFile;
use nuxie_runtime::ArtboardInstance;

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    definition
        .properties
        .iter()
        .chain(definition.ancestors.iter().flat_map(|ancestor| {
            nuxie_schema::definition_by_name(ancestor)
                .expect("ancestor definition")
                .properties
                .iter()
        }))
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("property {type_name}.{property_name}"))
        .key
        .int
}

fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(type_name)
            .expect("schema definition")
            .type_key
            .int,
        properties,
    }
}

fn uint_property(type_name: &str, name: &str, value: u64) -> FixtureProperty {
    FixtureProperty {
        key: property_key(type_name, name),
        value: FixtureValue::Uint(value),
    }
}

fn node_instance() -> ArtboardInstance {
    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record("Node", vec![uint_property("Node", "parentId", 0)]),
    ])
    .expect("minimal Node fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("minimal Node graph builds");
    ArtboardInstance::from_graph(&file, graphs.artboards.first().expect("artboard"))
        .expect("Node artboard instantiates")
}

#[test]
fn node_instances() {
    let node = node_instance();
    assert_eq!(
        node.double_property(1, property_key("Node", "x")),
        Some(0.0)
    );
}

#[test]
fn node_x_function_returns_x_value() {
    let mut node = node_instance();
    assert_eq!(
        node.double_property(1, property_key("Node", "x")),
        Some(0.0)
    );
    assert!(node.set_double_property(1, property_key("Node", "x"), 2.0));
    assert_eq!(
        node.double_property(1, property_key("Node", "x")),
        Some(2.0)
    );
}
