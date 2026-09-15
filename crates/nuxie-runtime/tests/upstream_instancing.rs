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

#[test]
fn semantics_follow_registered_custom_clip_paths() {
    use nuxie_runtime::source::{
        generated::core_registry::CoreRegistry,
        generated::shapes::clipping_shape_base::ClippingShapeBase,
        semantic::semantic_provider::{semantic_bounds, semantic_source_is_visible},
    };

    let (file, _renderer) = load_file("circle_clips.riv");
    let instance = file.with_file(File::artboard_default).expect("instance");
    Artboard::update_components_handle(&instance.core_handle());
    let node = instance
        .with_artboard(|artboard| artboard.find_handle::<Shape>("TopEllipse"))
        .expect("TopEllipse");
    let clips = node
        .with(|node| node.as_drawable().unwrap().clipping_shapes().to_vec())
        .unwrap();
    assert_eq!(clips.len(), 2);
    for clip in &clips {
        assert!(CoreRegistry::set_bool_handle(
            clip,
            ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
            false
        ));
    }
    let unclipped = semantic_bounds(Some(&node));
    assert!(
        !unclipped.is_empty_or_nan(),
        "fixture must provide semantic geometry"
    );
    assert!(semantic_source_is_visible(&node));

    let clip = &clips[0];
    assert!(CoreRegistry::set_bool_handle(
        clip,
        ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
        true
    ));
    clip.with_downcast_mut::<ClippingShape, _>(|clip| {
        clip.path().expect("initialized rendered clip").rewind();
    })
    .unwrap();
    assert!(
        semantic_bounds(Some(&node)).is_empty_or_nan(),
        "empty rendered clip must hide the target"
    );
    assert!(!semantic_source_is_visible(&node));

    let midpoint = (unclipped.min_x + unclipped.max_x) * 0.5;
    clip.with_downcast_mut::<ClippingShape, _>(|clip| {
        clip.path().unwrap().add_rect(
            nuxie_runtime::source::math::aabb::Aabb::new(
                midpoint,
                unclipped.min_y,
                unclipped.max_x,
                unclipped.max_y,
            ),
            nuxie_runtime::source::math::path_types::PathDirection::Clockwise,
        );
    })
    .unwrap();
    let partial = semantic_bounds(Some(&node));
    assert!((partial.min_x - midpoint).abs() < 0.01);
    assert!((partial.max_x - unclipped.max_x).abs() < 0.01);
    assert!((partial.min_y - unclipped.min_y).abs() < 0.01);
    assert!((partial.max_y - unclipped.max_y).abs() < 0.01);
    assert!(semantic_source_is_visible(&node));

    assert!(CoreRegistry::set_bool_handle(
        clip,
        ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
        false
    ));
    assert_eq!(
        semantic_bounds(Some(&node)),
        unclipped,
        "disabled clip must restore the unchanged target"
    );
    assert!(semantic_source_is_visible(&node));
}

