//! Regression for f41cd8f3's pre-7.3 LayoutComponent transform compatibility gate.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry, layout_component_base::LayoutComponentBase,
        transform_component_base::TransformComponentBase,
    },
    math::mat2d::Mat2D,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};

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
