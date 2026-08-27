use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait AssetFontValue: ViewModelInstanceValue {
    type Font;
    fn set_value(&self, value: *mut Self::Font);
    #[cfg(feature = "testing")]
    fn font_value(&self) -> *mut Self::Font;
}
pub struct ViewModelInstanceAssetFontRuntime<T: AssetFontValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: AssetFontValue> ViewModelInstanceAssetFontRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn set_value(&self, value: *mut T::Font) {
        self.base.value().set_value(value)
    }
    pub fn data_type(&self) -> DataType {
        DataType::AssetFont
    }
    #[cfg(feature = "testing")]
    pub fn testing_value(&self) -> *mut T::Font {
        self.base.value().font_value()
    }
}
