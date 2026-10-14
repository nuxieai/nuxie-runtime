use crate::source::{
    core::CoreObject,
    generated::data_bind::converters::string_template::string_template_argument_number_base::StringTemplateArgumentNumberBase,
};
#[derive(Default)]
pub struct StringTemplateArgumentNumber {
    pub base: StringTemplateArgumentNumberBase,
}
impl StringTemplateArgumentNumber {
    pub fn new(name: String, value: f32) -> Self {
        let mut result = Self::default();
        result.base.base.base.set_name_value(name);
        result.base.set_property_value_value(value);
        result
    }
    pub fn name(&self) -> &str {
        self.base.base.name()
    }
    pub fn property_value(&self) -> f32 {
        self.base.property_value()
    }
    pub fn set_property_value(&mut self, value: f32) {
        if self.base.set_property_value_value(value) {
            self.core_mut().notify_property_changed(483);
        }
    }
    pub fn set_name(&mut self, name: String) {
        if self.base.base.base.set_name_value(name) {
            self.core_mut().notify_property_changed(481);
        }
    }
    pub fn append_value(&self, output: &mut String, spec: &str) {
        output.push_str(
            &super::string_template_format::StringTemplateFormat::format_number(
                self.base.property_value(),
                spec,
            ),
        );
    }
}
