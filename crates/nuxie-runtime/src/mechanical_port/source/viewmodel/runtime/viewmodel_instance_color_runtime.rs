use super::viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime};
use crate::mechanical_port::source::viewmodel::viewmodel_instance_color::ViewModelInstanceColor;

#[derive(Clone)]
pub struct ViewModelInstanceColorRuntime {
    base: ViewModelInstanceValueRuntime,
}

impl ViewModelInstanceColorRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::Color).then_some(Self { base })
    }
    pub fn value(&self) -> i32 {
        self.base
            .handle()
            .with_downcast::<ViewModelInstanceColor, _>(ViewModelInstanceColor::value)
            .unwrap_or_default()
    }
    pub fn set_value(&self, value: i32) {
        self.base
            .handle()
            .with_downcast_mut::<ViewModelInstanceColor, _>(|property| property.set_value(value));
    }
    pub fn rgb(&self, r: i32, g: i32, b: i32) {
        let alpha = ((self.value() as u32 & 0xff00_0000) >> 24) as i32;
        self.argb(alpha, r, g, b);
    }
    pub fn alpha(&self, a: i32) {
        let color = self.value();
        self.argb(a, (color >> 16) & 0xff, (color >> 8) & 0xff, color & 0xff);
    }
    pub fn argb(&self, a: i32, r: i32, g: i32, b: i32) {
        self.set_value(
            (((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32) as i32,
        );
    }
    pub fn data_type(&self) -> DataType {
        DataType::Color
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
