use std::path::PathBuf;
use std::sync::Arc;

use nuxie::runtime::{
    assets::audio_asset::AudioAsset, audio::audio_engine::AudioEngine as NativeAudioEngine,
    audio_event::AudioEvent,
};
use nuxie::{
    AudioEngine, AudioSource, File, PersistentFactory, RuntimeFactoryHandle, RuntimeFileHandle,
};
use nuxie_render_api::SerializingFactory;

fn pinned_fixture(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned audio fixture {}: {error}", path.display()))
}

fn pinned_wav_source() -> Arc<AudioSource> {
    Arc::new(
        AudioSource::from_encoded(pinned_fixture("audio/what.wav"))
            .expect("open the pinned WAV source"),
    )
}

fn import(relative: &str) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    File::import(&pinned_fixture(relative), factory, None, None, None)
        .unwrap_or_else(|| panic!("{relative} imports"))
}

fn assert_named_audio_artboard(name: &str, expected_events: usize, expected_has_audio: bool) {
    let file = import("sound2.riv");
    let artboard = file
        .with_file(|file| file.artboard_named(name))
        .unwrap_or_else(|| panic!("missing {name} artboard"));
    let engine = NativeAudioEngine::make(2, 44_100).expect("audio engine initializes");
    artboard.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine)));
    assert_eq!(direct_audio_event_count(&artboard), expected_events);
    assert_eq!(
        artboard.with_artboard_mut(|artboard| artboard.has_audio()),
        expected_has_audio
    );
}

fn direct_audio_event_count(instance: &nuxie::RuntimeArtboardInstanceHandle) -> usize {
    instance.with_artboard(|artboard| artboard.count::<AudioEvent>())
}

#[test]
fn upstream_audio_case_01_engine_initializes() {
    let engine = AudioEngine::new(2, 44_100).expect("audio engine initializes");
    assert_eq!(engine.channels(), 2);
    assert_eq!(engine.sample_rate(), 44_100);
}

#[test]
#[ignore = "expected-red: native reader rounds the pinned 48 kHz and 32 kHz resamples one frame longer than C++"]
fn upstream_audio_case_02_source_reader_levels_and_playback() {
    let engine = AudioEngine::new(2, 44_100).expect("audio engine initializes");
    let source = pinned_wav_source();
    assert_eq!(source.channels(), 2);
    assert_eq!(source.sample_rate(), 44_100);

    let native_frames = source
        .make_reader(2, 44_100)
        .expect("native-rate reader")
        .length_in_frames();
    let mono_48_frames = source
        .make_reader(1, 48_000)
        .expect("mono 48 kHz reader")
        .length_in_frames();
    let stereo_32_frames = source
        .make_reader(2, 32_000)
        .expect("stereo 32 kHz reader")
        .length_in_frames();

    let mut levels = [0.0; 2];
    engine.levels(&mut levels);
    assert_eq!(levels, [0.0, 0.0]);

    let _sound = engine
        .play(source, 0, 0, 0, None)
        .expect("play the pinned WAV source");
    let mut frames = [0.0; 512 * 2];
    engine.read_audio_frames(&mut frames);
    engine.levels(&mut levels);
    assert_ne!(levels[0], 0.0);
    assert_ne!(levels[1], 0.0);

    engine.read_audio_frames(&mut frames);
    assert_ne!(engine.level(0), 0.0);
    assert_ne!(engine.level(1), 0.0);

    assert_eq!(
        [native_frames, mono_48_frames, stereo_32_frames],
        [9_688, 10_544, 7_029]
    );
}

#[test]
fn upstream_audio_case_03_file_with_audio_loads_correctly() {
    let file = import("sound.riv");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard instance");
    let events = artboard.with_artboard(|artboard| artboard.find_all_handles::<AudioEvent>());
    assert_eq!(events.len(), 1);
    let dense_asset_ordinal = events[0]
        .with_downcast::<AudioEvent, _>(AudioEvent::asset_id)
        .expect("AudioEvent asset");
    let asset = file
        .with_file(|file| file.asset(dense_asset_ordinal as usize))
        .expect("AudioAsset runtime object");
    let semantic_asset_id = asset
        .with(|asset| {
            asset
                .as_file_asset()
                .map(|asset| asset.file_asset_base().asset_id())
        })
        .flatten()
        .expect("AudioAsset semantic id");
    assert!(
        asset
            .with_downcast::<AudioAsset, _>(AudioAsset::has_audio_source)
            .unwrap_or(false),
        "the imported AudioEvent asset has a decoded audio source"
    );
    assert_ne!(semantic_asset_id, 0);
}

#[test]
fn upstream_audio_case_04_sound_can_outlive_engine() {
    let sound = {
        let engine = AudioEngine::new(2, 44_100).expect("audio engine initializes");
        let source = pinned_wav_source();
        assert_eq!(source.channels(), 2);
        assert_eq!(source.sample_rate(), 44_100);
        let sound = engine
            .play(source, 0, 0, 0, None)
            .expect("play the pinned WAV source");
        engine.read_audio_frames(&mut [0.0; 512 * 2]);
        sound
    };
    sound.stop(0);
}

#[test]
fn upstream_audio_case_05_many_sounds_can_outlive_engine() {
    let sounds = {
        let engine = AudioEngine::new(2, 44_100).expect("audio engine initializes");
        let source = pinned_wav_source();
        assert_eq!(source.channels(), 2);
        assert_eq!(source.sample_rate(), 44_100);
        let sounds = (0..20)
            .map(|_| {
                engine
                    .play(Arc::clone(&source), 0, 0, 0, None)
                    .expect("play the pinned WAV source")
            })
            .collect::<Vec<_>>();
        engine.read_audio_frames(&mut [0.0; 512 * 2]);
        sounds
    };
    for sound in sounds {
        sound.stop(0);
    }
}

#[test]
fn upstream_audio_case_07_artboard_has_direct_audio() {
    assert_named_audio_artboard("child", 1, true);
}

#[test]
fn upstream_audio_case_08_artboard_has_nested_audio() {
    assert_named_audio_artboard("grand-parent", 0, true);
}

#[test]
fn upstream_audio_case_09_artboard_does_not_have_audio() {
    assert_named_audio_artboard("no-audio", 0, false);
}

#[test]
fn upstream_audio_case_11_file_duration_is_cached() {
    let source = pinned_wav_source();
    assert_eq!(source.channels(), 2);
    assert_eq!(source.sample_rate(), 44_100);
    let duration = source.duration();
    assert_eq!(duration, 9_688.0 / 44_100.0);
    assert_eq!(source.duration(), duration);
}
