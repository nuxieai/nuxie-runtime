//! Direct owner for pinned `src/constraints/scrolling/scroll_constraint.cpp`.

use super::super::*;

pub(in crate::constraints) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint_handle: ComponentHandle,
) -> bool {
    // `ScrollConstraint::constrain` / `constrainChild`.
    let constraint_local = artboard.component_at(constraint_handle).local_id;
    let Some(content) = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
        .and_then(|scroll| scroll.content)
    else {
        return false;
    };
    if component_index != content {
        return false;
    }
    let retained_layout_bounds = artboard.retained_layout_bounds_arc();
    let layout_bounds = retained_layout_bounds.as_deref();
    artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("ScrollConstraint occurrence retains its concrete state")
        .layout_initialized = true;
    let intent_changed = {
        let scroll_constraint = artboard
            .objects
            .component(constraint_handle)
            .and_then(|component| component.concrete.scroll.as_ref())
            .expect("ScrollConstraint occurrence retains its concrete state");
        if scroll_constraint.intent_x.is_some() || scroll_constraint.intent_y.is_some() {
            let include_item_bounds = scroll_constraint
                .intent_x
                .into_iter()
                .chain(scroll_constraint.intent_y)
                .any(|intent| intent.space == RuntimeScrollSpace::Index);
            let scroll_metrics = build_runtime_scroll_layout_metrics(
                artboard,
                constraint_handle,
                scroll_constraint,
                layout_bounds,
                include_item_bounds,
            );
            resolve_runtime_scroll_intents(artboard, constraint_local, &scroll_metrics)
        } else {
            false
        }
    };
    let scroll_constraint = artboard
        .objects
        .component(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_ref())
        .expect("ScrollConstraint occurrence retains its concrete state");
    let metrics = build_runtime_scroll_layout_metrics(
        artboard,
        constraint_handle,
        scroll_constraint,
        layout_bounds,
        false,
    );
    let (clamped_x, clamped_y) =
        clamped_scroll_constraint_offsets(artboard, constraint_handle, &metrics);
    let offset_x = if metrics.constrains_horizontal() {
        clamped_x
    } else {
        0.0
    };
    let offset_y = if metrics.constrains_vertical() {
        clamped_y
    } else {
        0.0
    };
    let scroll_transform = Mat2D([1.0, 0.0, 0.0, 1.0, offset_x, offset_y]);
    let scroll = artboard
        .objects
        .component_mut(constraint_handle)
        .and_then(|component| component.concrete.scroll.as_mut())
        .expect("ScrollConstraint occurrence retains its concrete state");
    scroll.scroll_transform = scroll_transform;
    scroll.child_constraint_applied_count = 0;
    intent_changed
}
