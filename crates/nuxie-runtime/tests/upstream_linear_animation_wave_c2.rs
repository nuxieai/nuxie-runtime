//! Native owner ports of cases 003, 005, and 006 in pinned
//! `tests/unit_tests/runtime/linear_animation_test.cpp`.

use std::{cell::RefCell, path::PathBuf};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        keyed_callback_reporter::KeyedCallbackReporter,
        keyed_object::{KeyedObject, KeyedObjectContext},
        linear_animation::LinearAnimation,
    },
    core::CoreArena,
    core_context::CoreContext,
    generated::{
        animation::{
            keyed_object_base::KeyedObjectBase, linear_animation_base::LinearAnimationBase,
        },
        core_registry::CoreRegistry,
    },
    importers::{import_stack::ImportStack, linear_animation_importer::LinearAnimationImporter},
    shapes::shape::Shape,
    status_code::StatusCode,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn load_file(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("{name} imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    file
}

#[test]
fn wave_c2_linear_definition_003_quantize_goes_to_whole_frames() {
    let file = load_file("quantize_test.riv");
    // The pinned case deliberately applies to the imported source Artboard.
    let artboard = file.with_file(File::artboard).expect("source artboard");
    let animation = artboard
        .with_downcast::<Artboard, _>(Artboard::first_animation)
        .flatten()
        .expect("first animation");
    assert!(
        animation
            .with_downcast::<LinearAnimation, _>(|animation| animation.quantize())
            .expect("definition")
    );
    let shapes = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_all_handles::<Shape>())
        .expect("source");
    assert_eq!(shapes.len(), 1);
    let ellipse = &shapes[0];
    let apply = |time| {
        animation
            .with_downcast_mut::<LinearAnimation, _>(|animation| {
                artboard
                    .with_downcast_mut::<Artboard, _>(|artboard| {
                        animation.apply(artboard, time, 1.0, None)
                    })
                    .expect("source Artboard");
            })
            .expect("mutable animation definition");
    };

    apply(0.0);
    assert_eq!(
        ellipse.with_downcast::<Shape, _>(|shape| shape.base.x()),
        Some(0.0)
    );
    apply(0.5);
    assert_eq!(
        ellipse.with_downcast::<Shape, _>(|shape| shape.base.x()),
        Some(160.0)
    );
    assert!(CoreRegistry::set_bool_handle(
        &animation,
        i32::from(LinearAnimationBase::QUANTIZE_PROPERTY_KEY),
        false
    ));
    apply(0.5);
    assert_eq!(
        ellipse.with_downcast::<Shape, _>(|shape| shape.base.x()),
        Some(200.0)
    );
}

/// Instrument only resolution calls; every answer comes from the actual source
/// Artboard. This exercises real KeyedObject MissingObject/Ok behavior instead
/// of replacing its lifecycle with the C++ test's status-returning subclass.
struct CountingContext<'a> {
    artboard: &'a mut Artboard,
    resolutions: RefCell<Vec<u32>>,
}

impl CoreContext for CountingContext<'_> {
    fn core_arena(&self) -> &CoreArena {
        self.artboard.core_arena()
    }

    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.artboard.resolve_handle(id)
    }
}

impl KeyedObjectContext for CountingContext<'_> {
    fn resolves_object(&self, id: u32) -> bool {
        self.resolutions.borrow_mut().push(id);
        self.artboard.resolves_object(id)
    }

    fn resolve_object(&mut self, id: u32) -> Option<CoreHandle> {
        self.artboard.resolve_object(id)
    }

    fn object_supports_property(&self, id: u32, key: u32) -> bool {
        self.artboard.object_supports_property(id, key)
    }

    fn overrides_keyed_interpolation(&self, object: &CoreHandle, key: u32) -> bool {
        self.artboard.overrides_keyed_interpolation(object, key)
    }
}

