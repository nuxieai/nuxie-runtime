use crate::mechanical_port::source::generated::data_bind::bindable_property_number_base::{
    BindablePropertyNumberBase, BindablePropertyNumberBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyNumber {
    pub base: BindablePropertyNumberBase,
}
impl BindablePropertyNumberBaseCallbacks for BindablePropertyNumber {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
impl BindablePropertyNumber {
    pub const DEFAULT_VALUE: f32 = 0.0;
}
