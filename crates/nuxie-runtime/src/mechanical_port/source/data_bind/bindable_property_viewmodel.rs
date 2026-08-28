use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::data_bind::bindable_property_viewmodel_base::BindablePropertyViewModelBase,
};

#[derive(Default)]
pub struct BindablePropertyViewModel {
    pub base: BindablePropertyViewModelBase,
    view_model_instance: Option<CoreHandle>,
}
impl BindablePropertyViewModel {
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn set_view_model_instance_value(&mut self, value: Option<CoreHandle>) {
        self.view_model_instance = value
    }
    pub fn view_model_instance_value(&self) -> Option<CoreHandle> {
        self.view_model_instance.clone()
    }
    pub fn set_view_model_instance(&mut self, value: Option<CoreHandle>) {
        self.set_view_model_instance_value(value)
    }
    pub fn view_model_instance(&self) -> Option<CoreHandle> {
        self.view_model_instance_value()
    }
}
