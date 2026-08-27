use super::{data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger};
use core::any::Any;
use std::rc::Rc;
pub trait BlobAsset: Any {}
pub struct DataValueAssetBlob {
    integer: DataValueInteger,
    file_asset: Option<Rc<dyn BlobAsset>>,
}
impl Default for DataValueAssetBlob {
    fn default() -> Self {
        Self::new(Self::DEFAULT_VALUE)
    }
}
impl DataValueAssetBlob {
    pub const TYPE_KEY: DataType = DataType::AssetBlob;
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn new(value: u32) -> Self {
        Self {
            integer: DataValueInteger::new(value),
            file_asset: None,
        }
    }
    pub fn value(&self) -> u32 {
        self.integer.value()
    }
    pub fn set_value(&mut self, value: u32) {
        self.integer.set_value(value)
    }
    pub fn file_asset(&self) -> Option<Rc<dyn BlobAsset>> {
        self.file_asset.clone()
    }
    pub fn set_blob_value(&mut self, blob: Option<Rc<dyn BlobAsset>>) {
        self.file_asset = blob
    }
    pub fn blob_value(&self) -> Option<&dyn BlobAsset> {
        self.file_asset.as_deref()
    }
}
impl DataValue for DataValueAssetBlob {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, t: DataType) -> bool {
        t == DataType::AssetBlob || t == DataType::Integer
    }
}
