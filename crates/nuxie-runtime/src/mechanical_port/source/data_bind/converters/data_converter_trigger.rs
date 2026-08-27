use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_integer::DataValueInteger,
    data_value_trigger::DataValueTrigger,
};
#[derive(Default)]
pub struct DataConverterTrigger {
    output: DataValueTrigger,
}
impl DataConverterTrigger {
    pub fn output_type(&self) -> DataType {
        DataType::Trigger
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let value = input
            .as_any()
            .downcast_ref::<DataValueInteger>()
            .map_or(0, |value| value.value().wrapping_add(1));
        self.output.set_value(value);
        &self.output
    }
}
