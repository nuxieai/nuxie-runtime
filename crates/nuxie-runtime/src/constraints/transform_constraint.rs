//! Direct owner for pinned `src/constraints/transform_constraint.cpp`.

use super::*;

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
    state: RuntimeConstraintState,
) -> bool {
    // Ported from C++ `src/constraints/transform_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    let Some(target_index) = state.target else {
        return false;
    };
    if artboard.component_at(target_index).is_collapsed() {
        return false;
    }

    let transform_a = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let mut transform_b = target_transform_for_transform_constraint(
        artboard,
        target_index,
        retained_constraint_double(artboard, constraint_local, keys.origin_x, 0.0),
        retained_constraint_double(artboard, constraint_local, keys.origin_y, 0.0),
    );
    if retained_constraint_space(artboard, constraint_local, keys.source_space)
        == TransformSpace::Local
    {
        let Some(inverse) = invert(parent_world_transform(artboard, target_index)) else {
            return false;
        };
        transform_b = inverse.multiply(transform_b);
    }
    if retained_constraint_space(artboard, constraint_local, keys.dest_space)
        == TransformSpace::Local
    {
        transform_b = parent_world_transform(artboard, component_index).multiply(transform_b);
    }

    let (constrained, components_a, components_b) = constrained_world_transform(
        transform_a,
        transform_b,
        retained_constraint_double(artboard, constraint_local, keys.strength, 1.0),
    );
    retain_transform_constraint_components(artboard, constraint, components_a, components_b);
    write_world_transform(artboard, component_index, constrained)
}
