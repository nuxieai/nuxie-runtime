use crate::source::{
    core::CoreObject,
    generated::data_bind::converters::string_template::string_template_argument_string_base::StringTemplateArgumentStringBase,
};
#[derive(Default)]
pub struct StringTemplateArgumentString {
    pub base: StringTemplateArgumentStringBase,
}
impl StringTemplateArgumentString {
    pub fn new(name: String, value: String) -> Self {
        let mut result = Self::default();
        result.base.base.base.set_name_value(name);
        result.base.set_property_value_value(value);
        result
    }
    pub fn name(&self) -> &str {
        self.base.base.name()
    }
    pub fn property_value(&self) -> &str {
        self.base.property_value()
    }
    pub fn set_property_value(&mut self, value: String) {
        if self.base.set_property_value_value(value) {
            self.core_mut().notify_property_changed(482);
        }
    }
    pub fn set_name(&mut self, name: String) {
        if self.base.base.base.set_name_value(name) {
            self.core_mut().notify_property_changed(481);
        }
    }
    pub fn append_value(&self, output: &mut String, _spec: &str) {
        output.push_str(self.base.property_value());
    }
}
