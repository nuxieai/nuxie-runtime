use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
    },
    generated::data_bind::converters::data_converter_rounder_base::{
        DataConverterRounderBase, DataConverterRounderBaseCallbacks,
    },
};
pub struct DataConverterRounder {
    pub base: DataConverterRounderBase,
    output: DataValueNumber,
}

impl Default for DataConverterRounder {
    fn default() -> Self {
        Self {
            base: DataConverterRounderBase::default(),
            output: DataValueNumber::default(),
        }
    }
}

impl DataConverterRounder {
    pub fn new(decimals: u32) -> Self {
        let mut converter = Self::default();
        if converter.base.set_decimals_value(decimals) {
            DataConverterRounderBaseCallbacks::decimals_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterRounderBase::DECIMALS_PROPERTY_KEY);
        }
        converter
    }
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let result = input.as_any().downcast_ref::<DataValueNumber>().map_or(
            DataValueNumber::DEFAULT_VALUE,
            |number| {
                let rounder = 10.0_f32.powf(self.base.decimals() as f32);
                (number.value() * rounder).round() / rounder
            },
        );
        self.output.set_value(result);
        &self.output
    }
}

crate::impl_data_converter_capability_forward!(DataConverterRounder, base.base);
