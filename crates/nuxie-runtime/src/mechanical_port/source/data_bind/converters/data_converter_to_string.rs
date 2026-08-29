use crate::mechanical_port::source::{
    data_bind::{
        converters::data_converter_string_remove_zeros::DataConverterStringRemoveZeros,
        data_values::{
            data_type::DataType, data_value::DataValue, data_value_boolean::DataValueBoolean,
            data_value_color::DataValueColor, data_value_enum::DataValueEnum,
            data_value_number::DataValueNumber, data_value_string::DataValueString,
            data_value_symbol_list_index::DataValueSymbolListIndex,
            data_value_trigger::DataValueTrigger,
        },
    },
    generated::data_bind::converters::data_converter_to_string_base::{
        DataConverterToStringBase, DataConverterToStringBaseCallbacks,
    },
};

pub const ROUND: u32 = 1;
pub const TRAILING_ZEROS: u32 = 2;
pub const FORMAT_WITH_COMMAS: u32 = 4;

#[derive(Default)]
pub struct ColorConverter {
    h: i32,
    l: i32,
    s: i32,
    color: i32,
}

impl ColorConverter {
    pub fn set_color(&mut self, value: i32) {
        if self.color != value {
            self.color = value;
            self.h = -1;
            self.l = -1;
            self.s = -1;
        }
    }

    fn alpha(&self) -> i32 {
        (self.color >> 24) & 255
    }

    fn red(&self) -> i32 {
        (self.color >> 16) & 255
    }

    fn green(&self) -> i32 {
        (self.color >> 8) & 255
    }

    fn blue(&self) -> i32 {
        self.color & 255
    }

    fn calculate_hsl(&mut self) {
        let r = self.red() as f32 / 255.0;
        let g = self.green() as f32 / 255.0;
        let b = self.blue() as f32 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let delta = max - min;
        let mut hue = 0.0;
        if delta != 0.0 {
            hue = if max == r {
                ((g - b) / delta) % 6.0
            } else if max == g {
                (b - r) / delta + 2.0
            } else {
                (r - g) / delta + 4.0
            };
        }

        self.h = (hue * 60.0).round() as i32;
        if self.h < 0 {
            self.h += 360;
        }

        let lum = (max + min) / 2.0;
        let sat = if delta == 0.0 {
            0.0
        } else {
            delta / (1.0 - (2.0 * lum - 1.0).abs())
        };
        self.l = (lum * 100.0).round() as i32;
        self.s = (sat * 100.0).round() as i32;
    }

    fn marker(&mut self, marker: char) -> Option<String> {
        Some(match marker {
            'r' => self.red().to_string(),
            'g' => self.green().to_string(),
            'b' => self.blue().to_string(),
            'a' => self.alpha().to_string(),
            'R' => format!("{:02X}", self.red()),
            'G' => format!("{:02X}", self.green()),
            'B' => format!("{:02X}", self.blue()),
            'A' => format!("{:02X}", self.alpha()),
            'h' => {
                if self.h == -1 {
                    self.calculate_hsl();
                }
                self.h.to_string()
            }
            'l' => {
                if self.l == -1 {
                    self.calculate_hsl();
                }
                self.l.to_string()
            }
            's' => {
                if self.s == -1 {
                    self.calculate_hsl();
                }
                self.s.to_string()
            }
            _ => return None,
        })
    }
}

pub struct DataConverterToString {
    pub base: DataConverterToStringBase,
    output: DataValueString,
    converter: ColorConverter,
}

impl Default for DataConverterToString {
    fn default() -> Self {
        Self {
            base: DataConverterToStringBase::default(),
            output: DataValueString::default(),
            converter: ColorConverter::default(),
        }
    }
}

impl DataConverterToString {
    fn cpp_to_string(value: f32) -> String {
        if value.is_nan() {
            // Pinned std::to_string(float) delegates to the C `%f`
            // conversion, which spells both positive and negative NaNs as
            // lowercase `nan` on the pinned macOS validation host.
            "nan".to_owned()
        } else if value == f32::INFINITY {
            "inf".to_owned()
        } else if value == f32::NEG_INFINITY {
            "-inf".to_owned()
        } else {
            format!("{value:.6}")
        }
    }

