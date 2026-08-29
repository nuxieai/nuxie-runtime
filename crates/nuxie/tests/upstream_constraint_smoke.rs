//! Direct ports of the pinned constraint tests in `tests/unit_tests/runtime`.

use std::path::PathBuf;

use nuxie::{
    CoreHandle, File, Mat2D, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::{
    bones::{bone::Bone, skin::Skin},
    constraints::{distance_constraint::DistanceConstraint, ik_constraint::IKConstraint},
    core::CoreType,
    generated::{core_registry::CoreRegistry, node_base::NodeBase},
    shapes::shape::Shape,
    transform_component::TransformComponent,
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

fn fixture(
    name: &str,
) -> (
    PersistentFactory<SerializingFactory>,
    RuntimeArtboardInstanceHandle,
) {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let file = File::import(
        &pinned_fixture(name),
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("import fixture");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    (factory, artboard)
}

fn named<T: CoreType>(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.base.find_handle::<T>(name))
        .unwrap_or_else(|| panic!("component {name}"))
}

fn world_transform(owner: &CoreHandle) -> Mat2D {
    owner
        .with(|owner| {
            owner
                .as_world_transform_component()
                .map(|owner| *owner.world_transform())
        })
        .flatten()
        .expect("WorldTransformComponent")
}

fn graph_order(owner: &CoreHandle) -> u32 {
    owner
        .with(|owner| owner.as_component().expect("Component").graph_order())
        .expect("live component")
}

#[test]
fn rotation_constraint_updates_world_transform() {
    let (_factory, artboard) = fixture("rotation_constraint.riv");
    let target = named::<TransformComponent>(&artboard, "target");
    let rectangle = named::<TransformComponent>(&artboard, "rect");

    artboard.advance_default(0.0);
    let target_components = world_transform(&target).decompose();
    let rectangle_components = world_transform(&rectangle).decompose();

    assert_eq!(
        target_components.rotation(),
        rectangle_components.rotation()
    );
}

#[test]
fn scale_constraint_updates_world_transform() {
    let (_factory, artboard) = fixture("scale_constraint.riv");
    let target = named::<TransformComponent>(&artboard, "target");
    let rectangle = named::<TransformComponent>(&artboard, "rect");

    artboard.advance_default(0.0);
    let target_components = world_transform(&target).decompose();
    let rectangle_components = world_transform(&rectangle).decompose();

    assert_eq!(target_components.scale_x(), rectangle_components.scale_x());
    assert_eq!(target_components.scale_y(), rectangle_components.scale_y());
}

#[test]
fn transform_constraint_updates_world_transform() {
    let (_factory, artboard) = fixture("transform_constraint.riv");
    let target = named::<TransformComponent>(&artboard, "Target");
    let rectangle = named::<TransformComponent>(&artboard, "Rectangle");

    artboard.advance_default(0.0);
    let target_transform = world_transform(&target);
    let rectangle_transform = world_transform(&rectangle);

    assert!(
        target_transform
            .values()
            .iter()
            .zip(rectangle_transform.values())
            .all(|(target, rectangle)| (target - rectangle).abs() <= 0.0001)
    );
}

#[test]
fn translation_constraint_updates_world_transform() {
    let (_factory, artboard) = fixture("translation_constraint.riv");
    let target = named::<TransformComponent>(&artboard, "target");
    let rectangle = named::<TransformComponent>(&artboard, "rect");

    artboard.advance_default(0.0);
    let target_components = world_transform(&target).decompose();
    let rectangle_components = world_transform(&rectangle).decompose();

    assert_eq!(target_components.x(), rectangle_components.x());
    assert_eq!(target_components.y(), rectangle_components.y());
}

#[test]
fn ik_with_skinned_bones_orders_correctly() {
    let (_factory, artboard) = fixture("complex_ik_dependency.riv");
    let one = named::<Bone>(&artboard, "One");
    let two = named::<Bone>(&artboard, "Two");
    let skin = artboard
        .with_artboard(|artboard| {
            artboard
                .base
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(Skin::TYPE_KEY))
                .cloned()
        })
        .expect("Skin");
    let first_constraint = two
        .with(|two| {
            two.as_transform_component()
                .expect("Bone transform")
                .constraints()[0]
                .clone()
        })
        .expect("live bone");

    assert!(first_constraint.is_type_of(IKConstraint::TYPE_KEY));
    assert!(graph_order(&skin) > graph_order(&one));
    assert!(graph_order(&skin) > graph_order(&two));
}

#[test]
fn distance_constraints_move_items_as_expected() {
    let (_factory, artboard) = fixture("distance_constraint.riv");
    let a = named::<Shape>(&artboard, "A");
    let b = named::<Shape>(&artboard, "B");
    let constraints = a
        .with_downcast::<Shape, _>(|shape| shape.base.constraints().to_vec())
        .expect("Shape A");

    assert_eq!(constraints.len(), 1);
    assert!(constraints[0].is_type_of(DistanceConstraint::TYPE_KEY));
    assert_eq!(
        constraints[0]
            .with_downcast::<DistanceConstraint, _>(|constraint| constraint.base.mode_value()),
        Some(1)
    );

    assert!(CoreRegistry::set_double_handle(
        &b,
        NodeBase::X_PROPERTY_KEY.into(),
        259.31,
    ));
    assert!(CoreRegistry::set_double_handle(
        &b,
        NodeBase::Y_PROPERTY_KEY.into(),
        137.87,
    ));
    artboard.advance_default(0.0);

    let [_, _, _, _, x, y] = *world_transform(&a).values();
    let expected_x = 259.280_88;
    let expected_y = 62.870_003;
    assert!(((x - expected_x).powi(2) + (y - expected_y).powi(2)).sqrt() < 0.001);
}
