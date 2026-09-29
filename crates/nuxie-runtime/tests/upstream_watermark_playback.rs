//! Translation of all cases in `tests/unit_tests/runtime/watermark_playback_test.cpp`
//! at upstream `d8727299e08ba8517887a8a9f0fae80819208294`.

use std::{path::PathBuf, sync::Mutex, time::Duration};

use nuxie_render_api::{
    BlendMode, ImageSampler, Mat2D, PersistentFactory, RecordingFactory, RenderBuffer, RenderImage,
    RenderPaint, RenderPath, Renderer,
};
use nuxie_runtime::source::{
    artboard::RuntimeArtboardInstanceHandle,
    assets::manifest_asset::ManifestAsset,
    generated::{core_registry::CoreRegistry, node_base::NodeBase},
    node::Node,
    watermark::Watermark,
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

// Catch executes these cases sequentially; preserve that around File's global
// deterministic-mode switch in Rust's concurrent test harness.
static PLAYBACK_LOCK: Mutex<()> = Mutex::new(());

struct DeterministicTime;

impl DeterministicTime {
    fn new() -> Self {
        File::set_deterministic_mode(true);
        Self
    }
}

impl Drop for DeterministicTime {
    fn drop(&mut self) {
        File::set_deterministic_mode(false);
    }
}

// Like upstream NoOpRenderer with drawPath overridden, retain only path
// identity. Pointer values are never dereferenced or used to retain owners.
#[derive(Default)]
struct PathRecordingRenderer {
    paths: Vec<usize>,
}

impl PathRecordingRenderer {
    fn saw_any_of(&self, others: &[usize]) -> bool {
        self.paths.iter().any(|path| others.contains(path))
    }
}

impl Renderer for PathRecordingRenderer {
    fn save(&mut self) {}
    fn restore(&mut self) {}
    fn transform(&mut self, _: Mat2D) {}
    fn draw_path(&mut self, path: &dyn RenderPath, _: &dyn RenderPaint) {
        self.paths
            .push(path as *const dyn RenderPath as *const () as usize);
    }
    fn clip_path(&mut self, _: &dyn RenderPath) {}
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}

fn read_rive_file(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    File::import(&bytes, factory, None, None, None).expect("fixture imports")
}

fn record_draw(artboard: &RuntimeArtboardInstanceHandle) -> Vec<usize> {
    let mut renderer = PathRecordingRenderer::default();
    artboard.draw(&mut renderer);
    renderer.paths
}

fn record_draw_internal(artboard: &RuntimeArtboardInstanceHandle) -> Vec<usize> {
    let mut renderer = PathRecordingRenderer::default();
    artboard.draw_internal(&mut renderer);
    renderer.paths
}

fn node_positions(artboard: &RuntimeArtboardInstanceHandle) -> Vec<f32> {
    let nodes = artboard.with_artboard(|artboard| artboard.find_all_handles::<Node>());
    let mut positions = Vec::new();
    for node in nodes {
        positions.push(
            CoreRegistry::get_double_handle(&node, NodeBase::X_PROPERTY_KEY.into())
                .expect("Node x"),
        );
        positions.push(
            CoreRegistry::get_double_handle(&node, NodeBase::Y_PROPERTY_KEY.into())
                .expect("Node y"),
        );
    }
    positions
}

fn make_watermark(file: &RuntimeFileHandle, paths: Option<&mut Vec<usize>>) -> Box<Watermark> {
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);
    if let Some(paths) = paths {
        *paths = record_draw(&artboard);
    }
    Box::new(Watermark::new(artboard, state_machine))
}

fn has_watermark(artboard: &RuntimeArtboardInstanceHandle) -> bool {
    artboard.with_artboard(|artboard| artboard.watermark().is_some())
}

fn watermark_is_playing(artboard: &RuntimeArtboardInstanceHandle) -> bool {
    artboard.with_artboard(|artboard| artboard.watermark().expect("watermark").is_playing())
}

