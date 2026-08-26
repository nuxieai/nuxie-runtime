fn wave_b4_assert_follow_path_owner_flow(fixture: &str) {
    let fixture_path = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(fixture);
    let file = read_runtime_file(
        &std::fs::read(&fixture_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", fixture_path.display())),
    )
    .unwrap_or_else(|error| panic!("import {fixture}: {error:#}"));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("graph {fixture}: {error:#}"));
    let graph = graphs.artboards.first().expect("default artboard");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .unwrap_or_else(|error| panic!("instantiate {fixture}: {error:#}"));

    // C++ `find<TransformComponent>` is a typed lookup. Rust's graph keeps
    // that exact inheritance result in the transform capability bit.
    let target = graph
        .component_named("target")
        .expect("target TransformComponent");
    assert!(
        target.capabilities.transform,
        "target is a TransformComponent"
    );
    let rectangle = graph
        .component_named("rect")
        .expect("rect TransformComponent");
    assert!(
        rectangle.capabilities.transform,
        "rect is a TransformComponent"
    );

    // Preserve the root Artboard settlement boundary, then run both results
    // through the runtime Mat2D::decompose owner before reading x/y.
    artboard.advance(0.0).expect("root Artboard::advance(0)");
    let target_components = Mat2D(
        artboard
            .object_world_transform(target.local_id)
            .expect("target world transform")
            .0,
    )
    .decompose();
    let rectangle_components = Mat2D(
        artboard
            .object_world_transform(rectangle.local_id)
            .expect("rect world transform")
            .0,
    )
    .decompose();
    assert_eq!(target_components.x, rectangle_components.x);
    assert_eq!(target_components.y, rectangle_components.y);
}

#[test]
fn wave_b4_follow_path_case_001_exact_owner_flow() {
    wave_b4_assert_follow_path_owner_flow("follow_path.riv");
}

#[test]
fn wave_b4_follow_path_case_002_exact_owner_flow() {
    wave_b4_assert_follow_path_owner_flow("follow_path_with_0_opacity.riv");
}

#[test]
fn wave_b4_follow_path_case_003_exact_owner_flow() {
    wave_b4_assert_follow_path_owner_flow("follow_path_path_0_opacity.riv");
}
