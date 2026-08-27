//! Direct owner for pinned C++ `DataConverterToNumber`.
//!
//! C++ retains one `DataValueNumber m_output`. Rust carries that number in the
//! occurrence-local converter state because graph values themselves are owned
//! by value.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(input: &RuntimeDataBindGraphValue, output: &mut f32) {
    match input {
        RuntimeDataBindGraphValue::String(value) => convert_string(value, output),
        RuntimeDataBindGraphValue::Enum(value) => convert_enum(*value, output),
        RuntimeDataBindGraphValue::Number(value) => *output = *value,
        RuntimeDataBindGraphValue::Color(value) => convert_color(*value, output),
        RuntimeDataBindGraphValue::Boolean(value) => *output = if *value { 1.0 } else { 0.0 },
        RuntimeDataBindGraphValue::SymbolListIndex(value) => *output = (*value as u32) as f32,
        _ => *output = 0.0,
    }
}

fn convert_string(value: &[u8], output: &mut f32) {
    *output = nuxie_binary::data_converter_to_number_string_value(value, *output);
}

fn convert_color(value: u32, output: &mut f32) {
    *output = value as f32;
}

fn convert_enum(value: u64, output: &mut f32) {
    *output = (value as u32) as f32;
}

pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::data_converter_to_number_output_type()
}
