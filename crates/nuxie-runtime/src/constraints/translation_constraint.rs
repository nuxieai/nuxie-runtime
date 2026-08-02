//! Direct owner for pinned `src/constraints/translation_constraint.cpp`.

use super::*;

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
    state: RuntimeConstraintState,
) -> bool {
    // Ported from C++ `src/constraints/translation_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    let target_index = state.target;
    if target_index.is_some_and(|index| artboard.component_at(index).is_collapsed()) {
        return false;
    }

    let world = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let translation_a = (world.0[4], world.0[5]);
    let mut translation_b = translation_a;

    if let Some(target_index) = target_index {
        let mut transform_b = artboard
            .component_at(target_index)
            .transform
            .world_transform;
        if retained_constraint_space(artboard, constraint_local, keys.source_space)
            == TransformSpace::Local
        {
            let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
                return false;
            };
            transform_b = inverse.multiply(transform_b);
        }
        translation_b = (transform_b.0[4], transform_b.0[5]);

        let dest_space = retained_constraint_space(artboard, constraint_local, keys.dest_space);
        if !retained_constraint_bool(artboard, constraint_local, keys.does_copy, true) {
            translation_b.0 = if dest_space == TransformSpace::Local {
                0.0
            } else {
                translation_a.0
            };
        } else {
            translation_b.0 *=
                retained_constraint_double(artboard, constraint_local, keys.copy_factor, 1.0);
            if retained_constraint_bool(artboard, constraint_local, keys.offset, false) {
                translation_b.0 += retained_authored_transform_property(
                    artboard,
                    component_index,
                    TransformProperty::X,
                );
            }
        }

        if !retained_constraint_bool(artboard, constraint_local, keys.does_copy_y, true) {
            translation_b.1 = if dest_space == TransformSpace::Local {
                0.0
            } else {
                translation_a.1
            };
        } else {
            translation_b.1 *=
                retained_constraint_double(artboard, constraint_local, keys.copy_factor_y, 1.0);
            if retained_constraint_bool(artboard, constraint_local, keys.offset, false) {
                translation_b.1 += retained_authored_transform_property(
                    artboard,
                    component_index,
                    TransformProperty::Y,
                );
            }
        }

        if dest_space == TransformSpace::Local {
            translation_b = parent_world_transform(artboard, component_index)
                .transform_point(translation_b.0, translation_b.1);
        }
    }

    let clamp_local = retained_constraint_space(artboard, constraint_local, keys.min_max_space)
        == TransformSpace::Local;
    if clamp_local {
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        translation_b = inverse.transform_point(translation_b.0, translation_b.1);
    }
    if retained_constraint_bool(artboard, constraint_local, keys.max, false)
        && translation_b.0
            > retained_constraint_double(artboard, constraint_local, keys.max_value, 0.0)
    {
        translation_b.0 =
            retained_constraint_double(artboard, constraint_local, keys.max_value, 0.0);
    }
    if retained_constraint_bool(artboard, constraint_local, keys.min, false)
        && translation_b.0
            < retained_constraint_double(artboard, constraint_local, keys.min_value, 0.0)
    {
        translation_b.0 =
            retained_constraint_double(artboard, constraint_local, keys.min_value, 0.0);
    }
    if retained_constraint_bool(artboard, constraint_local, keys.max_y, false)
        && translation_b.1
            > retained_constraint_double(artboard, constraint_local, keys.max_value_y, 0.0)
    {
        translation_b.1 =
            retained_constraint_double(artboard, constraint_local, keys.max_value_y, 0.0);
    }
    if retained_constraint_bool(artboard, constraint_local, keys.min_y, false)
        && translation_b.1
            < retained_constraint_double(artboard, constraint_local, keys.min_value_y, 0.0)
    {
        translation_b.1 =
            retained_constraint_double(artboard, constraint_local, keys.min_value_y, 0.0);
    }
    if clamp_local {
        translation_b = parent_world_transform(artboard, component_index)
            .transform_point(translation_b.0, translation_b.1);
    }

    let t = retained_constraint_double(artboard, constraint_local, keys.strength, 1.0);
    let ti = 1.0 - t;
    let new_x = translation_a.0 * ti + translation_b.0 * t;
    let new_y = translation_a.1 * ti + translation_b.1 * t;

    let mut transform = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    transform.0[4] = new_x;
    transform.0[5] = new_y;
    write_world_transform(artboard, component_index, transform)
}
