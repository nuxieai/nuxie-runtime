//! Both nested_artboard_test.cpp regressions added by upstream 2914b081.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::nested_artboard::NestedArtboard;
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

fn load(asset: &str) -> RuntimeFileHandle {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(asset)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(&bytes, factory, None, None, None).unwrap()
}

#[test]
fn nested_linear_animations_release_their_animation_instance() {
    let file = load("joystick_nested_remap.riv");
    let artboard = file
        .with_file(|file| file.artboard_named("parent"))
        .unwrap();
    artboard.advance_default(0.0);
    let nested = artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>());
    assert!(!nested.is_empty());
    let mut checked = 0;
    for nested in nested {
        let animations = nested
            .with(|nested| {
                nested
                    .as_nested_artboard()
                    .unwrap()
                    .nested_animations()
                    .to_vec()
            })
            .unwrap();
        for animation in animations {
            let is_linear = animation
                .with(|animation| animation.as_nested_linear_animation().is_some())
                .unwrap();
            if !is_linear {
                continue;
            }
            assert!(
                animation
                    .with(|animation| {
                        animation
                            .as_nested_linear_animation()
                            .unwrap()
                            .animation_instance()
                            .is_some()
                    })
                    .unwrap()
            );
            animation
                .with_mut(|animation| {
                    animation.nested_animation_release_dependencies();
                })
                .unwrap();
            assert!(
                animation
                    .with(|animation| {
                        animation
                            .as_nested_linear_animation()
                            .unwrap()
                            .animation_instance()
                            .is_none()
                    })
                    .unwrap()
            );
            checked += 1;
        }
    }
    assert!(checked > 0);
}

#[test]
fn nested_remap_animation_with_data_bound_keyframes_tears_down_cleanly() {
    let file = load("data_bound_keyframe_test.riv");
    let artboard = file
        .with_file(|file| file.artboard_named("parent"))
        .unwrap();
    let vmi = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi));
    let nested = artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>());
    assert_eq!(nested.len(), 1);
    // Keep only the weak arena occurrence: retaining a mounted instance here
    // would postpone its destructor past the source's explicit parent reset.
    let mounted = nested[0]
        .with(|nested| {
            nested
                .as_nested_artboard()
                .unwrap()
                .artboard_instance_default()
                .map(|instance| instance.core_handle())
        })
        .flatten()
        .unwrap();
    let binds_before_apply = mounted.data_bind_container().unwrap().data_binds().len();
    for _ in 0..4 {
        machine.advance_and_apply(0.1);
    }
    assert!(mounted.data_bind_container().unwrap().data_binds().len() > binds_before_apply);
    drop(artboard);
}
