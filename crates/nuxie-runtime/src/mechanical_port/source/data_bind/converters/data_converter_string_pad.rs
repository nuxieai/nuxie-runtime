use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_string::DataValueString,
    },
    generated::data_bind::converters::data_converter_string_pad_base::{
        DataConverterStringPadBase, DataConverterStringPadBaseCallbacks,
    },
};
pub struct DataConverterStringPad {
    pub base: DataConverterStringPadBase,
    output: DataValueString,
}

impl Default for DataConverterStringPad {
    fn default() -> Self {
        Self {
            base: DataConverterStringPadBase::default(),
            output: DataValueString::default(),
        }
    }
}

impl DataConverterStringPad {
    pub fn new(length: usize, pad_type: u32, text: String) -> Self {
        let mut converter = Self::default();
        if converter.base.set_length_value(length as u32) {
            DataConverterStringPadBaseCallbacks::length_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterStringPadBase::LENGTH_PROPERTY_KEY);
        }
        if converter.base.set_pad_type_value(pad_type) {
            DataConverterStringPadBaseCallbacks::pad_type_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterStringPadBase::PAD_TYPE_PROPERTY_KEY);
        }
        if converter.base.set_text_value(text) {
            DataConverterStringPadBaseCallbacks::text_changed(&mut converter);
            crate::mechanical_port::source::core::CoreObject::core_mut(&mut converter)
                .notify_property_changed(DataConverterStringPadBase::TEXT_PROPERTY_KEY);
        }
        converter
    }
    pub fn output_type(&self) -> DataType {
        DataType::String
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let mut input_value = input
            .as_any()
            .downcast_ref::<DataValueString>()
            .map_or_else(Vec::new, |value| value.value().as_bytes().to_vec());
        if input.as_any().is::<DataValueString>() {
            let mut input_length = input_value.len();
            let length = self.base.length() as usize;
            if input_length < length && !self.base.text().is_empty() {
                let pad_pattern = self.base.text().as_bytes();
                let pad_length = pad_pattern.len();
                input_value.reserve(length);
                let mut pad_text = Vec::new();
                let pad_text_size = length - input_length;
                pad_text.reserve(pad_text_size);
                while input_length < length {
                    let max_length = if pad_text_size > pad_length {
                        pad_length
                    } else {
                        pad_text_size
                    };
                    pad_text.extend_from_slice(&pad_pattern[..max_length]);
                    input_length += max_length;
                }
                if self.base.pad_type() == 1 {
                    input_value.extend_from_slice(&pad_text[..pad_text_size]);
                } else {
                    input_value.splice(0..0, pad_text[..pad_text_size].iter().copied());
                }
            }
        }
        // Upstream std::string padding counts bytes and may cut a UTF-8 sequence.
        // Preserve its byte operations, then use the runtime's text-boundary
        // replacement policy once on the final output. Valid Unicode is unchanged.
        self.output
            .set_value(String::from_utf8_lossy(&input_value).into_owned());
        &self.output
    }
    pub fn length_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
    pub fn pad_type_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
    pub fn text_changed(&mut self) {
        self.base.base.mark_converter_dirty()
    }
}

impl DataConverterStringPadBaseCallbacks for DataConverterStringPad {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn length_changed(&mut self) {
        Self::length_changed(self);
    }

    fn pad_type_changed(&mut self) {
        Self::pad_type_changed(self);
    }

    fn text_changed(&mut self) {
        Self::text_changed(self);
    }
}

crate::impl_data_converter_capability_forward!(DataConverterStringPad, base.base);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn padding_preserves_valid_unicode_and_replaces_only_incomplete_output() {
        for (input, length, pattern, prefix, suffix) in [
            ("a", 2, "é", "�a", "a�"),
            ("a", 3, "é", "éa", "aé"),
            ("a", 4, "é", "é�a", "aé�"),
            ("a", 5, "é", "ééa", "aéé"),
            ("a", 3, "🙂", "�a", "a�"),
            ("a", 5, "🙂", "🙂a", "a🙂"),
            ("é", 4, "0", "00é", "é00"),
            ("é", 1, "0", "é", "é"),
            ("a", 5, "", "a", "a"),
            ("a", 4, "xy", "xyxa", "axyx"),
        ] {
            for (side, expected) in [(0, prefix), (1, suffix)] {
                let mut converter = DataConverterStringPad::new(length, side, pattern.into());
                let input = DataValueString::new(input.into());
                for _ in 0..2 {
                    let output = converter.convert(&input);
                    assert_eq!(
                        output
                            .as_any()
                            .downcast_ref::<DataValueString>()
                            .unwrap()
                            .value(),
                        expected
                    );
                }
            }
        }
    }
}
