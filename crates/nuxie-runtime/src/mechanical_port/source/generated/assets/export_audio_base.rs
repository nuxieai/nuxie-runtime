use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    core::{binary_reader::BinaryReader, field_types::core_double_type::CoreDoubleType},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub trait ExportAudioBaseCallbacks: FileAssetBaseCallbacks {
    fn volume_changed(&mut self) {}
}

pub struct ExportAudioBase {
    pub base: FileAsset,
    volume: f32,
}

impl Default for ExportAudioBase {
    fn default() -> Self {
        Self {
            base: FileAsset::default(),
            volume: 1.0,
        }
    }
}

impl ExportAudioBase {
    pub const TYPE_KEY: u16 = 422;
    pub const VOLUME_PROPERTY_KEY: u16 = 530;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 103 | 99)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn file_asset(&self) -> &FileAsset {
        &self.base
    }

    pub fn file_asset_mut(&mut self) -> &mut FileAsset {
        &mut self.base
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn set_volume<C: ExportAudioBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if !self.set_volume_value(value) {
            return;
        }
        callbacks.volume_changed();
        callbacks.notify_property_changed(Self::VOLUME_PROPERTY_KEY);
    }
    pub(crate) fn set_volume_value(&mut self, value: f32) -> bool {
        if self.volume == value {
            return false;
        }
        self.volume = value;
        true
    }

    pub fn copy<C: ExportAudioBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.volume = object.volume;
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn deserialize<C: ExportAudioBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::VOLUME_PROPERTY_KEY => {
                self.volume = CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ExportAudioBase {
    type Target = FileAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ExportAudioBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
