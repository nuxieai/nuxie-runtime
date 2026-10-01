// Copyright (c) Meta Platforms, Inc. and affiliates.
// This source code is licensed under the MIT license found in the LICENSE
// file in the root directory of this source tree.
//! Translated from pinned Yoga tests/YGMeasureCacheTest.cpp (all six cases).
//! Exercise actual trees and retain upstream callback-count assertions.

use crate::prelude::*;

// Pinned Yoga YGNodeComputeFlexBasisForChildren: a single flexible child
// receives a raw zero basis; minimum constraints and padding act at different
// later stages. These expectations are reproduced by the C++ layout engine.
fn single_flex_case(row: bool, minimum: bool, padding: bool) -> (usize, f32) {
    let mut tree = TaffyTree::new();
    tree.disable_rounding();
    let mut style = yoga_style();
    style.flex_grow = if minimum || padding { 0.5 } else { 1.0 };
    style.flex_shrink = 1.0;
    if minimum {
        if row { style.min_size.width = length(20.0); }
        else { style.min_size.height = length(20.0); }
    }
    if padding {
        if row { style.padding.left = length(10.0); style.padding.right = length(10.0); }
        else { style.padding.top = length(10.0); style.padding.bottom = length(10.0); }
    }
    let child = tree.new_leaf_with_context(style, ()).unwrap();
    let root = tree.new_with_children(Style {
        flex_direction: if row { FlexDirection::Row } else { FlexDirection::Column },
        size: Size { width: length(100.0), height: length(100.0) },
        ..yoga_style()
    }, &[child]).unwrap();
    let mut calls = 0;
    tree.compute_layout_with_measure_and_probe(
        root, Size::MAX_CONTENT, |_, _, _| Default::default(),
        |known, _, _, _, _, _| {
            calls += 1;
            Size { width: known.width.unwrap_or(20.0), height: known.height.unwrap_or(10.0) }
        },
    ).unwrap();
    let size = tree.layout(child).unwrap().size;
    (calls, if row { size.width } else { size.height })
}

#[test]
fn rive_single_flex_child_skips_intrinsic_basis_measurement() {
    for row in [false, true] {
        assert_eq!(single_flex_case(row, false, false), (0, 100.0));
    }
}

#[test]
fn rive_single_flex_minimum_is_bounded_before_fractional_growth() {
    for row in [false, true] {
        assert_eq!(single_flex_case(row, true, false), (0, 60.0));
    }
}

#[test]
fn rive_single_flex_padding_does_not_inflate_raw_basis() {
    for row in [false, true] {
        assert_eq!(single_flex_case(row, false, true), (0, 50.0));
    }
}

#[test]
fn rive_single_flex_eligibility_matches_upstream() {
    // Sibling: 0 absent, 1 hidden flexible, 2 absolute flexible,
    // 3 visible grow-only, 4 visible shrink-only.
    for row in [false, true] {
        for (grow, shrink, sibling, exact, skip) in [
            (1.0, 1.0, 0, true, true),
            (1.0, 1.0, 1, true, false),
            (1.0, 1.0, 2, true, true),
            (1.0, 0.0, 0, true, false),
            (0.00005, 1.0, 0, true, false),
            (0.0001, 1.0, 0, true, true),
            (1.0, 1.0, 0, false, false),
            (1.0, 1.0, 3, true, false),
            (1.0, 1.0, 4, true, false),
        ] {
            let mut tree = TaffyTree::new();
            tree.disable_rounding();
            let child = tree.new_leaf_with_context(Style {
                flex_grow: grow, flex_shrink: shrink, ..yoga_style()
            }, ()).unwrap();
            let mut children = vec![child];
            if sibling != 0 {
                children.push(tree.new_leaf(Style {
                    flex_grow: if sibling == 4 { 0.0 } else { 1.0 },
                    flex_shrink: if sibling == 3 { 0.0 } else { 1.0 },
                    display: if sibling == 1 { Display::None } else { Display::Flex },
                    position: if sibling == 2 { Position::Absolute } else { Position::Relative },
                    size: Size { width: length(10.0), height: length(10.0) },
                    ..yoga_style()
                }).unwrap());
            }
            let root = tree.new_with_children(Style {
                flex_direction: if row { FlexDirection::Row } else { FlexDirection::Column },
                size: if exact { Size { width: length(100.0), height: length(100.0) } } else { Size::auto() },
                max_size: Size { width: length(100.0), height: length(100.0) },
                ..yoga_style()
            }, &children).unwrap();
            let mut calls = 0;
            tree.compute_layout_with_measure_and_probe(
                root, Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) },
                |_, _, _| Default::default(),
                |known, _, node, _, _, _| {
                    if node == child { calls += 1; }
                    Size { width: known.width.unwrap_or(20.0), height: known.height.unwrap_or(10.0) }
                },
            ).unwrap();
            assert_eq!(calls == 0, skip, "row={row} grow={grow} shrink={shrink} sibling={sibling} exact={exact}");
        }
    }
}

