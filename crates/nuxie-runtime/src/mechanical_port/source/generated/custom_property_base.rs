use crate::mechanical_port::source::component::Component;

pub struct CustomPropertyBase {
    pub base: Component,
}
impl Default for CustomPropertyBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
        }
    }
}
impl CustomPropertyBase {
    pub const TYPE_KEY: u16 = 167;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
}