#[test]
fn wave_c2_linear_definition_005_missing_keyed_object_does_not_stop_initialization() {
    let file = load_file("quantize_test.riv");
    let artboard = file.with_file(File::artboard).expect("source artboard");
    artboard
        .with_downcast::<Artboard, _>(|artboard| {
            assert!(artboard.resolve_handle(99).is_none());
            assert!(artboard.resolve_handle(1).is_some());
        })
        .expect("actual missing/valid targets");

    let arena = CoreArena::default();
    let animation = arena.insert(LinearAnimation::default());
    let missing = arena.insert(KeyedObject::default());
    let valid = arena.insert(KeyedObject::default());
    for (object, id) in [(&missing, 99), (&valid, 1)] {
        assert!(object.is_type_of(KeyedObjectBase::TYPE_KEY));
        assert!(CoreRegistry::set_uint_handle(
            object,
            i32::from(KeyedObjectBase::OBJECT_ID_PROPERTY_KEY),
            id
        ));
        assert_eq!(
            object.with_downcast::<KeyedObject, _>(|object| object.object_id()),
            Some(id)
        );
    }

    // Preserve the prior test's successful-import assertions using the actual
    // native importer, before initialization removes the missing target.
    let mut stack = ImportStack::default();
    assert_eq!(
        stack.make_latest(
            LinearAnimationBase::TYPE_KEY,
            Some(Box::new(LinearAnimationImporter::new(animation.clone())))
        ),
        StatusCode::Ok
    );
    for object in [&missing, &valid] {
        assert_eq!(
            object.with_downcast_mut::<KeyedObject, _>(|object| object.import(&mut stack)),
            Some(StatusCode::Ok)
        );
    }
    assert_eq!(stack.resolve(), StatusCode::Ok);

    artboard
        .with_downcast_mut::<Artboard, _>(|artboard| {
            let mut context = CountingContext {
                artboard,
                resolutions: RefCell::new(Vec::new()),
            };
            let status = animation
                .with_downcast_mut::<LinearAnimation, _>(|animation| {
                    animation.on_added_dirty(&mut context)
                })
                .expect("native LinearAnimation lifecycle");
            assert_eq!(status, StatusCode::Ok);
            // Each real owner is visited once, in source order, even after failure.
            assert_eq!(*context.resolutions.borrow(), [99, 1]);
        })
        .expect("source context");
    let retained = animation
        .with_downcast::<LinearAnimation, _>(|animation| animation.keyed_objects().to_vec())
        .expect("native keyed objects");
    assert_eq!(retained.len(), 1);
    assert_eq!(retained[0], valid);
    assert_eq!(
        retained[0].with_downcast::<KeyedObject, _>(|object| object.object_id()),
        Some(1)
    );
}

#[derive(Default)]
struct TestReporter {
    objects: Vec<u32>,
}

impl KeyedCallbackReporter for TestReporter {
    fn report_keyed_callback(&mut self, object_id: u32, _property_key: u32, _elapsed_seconds: f32) {
        self.objects.push(object_id);
    }
}

#[test]
fn wave_c2_linear_definition_006_looping_timeline_events_load_and_report() {
    let file = load_file("looping_timeline_events.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("artboard instance");
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.animation_count()),
        1
    );
    let mut animation = artboard.animation_at(0).expect("animation instance");
    let mut reporter = TestReporter::default();
    for (seconds, expected_time, expected_count) in [
        (0.1, 0.1, 1),
        (0.32, 0.42, 2),
        (0.3, 0.72, 2),
        (0.28, 0.0, 3),
    ] {
        animation.advance(seconds, Some(&mut reporter));
        assert_eq!(animation.time(), expected_time);
        assert_eq!(reporter.objects.len(), expected_count);
    }
    animation.advance(1.01, Some(&mut reporter));
    // The pinned final assertion uses Catch Approx with its default scale 0.
    let expected_time = 0.01_f32;
    assert!((animation.time() - expected_time).abs() <= f32::EPSILON * 100.0 * expected_time.abs());
    assert_eq!(reporter.objects.len(), 7);
}
