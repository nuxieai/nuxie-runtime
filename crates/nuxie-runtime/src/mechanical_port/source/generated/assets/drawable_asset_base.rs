use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    core::{binary_reader::BinaryReader, field_types::core_double_type::CoreDoubleType},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub trait DrawableAssetBaseCallbacks: FileAssetBaseCallbacks {
    fn height_changed(&mut self) {}
    fn width_changed(&mut self) {}
}

pub struct DrawableAssetBase {
    pub base: FileAsset,
    height: f32,
    width: f32,
}

impl Default for DrawableAssetBase {
    fn default() -> Self {
        Self {
            base: FileAsset::default(),
            height: 0.0,
            width: 0.0,
        }
    }
}

impl DrawableAssetBase {
    pub const TYPE_KEY: u16 = 104;
    pub const HEIGHT_PROPERTY_KEY: u16 = 207;
    pub const WIDTH_PROPERTY_KEY: u16 = 208;

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

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn set_height<C: DrawableAssetBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if self.height == value {
            return;
        }
        self.height = value;
        callbacks.height_changed();
        callbacks.notify_property_changed(Self::HEIGHT_PROPERTY_KEY);
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn set_width<C: DrawableAssetBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if self.width == value {
            return;
        }
        self.width = value;
        callbacks.width_changed();
        callbacks.notify_property_changed(Self::WIDTH_PROPERTY_KEY);
    }

    pub fn copy<C: DrawableAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.height = object.height;
        self.width = object.width;
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn deserialize<C: DrawableAssetBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::HEIGHT_PROPERTY_KEY => {
                self.height = CoreDoubleType::deserialize(reader);
                true
            }
            Self::WIDTH_PROPERTY_KEY => {
                self.width = CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}
