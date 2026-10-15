//! Supplemental coverage for the public supplied-definition/instance overloads
//! introduced by 8f06a91c. These do not enable the native REV authoring backend.
use nuxie_runtime::source::{
    animation::{
        linear_animation::LinearAnimation, nested_linear_animation::NestedLinearAnimation,
        nested_state_machine::NestedStateMachine,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreArena,
    generated::{
        animation::animation_base::AnimationBase, core_registry::CoreRegistry,
        nested_animation_base::NestedAnimationBase,
    },
    nested_artboard::NestedArtboard,
};

#[test]
fn supplied_linear_definition_bypasses_index_and_none_clears_instance() {
    let arena = CoreArena::default();
    let definition = arena.insert(LinearAnimation::default());
    assert!(CoreRegistry::set_string_handle(
        &definition,
        AnimationBase::NAME_PROPERTY_KEY.into(),
        "supplied definition".into()
    ));
    let mut nested = NestedLinearAnimation::default();
    assert_eq!(nested.base.animation_id(), u32::MAX);
    nested.initialize_animation_with_definition(
        RuntimeArtboardInstanceWeakHandle::default(),
        Some(definition),
    );
    assert_eq!(
        nested.animation_instance().unwrap().name(),
        "supplied definition"
    );
    nested.initialize_animation_with_definition(RuntimeArtboardInstanceWeakHandle::default(), None);
    assert!(nested.animation_instance().is_none());
}

#[test]
fn supplied_machine_retains_identity_and_shares_parent_focus_manager() {
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    let root = std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap());
    let bytes = std::fs::read(root.join("tests/unit_tests/assets/hidden_hit_targets.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file
        .with_file(|file| file.artboard_named("NestedHost"))
        .unwrap();
    let _parent_machine = artboard.state_machine_at(0).unwrap();
    artboard.advance_default(0.0);
    let mut checked = 0;
    for nested_artboard in
        artboard.with_artboard(|board| board.find_all_handles::<NestedArtboard>())
    {
        let (child, animations) = nested_artboard
            .with_downcast::<NestedArtboard, _>(|nested| {
                (
                    nested.artboard_instance_default(),
                    nested.nested_animations().to_vec(),
                )
            })
            .unwrap();
        let Some(child) = child else {
            continue;
        };
        for owner in animations {
            if owner
                .with_downcast::<NestedStateMachine, _>(|_| ())
                .is_none()
            {
                continue;
            }
            let supplied = child.state_machine_at(0).unwrap();
            assert!(CoreRegistry::set_uint_handle(
                &owner,
                NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY.into(),
                u32::MAX
            ));
            NestedStateMachine::initialize_animation_handle_with_instance(
                &owner,
                child.downgrade(),
                Some(supplied.clone()),
            );
            let installed = owner
                .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
                .flatten()
                .unwrap();
            assert!(installed.downgrade().ptr_eq(&supplied.downgrade()));
            let parent_focus = artboard
                .with_artboard(|board| board.focus_manager_handle())
                .unwrap();
            let nested_focus = installed
                .with_instance(|machine| machine.focus_manager())
                .unwrap();
            assert!(parent_focus.ptr_eq(&nested_focus));
            NestedStateMachine::initialize_animation_handle_with_instance(
                &owner,
                child.downgrade(),
                None,
            );
            assert!(
                owner
                    .with_downcast::<NestedStateMachine, _>(
                        NestedStateMachine::state_machine_instance
                    )
                    .flatten()
                    .is_none()
            );
            checked += 1;
        }
    }
    assert!(checked > 0);
}

#[cfg(feature = "editor")]
#[test]
fn editor_setter_and_keyboard_clear_require_no_native_editor_arena() {
    use nuxie_runtime::source::{
        animation::{
            animation_state::AnimationState,
            listener_types::listener_input_type_keyboard::ListenerInputTypeKeyboard,
        },
        inputs::keyboard_input::KeyboardInput,
    };
    let arena = CoreArena::default();
    let definition = arena.insert(LinearAnimation::default());
    let mut state = AnimationState::default();
    state.set_animation_for_editor(Some(definition.clone()));
    assert_eq!(state.animation(), Some(definition));
    state.set_animation_for_editor(None);
    assert!(state.animation().is_none());
    let mut listener = ListenerInputTypeKeyboard::default();
    listener.add_keyboard_input(arena.insert(KeyboardInput::default()));
    assert_eq!(listener.keyboard_input_count(), 1);
    listener.clear_keyboard_inputs_for_editor();
    assert_eq!(listener.keyboard_input_count(), 0);
}
