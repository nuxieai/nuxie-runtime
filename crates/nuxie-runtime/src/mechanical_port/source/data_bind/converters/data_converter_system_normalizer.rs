use super::data_converter_operation_value::DataConverterOperationValue;
use crate::mechanical_port::source::data_bind::data_values::{
    data_value::DataValue, data_value_number::DataValueNumber,
};
pub const TO_SOURCE: u32 = 1;
pub const TO_TARGET: u32 = 2;
pub struct DataConverterSystemNormalizer {
    operation: DataConverterOperationValue,
}
impl DataConverterSystemNormalizer {
    pub fn convert(&mut self, input: &dyn DataValue, flags: u32) -> Box<dyn DataValue> {
        if flags & TO_SOURCE == TO_SOURCE {
            self.operation.reverse_convert(input)
        } else {
            let output = self
                .operation
                .convert(input)
                .as_any()
                .downcast_ref::<DataValueNumber>()
                .unwrap();
            Box::new(DataValueNumber::new(output.value()))
        }
    }
    pub fn reverse_convert(&mut self, input: &dyn DataValue, flags: u32) -> Box<dyn DataValue> {
        if flags & TO_TARGET == TO_TARGET {
            let output = self
                .operation
                .convert(input)
                .as_any()
                .downcast_ref::<DataValueNumber>()
                .unwrap();
            Box::new(DataValueNumber::new(output.value()))
        } else {
            self.operation.reverse_convert(input)
        }
    }
}
