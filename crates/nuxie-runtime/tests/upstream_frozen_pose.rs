//! Direct translation of frozen_pose_test.cpp at upstream be93c2ce.
#![cfg(feature = "testing")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    RuntimeFactoryHandle,
    source::{
        animation::state_machine_instance::{
            RuntimeStateMachineInstanceHandle, StateMachineInstance,
        },
        artboard::RuntimeArtboardInstanceHandle,
        artboard_component_list::ArtboardComponentList,
        core::CoreHandle,
        file::{File, RuntimeFileHandle},
        generated::{
            core_registry::CoreRegistry, transform_component_base::TransformComponentBase,
            world_transform_component_base::WorldTransformComponentBase,
        },
        shapes::{
            paint::{fill::Fill, solid_color::SolidColor},
            shape::Shape,
        },
        viewmodel::{
            viewmodel_instance::ViewModelInstance,
            viewmodel_instance_boolean::ViewModelInstanceBoolean,
        },
    },
};

const RED: i32 = 0xffff0000u32 as i32;
const BLUE: i32 = 0xff0000ffu32 as i32;
const GREEN: i32 = 0xff00ff00u32 as i32;

fn read_file(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .expect("RIVE_RUNTIME_DIR points to the pinned upstream fixtures");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}

fn rectangle_color(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    let shape = artboard
        .with_artboard(|a| a.find_handle::<Shape>("rectangle"))
        .unwrap();
    let fill = shape
        .with(|shape| shape.as_container_component().unwrap().children()[1].clone())
        .unwrap();
    assert!(fill.with_downcast::<Fill, _>(|_| ()).is_some());
    let paint = fill
        .with(|fill| fill.as_shape_paint().unwrap().paint().unwrap())
        .unwrap();
    assert!(paint.with_downcast::<SolidColor, _>(|_| ()).is_some());
    paint
}

fn color_value(color: &CoreHandle) -> i32 {
    color
        .with_downcast::<SolidColor, _>(|color| color.base.color_value())
        .unwrap()
}

fn set_color(color: &CoreHandle, value: i32) {
    color
        .with_downcast_mut::<SolidColor, _>(|color| color.set_color_value(value))
        .unwrap();
}

fn square(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|a| a.find_handle::<Shape>("square"))
        .unwrap()
}

fn opacity(shape: &CoreHandle) -> f32 {
    shape
        .with_downcast::<Shape, _>(|shape| shape.opacity())
        .unwrap()
}

fn rotate(artboard: &RuntimeArtboardInstanceHandle, value: f32) {
    assert!(CoreRegistry::set_double_handle(
        &artboard.core_handle(),
        i32::from(TransformComponentBase::ROTATION_PROPERTY_KEY),
        value
    ));
}

struct FrozenPoseScene {
    _file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    view_model: CoreHandle,
    machine: RuntimeStateMachineInstanceHandle,
}

