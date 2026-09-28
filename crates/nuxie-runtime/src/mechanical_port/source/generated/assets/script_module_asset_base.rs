use crate::mechanical_port::source::{
    assets::{file_asset::FileAsset, script_module_asset::ScriptModuleAsset},
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub trait ScriptModuleAssetBaseCallbacks: FileAssetBaseCallbacks {
    fn language_changed(&mut self) {}
}

#[derive(Default)]
pub struct ScriptModuleAssetBase {
    pub base: FileAsset,
    language: u32,
}

impl ScriptModuleAssetBase {
    pub const TYPE_KEY: u16 = 1071;
    pub const LANGUAGE_PROPERTY_KEY: u16 = 1087;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, 1071 | 103 | 99)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn language(&self) -> u32 {
        self.language
    }
    pub(crate) fn set_language_value(&mut self, value: u32) -> bool {
        if self.language == value {
            return false;
        }
        self.language = value;
        true
    }
    pub fn set_language<C: ScriptModuleAssetBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if self.set_language_value(value) {
            callbacks.language_changed();
            FileAssetBaseCallbacks::notify_property_changed(callbacks, Self::LANGUAGE_PROPERTY_KEY);
        }
    }
    pub fn copy<C: ScriptModuleAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.language = object.language;
        self.base.base.copy(&object.base.base, callbacks);
    }
    pub fn clone_into<C: ScriptModuleAssetBaseCallbacks>(
        &self,
        callbacks: &mut C,
    ) -> ScriptModuleAsset {
        let mut cloned = ScriptModuleAsset::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn deserialize<C: ScriptModuleAssetBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        if key == Self::LANGUAGE_PROPERTY_KEY {
            self.language = CoreUintType::deserialize(reader);
            true
        } else {
            self.base.base.deserialize(key, reader, callbacks)
        }
    }
}

impl std::ops::Deref for ScriptModuleAssetBase {
    type Target = FileAsset;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ScriptModuleAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
