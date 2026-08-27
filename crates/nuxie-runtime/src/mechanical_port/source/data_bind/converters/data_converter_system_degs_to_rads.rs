use super::data_converter_operation_value::DataConverterOperationValue;
use crate::mechanical_port::source::data_bind::data_values::data_value::DataValue;
pub const TO_SOURCE: u32 = 1;
pub const TO_TARGET: u32 = 2;
pub struct DataConverterSystemDegsToRads {
    operation: DataConverterOperationValue,
}
impl DataConverterSystemDegsToRads {
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue, flags: u32) -> Box<dyn DataValue> {
        if flags & TO_SOURCE == TO_SOURCE {
            self.operation.reverse_convert(input)
        } else {
            clone_value(self.operation.convert(input))
        }
    }
    pub fn reverse_convert<'a>(
        &'a mut self,
        input: &dyn DataValue,
        flags: u32,
    ) -> Box<dyn DataValue> {
        if flags & TO_TARGET == TO_TARGET {
            clone_value(self.operation.convert(input))
        } else {
            self.operation.reverse_convert(input)
        }
    }
}
fn clone_value(value: &dyn DataValue) -> Box<dyn DataValue> {
    let number=value.as_any().downcast_ref::<crate::mechanical_port::source::data_bind::data_values::data_value_number::DataValueNumber>().unwrap();
    Box::new(crate::mechanical_port::source::data_bind::data_values::data_value_number::DataValueNumber::new(number.value()))
}
