//! Direct Rust home for pinned C++
//! `include/rive/constraints/list_follow_path_constraint.hpp` and
//! `src/constraints/list_follow_path_constraint.cpp`.
//!
//! This derived owner calls the FollowPath base dependency builder first,
//! registers itself once on the constrained list, and owns the list-item
//! spacing/constrain loop.

use crate::components::{ComponentHandle, RuntimeConstraintKind, TransformComponents};
use crate::objects::InstanceObjectArena;
use crate::{ArtboardInstance, Mat2D};

use super::follow_path_constraint::{
    self, FOLLOW_PATH_DISTANCE_PROPERTY_KEY, constrain_components, target_transform_at_distance,
};
use super::retained_constraint_double;

pub(crate) const LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY: u16 = 888;
pub(crate) const LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY: u16 = 889;

pub(crate) fn double_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    kind == RuntimeConstraintKind::ListFollowPath
        && matches!(
            property_key,
            LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY
                | LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY
        )
}

/// `ListFollowPathConstraint::buildDependencies` invokes the FollowPath base
/// first, then registers this exact occurrence once on its ConstrainableList
/// parent (`src/constraints/list_follow_path_constraint.cpp:57-66`).
pub(crate) fn build_dependencies(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> anyhow::Result<()> {
    follow_path_constraint::build_dependencies(objects, handle)?;
    let Some(list) = objects
        .component(handle)
        .and_then(|component| component.parent)
    else {
        return Ok(());
    };
    let Some(constraints) = objects
        .component_mut(list)
        .and_then(|component| component.concrete.constrainable_list.as_mut())
        .map(|list| &mut list.constraints)
    else {
        return Ok(());
    };
    assert!(
        !constraints.contains(&handle),
        "C++ ConstrainableList requires unique constraint registration"
    );
    constraints.push(handle);
    Ok(())
}

/// Literal `ListFollowPathConstraint::constrainList` owner translation
/// (`src/constraints/list_follow_path_constraint.cpp:17-41`).
pub(crate) fn constrain_list(
    artboard: &ArtboardInstance,
    list_component_index: ComponentHandle,
    constraint: ComponentHandle,
    item_transforms: &mut [Mat2D],
) -> bool {
    let constraint_local = artboard.component_at(constraint).local_id;
    let target = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target());
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
        let components = constrain_at_offset(
            artboard,
            constraint_local,
            constraint,
            target,
            list_component_index,
            *transform,
            list_transform,
            start_offset + index as f32 * offset_distance,
        );
        let next = Mat2D::compose(components);
        if *transform != next {
            *transform = next;
            changed = true;
        }
    }

    changed
}

/// Literal `ListFollowPathConstraint::constrainAtOffset` owner translation
/// (`src/constraints/list_follow_path_constraint.cpp:43-55`).
fn constrain_at_offset(
    artboard: &ArtboardInstance,
    constraint_local: usize,
    constraint: ComponentHandle,
    target: Option<ComponentHandle>,
    list_component_index: ComponentHandle,
    component_transform: Mat2D,
    list_transform: Mat2D,
    component_offset: f32,
) -> TransformComponents {
    let Some(target) = target.filter(|target| !artboard.component_at(*target).is_collapsed())
    else {
        return TransformComponents::default();
    };
    let transform_b = target_transform_at_distance(
        artboard,
        constraint,
        target,
        list_component_index,
        component_offset,
    );
    constrain_components(
        artboard,
        constraint_local,
        target,
        component_transform,
        transform_b,
        list_transform,
    )
}
