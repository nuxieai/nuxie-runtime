use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    core::{binary_reader::BinaryReader, field_types::core_string_type::CoreStringType},
    generated::assets::file_asset_base::FileAssetBaseCallbacks,
};

pub trait TextAssetBaseCallbacks: FileAssetBaseCallbacks {
    fn folder_path_changed(&mut self) {}
}

pub struct TextAssetBase {
    pub base: FileAsset,
    folder_path: String,
}

impl Default for TextAssetBase {
    fn default() -> Self {
        Self {
            base: FileAsset::default(),
            folder_path: String::new(),
        }
    }
}

impl TextAssetBase {
    pub const TYPE_KEY: u16 = 971;
    pub const FOLDER_PATH_PROPERTY_KEY: u16 = 926;

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

    pub fn folder_path(&self) -> &str {
        &self.folder_path
    }

    pub fn set_folder_path<C: TextAssetBaseCallbacks>(&mut self, value: String, callbacks: &mut C) {
        if !self.set_folder_path_value(value) {
            return;
        }
        callbacks.folder_path_changed();
        callbacks.notify_property_changed(Self::FOLDER_PATH_PROPERTY_KEY);
    }
    pub(crate) fn set_folder_path_value(&mut self, value: String) -> bool {
        if self.folder_path == value {
            return false;
        }
        self.folder_path = value;
        true
    }

    pub fn copy<C: TextAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.folder_path.clone_from(&object.folder_path);
        self.base.base.copy(&object.base.base, callbacks);
    }

    pub fn deserialize<C: TextAssetBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::FOLDER_PATH_PROPERTY_KEY => {
                self.folder_path = CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextAssetBase {
    type Target = FileAsset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
