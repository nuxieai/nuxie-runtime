use std::collections::HashSet;

use crate::mechanical_port::source::{
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::CoreHandle,
    generated::core_registry::CoreCapabilities,
    layout::layout_node_provider::LayoutNodeProvider,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    virtualizing_component::{self, VirtualizedDirection, VirtualizingComponent},
};

pub struct ScrollVirtualizer {
    realized_index_start: i32,
    realized_index_end: i32,
    offset: f32,
    infinite: bool,
    viewport_size: f32,
    direction: VirtualizedDirection,
}

impl Default for ScrollVirtualizer {
    fn default() -> Self {
        Self {
            realized_index_start: 0,
            realized_index_end: 0,
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
    fn with_scroll<R>(scroll: &CoreHandle, use_scroll: impl FnOnce(&ScrollConstraint) -> R) -> R {
        scroll
            .with_downcast::<ScrollConstraint, _>(use_scroll)
            .expect("live ScrollConstraint")
    }
    fn with_virtualizer_mut<R>(
        child: &CoreHandle,
        use_virtualizer: impl FnOnce(&mut dyn VirtualizingComponent) -> R,
    ) -> Option<R> {
        child
            .with_mut(|child| virtualizing_component::from(child).map(use_virtualizer))
            .flatten()
    }

    pub fn reset(&mut self) {
        self.realized_index_end = 0;
        self.realized_index_start = self.realized_index_end;
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
        scroll: &CoreHandle,
        children: &[CoreHandle],
        offset: f32,
        direction: VirtualizedDirection,
    ) -> bool {
        let horizontal = direction == VirtualizedDirection::Horizontal;
        let content_size = if horizontal {
            Self::with_scroll(scroll, ScrollConstraint::content_width) as f64
        } else {
            Self::with_scroll(scroll, ScrollConstraint::content_height) as f64
        };
        if content_size > 0.0 {
            let normalized_offset = -offset;
            self.direction = direction;
            self.viewport_size = if horizontal {
                Self::with_scroll(scroll, ScrollConstraint::viewport_width)
            } else {
                Self::with_scroll(scroll, ScrollConstraint::viewport_height)
            };
            self.infinite = Self::with_scroll(scroll, |scroll| scroll.infinite());
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

    pub fn virtualize(&mut self, scroll: &CoreHandle, children: &[CoreHandle]) {
        let total_item_count: i32 = children
            .iter()
            .map(|child| Self::with_provider(child, |child| child.num_layout_nodes() as i32))
            .sum();
        let last_realized_index_start = if self.infinite && total_item_count > 0 {
            self.realized_index_start % total_item_count
        } else {
            self.realized_index_start
        };
        let last_realized_index_end = if self.infinite && total_item_count > 0 {
            self.realized_index_end % total_item_count
        } else {
            self.realized_index_end
        };
        self.realized_index_start = 0;
        self.realized_index_end = total_item_count - 1;
        let mut running_size = 0.0;
        let mut running_offset = 0.0;
        let mut running_index = 0;
        let mut child_index = 0usize;
        let mut current_child_index = 0usize;
        let horizontal = self.direction == VirtualizedDirection::Horizontal;
        let gap = if horizontal {
            Self::with_scroll(scroll, |scroll| scroll.gap().x)
        } else {
            Self::with_scroll(scroll, |scroll| scroll.gap().y)
        };
        let mut changed_components = Vec::<CoreHandle>::new();

        for child in children {
            Self::with_virtualizer_mut(child, |virtualizer| {
                virtualizer.set_visible_indices(-1, -1);
                virtualizer.set_realized_indices(-1, -1);
            });
        }

        'find_start: for child in children {
            let count = Self::with_provider(child, |provider| provider.num_layout_nodes());
            for item_index in 0..count {
                let size = self.get_item_size(child, item_index, horizontal);
                if running_size + size > self.offset {
                    running_offset = running_size - self.offset;
                    self.realized_index_start = running_index;
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
                    self.realized_index_start = running_index;
                    break 'find_start;
                }
                running_size += gap;
            }
            child_index += 1;
        }

        child_index %= children.len();
        let mut item = self.realized_index_start;
        let mut wrapped = false;
        let mut cycle_count = 0;
        'find_end: while item < total_item_count && cycle_count < 2 {
            let child = &children[child_index];
            let count = Self::with_provider(child, |provider| provider.num_layout_nodes());
            for local in current_child_index..count {
                let size = self.get_item_size(child, local, horizontal);
                if running_size + size + gap >= self.offset + self.viewport_size {
                    self.realized_index_end = if self.infinite && wrapped {
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

        // Keep `virtualizeBuffer` lines realized on each side of the visible range
        // so items are mounted and advancing before they scroll in. Buffered items
        // are drawn (clipped away by a normal viewport), but stay out of the
        // visible range, which is what reports measured sizes back to us.
        let mut visible_index_start = self.realized_index_start;
        let mut visible_index_end = self.realized_index_end;
        let buffer = Self::with_scroll(scroll, |scroll| i32::from(scroll.virtualize_buffer()))
            .min(total_item_count);
        if buffer > 0 && total_item_count > 0 {
            let visible_span = self.realized_index_end - self.realized_index_start + 1;
            let max_extra = (total_item_count - visible_span).max(0);
            let before = buffer.min(if self.infinite {
                max_extra
            } else {
                self.realized_index_start
            });
            let after = buffer.min(if self.infinite {
                max_extra - before
            } else {
                total_item_count - 1 - self.realized_index_end
            });
            let before = before.max(0);
            let after = after.max(0);
            for k in 1..=before {
                running_offset -= self.get_item_size_at(
                    self.realized_index_start - k,
                    children,
                    total_item_count,
                    horizontal,
                ) + gap;
            }
            self.realized_index_start -= before;
            self.realized_index_end += after;
            if self.infinite {
                // Indices are modular when infinite, so bias the widened
                // range into positive space and keep visible bounds in the
                // same frame.
                self.realized_index_start += total_item_count;
                self.realized_index_end += total_item_count;
                visible_index_start += total_item_count;
                visible_index_end += total_item_count;
            }
        }

        let actual_start = if self.infinite && total_item_count > 0 {
            self.realized_index_start % total_item_count
        } else {
            self.realized_index_start
        };
        let actual_end = if self.infinite && total_item_count > 0 {
            self.realized_index_end % total_item_count
        } else {
            self.realized_index_end
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
        if last_realized_index_start <= last_realized_index_end {
            for index in last_realized_index_start..=last_realized_index_end {
                if !used.contains(&index) {
                    recycle.push(index);
                }
            }
        } else {
            for index in last_realized_index_start..total_item_count {
                if !used.contains(&index) {
                    recycle.push(index);
                }
            }
            for index in 0..=last_realized_index_end {
                if !used.contains(&index) {
                    recycle.push(index);
                }
            }
        }
        self.recycle_items(recycle, children, total_item_count);

        let mut visible_indices = vec![Vec2D::new(-1.0, -1.0); children.len()];
        let mut realized_indices = vec![Vec2D::new(-1.0, -1.0); children.len()];
        for global_index in self.realized_index_start..=self.realized_index_end {
            let actual_index = if self.infinite {
                global_index % total_item_count
            } else {
                global_index
            };
            // Buffered items are realized and drawn, but only on screen items
            // report their measured size back.
            let is_visible =
                global_index >= visible_index_start && global_index <= visible_index_end;
            let mut running_total = 0;
            'providers: for (provider_index, child) in children.iter().enumerate() {
                let count =
                    Self::with_provider(child, |provider| provider.num_layout_nodes()) as i32;
                let start = running_total;
                let end = start + count;
                if start < end && actual_index < end && actual_index >= start {
                    let local = (actual_index - start) as usize;
                    if realized_indices[provider_index].x == -1.0 {
                        realized_indices[provider_index].x = local as f32;
                    }
                    realized_indices[provider_index].y = local as f32;
                    if is_visible {
                        if visible_indices[provider_index].x == -1.0 {
                            visible_indices[provider_index].x = local as f32;
                        }
                        visible_indices[provider_index].y = local as f32;
                    }
                    let Some(changed) = Self::with_virtualizer_mut(child, |virtualizer| {
                        virtualizer.item(local as i32).is_none()
                    }) else {
                        running_total = end;
                        continue;
                    };
                    if changed {
                        assert!(virtualizing_component::add_virtualizable_handle(
                            child,
                            local as i32
                        ));
                        if !changed_components.contains(child) {
                            changed_components.push(child.clone());
                        }
                    }
                    let size = self.get_item_size(child, local, horizontal);
                    let virtualizable = Self::with_virtualizer_mut(child, |virtualizer| {
                        virtualizer.item(local as i32)
                    })
                    .expect("live VirtualizingComponent");
                    if let Some(virtualizable) = virtualizable {
                        let invertible = child
                            .with(|child| {
                                let component = child
                                    .as_transform_component()
                                    .expect("virtualizing transform");
                                let mut inverse = Mat2D::default();
                                component.world_transform().invert(&mut inverse)
                            })
                            .expect("live virtualizing transform");
                        if !invertible {
                            continue 'providers;
                        }
                        let location = if horizontal {
                            Vec2D::new(
                                running_offset,
                                virtualizable.with_artboard(|a| a.base.layout_y()),
                            )
                        } else {
                            Vec2D::new(
                                virtualizable.with_artboard(|a| a.base.layout_x()),
                                running_offset,
                            )
                        };
                        Self::with_virtualizer_mut(child, |virtualizer| {
                            virtualizer.set_virtualizable_position(local as i32, location);
                        });
                    }
                    running_offset += size + gap;
                    break;
                }
                running_total = end;
            }
        }

        for (index, child) in children.iter().enumerate() {
            let visible = visible_indices[index];
            Self::with_virtualizer_mut(child, |virtualizer| {
                virtualizer.set_visible_indices(visible.x as i32, visible.y as i32);
                let realized = realized_indices[index];
                virtualizer.set_realized_indices(realized.x as i32, realized.y as i32);
            });
        }
        for child in changed_components {
            Self::with_virtualizer_mut(&child, |virtualizer| {
                virtualizer.virtualizable_changed();
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
                let end = start
                    + Self::with_provider(child, |provider| provider.num_layout_nodes()) as i32;
                if start < end && actual_index < end && actual_index >= start {
                    Self::with_virtualizer_mut(child, |virtualizer| {
                        virtualizer.remove_virtualizable(actual_index - start);
                    });
                    break;
                }
                running_total = end;
            }
        }
    }

    fn get_item_size(&self, child: &CoreHandle, index: usize, horizontal: bool) -> f32 {
        if let Some(size) =
            Self::with_virtualizer_mut(child, |virtualizer| virtualizer.item_size(index as i32))
        {
            return if horizontal { size.x } else { size.y };
        }
        Self::with_provider_mut(child, |child| {
            let bounds = child.layout_bounds();
            if horizontal {
                bounds.width()
            } else {
                bounds.height()
            }
        })
    }

    fn get_item_size_at(
        &self,
        global_index: i32,
        children: &[CoreHandle],
        total_item_count: i32,
        horizontal: bool,
    ) -> f32 {
        if total_item_count <= 0 {
            return 0.0;
        }
        let mut index = global_index % total_item_count;
        if index < 0 {
            index += total_item_count;
        }
        let mut running_total = 0;
        for child in children {
            let end = running_total
                + Self::with_provider(child, |provider| provider.num_layout_nodes()) as i32;
            if index < end {
                return self.get_item_size(child, (index - running_total) as usize, horizontal);
            }
            running_total = end;
        }
        0.0
    }
}
