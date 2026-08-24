//! Direct port of pinned `tests/unit_tests/runtime/bound_bones_test.cpp`.

use std::path::PathBuf;

use nuxie::File;

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

#[test]
fn bound_bones_load_correctly() {
    let file = File::import(&pinned_fixture("off_road_car.riv")).expect("import fixture");
    let graph = file.default_artboard().expect("default artboard").graph();
    let node = graph
        .component_named("transmission_front_testing")
        .expect("transmission shape");
    assert_eq!(node.type_name, "Shape");
    let paths = graph
        .paths
        .iter()
        .filter(|path| node.children.contains(&path.local_id))
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1);
    let path = paths[0];
    assert_eq!(path.type_name, "PointsPath");
    let skin = graph
        .skeletal_skins
        .iter()
        .find(|skin| skin.skinnable_local == Some(path.local_id))
        .expect("points-path skin");
    assert_eq!(skin.tendons.len(), 2);
    assert!(skin.tendons[0].bone_local.is_some());
    assert!(skin.tendons[1].bone_local.is_some());

    for vertex in &path.vertices {
        assert!(vertex.weight_local.is_some());
    }
}
