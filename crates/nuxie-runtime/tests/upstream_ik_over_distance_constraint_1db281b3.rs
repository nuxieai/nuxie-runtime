//! Direct translation of `tests/unit_tests/runtime/ik_over_distance_constraint_test.cpp`
//! at upstream 1db281b3e82baf850635fd7aa2092920a80b6a2c.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    bones::{bone::Bone, root_bone::RootBone},
    constraints::{distance_constraint::DistanceConstraint, ik_constraint::IKConstraint},
    core::CoreType,
    generated::{core_registry::CoreRegistry, node_base::NodeBase},
    math::vec2d::Vec2D,
    node::Node,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};

fn fixture_path() -> PathBuf {
    PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/ik_over_distance_constraint.riv")
}

#[test]
fn ik_leaves_a_distance_constrained_bone_above_the_tip_where_it_was() {
    let path = fixture_path();
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, factory, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("fixture imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let artboard = file.with_file(File::artboard).expect("authored artboard");

    let (main, root, tip, distance_target, ik_target) = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            (
                artboard.find_handle::<Node>("main"),
                artboard.find_handle::<RootBone>("Root Bone"),
                artboard.find_handle::<Bone>("Bone 1"),
                artboard.find_handle::<Node>("Distance Target"),
                artboard.find_handle::<Node>("IK Target"),
            )
        })
        .map(|(main, root, tip, distance_target, ik_target)| {
            (
                main.expect("main"),
                root.expect("Root Bone"),
                tip.expect("Bone 1"),
                distance_target.expect("Distance Target"),
                ik_target.expect("IK Target"),
            )
        })
        .expect("Artboard");

    let advance = || {
        Artboard::advance_handle(
            &artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
    };
    let world_translation = |object: &nuxie_runtime::CoreHandle| {
        object
            .with(|object| {
                object
                    .as_transform_component()
                    .expect("TransformComponent")
                    .world_translation()
            })
            .expect("live component")
    };
    let tip_translation = || {
        tip.with_downcast::<Bone, _>(Bone::tip_world_translation)
            .expect("Bone 1")
    };

    let root_constraints = root
        .with(|root| {
            root.as_transform_component()
                .expect("Root Bone transform")
                .constraints()
                .to_vec()
        })
        .expect("Root Bone");
    assert_eq!(root_constraints.len(), 1);
    assert!(root_constraints[0].is_type_of(DistanceConstraint::TYPE_KEY));
    let tip_constraints = tip
        .with(|tip| {
            tip.as_transform_component()
                .expect("Bone 1 transform")
                .constraints()
                .to_vec()
        })
        .expect("Bone 1");
    assert_eq!(tip_constraints.len(), 1);
    assert!(tip_constraints[0].is_type_of(IKConstraint::TYPE_KEY));

    advance();
    assert!(
        (Vec2D::distance(
            world_translation(&root),
            world_translation(&distance_target)
        ) - 1.0)
            .abs()
            <= 0.001
    );
    assert!(Vec2D::distance(tip_translation(), world_translation(&ik_target)) < 0.5);
    let before = world_translation(&root);

    let x =
        CoreRegistry::get_double_handle(&main, NodeBase::X_PROPERTY_KEY.into()).expect("main.x");
    assert!(CoreRegistry::set_double_handle(
        &main,
        NodeBase::X_PROPERTY_KEY.into(),
        x + 100.0,
    ));
    advance();

    assert!(
        (Vec2D::distance(
            world_translation(&root),
            world_translation(&distance_target)
        ) - 1.0)
            .abs()
            <= 0.001
    );
    assert!(Vec2D::distance(world_translation(&root), before) < 2.0);
}