#[test]
fn a_watermark_holds_back_the_artboard_it_is_attached_to() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    let _deterministic = DeterministicTime::new();
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);

    let host_paths = record_draw(&artboard);
    assert!(!host_paths.is_empty());
    let frozen_positions = node_positions(&artboard);
    assert!(!frozen_positions.is_empty());

    let mut watermark_paths = Vec::new();
    let watermark = make_watermark(&file, Some(&mut watermark_paths));
    artboard.with_artboard_mut(|artboard| artboard.set_watermark(Some(watermark)));
    assert!(has_watermark(&artboard));
    assert!(!watermark_paths.is_empty());

    for _ in 0..10 {
        assert!(state_machine.advance_and_apply(0.016));
        assert!(has_watermark(&artboard));
        assert!(watermark_is_playing(&artboard));
        let mut renderer = PathRecordingRenderer::default();
        artboard.draw(&mut renderer);
        assert!(!renderer.paths.is_empty());
        assert!(!renderer.saw_any_of(&host_paths));
        assert_eq!(node_positions(&artboard), frozen_positions);
    }
}

#[test]
fn the_artboard_takes_over_once_the_watermark_is_done() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    let _deterministic = DeterministicTime::new();
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);

    let host_paths = record_draw(&artboard);
    let frozen_positions = node_positions(&artboard);
    let watermark = make_watermark(&file, None);
    artboard.with_artboard_mut(|artboard| artboard.set_watermark(Some(watermark)));

    let control = file
        .with_file(File::artboard_default)
        .expect("control artboard");
    let control_state_machine = control.state_machine_at(0).expect("control state machine");
    control_state_machine.advance_and_apply(0.0);

    for _ in 0..800 {
        if !has_watermark(&artboard) {
            break;
        }
        state_machine.advance_and_apply(0.016);
    }
    assert!(!has_watermark(&artboard));

    // The handover frame already advanced the host by one real frame.
    let mut host_frames = 1;
    for _ in 0..60 {
        state_machine.advance_and_apply(0.016);
        host_frames += 1;
    }
    for _ in 0..host_frames {
        control_state_machine.advance_and_apply(0.016);
    }
    let control_positions = node_positions(&control);
    assert_ne!(control_positions, frozen_positions);
    assert_eq!(node_positions(&artboard), control_positions);

    let mut renderer = PathRecordingRenderer::default();
    artboard.draw(&mut renderer);
    assert!(renderer.saw_any_of(&host_paths));
}

#[test]
fn a_watermark_that_is_never_advanced_does_not_hide_the_artboard() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);

    let host_paths = record_draw(&artboard);
    let watermark = make_watermark(&file, None);
    artboard.with_artboard_mut(|artboard| artboard.set_watermark(Some(watermark)));

    let mut renderer = PathRecordingRenderer::default();
    artboard.draw(&mut renderer);
    assert!(renderer.saw_any_of(&host_paths));
}

#[test]
fn draw_internal_never_consults_the_watermark() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    let _deterministic = DeterministicTime::new();
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);
    let host_paths = record_draw(&artboard);
    assert!(!host_paths.is_empty());

    let mut watermark_paths = Vec::new();
    let watermark = make_watermark(&file, Some(&mut watermark_paths));
    artboard.with_artboard_mut(|artboard| artboard.set_watermark(Some(watermark)));
    assert!(state_machine.advance_and_apply(0.016));
    assert!(watermark_is_playing(&artboard));

    let mut renderer = PathRecordingRenderer::default();
    artboard.draw_internal(&mut renderer);
    assert!(renderer.saw_any_of(&host_paths));
    assert!(!renderer.saw_any_of(&watermark_paths));
}

