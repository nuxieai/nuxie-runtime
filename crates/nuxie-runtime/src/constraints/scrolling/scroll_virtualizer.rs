//! Direct owner for pinned `src/constraints/scrolling/scroll_virtualizer.cpp`.

use super::super::*;

pub(crate) fn constrain_scroll_virtualizer(
    artboard: &mut ArtboardInstance,
    constraint: ComponentHandle,
    force: bool,
) -> bool {
    let Some((applied, child_count, has_virtualizer)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .map(|scroll| {
            (
                scroll.child_constraint_applied_count,
                scroll.layout_children.len(),
                scroll.virtualizer.is_some(),
            )
        })
    else {
        return false;
    };
    let constraint_local = artboard.component_at(constraint).local_id;
    let virtualize = constraint_bool(
        artboard,
        constraint_local,
        "ScrollConstraint",
        "virtualize",
        false,
    );
    if !virtualize || !has_virtualizer || (!force && applied < child_count) {
        return false;
    }
    let computed_layout_bounds = artboard
        .runtime_graph()
        .and_then(|graph| artboard.runtime_taffy_layout_bounds(graph, artboard.runtime_file()));
    let retained_layout_bounds = artboard.layout_constraint_bounds.clone();
    let layout_bounds = retained_layout_bounds
        .as_deref()
        .or(computed_layout_bounds.as_ref());
    let metrics = {
        let scroll = artboard
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint remains live");
        build_runtime_scroll_layout_metrics(artboard, constraint, scroll, layout_bounds, false)
    };
    let direction = if metrics.main_axis_horizontal {
        RuntimeScrollAxis::X
    } else {
        RuntimeScrollAxis::Y
    };
    let (clamped_x, clamped_y) = clamped_scroll_constraint_offsets(artboard, constraint, &metrics);
    let offset = match direction {
        RuntimeScrollAxis::X => clamped_x,
        RuntimeScrollAxis::Y => clamped_y,
    };
    let viewport_size = metrics.viewport_size(direction);
    let infinite = metrics.infinite;
    let content_size = match direction {
        RuntimeScrollAxis::X => metrics.content_width,
        RuntimeScrollAxis::Y => metrics.content_height,
    };
    // Pinned `ScrollVirtualizer::constrain` returns true but leaves every
    // retained field untouched when content size is non-positive.
    if content_size <= 0.0 {
        return true;
    }
    let provider_item_sizes = {
        let scroll = artboard
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint remains live");
        virtualized_provider_item_sizes(artboard, layout_bounds, scroll, None)
    };
    let gap = match direction {
        RuntimeScrollAxis::X => metrics.gap_x,
        RuntimeScrollAxis::Y => metrics.gap_y,
    };
    let range = exact_scroll_virtualizer_range(
        &provider_item_sizes,
        direction == RuntimeScrollAxis::X,
        gap,
        viewport_size,
        offset,
        infinite,
        content_size,
    );
    let (last_visible_start, last_visible_end) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| scroll.virtualizer.as_ref())
        .map(|virtualizer| (virtualizer.visible_start, virtualizer.visible_end))
        .unwrap_or((0, 0));
    {
        let virtualizer = artboard
            .objects
            .component_mut(constraint)
            .and_then(|component| component.concrete.scroll.as_mut())
            .and_then(|scroll| scroll.virtualizer.as_mut())
            .expect("virtualized ScrollConstraint owns its virtualizer");
        virtualizer.offset = normalized_scroll_virtualizer_offset(offset, infinite, content_size);
        virtualizer.infinite = infinite;
        virtualizer.viewport_size = viewport_size;
        virtualizer.direction = direction;
        virtualizer.visible_start = range.visible_start;
        virtualizer.visible_end = range.visible_end;
    }

    let providers = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.scroll.as_ref())
        .map(|scroll| scroll.layout_children.clone())
        .unwrap_or_default();
    let provider_locals = providers
        .iter()
        .map(|provider| artboard.objects.component_local_id(*provider))
        .collect::<Vec<_>>();
    for provider_local in provider_locals.iter().flatten().copied() {
        if artboard.component_list_state(provider_local).is_some() {
            artboard.set_component_list_visible_indices(provider_local, -1, -1);
        }
    }

    let total_item_count = provider_item_sizes.iter().map(Vec::len).sum::<usize>();
    if total_item_count == 0 {
        return true;
    }
    let actual_start = if infinite {
        range.visible_start.rem_euclid(total_item_count as i32)
    } else {
        range.visible_start
    };
    let actual_end = if infinite {
        range.visible_end.rem_euclid(total_item_count as i32)
    } else {
        range.visible_end
    };
    let mut used_indices = BTreeSet::new();
    if actual_start <= actual_end {
        used_indices.extend(actual_start..=actual_end);
    } else {
        used_indices.extend(actual_start..total_item_count as i32);
        used_indices.extend(0..=actual_end);
    }
    let last_start = if infinite {
        last_visible_start.rem_euclid(total_item_count as i32)
    } else {
        last_visible_start
    };
    let last_end = if infinite {
        last_visible_end.rem_euclid(total_item_count as i32)
    } else {
        last_visible_end
    };
    let mut indices_to_recycle = Vec::new();
    let mut consider_previous = |index: i32| {
        if index >= 0 && !used_indices.contains(&index) {
            indices_to_recycle.push(index as usize);
        }
    };
    if last_start <= last_end {
        for index in last_start..=last_end {
            consider_previous(index);
        }
    } else {
        for index in last_start..total_item_count as i32 {
            consider_previous(index);
        }
        for index in 0..=last_end {
            consider_previous(index);
        }
    }
    indices_to_recycle.sort_unstable();

    let locate_item = |actual_index: usize| {
        let mut running_total = 0usize;
        for (provider_index, child) in provider_item_sizes.iter().enumerate() {
            let start = running_total;
            let end = start + child.len();
            if start < end && actual_index >= start && actual_index < end {
                return Some((provider_index, actual_index - start));
            }
            running_total = end;
        }
        None
    };
    for actual_index in indices_to_recycle {
        let Some((provider_index, logical_index)) = locate_item(actual_index) else {
            continue;
        };
        let Some(provider_local) = provider_locals.get(provider_index).copied().flatten() else {
            continue;
        };
        artboard.remove_component_list_virtualizable(provider_local, logical_index);
    }

    let Some(file) = artboard.runtime_file_arc() else {
        return true;
    };
    let mut visible_indices = vec![(-1_i32, -1_i32); providers.len()];
    let mut changed_providers = BTreeSet::new();
    let mut running_offset = range.running_offset;
    for global_index in range.visible_start..=range.visible_end {
        let actual_index = if infinite {
            global_index.rem_euclid(total_item_count as i32) as usize
        } else {
            global_index as usize
        };
        let Some((provider_index, logical_index)) = locate_item(actual_index) else {
            continue;
        };
        let Some(provider_local) = provider_locals.get(provider_index).copied().flatten() else {
            continue;
        };
        if artboard.component_list_state(provider_local).is_none() {
            continue;
        }
        let visible = &mut visible_indices[provider_index];
        if visible.0 == -1 {
            visible.0 = logical_index as i32;
        }
        visible.1 = logical_index as i32;
        if !artboard.virtualizing_component_has_item(provider_local, logical_index)
            && artboard.add_component_list_virtualizable(&file, provider_local, logical_index)
        {
            changed_providers.insert(provider_local);
        }
        if artboard.virtualizing_component_has_item(provider_local, logical_index) {
            let layout_position = artboard
                .component_list_virtualizable_layout_position(provider_local, logical_index);
            // The pinned virtualizer replaces only the main-axis coordinate.
            // The cross axis stays on the mounted Artboard root's transferred
            // Yoga node (`scroll_virtualizer.cpp:269-291`).
            let position = if direction == RuntimeScrollAxis::X {
                (running_offset, layout_position.1)
            } else {
                (layout_position.0, running_offset)
            };
            artboard.set_component_list_virtualizable_position(
                provider_local,
                logical_index,
                position,
            );
        }
        let size = provider_item_sizes[provider_index][logical_index];
        running_offset += if direction == RuntimeScrollAxis::X {
            size.0
        } else {
            size.1
        } + gap;
    }
    for (provider_index, provider_local) in provider_locals.into_iter().enumerate() {
        let Some(provider_local) = provider_local else {
            continue;
        };
        if artboard.component_list_state(provider_local).is_some() {
            let visible = visible_indices[provider_index];
            artboard.set_component_list_visible_indices(provider_local, visible.0, visible.1);
        }
    }
    for provider_local in changed_providers {
        artboard.component_list_virtualizable_changed(provider_local);
    }
    true
}

