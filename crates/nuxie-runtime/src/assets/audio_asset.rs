use nuxie_audio::AudioSource;
use nuxie_binary::RuntimeFile;
use nuxie_render_api::Factory as RenderFactory;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

/// File-owned decoded `AudioAsset` sources, keyed by file-global object id.
///
/// Invalid encoded bytes clear the concrete source but never fail file import,
/// matching `AudioAsset::decode` returning true even when `MakeAudioSource`
/// returns null.
#[derive(Debug, Default)]
pub struct RuntimeAudioAssetOwners {
    sources: RefCell<BTreeMap<u32, Arc<AudioSource>>>,
}

impl RuntimeAudioAssetOwners {
    pub fn from_runtime(runtime: &RuntimeFile) -> Self {
        let owners = Self::default();
        for entry in runtime.imported_file_assets_with_contents() {
            if entry.asset.type_name == "AudioAsset"
                && let Some(bytes) = entry.contents
            {
                owners.replace(
                    entry.asset.id,
                    AudioSource::from_encoded(bytes.to_vec()).ok().map(Arc::new),
                );
            }
        }
        owners
    }

    pub fn get(&self, asset_global: u32) -> Option<Arc<AudioSource>> {
        self.sources.borrow().get(&asset_global).cloned()
    }

    /// Decode and atomically replace the concrete source. As in pinned
    /// `AudioAsset::decode`, the Factory argument is ignored. The return value
    /// is import success, not decoder success, and is therefore always true.
    pub fn decode(
        &self,
        asset_global: u32,
        bytes: &[u8],
        _factory: &mut dyn RenderFactory,
    ) -> bool {
        // Pinned AudioAsset::decode ignores its Factory argument and directly
        // constructs an AudioSource. Factory::decodeAudio is a separate helper
        // used by command/host surfaces.
        self.replace(
            asset_global,
            AudioSource::from_encoded(bytes.to_vec()).ok().map(Arc::new),
        );
        true
    }

    fn replace(&self, asset_global: u32, source: Option<Arc<AudioSource>>) {
        match source {
            Some(source) => {
                self.sources.borrow_mut().insert(asset_global, source);
            }
            None => {
                self.sources.borrow_mut().remove(&asset_global);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::read_runtime_file_with_scripting;
    use std::path::PathBuf;

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
    fn sound_fixture_installs_an_owned_embedded_audio_source() {
        let bytes = pinned_fixture("sound.riv");
        let runtime = read_runtime_file_with_scripting(&bytes).expect("sound.riv imports");
        let audio = runtime
            .file_assets()
            .into_iter()
            .find(|asset| asset.type_name == "AudioAsset")
            .expect("AudioAsset");
        let owners = RuntimeAudioAssetOwners::from_runtime(&runtime);
        let source = owners.get(audio.id).expect("decoded embedded source");
        assert_eq!(source.channels(), 2);
        assert!(!source.bytes().is_empty());
        drop(runtime);
        drop(bytes);
        assert!(!source.bytes().is_empty(), "source owns the encoded bytes");
    }
}
