#[derive(Debug)]
pub struct BindablePropertyViewModel {
    view_model_instance: *mut (),
}
impl Default for BindablePropertyViewModel {
    fn default() -> Self {
        Self {
            view_model_instance: core::ptr::null_mut(),
        }
    }
}
impl BindablePropertyViewModel {
    pub const DEFAULT_VALUE: u32 = u32::MAX;
    pub fn set_view_model_instance_value(&mut self, value: *mut ()) {
        self.view_model_instance = value
    }
    pub fn view_model_instance_value(&self) -> *mut () {
        self.view_model_instance
    }
    pub fn set_view_model_instance(&mut self, value: *mut ()) {
        self.set_view_model_instance_value(value)
    }
    pub fn view_model_instance(&self) -> *mut () {
        self.view_model_instance_value()
    }
}
