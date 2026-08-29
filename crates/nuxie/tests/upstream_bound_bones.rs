//! Direct port of pinned `tests/unit_tests/runtime/bound_bones_test.cpp`.

use std::path::PathBuf;

use nuxie::{File, PersistentFactory, RuntimeFactoryHandle};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::{
    bones::{skin::Skin, skinnable::SkinnableBehavior, tendon::Tendon},
    shapes::{points_path::PointsPath, shape::Shape},
};

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
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture("off_road_car.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let shape = artboard
        .with_artboard(|artboard| {
            artboard
                .base
                .find_handle::<Shape>("transmission_front_testing")
        })
        .expect("transmission shape");
    let paths = shape
        .with_downcast::<Shape, _>(Shape::paths)
        .expect("shape owner");
    assert_eq!(paths.len(), 1);
    let path = paths[0].clone();
    assert!(path.is_type_of(PointsPath::TYPE_KEY));
    let skin = path
        .with_downcast::<PointsPath, _>(PointsPath::skin)
        .flatten()
        .expect("points-path skin");
    let tendons = skin
        .with_downcast::<Skin, _>(|skin| skin.tendons().to_vec())
        .expect("skin owner");
    assert_eq!(tendons.len(), 2);
    assert!(
        tendons[0]
            .with_downcast::<Tendon, _>(Tendon::bone)
            .flatten()
            .is_some()
    );
    assert!(
        tendons[1]
            .with_downcast::<Tendon, _>(Tendon::bone)
            .flatten()
            .is_some()
    );

    let vertices = path
        .with(|path| {
            path.as_path().map(|path| {
                path.vertices()
                    .iter()
                    .filter_map(|vertex| vertex.authored_handle())
                    .collect::<Vec<_>>()
            })
        })
        .flatten()
        .expect("path owner");
    for vertex in vertices {
        assert!(
            vertex
                .with(|vertex| {
                    vertex
                        .as_vertex_behavior()
                        .is_some_and(|vertex| vertex.weight_handle().is_some())
                })
                .expect("vertex owner")
        );
    }
}
