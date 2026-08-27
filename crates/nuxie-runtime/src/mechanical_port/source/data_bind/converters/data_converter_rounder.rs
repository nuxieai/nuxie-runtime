use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
};
pub struct DataConverterRounder {
    decimals: i32,
    output: DataValueNumber,
}
impl DataConverterRounder {
    pub fn new(decimals: i32) -> Self {
        Self {
            decimals,
            output: DataValueNumber::default(),
        }
    }
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let result = input.as_any().downcast_ref::<DataValueNumber>().map_or(
            DataValueNumber::DEFAULT_VALUE,
            |number| {
                let rounder = 10.0_f32.powf(self.decimals as f32);
                (number.value() * rounder).round() / rounder
            },
        );
        self.output.set_value(result);
        &self.output
    }
}
