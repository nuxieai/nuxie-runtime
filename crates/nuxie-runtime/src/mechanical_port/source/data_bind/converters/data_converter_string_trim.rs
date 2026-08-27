use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_string::DataValueString,
    },
    generated::data_bind::converters::data_converter_string_trim_base::{
        DataConverterStringTrimBase, DataConverterStringTrimBaseCallbacks,
    },
};
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrimType {
    None = 0,
    Start = 1,
    End = 2,
    All = 3,
}
pub struct DataConverterStringTrim {
    pub base: DataConverterStringTrimBase,
    output: DataValueString,
}

impl Default for DataConverterStringTrim {
    fn default() -> Self {
        Self {
            base: DataConverterStringTrimBase::default(),
            output: DataValueString::default(),
        }
    }
}

impl DataConverterStringTrim {
    pub fn new(trim_type: TrimType) -> Self {
        let mut converter = Self::default();
        converter.base.set_trim_type(
            trim_type as u32,
            &mut DataConverterStringTrimInitializationCallbacks,
        );
        converter
    }
    fn ltrim(value: &mut String) {
        *value = value
            .trim_start_matches(char::is_ascii_whitespace)
            .to_owned()
    }
    fn rtrim(value: &mut String) {
        *value = value.trim_end_matches(char::is_ascii_whitespace).to_owned()
    }
    fn trim(value: &mut String) {
        Self::rtrim(value);
        Self::ltrim(value)
    }
    pub fn output_type(&self) -> DataType {
        DataType::String
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let mut value = input
            .as_any()
            .downcast_ref::<DataValueString>()
            .map_or_else(String::new, |value| value.value().to_owned());
        if input.as_any().is::<DataValueString>() {
            match self.base.trim_type() {
                mode if mode == TrimType::Start as u32 => Self::ltrim(&mut value),
                mode if mode == TrimType::End as u32 => Self::rtrim(&mut value),
                mode if mode == TrimType::All as u32 => Self::trim(&mut value),
                _ => {}
            }
        }
        self.output.set_value(value);
        &self.output
    }
    pub fn trim_type_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
}

impl DataConverterStringTrimBaseCallbacks for DataConverterStringTrim {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn trim_type_changed(&mut self) {
        Self::trim_type_changed(self);
    }
}

struct DataConverterStringTrimInitializationCallbacks;

impl DataConverterStringTrimBaseCallbacks for DataConverterStringTrimInitializationCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
