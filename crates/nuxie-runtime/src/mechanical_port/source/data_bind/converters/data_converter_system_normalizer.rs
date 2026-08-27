use crate::mechanical_port::source::data_bind::data_values::{
    data_value::DataValue, data_value_number::DataValueNumber,
};
use crate::mechanical_port::source::generated::data_bind::converters::data_converter_system_normalizer_base::DataConverterSystemNormalizerBase;
pub const TO_SOURCE: u32 = 1;
pub const TO_TARGET: u32 = 2;
#[derive(Default)]
pub struct DataConverterSystemNormalizer {
    pub base: DataConverterSystemNormalizerBase,
}
impl DataConverterSystemNormalizer {
    pub fn convert(&mut self, input: &dyn DataValue, flags: u32) -> Box<dyn DataValue> {
        if flags & TO_SOURCE == TO_SOURCE {
            self.base.base.reverse_convert(input)
        } else {
            let output = self
                .base
                .base
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
                .base
                .base
                .convert(input)
                .as_any()
                .downcast_ref::<DataValueNumber>()
                .unwrap();
            Box::new(DataValueNumber::new(output.value()))
        } else {
            self.base.base.reverse_convert(input)
        }
    }
}
