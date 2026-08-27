//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_list.cpp`.
//!
//! The shared context owner synchronizes the retained list source and runs
//! the converter. This module owns selection of the concrete List context
//! value and its `DataBindListItemConsumer::updateList` delivery boundary.

use nuxie_binary::RuntimeDataType;

use crate::data_bind_graph::{RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue};

/// Whether pinned `DataBind::bind` selects `DataBindContextValueList` for
/// this occurrence. The choice is made from `DataBind::outputType()` before
/// `apply` inspects or dispatches the target.
pub(crate) fn owns_output(
    source: &RuntimeDataBindGraphValue,
    converter: Option<&RuntimeDataBindGraphConverter>,
) -> bool {
    match converter.map(RuntimeDataBindGraphConverter::cpp_output_data_type) {
        Some(RuntimeDataType::List) => true,
        None | Some(RuntimeDataType::None | RuntimeDataType::Input) => {
            matches!(source, RuntimeDataBindGraphValue::List { .. })
        }
        _ => false,
    }
}

pub(crate) fn item_count_changed(previous: usize, next: usize) -> bool {
    previous != next
}

/// C++ `DataBindContextValueList::apply` ignores the target property key once
/// the target has crossed the typed list-consumer boundary. Rust validates the
/// consumer before this point and likewise delivers only the ordered rows.
pub(crate) fn apply_to_consumer<T>(rows: Option<Vec<T>>) -> Result<Vec<T>, ()> {
    rows.ok_or(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_owner_is_selected_before_target_dispatch() {
        assert!(owns_output(
            &RuntimeDataBindGraphValue::Number(0.0),
            Some(&RuntimeDataBindGraphConverter::NumberToList {
                global_id: 1,
                view_model_id: 0,
                view_model_count: 1,
            }),
        ));
        assert!(!owns_output(
            &RuntimeDataBindGraphValue::List { item_count: 0 },
            Some(&RuntimeDataBindGraphConverter::ListToLength),
        ));
        assert!(owns_output(
            &RuntimeDataBindGraphValue::List { item_count: 0 },
            None,
        ));
    }

    #[test]
    fn invalid_converter_output_is_not_a_stale_list_success() {
        assert_eq!(apply_to_consumer::<u8>(None), Err(()));
        assert_eq!(apply_to_consumer(Some(vec![1, 2])), Ok(vec![1, 2]));
    }
}
