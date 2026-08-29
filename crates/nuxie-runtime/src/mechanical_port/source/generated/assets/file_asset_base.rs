use crate::mechanical_port::source::{
    assets::asset::Asset,
    core::{
        binary_reader::BinaryReader,
        field_types::{
            core_bytes_type::CoreBytesType, core_string_type::CoreStringType,
            core_uint_type::CoreUintType,
        },
    },
};

pub trait FileAssetBaseCallbacks:
    crate::mechanical_port::source::generated::assets::asset_base::AssetBaseCallbacks
{
    fn asset_id_changed(&mut self) {}
    fn cdn_uuid_changed(&mut self) {}
    fn cdn_base_url_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
    fn decode_cdn_uuid(&mut self, value: &[u8]);
    fn copy_cdn_uuid(&mut self, object: &FileAssetBase);
}

pub struct FileAssetBase {
    pub base: Asset,
    asset_id: u32,
    cdn_base_url: String,
}

impl Default for FileAssetBase {
    fn default() -> Self {
        Self {
            base: Asset::default(),
            asset_id: 0,
            cdn_base_url: "https://public.rive.app/cdn/uuid".to_owned(),
        }
    }
}

impl FileAssetBase {
    pub const TYPE_KEY: u16 = 103;
    pub const ASSET_ID_PROPERTY_KEY: u16 = 204;
    pub const CDN_UUID_PROPERTY_KEY: u16 = 359;
    pub const CDN_BASE_URL_PROPERTY_KEY: u16 = 362;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 99)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn name(&self) -> &str {
        self.base.base.name()
    }

    pub fn asset_id(&self) -> u32 {
        self.asset_id
    }

    pub fn set_asset_id<C: FileAssetBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_asset_id_value(value) {
            return;
        }
        callbacks.asset_id_changed();
        FileAssetBaseCallbacks::notify_property_changed(callbacks, Self::ASSET_ID_PROPERTY_KEY);
    }
    pub(crate) fn set_asset_id_value(&mut self, value: u32) -> bool {
        if self.asset_id == value {
            return false;
        }
        self.asset_id = value;
        true
    }

    pub fn cdn_base_url(&self) -> &str {
        &self.cdn_base_url
    }

    pub fn set_cdn_base_url<C: FileAssetBaseCallbacks>(
        &mut self,
        value: String,
        callbacks: &mut C,
    ) {
        if !self.set_cdn_base_url_value(value) {
            return;
        }
        callbacks.cdn_base_url_changed();
        FileAssetBaseCallbacks::notify_property_changed(callbacks, Self::CDN_BASE_URL_PROPERTY_KEY);
    }
    pub(crate) fn set_cdn_base_url_value(&mut self, value: String) -> bool {
        if self.cdn_base_url == value {
            return false;
        }
        self.cdn_base_url = value;
        true
    }

    pub fn copy<C: FileAssetBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.asset_id = object.asset_id;
        callbacks.copy_cdn_uuid(object);
        self.cdn_base_url.clone_from(&object.cdn_base_url);
        self.base.base.copy(&object.base.base);
    }

    pub fn deserialize<C: FileAssetBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::ASSET_ID_PROPERTY_KEY => {
                self.asset_id = CoreUintType::deserialize(reader);
                true
            }
            Self::CDN_UUID_PROPERTY_KEY => {
                callbacks.decode_cdn_uuid(CoreBytesType::deserialize(reader).as_slice());
                true
            }
            Self::CDN_BASE_URL_PROPERTY_KEY => {
                self.cdn_base_url = CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader),
        }
    }
}

impl std::ops::Deref for FileAssetBase {
    type Target = Asset;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FileAssetBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
