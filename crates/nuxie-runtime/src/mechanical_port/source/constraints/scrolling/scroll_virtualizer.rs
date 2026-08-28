use std::collections::HashSet;

use crate::mechanical_port::source::{
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::CoreHandle,
    generated::core_registry::CoreCapabilities,
    layout::layout_node_provider::LayoutNodeProvider,
    math::vec2d::Vec2D,
    virtualizing_component::{VirtualizedDirection, VirtualizingComponent},
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

    fn with_provider<R>(
        child: &CoreHandle,
        use_provider: impl FnOnce(&dyn LayoutNodeProvider) -> R,
    ) -> R {
        child
            .with(|child| child.as_layout_node_provider().map(use_provider))
            .flatten()
            .expect("ScrollVirtualizer children remain LayoutNodeProviders")
    }

    fn with_provider_mut<R>(
        child: &CoreHandle,
        use_provider: impl FnOnce(&mut dyn LayoutNodeProvider) -> R,
    ) -> R {
        child
            .with_mut(|child| child.as_layout_node_provider_mut().map(use_provider))
            .flatten()
            .expect("ScrollVirtualizer children remain LayoutNodeProviders")
    }

    pub fn constrain(
        &mut self,
        scroll: &mut ScrollConstraint,
        children: &[CoreHandle],
        offset: f32,
        direction: VirtualizedDirection,
    ) -> bool {
        let horizontal = direction == VirtualizedDirection::Horizontal;
        let content_size = if horizontal {
            scroll.content_width() as f64
        } else {
            scroll.content_height() as f64
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
                    let multiplier = (f64::from(offset) / content_size).floor() as i32 + 1;
                    self.offset =
                        (-1.0 * (f64::from(offset) - f64::from(multiplier) * content_size)) as f32;
                } else {
                    self.offset = -offset;
                }
            } else {
                let multiplier = (f64::from(normalized_offset) / content_size).floor() as i32;
                self.offset = if multiplier > 0 {
                    (f64::from(normalized_offset) % (f64::from(multiplier) * content_size)) as f32
                } else {
                    normalized_offset
                };
            }
            self.virtualize(scroll, children);
        }
        true
    }

    pub fn virtualize(&mut self, scroll: &mut ScrollConstraint, children: &[CoreHandle]) {
        let total_item_count: i32 = children
            .iter()
            .map(|child| Self::with_provider(child, |child| child.num_layout_nodes() as i32))
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
        let mut changed_components = Vec::<CoreHandle>::new();

        for child in children {
            Self::with_provider_mut(child, |child| {
                if let Some(component) = child.transform_component_mut() {
                    if let Some(virtualizer) = VirtualizingComponent::from(component) {
                        virtualizer.set_visible_indices(-1, -1);
                    }
                }
            });
        }

        'find_start: for child in children {
            let count = Self::with_provider(child, LayoutNodeProvider::num_layout_nodes);
            for item_index in 0..count {
                let size = self.get_item_size(child, item_index, horizontal);
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
            let child = &children[child_index];
            let count = Self::with_provider(child, LayoutNodeProvider::num_layout_nodes);
            for local in current_child_index..count {
                let size = self.get_item_size(child, local, horizontal);
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
            'providers: for (provider_index, child) in children.iter().enumerate() {
                let count = Self::with_provider(child, LayoutNodeProvider::num_layout_nodes) as i32;
                let start = running_total;
                let end = start + count;
                if start < end && actual_index < end && actual_index >= start {
                    let local = (actual_index - start) as usize;
                    if visible_indices[provider_index].x == -1.0 {
                        visible_indices[provider_index].x = local as f32;
                    }
                    visible_indices[provider_index].y = local as f32;
                    let size = self.get_item_size(child, local, horizontal);
                    let parent_world_invertible = Self::with_provider(child, |child| {
                        child.transform_component().is_some_and(|component| {
                            component.world_transform().inverted().is_some()
                        })
                    });
                    let (is_virtualizing, changed, invertible) =
                        Self::with_provider_mut(child, |child| {
                            let Some(component) = child.transform_component_mut() else {
                                return (false, false, true);
                            };
                            let Some(virtualizer) = VirtualizingComponent::from(component) else {
                                return (false, false, true);
                            };
                            let local = local as i32;
                            let changed = virtualizer.item(local).is_none();
                            if changed {
                                virtualizer.add_virtualizable(local);
                            }
                            if let Some(virtualizable) = virtualizer.item(local) {
                                if !parent_world_invertible {
                                    return (true, changed, false);
                                }
                                let location = if horizontal {
                                    Vec2D::new(running_offset, virtualizable.layout_y())
                                } else {
                                    Vec2D::new(virtualizable.layout_x(), running_offset)
                                };
                                virtualizer.set_virtualizable_position(local, location);
                            }
                            (true, changed, true)
                        });
                    if !is_virtualizing {
                        running_total = end;
                        continue;
                    }
                    if changed && !changed_components.contains(child) {
                        changed_components.push(child.clone());
                    }
                    if !invertible {
                        continue 'providers;
                    }
                    running_offset += size + gap;
                    break;
                }
                running_total = end;
            }
        }

        for (index, child) in children.iter().enumerate() {
            let visible = visible_indices[index];
            Self::with_provider_mut(child, |child| {
                if let Some(component) = child.transform_component_mut() {
                    if let Some(virtualizer) = VirtualizingComponent::from(component) {
                        virtualizer.set_visible_indices(visible.x as i32, visible.y as i32);
                    }
                }
            });
        }
        for child in changed_components {
            Self::with_provider_mut(&child, |child| {
                if let Some(component) = child.transform_component_mut() {
                    if let Some(virtualizer) = VirtualizingComponent::from(component) {
                        virtualizer.virtualizable_changed();
                    }
                }
            });
        }
    }

    fn recycle_items(
        &mut self,
        mut indices: Vec<i32>,
        children: &[CoreHandle],
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
            for child in children {
                let start = running_total;
                let end =
                    start + Self::with_provider(child, LayoutNodeProvider::num_layout_nodes) as i32;
                if start < end && actual_index < end && actual_index >= start {
                    Self::with_provider_mut(child, |child| {
                        if let Some(component) = child.transform_component_mut() {
                            if let Some(virtualizer) = VirtualizingComponent::from(component) {
                                virtualizer.remove_virtualizable(actual_index - start);
                            }
                        }
                    });
                    break;
                }
                running_total = end;
            }
        }
    }

    fn get_item_size(&self, child: &CoreHandle, index: usize, horizontal: bool) -> f32 {
        Self::with_provider_mut(child, |child| {
            if let Some(component) = child.transform_component_mut() {
                if let Some(virtualizer) = VirtualizingComponent::from(component) {
                    let size = virtualizer.item_size(index as i32);
                    return if horizontal { size.x } else { size.y };
                }
            }
            let bounds = child.layout_bounds();
            if horizontal {
                bounds.width()
            } else {
                bounds.height()
            }
        })
    }
}
