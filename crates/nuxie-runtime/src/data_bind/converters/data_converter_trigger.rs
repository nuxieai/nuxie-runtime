//! Direct owner for C++ `DataConverterTrigger`.

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(value: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::data_converter_trigger::convert(value)
}

pub(crate) fn reverse(value: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::data_converter_trigger::reverse_convert(value)
}
