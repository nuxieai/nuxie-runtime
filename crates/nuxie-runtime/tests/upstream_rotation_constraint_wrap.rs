//! rotation_constraint_test.cpp at upstream 04104ce23473313d181489440c98fb087b991b68.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags, core::CoreHandle, math::math_types::PI,
    transform_component::TransformComponent,
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn load(name: &str) -> (RuntimeFileHandle, CoreHandle) {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR"),
    )
    .join("tests/unit_tests/assets");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &std::fs::read(root.join(name)).expect("pinned rotation constraint fixture"),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let artboard = file.with_file(File::artboard).expect("source artboard");
    (file, artboard)
}

fn advance(root: &CoreHandle) {
    Artboard::advance_handle(
        root,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}

fn rotation(component: &CoreHandle) -> f32 {
    component
        .with(|component| {
            component
                .as_transform_component()
                .unwrap()
                .world_transform()
                .decompose()
                .rotation()
        })
        .unwrap()
}

fn world_rotation(root: &CoreHandle, name: &str) -> f32 {
    let component = root
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<TransformComponent>(name))
        .flatten()
        .expect(name);
    rotation(&component)
}

#[test]
fn rotation_constraint_updates_world_transform() {
    let (_file, artboard) = load("rotation_constraint.riv");
    advance(&artboard);
    assert_eq!(
        world_rotation(&artboard, "target"),
        world_rotation(&artboard, "rect")
    );
}

#[test]
fn rotation_constraint_limits_hold_across_the_half_turn() {
    let (_file, artboard) = load("rotation_constraint_wrap.riv");
    advance(&artboard);
    for (actual, expected) in [
        (world_rotation(&artboard, "min_at_half_turn").abs(), PI),
        (
            world_rotation(&artboard, "range_across_half_turn"),
            -2.7925268,
        ),
        (world_rotation(&artboard, "max_zero_clamped"), 0.0),
        (world_rotation(&artboard, "max_zero_free"), -2.5),
    ] {
        assert!(
            (actual - expected).abs() <= 0.0001,
            "{actual} != {expected}"
        );
    }
}

#[cfg(any(feature = "testing", feature = "tools"))]
#[test]
fn rotation_constraint_limits_clamp_off_center_angles() {
    use nuxie_runtime::source::{
        constraints::rotation_constraint::RotationConstraint,
        core::CoreArena,
        generated::{
            component_base::ComponentBase,
            constraints::transform_component_constraint_base::TransformComponentConstraintBase as Limits,
            core_registry::CoreRegistry, transform_component_base::TransformComponentBase,
        },
        node::Node,
        status_code::StatusCode,
    };
    let degrees = |value: f32| value * PI / 180.0;
    for (angle, min, min_value, max, max_value, expected) in [
        (-120.0, false, 0.0, true, 90.0, 90.0),
        (120.0, true, -90.0, false, 0.0, -90.0),
        (-150.0, true, 0.0, true, 90.0, 90.0),
        (-60.0, true, 0.0, true, 90.0, 0.0),
        (45.0, true, 90.0, true, 0.0, 90.0),
        (-170.0, true, 90.0, true, 0.0, 90.0),
        (170.0, true, -180.0, false, 0.0, -180.0),
        (200.0, true, 150.0, true, 210.0, -160.0),
    ] {
        let arena = CoreArena::default();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let root = arena.insert(Artboard::with_factory(
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        ));
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.set_core_arena(arena.clone());
            artboard.add_object(Some(root.clone()));
        })
        .unwrap();
        let node = arena.insert(Node::default());
        assert!(CoreRegistry::set_double_handle(
            &node,
            TransformComponentBase::ROTATION_PROPERTY_KEY.into(),
            degrees(angle)
        ));
        assert!(CoreRegistry::set_uint_handle(
            &node,
            ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
            0
        ));
        let constraint = arena.insert(RotationConstraint::default());
        assert!(CoreRegistry::set_bool_handle(
            &constraint,
            Limits::MIN_PROPERTY_KEY.into(),
            min
        ));
        assert!(CoreRegistry::set_double_handle(
            &constraint,
            Limits::MIN_VALUE_PROPERTY_KEY.into(),
            degrees(min_value)
        ));
        assert!(CoreRegistry::set_bool_handle(
            &constraint,
            Limits::MAX_PROPERTY_KEY.into(),
            max
        ));
        assert!(CoreRegistry::set_double_handle(
            &constraint,
            Limits::MAX_VALUE_PROPERTY_KEY.into(),
            degrees(max_value)
        ));
        assert!(CoreRegistry::set_uint_handle(
            &constraint,
            ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
            1
        ));
        root.with_downcast_mut::<Artboard, _>(|artboard| {
            artboard.add_object(Some(node.clone()));
            artboard.add_object(Some(constraint));
        })
        .unwrap();
        assert_eq!(Artboard::initialize_handle(&root), StatusCode::Ok);
        advance(&root);
        let mut actual = rotation(&node);
        let mut expected = degrees(expected);
        if expected.abs() == PI {
            actual = actual.abs();
            expected = expected.abs();
        }
        assert!(
            (actual - expected).abs() <= 0.0001,
            "rotation={angle} min={min_value} max={max_value}: {actual} != {expected}"
        );
    }
}
