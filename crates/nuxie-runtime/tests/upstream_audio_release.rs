//! Supplemental ownership coverage for cdeabe75 AudioEngine::ReleaseRuntimeEngine.
use nuxie_runtime::source::audio::audio_engine::{AudioEngine, AudioRenderer};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

struct SilentRenderer;
impl AudioRenderer for SilentRenderer {
    fn render(&mut self, output: &mut [f32], _frames: u32, _start_frame: u64) {
        output.fill(0.0);
    }
}

struct ReenterRuntimeOnDrop(Arc<AtomicUsize>);
impl AudioRenderer for ReenterRuntimeOnDrop {
    fn render(&mut self, output: &mut [f32], _: u32, _: u64) {
        output.fill(0.0);
    }
}
impl Drop for ReenterRuntimeOnDrop {
    fn drop(&mut self) {
        let _ = AudioEngine::runtime_engine(false);
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn release_runtime_engine_releases_singleton_but_not_retained_engine() {
    // Upstream advances the graph clock while a source is attached, in whole
    // mixing blocks. Keep this ownership test's 24-frame steps explicit.
    AudioEngine::set_buffer_frames(24);
    let retained = AudioEngine::make_and_store(1, 24_000).expect("engine");
    AudioEngine::set_buffer_frames(0);
    assert_eq!(retained.buffer_frames(), 24);
    let renderer = retained.attach_renderer(SilentRenderer);
    retained.advance(24);
    AudioEngine::release_runtime_engine();
    assert!(AudioEngine::runtime_engine(false).is_none());
    assert_eq!(retained.channels(), 1);
    assert_eq!(retained.time_in_frames(), 24);
    retained.advance(24);
    assert_eq!(retained.time_in_frames(), 48);
    retained.detach_renderer(renderer);

    let fresh = AudioEngine::runtime_engine(true).expect("fresh default engine");
    assert!(!Arc::ptr_eq(&retained, &fresh));
    assert_eq!(fresh.channels(), 2);
    assert_eq!(fresh.sample_rate(), 48_000);
    assert_eq!(fresh.buffer_frames(), 128);
    assert_eq!(fresh.time_in_frames(), 0);

    // Successful reads expose actual mixed frames separately, including an
    // idle graph and a partial cached block after renderer detachment.
    let mut read = u64::MAX;
    let mut output = [1.0; 8];
    assert!(fresh.read_audio_frames(&mut output, 4, Some(&mut read)));
    assert_eq!(read, 0);
    assert_eq!(output, [0.0; 8]);
    let mut sum = [0.25; 8];
    assert!(fresh.sum_audio_frames(&mut sum, 4));
    assert_eq!(sum, [0.25; 8]);
    let renderer = fresh.attach_renderer(SilentRenderer);
    assert!(fresh.read_audio_frames(&mut output, 4, Some(&mut read)));
    assert_eq!(read, 4);
    fresh.detach_renderer(renderer);
    let mut tail = [1.0; 256];
    assert!(fresh.read_audio_frames(&mut tail, 128, Some(&mut read)));
    assert_eq!(read, 124);
    assert_eq!(tail, [0.0; 256]);
    assert_eq!(fresh.time_in_frames(), 128);
    assert!(fresh.read_audio_frames(&mut output, 4, Some(&mut read)));
    assert_eq!(read, 0);
    assert!(!fresh.read_audio_frames(&mut output, 3, Some(&mut read)));
    assert!(!fresh.sum_audio_frames(&mut output, 3));
    AudioEngine::release_runtime_engine();
    assert!(AudioEngine::runtime_engine(false).is_none());

    // Replacement destroys the old callback only after publishing the new
    // singleton and releasing its lock; user destructors may query it.
    let drops = Arc::new(AtomicUsize::new(0));
    let old = AudioEngine::make_and_store(2, 48_000).unwrap();
    old.attach_renderer(ReenterRuntimeOnDrop(drops.clone()));
    drop(old);
    let replacement = AudioEngine::make_and_store(2, 44_100).unwrap();
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    drop(replacement);
    AudioEngine::release_runtime_engine();

    // A backend singleton can exist without a runtime wrapper singleton.
    // Releasing it must not hold the wrapper lock during callback destruction.
    let backend = nuxie_audio::AudioEngine::make_and_store(2, 48_000).unwrap();
    backend.attach_renderer(ReenterRuntimeOnDrop(drops.clone()));
    drop(backend);
    AudioEngine::release_runtime_engine();
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}
