use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
};
pub trait KeyFrameInterpolator {
    fn transform(&self, value: f32) -> f32;
}
pub const CLAMP_LOWER: u32 = 1;
pub const CLAMP_UPPER: u32 = 2;
pub const MODULO: u32 = 4;
pub const REVERSE: u32 = 8;
pub struct DataConverterRangeMapper {
    interpolator: Option<*mut dyn KeyFrameInterpolator>,
    min_input: f32,
    max_input: f32,
    min_output: f32,
    max_output: f32,
    flags: u32,
    interpolation_type: u32,
    output: DataValueNumber,
    dirty: bool,
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
        Self {
            interpolator: None,
            min_input,
            max_input,
            min_output,
            max_output,
            flags,
            interpolation_type,
            output: DataValueNumber::default(),
            dirty: false,
        }
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
                if value < min_input && self.flags & CLAMP_LOWER == CLAMP_LOWER {
                    value = min_input
                } else if value > max_input && self.flags & CLAMP_UPPER == CLAMP_UPPER {
                    value = max_input
                }
                if (value < min_input || value > max_input) && self.flags & MODULO == MODULO {
                    value = (Self::positive_mod(value, max_input - min_input) + min_input).abs();
                }
                let mut percentage = (value - min_input) / (max_input - min_input);
                if self.flags & REVERSE == REVERSE {
                    percentage = 1.0 - percentage;
                }
                if let Some(interpolator) = self
                    .interpolator
                    .filter(|_| percentage > 0.0 && percentage < 1.0)
                {
                    percentage = unsafe { (&*interpolator).transform(percentage) }
                } else if self.interpolation_type == 0 {
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
            self.min_input,
            self.max_input,
            self.min_output,
            self.max_output,
        )
    }
    pub fn reverse_convert(&mut self, input: &dyn DataValue) -> &DataValueNumber {
        self.calculate_range(
            input,
            self.min_output,
            self.max_output,
            self.min_input,
            self.max_input,
        )
    }
    pub fn set_interpolator(&mut self, value: Option<*mut dyn KeyFrameInterpolator>) {
        self.interpolator = value
    }
    pub fn interpolator(&self) -> Option<*mut dyn KeyFrameInterpolator> {
        self.interpolator
    }
    pub fn copy(&mut self, other: &Self) {
        self.interpolator = other.interpolator;
        self.min_input = other.min_input;
        self.max_input = other.max_input;
        self.min_output = other.min_output;
        self.max_output = other.max_output;
        self.flags = other.flags;
        self.interpolation_type = other.interpolation_type;
    }
    pub fn min_input_changed(&mut self) {
        self.dirty = true
    }
    pub fn max_input_changed(&mut self) {
        self.dirty = true
    }
    pub fn min_output_changed(&mut self) {
        self.dirty = true
    }
    pub fn max_output_changed(&mut self) {
        self.dirty = true
    }
}
