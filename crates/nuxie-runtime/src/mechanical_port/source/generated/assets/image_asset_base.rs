use crate::mechanical_port::source::{
    assets::{drawable_asset::DrawableAsset, image_asset::ImageAsset},
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    generated::assets::drawable_asset_base::DrawableAssetBaseCallbacks,
};

pub trait ImageAssetBaseCallbacks: DrawableAssetBaseCallbacks {
    fn sampler_filter_changed(&mut self) {}
    fn sampler_wrap_x_changed(&mut self) {}
    fn sampler_wrap_y_changed(&mut self) {}
}

pub struct ImageAssetBase {
    pub base: DrawableAsset,
    sampler_filter: u8,
    sampler_wrap_x: u8,
    sampler_wrap_y: u8,
}

impl Default for ImageAssetBase {
    fn default() -> Self {
        Self {
            base: DrawableAsset::default(),
            sampler_filter: 0,
            sampler_wrap_x: 0,
            sampler_wrap_y: 0,
        }
    }
}

impl ImageAssetBase {
    pub const TYPE_KEY: u16 = 105;
    pub const SAMPLER_FILTER_PROPERTY_KEY: u16 = 1073;
    pub const SAMPLER_WRAP_X_PROPERTY_KEY: u16 = 1074;
    pub const SAMPLER_WRAP_Y_PROPERTY_KEY: u16 = 1075;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 104 | 103 | 99)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn file_asset(&self) -> &crate::mechanical_port::source::assets::file_asset::FileAsset {
        self.base.base.file_asset()
    }

    pub fn sampler_filter(&self) -> u8 {
        self.sampler_filter
    }

    pub fn set_sampler_filter<C: ImageAssetBaseCallbacks>(&mut self, value: u8, callbacks: &mut C) {
        if !self.set_sampler_filter_value(value) {
            return;
        }
        callbacks.sampler_filter_changed();
        crate::mechanical_port::source::generated::assets::file_asset_base::FileAssetBaseCallbacks::notify_property_changed(callbacks, Self::SAMPLER_FILTER_PROPERTY_KEY);
    }

    pub(crate) fn set_sampler_filter_value(&mut self, value: u8) -> bool {
        if self.sampler_filter == value {
            return false;
        }
        self.sampler_filter = value;
        true
    }

    pub fn sampler_wrap_x(&self) -> u8 {
        self.sampler_wrap_x
    }

    pub fn set_sampler_wrap_x<C: ImageAssetBaseCallbacks>(&mut self, value: u8, callbacks: &mut C) {
        if !self.set_sampler_wrap_x_value(value) {
            return;
        }
        callbacks.sampler_wrap_x_changed();
        crate::mechanical_port::source::generated::assets::file_asset_base::FileAssetBaseCallbacks::notify_property_changed(callbacks, Self::SAMPLER_WRAP_X_PROPERTY_KEY);
    }

    pub(crate) fn set_sampler_wrap_x_value(&mut self, value: u8) -> bool {
        if self.sampler_wrap_x == value {
            return false;
        }
        self.sampler_wrap_x = value;
        true
    }

    pub fn sampler_wrap_y(&self) -> u8 {
        self.sampler_wrap_y
    }

    pub fn set_sampler_wrap_y<C: ImageAssetBaseCallbacks>(&mut self, value: u8, callbacks: &mut C) {
        if !self.set_sampler_wrap_y_value(value) {
            return;
        }
        callbacks.sampler_wrap_y_changed();
        crate::mechanical_port::source::generated::assets::file_asset_base::FileAssetBaseCallbacks::notify_property_changed(callbacks, Self::SAMPLER_WRAP_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_sampler_wrap_y_value(&mut self, value: u8) -> bool {
        if self.sampler_wrap_y == value {
            return false;
        }
        self.sampler_wrap_y = value;
        true
    }

    pub fn copy<C: ImageAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.sampler_filter = object.sampler_filter;
        self.sampler_wrap_x = object.sampler_wrap_x;
        self.sampler_wrap_y = object.sampler_wrap_y;
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn clone_into<C: ImageAssetBaseCallbacks>(&self, callbacks: &mut C) -> ImageAsset {
        let mut cloned = ImageAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn deserialize<C: ImageAssetBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::SAMPLER_FILTER_PROPERTY_KEY => {
                self.sampler_filter = CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::SAMPLER_WRAP_X_PROPERTY_KEY => {
                self.sampler_wrap_x = CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::SAMPLER_WRAP_Y_PROPERTY_KEY => {
                self.sampler_wrap_y = CoreUintType::deserialize(reader) as u8;
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ImageAssetBase {
    type Target = DrawableAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ImageAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
