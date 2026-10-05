//! Upstream tests/unit_tests/runtime/scroll_window_sweep_test.cpp.
use super::*;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Default, Debug)]
struct ScanResult {
    realized: BTreeMap<i32, f32>,
    visible: BTreeSet<i32>,
}
fn wrap_index(index: i32, n: i32) -> i32 {
    let wrapped = index % n;
    if wrapped < 0 { wrapped + n } else { wrapped }
}
fn reference_scan(
    sizes: &[f32],
    gap: f32,
    offset: f32,
    viewport: f32,
    infinite: bool,
    buffer: i32,
) -> ScanResult {
    let n = sizes.len() as i32;
    let mut start = 0;
    let mut end = n - 1;
    let mut running_size = 0.0;
    let mut running_offset = 0.0;
    let mut running_index = 0;
    for &size in sizes {
        if running_size + size > offset {
            running_offset = running_size - offset;
            start = running_index;
            break;
        }
        running_size += size;
        running_index += 1;
        if running_size + gap > offset {
            if running_index == n {
                running_index = 0;
            }
            running_size += gap;
            running_offset = running_size - offset;
            start = running_index;
            break;
        }
        running_size += gap;
    }
    let mut i = start;
    let mut cursor = start;
    let mut wrapped = false;
    let mut cycle_count = 0;
    let mut done = false;
    while !done && i < n && cycle_count < 2 {
        for j in cursor..n {
            let size = sizes[j as usize];
            if running_size + size + gap >= offset + viewport {
                end = if infinite {
                    if wrapped { i + n } else { i }
                } else {
                    i
                };
                done = true;
                break;
            }
            running_size += size + gap;
            if infinite && i == n - 1 {
                wrapped = true;
                i = -1;
                cycle_count += 1;
            }
            i += 1;
        }
        cursor = 0;
    }
    let mut visible_start = start;
    let mut visible_end = end;
    if buffer > 0 && n > 0 {
        let visible_span = end - start + 1;
        let max_extra = 0.max(n - visible_span);
        let before = 0.max(buffer.min(if infinite { max_extra } else { start }));
        let after = 0.max(buffer.min(if infinite {
            max_extra - before
        } else {
            n - 1 - end
        }));
        for k in 1..=before {
            running_offset -= sizes[wrap_index(start - k, n) as usize] + gap;
        }
        start -= before;
        end += after;
        if infinite {
            start += n;
            end += n;
            visible_start += n;
            visible_end += n;
        }
    }
    let mut result = ScanResult::default();
    for k in start..=end {
        let index = if infinite { k % n } else { k };
        result.realized.insert(index, running_offset);
        if k >= visible_start && k <= visible_end {
            result.visible.insert(index);
        }
        running_offset += sizes[index as usize] + gap;
    }
    result
}
fn model_scan(
    layout: &VirtualLayout,
    offset: f32,
    viewport: f32,
    infinite: bool,
    buffer: i32,
) -> ScanResult {
    let n = layout.line_count();
    let window = layout.window(offset, viewport, infinite, buffer);
    let mut result = ScanResult::default();
    for line in window.start..=window.end {
        let index = wrap_index(line, n);
        result
            .realized
            .insert(index, layout.line_start(line) - offset);
        if window.is_visible(line) {
            result.visible.insert(index);
        }
    }
    result
}
struct Geometry {
    name: &'static str,
    sizes: Vec<f32>,
    gap: f32,
    viewport: f32,
}
fn geometries() -> Vec<Geometry> {
    // Exact first 23 `20 + (std::mt19937(7)() % 90)` values from upstream.
    let mixed = vec![
        65.0, 42.0, 81.0, 76.0, 33.0, 107.0, 37.0, 79.0, 78.0, 91.0, 108.0, 21.0, 28.0, 107.0,
        22.0, 60.0, 56.0, 102.0, 23.0, 98.0, 86.0, 60.0, 25.0,
    ];
    let uniform = vec![100.0; 20];
    let mut out = Vec::new();
    for gap in [0.0, 10.0, -10.0, 37.5] {
        for viewport in [130.0, 300.0, 333.0, 640.0] {
            out.push(Geometry {
                name: "uniform",
                sizes: uniform.clone(),
                gap,
                viewport,
            });
            out.push(Geometry {
                name: "mixed",
                sizes: mixed.clone(),
                gap,
                viewport,
            });
        }
    }
    out.push(Geometry {
        name: "gap wider than viewport",
        sizes: vec![40.0; 12],
        gap: 80.0,
        viewport: 50.0,
    });
    out.push(Geometry {
        name: "single item",
        sizes: vec![120.0],
        gap: 10.0,
        viewport: 90.0,
    });
    out.push(Geometry {
        name: "two items",
        sizes: vec![60.0, 90.0],
        gap: 5.0,
        viewport: 70.0,
    });
    out
}
#[test]
fn window_matches_the_old_virtualizer_scan() {
    for geometry in geometries() {
        let sizes = &geometry.sizes;
        let mut layout = VirtualLayout::default();
        layout.build_linear(sizes.len() as i32, |i| sizes[i as usize], geometry.gap);
        let largest = sizes.iter().copied().fold(f32::NEG_INFINITY, max);
        for infinite in [false, true] {
            if infinite && geometry.viewport + largest >= layout.cycle_extent() {
                continue;
            }
            for buffer in [0, 1, 2] {
                let from = if infinite { 0.0 } else { -50.0 };
                let to = if infinite {
                    layout.cycle_extent()
                } else {
                    max(0.0, layout.extent() - geometry.viewport)
                };
                let mut offset = from;
                while offset <= to {
                    if infinite && offset >= layout.cycle_extent() {
                        break;
                    }
                    let expected = reference_scan(
                        sizes,
                        geometry.gap,
                        offset,
                        geometry.viewport,
                        infinite,
                        buffer,
                    );
                    let actual = model_scan(&layout, offset, geometry.viewport, infinite, buffer);
                    let context = format!(
                        "{} gap {} viewport {} infinite {infinite} buffer {buffer} offset {offset}",
                        geometry.name, geometry.gap, geometry.viewport
                    );
                    assert_eq!(
                        expected.realized.keys().collect::<Vec<_>>(),
                        actual.realized.keys().collect::<Vec<_>>(),
                        "{context}"
                    );
                    for (index, position) in &expected.realized {
                        assert!(
                            (actual.realized[index] - position).abs() <= 1e-3,
                            "{context}: item {index} at {position} vs {}",
                            actual.realized[index]
                        );
                    }
                    assert_eq!(expected.visible, actual.visible, "{context}");
                    offset += 0.25;
                }
            }
        }
    }
}
#[test]
fn maps_items_to_lines() {
    let mut l = VirtualLayout::default();
    let sizes = [50.0, 60.0, 70.0];
    l.build_linear(3, |i| sizes[i as usize], 10.0);
    assert_eq!(l.line_count(), 3);
    assert_eq!(l.extent(), 200.0);
    assert_eq!(l.cycle_extent(), 210.0);
    assert_eq!(l.line_start(2), 130.0);
    assert_eq!(l.line_start(3), 210.0);
    assert_eq!(l.line_start(-1), 130.0 - 210.0);
    assert_eq!(l.line_of_item(1), 1);
    assert_eq!(l.line_first_item(4), 1);
    assert_eq!(l.line_last_item(-1), 2);
}
#[test]
fn windows_lines_that_start_before_earlier_ones() {
    let mut l = VirtualLayout::default();
    let sizes = [100.0, 10.0, 10.0];
    l.build_linear(3, |i| sizes[i as usize], -50.0);
    let w = l.window(0.0, 40.0, false, 0);
    assert_eq!(w.visible_start, 0);
    assert_eq!(w.visible_end, 2);
}
