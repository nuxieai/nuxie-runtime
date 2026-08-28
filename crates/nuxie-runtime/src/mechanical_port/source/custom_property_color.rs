use crate::mechanical_port::source::generated::custom_property_color_base::{
    CustomPropertyColorBase, CustomPropertyColorBaseCallbacks,
};

#[derive(Default)]
pub struct CustomPropertyColor {
    pub base: CustomPropertyColorBase,
}

impl CustomPropertyColorBaseCallbacks for CustomPropertyColor {
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

impl std::ops::Deref for CustomPropertyColor {
    type Target = CustomPropertyColorBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomPropertyColor {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
