//! Complete added bounds_test.cpp case at 7133967b25572642eb228f29254e64ed0b626469.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    component::ComponentOccurrenceHandle,
    generated::{
        core_registry::CoreRegistry, node_base::NodeBase,
        transform_component_base::TransformComponentBase,
    },
    math::aabb::Aabb,
    shapes::shape::Shape,
};
use nuxie_runtime::{Artboard, CoreHandle, File, RuntimeFactoryHandle};
use std::path::PathBuf;

fn advance(artboard: &CoreHandle) {
    Artboard::advance_handle(
        artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn set(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}
fn mark_transform_dirty(owner: &CoreHandle) {
    owner
        .with_mut(|object| {
            object
                .as_transform_component_mut()
                .unwrap()
                .mark_transform_dirty()
        })
        .unwrap();
}
fn local_bounds(shape: &CoreHandle) -> Aabb {
    shape
        .with_downcast::<Shape, _>(Shape::local_bounds)
        .unwrap()
}
fn check_bounds(bounds: Aabb, expected: [f32; 4]) {
    for (actual, expected) in [bounds.left(), bounds.top(), bounds.right(), bounds.bottom()]
        .into_iter()
        .zip(expected)
    {
        // Catch Approx's default relative epsilon (evaluated in double).
        assert!(
            (f64::from(actual) - f64::from(expected)).abs()
                <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
            "expected {expected}, got {actual}"
        );
    }
}

#[test]
fn local_bounds_cancel_the_shapes_own_transform() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/local_bounds.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard).unwrap();
    let shape = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Shape>("Shape3"))
        .flatten()
        .expect("Shape3");
    let paths = shape
        .with_downcast::<Shape, _>(|shape| shape.paths().to_vec())
        .unwrap();
    assert_eq!(paths.len(), 1);
    let path = &paths[0];

    set(path, NodeBase::X_PROPERTY_KEY, 20.0);
    set(path, NodeBase::Y_PROPERTY_KEY, 10.0);
    mark_transform_dirty(path);
    advance(&artboard);
    check_bounds(local_bounds(&shape), [20.0, 10.0, 80.0, 70.0]);

    set(
        &shape,
        TransformComponentBase::ROTATION_PROPERTY_KEY,
        std::f32::consts::PI / 2.0,
    );
    set(&shape, TransformComponentBase::SCALE_X_PROPERTY_KEY, 2.0);
    mark_transform_dirty(&shape);
    advance(&artboard);
    check_bounds(local_bounds(&shape), [20.0, 10.0, 80.0, 70.0]);

    set(path, NodeBase::X_PROPERTY_KEY, 50.0);
    mark_transform_dirty(path);
    advance(&artboard);
    check_bounds(local_bounds(&shape), [50.0, 10.0, 110.0, 70.0]);

    ComponentOccurrenceHandle::Authored(path.clone()).collapse(true);
    let bounds = local_bounds(&shape);
    let fresh = shape
        .with_downcast::<Shape, _>(Shape::compute_local_bounds)
        .unwrap();
    assert_eq!(bounds.left(), fresh.left());
    assert_eq!(bounds.top(), fresh.top());
    assert_eq!(bounds.right(), fresh.right());
    assert_eq!(bounds.bottom(), fresh.bottom());

    ComponentOccurrenceHandle::Authored(path.clone()).collapse(false);
    check_bounds(local_bounds(&shape), [50.0, 10.0, 110.0, 70.0]);
}
