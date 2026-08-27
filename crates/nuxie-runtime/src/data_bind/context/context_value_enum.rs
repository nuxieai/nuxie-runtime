//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_enum.cpp`.
//!
//! The shared context owner synchronizes the source and runs the converter;
//! this module owns the concrete Enum result, its raw CoreUint projection,
//! and the source-enum metadata branch used only by `Solo`.

use nuxie_binary::RuntimeDataType;

use crate::data_bind_graph::{RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue};

pub(crate) fn integer_payload(value: &RuntimeDataBindGraphValue) -> Option<u64> {
    match value {
        RuntimeDataBindGraphValue::Integer(value)
        | RuntimeDataBindGraphValue::Enum(value)
        | RuntimeDataBindGraphValue::SymbolListIndex(value)
        | RuntimeDataBindGraphValue::Asset(value)
        | RuntimeDataBindGraphValue::Artboard(value)
        | RuntimeDataBindGraphValue::Trigger(value) => Some(*value),
        _ => None,
    }
}

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    integer_payload(next).map(RuntimeDataBindGraphValue::Enum)
}

/// Whether pinned `DataBind::bind` selects `DataBindContextValueEnum` for
/// this occurrence. A concrete converter output wins; `none` and `input`
/// fall back to the live source type, while `any` stays dynamically typed.
pub(crate) fn owns_output(
    source: &RuntimeDataBindGraphValue,
    converter: Option<&RuntimeDataBindGraphConverter>,
) -> bool {
    match converter.map(RuntimeDataBindGraphConverter::cpp_output_data_type) {
        Some(RuntimeDataType::EnumType) => true,
        None | Some(RuntimeDataType::None | RuntimeDataType::Input) => {
            matches!(source, RuntimeDataBindGraphValue::Enum(_))
        }
        _ => false,
    }
}

/// `calculateValue<DataValueEnum, uint32_t>` returns the Enum payload, or
/// inherited `DataValueInteger::defaultValue` (zero) for a wrong concrete
/// DataValue.
pub(crate) fn calculate_value(value: &RuntimeDataBindGraphValue) -> u64 {
    match value {
        RuntimeDataBindGraphValue::Enum(value) => *value,
        _ => 0,
    }
}

/// Pinned Enum's CoreUint branch writes the calculated payload unchanged.
/// This is deliberately distinct from Number's clamp-and-round projection.
pub(crate) fn core_uint_value(value: &RuntimeDataBindGraphValue) -> u64 {
    calculate_value(value)
}

/// `Solo` is the only target that does not receive the raw CoreUint value.
/// C++ checks the unconverted `m_dataValue` for `DataValueEnum`, uses its
/// retained `DataEnum`, and indexes that metadata with the converted value.
pub(crate) fn solo_value_name<'a>(
    source: &RuntimeDataBindGraphValue,
    converted_value: u64,
    value_names: &'a [Vec<u8>],
) -> Option<&'a [u8]> {
    matches!(source, RuntimeDataBindGraphValue::Enum(_))
        .then_some(())
        .and_then(|()| usize::try_from(converted_value).ok())
        .and_then(|index| value_names.get(index))
        .map(Vec::as_slice)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calculate_value_uses_cpp_enum_default_for_wrong_data_value_type() {
        assert_eq!(calculate_value(&RuntimeDataBindGraphValue::Enum(7)), 7);
        assert_eq!(calculate_value(&RuntimeDataBindGraphValue::Integer(7)), 0);
    }

    #[test]
    fn output_owner_is_selected_before_target_dispatch() {
        assert!(owns_output(&RuntimeDataBindGraphValue::Enum(0), None));
        assert!(!owns_output(
            &RuntimeDataBindGraphValue::Enum(0),
            Some(&RuntimeDataBindGraphConverter::Scripted {
                global_id: 1,
                serialized_implemented_methods: 0,
                definition: Default::default(),
                instance: None,
            }),
        ));
    }

    #[test]
    fn solo_name_requires_the_raw_enum_source() {
        let names = vec![b"first".to_vec(), b"second".to_vec()];
        assert_eq!(
            solo_value_name(&RuntimeDataBindGraphValue::Enum(0), 1, &names),
            Some(b"second".as_slice()),
        );
        assert_eq!(
            solo_value_name(&RuntimeDataBindGraphValue::Integer(0), 1, &names),
            None,
        );
    }
}
