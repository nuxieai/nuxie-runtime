use crate::mechanical_port::source::generated::custom_property_enum_base::{
    CustomPropertyEnumBase, CustomPropertyEnumBaseCallbacks,
};

#[derive(Default)]
pub struct CustomPropertyEnum {
    pub base: CustomPropertyEnumBase,
}

impl CustomPropertyEnumBaseCallbacks for CustomPropertyEnum {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
