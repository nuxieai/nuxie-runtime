//! Source-derived dependency regressions, separate from the seven upstream
//! 8e8492f8 fixture cases. Rive Yoga's available-inner-size calculation clamps
//! the grid min-content probe's zero bound to the authored minimum before
//! collecting flex lines; clamping only the final container width is too late.

use taffy::{TaffyTree, prelude::*};

fn wrapping_row(min_width: f32, child_width: f32, margin_left: f32) -> [Layout; 3] {
    let mut tree = TaffyTree::<()>::new();
    tree.disable_rounding();
    let child_style = Style {
        size: Size {
            width: length(child_width),
            height: length(10.0),
        },
        flex_shrink: 0.0,
        ..Default::default()
    };
    let first = tree.new_leaf(child_style.clone()).unwrap();
    let second = tree.new_leaf(child_style).unwrap();
    let parent = tree
        .new_with_children(
            Style {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                flex_wrap: FlexWrap::Wrap,
                min_size: Size {
                    width: length(min_width),
                    height: auto(),
                },
                margin: Rect {
                    left: length(margin_left),
                    right: length(0.0),
                    top: length(0.0),
                    bottom: length(0.0),
                },
                ..Default::default()
            },
            &[first, second],
        )
        .unwrap();
    tree.compute_layout(
        parent,
        Size {
            width: AvailableSpace::MinContent,
            height: AvailableSpace::MaxContent,
        },
    )
    .unwrap();
    [
        *tree.layout(parent).unwrap(),
        *tree.layout(first).unwrap(),
        *tree.layout(second).unwrap(),
    ]
}

#[test]
fn intrinsic_probe_minimum_keeps_two_children_on_one_line() {
    let [parent, first, second] = wrapping_row(100.0, 40.0, 0.0);
    assert_eq!((parent.size.width, parent.size.height), (100.0, 10.0));
    assert_eq!((first.location.x, first.location.y), (0.0, 0.0));
    assert_eq!((second.location.x, second.location.y), (40.0, 0.0));
}

#[test]
fn intrinsic_probe_minimum_still_wraps_when_children_do_not_fit() {
    let [parent, first, second] = wrapping_row(70.0, 40.0, 0.0);
    assert_eq!((parent.size.width, parent.size.height), (70.0, 20.0));
    assert_eq!((first.location.x, first.location.y), (0.0, 0.0));
    assert_eq!((second.location.x, second.location.y), (0.0, 10.0));
}

#[test]
fn intrinsic_probe_promotes_overflow_from_total_before_wrapping() {
    // Zero available width minus -100 margin yields 100 inner space.
    // Both 60px bases total 120, so Yoga promotes AtMost to Exactly even
    // though the longest wrapped line is only 60px.
    let [parent, first, second] = wrapping_row(0.0, 60.0, -100.0);
    assert_eq!((parent.size.width, parent.size.height), (100.0, 20.0));
    assert_eq!((first.location.x, first.location.y), (0.0, 0.0));
    assert_eq!((second.location.x, second.location.y), (0.0, 10.0));
}
