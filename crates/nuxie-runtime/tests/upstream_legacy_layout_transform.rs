//! Regression for f41cd8f3's pre-7.3 LayoutComponent transform compatibility gate.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry, layout_component_base::LayoutComponentBase,
        node_base::NodeBase, transform_component_base::TransformComponentBase,
    },
    math::mat2d::Mat2D,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
};

const ROTATION: f32 = 0.37;
const SCALE_X: f32 = 1.75;
const SCALE_Y: f32 = 0.625;

fn pinned_stack_fixture() -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/layout/stack.riv");
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()))
}

fn imported_instance_world_transform(bytes: &[u8]) -> Mat2D {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("stack.riv imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);

    let source = file.with_file(File::artboard).expect("source artboard");
    let (local, source_layout) = source
        .with_downcast::<Artboard, _>(|artboard| {
            artboard
                .objects()
                .iter()
                .enumerate()
                .find_map(|(local, object)| {
                    let object = object.as_ref()?;
                    (object.core_type() == Some(LayoutComponentBase::TYPE_KEY))
                        .then(|| (local, object.clone()))
                })
        })
        .flatten()
        .expect("non-artboard LayoutComponent");

    for (property, value) in [
        (TransformComponentBase::ROTATION_PROPERTY_KEY, ROTATION),
        (TransformComponentBase::SCALE_X_PROPERTY_KEY, SCALE_X),
        (TransformComponentBase::SCALE_Y_PROPERTY_KEY, SCALE_Y),
    ] {
        assert!(CoreRegistry::set_double_handle(
            &source_layout,
            i32::from(property),
            value,
        ));
    }

    let instance = Artboard::instance_from_handle(&source).expect("cloned artboard instance");
    instance.update_pass(true);
    let cloned_layout = instance
        .with_artboard(|artboard| artboard.objects().get(local).cloned().flatten())
        .expect("cloned LayoutComponent");
    cloned_layout
        .with(|object| {
            *object
                .as_world_transform_component()
                .expect("world-transform component")
                .world_transform()
        })
        .expect("live cloned LayoutComponent")
}

fn assert_near(actual: f32, expected: f32, label: &str) {
    assert!(
        (actual - expected).abs() <= 0.0001,
        "{label}: expected {expected}, got {actual}"
    );
}

fn imported_offset_layout(name: &str) -> (RuntimeFileHandle, CoreHandle, CoreHandle) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets/layout")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let artboard = file.with_file(File::artboard).expect("source artboard");
    assert!(Artboard::update_pass_handle(&artboard, true));
    let layout = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| {
                    object
                        .with(|object| {
                            object
                                .as_layout_component()
                                .is_some_and(|layout| layout.layout_width() == 50.0)
                        })
                        .unwrap_or(false)
                })
                .cloned()
        })
        .flatten()
        .expect("50x50 offset layout");
    (file, artboard, layout)
}

#[test]
fn pre_7_3_layout_transform_stays_translation_only_after_clone() {
    let current = pinned_stack_fixture();
    assert_eq!(
        current.get(..6),
        Some(b"RIVE\x07\x03".as_slice()),
        "fixture must have the exact 7.3 single-byte header"
    );
    let mut legacy = current.clone();
    legacy[5] = 2;
    assert_eq!(&legacy[..5], &current[..5]);
    assert_eq!(&legacy[6..], &current[6..]);

    let legacy_world = imported_instance_world_transform(&legacy);
    let current_world = imported_instance_world_transform(&current);

    let mut authored_transform = Mat2D::from_rotation(ROTATION);
    authored_transform.scale_by_values(SCALE_X, SCALE_Y);
    let expected_current = legacy_world * authored_transform;
    for index in 0..4 {
        assert_near(
            current_world.values()[index],
            expected_current.values()[index],
            "7.3 composed rotation/scale",
        );
    }
    assert!(
        legacy_world.values()[..4]
            .iter()
            .zip(&current_world.values()[..4])
            .any(|(legacy, current)| (legacy - current).abs() > 0.0001),
        "7.2 must ignore the stored rotation/scale that 7.3 composes"
    );
}

#[test]
fn current_layout_xy_offsets_its_solved_slot_and_round_trips() {
    let (_file, artboard, layout) = imported_offset_layout("transform_offset.riv");
    layout.with(|object| {
        let layout = object.as_layout_component().expect("LayoutComponent");
        assert_eq!(layout.layout_x(), 0.0);
        assert_eq!(layout.layout_y(), 0.0);
        assert_near(layout.base.world_transform()[4], 30.0, "world x offset");
        assert_near(layout.base.world_transform()[5], 12.0, "world y offset");
        assert_eq!(layout.origin_offset().x, 0.0);
        assert_eq!(layout.local_anchor().x, 0.0);
    });
    assert_eq!(
        CoreRegistry::get_double_handle(&layout, i32::from(NodeBase::X_PROPERTY_KEY)),
        Some(30.0)
    );
    assert_eq!(
        CoreRegistry::get_double_handle(&layout, i32::from(NodeBase::Y_PROPERTY_KEY)),
        Some(12.0)
    );
    assert!(CoreRegistry::set_double_handle(
        &layout,
        i32::from(NodeBase::X_PROPERTY_KEY),
        45.0,
    ));
    Artboard::update_pass_handle(&artboard, true);
    layout.with(|object| {
        let layout = object.as_layout_component().expect("LayoutComponent");
        assert_eq!(layout.layout_x(), 0.0);
        assert_near(layout.base.transform()[4], 45.0, "own transform x");
        assert_near(layout.base.transform()[5], 12.0, "own transform y");
        assert_near(layout.composed_translation().x, 45.0, "composed x");
        assert_near(layout.composed_translation().y, 12.0, "composed y");
        assert_eq!(layout.local_bounds().left(), 0.0);
        assert_near(
            layout.world_bounds().left(),
            layout.base.world_transform()[4],
            "world bounds left",
        );
    });
    assert_eq!(
        CoreRegistry::get_double_handle(
            &layout,
            i32::from(NodeBase::COMPUTED_LOCAL_X_PROPERTY_KEY),
        ),
        Some(45.0),
    );
    artboard
        .with_downcast::<Artboard, _>(|artboard| {
            assert_eq!(artboard.pivot_origin_x(), artboard.origin_x());
            assert_eq!(artboard.local_anchor().x, 0.0);
        })
        .expect("Artboard");
}

#[test]
fn pre_7_3_layout_ignores_stored_xy_offset() {
    let (_file, _artboard, layout) = imported_offset_layout("transform_offset_legacy.riv");
    layout.with(|object| {
        let layout = object.as_layout_component().expect("LayoutComponent");
        assert_eq!(layout.layout_x(), 0.0);
        assert_near(layout.base.world_transform()[4], 0.0, "legacy world x");
        assert_near(layout.base.world_transform()[5], 0.0, "legacy world y");
        assert_ne!(layout.composed_translation().x, 30.0);
        assert_ne!(layout.composed_translation().y, 12.0);
    });
}
