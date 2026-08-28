use crate::mechanical_port::source::generated::custom_property_base::CustomPropertyBase;

#[derive(Default)]
pub struct CustomProperty {
    pub base: CustomPropertyBase,
}

impl std::ops::Deref for CustomProperty {
    type Target = CustomPropertyBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CustomProperty {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
