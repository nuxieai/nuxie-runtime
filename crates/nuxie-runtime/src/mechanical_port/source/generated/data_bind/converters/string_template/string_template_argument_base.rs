use crate::source::core::Core;
use crate::source::core::binary_reader::BinaryReader;
pub struct StringTemplateArgumentBase {
    pub base: Core,
    name: String,
}
impl Default for StringTemplateArgumentBase {
    fn default() -> Self {
        Self {
            base: Default::default(),
            name: String::new(),
        }
    }
}
impl StringTemplateArgumentBase {
    pub const TYPE_KEY: u16 = 156;
    pub fn is_type_of(key: u16) -> bool {
        key == Self::TYPE_KEY
            || crate::source::generated::core_type_tree::has_ancestor(Self::TYPE_KEY, key)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub const NAME_PROPERTY_KEY: u16 = 481;
    pub fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn set_name_value(&mut self, value: String) -> bool {
        if self.name == value {
            return false;
        }
        self.name = value;
        true
    }
    pub fn copy(&mut self, other: &Self) {
        self.name = other.name.clone();
    }
    pub fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            481 => {
                self.name =
                    crate::source::core::field_types::core_string_type::CoreStringType::deserialize(
                        reader,
                    );
                true
            }
            _ => false,
        }
    }
}
impl std::ops::Deref for StringTemplateArgumentBase {
    type Target = Core;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StringTemplateArgumentBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
