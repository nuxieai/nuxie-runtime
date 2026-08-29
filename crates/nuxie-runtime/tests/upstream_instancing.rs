//! Direct native-owner ports of all three cases in pinned
//! `tests/unit_tests/runtime/instancing_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory, RecordingRenderer};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
    source::shapes::{clipping_shape::ClippingShape, shape::Shape},
};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn load_file(name: &str) -> (RuntimeFileHandle, RecordingRenderer) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let renderer = factory.borrow().make_renderer();
    let retained = RuntimeFactoryHandle::from_factory(&mut factory)
        .expect("explicit retained RecordingFactory");
    let mut result = ImportResult::Malformed;
    let file = File::import(
        &pinned_fixture(name),
        retained,
        Some(&mut result),
        None,
        None,
    )
    .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    (file, renderer)
}

#[test]
fn cloning_an_ellipse_works() {
    let (file, _renderer) = load_file("circle_clips.riv");
    let source = file.with_file(File::artboard).expect("default artboard");
    let node = source
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Shape>("TopEllipse"))
        .flatten()
        .expect("TopEllipse shape");
    let cloned_node = node.clone_occurrence().expect("individual Shape clone");
    let position = node
        .with_downcast::<Shape, _>(|shape| (shape.base.x(), shape.base.y()))
        .expect("source Shape");
    let cloned_position = cloned_node
        .with_downcast::<Shape, _>(|shape| (shape.base.x(), shape.base.y()))
        .expect("cloned Shape");
    assert_eq!(position.0, cloned_position.0);
    assert_eq!(position.1, cloned_position.1);
    assert!(cloned_node.remove_occurrence());
}

#[test]
fn instancing_artboard_clones_clipped_properties() {
    let (file, mut renderer) = load_file("circle_clips.riv");
    let source = file.with_file(File::artboard).expect("default artboard");
    assert_eq!(
        source.with_downcast::<Artboard, _>(Artboard::is_instance),
        Some(false),
    );
    let instance = file
        .with_file(File::artboard_default)
        .expect("default instance");
    assert!(instance.with_artboard(|artboard| artboard.is_instance()));
    let node = instance
        .with_artboard(|artboard| artboard.find_handle::<Shape>("TopEllipse"))
        .expect("TopEllipse is a Shape");
    let clipping_shapes = node
        .with(|node| {
            node.as_drawable()
                .expect("Shape Drawable")
                .clipping_shapes()
                .to_vec()
        })
        .expect("live TopEllipse");
    assert_eq!(clipping_shapes.len(), 2);
    let source_names: Vec<String> = clipping_shapes
        .iter()
        .map(|clipping| {
            let source = clipping
                .with_downcast::<ClippingShape, _>(ClippingShape::source)
                .flatten()
                .expect("clipping source");
            source
                .with(|source| {
                    source
                        .as_component()
                        .expect("source Component")
                        .base
                        .name()
                        .to_owned()
                })
                .expect("live clipping source")
        })
        .collect();
    assert_eq!(source_names[0], "ClipRect2");
    assert_eq!(source_names[1], "BabyEllipse");

    Artboard::update_components_handle(&instance.core_handle());
    instance.draw(&mut renderer);
}

// Integration tests cannot access LinearAnimation's cfg(test) global counter.
// Native CoreHandle is weak: observe the retirement of these exact authored
// animations without retaining them or inventing a second lifetime graph.
fn deleted_animation_count(animations: &[CoreHandle]) -> usize {
    animations
        .iter()
        .filter(|animation| !animation.is_alive())
        .count()
}

#[test]
fn instancing_artboard_does_not_clone_animations() {
    let (file, _renderer) = load_file("juice.riv");
    let source = file.with_file(File::artboard).expect("default artboard");
    let instance = file
        .with_file(File::artboard_default)
        .expect("default instance");
    let source_animation_count = source
        .with_downcast::<Artboard, _>(Artboard::animation_count)
        .expect("source animation count");
    let instance_animation_count = instance.with_artboard(|artboard| artboard.animation_count());
    assert_eq!(source_animation_count, instance_animation_count);
    assert_eq!(
        source
            .with_downcast::<Artboard, _>(Artboard::first_animation)
            .flatten(),
        instance.with_artboard(|artboard| artboard.first_animation()),
    );

    let animations = source
        .with_downcast::<Artboard, _>(|artboard| artboard.animation_handles().to_vec())
        .expect("authored animation handles");
    assert_eq!(deleted_animation_count(&animations), 0);
    let number_of_animations = source_animation_count;
    drop(instance);
    assert_eq!(deleted_animation_count(&animations), 0);
    drop(source);
    drop(file);
    assert_eq!(deleted_animation_count(&animations), number_of_animations);
}
