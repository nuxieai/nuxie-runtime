//! Both cases from runtime/fade_dirt_test.cpp at upstream 955d6a05.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    bones::skin::Skin,
    component::{ComponentDirt, ComponentOccurrenceHandle},
    constraints::{constraint::Constraint, targeted_constraint::TargetedConstraint},
    core::CoreType,
    generated::{
        constraints::targeted_constraint_base::TargetedConstraintBase, core_registry::CoreRegistry,
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    transform_component::TransformComponent,
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn file() -> RuntimeFileHandle {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(upstream).join("tests/unit_tests/assets/zombie_skins.riv"),
    )
    .expect("zombie fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("zombie imports")
}
fn parent(object: &CoreHandle) -> CoreHandle {
    object
        .with(|object| object.as_component().unwrap().parent_handle())
        .flatten()
        .unwrap()
}
fn opacity(artboard: &CoreHandle, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        artboard,
        i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
        value
    ));
}

#[test]
fn a_fade_neither_reskins_nor_reconstrains() {
    let file = file();
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    artboard.advance_default(0.0);
    let skins = artboard.with_artboard(|artboard| artboard.find_all_handles::<Skin>());
    let constraints = artboard.with_artboard(|artboard| artboard.find_all_handles::<Constraint>());
    assert!(!skins.is_empty());
    assert!(!constraints.is_empty());
    opacity(&artboard.core_handle(), 0.5);
    for skin in skins {
        parent(&skin)
            .with(|object| {
                let component = object.as_component().unwrap();
                assert!(!component.has_dirt(ComponentDirt::PATH));
                assert!(!component.has_dirt(ComponentDirt::VERTICES));
            })
            .unwrap();
    }
    for constraint in constraints {
        parent(&constraint)
            .with(|object| {
                assert!(
                    !object
                        .as_component()
                        .unwrap()
                        .has_dirt(ComponentDirt::TRANSFORM)
                );
            })
            .unwrap();
    }
}

#[test]
fn constrained_parts_hidden_during_a_fade_come_back_constrained() {
    let file = file();
    let hidden = file.with_file(File::artboard_default).unwrap();
    let shown = file.with_file(File::artboard_default).unwrap();
    hidden.advance_default(0.0);
    shown.advance_default(0.0);
    let hidden_constraints = hidden.with_artboard(|a| a.find_all_handles::<TargetedConstraint>());
    let shown_constraints = shown.with_artboard(|a| a.find_all_handles::<TargetedConstraint>());
    assert!(!hidden_constraints.is_empty());
    assert_eq!(hidden_constraints.len(), shown_constraints.len());
    let mut fade = 1.0;
    for (hidden_constraint, shown_constraint) in hidden_constraints.iter().zip(&shown_constraints) {
        let constrained = parent(hidden_constraint);
        let reference = parent(shown_constraint);
        let target_key = i32::from(TargetedConstraintBase::TARGET_ID_PROPERTY_KEY);
        let hidden_target = hidden.with_artboard(|a| {
            a.resolve_handle(CoreRegistry::get_id_handle(hidden_constraint, target_key).unwrap())
        });
        let shown_target = shown.with_artboard(|a| {
            a.resolve_handle(CoreRegistry::get_id_handle(shown_constraint, target_key).unwrap())
        });
        let Some(hidden_target) =
            hidden_target.filter(|t| t.is_type_of(TransformComponent::TYPE_KEY))
        else {
            continue;
        };
        let shown_target = shown_target.unwrap();
        fade = if fade == 1.0 { 0.5 } else { 1.0 };
        ComponentOccurrenceHandle::Authored(constrained.clone()).collapse(true);
        hidden.advance_default(0.0);
        let rotation = i32::from(TransformComponentBase::ROTATION_PROPERTY_KEY);
        let value = CoreRegistry::get_double_handle(&hidden_target, rotation).unwrap();
        assert!(CoreRegistry::set_double_handle(
            &hidden_target,
            rotation,
            value + 0.3
        ));
        opacity(&hidden.core_handle(), fade);
        hidden.advance_default(0.0);
        ComponentOccurrenceHandle::Authored(constrained.clone()).collapse(false);
        hidden.advance_default(0.0);
        let value = CoreRegistry::get_double_handle(&shown_target, rotation).unwrap();
        assert!(CoreRegistry::set_double_handle(
            &shown_target,
            rotation,
            value + 0.3
        ));
        opacity(&shown.core_handle(), fade);
        shown.advance_default(0.0);
        let world = |handle: &CoreHandle| {
            handle
                .with(|object| *object.as_transform_component().unwrap().world_transform())
                .unwrap()
        };
        assert_eq!(world(&constrained), world(&reference));
    }
}
