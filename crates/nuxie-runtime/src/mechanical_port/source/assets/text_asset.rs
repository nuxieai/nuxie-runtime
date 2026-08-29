use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    generated::assets::{
        asset_base::AssetBaseCallbacks,
        file_asset_base::{FileAssetBase, FileAssetBaseCallbacks},
        text_asset_base::{TextAssetBase, TextAssetBaseCallbacks},
    },
};

pub struct TextAsset {
    pub base: TextAssetBase,
    verified: bool,
}

impl Default for TextAsset {
    fn default() -> Self {
        Self {
            base: TextAssetBase::default(),
            verified: false,
        }
    }
}

impl TextAsset {
    pub fn folder_path(&self) -> &str {
        self.base.folder_path()
    }

    pub fn set_folder_path(&mut self, value: String) {
        if self.base.set_folder_path_value(value) {
            self.base
                .base
                .base
                .base
                .base
                .base
                .notify_property_changed(TextAssetBase::FOLDER_PATH_PROPERTY_KEY);
        }
    }

    pub fn verified(&self) -> bool {
        self.verified
    }

    pub(crate) fn set_verified(&mut self, verified: bool) {
        self.verified = verified;
    }
}

impl AssetBaseCallbacks for TextAsset {
    fn notify_property_changed(&mut self, property_key: u16) {
        AssetBaseCallbacks::notify_property_changed(&mut self.base.base, property_key);
    }
}

impl FileAssetBaseCallbacks for TextAsset {
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

impl TextAssetBaseCallbacks for TextAsset {}

impl std::ops::Deref for TextAsset {
    type Target = TextAssetBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
