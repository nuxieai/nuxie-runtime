use std::{cell::Cell, path::PathBuf, rc::Rc};

use nuxie::{
    Factory, File, FileAssetLoader, FileAssetLoaderRef, PersistentFactory, RuntimeFactoryHandle,
    RuntimeFileHandle,
    runtime::{
        assets::audio_asset::AudioAsset,
        audio::{audio_engine::AudioEngine as NativeAudioEngine, audio_format::AudioFormat},
        audio_event::AudioEvent,
    },
};
use nuxie_render_api::{NullFactory, RecordingFactory};

fn pinned_fixture(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned audio fixture {}: {error}", path.display()))
}

fn import(relative: &str, loader: Option<FileAssetLoaderRef>) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    File::import(&pinned_fixture(relative), factory, None, loader, None)
        .unwrap_or_else(|| panic!("{relative} imports"))
}

struct AudioLoader {
    saw_audio: Rc<Cell<bool>>,
    format: Option<AudioFormat>,
}

impl FileAssetLoader for AudioLoader {
    fn load_contents(
        &mut self,
        asset: nuxie::CoreHandle,
        in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        asset
            .with_downcast_mut::<AudioAsset, _>(|asset| {
                self.saw_audio.set(true);
                assert!(!in_band_bytes.is_empty());
                assert!(asset.decode(&mut in_band_bytes.to_vec(), factory));
                let source = asset.audio_source().expect("embedded audio decodes");
                if let Some(format) = self.format {
                    assert_eq!(source.format(), format);
                }
            })
            .is_some()
    }
}

#[test]
fn factory_decode_audio_owns_and_decodes_the_pinned_wav() {
    let mut factory = NullFactory::new();
    let bytes = pinned_fixture("audio/what.wav");
    let source = factory.decode_audio(&bytes).expect("Factory decodes WAV");
    drop(bytes);
    assert_eq!(source.channels(), 2);
    assert_eq!(source.sample_rate(), 44_100);
    assert!(!source.bytes().is_empty());
}

#[test]
fn sound_fixture_loads_embedded_audio_and_host_loader_gets_first_refusal() {
    let embedded = import("sound.riv", None);
    let embedded_asset = embedded
        .with_file(|file| file.asset(0))
        .expect("embedded AudioAsset");
    assert!(
        embedded_asset
            .with_downcast::<AudioAsset, _>(AudioAsset::has_audio_source)
            .unwrap_or(false)
    );

    let saw_audio = Rc::new(Cell::new(false));
    let loader = FileAssetLoaderRef::new(Box::new(AudioLoader {
        saw_audio: saw_audio.clone(),
        format: None,
    }));
    let loaded = import("sound.riv", Some(loader));
    assert!(saw_audio.get());
    assert!(
        loaded
            .with_file(|file| file.asset(0))
            .and_then(|asset| asset.with_downcast::<AudioAsset, _>(AudioAsset::audio_source))
            .flatten()
            .is_some()
    );
}

#[test]
fn sound2_fixture_decodes_its_embedded_flac() {
    let decoded_flac = Rc::new(Cell::new(false));
    let loader = FileAssetLoaderRef::new(Box::new(AudioLoader {
        saw_audio: decoded_flac.clone(),
        format: Some(AudioFormat::Flac),
    }));
    let _file = import("sound2.riv", Some(loader));
    assert!(decoded_flac.get());
}

#[test]
fn sound_fixtures_match_direct_nested_and_no_audio_queries() {
    let sound = import("sound.riv", None);
    let sound_artboard = sound
        .with_file(|file| file.artboard_default())
        .expect("sound artboard instance");
    assert!(sound_artboard.with_artboard_mut(|artboard| artboard.has_audio()));

    let sound2 = import("sound2.riv", None);
    for (name, expected) in [("child", true), ("grand-parent", true), ("no-audio", false)] {
        let instance = sound2
            .with_file(|file| file.artboard_named(name))
            .unwrap_or_else(|| panic!("missing {name} artboard"));
        assert_eq!(
            instance.with_artboard_mut(|artboard| artboard.has_audio()),
            expected,
            "{name} hasAudio"
        );
    }
}

