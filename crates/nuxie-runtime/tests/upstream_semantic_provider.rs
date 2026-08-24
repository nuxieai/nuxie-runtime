//! Direct ports of all six cases in pinned
//! `tests/unit_tests/runtime/semantic_provider_test.cpp`.

use std::path::PathBuf;

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_runtime::{ArtboardInstance, ResolvedSemanticData, SemanticProvider};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

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

fn simpsons() -> (RuntimeFile, GraphFile, ArtboardInstance) {
    let file = read_runtime_file(&pinned_fixture("semantic/simpsons.riv"))
        .expect("semantic/simpsons.riv imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("semantic/simpsons.riv graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut state_machine = artboard.state_machine_instance(0).expect("state machine 0");
    for _ in 0..10 {
        state_machine
            .advance_and_apply(&mut artboard, 0.1)
            .expect("semantic fixture advances");
    }
    (file, graphs, artboard)
}

fn semantic_data_hosts(graph: &ArtboardGraph) -> Vec<(usize, usize)> {
    graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("SemanticData"))
        .filter_map(|data| {
            let host = graph
                .components
                .iter()
                .find(|component| component.local_id == data.local_id)?
                .parent_local?;
            Some((data.local_id, host))
        })
        .collect()
}

#[test]
fn can_infer_semantics_null_returns_false() {
    let (_file, _graphs, artboard) = simpsons();
    assert!(!SemanticProvider::can_infer_semantics(
        &artboard,
        usize::MAX
    ));
}

#[test]
fn resolve_semantic_data_null_returns_default() {
    let (_file, _graphs, artboard) = simpsons();
    let resolved = SemanticProvider::resolve_semantic_data(&artboard, usize::MAX);
    assert_eq!(resolved, ResolvedSemanticData::default());
    assert!(!resolved.has_semantics);
    assert_eq!(resolved.role, 0);
    assert!(resolved.label.is_empty());
}

#[test]
fn semantic_bounds_null_returns_empty() {
    let (_file, _graphs, mut artboard) = simpsons();
    let bounds = SemanticProvider::semantic_bounds(&mut artboard, usize::MAX);
    assert!(bounds.is_empty_or_nan());
}

#[test]
fn can_infer_semantics_text_returns_true_for_any_text_component() {
    let (_file, graphs, artboard) = simpsons();
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut saw_text = false;
    for text in graph
        .local_objects
        .iter()
        .filter(|object| object.type_name == Some("Text"))
    {
        assert!(SemanticProvider::can_infer_semantics(
            &artboard,
            text.local_id
        ));
        saw_text = true;
    }
    assert!(saw_text);
}

#[test]
fn resolve_semantic_data_on_a_node_hosting_a_semantic_data_child_uses_the_explicit_role_and_label()
{
    let (_file, graphs, artboard) = simpsons();
    let graph = graphs.artboards.first().expect("default artboard graph");
    let role_key = property_key("SemanticData", "role");
    let label_key = property_key("SemanticData", "label");
    let mut checked = 0;
    for (data, host) in semantic_data_hosts(graph) {
        let resolved = SemanticProvider::resolve_semantic_data(&artboard, host);
        assert!(resolved.has_semantics);
        assert_eq!(
            u64::from(resolved.role),
            artboard
                .debug_uint_property(data, role_key)
                .expect("SemanticData.role")
        );
        assert_eq!(
            resolved.label.as_bytes(),
            artboard
                .debug_string_property(data, label_key)
                .expect("SemanticData.label")
        );
        checked += 1;
    }
    assert!(checked > 0);
}

#[test]
fn semantic_bounds_on_a_laid_out_drawable_node_produces_non_empty_bounds() {
    let (_file, graphs, mut artboard) = simpsons();
    let graph = graphs.artboards.first().expect("default artboard graph");
    let label_key = property_key("SemanticData", "label");
    let mut checked = 0;
    for (data, host) in semantic_data_hosts(graph) {
        let label = artboard
            .debug_string_property(data, label_key)
            .map(|bytes| String::from_utf8_lossy(bytes).into_owned())
            .unwrap_or_default();
        let bounds = SemanticProvider::semantic_bounds(&mut artboard, host);
        assert!(!bounds.is_empty_or_nan(), "{label}");
        assert!(bounds.max_x - bounds.min_x > 0.0, "{label}");
        assert!(bounds.max_y - bounds.min_y > 0.0, "{label}");
        checked += 1;
    }
    assert!(checked > 0);
}
