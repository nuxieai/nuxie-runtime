//! All three cases from runtime/fade_dirt_test.cpp at upstream 02e99cf4.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    bones::skin::Skin,
    bones::skinnable::SkinnableBehavior,
    component::{ComponentDirt, ComponentOccurrenceHandle},
    constraints::{constraint::Constraint, targeted_constraint::TargetedConstraint},
    core::CoreType,
    generated::{
        constraints::targeted_constraint_base::TargetedConstraintBase, core_registry::CoreRegistry,
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    math::vec2d::Vec2D,
    shapes::{
        paint::{stroke::Stroke, trim_path::TrimPath},
        shape::Shape,
    },
    transform_component::TransformComponent,
};
use nuxie_runtime::{CoreHandle, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn file() -> RuntimeFileHandle {
    read_file("zombie_skins.riv")
}
fn read_file(asset: &str) -> RuntimeFileHandle {
    let upstream = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(upstream)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .expect("pinned fade fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("fade fixture imports")
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

fn has_skinned_path(shape: &CoreHandle) -> bool {
    let paths = shape.with_downcast::<Shape, _>(Shape::paths).unwrap();
    paths.iter().any(|path| {
        path.with(|object| {
            object
                .as_points_path()
                .is_some_and(|path| path.skin().is_some())
        })
        .unwrap_or(false)
    })
}

#[test]
fn a_skinned_shape_measures_its_trim_path_when_it_fades_in() {
    // This fixture hides skinned shapes stroked with animated trim paths at
    // zero opacity and fades them in on hover. Showing must invalidate the
    // paint even when the skin no longer gets dirtied by an opacity change.
    let file = read_file("electrified_button_simple.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let machine = artboard
        .state_machine_named("button")
        .expect("button state machine");
    machine.advance_and_apply(0.0);

    let mut hidden = Vec::new();
    for trim in artboard.with_artboard(|artboard| artboard.find_all_handles::<TrimPath>()) {
        let parent = trim
            .with(|object| object.as_component().unwrap().parent_handle())
            .flatten();
        let Some(stroke) = parent.filter(|parent| parent.is_type_of(Stroke::TYPE_KEY)) else {
            continue;
        };
        let shape = stroke
            .with(|object| object.as_component().unwrap().parent_handle())
            .flatten();
        if let Some(shape) = shape.filter(|shape| shape.is_type_of(Shape::TYPE_KEY)) {
            if has_skinned_path(&shape)
                && stroke
                    .with_downcast::<Stroke, _>(|stroke| stroke.base.base.render_opacity())
                    .unwrap()
                    == 0.0
            {
                hidden.push((trim, stroke));
            }
        }
    }
    assert!(!hidden.is_empty());

    machine.with_instance_mut(|machine| machine.pointer_move(Vec2D::new(250.0, 250.0), 0.0, 0));
    machine.advance_and_apply(0.0);

    for (trim, stroke) in hidden {
        let provider = stroke
            .with_downcast::<Stroke, _>(|stroke| {
                assert_ne!(stroke.base.base.render_opacity(), 0.0);
                *stroke.base.base.path_provider()
            })
            .unwrap();
        let effect_path = trim
            .with_mut(|object| {
                object
                    .as_stroke_effect_mut()
                    .unwrap()
                    .effect_path(&provider)
            })
            .flatten()
            .expect("trim effect path");
        assert!(!effect_path.borrow().raw_path().empty());
    }
}