pub(crate) fn component_list_virtualization(
    artboard: &ArtboardInstance,
    list_local: usize,
) -> Option<()> {
    let list = artboard.component_handle(list_local)?;
    let constraint = artboard
        .objects
        .component(list)?
        .concrete
        .constrainable_list
        .as_ref()?
        .layout_constraints
        .iter()
        .copied()
        .find(|constraint| {
            artboard
                .objects
                .component(*constraint)
                .and_then(|component| component.concrete.scroll.as_ref())
                .is_some()
        })?;
    let constraint_local = artboard.objects.component_local_id(constraint)?;
    if !constraint_bool(
        artboard,
        constraint_local,
        "ScrollConstraint",
        "virtualize",
        false,
    ) {
        return None;
    }
    Some(())
}

pub(in crate::constraints) fn virtualized_provider_item_sizes(
    artboard: &ArtboardInstance,
    layout_bounds: Option<&std::collections::BTreeMap<usize, crate::draw::RuntimeLayoutBounds>>,
    constraint: &RuntimeScrollConstraintState,
    current_list: Option<(usize, &[(f32, f32)])>,
) -> Vec<Vec<(f32, f32)>> {
    constraint
        .layout_children
        .iter()
        .map(|provider| {
            let Some(provider_local) = artboard.objects.component_local_id(*provider) else {
                return Vec::new();
            };
            if artboard
                .objects
                .component(*provider)
                .is_some_and(|component| component.concrete.constrainable_list.is_some())
            {
                if current_list.is_some_and(|(list_local, _)| provider_local == list_local) {
                    current_list
                        .map(|(_, item_sizes)| item_sizes.to_vec())
                        .unwrap_or_default()
                } else {
                    artboard
                        .component_list_state(provider_local)
                        .map(|list| &list.logical_items)
                        .map(|items| items.iter().map(|item| item.size).collect())
                        .unwrap_or_default()
                }
            } else {
                vec![(
                    layout_component_axis_size(artboard, layout_bounds, provider_local, true),
                    layout_component_axis_size(artboard, layout_bounds, provider_local, false),
                )]
            }
        })
        .collect()
}

