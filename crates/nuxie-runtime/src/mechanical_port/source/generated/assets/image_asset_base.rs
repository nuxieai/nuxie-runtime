use crate::mechanical_port::source::{
    assets::{drawable_asset::DrawableAsset, image_asset::ImageAsset},
    generated::assets::drawable_asset_base::DrawableAssetBaseCallbacks,
};

pub struct ImageAssetBase {
    pub base: DrawableAsset,
}

impl Default for ImageAssetBase {
    fn default() -> Self {
        Self {
            base: DrawableAsset::default(),
        }
    }
}

impl ImageAssetBase {
    pub const TYPE_KEY: u16 = 105;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 104 | 103 | 99)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn file_asset(&self) -> &crate::mechanical_port::source::assets::file_asset::FileAsset {
        self.base.base.file_asset()
    }
    pub fn copy<C: DrawableAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.base.base.copy(&object.base.base, callbacks);
    }
    pub fn clone_into<C: DrawableAssetBaseCallbacks>(&self, callbacks: &mut C) -> ImageAsset {
        let mut cloned = ImageAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
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
