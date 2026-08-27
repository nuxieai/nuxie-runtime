use crate::mechanical_port::source::{
    assets::{blob_asset::BlobAsset, file_asset::FileAsset},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub struct BlobAssetBase {
    pub base: FileAsset,
}

impl Default for BlobAssetBase {
    fn default() -> Self {
        Self {
            base: FileAsset::default(),
        }
    }
}

impl BlobAssetBase {
    pub const TYPE_KEY: u16 = 649;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 103 | 99)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn file_asset(&self) -> &FileAsset {
        &self.base
    }

    pub fn copy<C: FileAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn clone_into<C: FileAssetBaseCallbacks>(&self, callbacks: &mut C) -> BlobAsset {
        let mut cloned = BlobAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
}
