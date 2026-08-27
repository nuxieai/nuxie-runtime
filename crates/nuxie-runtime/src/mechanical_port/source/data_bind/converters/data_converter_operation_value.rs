use super::data_converter_operation::{ArithmeticOperation, DataConverterOperation};
use crate::mechanical_port::source::data_bind::data_values::data_value::DataValue;
pub struct DataConverterOperationValue {
    base: DataConverterOperation,
    operation_value: f32,
}
impl DataConverterOperationValue {
    pub fn new(operation: ArithmeticOperation, operation_value: f32) -> Self {
        Self {
            base: DataConverterOperation::new(operation),
            operation_value,
        }
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        self.base.convert_value(input, self.operation_value)
    }
    pub fn reverse_convert(&self, input: &dyn DataValue) -> Box<dyn DataValue> {
        Box::new(self.base.reverse_convert_value(input, self.operation_value))
    }
    pub fn operation_value_changed(&mut self) {
        self.base.mark_converter_dirty()
    }
}
