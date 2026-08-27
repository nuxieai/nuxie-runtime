use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait AssetBlobValue: ViewModelInstanceValue {
    type Blob;
    fn set_value(&self, value: *mut Self::Blob);
    #[cfg(feature = "testing")]
    fn asset_value(&self) -> *mut Self::Blob;
}
pub struct ViewModelInstanceAssetBlobRuntime<T: AssetBlobValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: AssetBlobValue> ViewModelInstanceAssetBlobRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn set_value(&self, value: *mut T::Blob) {
        self.base.value().set_value(value)
    }
    pub fn data_type(&self) -> DataType {
        DataType::AssetBlob
    }
    #[cfg(feature = "testing")]
    pub fn testing_value(&self) -> *mut T::Blob {
        self.base.value().asset_value()
    }
}
