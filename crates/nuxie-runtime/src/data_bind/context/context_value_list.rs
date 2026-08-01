//! Structural list cache owned by C++ `ContextValueList`.

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
    fn invalid_converter_output_is_not_a_stale_list_success() {
        assert_eq!(apply_to_consumer::<u8>(None), Err(()));
        assert_eq!(apply_to_consumer(Some(vec![1, 2])), Ok(vec![1, 2]));
    }
}
