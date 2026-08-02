use std::path::PathBuf;

use nuxie::{AudioEngine, Factory, File, RuntimeFileAssetKind};
use nuxie_render_api::NullFactory;

fn pinned_fixture(relative: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(relative);
    std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned audio fixture {}: {error}", path.display()))
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
    let bytes = pinned_fixture("sound.riv");
    let embedded = File::import(&bytes).expect("sound.riv imports");
    let source = embedded
        .audio_asset_source(52_054)
        .expect("embedded AudioAsset source");
    assert!(!source.bytes().is_empty());

    let mut factory = NullFactory::new();
    let mut saw_audio = false;
    let mut loader =
        |asset: &nuxie::RuntimeFileAsset, in_band: &[u8], factory: &mut dyn Factory| {
            if asset.kind() != RuntimeFileAssetKind::Audio {
                return false;
            }
            saw_audio = true;
            assert!(!in_band.is_empty());
            assert!(asset.decode(in_band, factory));
            assert!(asset.audio_source().is_some());
            true
        };
    let loaded = File::import_with_asset_loader(&bytes, &mut factory, &mut loader)
        .expect("loader import succeeds");
    assert!(saw_audio);
    assert!(loaded.audio_asset_source(52_054).is_some());
}

#[test]
fn sound2_fixture_decodes_its_embedded_flac() {
    let bytes = pinned_fixture("sound2.riv");
    let mut factory = NullFactory::new();
    let mut decoded_flac = false;
    let mut loader =
        |asset: &nuxie::RuntimeFileAsset, in_band: &[u8], factory: &mut dyn Factory| {
            if asset.kind() != RuntimeFileAssetKind::Audio {
                return false;
            }
            assert!(asset.decode(in_band, factory));
            let source = asset.audio_source().expect("embedded FLAC decodes");
            assert_eq!(source.format(), nuxie::AudioFormat::Flac);
            decoded_flac = true;
            true
        };
    File::import_with_asset_loader(&bytes, &mut factory, &mut loader).expect("sound2.riv imports");
    assert!(decoded_flac);
}

#[test]
fn sound_fixtures_match_direct_nested_and_no_audio_queries() {
    let sound = File::import(&pinned_fixture("sound.riv")).expect("sound.riv imports");
    assert!(
        sound
            .default_artboard()
            .expect("default artboard")
            .instantiate()
            .expect("sound artboard instance")
            .has_audio()
    );

    let sound2 = File::import(&pinned_fixture("sound2.riv")).expect("sound2.riv imports");
    for (name, expected) in [("child", true), ("grand-parent", true), ("no-audio", false)] {
        let instance = sound2
            .artboard_named(name)
            .unwrap_or_else(|| panic!("missing {name} artboard"))
            .instantiate()
            .unwrap_or_else(|error| panic!("instantiate {name}: {error:#}"));
        assert_eq!(instance.has_audio(), expected, "{name} hasAudio");
    }
}

#[test]
fn audio_event_playback_multiplies_artboard_volume_and_stops_with_its_artboard() {
    let file = File::import(&pinned_fixture("sound.riv")).expect("sound.riv imports");
    let engine = AudioEngine::new(2, 44_100).expect("headless engine");
    let mut first = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("first artboard instance");
    let mut second = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("second artboard instance");
    first.set_audio_engine(Some(engine.clone()));
    second.set_audio_engine(Some(engine.clone()));
    first.set_volume(0.25);
    let first_event = first
        .raw()
        .components()
        .iter()
        .find(|component| component.type_name == "AudioEvent")
        .expect("first AudioEvent")
        .local_id;
    let second_event = second
        .raw()
        .components()
        .iter()
        .find(|component| component.type_name == "AudioEvent")
        .expect("second AudioEvent")
        .local_id;

    let first_sound = first
        .play_audio_event(first_event)
        .expect("dense asset ordinal resolves and plays");
    assert_eq!(first_sound.volume(), 0.25);
    first
        .play_audio_event(first_event)
        .expect("second first sound");
    second
        .play_audio_event(second_event)
        .expect("second artboard sound");
    first
        .play_audio_event(first_event)
        .expect("third first sound");
    assert_eq!(engine.playing_sound_count(), 4);

    first.set_volume(0.0);
    assert!(first.play_audio_event(first_event).is_none());
    assert_eq!(engine.playing_sound_count(), 4);
    drop(first);
    assert_eq!(engine.playing_sound_count(), 1);
    drop(second);
    assert_eq!(engine.playing_sound_count(), 0);
}