#[test]
fn audio_event_playback_multiplies_artboard_volume_and_stops_with_its_artboard() {
    let file = import("sound.riv", None);
    let engine = NativeAudioEngine::make(2, 44_100).expect("headless engine");
    let first = file
        .with_file(|file| file.artboard_default())
        .expect("first artboard instance");
    let second = file
        .with_file(|file| file.artboard_default())
        .expect("second artboard instance");
    first.with_artboard_mut(|artboard| {
        artboard.set_audio_engine(Some(engine.clone()));
        artboard.set_volume(0.25);
    });
    second.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let first_event = first
        .with_artboard(|artboard| artboard.object_handle_at::<AudioEvent>(0))
        .expect("first AudioEvent");
    let second_event = second
        .with_artboard(|artboard| artboard.object_handle_at::<AudioEvent>(0))
        .expect("second AudioEvent");

    first_event
        .with_downcast_mut::<AudioEvent, _>(AudioEvent::play)
        .expect("actual first AudioEvent");
    let first_sound = engine.playing_sounds_head().expect("first sound plays");
    assert_eq!(first_sound.volume(), 0.25);
    first_event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    second_event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    first_event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    assert_eq!(engine.playing_sound_count(), 4);

    first.with_artboard_mut(|artboard| artboard.set_volume(0.0));
    first_event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    assert_eq!(engine.playing_sound_count(), 4);
    drop(first);
    assert_eq!(engine.playing_sound_count(), 1);
    drop(second);
    assert_eq!(engine.playing_sound_count(), 0);
}

#[test]
fn audio_event_artboard_clone_retains_its_asset_and_stops_independently() {
    let file = import("sound.riv", None);
    let engine = NativeAudioEngine::make(2, 44_100).expect("headless engine");
    let original = file
        .with_file(|file| file.artboard_default())
        .expect("original artboard instance");
    original.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let cloned = original.instance().expect("clone artboard instance");
    cloned.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let original_event = original
        .with_artboard(|artboard| artboard.object_handle_at::<AudioEvent>(0))
        .expect("original AudioEvent");
    let cloned_event = cloned
        .with_artboard(|artboard| artboard.object_handle_at::<AudioEvent>(0))
        .expect("cloned AudioEvent");

    original_event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    cloned_event.with_downcast_mut::<AudioEvent, _>(AudioEvent::play);
    assert_eq!(engine.playing_sound_count(), 2);

    drop(original);
    assert_eq!(engine.playing_sound_count(), 1);
    drop(cloned);
    assert_eq!(engine.playing_sound_count(), 0);
}

#[cfg(feature = "scripting")]
#[test]
fn scripted_audio_plays_and_updates_volume_from_the_pinned_fixture() {
    use nuxie::{
        FileImportLimits, ScriptExecutionLimits, Vec2D, ViewModelInstanceRuntime,
        import_unsigned_scripted,
    };

    let engine = NativeAudioEngine::make_and_store(2, 44_100).expect("runtime audio engine");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let scripted = import_unsigned_scripted(
        &pinned_fixture("audio_script.riv"),
        &mut factory,
        None,
        FileImportLimits::new(),
        ScriptExecutionLimits::new(),
    )
    .expect("audio_script.riv imports with trusted scripts");
    scripted.vm().set_is_playing(true);
    let file = scripted.native_file();
    let instance = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard instance");
    instance.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let machine = instance
        .default_state_machine_handle()
        .expect("default state machine");
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(instance.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(instance.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("audio script view model");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    instance.bind_view_model_instance(Some(view_model.instance()));

    machine.advance_and_apply(0.016);
    assert_eq!(engine.playing_sound_count(), 0);

    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(25.0, 25.0), 1);
        machine.pointer_up(Vec2D::new(25.0, 25.0), 1);
    });
    machine.advance_and_apply(0.016);
    assert_eq!(engine.playing_sound_count(), 1);

    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(200.0, 200.0), 2);
        machine.pointer_up(Vec2D::new(200.0, 200.0), 2);
    });
    machine.advance_and_apply(0.016);
    assert_eq!(
        engine
            .playing_sounds_head()
            .expect("scripted sound remains live")
            .volume(),
        0.1
    );
}
