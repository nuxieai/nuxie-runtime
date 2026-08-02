//! Direct owner for pinned `src/constraints/distance_constraint.cpp`.

use super::*;

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
    state: crate::components::RuntimeConstraintState,
) -> bool {
    // Ported from C++ `src/constraints/distance_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let keys = RUNTIME_CONSTRAINT_PROPERTY_KEYS;
    let Some(target_index) = state.target else {
        return false;
    };
    if artboard.component_at(target_index).is_collapsed() {
        return false;
    }

    let target_transform = artboard
        .component_at(target_index)
        .transform
        .world_transform;
    let target_x = target_transform.0[4];
    let target_y = target_transform.0[5];

    let world = artboard
        .component_at(component_index)
        .transform
        .world_transform;
    let our_x = world.0[4];
    let our_y = world.0[5];
    let to_target_x = our_x - target_x;
    let to_target_y = our_y - target_y;
    // C++ Vec2D::length is literal sqrt(x*x + y*y), not hypot; preserve the
    // operation order because overflow/rounding are observable.
    let current_distance = (to_target_x * to_target_x + to_target_y * to_target_y).sqrt();
    let distance = retained_constraint_double(artboard, constraint_local, keys.distance, 100.0);

    match retained_constraint_uint(artboard, constraint_local, keys.mode, 0) {
        0 if current_distance < distance => return false,
        1 if current_distance > distance => return false,
        _ => {}
    }
    if current_distance < 0.001 {
        return false;
    }

    let scale = distance / current_distance;
    let constrained_x = target_x + to_target_x * scale;
    let constrained_y = target_y + to_target_y * scale;
    let strength = retained_constraint_double(artboard, constraint_local, keys.strength, 1.0);
    let new_x = our_x + (constrained_x - our_x) * strength;
    let new_y = our_y + (constrained_y - our_y) * strength;

    let world = &mut artboard
        .component_at_mut(component_index)
        .transform
        .world_transform
        .0;
    if world[4] == new_x && world[5] == new_y {
        return false;
    }
    world[4] = new_x;
    world[5] = new_y;
    true
}
