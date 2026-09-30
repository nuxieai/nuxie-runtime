//! Complete ik_constraint_stacking_test.cpp at cb8aa75d06b0341deaf5a55171844f37bf1ac882.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    bones::{bone::Bone, root_bone::RootBone},
    constraints::{ik_constraint::IKConstraint, rotation_constraint::RotationConstraint},
    core::{CoreHandle, CoreType},
    generated::{
        constraints::constraint_base::ConstraintBase, core_registry::CoreRegistry,
        node_base::NodeBase,
    },
    math::vec2d::Vec2D,
    node::Node,
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn load(name: &str) -> (RuntimeFileHandle, CoreHandle) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(|root| std::path::PathBuf::from(root).join("tests/unit_tests/assets"))
        .unwrap_or_else(|| {
            std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sync")
        });
    let bytes = std::fs::read(root.join(name)).expect("pinned IK fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let artboard = file.with_file(File::artboard).expect("source artboard");
    (file, artboard)
}
fn advance(artboard: &CoreHandle) {
    Artboard::advance_handle(
        artboard,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn constraints(owner: &CoreHandle) -> Vec<CoreHandle> {
    owner
        .with(|owner| {
            owner
                .as_transform_component()
                .unwrap()
                .constraints()
                .to_vec()
        })
        .unwrap()
}
fn strength(owner: &CoreHandle) -> f32 {
    CoreRegistry::get_double_handle(owner, ConstraintBase::STRENGTH_PROPERTY_KEY.into()).unwrap()
}

#[test]
fn a_zero_strength_ik_does_not_undo_the_ik_stacked_before_it() {
    let (_file, artboard) = load("ik_stacked_constraints.riv");
    let (tip, target) = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            (
                artboard.find_handle::<Bone>("Bone 1").expect("Bone 1"),
                artboard
                    .find_handle::<Node>("IK Target")
                    .expect("IK Target"),
            )
        })
        .unwrap();
    let constraints = constraints(&tip);
    assert_eq!(constraints.len(), 2);
    let solve = &constraints[0];
    let off = &constraints[1];
    assert!(solve.is_type_of(IKConstraint::TYPE_KEY));
    assert!(off.is_type_of(IKConstraint::TYPE_KEY));
    assert!((strength(solve) - 1.0).abs() <= 100.0 * f32::EPSILON);
    assert_eq!(strength(off), 0.0);
    let distance = || {
        Vec2D::distance(
            tip.with_downcast::<Bone, _>(Bone::tip_world_translation)
                .unwrap(),
            target
                .with(|target| target.as_transform_component().unwrap().world_translation())
                .unwrap(),
        )
    };
    advance(&artboard);
    assert!(distance() < 0.5);
    assert!(CoreRegistry::set_double_handle(
        &target,
        NodeBase::X_PROPERTY_KEY.into(),
        300.0
    ));
    assert!(CoreRegistry::set_double_handle(
        &target,
        NodeBase::Y_PROPERTY_KEY.into(),
        160.0
    ));
    advance(&artboard);
    assert!(distance() < 0.5);
}

#[test]
fn a_zero_strength_ik_leaves_a_rotation_constrained_chain_bone_alone() {
    let (_file, artboard) = load("ik_over_rotation_constraint.riv");
    let (root, tip) = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            (
                artboard
                    .find_handle::<RootBone>("Root Bone")
                    .expect("Root Bone"),
                artboard.find_handle::<Bone>("Bone 1").expect("Bone 1"),
            )
        })
        .unwrap();
    let root_constraints = constraints(&root);
    assert_eq!(root_constraints.len(), 1);
    assert!(root_constraints[0].is_type_of(RotationConstraint::TYPE_KEY));
    let tip_constraints = constraints(&tip);
    assert_eq!(tip_constraints.len(), 1);
    assert!(tip_constraints[0].is_type_of(IKConstraint::TYPE_KEY));
    assert_eq!(strength(&tip_constraints[0]), 0.0);
    advance(&artboard);
    let rotation = root
        .with(|root| {
            root.as_transform_component()
                .unwrap()
                .world_transform()
                .decompose()
                .rotation()
        })
        .unwrap();
    assert!((rotation - 0.7853982f32).abs() <= 0.001);
}
