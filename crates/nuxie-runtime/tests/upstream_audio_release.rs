//! Supplemental ownership coverage for cdeabe75 AudioEngine::ReleaseRuntimeEngine.
use nuxie_runtime::source::audio::audio_engine::AudioEngine;
use std::sync::Arc;

#[test]
fn release_runtime_engine_releases_singleton_but_not_retained_engine() {
    let retained = AudioEngine::make_and_store(1, 24_000).expect("engine");
    retained.advance(24);
    AudioEngine::release_runtime_engine();
    assert!(AudioEngine::runtime_engine(false).is_none());
    assert_eq!(retained.channels(), 1);
    assert_eq!(retained.time_in_frames(), 24);
    retained.advance(24);
    assert_eq!(retained.time_in_frames(), 48);

    let fresh = AudioEngine::runtime_engine(true).expect("fresh default engine");
    assert!(!Arc::ptr_eq(&retained, &fresh));
    assert_eq!(fresh.channels(), 2);
    assert_eq!(fresh.sample_rate(), 48_000);
    assert_eq!(fresh.time_in_frames(), 0);
    AudioEngine::release_runtime_engine();
    assert!(AudioEngine::runtime_engine(false).is_none());
}
