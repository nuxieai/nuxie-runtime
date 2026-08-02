//! Direct owner for pinned `src/constraints/ik_constraint.cpp`.

use super::*;

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    _component_index: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    // Ported from C++ `src/constraints/ik_constraint.cpp`.
    let constraint_local = artboard.component_at(constraint).local_id;
    let Some(target_index) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target)
    else {
        return false;
    };
    if artboard.component_at(target_index).is_collapsed() {
        return false;
    }

    let invert_direction = retained_constraint_bool(
        artboard,
        constraint_local,
        IK_INVERT_DIRECTION_PROPERTY_KEY,
        false,
    );
    let world_target_translation = world_translation(
        artboard
            .component_at(target_index)
            .transform
            .world_transform,
    );
    let mut chain = std::mem::take(
        &mut artboard
            .objects
            .component_mut(constraint)
            .and_then(|component| component.concrete.ik.as_mut())
            .expect("IKConstraint apply requires its concrete owner")
            .chain,
    );
    let mut changed = false;
    for link in &mut chain {
        let bone_index = link.bone;
        let parent_world = parent_world_transform(artboard, bone_index);
        link.parent_world_inverse = parent_world.invert_or_identity();
        let bone_transform = link
            .parent_world_inverse
            .multiply(artboard.component_at(bone_index).transform.world_transform);
        changed |= write_local_transform(artboard, bone_index, bone_transform);
        link.transform_components = bone_transform.decompose();
    }

    match chain.len() {
        0 => {}
        1 => {
            changed |= solve_ik1(artboard, &mut chain, 0, world_target_translation);
        }
        2 => {
            changed |= solve_ik2(
                artboard,
                &mut chain,
                0,
                1,
                world_target_translation,
                invert_direction,
            );
        }
        count => {
            let tip_index = count - 1;
            for index in 0..tip_index {
                changed |= solve_ik2(
                    artboard,
                    &mut chain,
                    index,
                    tip_index,
                    world_target_translation,
                    invert_direction,
                );
                for child_index in (index + 1)..tip_index {
                    let bone_index = chain[child_index].bone;
                    chain[child_index].parent_world_inverse =
                        parent_world_transform(artboard, bone_index).invert_or_identity();
                }
            }
        }
    }

    let strength = retained_constraint_double(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
        1.0,
    );
    if strength != 1.0 {
        for index in 0..chain.len() {
            let from_angle =
                chain[index].transform_components.rotation % (std::f32::consts::PI * 2.0);
            let to_angle = chain[index].angle % (std::f32::consts::PI * 2.0);
            let mut diff = to_angle - from_angle;
            if diff > std::f32::consts::PI {
                diff -= std::f32::consts::PI * 2.0;
            } else if diff < -std::f32::consts::PI {
                diff += std::f32::consts::PI * 2.0;
            }
            changed |= constrain_ik_rotation(artboard, &chain[index], from_angle + diff * strength);
        }
    }

    artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.ik.as_mut())
        .expect("IKConstraint owner remained live during apply")
        .chain = chain;
    changed
}

fn solve_ik1(
    artboard: &mut ArtboardInstance,
    chain: &mut [RuntimeIkChainLink],
    index: usize,
    world_target_translation: (f32, f32),
) -> bool {
    let bone_index = chain[index].bone;
    let p_a = world_translation(artboard.component_at(bone_index).transform.world_transform);
    let to_target = (
        world_target_translation.0 - p_a.0,
        world_target_translation.1 - p_a.1,
    );
    let to_target_local = chain[index]
        .parent_world_inverse
        .transform_direction(to_target.0, to_target.1);
    let rotation = point_atan2(to_target_local);
    chain[index].angle = rotation;
    constrain_ik_rotation(artboard, &chain[index], rotation)
}

