//! Direct Rust owner for the virtualized-list methods in pinned C++
//! `include/rive/artboard_component_list.hpp` and
//! `src/artboard_component_list.cpp`.
//!
//! The complete ArtboardComponentList owner is extracted incrementally. This
//! file holds its virtualized mounted-item operations; the authored-order
//! construction and focus lifecycle still remain in `artboard.rs` until their
//! active frame-loop family closes.

use crate::{ArtboardInstance, Mat2D};
use nuxie_binary::RuntimeFile;

impl ArtboardInstance {
    pub(crate) fn virtualizing_component_has_item(
        &self,
        list_local_id: usize,
        logical_index: usize,
    ) -> bool {
        let Some(list_local_id) = crate::virtualizing_component::from(self, list_local_id) else {
            return false;
        };
        self.component_list_items(list_local_id)
            .is_some_and(|items| items.iter().any(|item| item.logical_index == logical_index))
    }

    pub(crate) fn add_component_list_virtualizable(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        logical_index: usize,
    ) -> bool {
        if self.virtualizing_component_has_item(list_local_id, logical_index) {
            return false;
        }
        let Some(logical) = self
            .component_list_state(list_local_id)
            .and_then(|list| list.logical_items.get(logical_index))
            .cloned()
        else {
            return false;
        };
        let Some(item) =
            self.create_component_list_item_instance(file, list_local_id, logical_index, logical)
        else {
            return false;
        };
        let transform = item.transform;
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return false;
        };
        list.items.push(item);
        list.item_transforms.push(transform);
        *list.order_cache.borrow_mut() = Default::default();
        self.mark_nested_structure_changed();
        self.mark_layout_changed();
        self.mark_prepared_changed();
        true
    }

    pub(crate) fn remove_component_list_virtualizable(
        &mut self,
        list_local_id: usize,
        logical_index: usize,
    ) -> bool {
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return false;
        };
        let Some(index) = list
            .items
            .iter()
            .position(|item| item.logical_index == logical_index)
        else {
            return false;
        };
        // State machines precede the child Artboard in the item struct, so
        // dropping the mounted item preserves the C++ teardown order. FL-D
        // replaces this drop with the C++ resource pools.
        list.items.remove(index);
        if index < list.item_transforms.len() {
            list.item_transforms.remove(index);
        }
        *list.order_cache.borrow_mut() = Default::default();
        self.mark_nested_structure_changed();
        self.mark_layout_changed();
        self.mark_prepared_changed();
        true
    }

    pub(crate) fn set_component_list_visible_indices(
        &mut self,
        list_local_id: usize,
        start: i32,
        end: i32,
    ) {
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return;
        };
        list.visible_start = start;
        list.visible_end = end;
        *list.order_cache.borrow_mut() = Default::default();
    }

    pub(crate) fn set_component_list_virtualizable_position(
        &mut self,
        list_local_id: usize,
        logical_index: usize,
        position: (f32, f32),
    ) {
        let uses_layout = self
            .component(list_local_id)
            .and_then(|component| component.parent)
            .and_then(|parent| self.objects.component(parent))
            .is_some_and(|parent| parent.concrete.layout.is_some());
        let origin = self
            .component_list_state(list_local_id)
            .and_then(|list| {
                list.items
                    .iter()
                    .find(|item| item.logical_index == logical_index)
            })
            .map(|item| {
                let size = crate::draw::runtime_component_list_item_layout_size(item);
                let origin = if uses_layout {
                    (size.0 * item.child.origin_x, size.1 * item.child.origin_y)
                } else {
                    (0.0, 0.0)
                };
                origin
            })
            .unwrap_or((0.0, 0.0));
        let Some(list) = self.component_list_state_mut(list_local_id) else {
            return;
        };
        let Some(item_index) = list
            .items
            .iter()
            .position(|item| item.logical_index == logical_index)
        else {
            return;
        };
        // `Artboard::origin()` is negative when mounted with FrameOrigin off,
        // so C++ `position - origin` adds the mounted layout origin here.
        // Retain the final owner transform at the same mutation site
        // (`artboard_component_list.cpp:1728-1740`;
        // `artboard.cpp:1729-1734`).
        let transform = Mat2D([
            1.0,
            0.0,
            0.0,
            1.0,
            position.0 + origin.0,
            position.1 + origin.1,
        ]);
        list.items[item_index].transform = transform;
        if let Some(retained) = list.item_transforms.get_mut(item_index) {
            *retained = transform;
        }
    }

    pub(crate) fn component_list_virtualizable_layout_position(
        &self,
        list_local_id: usize,
        logical_index: usize,
    ) -> (f32, f32) {
        self.component_list_state(list_local_id)
            .and_then(|list| {
                list.items
                    .iter()
                    .find(|item| item.logical_index == logical_index)
            })
            .and_then(|item| {
                item.child
                    .component(0)
                    .and_then(|component| component.concrete.layout.as_ref())
                    .map(|layout| layout.position())
            })
            .unwrap_or((0.0, 0.0))
    }

    pub(crate) fn component_list_virtualizable_changed(&mut self, list_local_id: usize) {
        if let Some(list) = self.component_list_state(list_local_id) {
            *list.order_cache.borrow_mut() = Default::default();
        }
    }
}
