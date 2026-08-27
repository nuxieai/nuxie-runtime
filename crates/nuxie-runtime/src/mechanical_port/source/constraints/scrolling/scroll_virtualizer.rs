use std::collections::{BTreeSet, HashSet};

use crate::mechanical_port::source::{
    artboard::ArtboardInstance,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    layout::layout_node_provider::LayoutNodeProvider,
    math::vec2d::Vec2D,
    virtualizing_component::{Virtualizable, VirtualizedDirection, VirtualizingComponent},
};

pub struct ScrollVirtualizer {
    visible_index_start: i32,
    visible_index_end: i32,
    offset: f32,
    infinite: bool,
    viewport_size: f32,
    direction: VirtualizedDirection,
}

impl Default for ScrollVirtualizer {
    fn default() -> Self {
        Self {
            visible_index_start: 0,
            visible_index_end: 0,
            offset: 0.0,
            infinite: false,
            viewport_size: 0.0,
            direction: VirtualizedDirection::Horizontal,
        }
    }
}

impl Drop for ScrollVirtualizer {
    fn drop(&mut self) {
        self.reset();
    }
}

impl ScrollVirtualizer {
    pub fn reset(&mut self) {
        self.visible_index_end = 0;
        self.visible_index_start = self.visible_index_end;
    }

    pub fn constrain(
        &mut self,
        scroll: &mut ScrollConstraint,
        children: &mut [*mut dyn LayoutNodeProvider],
        offset: f32,
        direction: VirtualizedDirection,
    ) -> bool {
        let horizontal = direction == VirtualizedDirection::Horizontal;
        let content_size = if horizontal {
            scroll.content_width()
        } else {
            scroll.content_height()
        };
        if content_size > 0.0 {
            let normalized_offset = -offset;
            self.direction = direction;
            self.viewport_size = if horizontal {
                scroll.viewport_width()
            } else {
                scroll.viewport_height()
            };
            self.infinite = scroll.infinite();
            if offset > 0.0 {
                if self.infinite {
                    let multiplier = (offset / content_size).floor() as i32 + 1;
                    self.offset = -1.0 * (offset - multiplier as f32 * content_size);
                } else {
                    self.offset = -offset;
                }
            } else {
                let multiplier = (normalized_offset / content_size).floor() as i32;
                self.offset = if multiplier > 0 {
                    normalized_offset % (multiplier as f32 * content_size)
                } else {
                    normalized_offset
                };
            }
            self.virtualize(scroll, children);
        }
        true
    }

    pub fn virtualize(
        &mut self,
        scroll: &mut ScrollConstraint,
        children: &mut [*mut dyn LayoutNodeProvider],
    ) {
        let total_item_count: i32 = children
            .iter()
            .map(|child| unsafe { (**child).num_layout_nodes() as i32 })
            .sum();
        let last_start = if self.infinite && total_item_count > 0 {
            self.visible_index_start % total_item_count
        } else {
            self.visible_index_start
        };
        let last_end = if self.infinite && total_item_count > 0 {
            self.visible_index_end % total_item_count
        } else {
            self.visible_index_end
        };
        self.visible_index_start = 0;
        self.visible_index_end = total_item_count - 1;
        let mut running_size = 0.0;
        let mut running_offset = 0.0;
        let mut running_index = 0;
        let mut child_index = 0usize;
        let mut current_child_index = 0usize;
        let horizontal = self.direction == VirtualizedDirection::Horizontal;
        let gap = if horizontal {
            scroll.gap().x
        } else {
            scroll.gap().y
        };
        let mut changed_components = BTreeSet::<usize>::new();

        for child in children.iter().copied() {
            unsafe {
                if let Some(component) = (*child).transform_component_mut() {
                    if let Some(virtualizer) = VirtualizingComponent::from(component) {
                        virtualizer.set_visible_indices(-1, -1);
                    }
                }
            }
        }

        'find_start: for child in children.iter().copied() {
            let count = unsafe { (*child).num_layout_nodes() };
            for item_index in 0..count {
                let size = self.get_item_size(unsafe { &mut *child }, item_index, horizontal);
                if running_size + size > self.offset {
                    running_offset = running_size - self.offset;
                    self.visible_index_start = running_index;
                    if current_child_index == children.len() - 1 {
                        child_index += 1;
                        current_child_index = 0;
                    } else {
                        current_child_index += 1;
                    }
                    break 'find_start;
                }
                running_size += size;
                current_child_index = item_index;
                running_index += 1;
                if running_size + gap > self.offset {
                    if running_index == total_item_count {
                        running_index = 0;
                    }
                    if current_child_index == children.len() - 1 {
                        child_index += 1;
                        current_child_index = 0;
                    } else {
                        current_child_index += 1;
                    }
                    running_size += gap;
                    running_offset = running_size - self.offset;
                    self.visible_index_start = running_index;
                    break 'find_start;
                }
                running_size += gap;
            }
            child_index += 1;
        }

