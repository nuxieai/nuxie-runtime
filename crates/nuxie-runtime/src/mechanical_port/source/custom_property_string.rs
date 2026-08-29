use crate::mechanical_port::source::generated::custom_property_string_base::{
    CustomPropertyStringBase, CustomPropertyStringBaseCallbacks,
};

#[derive(Default)]
pub struct CustomPropertyString {
    pub base: CustomPropertyStringBase,
}

impl CustomPropertyStringBaseCallbacks for CustomPropertyString {
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

impl std::ops::Deref for CustomPropertyString {
    type Target = CustomPropertyStringBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyString {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
