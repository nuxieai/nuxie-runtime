use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_string::DataValueString,
};
pub struct DataConverterStringPad {
    length: usize,
    pad_type: u32,
    text: String,
    output: DataValueString,
    dirty: bool,
}
impl DataConverterStringPad {
    pub fn new(length: usize, pad_type: u32, text: String) -> Self {
        Self {
            length,
            pad_type,
            text,
            output: DataValueString::default(),
            dirty: false,
        }
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
            if input_length < self.length && !self.text.is_empty() {
                let pad_pattern = &self.text;
                let pad_length = pad_pattern.len();
                input_value.reserve(self.length);
                let mut pad_text = String::new();
                let pad_text_size = self.length - input_length;
                pad_text.reserve(pad_text_size);
                while input_length < self.length {
                    let max_length = if pad_text_size > pad_length {
                        pad_length
                    } else {
                        pad_text_size
                    };
                    pad_text.push_str(&pad_pattern[..max_length]);
                    input_length += max_length;
                }
                if self.pad_type == 1 {
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
        self.dirty = true
    }
    pub fn pad_type_changed(&mut self) {
        self.dirty = true
    }
    pub fn text_changed(&mut self) {
        self.dirty = true
    }
}