pub(in crate::constraints) fn virtualized_provider_content_size(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    infinite: bool,
    leading_padding: f32,
    trailing_padding: f32,
) -> f32 {
    // This intentionally follows `ScrollConstraint::contentWidth/Height`, not
    // merely the flattened node count. Each provider contributes its aggregate
    // layout bounds, then the content layout contributes the inter-provider
    // gaps. For non-empty providers this is algebraically the same as one gap
    // between every flat node; retaining the two levels also matches C++ for an
    // empty list provider.
    let providers_extent = provider_item_sizes
        .iter()
        .map(|items| {
            let item_extent = items
                .iter()
                .map(|size| {
                    let value = if is_horizontal { size.0 } else { size.1 };
                    if value.is_finite() {
                        value.max(0.0)
                    } else {
                        0.0
                    }
                })
                .sum::<f32>();
            item_extent + gap * items.len().saturating_sub(1) as f32
        })
        .sum::<f32>();
    let inter_provider_gap_count = if infinite {
        provider_item_sizes.len()
    } else {
        provider_item_sizes.len().saturating_sub(1)
    };
    let padding = if infinite {
        // An infinite carousel's repeat length excludes layout padding.
        0.0
    } else {
        leading_padding + trailing_padding
    };
    providers_extent + gap * inter_provider_gap_count as f32 + padding
}

