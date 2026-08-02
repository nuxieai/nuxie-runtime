//! Direct owner for pinned `src/constraints/constraint.cpp`.

use super::*;

pub(crate) fn constraint_double_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    (keys.strength == property_key && kind != RuntimeConstraintKind::Ik)
        || (kind == RuntimeConstraintKind::Distance && keys.distance == property_key)
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
            && (keys.origin_x == property_key || keys.origin_y == property_key))
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
    kind == RuntimeConstraintKind::Distance && RUNTIME_CONSTRAINT_PROPERTY_KEYS.mode == property_key
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
                .is_some_and(|state| state.kind == RuntimeConstraintKind::ListFollowPath)
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
            apply_scroll_bar_constraint(artboard, component_index, constraint)
        }
        RuntimeConstraintKind::Ik => ik_constraint::apply(artboard, component_index, constraint),
        _ => false,
    }
}
