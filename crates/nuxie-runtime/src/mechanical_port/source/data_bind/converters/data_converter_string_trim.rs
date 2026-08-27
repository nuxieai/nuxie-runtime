use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_string::DataValueString,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TrimType {
    Start,
    End,
    All,
    Other,
}
pub struct DataConverterStringTrim {
    trim_type: TrimType,
    output: DataValueString,
    dirty: bool,
}
impl DataConverterStringTrim {
    pub fn new(trim_type: TrimType) -> Self {
        Self {
            trim_type,
            output: DataValueString::default(),
            dirty: false,
        }
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
            match self.trim_type {
                TrimType::Start => Self::ltrim(&mut value),
                TrimType::End => Self::rtrim(&mut value),
                TrimType::All => Self::trim(&mut value),
                TrimType::Other => {}
            }
        }
        self.output.set_value(value);
        &self.output
    }
    pub fn trim_type_changed(&mut self) {
        self.dirty = true
    }
}