#[test]
fn audio_event_artboard_clone_retains_its_asset_and_stops_independently() {
    let file = File::import(&pinned_fixture("sound.riv")).expect("sound.riv imports");
    let engine = AudioEngine::new(2, 44_100).expect("headless engine");
    let mut original = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("original artboard instance");
    original.set_audio_engine(Some(engine.clone()));
    let event_local = original
        .raw()
        .components()
        .iter()
        .find(|component| component.type_name == "AudioEvent")
        .expect("AudioEvent")
        .local_id;
    let cloned = original.raw().clone();

    original
        .play_audio_event(event_local)
        .expect("original retains AudioAsset");
    cloned
        .play_audio_event(event_local)
        .expect("clone retains AudioAsset");
    assert_eq!(engine.playing_sound_count(), 2);

    drop(original);
    assert_eq!(engine.playing_sound_count(), 1);
    drop(cloned);
    assert_eq!(engine.playing_sound_count(), 0);
}

#[cfg(feature = "scripting")]
#[test]
fn scripted_audio_plays_and_updates_volume_from_the_pinned_fixture() {
    use std::sync::Arc;

    use nuxie::{OwnedArtboardInstance, PersistentFactory, RecordingFactory};

    let engine = AudioEngine::make_and_store(2, 44_100).expect("runtime audio engine");
    let file = Arc::new(
        File::import_with_unsigned_scripts(&pinned_fixture("audio_script.riv"))
            .expect("audio_script.riv imports with trusted scripts"),
    );
    let mut instance =
        OwnedArtboardInstance::instantiate_default(file).expect("default artboard instance");
    instance.set_audio_engine(Some(engine.clone()));
    let mut machine = instance
        .default_state_machine_instance()
        .expect("default state machine");
    let mut view_model = instance
        .instantiate_default_view_model_instance()
        .or_else(|| instance.instantiate_view_model())
        .expect("audio script view model");
    let mut factory = PersistentFactory::new(RecordingFactory::new());

    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut factory,
        )
        .expect("initialize audio script");
    assert_eq!(engine.playing_sound_count(), 0);

    {
        let mut context = view_model.raw_mut();
        assert!(machine.pointer_down_with_owned_view_model_context(
            instance.raw_mut(),
            25.0,
            25.0,
            1,
            &mut context,
        ));
        assert!(machine.pointer_up_with_owned_view_model_context(
            instance.raw_mut(),
            25.0,
            25.0,
            1,
            &mut context,
        ));
    }
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut factory,
        )
        .expect("run scripted play callback");
    assert_eq!(engine.playing_sound_count(), 1);

    {
        let mut context = view_model.raw_mut();
        machine.pointer_down_with_owned_view_model_context(
            instance.raw_mut(),
            200.0,
            200.0,
            2,
            &mut context,
        );
        machine.pointer_up_with_owned_view_model_context(
            instance.raw_mut(),
            200.0,
            200.0,
            2,
            &mut context,
        );
    }
    instance
        .try_advance_with_state_machines_and_view_model_and_factory(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
            &mut factory,
        )
        .expect("run scripted volume callback");
    assert_eq!(
        engine
            .playing_sounds_head()
            .expect("scripted sound remains live")
            .volume(),
        0.1
    );
}
