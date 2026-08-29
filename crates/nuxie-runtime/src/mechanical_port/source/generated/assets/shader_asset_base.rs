use crate::mechanical_port::source::{
    assets::{shader_asset::ShaderAsset, text_asset::TextAsset},
    generated::assets::text_asset_base::TextAssetBaseCallbacks,
};

pub struct ShaderAssetBase {
    pub base: TextAsset,
}

impl Default for ShaderAssetBase {
    fn default() -> Self {
        Self {
            base: TextAsset::default(),
        }
    }
}

impl ShaderAssetBase {
    pub const TYPE_KEY: u16 = 970;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 971 | 103 | 99)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy<C: TextAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.base.base.copy(&object.base.base, callbacks);
    }
    pub fn clone_into<C: TextAssetBaseCallbacks>(&self, callbacks: &mut C) -> ShaderAsset {
        let mut cloned = ShaderAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
}

impl std::ops::Deref for ShaderAssetBase {
    type Target = TextAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ShaderAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
