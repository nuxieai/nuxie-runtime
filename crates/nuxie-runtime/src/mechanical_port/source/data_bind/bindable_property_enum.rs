use crate::mechanical_port::source::generated::data_bind::bindable_property_enum_base::{
    BindablePropertyEnumBase, BindablePropertyEnumBaseCallbacks,
};

#[derive(Default)]
pub struct BindablePropertyEnum {
    pub base: BindablePropertyEnumBase,
}
impl BindablePropertyEnumBaseCallbacks for BindablePropertyEnum {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
impl BindablePropertyEnum {
    pub const DEFAULT_VALUE: u16 = 0;
}
