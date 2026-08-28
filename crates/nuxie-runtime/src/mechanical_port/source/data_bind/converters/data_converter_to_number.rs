use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_boolean::DataValueBoolean,
        data_value_color::DataValueColor, data_value_enum::DataValueEnum,
        data_value_number::DataValueNumber, data_value_string::DataValueString,
        data_value_symbol_list_index::DataValueSymbolListIndex,
    },
    generated::data_bind::converters::data_converter_to_number_base::DataConverterToNumberBase,
};
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
            // Keep the pinned `std::atof` prefix/range semantics without a C
            // runtime dependency. This shared parser is also the authority
            // used by the existing Rust integration and remains available on
            // `wasm32-unknown-unknown`.
            nuxie_binary::data_converter_to_number_string_value(
                value.value().as_bytes(),
                self.output.value(),
            )
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

crate::impl_data_converter_capability_forward!(DataConverterToNumber, base.base);
