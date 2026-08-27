use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_boolean::DataValueBoolean,
        data_value_color::DataValueColor, data_value_enum::DataValueEnum,
        data_value_number::DataValueNumber, data_value_string::DataValueString,
        data_value_symbol_list_index::DataValueSymbolListIndex,
    },
    generated::data_bind::converters::data_converter_to_number_base::DataConverterToNumberBase,
};
use std::ffi::{CString, c_char, c_double};
unsafe extern "C" {
    fn atof(value: *const c_char) -> c_double;
}
#[derive(Default)]
pub struct DataConverterToNumber {
    pub base: DataConverterToNumberBase,
    output: DataValueNumber,
}
impl DataConverterToNumber {
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let value = if let Some(value) = input.as_any().downcast_ref::<DataValueString>() {
            CString::new(value.value())
                .ok()
                .map_or(self.output.value(), |text| unsafe {
                    atof(text.as_ptr()) as f32
                })
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueEnum>() {
            value.value() as f32
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueNumber>() {
            value.value()
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueColor>() {
            value.value() as f32
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueBoolean>() {
            if value.value() { 1.0 } else { 0.0 }
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueSymbolListIndex>() {
            value.value() as f32
        } else {
            DataValueNumber::DEFAULT_VALUE
        };
        self.output.set_value(value);
        &self.output
    }
}