#[test]
fn custom_clip_changes_refresh_stationary_semantic_snapshots() {
    use nuxie_runtime::source::{
        generated::{
            component_base::ComponentBase, core_registry::CoreRegistry, node_base::NodeBase,
            shapes::clipping_shape_base::ClippingShapeBase,
        },
        semantic::{
            semantic_data::SemanticData,
            semantic_manager::{RuntimeSemanticManagerHandle, SemanticManager},
        },
    };
    let (file, _renderer) = load_file("circle_clips.riv");
    let source = file.with_file(File::artboard).unwrap();
    let target_index = source
        .with_downcast::<Artboard, _>(|artboard| {
            artboard.object_index(&artboard.find_handle::<Shape>("TopEllipse").unwrap())
        })
        .unwrap();
    let data = file.with_file(|file| file.core_arena().insert(SemanticData::default()));
    assert!(CoreRegistry::set_uint_handle(
        &data,
        ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
        target_index as u32
    ));
    data.with_downcast_mut::<SemanticData, _>(|data| {
        data.set_label("Clipped control".to_owned());
        data.set_role(1);
    });
    source.with_downcast_mut::<Artboard, _>(|artboard| artboard.add_object(Some(data)));
    let instance = file.with_file(File::artboard_default).unwrap();
    instance.advance_default(0.0);
    let node = instance
        .with_artboard(|artboard| artboard.find_handle::<Shape>("TopEllipse"))
        .unwrap();
    let clips = node
        .with(|object| object.as_drawable().unwrap().clipping_shapes().to_vec())
        .unwrap();
    for clip in &clips {
        assert!(CoreRegistry::set_bool_handle(
            clip,
            ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
            false
        ));
    }
    let manager = RuntimeSemanticManagerHandle::new(SemanticManager::new());
    instance.build_semantic_tree(Some(manager.clone()), None);
    let initial = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    let target = initial
        .iter()
        .find(|node| node.label == "Clipped control")
        .unwrap();
    let id = target.id;
    let original = target.bounds();
    let transform = node
        .with(|object| *object.as_node().unwrap().world_transform())
        .unwrap();
    let clip = &clips[0];
    let clip_source = clip
        .with_downcast::<ClippingShape, _>(ClippingShape::source)
        .flatten()
        .unwrap();
    let x_key = NodeBase::X_PROPERTY_KEY.into();
    let x = CoreRegistry::get_double_handle(&clip_source, x_key).unwrap();
    assert!(CoreRegistry::set_double_handle(
        &clip_source,
        x_key,
        x + 10000.0
    ));
    assert!(CoreRegistry::set_bool_handle(
        clip,
        ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
        true
    ));
    instance.advance_default(0.0);
    let path_bounds = clip
        .with_downcast_mut::<ClippingShape, _>(|clip| clip.path().unwrap().raw_path().bounds())
        .unwrap();
    assert!(
        path_bounds.min_x > original.max_x,
        "rendered clip must move completely beyond the target"
    );
    assert_eq!(
        node.with(|object| *object.as_node().unwrap().world_transform())
            .unwrap(),
        transform
    );
    let hidden = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    assert!(
        !hidden.iter().any(|node| node.id == id),
        "displaced custom clip must retire the stationary control"
    );
    assert!(CoreRegistry::set_double_handle(&clip_source, x_key, x));
    instance.advance_default(0.0);
    let returned = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    assert!(
        returned.iter().any(|node| node.id == id),
        "moving the enabled clip back restores the same control"
    );
    assert!(CoreRegistry::set_double_handle(
        &clip_source,
        x_key,
        x + 10000.0
    ));
    instance.advance_default(0.0);
    assert!(
        !manager.with_semantic_manager_mut(|manager| manager
            .snapshot()
            .iter()
            .any(|node| node.id == id))
    );
    assert!(CoreRegistry::set_bool_handle(
        clip,
        ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
        false
    ));
    instance.advance_default(0.0);
    let restored = manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec());
    assert_eq!(
        restored
            .iter()
            .find(|node| node.id == id)
            .expect("same control returns")
            .bounds(),
        original
    );
    use nuxie_runtime::source::{
        math::{aabb::Aabb, path_types::PathDirection},
        semantic::semantic_provider::{SemanticGeometryError, validate_semantic_geometry},
    };
    assert_eq!(validate_semantic_geometry(&instance.core_handle()), Ok(()));
    clip.with_downcast_mut::<ClippingShape, _>(|clip| {
        let path = clip.path().unwrap();
        path.rewind();
        for _ in 0..5000 {
            path.add_rect(
                Aabb::new(
                    original.min_x,
                    original.min_y,
                    original.max_x,
                    original.max_y,
                ),
                PathDirection::Clockwise,
            );
        }
    });
    assert!(CoreRegistry::set_bool_handle(
        clip,
        ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
        true
    ));
    instance.advance_default(0.0);
    assert_eq!(
        validate_semantic_geometry(&instance.core_handle()),
        Err(SemanticGeometryError::LimitExceeded),
        "an excluded control with an unsupported clip must fail capture validation"
    );
    assert!(CoreRegistry::set_bool_handle(
        clip,
        ClippingShapeBase::IS_VISIBLE_PROPERTY_KEY.into(),
        false
    ));
    instance.advance_default(0.0);
    assert_eq!(
        validate_semantic_geometry(&instance.core_handle()),
        Ok(()),
        "disabled over-budget clip must not prevent recovery"
    );
}
