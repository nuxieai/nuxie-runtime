use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_list::DataValueList,
        data_value_number::DataValueNumber,
    },
    generated::data_bind::converters::data_converter_list_to_length_base::DataConverterListToLengthBase,
};
#[derive(Default)]
pub struct DataConverterListToLength {
    pub base: DataConverterListToLengthBase,
    output: DataValueNumber,
}
impl DataConverterListToLength {
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    pub fn convert<'a>(&'a mut self, input: &mut dyn DataValue) -> &'a dyn DataValue {
        let value = input
            .as_any_mut()
            .downcast_mut::<DataValueList>()
            .map_or(DataValueNumber::DEFAULT_VALUE, |list| {
                list.value().len() as f32
            });
        self.output.set_value(value);
        &self.output
    }
}
