//! Pinned C++ `DataConverterTrigger` conversion semantics.
//!
//! The forward conversion accepts only `DataValueInteger` and increments its
//! `uint32_t` payload with the C++ unsigned wrap. The class does not override
//! `DataConverter::reverseConvert`, so reverse conversion is the base-class
//! identity operation for every concrete `DataValue`
//! (`data_converter_trigger.cpp:8-19`; `data_converter.hpp:15-22`).

use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn convert(value: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    let value = match value {
        // C++ `DataValueTrigger` derives from `DataValueInteger`, so both
        // concrete values satisfy `input->is<DataValueInteger>()`
        // (`data_value_trigger.hpp:8-18`).
        RuntimeDataBindGraphValue::Integer(value) | RuntimeDataBindGraphValue::Trigger(value) => {
            *value
        }
        _ => return Some(RuntimeDataBindGraphValue::Trigger(0)),
    };
    Some(RuntimeDataBindGraphValue::Trigger(u64::from(
        (value as u32).wrapping_add(1),
    )))
}

pub(crate) fn reverse_convert(
    value: &RuntimeDataBindGraphValue,
) -> Option<RuntimeDataBindGraphValue> {
    Some(value.clone())
}

#[cfg(test)]
mod tests {
    use super::{convert, reverse_convert};
    use crate::data_bind_graph::RuntimeDataBindGraphValue;

    #[test]
    fn forward_uses_integer_input_and_reverse_is_untyped_identity() {
        assert_eq!(
            convert(&RuntimeDataBindGraphValue::Integer(u64::from(u32::MAX))),
            Some(RuntimeDataBindGraphValue::Trigger(0))
        );
        assert_eq!(
            convert(&RuntimeDataBindGraphValue::Trigger(0)),
            Some(RuntimeDataBindGraphValue::Trigger(1))
        );
        assert_eq!(
            convert(&RuntimeDataBindGraphValue::Number(7.0)),
            Some(RuntimeDataBindGraphValue::Trigger(0))
        );
        assert_eq!(
            reverse_convert(&RuntimeDataBindGraphValue::Integer(9)),
            Some(RuntimeDataBindGraphValue::Integer(9))
        );
    }
}
