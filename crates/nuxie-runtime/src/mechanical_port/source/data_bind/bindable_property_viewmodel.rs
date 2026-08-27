use std::ptr::NonNull;

use crate::mechanical_port::source::{
    generated::data_bind::bindable_property_viewmodel_base::BindablePropertyViewModelBase,
    viewmodel::viewmodel_instance::ViewModelInstance,
};

#[derive(Default)]
pub struct BindablePropertyViewModel {
    pub base: BindablePropertyViewModelBase,
    view_model_instance: Option<NonNull<ViewModelInstance>>,
}
impl BindablePropertyViewModel {
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn set_view_model_instance_value(&mut self, value: Option<NonNull<ViewModelInstance>>) {
        self.view_model_instance = value
    }
    pub fn view_model_instance_value(&self) -> Option<NonNull<ViewModelInstance>> {
        self.view_model_instance
    }
    pub fn set_view_model_instance(&mut self, value: Option<NonNull<ViewModelInstance>>) {
        self.set_view_model_instance_value(value)
    }
    pub fn view_model_instance(&self) -> Option<NonNull<ViewModelInstance>> {
        self.view_model_instance_value()
    }
}
