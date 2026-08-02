use std::path::PathBuf;

use nuxie::{Factory, File, RuntimeFileAssetKind};
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
