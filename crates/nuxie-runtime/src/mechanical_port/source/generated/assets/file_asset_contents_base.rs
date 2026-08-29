use crate::mechanical_port::source::{
    assets::file_asset_contents::FileAssetContents,
    core::{Core, binary_reader::BinaryReader, field_types::core_bytes_type::CoreBytesType},
};

pub trait FileAssetContentsBaseCallbacks {
    fn decode_bytes(&mut self, value: &[u8]);
    fn copy_bytes(&mut self, object: &FileAssetContentsBase);
    fn decode_signature(&mut self, value: &[u8]);
    fn copy_signature(&mut self, object: &FileAssetContentsBase);
    fn bytes_changed(&mut self) {}
    fn signature_changed(&mut self) {}
}

#[derive(Default)]
pub struct FileAssetContentsBase {
    pub base: Core,
}

impl FileAssetContentsBase {
    pub const TYPE_KEY: u16 = 106;
    pub const BYTES_PROPERTY_KEY: u16 = 212;
    pub const SIGNATURE_PROPERTY_KEY: u16 = 911;

    pub fn is_type_of(type_key: u16) -> bool {
        type_key == Self::TYPE_KEY
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn clone_into(
        &self,
        callbacks: &mut impl FileAssetContentsBaseCallbacks,
    ) -> FileAssetContents {
        let mut cloned = FileAssetContents::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FileAssetContentsBaseCallbacks) {
        callbacks.copy_bytes(object);
        callbacks.copy_signature(object);
    }

    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FileAssetContentsBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::BYTES_PROPERTY_KEY => {
                callbacks.decode_bytes(CoreBytesType::deserialize(reader).as_slice());
                true
            }
            Self::SIGNATURE_PROPERTY_KEY => {
                callbacks.decode_signature(CoreBytesType::deserialize(reader).as_slice());
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for FileAssetContentsBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FileAssetContentsBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
