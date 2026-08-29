//! Audio backend and native scene differentials against the fingerprinted C++ oracle.

use nuxie_render_api::Mat2D;
#[cfg(feature = "tools")]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
#[cfg(feature = "tools")]
use nuxie_runtime::source::{
    assets::audio_asset::AudioAsset, audio::audio_engine::AudioEngine as NativeAudioEngine,
    audio_event::AudioEvent, nested_artboard::NestedArtboard,
};
use nuxie_runtime::{AudioEngine, AudioSource};
#[cfg(feature = "tools")]
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};
use serde::Deserialize;
use std::{path::PathBuf, process::Command, sync::Arc};

type CppProbeFile = serde_json::Value;
#[allow(dead_code)]
mod cpp_probe_support;
use cpp_probe_support::*;

fn build_and_require_probe(test: &str) -> PathBuf {
    if let Some(path) = probe_path() {
        return path;
    }
    let runtime_root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let status = Command::new("make")
        .arg("cpp-probe")
        .current_dir(repo_root())
        .env("RIVE_RUNTIME_DIR", runtime_root)
        .status()
        .unwrap_or_else(|error| panic!("{test}: failed to build the pinned C++ probe: {error}"));
    assert!(status.success(), "{test}: pinned C++ probe build failed");
    probe_path().unwrap_or_else(|| panic!("{test}: cpp-probe build produced no executable"))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppAudioOracle {
    channels: u32,
    sample_rate: u32,
    format: u32,
    duration: f32,
    native_frames: u64,
    mono48_frames: u64,
    stereo32_frames: u64,
    buffered_duration: f32,
    mp3_duration_positive: bool,
    mp3_duration_cached: bool,
    read_succeeded: bool,
    frames_read: u64,
    frame_clock: u64,
    completed: bool,
    window_energy: Vec<f32>,
    interior_read_succeeded: bool,
    interior_frames_read: u64,
    interior_completed: bool,
    interior_window_energy: Vec<f32>,
    seconds_read_succeeded: bool,
    seconds_frames_read: u64,
    seconds_completed: bool,
    seconds_samples: Vec<f32>,
    before_artboard_stop: usize,
    after_artboard_stop: usize,
    stopped_completed_before_dispose: bool,
    after_stopped_replay: usize,
    stopped_completed_after_dispose: bool,
    after_deferred_dispose: usize,
    stopped_volume: f32,
    seek_succeeded: bool,
    seek_cursor: u64,
    seek_cursor_after_read: u64,
    outliving_completed: bool,
    many_outliving_count: usize,
    many_outliving_completed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CppAudioRivOracle {
    sound_import: bool,
    event_count: usize,
    dense_asset_ordinal: u32,
    semantic_asset_id: u32,
    has_audio_source: bool,
    playing_after_play: usize,
    event_volume: f32,
    playing_after_zero_volume: usize,
    playing_after_second_play: usize,
    playing_after_drop: usize,
    fallback_playing_after_play: usize,
    fallback_playing_after_drop: usize,
    sound2_import: bool,
    child_has_audio: bool,
    child_event_count: usize,
    grand_parent_has_audio: bool,
    grand_parent_event_count: usize,
    nested_engine_propagated: bool,
    nested_volume_propagated: f32,
    nested_playing_after_play: usize,
    nested_event_volume: f32,
    no_audio_has_audio: bool,
    no_audio_event_count: usize,
}

#[test]
fn audio_source_reader_and_headless_schedule_match_pinned_cpp() {
    let probe =
        build_and_require_probe("audio_source_reader_and_headless_schedule_match_pinned_cpp");
    let fixture = cpp_runtime_fixture("audio/what.wav");
    let mp3_fixture = cpp_runtime_fixture("audio/song.mp3");
    let output = Command::new(probe)
        .args([
            "--audio-oracle",
            fixture.to_string_lossy().as_ref(),
            mp3_fixture.to_string_lossy().as_ref(),
        ])
        .output()
        .expect("run pinned audio oracle");
    assert!(
        output.status.success(),
        "audio oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cpp: CppAudioOracle =
        serde_json::from_slice(&output.stdout).expect("decode pinned audio oracle JSON");

    let bytes = std::fs::read(&fixture).expect("read pinned WAV");
    let source = AudioSource::from_encoded(bytes).expect("Rust WAV source");
    assert_eq!(source.channels(), cpp.channels);
    assert_eq!(source.sample_rate(), cpp.sample_rate);
    assert_eq!(cpp.format, 1, "pinned AudioFormat::wav ordinal");
    assert!((source.duration() - cpp.duration).abs() < 1.0e-6);
    assert_eq!(
        source
            .make_reader(2, 44_100)
            .expect("native reader")
            .length_in_frames(),
        cpp.native_frames
    );
    assert!(
        source
            .make_reader(1, 48_000)
            .expect("48 kHz reader")
            .length_in_frames()
            .abs_diff(cpp.mono48_frames)
            <= 2
    );
    assert!(
        source
            .make_reader(2, 32_000)
            .expect("32 kHz reader")
            .length_in_frames()
            .abs_diff(cpp.stereo32_frames)
            <= 2
    );

    let engine = AudioEngine::new(2, 44_100).expect("Rust headless engine");
    let source = Arc::new(source);

    let sound = engine
        .play(Arc::clone(&source), 512, 1024, 0, None)
        .expect("scheduled sound");
    sound.set_volume(0.5);
    let mut scheduled = vec![0.0; 1536 * 2];
    let rust_frames_read = scheduled
        .chunks_exact_mut(512 * 2)
        .map(|block| engine.read_audio_frames(block))
        .sum::<u64>();
    assert_eq!(rust_frames_read, cpp.frames_read);
    assert!(cpp.read_succeeded);
    let rust_window_energy = scheduled
        .chunks_exact(512 * 2)
        .map(|window| window.iter().map(|sample| sample.abs()).sum::<f32>())
        .collect::<Vec<_>>();
    assert_eq!(
        rust_window_energy
            .iter()
            .map(|energy| *energy > 0.0)
            .collect::<Vec<_>>(),
        cpp.window_energy
            .iter()
            .map(|energy| *energy > 0.0)
            .collect::<Vec<_>>(),
        "decoder PCM is tolerant, but absolute scheduling/clipping windows are exact"
    );
    assert_eq!(engine.time_in_frames(), cpp.frame_clock);
    assert_eq!(sound.completed(), cpp.completed);
    assert_eq!(engine.playing_sound_count(), 0);

    let interior_engine = AudioEngine::new(2, 44_100).expect("interior engine");
    let interior_sound = interior_engine
        .play(Arc::clone(&source), 513, 1025, 0, None)
        .expect("interior scheduled sound");
    let mut interior_scheduled = vec![0.0; 1536 * 2];
    assert_eq!(
        interior_engine.read_audio_frames(&mut interior_scheduled),
        cpp.interior_frames_read
    );
    assert!(cpp.interior_read_succeeded);
    let rust_interior_energy = interior_scheduled
        .chunks_exact(512 * 2)
        .map(|window| window.iter().map(|sample| sample.abs()).sum::<f32>() > 0.0)
        .collect::<Vec<_>>();
    assert_eq!(
        rust_interior_energy,
        cpp.interior_window_energy
            .iter()
            .map(|energy| *energy > 0.0)
            .collect::<Vec<_>>(),
        "non-block-aligned absolute scheduling windows are exact"
    );
    assert_eq!(interior_sound.completed(), cpp.interior_completed);

    let control_source = Arc::new(
        AudioSource::from_buffered((0..8).map(|sample| sample as f32).collect::<Vec<_>>(), 1, 4)
            .expect("control source"),
    );
    let buffered_duration = AudioSource::from_buffered(vec![0.0; 48_000 * 2], 2, 48_000)
        .expect("buffered duration source")
        .duration();
    assert_eq!(buffered_duration, cpp.buffered_duration);
    let mp3_source = AudioSource::from_encoded(
        std::fs::read(&mp3_fixture).expect("read pinned MP3 duration fixture"),
    )
    .expect("Rust MP3 source");
    let mp3_duration = mp3_source.duration();
    assert_eq!(mp3_duration > 0.0, cpp.mp3_duration_positive);
    assert_eq!(
        mp3_source.duration() == mp3_duration,
        cpp.mp3_duration_cached
    );
    let seconds_engine = AudioEngine::new(1, 4).expect("seconds engine");
    let seconds_sound = seconds_engine
        .play_seconds(Arc::clone(&control_source), 0.5, 6, 1, None)
        .expect("seconds sound");
    let mut seconds_samples = [0.0; 16];
    let seconds_frames_read = seconds_samples
        .chunks_exact_mut(8)
        .map(|block| seconds_engine.read_audio_frames(block))
        .sum::<u64>();
    assert_eq!(seconds_frames_read, cpp.seconds_frames_read);
    assert!(cpp.seconds_read_succeeded);
    assert_eq!(seconds_samples.as_slice(), cpp.seconds_samples.as_slice());
    assert_eq!(seconds_sound.completed(), cpp.seconds_completed);

    let lifecycle_engine = AudioEngine::new(1, 4).expect("lifecycle engine");
    let stopped = lifecycle_engine
        .play(
            Arc::clone(&control_source),
            0,
            0,
            0,
            Some(nuxie_runtime::AudioArtboardId(1)),
        )
        .expect("stopped sound");
    let _stopped2 = lifecycle_engine
        .play(
            Arc::clone(&control_source),
            0,
            0,
            0,
            Some(nuxie_runtime::AudioArtboardId(1)),
        )
        .expect("second stopped sound");
    let retained = lifecycle_engine
        .play(
            Arc::clone(&control_source),
            0,
            0,
            0,
            Some(nuxie_runtime::AudioArtboardId(2)),
        )
        .expect("retained sound");
    stopped.set_volume(0.25);
    assert_eq!(stopped.volume(), cpp.stopped_volume);
    assert_eq!(
        lifecycle_engine.playing_sound_count(),
        cpp.before_artboard_stop
    );
    lifecycle_engine.stop_artboard(nuxie_runtime::AudioArtboardId(1));
    assert_eq!(
        lifecycle_engine.playing_sound_count(),
        cpp.after_artboard_stop
    );
    assert_eq!(stopped.completed(), cpp.stopped_completed_before_dispose);
    stopped.play();
    assert_eq!(
        lifecycle_engine.playing_sound_count(),
        cpp.after_stopped_replay
    );
    let _cleanup = lifecycle_engine
        .play(Arc::clone(&control_source), 0, 0, 0, None)
        .expect("cleanup trigger");
    assert_eq!(stopped.completed(), cpp.stopped_completed_after_dispose);
    assert_eq!(
        lifecycle_engine.playing_sound_count(),
        cpp.after_deferred_dispose
    );
    assert_eq!(retained.seek(3), cpp.seek_succeeded);
    assert_eq!(retained.time_in_frames(), cpp.seek_cursor);
    lifecycle_engine.read_audio_frames(&mut [0.0]);
    assert_eq!(retained.time_in_frames(), cpp.seek_cursor_after_read);

    let outliving = {
        let temporary_engine = AudioEngine::new(1, 4).expect("temporary engine");
        temporary_engine
            .play(control_source, 0, 0, 0, None)
            .expect("outliving sound")
    };
    outliving.stop(0);
    assert_eq!(outliving.completed(), cpp.outliving_completed);

    let many_outliving = {
        let temporary_engine = AudioEngine::new(2, 44_100).expect("many-sound engine");
        let sounds = (0..20)
            .map(|_| {
                temporary_engine
                    .play(Arc::clone(&source), 0, 0, 0, None)
                    .expect("many outliving sound")
            })
            .collect::<Vec<_>>();
        temporary_engine.read_audio_frames(&mut vec![0.0; 512 * 2]);
        sounds
    };
    for sound in &many_outliving {
        sound.stop(0);
    }
    assert_eq!(many_outliving.len(), cpp.many_outliving_count);
    assert_eq!(
        many_outliving.iter().all(|sound| sound.completed()),
        cpp.many_outliving_completed
    );
}

#[cfg(feature = "tools")]
fn import_audio_fixture(path: &std::path::Path) -> RuntimeFileHandle {
    let bytes = std::fs::read(path).expect("read pinned audio scene");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    File::import(&bytes, factory, None, None, None).expect("native audio scene imports")
}

#[cfg(feature = "tools")]
fn play_audio_event(event: &CoreHandle) {
    event
        .with_downcast_mut::<AudioEvent, _>(AudioEvent::play)
        .expect("actual AudioEvent owner");
}

#[cfg(feature = "tools")]
fn first_nested_audio_artboard(
    artboard: &RuntimeArtboardInstanceHandle,
) -> Option<RuntimeArtboardInstanceHandle> {
    // Same depth-first traversal as the C++ oracle, over mounted instances.
    let nested = artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>());
    for host in nested {
        let Some(child) = host
            .with(|host| host.as_nested_artboard()?.artboard_instance_default())
            .flatten()
        else {
            continue;
        };
        if child.with_artboard(|child| child.count::<AudioEvent>() != 0) {
            return Some(child);
        }
        if let Some(descendant) = first_nested_audio_artboard(&child) {
            return Some(descendant);
        }
    }
    None
}

// The native engine's source-testing sound-list observations are tools-only,
// matching the pinned probe's inspection of m_playingSoundsHead.
#[cfg(feature = "tools")]
#[test]
fn audio_riv_load_playback_volume_and_has_audio_match_pinned_cpp() {
    let probe =
        build_and_require_probe("audio_riv_load_playback_volume_and_has_audio_match_pinned_cpp");
    let sound_fixture = cpp_runtime_fixture("sound.riv");
    let sound2_fixture = cpp_runtime_fixture("sound2.riv");
    let output = Command::new(probe)
        .args([
            "--audio-riv-oracle",
            sound_fixture.to_string_lossy().as_ref(),
            sound2_fixture.to_string_lossy().as_ref(),
        ])
        .output()
        .expect("run pinned AudioEvent fixture oracle");
    assert!(
        output.status.success(),
        "AudioEvent fixture oracle failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let cpp: CppAudioRivOracle =
        serde_json::from_slice(&output.stdout).expect("decode AudioEvent fixture oracle JSON");

    let sound = import_audio_fixture(&sound_fixture);
    assert!(cpp.sound_import);
    let artboard = sound
        .with_file(|file| file.artboard_default())
        .expect("sound artboard");
    let engine = NativeAudioEngine::make(2, 44_100).expect("native headless engine");
    artboard.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let audio_events = artboard.with_artboard(|artboard| artboard.find_all_handles::<AudioEvent>());
    assert_eq!(audio_events.len(), cpp.event_count);
    let event = &audio_events[0];
    let ordinal = event
        .with_downcast::<AudioEvent, _>(AudioEvent::asset_id)
        .expect("event asset id");
    assert_eq!(ordinal, cpp.dense_asset_ordinal);
    let asset = sound
        .with_file(|file| file.asset(ordinal as usize))
        .expect("dense AudioAsset ordinal");
    // Mutate the actual generated property, in the same order as the oracle.
    asset
        .with_downcast_mut::<AudioAsset, _>(|asset| asset.base.set_volume(0.5))
        .expect("AudioAsset owner");
    artboard.with_artboard_mut(|artboard| artboard.set_volume(0.25));

    play_audio_event(event);
    assert_eq!(engine.playing_sound_count(), cpp.playing_after_play);
    let first = engine
        .playing_sounds_head()
        .expect("AudioEvent scheduled a sound");
    assert_eq!(first.volume(), cpp.event_volume);
    artboard.with_artboard_mut(|artboard| artboard.set_volume(0.0));
    // Upstream play() returns void. No new sound is observed in the real
    // engine, instead of relying on the retired facade's Option return.
    play_audio_event(event);
    assert_eq!(engine.playing_sound_count(), cpp.playing_after_zero_volume);
    assert_eq!(
        engine
            .playing_sounds_head()
            .expect("retained first sound")
            .volume(),
        first.volume()
    );
    artboard.with_artboard_mut(|artboard| artboard.set_volume(0.25));
    play_audio_event(event);
    assert_eq!(engine.playing_sound_count(), cpp.playing_after_second_play);
    assert_eq!(
        asset
            .with(|asset| asset.as_file_asset().unwrap().file_asset_base().asset_id())
            .unwrap(),
        cpp.semantic_asset_id
    );
    assert_eq!(
        asset
            .with_downcast::<AudioAsset, _>(AudioAsset::has_audio_source)
            .unwrap(),
        cpp.has_audio_source
    );
    drop(artboard);
    assert_eq!(engine.playing_sound_count(), cpp.playing_after_drop);

    let fallback_engine =
        NativeAudioEngine::make_and_store(2, 44_100).expect("stored runtime engine");
    let fallback_artboard = sound
        .with_file(|file| file.artboard_default())
        .expect("runtime-engine artboard");
    let fallback_event = fallback_artboard
        .with_artboard(|artboard| artboard.object_handle_at::<AudioEvent>(0))
        .expect("fallback AudioEvent");
    play_audio_event(&fallback_event);
    assert_eq!(
        fallback_engine.playing_sound_count(),
        cpp.fallback_playing_after_play
    );
    drop(fallback_artboard);
    assert_eq!(
        fallback_engine.playing_sound_count(),
        cpp.fallback_playing_after_drop
    );
    fallback_engine.stop_all();

    let sound2 = import_audio_fixture(&sound2_fixture);
    assert!(cpp.sound2_import);
    let child = sound2
        .with_file(|file| file.artboard_named("child"))
        .expect("child artboard");
    let grand_parent = sound2
        .with_file(|file| file.artboard_named("grand-parent"))
        .expect("grand-parent artboard");
    let no_audio = sound2
        .with_file(|file| file.artboard_named("no-audio"))
        .expect("no-audio artboard");
    let nested_engine = NativeAudioEngine::make(2, 44_100).expect("nested-chain engine");
    grand_parent.with_artboard_mut(|artboard| {
        artboard.set_audio_engine(Some(nested_engine.clone()));
        artboard.set_volume(0.125);
    });
    let nested_audio =
        first_nested_audio_artboard(&grand_parent).expect("nested AudioEvent artboard");
    let nested_engine_propagated = nested_audio.with_artboard(|artboard| {
        artboard
            .audio_engine()
            .is_some_and(|engine| Arc::ptr_eq(&engine, &nested_engine))
    });
    assert_eq!(nested_engine_propagated, cpp.nested_engine_propagated);
    assert_eq!(
        nested_audio.with_artboard(|artboard| artboard.volume()),
        cpp.nested_volume_propagated
    );
    let nested_event = nested_audio
        .with_artboard(|artboard| artboard.object_handle_at::<AudioEvent>(0))
        .expect("nested event");
    play_audio_event(&nested_event);
    assert_eq!(
        nested_engine.playing_sound_count(),
        cpp.nested_playing_after_play
    );
    assert_eq!(
        nested_engine
            .playing_sounds_head()
            .expect("nested AudioEvent sound")
            .volume(),
        cpp.nested_event_volume
    );

    for (name, instance, cpp_has_audio, cpp_event_count) in [
        ("child", &child, cpp.child_has_audio, cpp.child_event_count),
        (
            "grand-parent",
            &grand_parent,
            cpp.grand_parent_has_audio,
            cpp.grand_parent_event_count,
        ),
        (
            "no-audio",
            &no_audio,
            cpp.no_audio_has_audio,
            cpp.no_audio_event_count,
        ),
    ] {
        assert_eq!(
            instance.with_artboard_mut(|artboard| artboard.has_audio()),
            cpp_has_audio,
            "{name} hasAudio"
        );
        assert_eq!(
            instance.with_artboard(|artboard| artboard.count::<AudioEvent>()),
            cpp_event_count,
            "{name} direct AudioEvent count"
        );
    }
}
