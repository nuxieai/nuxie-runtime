use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    generated::assets::{
        asset_base::AssetBaseCallbacks,
        export_audio_base::{ExportAudioBase, ExportAudioBaseCallbacks},
        file_asset_base::{FileAssetBase, FileAssetBaseCallbacks},
    },
};

pub struct ExportAudio {
    pub base: ExportAudioBase,
}

impl Default for ExportAudio {
    fn default() -> Self {
        Self {
            base: ExportAudioBase::default(),
        }
    }
}

impl ExportAudio {
    pub fn volume(&self) -> f32 {
        self.base.volume()
    }

    pub fn set_volume(&mut self, value: f32) {
        if self.base.set_volume_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(ExportAudioBase::VOLUME_PROPERTY_KEY);
        }
    }
}

impl AssetBaseCallbacks for ExportAudio {
    fn notify_property_changed(&mut self, property_key: u16) {
        AssetBaseCallbacks::notify_property_changed(&mut self.base.base, property_key);
    }
}

impl FileAssetBaseCallbacks for ExportAudio {
    fn notify_property_changed(&mut self, property_key: u16) {
        AssetBaseCallbacks::notify_property_changed(self, property_key);
    }

    fn decode_cdn_uuid(&mut self, value: &[u8]) {
        FileAsset::decode_cdn_uuid(&mut self.base.base, value);
    }

    fn copy_cdn_uuid(&mut self, object: &FileAssetBase) {
        FileAsset::copy_cdn_uuid(&mut self.base.base, object);
    }
}

impl ExportAudioBaseCallbacks for ExportAudio {}

impl std::ops::Deref for ExportAudio {
    type Target = ExportAudioBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ExportAudio {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
