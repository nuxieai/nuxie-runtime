//! Direct port of solo_test.cpp's hidden-option update case at 40e52541.
#![cfg(feature = "testing")]

use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    artboard::Artboard,
    generated::{core_registry::CoreRegistry, node_base::NodeBase, solo_base::SoloBase},
    math::mat2d::Mat2D,
    shapes::shape::Shape,
    solo::Solo,
};
use nuxie_runtime::{CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle};

fn collapsed(component: &CoreHandle) -> bool {
    component
        .with(|c| c.as_component().unwrap().is_collapsed())
        .unwrap()
}

fn world_transform(shape: &CoreHandle) -> Mat2D {
    shape
        .with(|s| *s.as_world_transform_component().unwrap().world_transform())
        .unwrap()
}

fn move_x(shape: &CoreHandle, delta: f32) {
    let x = shape.with(|s| s.as_node().unwrap().base.x()).unwrap();
    assert!(CoreRegistry::set_double_handle(
        shape,
        NodeBase::X_PROPERTY_KEY.into(),
        x + delta
    ));
}

fn assert_approx(actual: f32, expected: f32) {
    assert!(
        (f64::from(actual) - f64::from(expected)).abs()
            <= f64::from(f32::EPSILON) * 100.0 * f64::from(expected).abs()
    );
}

struct Scene {
    artboard: RuntimeArtboardInstanceHandle,
    solo: CoreHandle,
    blue: CoreHandle,
    green: CoreHandle,
}

fn show(scene: &Scene, option: &CoreHandle) {
    let id = scene.artboard.with_artboard(|a| a.id_of(option));
    assert!(CoreRegistry::set_uint_handle(
        &scene.solo,
        SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY.into(),
        id
    ));
    scene.artboard.advance_default(0.0);
}

#[test]
fn updates_skip_a_solos_hidden_options_and_reach_them_once_shown() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream fixture directory");
    let bytes =
        std::fs::read(std::path::PathBuf::from(root).join("tests/unit_tests/assets/solo_test.riv"))
            .unwrap();
    let mut factory = PersistentFactory::new(NullFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let load = || {
        let source = file.with_file(File::artboard).unwrap();
        let artboard = Artboard::instance_from_handle(&source).unwrap();
        artboard.advance_default(0.0);
        let solo = artboard.with_artboard(|a| a.find_all_handles::<Solo>()[0].clone());
        let children = solo
            .with(|s| s.as_container_component().unwrap().children().to_vec())
            .unwrap();
        let blue = children[0].clone();
        let green = children[1].clone();
        assert!(blue.with_downcast::<Shape, _>(|_| ()).is_some());
        assert!(green.with_downcast::<Shape, _>(|_| ()).is_some());
        assert!(!collapsed(&blue));
        assert!(collapsed(&green));
        Scene {
            artboard,
            solo,
            blue,
            green,
        }
    };

    let reference = load();
    show(&reference, &reference.green);
    move_x(&reference.green, 100.0);
    reference.artboard.advance_default(0.0);

    let scene = load();
    let blue_x = world_transform(&scene.blue).values()[4];
    let skips = Artboard::collapsed_run_skips();
    move_x(&scene.blue, 10.0);
    move_x(&scene.green, 100.0);
    scene.artboard.advance_default(0.0);
    assert!(Artboard::collapsed_run_skips() > skips);
    assert_approx(world_transform(&scene.blue).values()[4], blue_x + 10.0);

    show(&scene, &scene.green);
    assert!(collapsed(&scene.blue));
    assert_eq!(
        world_transform(&scene.green),
        world_transform(&reference.green)
    );

    show(&scene, &scene.blue);
    move_x(&scene.blue, 10.0);
    scene.artboard.advance_default(0.0);
    assert!(collapsed(&scene.green));
    assert_approx(world_transform(&scene.blue).values()[4], blue_x + 20.0);
}