#[test]
fn rive_single_flex_distribution_edges_match_upstream() {
    // Pinned C++ cases: overflowing fixed sibling, maximum, wrap+minimum,
    // and authored basis overridden by the selected raw zero basis.
    for row in [false, true] {
        for (case, expected_main, expected_cross_offset) in [
            (0, 20.0, 0.0), (1, 30.0, 0.0),
            (2, 60.0, 10.0), (3, 50.0, 0.0),
        ] {
            let mut tree = TaffyTree::new();
            tree.disable_rounding();
            let mut style = yoga_style();
            style.flex_grow = 0.5;
            style.flex_shrink = 1.0;
            if case == 0 || case == 2 {
                if row { style.min_size.width = length(20.0); }
                else { style.min_size.height = length(20.0); }
            }
            if case == 1 {
                if row { style.max_size.width = length(30.0); }
                else { style.max_size.height = length(30.0); }
            }
            if case == 3 { style.flex_basis = length(80.0); }
            let child = tree.new_leaf_with_context(style, ()).unwrap();
            let mut children = Vec::new();
            if case == 0 || case == 2 {
                let main = if case == 0 { 120.0 } else { 90.0 };
                children.push(tree.new_leaf(Style {
                    size: Size {
                        width: length(if row { main } else { 10.0 }),
                        height: length(if row { 10.0 } else { main }),
                    },
                    ..yoga_style()
                }).unwrap());
            }
            children.push(child);
            let root = tree.new_with_children(Style {
                flex_direction: if row { FlexDirection::Row } else { FlexDirection::Column },
                flex_wrap: if case == 2 { FlexWrap::Wrap } else { FlexWrap::NoWrap },
                align_content: Some(AlignContent::STRETCH),
                size: Size { width: length(100.0), height: length(100.0) },
                ..yoga_style()
            }, &children).unwrap();
            let mut calls = 0;
            tree.compute_layout_with_measure_and_probe(
                root, Size::MAX_CONTENT, |_, _, _| Default::default(),
                |known, _, node, _, _, _| {
                    if node == child { calls += 1; }
                    Size { width: known.width.unwrap_or(20.0), height: known.height.unwrap_or(10.0) }
                },
            ).unwrap();
            let layout = tree.layout(child).unwrap();
            let main = if row { layout.size.width } else { layout.size.height };
            let cross_offset = if row { layout.location.y } else { layout.location.x };
            assert_eq!((calls, main, cross_offset), (0, expected_main, expected_cross_offset), "row={row} case={case}");
        }
    }
}

#[test]
fn rive_bounded_cross_offer_is_not_an_exact_stretch_basis() {
    let mut tree = TaffyTree::new();
    tree.disable_rounding();
    let child = tree.new_leaf_with_context(yoga_style(), ()).unwrap();
    let root = tree.new_with_children(Style {
        max_size: Size { width: length(100.0), height: length(100.0) },
        ..yoga_style()
    }, &[child]).unwrap();
    let mut requests = Vec::new();
    tree.compute_layout_with_measure_and_probe(
        root, Size { width: AvailableSpace::Definite(100.0), height: AvailableSpace::Definite(100.0) }, |_, _, _| Default::default(),
        |known, available, _, _, _, _| {
            requests.push((known, available));
            Size { width: known.width.unwrap_or(20.0), height: known.height.unwrap_or(10.0) }
        },
    ).unwrap();
    // YGNodeComputeFlexBasisForChild requires hasExactWidth for stretch.
    // A max-width constrains the offer but does not decide the child's width.
    assert_eq!(requests[0].0.width, None);
    assert_eq!(requests[0].1.width, AvailableSpace::Definite(100.0));
}

fn yoga_style() -> Style {
    Style {
        flex_direction: FlexDirection::Column,
        flex_shrink: 0.0,
        // Yoga flex children have no CSS automatic content minimum.
        min_size: Size {
            width: length(0.0),
            height: length(0.0),
        },
        ..Style::default()
    }
}

#[derive(Clone, Copy)]
enum Measure {
    Min,
    Max,
    Fixed,
}

