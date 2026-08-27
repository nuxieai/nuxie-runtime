use crate::mechanical_port::source::{
    assets::{audio_asset::AudioAsset, export_audio::ExportAudio},
    generated::assets::export_audio_base::ExportAudioBaseCallbacks,
};

pub struct AudioAssetBase {
    pub base: ExportAudio,
}

impl Default for AudioAssetBase {
    fn default() -> Self {
        Self {
            base: ExportAudio::default(),
        }
    }
}

impl AudioAssetBase {
    pub const TYPE_KEY: u16 = 406;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 422 | 103 | 99)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn copy<C: ExportAudioBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn clone_into<C: ExportAudioBaseCallbacks>(&self, callbacks: &mut C) -> AudioAsset {
        let mut cloned = AudioAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
}