#[cfg(test)]
pub(in crate::constraints) fn test_virtualizer_placements_for_metrics(
    item_sizes: &[(f32, f32)],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
) -> Vec<TestVirtualizerPlacement> {
    test_virtualizer_placements_for_providers(
        &[item_sizes.to_vec()],
        is_horizontal,
        gap,
        viewport_size,
        scroll_offset,
        infinite,
        virtualized_provider_content_size(
            &[item_sizes.to_vec()],
            is_horizontal,
            gap,
            infinite,
            0.0,
            0.0,
        ),
    )
    .pop()
    .unwrap_or_default()
}

#[cfg(test)]
pub(in crate::constraints) fn test_virtualizer_placements_for_providers(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
    content_size: f32,
) -> Vec<Vec<TestVirtualizerPlacement>> {
    let range = exact_scroll_virtualizer_range(
        provider_item_sizes,
        is_horizontal,
        gap,
        viewport_size,
        scroll_offset,
        infinite,
        content_size,
    );
    let total_item_count = provider_item_sizes.iter().map(Vec::len).sum::<usize>();
    let mut placements = vec![Vec::new(); provider_item_sizes.len()];
    if total_item_count == 0 {
        return placements;
    }
    let mut running_offset = range.running_offset;
    for global_index in range.visible_start..=range.visible_end {
        let actual_index = if infinite {
            global_index.rem_euclid(total_item_count as i32) as usize
        } else {
            global_index as usize
        };
        let mut running_total = 0usize;
        for (provider_index, child) in provider_item_sizes.iter().enumerate() {
            let start = running_total;
            let end = start + child.len();
            if start < end && actual_index >= start && actual_index < end {
                let logical_index = actual_index - start;
                let item = TestVirtualizerPlacement {
                    logical_index,
                    position_x: if is_horizontal { running_offset } else { 0.0 },
                    position_y: if is_horizontal { 0.0 } else { running_offset },
                };
                if let Some(existing) = placements[provider_index]
                    .iter_mut()
                    .find(|existing| existing.logical_index == logical_index)
                {
                    *existing = item;
                } else {
                    placements[provider_index].push(item);
                }
                let size = provider_item_sizes[provider_index][logical_index];
                running_offset += (if is_horizontal { size.0 } else { size.1 }) + gap;
                break;
            }
            running_total = end;
        }
    }
    placements
}

pub(in crate::constraints) fn normalized_scroll_virtualizer_offset(
    offset: f32,
    infinite: bool,
    content_size: f32,
) -> f32 {
    let normalized_offset = -offset;
    if offset > 0.0 {
        if infinite {
            let offset_multiplier = (offset / content_size).floor() as i32 + 1;
            -1.0 * (offset - offset_multiplier as f32 * content_size)
        } else {
            -offset
        }
    } else {
        let offset_multiplier = (normalized_offset / content_size).floor() as i32;
        if offset_multiplier > 0 {
            normalized_offset % (offset_multiplier as f32 * content_size)
        } else {
            normalized_offset
        }
    }
}

