use crate::mechanical_port::source::{
    artboard::RuntimeArtboardInstanceWeakHandle,
    constraints::scrolling::{
        scroll_constraint::ScrollConstraint,
        virtual_layout::{VirtualLayout, VirtualWindow},
    },
    core::CoreHandle,
    generated::core_registry::CoreCapabilities,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    virtualizing_component::{self, VirtualizedDirection, VirtualizingComponent},
};

pub struct ScrollVirtualizer {
    anchor_item: i32,
    anchor_start: f32,
    anchor_item_count: i32,
    anchor_instance: Option<RuntimeArtboardInstanceWeakHandle>,
    offset: f32,
    infinite: bool,
    viewport_size: f32,
    windows_columns: bool,
    pinned_cells: bool,
    line_start: i32,
    line_end: i32,
    flow_offset: f32,
    column_start: f32,
    column_viewport: f32,
    direction: VirtualizedDirection,
}
impl Default for ScrollVirtualizer {
    fn default() -> Self {
        Self {
            anchor_item: -1,
            anchor_start: 0.0,
            anchor_item_count: 0,
            anchor_instance: None,
            offset: 0.0,
            infinite: false,
            viewport_size: 0.0,
            windows_columns: false,
            pinned_cells: false,
            line_start: 0,
            line_end: -1,
            flow_offset: 0.0,
            column_start: 0.0,
            column_viewport: 0.0,
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
    fn with_scroll<R>(scroll: &CoreHandle, f: impl FnOnce(&ScrollConstraint) -> R) -> R {
        scroll
            .with_downcast::<ScrollConstraint, _>(f)
            .expect("live ScrollConstraint")
    }
    pub(super) fn with_virtualizer_mut<R>(
        child: &CoreHandle,
        f: impl FnOnce(&mut dyn VirtualizingComponent) -> R,
    ) -> Option<R> {
        child
            .with_mut(|child| virtualizing_component::from(child).map(f))
            .flatten()
    }
    pub fn reset(&mut self) {
        self.anchor_item = -1;
    }
    pub fn realized_line_start(&self) -> i32 {
        self.line_start
    }
    pub fn realized_line_end(&self) -> i32 {
        self.line_end
    }
    pub fn anchor_moved(&self, layout: &VirtualLayout, children: &[CoreHandle]) -> f32 {
        if self.anchor_item < 0 {
            return 0.0;
        }
        let mut item = self.anchor_item;
        if let Some(anchor) = &self.anchor_instance {
            item = -1;
            for segment in 0..(children.len() as i32).min(layout.segment_count()) {
                let first = layout.segment_start(segment);
                let local = self.anchor_item - first;
                let found = Self::with_virtualizer_mut(&children[segment as usize], |virt| {
                    if local >= 0
                        && local < virt.item_count()
                        && virt
                            .item(local)
                            .is_some_and(|v| v.downgrade().ptr_eq(anchor))
                    {
                        return Some(self.anchor_item);
                    }
                    let mut held = Vec::new();
                    virt.realized_indices(&mut held);
                    held.into_iter()
                        .find(|index| {
                            virt.item(*index)
                                .is_some_and(|v| v.downgrade().ptr_eq(anchor))
                        })
                        .map(|index| first + index)
                })
                .flatten();
                if let Some(found) = found {
                    item = found;
                    break;
                }
            }
            if item < 0 || item >= layout.item_count() {
                return 0.0;
            }
        } else if layout.item_count() != self.anchor_item_count {
            return 0.0;
        }
        layout.line_start(layout.line_of_item(item)) - self.anchor_start
    }
    pub fn constrain(
        &mut self,
        scroll: &CoreHandle,
        children: &[CoreHandle],
        offset: f32,
        direction: VirtualizedDirection,
        flow_offset: f32,
        column_start: f32,
        column_viewport: f32,
        windows_columns: bool,
    ) -> bool {
        self.windows_columns = windows_columns;
        self.flow_offset = flow_offset;
        self.column_start = column_start;
        self.column_viewport = column_viewport;
        let horizontal = direction == VirtualizedDirection::Horizontal;
        let content_size = Self::with_scroll(scroll, |s| {
            if horizontal {
                s.content_width()
            } else {
                s.content_height()
            }
        }) as f64;
        if content_size > 0.0 {
            let normalized = -offset;
            self.direction = direction;
            self.viewport_size = Self::with_scroll(scroll, |s| {
                if horizontal {
                    s.viewport_width()
                } else {
                    s.viewport_height()
                }
            });
            self.infinite = Self::with_scroll(scroll, |s| s.infinite());
            self.offset = if offset > 0.0 {
                if self.infinite {
                    let multiplier = (f64::from(offset) / content_size).floor() as i32 + 1;
                    (-(f64::from(offset) - f64::from(multiplier) * content_size)) as f32
                } else {
                    -offset
                }
            } else {
                let multiplier = (f64::from(normalized) / content_size).floor() as i32;
                if multiplier > 0 {
                    (f64::from(normalized) % (f64::from(multiplier) * content_size)) as f32
                } else {
                    normalized
                }
            };
            self.virtualize(scroll, children);
        }
        true
    }
    pub fn virtualize(&mut self, scroll: &CoreHandle, children: &[CoreHandle]) {
        let Some(model) = Self::with_scroll(scroll, |s| s.virtual_layout()) else {
            return;
        };
        let layout = model.borrow();
        if layout.segment_count() != children.len() as i32 {
            return;
        }
        let horizontal = self.direction == VirtualizedDirection::Horizontal;
        let total = layout.item_count();
        let buffer = Self::with_scroll(scroll, |s| i32::from(s.virtualize_buffer())).min(total);
        let window = layout.window(self.offset, self.viewport_size, self.infinite, buffer);
        self.line_start = window.start;
        self.line_end = window.end;
        let windows_columns = self.windows_columns && layout.is_grid();
        let loops_columns = windows_columns && self.infinite;
        let mut flow_offset = self.flow_offset;
        let mut column_start = self.column_start;
        let column_cycle = layout.column_cycle_extent();
        if loops_columns && column_cycle > 0.0 {
            let cycles = (column_start / column_cycle).floor() * column_cycle;
            flow_offset -= cycles;
            column_start -= cycles;
        }
        let columns = if windows_columns {
            layout.column_window(column_start, self.column_viewport, buffer, loops_columns)
        } else {
            VirtualWindow::default()
        };
        let window_column = |line, item| {
            let column = item - layout.line_first_item(line);
            if loops_columns {
                columns.start + layout.wrap_column(column - columns.start)
            } else {
                column
            }
        };
        let in_columns = |line, item| {
            !windows_columns || {
                let column = window_column(line, item);
                column >= columns.start && column <= columns.end
            }
        };
        for child in children {
            Self::with_virtualizer_mut(child, |v| v.clear_virtual_window());
        }
        let mut used = vec![false; total as usize];
        for line in window.start..=window.end {
            for item in layout.line_first_item(line)..=layout.line_last_item(line) {
                if in_columns(line, item) {
                    used[item as usize] = true;
                }
            }
        }
        for (segment, child) in children.iter().enumerate() {
            Self::with_virtualizer_mut(child, |virt| {
                let mut held = Vec::new();
                virt.realized_indices(&mut held);
                let first = layout.segment_start(segment as i32);
                let count = if segment + 1 < children.len() {
                    layout.segment_start(segment as i32 + 1)
                } else {
                    total
                } - first;
                let held_items: Vec<_> = held
                    .iter()
                    .map(|i| virt.item(*i).map(|v| v.downgrade()))
                    .collect();
                let kept: Vec<_> = held
                    .iter()
                    .zip(&held_items)
                    .filter(|(i, _)| **i < count && used[(first + **i) as usize])
                    .map(|(_, v)| v.clone())
                    .collect();
                let same =
                    |a: &Option<RuntimeArtboardInstanceWeakHandle>,
                     b: &Option<RuntimeArtboardInstanceWeakHandle>| match (a, b)
                    {
                        (Some(a), Some(b)) => a.ptr_eq(b),
                        (None, None) => true,
                        _ => false,
                    };
                let mut recycled = Vec::new();
                for (index, shared) in held.iter().zip(held_items) {
                    if *index < count && used[(first + *index) as usize] {
                        continue;
                    }
                    if !kept.iter().any(|v| same(v, &shared))
                        && !recycled.iter().any(|v| same(v, &shared))
                    {
                        recycled.push(shared);
                        virt.remove_virtualizable(*index);
                    }
                }
            });
        }
        self.anchor_item = -1;
        for line in window.visible_start..=window.visible_end {
            if self.infinite || window.is_empty() || self.anchor_item >= 0 {
                break;
            }
            let first = layout.line_first_item(line);
            let last = layout.line_last_item(line);
            let shown = if windows_columns {
                columns.visible_end - columns.visible_start + 1
            } else {
                1
            };
            for n in 0..shown {
                let column = if windows_columns {
                    if loops_columns {
                        layout.wrap_column(columns.visible_start + n)
                    } else {
                        columns.visible_start + n
                    }
                } else {
                    0
                };
                if first + column <= last {
                    self.anchor_item = first + column;
                    self.anchor_start = layout.line_start(line);
                    self.anchor_item_count = total;
                    break;
                }
            }
        }
        let mut changed = Vec::<CoreHandle>::new();
        for line in window.start..=window.end {
            let visible = window.is_visible(line);
            let position = layout.line_start(line) - self.offset;
            let first = layout.line_first_item(line);
            let last = layout.line_last_item(line);
            let items: Vec<_> = if loops_columns {
                (columns.start..=columns.end)
                    .map(|c| first + layout.wrap_column(c))
                    .filter(|i| *i <= last)
                    .collect()
            } else {
                (first..=last).filter(|i| in_columns(line, *i)).collect()
            };
            for item in items {
                let segment = layout.segment_of(item);
                let child = &children[segment as usize];
                let local = item - layout.segment_start(segment);
                let Some(missing) = Self::with_virtualizer_mut(child, |virt| {
                    virt.add_to_virtual_window(
                        local,
                        visible
                            && (!windows_columns || columns.is_visible(window_column(line, item))),
                    );
                    virt.item(local).is_none()
                }) else {
                    continue;
                };
                if missing {
                    assert!(virtualizing_component::add_virtualizable_handle(
                        child, local
                    ));
                    if !changed.contains(child) {
                        changed.push(child.clone());
                    }
                }
                if layout.is_grid() || self.pinned_cells {
                    Self::with_virtualizer_mut(child, |v| {
                        v.set_virtualizable_cell(
                            local,
                            if windows_columns {
                                item - layout.line_first_item(line)
                            } else {
                                -1
                            },
                            if windows_columns {
                                line - window.start
                            } else {
                                -1
                            },
                        )
                    });
                }
                let Some(instance) = Self::with_virtualizer_mut(child, |v| v.item(local)).flatten()
                else {
                    continue;
                };
                let invertible = child
                    .with(|c| {
                        let mut inverse = Mat2D::default();
                        c.as_transform_component()
                            .expect("virtualizing transform")
                            .world_transform()
                            .invert(&mut inverse)
                    })
                    .expect("live virtualizing transform");
                if !invertible {
                    continue;
                }
                let mut flow = if !layout.flow_from_layout() {
                    layout.item_flow_offset(item)
                } else {
                    instance.with_artboard(|a| {
                        if horizontal {
                            a.base.layout_y()
                        } else {
                            a.base.layout_x()
                        }
                    })
                };
                flow -= flow_offset;
                if loops_columns {
                    flow += layout.column_start(window_column(line, item))
                        - layout.column_start(item - layout.line_first_item(line));
                }
                let position = position + layout.item_line_offset(item);
                let location = if horizontal {
                    Vec2D::new(position, flow)
                } else {
                    Vec2D::new(flow, position)
                };
                Self::with_virtualizer_mut(child, |v| {
                    v.set_virtualizable_position(local, location)
                });
            }
        }
        self.anchor_instance = if self.anchor_item >= 0 {
            let segment = layout.segment_of(self.anchor_item);
            Self::with_virtualizer_mut(&children[segment as usize], |v| {
                v.item(self.anchor_item - layout.segment_start(segment))
            })
            .flatten()
            .map(|v| v.downgrade())
        } else {
            None
        };
        self.pinned_cells = windows_columns;
        changed.sort_by_key(CoreHandle::slot_address);
        for child in changed {
            Self::with_virtualizer_mut(&child, |v| v.virtualizable_changed());
        }
    }
}
