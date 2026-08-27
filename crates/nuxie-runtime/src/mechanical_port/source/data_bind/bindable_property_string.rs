use crate::mechanical_port::source::generated::data_bind::bindable_property_string_base::{
    BindablePropertyStringBase, BindablePropertyStringBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyString {
    pub base: BindablePropertyStringBase,
}
impl BindablePropertyStringBaseCallbacks for BindablePropertyString {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
impl BindablePropertyString {
    pub const DEFAULT_VALUE: &'static str = "";
}
