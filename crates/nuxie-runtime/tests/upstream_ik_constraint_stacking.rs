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
            std::path::PathBuf::from(
                option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
            )
            .join("../../fixtures/sync")
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

#[test]
fn ik_inherited_dirty_callbacks_invalidate_the_entire_initialized_chain() {
    use nuxie_runtime::source::{
        component::ComponentOccurrenceHandle, component_dirt::ComponentDirt,
        generated::constraints::ik_constraint_base::IKConstraintBase,
    };
    let (_file, artboard) = load("ik_stacked_constraints.riv");
    advance(&artboard);
    let (root, tip, objects) = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            (
                artboard
                    .find_handle::<RootBone>("Root Bone")
                    .expect("Root Bone"),
                artboard.find_handle::<Bone>("Bone 1").expect("Bone 1"),
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .cloned()
                    .collect::<Vec<_>>(),
            )
        })
        .unwrap();
    let ik = constraints(&tip)[0].clone();
    let clear = || {
        for object in &objects {
            object.with_mut(|object| {
                if let Some(component) = object.as_component_mut() {
                    component.set_dirt(ComponentDirt::NONE);
                }
            });
        }
    };
    let transform_dirty = |owner: &CoreHandle| {
        owner
            .with(|owner| {
                owner
                    .as_component()
                    .unwrap()
                    .has_dirt(ComponentDirt::TRANSFORM)
            })
            .unwrap()
    };

    // Constraint::onDirty excludes opacity-only work, including the retained
    // Collapsed bit. Neither the tip nor IK's ancestor override should run.
    for opacity in [
        ComponentDirt::RENDER_OPACITY,
        ComponentDirt::RENDER_OPACITY | ComponentDirt::COLLAPSED,
    ] {
        clear();
        assert!(ComponentOccurrenceHandle::Authored(ik.clone()).add_dirt(opacity, false));
        assert!(!transform_dirty(&tip));
        assert!(!transform_dirty(&root));
    }

    clear();
    assert!(
        ComponentOccurrenceHandle::Authored(ik.clone())
            .add_dirt(ComponentDirt::WORLD_TRANSFORM, false)
    );
    assert!(transform_dirty(&tip), "base callback invalidates tip");
    assert!(
        transform_dirty(&root),
        "virtual IK override invalidates ancestor"
    );

    clear();
    assert!(CoreRegistry::set_double_handle(
        &ik,
        ConstraintBase::STRENGTH_PROPERTY_KEY.into(),
        0.5
    ));
    assert_eq!(strength(&ik), 0.5);
    assert!(transform_dirty(&tip));
    assert!(transform_dirty(&root));

    clear();
    assert!(CoreRegistry::set_double_handle(
        &ik,
        ConstraintBase::STRENGTH_PROPERTY_KEY.into(),
        0.5,
    ));
    assert!(!transform_dirty(&tip), "unchanged strength has no callback");
    assert!(!transform_dirty(&root));

    clear();
    let invert =
        CoreRegistry::get_bool_handle(&ik, IKConstraintBase::INVERT_DIRECTION_PROPERTY_KEY.into())
            .unwrap();
    assert!(CoreRegistry::set_bool_handle(
        &ik,
        IKConstraintBase::INVERT_DIRECTION_PROPERTY_KEY.into(),
        !invert
    ));
    assert_eq!(
        CoreRegistry::get_bool_handle(&ik, IKConstraintBase::INVERT_DIRECTION_PROPERTY_KEY.into()),
        Some(!invert)
    );
    assert!(transform_dirty(&tip));
    assert!(transform_dirty(&root));
}

#[test]
fn ik_handle_setters_reject_expired_owners() {
    use nuxie_runtime::source::{
        core::CoreArena, generated::constraints::ik_constraint_base::IKConstraintBase,
    };
    let expired = {
        let arena = CoreArena::default();
        arena.insert(IKConstraint::default())
    };
    assert!(!CoreRegistry::set_double_handle(
        &expired,
        ConstraintBase::STRENGTH_PROPERTY_KEY.into(),
        0.5
    ));
    assert!(!CoreRegistry::set_bool_handle(
        &expired,
        IKConstraintBase::INVERT_DIRECTION_PROPERTY_KEY.into(),
        true
    ));
}
