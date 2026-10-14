use crate::source::core::binary_reader::BinaryReader;
use crate::source::data_bind::converters::string_template::string_template_argument::StringTemplateArgument;
pub struct StringTemplateArgumentStringBase {
    pub base: StringTemplateArgument,
    property_value: String,
}
impl Default for StringTemplateArgumentStringBase {
    fn default() -> Self {
        Self {
            base: Default::default(),
            property_value: String::new(),
        }
    }
}
impl StringTemplateArgumentStringBase {
    pub const TYPE_KEY: u16 = 157;
    pub fn is_type_of(key: u16) -> bool {
        key == Self::TYPE_KEY
            || crate::source::generated::core_type_tree::has_ancestor(Self::TYPE_KEY, key)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 482;
    pub fn property_value(&self) -> &str {
        &self.property_value
    }
    pub(crate) fn set_property_value_value(&mut self, value: String) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn copy(&mut self, other: &Self) {
        self.property_value = other.property_value.clone();
        self.base.copy(&other.base);
    }
    pub fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        match key {
            482 => {
                self.property_value =
                    crate::source::core::field_types::core_string_type::CoreStringType::deserialize(
                        reader,
                    );
                true
            }
            _ => self.base.deserialize(key, reader),
        }
    }
}
impl std::ops::Deref for StringTemplateArgumentStringBase {
    type Target = StringTemplateArgument;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StringTemplateArgumentStringBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
