//! Literal image_listener_test.cpp at upstream fab1d5692.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        state_machine::StateMachine, state_machine_instance::RuntimeStateMachineInstanceHandle,
        state_machine_listener::StateMachineListener,
    },
    assets::image_asset::ImageAsset,
    generated::{
        animation::state_machine_listener_base::StateMachineListenerBase,
        core_registry::CoreRegistry,
    },
    listener_phase::ListenerPhase,
    math::vec2d::Vec2D,
    pointer_button::PointerButton,
    shapes::image::Image,
};
use nuxie_runtime::{
    Artboard, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn bytes() -> Vec<u8> {
    let root =
        std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream"));
    std::fs::read(root.join("tests/unit_tests/assets/image_listener_targets.riv")).unwrap()
}
fn import(bytes: &[u8]) -> RuntimeFileHandle {
    // Like SerializingFactory, this actually decodes the embedded PNG. A
    // NoOpFactory would change the fixture's hit geometry by leaving it absent.
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}
fn capture(file: &RuntimeFileHandle, name: &str) {
    let source = file
        .with_file(|f| f.artboard_named_source("Images"))
        .unwrap();
    let machine = source
        .with_downcast::<Artboard, _>(|a| a.state_machine_handle_at(0))
        .flatten()
        .unwrap();
    let listeners = machine
        .with_downcast::<StateMachine, _>(|m| {
            (0..m.listener_count())
                .map(|i| m.listener(i).unwrap())
                .collect::<Vec<_>>()
        })
        .unwrap();
    for listener in listeners {
        if listener
            .with_downcast::<StateMachineListener, _>(|l| l.name() == name)
            .unwrap()
        {
            assert!(CoreRegistry::set_uint_handle(
                &listener,
                StateMachineListenerBase::PHASE_VALUE_PROPERTY_KEY.into(),
                ListenerPhase::Capture as u32
            ));
            return;
        }
    }
    panic!("no listener named {name}");
}
struct Scene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
}
impl Scene {
    fn new(file: RuntimeFileHandle) -> Self {
        let artboard = file.with_file(|f| f.artboard_named("Images")).unwrap();
        let machine = artboard.state_machine_at(0).unwrap();
        machine.advance_and_apply(0.0);
        Self {
            _file: file,
            artboard,
            machine,
        }
    }
    fn press(&self, x: f32, y: f32) {
        let p = Vec2D::new(x, y);
        self.machine
            .with_instance_mut(|m| m.pointer_down(p, 0, PointerButton::Primary));
        self.machine
            .with_instance_mut(|m| m.pointer_up(p, 0, PointerButton::Primary));
        self.machine.advance_and_apply(0.0);
    }
    fn pressed(&self, name: &str) -> bool {
        self.machine
            .with_instance(|m| m.get_bool(name).unwrap().value())
    }
}
#[test]
fn images_are_pointer_listener_targets() {
    let scene = Scene::new(import(&bytes()));
    assert!(scene.machine.with_instance(|m| m.has_listeners()));
    assert_eq!(scene.machine.with_instance(|m| m.hit_components_count()), 6);
    scene.press(50.0, 50.0);
    assert!(scene.pressed("Direct"));
    scene.press(150.0, 50.0);
    assert!(scene.pressed("Group"));
    scene.press(290.0, 10.0);
    assert!(!scene.pressed("Meshed"));
    scene.press(210.0, 90.0);
    assert!(scene.pressed("Meshed"));
    scene.press(350.0, 50.0);
    assert!(!scene.pressed("Sliced"));
    scene.press(305.0, 5.0);
    assert!(scene.pressed("Sliced"));
    scene.press(450.0, 50.0);
    assert!(!scene.pressed("Hidden"));
    scene.press(550.0, 50.0);
    assert!(scene.pressed("Transparent"));
}
#[test]
fn image_capture_listeners_run_in_the_capture_phase() {
    for name in ["Direct", "Group"] {
        let file = import(&bytes());
        capture(&file, name);
        let scene = Scene::new(file);
        assert!(
            scene.machine.with_instance(|m| m.has_capture_listeners()),
            "{name}"
        );
        scene.press(if name == "Direct" { 50.0 } else { 150.0 }, 50.0);
        assert!(scene.pressed(name), "{name}");
    }
}
#[test]
fn images_without_a_decoded_image_take_no_hits() {
    let scene = Scene::new(import(&bytes()));
    let image = scene
        .artboard
        .with_artboard(|a| a.find_handle::<Image>("Direct"))
        .unwrap();
    let asset = image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .unwrap();
    ImageAsset::set_render_image_occurrence(&asset, None);
    scene.machine.advance_and_apply(0.0);
    scene.press(50.0, 50.0);
    assert!(!scene.pressed("Direct"));
}
#[test]
fn images_in_files_before_7_5_are_not_pointer_targets() {
    let mut bytes = bytes();
    assert!(bytes.len() > 6);
    assert_eq!(bytes[4], 7);
    bytes[5] = 4;
    let file = import(&bytes);
    let artboard = file.with_file(|f| f.artboard_named("Images")).unwrap();
    let image = artboard
        .with_artboard(|a| a.find_handle::<Image>("Direct"))
        .unwrap();
    assert!(
        !image
            .with_downcast::<Image, _>(Image::is_pointer_target)
            .unwrap()
    );
    let machine = artboard.state_machine_at(0).unwrap();
    machine.advance_and_apply(0.0);
    let scene = Scene {
        _file: file,
        artboard,
        machine,
    };
    assert_eq!(scene.machine.with_instance(|m| m.hit_components_count()), 0);
    assert!(!scene.machine.with_instance(|m| m.has_listeners()));
    for x in [50.0, 150.0, 550.0] {
        let p = Vec2D::new(x, 50.0);
        scene
            .machine
            .with_instance_mut(|m| m.pointer_down(p, 0, PointerButton::Primary));
        scene
            .machine
            .with_instance_mut(|m| m.pointer_up(p, 0, PointerButton::Primary));
    }
    scene.machine.advance_and_apply(0.0);
    for name in ["Direct", "Group", "Transparent"] {
        assert!(!scene.pressed(name));
    }
}

#[test]
fn direct_image_virtual_primary_hits_use_decoded_image_and_drawn_geometry() {
    let scene = Scene::new(import(&bytes()));
    let hit = |name: &str, x, y| {
        let image = scene
            .artboard
            .with_artboard(|a| a.find_handle::<Image>(name))
            .unwrap();
        image
            .with_mut(|image| image.component_hit_test_point(&Vec2D::new(x, y), false, true))
            .flatten()
            .unwrap()
    };
    assert!(hit("Direct", 50.0, 50.0));
    assert!(!hit("Direct", 150.0, 50.0));
    assert!(!hit("Meshed", 290.0, 10.0));
    assert!(hit("Meshed", 210.0, 90.0));
    assert!(!hit("Sliced", 350.0, 50.0));
    assert!(hit("Sliced", 305.0, 5.0));

    let image = scene
        .artboard
        .with_artboard(|a| a.find_handle::<Image>("Direct"))
        .unwrap();
    let asset = image
        .with_downcast::<Image, _>(Image::image_asset)
        .flatten()
        .unwrap();
    ImageAsset::set_render_image_occurrence(&asset, None);
    scene.machine.advance_and_apply(0.0);
    assert!(!hit("Direct", 50.0, 50.0));
}
