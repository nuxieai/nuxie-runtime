use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::rc::Rc;
pub trait ColorValue: ViewModelInstanceValue {
    fn property_value(&self) -> i32;
    fn set_property_value(&self, value: i32);
}
pub struct ViewModelInstanceColorRuntime<T: ColorValue> {
    base: ViewModelInstanceValueRuntime<T>,
}
impl<T: ColorValue> ViewModelInstanceColorRuntime<T> {
    pub fn new(value: Rc<T>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
        }
    }
    pub fn value(&self) -> i32 {
        self.base.value().property_value()
    }
    pub fn set_value(&self, value: i32) {
        self.base.value().set_property_value(value)
    }
    pub fn rgb(&self, r: i32, g: i32, b: i32) {
        let alpha = ((self.value() as u32 & 0xff00_0000) >> 24) as i32;
        self.argb(alpha, r, g, b)
    }
    pub fn alpha(&self, a: i32) {
        let color = self.value();
        self.argb(a, (color >> 16) & 0xff, (color >> 8) & 0xff, color & 0xff)
    }
    pub fn argb(&self, a: i32, r: i32, g: i32, b: i32) {
        self.set_value(
            (((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32) as i32,
        )
    }
    pub fn data_type(&self) -> DataType {
        DataType::Color
    }
}
