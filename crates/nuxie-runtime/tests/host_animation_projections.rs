use std::path::{Path, PathBuf};

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{ArtboardInstance, File, RuntimeFactoryHandle};

fn fixture_path(name: &str) -> PathBuf {
    let runtime_dir = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime_dir)
        .join("tests/unit_tests/assets")
        .join(name)
}

fn import_host_artboard(fixture: &str) -> (PersistentFactory<RecordingFactory>, ArtboardInstance) {
    let bytes = std::fs::read(fixture_path(fixture)).expect("pinned fixture bytes");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("fixture imports");
    let artboard = ArtboardInstance::from_native(file, 0).expect("default artboard instance");
    (factory, artboard)
}

#[test]
fn retained_linear_step_is_atomic_and_observes_keyed_events_in_source_order() {
    let (_factory, mut artboard) = import_host_artboard("looping_timeline_events.riv");
    let mut animation = artboard
        .linear_animation_instance(0)
        .expect("timeline animation");

    let first = artboard
        .advance_linear_animation_instance(&mut animation, 0.1)
        .expect("animation belongs to its artboard");
    assert!(first.changed);
    assert!(first.keep_going);
    assert_eq!(animation.time(), 0.1);
    assert_eq!(first.reported_events.len(), 1);
    assert_eq!(first.reported_events[0].event_local_index(), Some(1));
    assert_eq!(first.reported_events[0].seconds_delay(), 0.1);

    let second = artboard
        .advance_linear_animation_instance(&mut animation, 0.32)
        .expect("animation belongs to its artboard");
    assert!(second.changed);
    assert!(second.keep_going);
    assert_eq!(animation.time(), 0.42);
    assert_eq!(second.reported_events.len(), 1);
    assert_eq!(second.reported_events[0].event_local_index(), Some(1));
    assert!((second.reported_events[0].seconds_delay() - 0.003333_330_2).abs() < 1e-6);

    let spanning = artboard
        .advance_linear_animation_instance(&mut animation, 1.01)
        .expect("animation belongs to its artboard");
    assert_eq!(
        spanning
            .reported_events
            .iter()
            .map(|event| event.event_local_index())
            .collect::<Vec<_>>(),
        vec![Some(1), Some(1), Some(1)]
    );
    let spanning_delays = spanning
        .reported_events
        .iter()
        .map(|event| event.seconds_delay())
        .collect::<Vec<_>>();
    assert!((spanning_delays[0] - 0.43).abs() < 1e-6);
    assert!((spanning_delays[1] - 0.43).abs() < 1e-6);
    assert!((spanning_delays[2] - 0.013333_261).abs() < 1e-6);
}

#[test]
fn absolute_linear_apply_settles_the_translated_component_pass() {
    let (_factory, mut artboard) = import_host_artboard("quantize_test.riv");
    let mut animation = artboard
        .linear_animation_instance(0)
        .expect("quantized animation");

    let changed = artboard
        .apply_linear_animation_instance_at(&mut animation, 0.5, 1.0)
        .expect("animation belongs to its artboard");

    assert!(changed);
    assert_eq!(animation.time(), 0.5);
    assert!(
        !artboard.update_components(),
        "absolute apply already completes the exact translated update pass"
    );
}

#[test]
fn animation_projection_rejects_an_instance_owned_by_another_artboard() {
    let (_factory_a, mut artboard_a) = import_host_artboard("quantize_test.riv");
    let (_factory_b, artboard_b) = import_host_artboard("quantize_test.riv");
    let mut animation_b = artboard_b
        .linear_animation_instance(0)
        .expect("second artboard animation");

    assert!(
        artboard_a
            .advance_linear_animation_instance(&mut animation_b, 0.1)
            .is_err()
    );
    assert!(
        artboard_a
            .apply_linear_animation_instance_at(&mut animation_b, 0.5, 1.0)
            .is_err()
    );
}