impl FrozenPoseScene {
    fn new() -> Self {
        let file = read_file("frozen_pose.riv");
        let artboard = file
            .with_file(File::artboard_default)
            .unwrap()
            .instance()
            .unwrap();
        let view_model = file
            .with_file(|f| {
                f.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .unwrap();
        let machine = artboard.default_state_machine_handle().unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(view_model.clone()));
        machine.advance_and_apply(0.0);
        Self {
            _file: file,
            artboard,
            view_model,
            machine,
        }
    }

    fn advance(&self, frames: usize) {
        for _ in 0..frames {
            self.machine.advance_and_apply(1.0 / 60.0);
        }
    }

    fn frozen(&self, layer: usize) -> bool {
        self.machine.with_instance(|m| m.is_layer_frozen(layer))
    }
}

#[test]
fn a_layer_resting_on_a_pose_stops_reapplying_it() {
    let scene = FrozenPoseScene::new();
    assert_eq!(color_value(&rectangle_color(&scene.artboard)), RED);
    scene.advance(30);
    assert!(scene.frozen(0));
    let poses = StateMachineInstance::frozen_pose_advances();
    scene.advance(10);
    assert!(StateMachineInstance::frozen_pose_advances() > poses);
    assert_eq!(color_value(&rectangle_color(&scene.artboard)), RED);
}

#[test]
fn a_layer_holding_a_finished_animation_stops_reapplying_it() {
    let scene = FrozenPoseScene::new();
    scene.advance(30);
    let square = square(&scene.artboard);
    assert_eq!(opacity(&square), 1.0);
    assert!(scene.frozen(2));
    assert!(CoreRegistry::set_double_handle(
        &square,
        i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
        0.25
    ));
    assert!(!scene.frozen(2));
    scene.advance(1);
    assert_eq!(opacity(&square), 1.0);
    scene.advance(2);
    assert!(scene.frozen(2));
}

#[test]
fn a_write_to_the_artboard_makes_a_frozen_pose_reapply() {
    let scene = FrozenPoseScene::new();
    scene.advance(30);
    let color = rectangle_color(&scene.artboard);
    assert!(scene.frozen(0));
    set_color(&color, GREEN);
    assert!(!scene.frozen(0));
    scene.advance(1);
    assert_eq!(color_value(&color), RED);
}

#[test]
fn rotating_the_artboard_itself_makes_a_frozen_pose_reapply() {
    let scene = FrozenPoseScene::new();
    scene.advance(30);
    assert!(scene.frozen(0));
    rotate(&scene.artboard, 0.5);
    assert!(!scene.frozen(0));
    scene.advance(1);
    assert_eq!(scene.artboard.with_artboard(|a| a.rotation()), 0.0);
}

#[test]
fn a_frozen_pose_reapplies_once_a_later_layer_stops_overriding_it() {
    let scene = FrozenPoseScene::new();
    scene.advance(30);
    let flash = scene
        .view_model
        .with_downcast::<ViewModelInstance, _>(|vm| vm.property_value_named("flash"))
        .flatten()
        .unwrap();
    flash
        .with_downcast_mut::<ViewModelInstanceBoolean, _>(|flash| flash.set_value(true))
        .unwrap();
    scene.advance(1);
    assert_eq!(color_value(&rectangle_color(&scene.artboard)), BLUE);
    flash
        .with_downcast_mut::<ViewModelInstanceBoolean, _>(|flash| flash.set_value(false))
        .unwrap();
    scene.advance(30);
    assert_eq!(color_value(&rectangle_color(&scene.artboard)), RED);
    assert!(scene.frozen(0));
}

#[test]
fn writes_to_a_quiet_list_row_bring_its_pose_back() {
    let file = read_file("frozen_pose_rows.riv");
    let artboard = file.with_file(|f| f.artboard_named("rows_host")).unwrap();
    let machine = artboard.default_state_machine_handle().unwrap();
    let view_model = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(artboard.core_handle()));
    machine.with_instance_mut(|m| m.bind_view_model_instance(view_model));
    machine.advance_and_apply(0.0);
    let lists = artboard.with_artboard(|a| a.find_all_handles::<ArtboardComponentList>());
    assert_eq!(lists.len(), 1);
    let list = &lists[0];
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(|l| l.artboard_count())
            .unwrap(),
        2
    );
    let skips = ArtboardComponentList::quiet_row_skips();
    for _ in 0..30 {
        machine.advance_and_apply(1.0 / 60.0);
    }
    assert!(ArtboardComponentList::quiet_row_skips() > skips);
    let row = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
        .flatten()
        .unwrap();
    let color = rectangle_color(&row);
    assert_eq!(color_value(&color), RED);
    set_color(&color, GREEN);
    machine.advance_and_apply(1.0 / 60.0);
    assert_eq!(color_value(&color), RED);
    for _ in 0..10 {
        machine.advance_and_apply(1.0 / 60.0);
    }
    rotate(&row, 0.5);
    machine.advance_and_apply(1.0 / 60.0);
    assert_eq!(row.with_artboard(|a| a.rotation()), 0.0);
}
