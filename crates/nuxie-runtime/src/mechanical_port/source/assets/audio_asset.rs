use crate::mechanical_port::source::{
    audio::audio_source::AudioSourceRef, factory::Factory,
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
    pub fn decode(&mut self, bytes: &mut Vec<u8>, _factory: &mut Factory) -> bool {
        #[cfg(feature = "rive_audio")]
        {
            self.audio_source = Some(AudioSourceRef::make(std::mem::take(bytes)));
        }
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "wav"
    }

    #[cfg(feature = "testing")]
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
