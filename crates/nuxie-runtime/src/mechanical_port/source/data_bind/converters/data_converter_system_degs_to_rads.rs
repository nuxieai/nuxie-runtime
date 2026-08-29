use crate::mechanical_port::source::{
    core::CoreHandle, data_bind::data_values::data_type::DataType,
    data_bind::data_values::data_value::DataValue,
    generated::data_bind::converters::data_converter_system_degs_to_rads_base::DataConverterSystemDegsToRadsBase,
};
pub const TO_SOURCE: u32 = 1;
pub const TO_TARGET: u32 = 2;
#[derive(Default)]
pub struct DataConverterSystemDegsToRads {
    pub base: DataConverterSystemDegsToRadsBase,
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterSystemDegsToRads
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
impl DataConverterSystemDegsToRads {
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue, flags: u32) -> Box<dyn DataValue> {
        if flags & TO_SOURCE == TO_SOURCE {
            self.base.base.reverse_convert(input)
        } else {
            clone_value(self.base.base.convert(input))
        }
    }
    pub fn reverse_convert<'a>(
        &'a mut self,
        input: &dyn DataValue,
        flags: u32,
    ) -> Box<dyn DataValue> {
        if flags & TO_TARGET == TO_TARGET {
            clone_value(self.base.base.convert(input))
        } else {
            self.base.base.reverse_convert(input)
        }
    }
}
fn clone_value(value: &dyn DataValue) -> Box<dyn DataValue> {
    let number=value.as_any().downcast_ref::<crate::mechanical_port::source::data_bind::data_values::data_value_number::DataValueNumber>().unwrap();
    Box::new(crate::mechanical_port::source::data_bind::data_values::data_value_number::DataValueNumber::new(number.value()))
}