fn solve_ik2(
    artboard: &mut ArtboardInstance,
    chain: &mut [RuntimeIkChainLink],
    fk1_index: usize,
    fk2_index: usize,
    world_target_translation: (f32, f32),
    invert_direction: bool,
) -> bool {
    let first_child_index = chain[fk1_index].index + 1;
    let b1_index = chain[fk1_index].bone;
    let b2_index = chain[fk2_index].bone;
    let first_child_bone_index = chain[first_child_index].bone;
    let iworld = chain[fk1_index].parent_world_inverse;

    let mut p_a = world_translation(artboard.component_at(b1_index).transform.world_transform);
    let mut p_c = world_translation(
        artboard
            .component_at(first_child_bone_index)
            .transform
            .world_transform,
    );
    let mut p_b = tip_world_translation(artboard, b2_index);
    let mut p_bt = world_target_translation;

    p_a = iworld.transform_point(p_a.0, p_a.1);
    p_c = iworld.transform_point(p_c.0, p_c.1);
    p_b = iworld.transform_point(p_b.0, p_b.1);
    p_bt = iworld.transform_point(p_bt.0, p_bt.1);

    let av = point_sub(p_b, p_c);
    let bv = point_sub(p_c, p_a);
    let cv = point_sub(p_bt, p_a);
    let a = point_length(av);
    let b = point_length(bv);
    let c = point_length(cv);

    let angle_a = ((-a * a + b * b + c * c) / (2.0 * b * c))
        .clamp(-1.0, 1.0)
        .acos();
    let angle_c = ((a * a + b * b - c * c) / (2.0 * a * b))
        .clamp(-1.0, 1.0)
        .acos();

    let (r1, r2) = if artboard.component_parent_handle(b2_index) != Some(b1_index) {
        let second_child_index = fk1_index + 2;
        let second_child_world_inverse = chain[second_child_index].parent_world_inverse;
        let p_c_world = world_translation(
            artboard
                .component_at(first_child_bone_index)
                .transform
                .world_transform,
        );
        let p_b_world = tip_world_translation(artboard, b2_index);
        let av_local = second_child_world_inverse
            .transform_direction(p_b_world.0 - p_c_world.0, p_b_world.1 - p_c_world.1);
        let angle_correction = -point_atan2(av_local);
        if invert_direction {
            (
                point_atan2(cv) - angle_a,
                -angle_c + std::f32::consts::PI + angle_correction,
            )
        } else {
            (
                angle_a + point_atan2(cv),
                angle_c - std::f32::consts::PI + angle_correction,
            )
        }
    } else if invert_direction {
        (point_atan2(cv) - angle_a, -angle_c + std::f32::consts::PI)
    } else {
        (angle_a + point_atan2(cv), angle_c - std::f32::consts::PI)
    };

    let mut changed = false;
    changed |= constrain_ik_rotation(artboard, &chain[fk1_index], r1);
    changed |= constrain_ik_rotation(artboard, &chain[first_child_index], r2);
    if first_child_index != fk2_index {
        let bone_index = chain[fk2_index].bone;
        let parent_world = parent_world_transform(artboard, bone_index);
        let local = artboard.component_at(bone_index).transform.local_transform;
        changed |= write_world_transform(artboard, bone_index, parent_world.multiply(local));
    }

    chain[fk1_index].angle = r1;
    chain[first_child_index].angle = r2;
    changed
}

fn constrain_ik_rotation(
    artboard: &mut ArtboardInstance,
    state: &RuntimeIkChainLink,
    rotation: f32,
) -> bool {
    let bone_index = state.bone;
    let components = state.transform_components;
    let mut local_transform = Mat2D::from_rotation(rotation);
    local_transform.0[4] = components.x;
    local_transform.0[5] = components.y;
    local_transform.0[0] *= components.scale_x;
    local_transform.0[1] *= components.scale_x;
    local_transform.0[2] *= components.scale_y;
    local_transform.0[3] *= components.scale_y;
    if components.skew != 0.0 {
        local_transform.0[2] = local_transform.0[0] * components.skew + local_transform.0[2];
        local_transform.0[3] = local_transform.0[1] * components.skew + local_transform.0[3];
    }
    let parent_world = parent_world_transform(artboard, bone_index);
    write_local_world_transform(
        artboard,
        bone_index,
        local_transform,
        parent_world.multiply(local_transform),
    )
}
