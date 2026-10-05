use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::input::focusable::{Key, KeyModifiers};
use nuxie_runtime::{ArtboardInstance, File, RuntimeFactoryHandle, RuntimeOwnedViewModelHandle};

#[test]
fn host_keyboard_matches_upstream_key_combinations() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        std::path::Path::new(&root).join("tests/unit_tests/assets/keyboard_listener.riv"),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&bytes, retained, None, None, None).unwrap();
    let index = file
        .with_file(|file| {
            (0..file.artboard_count())
                .find(|&index| file.artboard_name_at(index) == "KeyboardInput")
        })
        .unwrap();
    let mut artboard = ArtboardInstance::from_native(file, index).unwrap();
    let file = artboard.native_file();
    let source = artboard.native_handle().core_handle();
    let model = file
        .with_file_mut(|file| file.create_default_view_model_instance_for_artboard(source))
        .and_then(|native| RuntimeOwnedViewModelHandle::from_native(file.clone(), native))
        .unwrap();
    artboard.bind_owned_view_model_handle(model.clone());
    let mut machine = artboard.state_machine_instance(0).unwrap();
    machine.bind_owned_view_model_handle(model.clone());
    macro_rules! advance {
        () => {
            artboard
                .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.016, true)
                .unwrap()
        };
    }
    macro_rules! count {
        ($value:expr) => {
            assert_eq!(
                model.borrow().number_value_by_property_name("keyCount"),
                Some($value as f32)
            )
        };
    }
    advance!();
    assert!(machine.has_focus_nodes(), "fixture focus nodes");
    assert!(machine.focus_next(), "fixture focus traversal");
    assert!(machine.focus_state().has_focus, "fixture focus state");
    advance!();
    machine.key_input(Key::A, KeyModifiers::NONE, true, false);
    advance!();
    count!(1);
    machine.key_input(Key::A, KeyModifiers::NONE, true, true);
    advance!();
    count!(1);
    machine.key_input(Key::A, KeyModifiers::NONE, false, false);
    advance!();
    count!(2);
    machine.key_input(Key::A, KeyModifiers::SHIFT, true, false);
    advance!();
    count!(2);
    machine.key_input(Key::E, KeyModifiers::NONE, false, false);
    machine.key_input(Key::E, KeyModifiers::NONE, true, true);
    machine.key_input(Key::E, KeyModifiers::NONE, true, false);
    count!(2);
    advance!();
    machine.key_input(Key::B, KeyModifiers::NONE, true, false);
    count!(2);
    advance!();
    machine.key_input(Key::B, KeyModifiers::NONE, false, false);
    advance!();
    count!(3);
    machine.key_input(Key::B, KeyModifiers::NONE, true, true);
    advance!();
    count!(4);
    machine.key_input(Key::D, KeyModifiers::NONE, true, false);
    advance!();
    count!(4);
    machine.key_input(
        Key::D,
        KeyModifiers::SHIFT | KeyModifiers::META,
        true,
        false,
    );
    advance!();
    count!(5);
    machine.key_input(
        Key::C,
        KeyModifiers::SHIFT | KeyModifiers::META,
        true,
        false,
    );
    advance!();
    count!(5);
    machine.key_input(Key::C, KeyModifiers::SHIFT, true, false);
    advance!();
    count!(6);
    machine.key_input(Key::X, KeyModifiers::SHIFT, true, false);
    advance!();
    count!(6);
}

fn import_host_artboard(bytes: &[u8]) -> ArtboardInstance {
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(bytes, retained, None, None, None).unwrap();
    ArtboardInstance::from_native(file, 0).unwrap()
}

#[test]
fn host_text_input_matches_upstream_focused_node_events() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        std::path::Path::new(&root).join("tests/unit_tests/assets/text_input_event.riv"),
    )
    .unwrap();
    let mut artboard = import_host_artboard(&bytes);
    let file = artboard.native_file();
    let source = artboard.native_handle().core_handle();
    let model = file
        .with_file_mut(|file| file.create_default_view_model_instance_for_artboard(source))
        .and_then(|native| RuntimeOwnedViewModelHandle::from_native(file.clone(), native))
        .unwrap();
    let mut machine = artboard.state_machine_instance(0).unwrap();
    machine.bind_owned_view_model_handle(model.clone());
    macro_rules! advance {
        () => {
            artboard
                .advance_state_machine_instances(std::slice::from_mut(&mut machine), 0.016, true)
                .unwrap()
        };
    }
    macro_rules! values {
        ($focused:expr, $keyed:expr, $texted:expr) => {
            for (name, expected) in [
                ("isFocused", $focused),
                ("hasKeyed", $keyed),
                ("hasTexted", $texted),
            ] {
                assert_eq!(
                    model.borrow().boolean_value_by_property_name(name),
                    Some(expected),
                    "{name}"
                );
            }
        };
    }
    advance!();
    assert!(machine.focus_next());
    advance!();
    values!(true, false, false);
    machine.key_input(Key::B, KeyModifiers::NONE, true, false);
    advance!();
    values!(true, false, false);
    machine.text_input("b");
    advance!();
    values!(true, false, true);
    machine.key_input(Key::A, KeyModifiers::NONE, true, false);
    advance!();
    values!(true, true, true);
}
