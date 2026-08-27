//! Direct owner for pinned `src/constraints/constraint.cpp`.

use super::*;

/// Direct translation of pinned `rive::getParentWorld`.
///
/// C++ returns a reference to a module-static identity matrix when the direct
/// parent is not a `WorldTransformComponent`. `Mat2D` is copy-sized in Rust,
/// so the same branch returns the identity value instead of a shared
/// reference.
pub(crate) fn parent_world_transform(
    artboard: &ArtboardInstance,
    component_index: ComponentHandle,
) -> Mat2D {
    let Some(parent) = artboard.component_parent_handle(component_index) else {
        return Mat2D::IDENTITY;
    };
    Some(artboard.component_at(parent))
        .filter(|parent| parent.capabilities.world_transform)
        .map(|parent| parent.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY)
}

pub(crate) fn constraint_double_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    (keys.strength == property_key && kind != RuntimeConstraintKind::Ik)
        || (kind == RuntimeConstraintKind::Distance
            && distance_constraint::double_property_changed(property_key))
        || (matches!(
            kind,
            RuntimeConstraintKind::FollowPath | RuntimeConstraintKind::ListFollowPath
        ) && matches!(
            property_key,
            FOLLOW_PATH_DISTANCE_PROPERTY_KEY
                | LIST_FOLLOW_PATH_DISTANCE_END_PROPERTY_KEY
                | LIST_FOLLOW_PATH_DISTANCE_OFFSET_PROPERTY_KEY
        ))
        || (kind == RuntimeConstraintKind::Transform
            && transform_constraint::double_property_changed(property_key))
}

pub(crate) fn constraint_is_ik_strength_property(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    kind == RuntimeConstraintKind::Ik && property_key == RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength
}

pub(crate) fn constraint_uint_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    kind == RuntimeConstraintKind::Distance
        && distance_constraint::uint_property_changed(property_key)
}

pub(crate) fn apply_constraints(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
) -> bool {
    let mut changed = false;
    let constraint_count = artboard.objects.constraint_len(component_index);
    for index in 0..constraint_count {
        let Some(constraint) = artboard.objects.constraint_at(component_index, index) else {
            continue;
        };
        if artboard
            .objects
            .component(component_index)
            .is_some_and(|component| component.concrete.constrainable_list.is_some())
            && artboard
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.constraint)
                .is_some_and(|state| list_constraint::from(state.kind))
        {
            continue;
        }
        changed |= apply_constraint(artboard, component_index, constraint);
    }
    changed
}

fn apply_constraint(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    let Some(state) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
    else {
        return false;
    };
    match state.kind {
        RuntimeConstraintKind::Distance => {
            distance_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Translation => {
            translation_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Rotation => {
            rotation_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Scale => {
            scale_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::Transform => {
            transform_constraint::apply(artboard, component_index, constraint, state)
        }
        RuntimeConstraintKind::FollowPath | RuntimeConstraintKind::ListFollowPath => {
            follow_path_constraint::apply(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::Scroll => {
            scrolling::scroll_constraint::apply(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::ScrollBar => {
            scrolling::scroll_bar_constraint::apply(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::Ik => ik_constraint::apply(artboard, component_index, constraint),
        _ => false,
    }
}
