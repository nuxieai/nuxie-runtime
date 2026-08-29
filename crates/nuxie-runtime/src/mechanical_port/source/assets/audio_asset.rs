use crate::mechanical_port::source::{
    audio::audio_source::{AudioSource, AudioSourceRef},
    factory::RuntimeFactoryHandle,
    generated::assets::audio_asset_base::AudioAssetBase,
};

pub struct AudioAsset {
    pub base: AudioAssetBase,
    audio_source: Option<AudioSourceRef>,
}

impl Default for AudioAsset {
    fn default() -> Self {
        Self {
            base: AudioAssetBase::default(),
            audio_source: None,
        }
    }
}

impl AudioAsset {
    pub fn decode(&mut self, bytes: &mut Vec<u8>, _factory: &RuntimeFactoryHandle) -> bool {
        let encoded = std::mem::take(bytes);
        self.audio_source = AudioSource::from_encoded(&encoded);
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "wav"
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn has_audio_source(&self) -> bool {
        self.audio_source.is_some()
    }

    pub fn audio_source(&self) -> Option<AudioSourceRef> {
        self.audio_source.clone()
    }

    pub fn set_audio_source(&mut self, source: Option<AudioSourceRef>) {
        self.audio_source = source;
    }
}
