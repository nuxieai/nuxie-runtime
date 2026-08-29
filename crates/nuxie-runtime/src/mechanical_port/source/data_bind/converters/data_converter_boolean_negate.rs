use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_boolean::DataValueBoolean,
    },
    generated::data_bind::converters::data_converter_boolean_negate_base::DataConverterBooleanNegateBase,
};
#[derive(Default)]
pub struct DataConverterBooleanNegate {
    pub base: DataConverterBooleanNegateBase,
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

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterBooleanNegate
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::convert(self, input));
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::reverse_convert(self, input));
    }

    fn output_type(&self) -> DataType {
        Self::output_type(self)
    }

    crate::data_converter_capability_lifecycle!(base.base);
}
