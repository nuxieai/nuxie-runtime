use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait AssetImageValue: ViewModelInstanceValue {
    type Image;
    fn set_value(&self, value: *mut Self::Image);
    #[cfg(feature = "testing")]
    fn image_value(&self) -> *mut Self::Image;
}
pub struct ViewModelInstanceAssetImageRuntime<T: AssetImageValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: AssetImageValue> ViewModelInstanceAssetImageRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn set_value(&self, value: *mut T::Image) {
        self.base.value().set_value(value)
    }
    pub fn data_type(&self) -> DataType {
        DataType::AssetImage
    }
    #[cfg(feature = "testing")]
    pub fn testing_value(&self) -> *mut T::Image {
        self.base.value().image_value()
    }
}
