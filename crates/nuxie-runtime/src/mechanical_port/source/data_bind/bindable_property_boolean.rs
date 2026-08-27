use crate::mechanical_port::source::generated::data_bind::bindable_property_boolean_base::{
    BindablePropertyBooleanBase, BindablePropertyBooleanBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyBoolean {
    pub base: BindablePropertyBooleanBase,
}
impl BindablePropertyBooleanBaseCallbacks for BindablePropertyBoolean {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
impl BindablePropertyBoolean {
    pub const DEFAULT_VALUE: bool = false;
}