        child_index %= children.len();
        let mut item = self.visible_index_start;
        let mut wrapped = false;
        let mut cycle_count = 0;
        'find_end: while item < total_item_count && cycle_count < 2 {
            let child = children[child_index];
            let count = unsafe { (*child).num_layout_nodes() };
            for local in current_child_index..count {
                let size = self.get_item_size(unsafe { &mut *child }, local, horizontal);
                if running_size + size + gap >= self.offset + self.viewport_size {
                    self.visible_index_end = if self.infinite && wrapped {
                        item + total_item_count
                    } else {
                        item
                    };
                    break 'find_end;
                }
                running_size += size + gap;
                running_index += 1;
                if self.infinite && item == total_item_count - 1 {
                    wrapped = true;
                    item = -1;
                    cycle_count += 1;
                }
                item += 1;
            }
            current_child_index = 0;
        }

        let actual_start = if self.infinite && total_item_count > 0 {
            self.visible_index_start % total_item_count
        } else {
            self.visible_index_start
        };
        let actual_end = if self.infinite && total_item_count > 0 {
            self.visible_index_end % total_item_count
        } else {
            self.visible_index_end
        };
        let mut used = HashSet::new();
        if actual_start <= actual_end {
            for index in actual_start..=actual_end {
                used.insert(index);
            }
        } else {
            for index in actual_start..total_item_count {
                used.insert(index);
            }
            for index in 0..=actual_end {
                used.insert(index);
            }
        }
        let mut recycle = Vec::new();
        if last_start <= last_end {
            for index in last_start..=last_end {
                if !used.contains(&index) {
                    recycle.push(index);
                }
            }
        } else {
            for index in last_start..total_item_count {
                if !used.contains(&index) {
                    recycle.push(index);
                }
            }
            for index in 0..=last_end {
                if !used.contains(&index) {
                    recycle.push(index);
                }
            }
        }
        self.recycle_items(recycle, children, total_item_count);

        let mut visible_indices = vec![Vec2D::new(-1.0, -1.0); children.len()];
        for global_index in self.visible_index_start..=self.visible_index_end {
            let actual_index = if self.infinite {
                global_index % total_item_count
            } else {
                global_index
            };
            let mut running_total = 0;
            for (provider_index, child) in children.iter().copied().enumerate() {
                let start = running_total;
                let end = start + unsafe { (*child).num_layout_nodes() as i32 };
                unsafe {
                    if let Some(component) = (*child).transform_component_mut() {
                        if let Some(virtualizer) = VirtualizingComponent::from(component) {
                            if start < end && actual_index < end && actual_index >= start {
                                let local = (actual_index - start) as usize;
                                if visible_indices[provider_index].x == -1.0 {
                                    visible_indices[provider_index].x = local as f32;
                                }
                                visible_indices[provider_index].y = local as f32;
                                if virtualizer.item(local).is_none() {
                                    virtualizer.add_virtualizable(local);
                                    changed_components
                                        .insert(virtualizer as *mut _ as *mut () as usize);
                                }
                                let size = self.get_item_size(&mut *child, local, horizontal);
                                if let Some(virtualizable) = virtualizer.item_mut(local) {
                                    if let Some(component) =
                                        virtualizable.virtualizable_component_mut()
                                    {
                                        if let Some(artboard) =
                                            component.as_mut::<ArtboardInstance>()
                                        {
                                            let parent_world = *(*child)
                                                .transform_component()
                                                .unwrap()
                                                .world_transform();
                                            let Some(_inverse) = parent_world.inverted() else {
                                                continue;
                                            };
                                            let location = if horizontal {
                                                Vec2D::new(running_offset, artboard.layout_y())
                                            } else {
                                                Vec2D::new(artboard.layout_x(), running_offset)
                                            };
                                            virtualizer.set_virtualizable_position(local, location);
                                        }
                                    }
                                }
                                running_offset += size + gap;
                                break;
                            }
                        }
                    }
                }
                running_total = end;
            }
        }

        for (index, child) in children.iter().copied().enumerate() {
            unsafe {
                if let Some(component) = (*child).transform_component_mut() {
                    if let Some(virtualizer) = VirtualizingComponent::from(component) {
                        let visible = visible_indices[index];
                        virtualizer.set_visible_indices(visible.x as i32, visible.y as i32);
                    }
                }
            }
        }
        for address in changed_components {
            unsafe {
                (&mut *(address as *mut VirtualizingComponent)).virtualizable_changed();
            }
        }
    }

    fn recycle_items(
        &mut self,
        mut indices: Vec<i32>,
        children: &mut [*mut dyn LayoutNodeProvider],
        total_item_count: i32,
    ) {
        if total_item_count == 0 {
            return;
        }
        indices.sort();
        for global_index in indices {
            let actual_index = if self.infinite {
                global_index % total_item_count
            } else {
                global_index
            };
            let mut running_total = 0;
            for child in children.iter().copied() {
                let start = running_total;
                let end = start + unsafe { (*child).num_layout_nodes() as i32 };
                unsafe {
                    if let Some(component) = (*child).transform_component_mut() {
                        if let Some(virtualizer) = VirtualizingComponent::from(component) {
                            if start < end && actual_index < end && actual_index >= start {
                                virtualizer.remove_virtualizable((actual_index - start) as usize);
                                break;
                            }
                        }
                    }
                }
                running_total = end;
            }
        }
    }

    fn get_item_size(
        &self,
        child: &mut dyn LayoutNodeProvider,
        index: usize,
        horizontal: bool,
    ) -> f32 {
        if let Some(component) = child.transform_component_mut() {
            if let Some(virtualizer) = VirtualizingComponent::from(component) {
                let size = virtualizer.item_size(index);
                return if horizontal { size.x } else { size.y };
            }
        }
        let bounds = child.layout_bounds();
        if horizontal {
            bounds.width()
        } else {
            bounds.height()
        }
    }
}
