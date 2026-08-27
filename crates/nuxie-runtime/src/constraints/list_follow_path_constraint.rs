//! Direct owner for pinned `src/constraints/list_follow_path_constraint.cpp`.

use super::*;

/// Direct routing for
/// `ListFollowPathConstraint::{distanceEnd,distanceOffset}Changed()`.
///
/// Both pinned callbacks call `markConstraintDirty()` after the generated
/// setter has retained its value.
pub(super) fn double_property_changed(property_key: u16) -> bool {
    matches!(
        property_key,
        LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY | LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY
    )
}

pub(super) fn constrain_list(
    artboard: &ArtboardInstance,
    list_component_index: ComponentHandle,
    constraint: ComponentHandle,
    item_transforms: &mut [Mat2D],
) -> bool {
    let constraint_local = artboard.component_at(constraint).local_id;
    let list_transform = artboard
        .component_at(list_component_index)
        .transform
        .world_transform;
    let distance = retained_constraint_double(
        artboard,
        constraint_local,
        FOLLOW_PATH_DISTANCE_PROPERTY_KEY,
        0.0,
    );
    let distance_end = retained_constraint_double(
        artboard,
        constraint_local,
        LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY,
        1.0,
    );
    let distance_offset = retained_constraint_double(
        artboard,
        constraint_local,
        LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY,
        0.0,
    );
    let count = item_transforms.len();
    let start_offset = distance_offset + distance;
    let start_to_end_distance = distance_end - distance;
    let offset_distance = if count <= 1 {
        0.0
    } else {
        start_to_end_distance / (count as f32 - 1.0)
    };
    let mut changed = false;

    for (index, transform) in item_transforms.iter_mut().enumerate() {
        let components = constrain_at_offset(
            artboard,
            list_component_index,
            constraint,
            *transform,
            list_transform,
            start_offset + index as f32 * offset_distance,
        );
        let next = Mat2D::compose(components);
        changed |= transform
            .0
            .iter()
            .zip(next.0)
            .any(|(from, to)| from.to_bits() != to.to_bits());

        // Pinned C++ invokes all six Mat2D setters unconditionally and in
        // field order. Preserve that write even when float equality treats a
        // signed-zero bit change as equal.
        transform.0[0] = next.0[0];
        transform.0[1] = next.0[1];
        transform.0[2] = next.0[2];
        transform.0[3] = next.0[3];
        transform.0[4] = next.0[4];
        transform.0[5] = next.0[5];
    }

    changed
}

fn constrain_at_offset(
    artboard: &ArtboardInstance,
    list_component_index: ComponentHandle,
    constraint: ComponentHandle,
    component_transform: Mat2D,
    parent_transform: Mat2D,
    component_offset: f32,
) -> TransformComponents {
    let Some(target) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target)
        .filter(|target| !artboard.component_at(*target).is_collapsed())
    else {
        return TransformComponents::default();
    };
    let transform_b = target_transform_for_follow_path_constraint_at_distance(
        artboard,
        constraint,
        target,
        list_component_index,
        component_offset,
    );
    follow_path_constrain_components(
        artboard,
        artboard.component_at(constraint).local_id,
        target,
        component_transform,
        transform_b,
        parent_transform,
    )
}
