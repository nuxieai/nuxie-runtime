use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
    },
    data_bind_flags::DataBindFlags,
};
use crate::mechanical_port::source::generated::data_bind::converters::data_converter_system_normalizer_base::DataConverterSystemNormalizerBase;
#[derive(Default)]
pub struct DataConverterSystemNormalizer {
    pub base: DataConverterSystemNormalizerBase,
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterSystemNormalizer
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        let flags = data_bind
            .with(|bind| bind.as_data_bind().map(|bind| bind.base.flags()))
            .flatten()
            .unwrap_or(0);
        let value = Self::convert(self, input, flags);
        output(value.as_ref());
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        let flags = data_bind
            .with(|bind| bind.as_data_bind().map(|bind| bind.base.flags()))
            .flatten()
            .unwrap_or(0);
        let value = Self::reverse_convert(self, input, flags);
        output(value.as_ref());
    }

    fn output_type(&self) -> DataType {
        DataType::Number
    }

    crate::data_converter_capability_lifecycle!(base.base.base.base.base.base);
}
impl DataConverterSystemNormalizer {
    pub fn convert(&mut self, input: &dyn DataValue, flags: u32) -> Box<dyn DataValue> {
        if flags & u32::from(DataBindFlags::DIRECTION.0) == u32::from(DataBindFlags::TO_SOURCE.0) {
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
        if flags & u32::from(DataBindFlags::DIRECTION.0) == u32::from(DataBindFlags::TO_TARGET.0) {
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
