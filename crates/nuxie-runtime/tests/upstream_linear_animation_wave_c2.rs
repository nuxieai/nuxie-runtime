//! Fixture-backed exact owner tests for the non-synthetic definition cases.

use std::path::PathBuf;

use nuxie_binary::{read_runtime_file, FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
use nuxie_graph::GraphFile;
use nuxie_runtime::ArtboardInstance;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn key(type_name: &str, property_name: &str) -> u16 {
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

fn uint(type_name: &str, property_name: &str, value: u64) -> FixtureProperty {
    FixtureProperty {
        key: key(type_name, property_name),
        value: FixtureValue::Uint(value),
    }
}

#[test]
#[ignore = "expected-red: RuntimeLinearAnimation definitions are immutable, so the pinned quantize(false) action cannot reach the owner"]
fn wave_c2_linear_definition_003_quantize_goes_to_whole_frames() {
    let runtime = read_runtime_file(&pinned_fixture("quantize_test.riv")).expect("fixture imports");
    let graphs = GraphFile::from_runtime_file(&runtime).expect("fixture graphs");
    let graph = &graphs.artboards[0];
    assert!(graph.animations[0].quantize);
    let shape = graph
        .components
        .iter()
        .find(|component| component.type_name == "Shape")
        .expect("one Shape")
        .local_id;
    assert_eq!(
        graph
            .components
            .iter()
            .filter(|component| component.type_name == "Shape")
            .count(),
        1
    );
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .expect("artboard instantiates");
    assert!(artboard.apply_linear_animation(0, 0.0, 1.0));
    assert_eq!(
        artboard
            .component_world_transform_with_scroll(shape)
            .expect("Shape transform")
            .tx(),
        0.0
    );
    assert!(artboard.apply_linear_animation(0, 0.5, 1.0));
    assert_eq!(
        artboard
            .component_world_transform_with_scroll(shape)
            .expect("Shape transform")
            .tx(),
        160.0
    );

    // This is the first unavailable action in the exact C++ stream: mutate
    // the imported definition, reapply 0.5, and observe 200 instead of 160.
    assert!(
        !graph.animations[0].quantize,
        "missing mutable LinearAnimation::quantize(false) owner"
    );
}

#[test]
fn wave_c2_linear_definition_005_missing_keyed_object_does_not_stop_initialization() {
    let runtime = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record("Node", vec![uint("Node", "parentId", 0)]),
        record("LinearAnimation", Vec::new()),
        record("KeyedObject", vec![uint("KeyedObject", "objectId", 99)]),
        record("KeyedObject", vec![uint("KeyedObject", "objectId", 1)]),
    ])
    .expect("fixture imports while retaining the valid sibling after MissingObject");
    let graphs = GraphFile::from_runtime_file(&runtime).expect("fixture graphs");
    let animation = &graphs.artboards[0].animations[0];
    assert_eq!(animation.keyed_objects.len(), 1);
    assert_eq!(animation.keyed_objects[0].object_id, 1);
}

#[test]
fn wave_c2_linear_definition_006_looping_timeline_events_load_and_report() {
    let runtime =
        read_runtime_file(&pinned_fixture("looping_timeline_events.riv")).expect("fixture imports");
    let graphs = GraphFile::from_runtime_file(&runtime).expect("fixture graphs");
    let graph = &graphs.artboards[0];
    assert_eq!(graph.animations.len(), 1);
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .expect("artboard instantiates");
    let mut animation = artboard
        .linear_animation_instance(0)
        .expect("animation instance");
    let mut reported = Vec::new();
    for (seconds, expected_time, expected_count) in [
        (0.1, 0.1, 1),
        (0.32, 0.42, 2),
        (0.3, 0.72, 2),
        (0.28, 0.0, 3),
        (1.01, 0.01, 7),
    ] {
        artboard.advance_linear_animation_instance_with_events(
            &mut animation,
            seconds,
            &mut reported,
        );
        assert!(
            (animation.time() - expected_time).abs()
                <= f32::EPSILON * 100.0 * expected_time.abs().max(1.0)
        );
        assert_eq!(reported.len(), expected_count);
    }
}
