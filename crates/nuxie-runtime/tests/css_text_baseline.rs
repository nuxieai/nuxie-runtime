use taffy::prelude::*;

#[test]
fn host_baselines_are_used_cached_invalidated_and_optional() {
    let mut tree = TaffyTree::<f32>::new();
    tree.disable_rounding();
    let small = tree
        .new_leaf_with_context(
            Style {
                size: Size {
                    width: Dimension::length(40.0),
                    height: Dimension::length(30.0),
                },
                ..Default::default()
            },
            20.0,
        )
        .unwrap();
    let large = tree
        .new_leaf_with_context(
            Style {
                size: Size {
                    width: Dimension::length(40.0),
                    height: Dimension::length(40.0),
                },
                ..Default::default()
            },
            32.0,
        )
        .unwrap();
    let root = tree
        .new_with_children(
            Style {
                align_items: Some(AlignItems::BASELINE),
                ..Default::default()
            },
            &[small, large],
        )
        .unwrap();
    let baseline: fn(NodeId, Option<&mut f32>, &Style) -> Option<f32> =
        |_, context, _| context.map(|value| *value);
    tree.compute_layout_with_measure_and_baseline(
        root,
        Size::MAX_CONTENT,
        |_, _, _, _, _| Size::ZERO,
        Some(baseline),
    )
    .unwrap();
    assert_eq!(tree.layout(small).unwrap().location.y, 12.0);
    assert_eq!(tree.layout(large).unwrap().location.y, 0.0);
    tree.set_node_context(small, Some(15.0)).unwrap();
    tree.compute_layout_with_measure_and_baseline(
        root,
        Size::MAX_CONTENT,
        |_, _, _, _, _| Size::ZERO,
        Some(baseline),
    )
    .unwrap();
    assert_eq!(tree.layout(small).unwrap().location.y, 17.0);
    tree.mark_dirty(small).unwrap();
    tree.mark_dirty(large).unwrap();
    tree.compute_layout_with_measure(root, Size::MAX_CONTENT, |_, _, _, _, _| Size::ZERO)
        .unwrap();
    assert_eq!(tree.layout(small).unwrap().location.y, 10.0);
}
