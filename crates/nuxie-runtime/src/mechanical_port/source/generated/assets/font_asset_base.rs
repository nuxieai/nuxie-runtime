use crate::mechanical_port::source::{
    assets::{file_asset::FileAsset, font_asset::FontAsset},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub struct FontAssetBase {
    pub base: FileAsset,
}

impl Default for FontAssetBase {
    fn default() -> Self {
        Self {
            base: FileAsset::default(),
        }
    }
}

impl FontAssetBase {
    pub const TYPE_KEY: u16 = 141;
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
    pub fn copy<C: FileAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.base.base.copy(&object.base.base, callbacks);
    }
    pub fn clone_into<C: FileAssetBaseCallbacks>(&self, callbacks: &mut C) -> FontAsset {
        let mut cloned = FontAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
}