#[test]
fn a_watermarked_file_pre_rolls_the_artboard_it_hands_out() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    let _deterministic = DeterministicTime::new();
    let file = read_rive_file("watermark_playback_test.riv");
    let manifest = file.with_file(File::manifest).expect("manifest");
    assert!(
        manifest
            .with_downcast::<ManifestAsset, _>(ManifestAsset::has_watermark)
            .unwrap()
    );
    assert!(file.with_file(File::artboard_count) > 1);
    assert_eq!(
        manifest.with_downcast::<ManifestAsset, _>(ManifestAsset::watermark_artboard_index),
        Some(1)
    );

    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    assert!(has_watermark(&artboard));
    let watermark_artboard = file
        .with_file(|file| file.artboard_at(1))
        .expect("watermark artboard");
    assert!(!has_watermark(&watermark_artboard));

    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);
    let host_paths = record_draw_internal(&artboard);
    assert!(!host_paths.is_empty());

    assert!(state_machine.advance_and_apply(0.016));
    assert!(watermark_is_playing(&artboard));
    let mut during_pre_roll = PathRecordingRenderer::default();
    artboard.draw(&mut during_pre_roll);
    assert!(!during_pre_roll.paths.is_empty());
    assert!(!during_pre_roll.saw_any_of(&host_paths));

    for _ in 0..800 {
        if !has_watermark(&artboard) {
            break;
        }
        state_machine.advance_and_apply(0.016);
    }
    assert!(!has_watermark(&artboard));

    let mut after_pre_roll = PathRecordingRenderer::default();
    artboard.draw(&mut after_pre_roll);
    assert!(after_pre_roll.saw_any_of(&host_paths));
}

#[test]
fn the_watermark_cannot_be_advanced_faster_than_real_time() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    assert!(!File::deterministic_mode());
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);
    let watermark = make_watermark(&file, None);
    artboard.with_artboard_mut(|artboard| artboard.set_watermark(Some(watermark)));

    assert!(state_machine.advance_and_apply(1000.0));
    assert!(has_watermark(&artboard));

    for _ in 0..500 {
        state_machine.advance_and_apply(0.5);
    }
    assert!(has_watermark(&artboard));
    assert!(watermark_is_playing(&artboard));

    for _ in 0..8000 {
        state_machine.advance_and_apply(0.004);
    }
    assert!(has_watermark(&artboard));
    assert!(watermark_is_playing(&artboard));

    let mut renderer = PathRecordingRenderer::default();
    artboard.draw(&mut renderer);
    assert!(!renderer.paths.is_empty());
    assert!(!renderer.saw_any_of(&record_draw_internal(&artboard)));
}

#[test]
fn a_file_without_a_watermark_section_vends_plain_artboards() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let manifest = file.with_file(File::manifest);
    assert!(manifest.is_none_or(|manifest| {
        !manifest
            .with_downcast::<ManifestAsset, _>(ManifestAsset::has_watermark)
            .unwrap()
    }));
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    assert!(!has_watermark(&artboard));
    for i in 0..file.with_file(File::artboard_count) {
        let artboard = file
            .with_file(|file| file.artboard_at(i))
            .expect("artboard");
        assert!(!has_watermark(&artboard));
    }
}

#[test]
fn idle_time_cannot_be_banked_to_drain_the_watermark() {
    let _lock = PLAYBACK_LOCK.lock().unwrap();
    assert!(!File::deterministic_mode());
    let file = read_rive_file("data_bind_keyframes_test.riv");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let state_machine = artboard.state_machine_at(0).expect("state machine");
    state_machine.advance_and_apply(0.0);
    let watermark = make_watermark(&file, None);
    artboard.with_artboard_mut(|artboard| artboard.set_watermark(Some(watermark)));

    std::thread::sleep(Duration::from_millis(400));
    for _ in 0..20 {
        state_machine.advance_and_apply(1000.0);
    }
    assert!(has_watermark(&artboard));
    assert!(watermark_is_playing(&artboard));
    assert!(
        artboard.with_artboard(|artboard| artboard.watermark().unwrap().elapsed_seconds()) < 0.3
    );
}
