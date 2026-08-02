//! Direct owner for pinned `src/constraints/rotation_constraint.cpp`.

use super::*;

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
    state: crate::components::RuntimeConstraintState,
) -> bool {
    // Ported from C++ `src/constraints/rotation_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    let target_index = state.target;
    if target_index.is_some_and(|index| artboard.component_at(index).is_collapsed()) {
        return false;
    }

    let transform_a = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let components_a = transform_a.decompose();
    let mut components_b = components_a;
    retain_constraint_component_a(artboard, constraint, components_a);

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

        components_b = transform_b.decompose();
        retain_constraint_component_b(artboard, constraint, components_b);
        let dest_space = retained_constraint_space(artboard, constraint_local, keys.dest_space);
        if !retained_constraint_bool(artboard, constraint_local, keys.does_copy, true) {
            components_b.rotation = if dest_space == TransformSpace::Local {
                0.0
            } else {
                components_a.rotation
            };
        } else {
            components_b.rotation *=
                retained_constraint_double(artboard, constraint_local, keys.copy_factor, 1.0);
            if retained_constraint_bool(artboard, constraint_local, keys.offset, false) {
                components_b.rotation += retained_authored_transform_property(
                    artboard,
                    component_index,
                    TransformProperty::Rotation,
                );
            }
        }

        if dest_space == TransformSpace::Local {
            transform_b = parent_world_transform(artboard, component_index)
                .multiply(Mat2D::compose(components_b));
            components_b = transform_b.decompose();
            retain_constraint_component_b(artboard, constraint, components_b);
        }
    } else {
        retain_constraint_component_b(artboard, constraint, components_b);
    }

    // RotationConstraint mutates its retained B scratch through copy/offset
    // before a possible singular clamp-space early return.
    retain_constraint_component_b(artboard, constraint, components_b);
    let clamp_local = retained_constraint_space(artboard, constraint_local, keys.min_max_space)
        == TransformSpace::Local;
    if clamp_local {
        let transform_b = Mat2D::compose(components_b);
        let Some(inverse) = invert(parent_world_transform(artboard, component_index)) else {
            return false;
        };
        components_b = inverse.multiply(transform_b).decompose();
        retain_constraint_component_b(artboard, constraint, components_b);
    }
    if retained_constraint_bool(artboard, constraint_local, keys.max, false)
        && components_b.rotation
            > retained_constraint_double(artboard, constraint_local, keys.max_value, 0.0)
    {
        components_b.rotation =
            retained_constraint_double(artboard, constraint_local, keys.max_value, 0.0);
    }
    if retained_constraint_bool(artboard, constraint_local, keys.min, false)
        && components_b.rotation
            < retained_constraint_double(artboard, constraint_local, keys.min_value, 0.0)
    {
        components_b.rotation =
            retained_constraint_double(artboard, constraint_local, keys.min_value, 0.0);
    }
    if clamp_local {
        let transform_b = parent_world_transform(artboard, component_index)
            .multiply(Mat2D::compose(components_b));
        components_b = transform_b.decompose();
        retain_constraint_component_b(artboard, constraint, components_b);
    }

    components_b.rotation = interpolated_rotation(
        components_a.rotation,
        components_b.rotation,
        retained_constraint_double(artboard, constraint_local, keys.strength, 1.0),
    );
    components_b.x = components_a.x;
    components_b.y = components_a.y;
    components_b.scale_x = components_a.scale_x;
    components_b.scale_y = components_a.scale_y;
    components_b.skew = components_a.skew;

    retain_constraint_component_b(artboard, constraint, components_b);
    write_world_transform(artboard, component_index, Mat2D::compose(components_b))
}
