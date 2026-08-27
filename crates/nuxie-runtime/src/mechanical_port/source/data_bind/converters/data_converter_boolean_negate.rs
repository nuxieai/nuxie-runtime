use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_boolean::DataValueBoolean,
};
#[derive(Default)]
pub struct DataConverterBooleanNegate {
    output: DataValueBoolean,
}
impl DataConverterBooleanNegate {
    pub fn output_type(&self) -> DataType {
        DataType::Boolean
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        self.output.set_value(
            input
                .as_any()
                .downcast_ref::<DataValueBoolean>()
                .map_or(DataValueBoolean::DEFAULT_VALUE, |value| !value.value()),
        );
        &self.output
    }
    pub fn reverse_convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        self.convert(input)
    }
}
