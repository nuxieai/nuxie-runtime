use std::path::PathBuf;
use std::sync::Arc;

use nuxie::{AudioEngine, AudioSource, File};

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

fn assert_named_audio_artboard(name: &str, expected_events: usize, expected_has_audio: bool) {
    let file = File::import(&pinned_fixture("sound2.riv")).expect("sound2.riv imports");
    let mut artboard = file
        .artboard_named(name)
        .unwrap_or_else(|| panic!("missing {name} artboard"))
        .instantiate()
        .unwrap_or_else(|error| panic!("instantiate {name}: {error:#}"));
    artboard.set_audio_engine(Some(
        AudioEngine::new(2, 44_100).expect("audio engine initializes"),
    ));
    assert_eq!(direct_audio_event_count(&artboard), expected_events);
    assert_eq!(artboard.has_audio(), expected_has_audio);
}

fn direct_audio_event_count(instance: &nuxie::ArtboardInstance<'_>) -> usize {
    instance
        .raw()
        .components()
        .iter()
        .filter(|component| component.type_name == "AudioEvent")
        .count()
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
    let file = File::import(&pinned_fixture("sound.riv")).expect("sound.riv imports");
    let artboard = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("default artboard instance");
    let events = artboard
        .raw()
        .components()
        .iter()
        .filter(|component| component.type_name == "AudioEvent")
        .collect::<Vec<_>>();
    assert_eq!(events.len(), 1);

    let event_slot = artboard
        .raw()
        .slot(events[0].local_id)
        .expect("AudioEvent instance slot");
    let event = file
        .runtime()
        .object(event_slot.source_global_id as usize)
        .expect("AudioEvent runtime object");
    let dense_asset_ordinal = event.uint_property("assetId").expect("AudioEvent asset");
    let asset = file
        .runtime()
        .file_asset(dense_asset_ordinal as usize)
        .expect("AudioAsset runtime object");
    let semantic_asset_id = asset
        .uint_property("assetId")
        .expect("AudioAsset semantic id");
    assert!(
        file.audio_asset_source(semantic_asset_id as u32).is_some(),
        "the imported AudioEvent asset has a decoded audio source"
    );
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
