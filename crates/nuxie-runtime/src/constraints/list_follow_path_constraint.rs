//! Direct owner for pinned `src/constraints/list_follow_path_constraint.cpp`.

use super::*;

pub(super) fn apply_to_transforms(
    artboard: &ArtboardInstance,
    list_component_index: ComponentHandle,
    constraint: ComponentHandle,
    item_transforms: &mut [Mat2D],
) -> bool {
    // Ported from C++ `src/constraints/list_follow_path_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let target = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target);
    let count = item_transforms.len();
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
    let start_offset = distance_offset + distance;
    let start_to_end_distance = distance_end - distance;
    let offset_distance = if count <= 1 {
        0.0
    } else {
        start_to_end_distance / (count as f32 - 1.0)
    };
    let list_transform = artboard
        .component_at(list_component_index)
        .transform
        .world_transform;
    let mut changed = false;

    for (index, transform) in item_transforms.iter_mut().enumerate() {
        let components = if let Some(target) =
            target.filter(|target| !artboard.component_at(*target).is_collapsed())
        {
            let transform_b = target_transform_for_follow_path_constraint_at_distance(
                artboard,
                constraint,
                target,
                list_component_index,
                start_offset + index as f32 * offset_distance,
            );
            follow_path_constrain_components(
                artboard,
                constraint_local,
                target,
                *transform,
                transform_b,
                list_transform,
            )
        } else {
            TransformComponents::default()
        };
        let next = Mat2D::compose(components);
        if *transform != next {
            *transform = next;
            changed = true;
        }
    }

    changed
}
