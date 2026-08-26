//! Exact executable ports of the non-Silver cases in pinned
//! `tests/unit_tests/runtime/follow_path_constraint_test.cpp`.

use nuxie_binary::read_runtime_file;
use nuxie_graph::GraphFile;
use nuxie_runtime::ArtboardInstance;
use std::path::{Path, PathBuf};

fn fixture_path(name: &str) -> PathBuf {
    let runtime = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime)
        .join("tests/unit_tests/assets")
        .join(name)
}

fn assert_target_and_rectangle_world_positions_match(fixture: &str) {
    let path = fixture_path(fixture);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let runtime =
        read_runtime_file(&bytes).unwrap_or_else(|error| panic!("import {fixture}: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&runtime)
        .unwrap_or_else(|error| panic!("graph {fixture}: {error:#}"));
    let graph = graphs.artboards.first().expect("default artboard");
    let mut artboard =
        ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
            .unwrap_or_else(|error| panic!("instantiate {fixture}: {error:#}"));
    let named = |name: &str| {
        graph
            .components
            .iter()
            .find(|component| component.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("{fixture} missing {name}"))
            .local_id
    };
    let target = named("target");
    let rectangle = named("rect");

    artboard.update_pass();
    let target = artboard
        .object_world_transform(target)
        .expect("target world transform");
    let rectangle = artboard
        .object_world_transform(rectangle)
        .expect("rectangle world transform");
    assert_eq!(target.0[4], rectangle.0[4]);
    assert_eq!(target.0[5], rectangle.0[5]);
}

#[test]
fn wave_b4_follow_path_case_001_updates_world_transform() {
    assert_target_and_rectangle_world_positions_match("follow_path.riv");
}

#[test]
fn wave_b4_follow_path_case_002_zero_constraint_opacity_updates_world_transform() {
    assert_target_and_rectangle_world_positions_match("follow_path_with_0_opacity.riv");
}

#[test]
fn wave_b4_follow_path_case_003_zero_path_opacity_updates_world_transform() {
    assert_target_and_rectangle_world_positions_match("follow_path_path_0_opacity.riv");
}
