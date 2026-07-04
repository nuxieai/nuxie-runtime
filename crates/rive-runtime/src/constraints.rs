use crate::ArtboardInstance;
use crate::properties::property_key_for_name;

/// Runtime constraint application for the C++ `src/constraints/` path.
pub(crate) fn apply_constraints(artboard: &mut ArtboardInstance, component_index: usize) -> bool {
    let constraint_locals = artboard.components[component_index]
        .constraint_locals
        .clone();
    constraint_locals
        .into_iter()
        .fold(false, |changed, constraint_local| {
            changed | apply_distance_constraint(artboard, component_index, constraint_local)
        })
}

fn apply_distance_constraint(
    artboard: &mut ArtboardInstance,
    component_index: usize,
    constraint_local: usize,
) -> bool {
    // Ported from C++ `src/constraints/distance_constraint.cpp`.
    let Some(slot) = artboard.slot(constraint_local) else {
        return false;
    };
    if slot.type_name != Some("DistanceConstraint") {
        return false;
    }

    let Some(target_local) = targeted_constraint_target_local(artboard, constraint_local) else {
        return false;
    };
    if artboard
        .component(target_local)
        .is_some_and(|target| target.is_collapsed())
    {
        return false;
    }

    let Some(target_index) = artboard.component_by_local.get(&target_local).copied() else {
        return false;
    };
    let target_transform = artboard.components[target_index].transform.world_transform;
    let target_x = target_transform.0[4];
    let target_y = target_transform.0[5];

    let world = artboard.components[component_index]
        .transform
        .world_transform;
    let our_x = world.0[4];
    let our_y = world.0[5];
    let to_target_x = our_x - target_x;
    let to_target_y = our_y - target_y;
    let current_distance = to_target_x.hypot(to_target_y);
    let distance = constraint_double(
        artboard,
        constraint_local,
        "DistanceConstraint",
        "distance",
        100.0,
    );

    match constraint_uint(
        artboard,
        constraint_local,
        "DistanceConstraint",
        "modeValue",
        0,
    ) {
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
    let strength = constraint_double(artboard, constraint_local, "Constraint", "strength", 1.0);
    let new_x = our_x + (constrained_x - our_x) * strength;
    let new_y = our_y + (constrained_y - our_y) * strength;

    let world = &mut artboard.components[component_index]
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

fn targeted_constraint_target_local(
    artboard: &ArtboardInstance,
    constraint_local: usize,
) -> Option<usize> {
    let target_key = property_key_for_name("TargetedConstraint", "targetId")?;
    let target_global =
        u32::try_from(artboard.uint_property(constraint_local, target_key)?).ok()?;
    artboard
        .slots
        .iter()
        .find(|slot| slot.source_global_id == target_global)
        .map(|slot| slot.local_id)
}

fn constraint_double(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.double_property(local_id, key))
        .unwrap_or(default)
}

fn constraint_uint(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: u64,
) -> u64 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.uint_property(local_id, key))
        .unwrap_or(default)
}
