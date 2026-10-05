//! Upstream tests/unit_tests/runtime/virtual_layout_grid_test.cpp.
use super::*;
#[derive(Clone)]
struct GridOptions {
    columns: i32,
    column_starts: Vec<f32>,
    template_rows: Vec<VirtualGridTrack>,
    auto_rows: Vec<VirtualGridTrack>,
    row_space: f32,
    align: LayoutCrossAlign,
}
impl Default for GridOptions {
    fn default() -> Self {
        Self {
            columns: 4,
            column_starts: vec![0.0, 110.0, 220.0, 330.0],
            template_rows: Vec::new(),
            auto_rows: Vec::new(),
            row_space: -1.0,
            align: LayoutCrossAlign::Start,
        }
    }
}
fn grid(l: &mut VirtualLayout, heights: &[f32], o: GridOptions) {
    l.begin_grid(10.0, o.columns, o.align, 0.0);
    *l.grid_column_starts() = o.column_starts;
    l.grid_rows().templates = o.template_rows;
    l.grid_rows().autos = o.auto_rows;
    l.grid_rows().space = o.row_space;
    l.begin_segment();
    for &h in heights {
        l.add_item(h, 100.0);
    }
    l.end();
}
fn columns(l: &mut VirtualLayout, with_lines: bool) {
    l.begin_grid(10.0, 4, LayoutCrossAlign::Start, 10.0);
    if with_lines {
        *l.grid_column_starts() = vec![0.0, 110.0, 220.0, 330.0, 430.0];
    }
    l.begin_segment();
    l.add_item(60.0, 100.0);
    l.end();
}
fn approx(actual: f32, expected: f32, margin: f32) {
    let delta = (f64::from(actual) - f64::from(expected)).abs();
    assert!(
        delta <= f64::from(margin)
            || delta <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
        "{actual} != {expected}"
    );
}
#[test]
fn grid_rows_hold_one_item_per_column() {
    let mut l = VirtualLayout::default();
    grid(&mut l, &[60.0; 10], GridOptions::default());
    assert!(l.is_grid());
    assert_eq!(l.line_count(), 3);
    assert_eq!(l.line_last_item(0), 3);
    assert_eq!(l.line_first_item(2), 8);
    assert_eq!(l.line_start(1), 70.0);
    assert_eq!(l.line_start(2), 140.0);
    assert_eq!(l.extent(), 200.0);
}
#[test]
fn grid_items_take_their_column_start() {
    let mut l = VirtualLayout::default();
    grid(&mut l, &[60.0; 6], GridOptions::default());
    for (i, e) in [0.0, 110.0, 220.0, 330.0, 0.0, 110.0]
        .into_iter()
        .enumerate()
    {
        assert_eq!(l.item_flow_offset(i as i32), e);
    }
}
#[test]
fn grid_row_tracks() {
    let mut l = VirtualLayout::default();
    grid(
        &mut l,
        &[60.0; 20],
        GridOptions {
            template_rows: vec![
                VirtualGridTrack::points(80.0),
                VirtualGridTrack::auto_size(),
            ],
            auto_rows: vec![
                VirtualGridTrack::points(40.0),
                VirtualGridTrack::points(50.0),
            ],
            ..Default::default()
        },
    );
    for (r, e) in [0.0, 90.0, 160.0, 210.0, 270.0].into_iter().enumerate() {
        assert_eq!(l.line_start(r as i32), e);
    }
    assert_eq!(l.extent(), 310.0);
    let mut l = VirtualLayout::default();
    grid(
        &mut l,
        &[60.0, 90.0, 60.0, 60.0, 30.0],
        GridOptions::default(),
    );
    assert_eq!(l.line_start(1), 100.0);
    assert_eq!(l.extent(), 130.0);
}
#[test]
fn grid_only_centers_within_a_row() {
    let mut l = VirtualLayout::default();
    let mut o = GridOptions {
        align: LayoutCrossAlign::Center,
        ..Default::default()
    };
    grid(&mut l, &[60.0, 20.0], o.clone());
    assert_eq!(l.item_line_offset(1), 20.0);
    o.align = LayoutCrossAlign::End;
    grid(&mut l, &[60.0, 20.0], o);
    assert_eq!(l.item_line_offset(1), 0.0);
}
#[test]
fn grid_fallbacks() {
    let mut l = VirtualLayout::default();
    grid(
        &mut l,
        &[60.0; 4],
        GridOptions {
            column_starts: vec![],
            ..Default::default()
        },
    );
    assert_eq!(l.item_flow_offset(3), 0.0);
    let mut l = VirtualLayout::default();
    grid(
        &mut l,
        &[60.0; 3],
        GridOptions {
            columns: 0,
            ..Default::default()
        },
    );
    assert_eq!(l.line_count(), 3);
}
#[test]
fn grid_windows_select_whole_rows() {
    let mut l = VirtualLayout::default();
    grid(&mut l, &[60.0; 20], GridOptions::default());
    let w = l.window(65.0, 130.0, false, 0);
    assert_eq!(w.start, 1);
    assert_eq!(w.end, 2);
    assert_eq!(l.line_first_item(w.start), 4);
    assert_eq!(l.line_last_item(w.end), 11);
}
#[test]
fn grid_column_windows() {
    let mut l = VirtualLayout::default();
    columns(&mut l, true);
    let w = l.column_window(0.0, 200.0, 0, false);
    assert_eq!(w.start, 0);
    assert_eq!(w.end, 1);
    let w = l.column_window(150.0, 200.0, 0, false);
    assert_eq!(w.start, 1);
    assert_eq!(w.end, 3);
    assert_eq!(l.column_window(100.0, 200.0, 0, false).visible_start, 1);
    let w = l.column_window(150.0, 100.0, 1, false);
    assert_eq!(w.visible_start, 1);
    assert_eq!(w.visible_end, 2);
    assert_eq!(w.start, 0);
    assert_eq!(w.end, 3);
    columns(&mut l, false);
    let w = l.column_window(150.0, 100.0, 0, false);
    assert_eq!(w.start, 0);
    assert_eq!(w.end, 3);
}
#[test]
fn grid_columns_cycle() {
    let mut l = VirtualLayout::default();
    columns(&mut l, true);
    approx(l.column_cycle_extent(), 440.0, 0.0);
    approx(l.column_start(4), 440.0, 0.0);
    approx(l.column_start(-1), -110.0, 0.0);
    assert_eq!(l.wrap_column(-1), 3);
    assert_eq!(l.wrap_column(5), 1);
    columns(&mut l, false);
    assert_eq!(l.column_cycle_extent(), 0.0);
    columns(&mut l, true);
    let w = l.column_window(300.0, 200.0, 0, true);
    assert_eq!(w.visible_start, 2);
    assert_eq!(w.visible_end, 4);
    let w = l.column_window(-50.0, 100.0, 0, true);
    assert_eq!(w.visible_start, -1);
    assert_eq!(w.visible_end, 0);
    let w = l.column_window(435.0, 50.0, 0, true);
    assert_eq!(w.visible_start, 4);
    assert_eq!(w.visible_end, 4);
    let w = l.column_window(300.0, 200.0, 1, true);
    assert_eq!(w.start, 1);
    assert_eq!(w.end, 4);
    let w = l.column_window(0.0, 2000.0, 2, true);
    assert_eq!(w.start, 0);
    assert_eq!(w.end, 3);
}
#[test]
fn cycling_column_windows_match_a_brute_force_sweep() {
    let mut l = VirtualLayout::default();
    columns(&mut l, true);
    let starts = [0.0, 110.0, 220.0, 330.0];
    let start_of = |column: i32| {
        let wrapped = ((column % 4) + 4) % 4;
        ((column - wrapped) / 4) as f32 * 440.0 + starts[wrapped as usize]
    };
    for viewport in [0.0, 50.0, 100.0, 200.0, 440.0, 1000.0] {
        let mut offset = -1000.0;
        while offset <= 1000.0 {
            let mut start = -20;
            while start_of(start) + 100.0 <= offset {
                start += 1;
            }
            let mut end = start;
            while end < start + 3 && start_of(end + 1) < offset + viewport {
                end += 1;
            }
            let w = l.column_window(offset, viewport, 0, true);
            assert_eq!(
                w.visible_start, start,
                "offset {offset} viewport {viewport}"
            );
            assert_eq!(w.visible_end, end, "offset {offset} viewport {viewport}");
            offset += 0.25;
        }
    }
}
struct RowCase {
    rows: &'static [VirtualGridTrack],
    heights: &'static [f32],
    gap: f32,
    row_space: f32,
    tops: &'static [f32],
    height: f32,
}
struct ColumnCase {
    columns: &'static [VirtualGridTrack],
    widths: &'static [f32],
    gap: f32,
    column_space: f32,
    resize: bool,
    starts: &'static [f32],
}
#[path = "virtual_layout_grid_cases.rs"]
mod cases;
#[test]
fn grid_rows_land_where_layout_puts_them() {
    for (index, c) in cases::ROW_CASES.iter().enumerate() {
        let mut l = VirtualLayout::default();
        l.begin_grid(c.gap, 1, LayoutCrossAlign::Start, 0.0);
        *l.grid_column_starts() = vec![0.0, 100.0];
        l.grid_rows().templates = c.rows.to_vec();
        l.grid_rows().space = c.row_space;
        l.begin_segment();
        for &h in c.heights {
            l.add_item(h, 100.0);
        }
        l.end();
        for (item, &top) in c.tops.iter().enumerate() {
            let actual =
                l.line_start(l.line_of_item(item as i32)) + l.item_line_offset(item as i32);
            let _ = index;
            approx(actual, top, 0.01);
        }
        if c.height >= 0.0 {
            approx(l.extent(), c.height, 0.01);
        }
    }
}
#[test]
fn grid_columns_land_where_layout_puts_them() {
    for c in cases::COLUMN_CASES {
        let count = c.columns.len() as i32;
        let mut l = VirtualLayout::default();
        l.begin_grid(10.0, count, LayoutCrossAlign::Start, c.gap);
        l.grid_columns().templates = c.columns.to_vec();
        l.grid_columns().space = c.column_space;
        l.grid_columns().resize = c.resize;
        l.begin_segment();
        for &w in c.widths {
            l.add_item(50.0, w);
        }
        l.end();
        let mut start = 0.0;
        for column in 0..count {
            approx(start, c.starts[column as usize], 0.01);
            start += l.grid_column_size(column) + c.gap;
        }
    }
}
