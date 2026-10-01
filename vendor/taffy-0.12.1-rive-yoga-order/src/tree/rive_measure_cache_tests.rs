// Copyright (c) Meta Platforms, Inc. and affiliates.
// This source code is licensed under the MIT license found in the LICENSE
// file in the root directory of this source tree.
//! Translated from pinned Yoga tests/YGMeasureCacheTest.cpp (all six cases).
//! Exercise actual trees and retain upstream callback-count assertions.

use crate::prelude::*;

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
