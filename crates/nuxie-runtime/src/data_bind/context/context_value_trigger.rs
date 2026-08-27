//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_trigger.cpp`.
//!
//! The shared context owner synchronizes the source and runs the converter;
//! this module owns the concrete Trigger result and its CoreUint projection.

use nuxie_binary::RuntimeDataType;

use crate::data_bind_graph::RuntimeDataBindGraphConverter;
use crate::data_bind_graph::RuntimeDataBindGraphValue;

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::context_value_enum::integer_payload(next).map(RuntimeDataBindGraphValue::Trigger)
}

/// Whether pinned `DataBind::bind` selects `DataBindContextValueTrigger` for
/// this occurrence. A concrete converter output wins; `none` and `input`
/// fall back to the live source type, while `any` stays dynamically typed.
pub(crate) fn owns_output(
    source: &RuntimeDataBindGraphValue,
    converter: Option<&RuntimeDataBindGraphConverter>,
) -> bool {
    match converter.map(RuntimeDataBindGraphConverter::cpp_output_data_type) {
        Some(RuntimeDataType::Trigger) => true,
        None | Some(RuntimeDataType::None | RuntimeDataType::Input) => {
            matches!(source, RuntimeDataBindGraphValue::Trigger(_))
        }
        _ => false,
    }
}

/// `calculateValue<DataValueTrigger, uint32_t>` returns the Trigger payload,
/// or `DataValueTrigger::defaultValue` (zero) for a wrong concrete DataValue.
pub(crate) fn calculate_value(value: &RuntimeDataBindGraphValue) -> u64 {
    match value {
        RuntimeDataBindGraphValue::Trigger(value) => *value,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_value_uses_cpp_trigger_default_for_wrong_data_value_type() {
        assert_eq!(calculate_value(&RuntimeDataBindGraphValue::Trigger(7)), 7);
        assert_eq!(calculate_value(&RuntimeDataBindGraphValue::Integer(7)), 0);
    }

    #[test]
    fn output_owner_is_selected_before_target_dispatch() {
        assert!(owns_output(
            &RuntimeDataBindGraphValue::String(Vec::new()),
            Some(&RuntimeDataBindGraphConverter::TriggerIncrement),
        ));
        assert!(!owns_output(
            &RuntimeDataBindGraphValue::Trigger(0),
            Some(&RuntimeDataBindGraphConverter::Scripted {
                global_id: 1,
                serialized_implemented_methods: 0,
                definition: Default::default(),
                instance: None,
            }),
        ));
    }
}
