use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
    },
    generated::data_bind::converters::data_converter_range_mapper_base::{
        DataConverterRangeMapperBase, DataConverterRangeMapperBaseCallbacks,
    },
};
pub const CLAMP_LOWER: u32 = 1;
pub const CLAMP_UPPER: u32 = 2;
pub const MODULO: u32 = 4;
pub const REVERSE: u32 = 8;
pub struct DataConverterRangeMapper {
    pub base: DataConverterRangeMapperBase,
    interpolator: Option<CoreHandle>,
    output: DataValueNumber,
}

impl Default for DataConverterRangeMapper {
    fn default() -> Self {
        Self {
            base: DataConverterRangeMapperBase::default(),
            interpolator: None,
            output: DataValueNumber::default(),
        }
    }
}

impl DataConverterRangeMapper {
    pub fn new(
        min_input: f32,
        max_input: f32,
        min_output: f32,
        max_output: f32,
        flags: u32,
        interpolation_type: u32,
    ) -> Self {
        let mut converter = Self::default();
        if converter.base.set_min_input_value(min_input) {
            DataConverterRangeMapperBaseCallbacks::min_input_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterRangeMapperBase::MIN_INPUT_PROPERTY_KEY);
        }
        if converter.base.set_max_input_value(max_input) {
            DataConverterRangeMapperBaseCallbacks::max_input_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterRangeMapperBase::MAX_INPUT_PROPERTY_KEY);
        }
        if converter.base.set_min_output_value(min_output) {
            DataConverterRangeMapperBaseCallbacks::min_output_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterRangeMapperBase::MIN_OUTPUT_PROPERTY_KEY);
        }
        if converter.base.set_max_output_value(max_output) {
            DataConverterRangeMapperBaseCallbacks::max_output_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterRangeMapperBase::MAX_OUTPUT_PROPERTY_KEY);
        }
        if converter.base.set_flags_value(flags) {
            DataConverterRangeMapperBaseCallbacks::flags_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterRangeMapperBase::FLAGS_PROPERTY_KEY);
        }
        if converter
            .base
            .set_interpolation_type_value(interpolation_type)
        {
            DataConverterRangeMapperBaseCallbacks::interpolation_type_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(
                    DataConverterRangeMapperBase::INTERPOLATION_TYPE_PROPERTY_KEY,
                );
        }
        converter
    }
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    fn positive_mod(value: f32, range: f32) -> f32 {
        let range = range.abs();
        let mut result = value % range;
        if result < 0.0 {
            result += range;
        }
        result
    }
    fn calculate_range(
        &mut self,
        input: &dyn DataValue,
        min_input: f32,
        max_input: f32,
        min_output: f32,
        max_output: f32,
    ) -> &DataValueNumber {
        let result = if let Some(number) = input.as_any().downcast_ref::<DataValueNumber>() {
            if min_output == max_output {
                min_output
            } else {
                let mut value = number.value();
                if value < min_input && self.base.flags() & CLAMP_LOWER == CLAMP_LOWER {
                    value = min_input
                } else if value > max_input && self.base.flags() & CLAMP_UPPER == CLAMP_UPPER {
                    value = max_input
                }
                if (value < min_input || value > max_input) && self.base.flags() & MODULO == MODULO
                {
                    value = (Self::positive_mod(value, max_input - min_input) + min_input).abs();
                }
                let mut percentage = (value - min_input) / (max_input - min_input);
                if self.base.flags() & REVERSE == REVERSE {
                    percentage = 1.0 - percentage;
                }
                if let Some(interpolator) = self
                    .interpolator
                    .as_ref()
                    .filter(|_| percentage > 0.0 && percentage < 1.0)
                {
                    percentage = interpolator
                        .with_mut(|interpolator| {
                            interpolator.keyframe_interpolator_transform(percentage)
                        })
                        .flatten()
                        .unwrap_or(percentage);
                } else if self.base.interpolation_type() == 0 {
                    percentage = if percentage <= 0.0 { 0.0 } else { 1.0 };
                }
                percentage * max_output + (1.0 - percentage) * min_output
            }
        } else {
            DataValueNumber::DEFAULT_VALUE
        };
        self.output.set_value(result);
        &self.output
    }
    pub fn convert(&mut self, input: &dyn DataValue) -> &DataValueNumber {
        self.calculate_range(
            input,
            self.base.min_input(),
            self.base.max_input(),
            self.base.min_output(),
            self.base.max_output(),
        )
    }
    pub fn reverse_convert(&mut self, input: &dyn DataValue) -> &DataValueNumber {
        self.calculate_range(
            input,
            self.base.min_output(),
            self.base.max_output(),
            self.base.min_input(),
            self.base.max_input(),
        )
    }
    pub fn interpolator_id(&self) -> u32 {
        self.base.interpolator_id()
    }
    pub fn set_interpolator(&mut self, value: Option<CoreHandle>) {
        self.interpolator = value
    }
    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.interpolator.clone()
    }
    pub fn copy(&mut self, other: &Self) {
        self.interpolator = other.interpolator.clone();
        let mut base = std::mem::take(&mut self.base);
        base.copy(&other.base, self);
        self.base = base;
    }
    pub fn min_input_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
    pub fn max_input_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
    pub fn min_output_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
    pub fn max_output_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
}

impl DataConverterRangeMapperBaseCallbacks for DataConverterRangeMapper {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn min_input_changed(&mut self) {
        Self::min_input_changed(self);
    }

    fn max_input_changed(&mut self) {
        Self::max_input_changed(self);
    }

    fn min_output_changed(&mut self) {
        Self::min_output_changed(self);
    }

    fn max_output_changed(&mut self) {
        Self::max_output_changed(self);
    }
}

crate::impl_data_converter_capability_bidi!(DataConverterRangeMapper, base.base);
