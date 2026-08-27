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
        let mut callbacks = DataConverterStringPadInitializationCallbacks;
        converter.base.set_length(length as u32, &mut callbacks);
        converter.base.set_pad_type(pad_type, &mut callbacks);
        converter.base.set_text(text, &mut callbacks);
        converter
    }
    pub fn output_type(&self) -> DataType {
        DataType::String
    }
    pub fn convert<'a>(&'a mut self, input: &dyn DataValue) -> &'a dyn DataValue {
        let mut input_value = input
            .as_any()
            .downcast_ref::<DataValueString>()
            .map_or_else(String::new, |value| value.value().to_owned());
        if input.as_any().is::<DataValueString>() {
            let mut input_length = input_value.len();
            let length = self.base.length() as usize;
            if input_length < length && !self.base.text().is_empty() {
                let pad_pattern = self.base.text();
                let pad_length = pad_pattern.len();
                input_value.reserve(length);
                let mut pad_text = String::new();
                let pad_text_size = length - input_length;
                pad_text.reserve(pad_text_size);
                while input_length < length {
                    let max_length = if pad_text_size > pad_length {
                        pad_length
                    } else {
                        pad_text_size
                    };
                    pad_text.push_str(&pad_pattern[..max_length]);
                    input_length += max_length;
                }
                if self.base.pad_type() == 1 {
                    input_value.push_str(&pad_text[..pad_text_size]);
                } else {
                    input_value.insert_str(0, &pad_text[..pad_text_size]);
                }
            }
        }
        self.output.set_value(input_value);
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

struct DataConverterStringPadInitializationCallbacks;

impl DataConverterStringPadBaseCallbacks for DataConverterStringPadInitializationCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