    pub fn new(decimals: usize, color_format: String, flags: u32) -> Self {
        let mut converter = Self::default();
        if converter.base.set_decimals_value(decimals as u32) {
            DataConverterToStringBaseCallbacks::decimals_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterToStringBase::DECIMALS_PROPERTY_KEY);
        }
        if converter.base.set_color_format_value(color_format) {
            DataConverterToStringBaseCallbacks::color_format_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterToStringBase::COLOR_FORMAT_PROPERTY_KEY);
        }
        if converter.base.set_flags_value(flags) {
            DataConverterToStringBaseCallbacks::flags_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterToStringBase::FLAGS_PROPERTY_KEY);
        }
        converter
    }

    pub fn output_type(&self) -> DataType {
        DataType::String
    }

    fn format_with_commas(value: &str) -> String {
        let (integer, fraction) = if let Some(dot) = value.find('.') {
            (&value[..dot], &value[dot..])
        } else {
            (value, "")
        };
        let mut integer = integer.to_owned();
        let mut position = integer.len() as isize - 3;
        while position > 0 && integer.as_bytes()[position as usize - 1].is_ascii_digit() {
            integer.insert(position as usize, ',');
            position -= 3;
        }
        integer + fraction
    }

    fn convert_number(&mut self, value: f32) {
        let mut output = if self.base.flags() & ROUND == ROUND {
            format!("{:.*}", self.base.decimals() as usize, value)
        } else {
            Self::cpp_to_string(value)
        };
        if self.base.flags() & TRAILING_ZEROS == TRAILING_ZEROS {
            output = DataConverterStringRemoveZeros::remove_zeros(output);
        }
        if self.base.flags() & FORMAT_WITH_COMMAS == FORMAT_WITH_COMMAS {
            output = Self::format_with_commas(&output);
        }
        self.output.set_value(output);
    }

    fn convert_color(&mut self, value: i32) {
        if self.base.color_format().is_empty() {
            self.output.set_value(value.to_string());
            return;
        }

        self.converter.set_color(value);
        let mut output = String::new();
        let mut escaped = false;
        let mut marker = false;
        for character in self.base.color_format().chars() {
            if escaped {
                output.push(character);
                escaped = false;
            } else if character == '\\' {
                if marker {
                    output.push('%');
                    marker = false;
                }
                escaped = true;
            } else if character == '%' {
                if marker {
                    output.push('%');
                }
                marker = true;
            } else if marker {
                if let Some(replacement) = self.converter.marker(character) {
                    output.push_str(&replacement);
                } else {
                    output.push('%');
                    output.push(character);
                }
                marker = false;
            } else {
                output.push(character);
            }
        }
        self.output.set_value(output);
    }

    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        if let Some(value) = input.as_any().downcast_ref::<DataValueNumber>() {
            self.convert_number(value.value());
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueEnum>() {
            if let Some(data_enum) = value.data_enum() {
                self.output.set_value(data_enum.value(value.value()));
            }
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueString>() {
            self.output.set_value(value.value().to_owned());
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueColor>() {
            self.convert_color(value.value());
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueBoolean>() {
            self.output
                .set_value(if value.value() { "1" } else { "0" }.to_owned());
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueTrigger>() {
            self.output.set_value(value.value().to_string());
        } else if let Some(value) = input.as_any().downcast_ref::<DataValueSymbolListIndex>() {
            self.output.set_value(value.value().to_string());
        } else {
            self.output.set_value(String::new());
        }
        &self.output
    }

    pub fn decimals_changed(&mut self) {
        self.base.base.mark_converter_dirty();
    }

    pub fn color_format_changed(&mut self) {
        self.base.base.mark_converter_dirty();
    }
}

impl DataConverterToStringBaseCallbacks for DataConverterToString {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn decimals_changed(&mut self) {
        Self::decimals_changed(self);
    }

    fn color_format_changed(&mut self) {
        Self::color_format_changed(self);
    }
}

crate::impl_data_converter_capability_forward!(DataConverterToString, base.base);

#[cfg(test)]
mod tests {
    use super::DataConverterToString;

    #[test]
    fn unrounded_non_finite_numbers_match_pinned_std_to_string() {
        assert_eq!(DataConverterToString::cpp_to_string(f32::NAN), "nan");
        assert_eq!(
            DataConverterToString::cpp_to_string(f32::from_bits(0xffc0_0000)),
            "nan"
        );
        assert_eq!(DataConverterToString::cpp_to_string(f32::INFINITY), "inf");
        assert_eq!(
            DataConverterToString::cpp_to_string(f32::NEG_INFINITY),
            "-inf"
        );
        assert_eq!(DataConverterToString::cpp_to_string(-0.0), "-0.000000");
    }
}
