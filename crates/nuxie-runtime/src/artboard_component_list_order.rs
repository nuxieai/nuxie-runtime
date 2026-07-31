use crate::components::{RuntimeComponentListOrderCache, RuntimeConstrainableListState};
use nuxie_binary::RuntimeFile;
use std::cell::Ref;

/// Retain C++ `ArtboardComponentList::orderedListIndices()` on the concrete
/// list occurrence. Drawing and gamepad propagation must consume the same
/// cached order; neither may reconstruct a private approximation.
pub(crate) fn runtime_component_list_order<'a>(
    runtime: &RuntimeFile,
    list: &'a RuntimeConstrainableListState,
) -> Ref<'a, RuntimeComponentListOrderCache> {
    let uses_draw_index_sort = list.logical_items.iter().any(|item| {
        runtime
            .view_model_property_for_symbol(item.context.borrow().view_model_index(), 16)
            .is_some()
    });
    let order_dirty = list.items.iter().any(|item| {
        item.draw_index_sink
            .as_ref()
            .is_some_and(|sink| !sink.peek_dirt().is_empty())
    });
    {
        let mut order = list.order_cache.borrow_mut();
        if !order.valid || order.indices.len() != list.items.len() || order_dirty {
            order.indices.clear();
            order.indices.extend(0..list.items.len());
            if uses_draw_index_sort {
                let draw_index = |item_index: usize| {
                    let item = &list.items[item_index];
                    runtime
                        .view_model_property_for_symbol(
                            item.context.borrow().view_model_index(),
                            16,
                        )
                        .and_then(|property| property.string_property("name"))
                        .and_then(|name| item.context.borrow().number_value_by_property_name(name))
                        .filter(|value| value.is_finite())
                        .unwrap_or(0.0)
                };
                order.indices.sort_by(|&left, &right| {
                    draw_index(left)
                        .partial_cmp(&draw_index(right))
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| {
                            list.items[left]
                                .logical_index
                                .cmp(&list.items[right].logical_index)
                        })
                });
            }
            for item in &list.items {
                if let Some(sink) = item.draw_index_sink.as_ref() {
                    sink.take_dirt();
                }
            }
            order.valid = true;
        }
    }
    list.order_cache.borrow()
}
