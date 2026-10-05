//! Upstream tests/unit_tests/runtime/virtual_layout_wrap_test.cpp.
use super::*;
#[derive(Clone, Copy)]
struct WrapOptions {
    line_gap: f32,
    flow_gap: f32,
    flow_start: f32,
    flow_extent: f32,
    justify: LayoutMainDistribute,
    align: LayoutCrossAlign,
    hugs_lines: bool,
}
impl Default for WrapOptions {
    fn default() -> Self {
        Self {
            line_gap: 10.0,
            flow_gap: 10.0,
            flow_start: 0.0,
            flow_extent: 500.0,
            justify: LayoutMainDistribute::Start,
            align: LayoutCrossAlign::Start,
            hugs_lines: false,
        }
    }
}
fn wrap(layout: &mut VirtualLayout, items: &[(f32, f32)], o: WrapOptions) {
    layout.begin_wrap(
        o.line_gap,
        o.flow_gap,
        o.flow_start,
        o.flow_extent,
        o.justify,
        o.align,
        o.hugs_lines,
    );
    layout.begin_segment();
    for &(extent, flow) in items {
        layout.add_item(extent, flow);
    }
    layout.end();
}
fn uniform(count: usize) -> Vec<(f32, f32)> {
    vec![(60.0, 100.0); count]
}
fn approx(actual: f32, expected: f32) {
    assert!(
        (f64::from(actual) - f64::from(expected)).abs()
            <= f64::from(100.0 * f32::EPSILON) * f64::from(expected).abs(),
        "{actual} != {expected}"
    );
}
#[test]
fn wrap_breaks_lines_where_flex_wrap_does() {
    let mut layout = VirtualLayout::default();
    wrap(&mut layout, &uniform(20), WrapOptions::default());
    assert_eq!(layout.line_count(), 5);
    for line in 0..5 {
        assert_eq!(layout.line_first_item(line), line * 4);
        assert_eq!(layout.line_last_item(line), line * 4 + 3);
        assert_eq!(layout.line_start(line), line as f32 * 70.0);
    }
    assert_eq!(layout.extent(), 340.0);
    assert_eq!(layout.line_of_item(9), 2);
    assert_eq!(layout.item_flow_offset(0), 0.0);
    assert_eq!(layout.item_flow_offset(1), 110.0);
    assert_eq!(layout.item_flow_offset(3), 330.0);
}
#[test]
fn wrap_line_edge_cases() {
    let mut layout = VirtualLayout::default();
    wrap(
        &mut layout,
        &uniform(8),
        WrapOptions {
            flow_extent: 430.0,
            ..Default::default()
        },
    );
    assert_eq!(layout.line_count(), 2);
    assert_eq!(layout.line_last_item(0), 3);
    let mut layout = VirtualLayout::default();
    wrap(&mut layout, &uniform(10), WrapOptions::default());
    assert_eq!(layout.line_count(), 3);
    assert_eq!(layout.line_first_item(2), 8);
    assert_eq!(layout.line_last_item(2), 9);
    let mut layout = VirtualLayout::default();
    wrap(
        &mut layout,
        &[(60.0, 100.0), (60.0, 600.0), (60.0, 100.0)],
        WrapOptions::default(),
    );
    assert_eq!(layout.line_count(), 3);
    assert_eq!(layout.line_first_item(1), 1);
    assert_eq!(layout.line_last_item(1), 1);
    let mut layout = VirtualLayout::default();
    wrap(
        &mut layout,
        &uniform(4),
        WrapOptions {
            flow_start: 25.0,
            ..Default::default()
        },
    );
    assert_eq!(layout.item_flow_offset(0), 25.0);
    assert_eq!(layout.item_flow_offset(1), 135.0);
    let mut layout = VirtualLayout::default();
    wrap(
        &mut layout,
        &[
            (60.0, 100.0),
            (90.0, 100.0),
            (60.0, 100.0),
            (60.0, 100.0),
            (40.0, 100.0),
        ],
        WrapOptions::default(),
    );
    assert_eq!(layout.line_start(1), 100.0);
    assert_eq!(layout.extent(), 140.0);
}
#[test]
fn wrap_justifies_each_line_on_its_own() {
    let mut layout = VirtualLayout::default();
    let mut o = WrapOptions {
        justify: LayoutMainDistribute::Center,
        ..Default::default()
    };
    wrap(&mut layout, &uniform(6), o);
    assert_eq!(layout.item_flow_offset(0), 35.0);
    assert_eq!(layout.item_flow_offset(4), 145.0);
    o.justify = LayoutMainDistribute::End;
    wrap(&mut layout, &uniform(6), o);
    assert_eq!(layout.item_flow_offset(0), 70.0);
    assert_eq!(layout.item_flow_offset(5), 400.0);
    o.justify = LayoutMainDistribute::SpaceBetween;
    wrap(&mut layout, &uniform(6), o);
    assert_eq!(layout.item_flow_offset(0), 0.0);
    approx(layout.item_flow_offset(3), 400.0);
    approx(layout.item_flow_offset(5), 400.0);
    wrap(&mut layout, &uniform(1), o);
    assert_eq!(layout.item_flow_offset(0), 0.0);
}
#[test]
fn wrap_aligns_items_within_their_line() {
    let mut layout = VirtualLayout::default();
    let items = [(60.0, 100.0), (20.0, 100.0)];
    let mut o = WrapOptions::default();
    wrap(&mut layout, &items, o);
    assert_eq!(layout.item_line_offset(1), 0.0);
    o.align = LayoutCrossAlign::Center;
    wrap(&mut layout, &items, o);
    assert_eq!(layout.item_line_offset(1), 20.0);
    o.align = LayoutCrossAlign::End;
    wrap(&mut layout, &items, o);
    assert_eq!(layout.item_line_offset(1), 40.0);
    assert_eq!(layout.item_line_offset(0), 0.0);
}
#[test]
fn wrap_windows_select_whole_lines() {
    let mut layout = VirtualLayout::default();
    wrap(&mut layout, &uniform(20), WrapOptions::default());
    let w = layout.window(0.0, 130.0, false, 0);
    assert_eq!(w.start, 0);
    assert_eq!(w.end, 1);
    assert_eq!(layout.line_last_item(w.end), 7);
    let w = layout.window(65.0, 130.0, false, 0);
    assert_eq!(w.start, 1);
    assert_eq!(w.end, 2);
}
#[test]
fn wrap_keeps_segments_across_lines() {
    let mut layout = VirtualLayout::default();
    layout.begin_wrap(
        10.0,
        10.0,
        0.0,
        500.0,
        LayoutMainDistribute::Start,
        LayoutCrossAlign::Start,
        false,
    );
    layout.begin_segment();
    layout.add_item(60.0, 100.0);
    layout.begin_segment();
    for _ in 0..6 {
        layout.add_item(60.0, 100.0);
    }
    layout.end();
    assert_eq!(layout.segment_of(0), 0);
    assert_eq!(layout.segment_of(4), 1);
    assert_eq!(layout.segment_start(1), 1);
    assert_eq!(layout.line_count(), 2);
}
#[test]
fn linear_reports_no_flow_or_line_offsets() {
    let mut layout = VirtualLayout::default();
    layout.build_linear(3, |_| 50.0, 10.0);
    assert!(!layout.wraps());
    assert_eq!(layout.item_flow_offset(1), 0.0);
    assert_eq!(layout.item_line_offset(1), 0.0);
}
#[test]
fn wrap_hugging_lines_justify_within_the_widest() {
    let mut layout = VirtualLayout::default();
    wrap(
        &mut layout,
        &uniform(6),
        WrapOptions {
            justify: LayoutMainDistribute::Center,
            hugs_lines: true,
            ..Default::default()
        },
    );
    assert_eq!(layout.line_count(), 2);
    assert_eq!(layout.item_flow_offset(0), 0.0);
    assert_eq!(layout.item_flow_offset(4), 110.0);
}