/// Literal range-selection prefix of pinned
/// `ScrollVirtualizer::virtualize`.
///
/// The odd `currentChildIndex` comparisons and the unchanged `childIndex`
/// inside the visible-end loop are intentional pin behavior, not cleanups
/// (`scroll_virtualizer.cpp:54-153`). Recycling and interface calls remain in
/// `constrain_scroll_virtualizer` so production never materializes a Rust-only
/// provider window.
pub(in crate::constraints) fn exact_scroll_virtualizer_range(
    provider_item_sizes: &[Vec<(f32, f32)>],
    is_horizontal: bool,
    gap: f32,
    viewport_size: f32,
    scroll_offset: f32,
    infinite: bool,
    content_size: f32,
) -> RuntimeScrollVirtualizerRange {
    let total_item_count = provider_item_sizes.iter().map(Vec::len).sum::<usize>();
    if provider_item_sizes.is_empty() || total_item_count == 0 || content_size <= 0.0 {
        return RuntimeScrollVirtualizerRange {
            visible_start: 0,
            visible_end: total_item_count as i32 - 1,
            running_offset: 0.0,
        };
    }
    let item_size = |provider: usize, index: usize| {
        let size = provider_item_sizes[provider][index];
        if is_horizontal { size.0 } else { size.1 }
    };
    let offset = normalized_scroll_virtualizer_offset(scroll_offset, infinite, content_size);

    let mut running_size = 0.0;
    let mut running_offset = 0.0;
    let mut running_index = 0usize;
    let mut child_index = 0usize;
    let mut current_child_index = 0usize;
    let mut visible_start = 0usize;
    let mut visible_end = total_item_count - 1;

    'find_start: for (i, child) in provider_item_sizes.iter().enumerate() {
        for j in 0..child.len() {
            let size = item_size(i, j);
            if running_size + size > offset {
                running_offset = running_size - offset;
                visible_start = running_index;
                if current_child_index == provider_item_sizes.len() - 1 {
                    child_index += 1;
                    current_child_index = 0;
                } else {
                    current_child_index += 1;
                }
                break 'find_start;
            }
            running_size += size;
            current_child_index = j;
            running_index += 1;
            if running_size + gap > offset {
                if running_index == total_item_count {
                    running_index = 0;
                }
                if current_child_index == provider_item_sizes.len() - 1 {
                    child_index += 1;
                    current_child_index = 0;
                } else {
                    current_child_index += 1;
                }
                running_size += gap;
                running_offset = running_size - offset;
                visible_start = running_index;
                break 'find_start;
            }
            running_size += gap;
        }
        child_index += 1;
    }

    child_index %= provider_item_sizes.len();
    let mut i = visible_start as i32;
    let mut wrapped = false;
    let mut cycle_count = 0;
    'find_end: while i < total_item_count as i32 && cycle_count < 2 {
        let child = &provider_item_sizes[child_index];
        for j in current_child_index..child.len() {
            let size = item_size(child_index, j);
            if running_size + size + gap >= offset + viewport_size {
                visible_end = if infinite && wrapped {
                    i as usize + total_item_count
                } else {
                    i as usize
                };
                break 'find_end;
            }
            running_size += size + gap;
            running_index += 1;
            if infinite && i == total_item_count as i32 - 1 {
                wrapped = true;
                i = -1;
                cycle_count += 1;
            }
            i += 1;
        }
        // Pinned C++ increments `runningIndex` in this loop even though the
        // visible-end result does not subsequently consult it
        // (`scroll_virtualizer.cpp:107-153`). Keep the literal translation
        // while making that deliberate dead write explicit to Rust.
        let _ = running_index;
        current_child_index = 0;
    }

    RuntimeScrollVirtualizerRange {
        visible_start: visible_start as i32,
        visible_end: visible_end as i32,
        running_offset,
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg(test)]
pub(in crate::constraints) struct TestVirtualizerPlacement {
    pub(in crate::constraints) logical_index: usize,
    pub(in crate::constraints) position_x: f32,
    pub(in crate::constraints) position_y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(in crate::constraints) struct RuntimeScrollVirtualizerRange {
    pub(in crate::constraints) visible_start: i32,
    pub(in crate::constraints) visible_end: i32,
    pub(in crate::constraints) running_offset: f32,
}
