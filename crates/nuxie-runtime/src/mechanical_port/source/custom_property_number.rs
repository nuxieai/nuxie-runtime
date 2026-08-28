use crate::mechanical_port::source::generated::custom_property_number_base::{
    CustomPropertyNumberBase, CustomPropertyNumberBaseCallbacks,
};

#[derive(Default)]
pub struct CustomPropertyNumber {
    pub base: CustomPropertyNumberBase,
}

impl CustomPropertyNumberBaseCallbacks for CustomPropertyNumber {
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

impl std::ops::Deref for CustomPropertyNumber {
    type Target = CustomPropertyNumberBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
