use crate::mechanical_port::source::generated::custom_property_boolean_base::{
    CustomPropertyBooleanBase, CustomPropertyBooleanBaseCallbacks,
};

#[derive(Default)]
pub struct CustomPropertyBoolean {
    pub base: CustomPropertyBooleanBase,
}

impl CustomPropertyBooleanBaseCallbacks for CustomPropertyBoolean {
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

impl std::ops::Deref for CustomPropertyBoolean {
    type Target = CustomPropertyBooleanBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyBoolean {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