fn calculate(
    tree: &mut TaffyTree<()>,
    root: NodeId,
    space: Size<AvailableSpace>,
    measure: Measure,
    count: &mut usize,
) {
    // YGNodeCalculateLayout treats finite owner dimensions as exact when the
    // root has no authored size/max. Mirror the runtime's root-style adapter.
    let original = tree.style(root).unwrap().clone();
    let mut resolved = original.clone();
    if resolved.size.width.is_auto() {
        if let AvailableSpace::Definite(width) = space.width {
            resolved.size.width = length(width);
        }
    }
    if resolved.size.height.is_auto() {
        if let AvailableSpace::Definite(height) = space.height {
            resolved.size.height = length(height);
        }
    }
    tree.set_style(root, resolved).unwrap();
    tree.compute_layout_with_measure_and_probe(
        root,
        space,
        |_, _, _| Default::default(),
        |known, available, _, _, _, _| {
            *count += 1;
            let axis = |known: Option<f32>, available: AvailableSpace| {
                let offered = match available {
                    AvailableSpace::Definite(size) => size.max(0.0),
                    AvailableSpace::MinContent => 0.0,
                    AvailableSpace::MaxContent => return 10.0,
                };
                match measure {
                    Measure::Min if known.is_none() && offered > 10.0 => 10.0,
                    _ => offered,
                }
            };
            let measured = match measure {
                Measure::Fixed => Size {
                    width: 84.0,
                    height: 49.0,
                },
                _ => Size {
                    width: axis(known.width, available.width),
                    height: axis(known.height, available.height),
                },
            };
            // The runtime callback projects known outer dimensions after measuring.
            Size {
                width: known.width.unwrap_or(measured.width),
                height: known.height.unwrap_or(measured.height),
            }
        },
    )
    .unwrap();
    tree.set_style(root, original).unwrap();
}

fn pair(align: Option<AlignItems>) -> (TaffyTree<()>, NodeId) {
    let mut tree = TaffyTree::new();
    tree.disable_rounding();
    let child = tree.new_leaf_with_context(yoga_style(), ()).unwrap();
    let root = tree
        .new_with_children(
            Style {
                align_items: align,
                ..yoga_style()
            },
            &[child],
        )
        .unwrap();
    (tree, root)
}

#[test]
fn measure_once_single_flexible_child() {
    let mut tree = TaffyTree::new();
    tree.disable_rounding();
    let child = tree
        .new_leaf_with_context(
            Style {
                flex_grow: 1.0,
                ..yoga_style()
            },
            (),
        )
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Row,
                align_items: Some(AlignItems::FLEX_START),
                size: Size {
                    width: length(100.0),
                    height: length(100.0),
                },
                ..yoga_style()
            },
            &[child],
        )
        .unwrap();
    let mut count = 0;
    calculate(&mut tree, root, Size::MAX_CONTENT, Measure::Max, &mut count);
    assert_eq!(count, 1);
}

#[test]
fn remeasure_with_same_exact_width_larger_than_needed_height() {
    let (mut tree, root) = pair(None);
    let mut count = 0;
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::Definite(100.0),
        },
        Measure::Min,
        &mut count,
    );
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::Definite(50.0),
        },
        Measure::Min,
        &mut count,
    );
    assert_eq!(count, 1);
}

#[test]
fn remeasure_with_same_atmost_width_larger_than_needed_height() {
    let (mut tree, root) = pair(Some(AlignItems::FLEX_START));
    let mut count = 0;
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::Definite(100.0),
        },
        Measure::Min,
        &mut count,
    );
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::Definite(50.0),
        },
        Measure::Min,
        &mut count,
    );
    assert_eq!(count, 1);
}

#[test]
fn remeasure_with_computed_width_larger_than_needed_height() {
    let (mut tree, root) = pair(Some(AlignItems::FLEX_START));
    let mut count = 0;
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::Definite(100.0),
        },
        Measure::Min,
        &mut count,
    );
    let mut style = tree.style(root).unwrap().clone();
    style.align_items = Some(AlignItems::STRETCH);
    tree.set_style(root, style).unwrap();
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(10.0),
            height: AvailableSpace::Definite(50.0),
        },
        Measure::Min,
        &mut count,
    );
    assert_eq!(count, 1);
}

#[test]
fn remeasure_with_atmost_computed_width_undefined_height() {
    let (mut tree, root) = pair(Some(AlignItems::FLEX_START));
    let mut count = 0;
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(100.0),
            height: AvailableSpace::MaxContent,
        },
        Measure::Min,
        &mut count,
    );
    calculate(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(10.0),
            height: AvailableSpace::MaxContent,
        },
        Measure::Min,
        &mut count,
    );
    assert_eq!(count, 1);
}

#[test]
fn remeasure_with_already_measured_value_smaller_but_still_float_equal() {
    let mut tree = TaffyTree::new();
    tree.disable_rounding();
    let measured = tree.new_leaf_with_context(yoga_style(), ()).unwrap();
    let parent = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Row,
                padding: Rect {
                    left: length(2.88),
                    right: length(2.88),
                    top: length(2.88),
                    bottom: length(2.88),
                },
                ..yoga_style()
            },
            &[measured],
        )
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                flex_direction: FlexDirection::Row,
                size: Size {
                    width: length(288.0),
                    height: length(288.0),
                },
                ..yoga_style()
            },
            &[parent],
        )
        .unwrap();
    let mut count = 0;
    calculate(
        &mut tree,
        root,
        Size::MAX_CONTENT,
        Measure::Fixed,
        &mut count,
    );
    assert_eq!(count, 1);
}
