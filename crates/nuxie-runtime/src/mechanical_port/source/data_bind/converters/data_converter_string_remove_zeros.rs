use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_string::DataValueString,
    },
    generated::data_bind::converters::data_converter_string_remove_zeros_base::DataConverterStringRemoveZerosBase,
};
#[derive(Default)]
pub struct DataConverterStringRemoveZeros {
    pub base: DataConverterStringRemoveZerosBase,
    output: DataValueString,
}
impl DataConverterStringRemoveZeros {
    pub fn remove_zeros(mut value: String) -> String {
        if value.contains('.') {
            while value.ends_with('0') {
                value.pop();
            }
            if value.ends_with('.') {
                value.pop();
            }
        }
        value
    }
    pub fn output_type(&self) -> DataType {
        DataType::String
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let value = input
            .as_any()
            .downcast_ref::<DataValueString>()
            .map_or_else(String::new, |input| {
                Self::remove_zeros(input.value().to_owned())
            });
        self.output.set_value(value);
        &self.output
    }
}

crate::impl_data_converter_capability_forward!(DataConverterStringRemoveZeros, base.base);
