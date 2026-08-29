use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    generated::assets::{
        asset_base::AssetBaseCallbacks,
        drawable_asset_base::{DrawableAssetBase, DrawableAssetBaseCallbacks},
        file_asset_base::{FileAssetBase, FileAssetBaseCallbacks},
    },
};

pub struct DrawableAsset {
    pub base: DrawableAssetBase,
}

impl Default for DrawableAsset {
    fn default() -> Self {
        Self {
            base: DrawableAssetBase::default(),
        }
    }
}

impl DrawableAsset {
    pub fn height(&self) -> f32 {
        self.base.height()
    }

    pub fn set_height(&mut self, value: f32) {
        if self.base.set_height_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(DrawableAssetBase::HEIGHT_PROPERTY_KEY);
        }
    }

    pub fn width(&self) -> f32 {
        self.base.width()
    }

    pub fn set_width(&mut self, value: f32) {
        if self.base.set_width_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(DrawableAssetBase::WIDTH_PROPERTY_KEY);
        }
    }
}

impl AssetBaseCallbacks for DrawableAsset {
    fn notify_property_changed(&mut self, property_key: u16) {
        AssetBaseCallbacks::notify_property_changed(&mut self.base.base, property_key);
    }
}

impl FileAssetBaseCallbacks for DrawableAsset {
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

impl DrawableAssetBaseCallbacks for DrawableAsset {}

impl std::ops::Deref for DrawableAsset {
    type Target = DrawableAssetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawableAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
