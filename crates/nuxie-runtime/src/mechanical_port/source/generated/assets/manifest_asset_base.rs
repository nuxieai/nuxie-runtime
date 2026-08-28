use crate::mechanical_port::source::{
    assets::{file_asset::FileAsset, manifest_asset::ManifestAsset},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub struct ManifestAssetBase {
    pub base: FileAsset,
}

impl Default for ManifestAssetBase {
    fn default() -> Self {
        Self {
            base: FileAsset::default(),
        }
    }
}

impl ManifestAssetBase {
    pub const TYPE_KEY: u16 = 642;
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
    pub fn clone_into<C: FileAssetBaseCallbacks>(&self, callbacks: &mut C) -> ManifestAsset {
        let mut cloned = ManifestAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
}

impl std::ops::Deref for ManifestAssetBase {
    type Target = FileAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ManifestAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
