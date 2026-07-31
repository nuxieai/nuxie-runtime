//! Structural list dependency owner matching C++ `DataBindListItemConsumer`.

pub(crate) fn changed(previous_item_count: usize, next_item_count: usize) -> bool {
    crate::context_value_list::item_count_changed(previous_item_count, next_item_count)
}
