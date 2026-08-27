use crate::mechanical_port::source::{
    assets::{asset::Asset, folder::Folder},
    generated::assets::asset_base::AssetBase,
};

pub struct FolderBase {
    pub base: Asset,
}

impl Default for FolderBase {
    fn default() -> Self {
        Self {
            base: Asset::default(),
        }
    }
}

impl FolderBase {
    pub const TYPE_KEY: u16 = 102;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 99)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }

    pub fn clone_into(&self) -> Folder {
        let mut cloned = Folder::default();
        cloned.base.copy(self);
        cloned
    }
}
