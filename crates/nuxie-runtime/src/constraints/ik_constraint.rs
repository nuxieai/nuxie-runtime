//! Direct Rust home for `include/rive/constraints/ik_constraint.hpp` and
//! `src/constraints/ik_constraint.cpp`.
//!
//! Artboard retains authored-order lifecycle orchestration; this module owns
//! the concrete IK occurrence, dependencies, property callbacks, and dirt
//! propagation.

use anyhow::{Context, Result};

use crate::ArtboardInstance;
use crate::Mat2D;
use crate::bones::bone;
use crate::components::{ComponentHandle, RuntimeConstraintKind, TransformComponents};
use crate::objects::InstanceObjectArena;

use super::{
    RUNTIME_CONSTRAINT_PROPERTY_KEYS, parent_world_transform, retained_constraint_bool,
    retained_constraint_double, write_world_transform,
};

pub(crate) const IK_INVERT_DIRECTION_PROPERTY_KEY: u16 = 174;
pub(crate) const IK_PARENT_BONE_COUNT_PROPERTY_KEY: u16 = 175;

/// One retained C++ `IKConstraint::BoneChainLink`.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeIkChainLink {
    pub(crate) index: usize,
    pub(crate) bone: ComponentHandle,
    pub(crate) angle: f32,
    pub(crate) transform_components: TransformComponents,
    pub(crate) parent_world_inverse: Mat2D,
}

/// Occurrence-local members owned by one C++ `IKConstraint`.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeIkState {
    pub(crate) chain: Vec<RuntimeIkChainLink>,
    #[cfg(test)]
    pub(crate) chain_builds: usize,
}

impl RuntimeIkState {
    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::default()
    }
}

/// `IKConstraint::onAddedClean` builds one occurrence-owned FK chain
/// root-to-tip after walking Bone parents tip-to-root, then registers the
/// direct off-chain Transform children on each ancestor
/// (`src/constraints/ik_constraint.cpp:23-74`).
pub(crate) fn on_added_clean(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
    local_id: usize,
) -> Result<()> {
    let tip = objects
        .component(handle)
        .and_then(|component| component.parent)
        .context("IKConstraint is missing its constrained parent")?;
    if objects
        .component(tip)
        .and_then(|component| component.concrete.bone.as_ref())
        .is_none()
    {
        anyhow::bail!("IKConstraint parent is not a Bone");
    }

    let mut reverse_chain = vec![tip];
    let mut current_bone = tip;
    let mut remaining = objects
        .uint_property(local_id, IK_PARENT_BONE_COUNT_PROPERTY_KEY)
        .unwrap_or(0);
    while remaining > 0 {
        let Some(parent) = objects
            .component(current_bone)
            .and_then(|component| component.parent)
        else {
            break;
        };
        if objects
            .component(parent)
            .and_then(|component| component.concrete.bone.as_ref())
            .is_none()
        {
            break;
        }
        remaining -= 1;
        current_bone = parent;
        bone::add_peer_constraint(objects, current_bone, handle);
        reverse_chain.push(current_bone);
    }

    let chain = reverse_chain
        .iter()
        .rev()
        .copied()
        .enumerate()
        .map(|(index, bone)| RuntimeIkChainLink {
            index,
            bone,
            angle: 0.0,
            transform_components: TransformComponents::default(),
            parent_world_inverse: Mat2D::IDENTITY,
        })
        .collect();
    let retained = objects
        .component_mut(handle)
        .expect("IKConstraint handle was validated")
        .concrete
        .ik
        .as_mut()
        .expect("IKConstraint owns retained chain state");
    retained.chain = chain;
    #[cfg(test)]
    {
        retained.chain_builds += 1;
    }

    for index in 1..reverse_chain.len() {
        let ancestor = reverse_chain[index];
        let chain_child = reverse_chain[index - 1];
        let children = objects
            .component(ancestor)
            .map(|component| component.children.clone())
            .unwrap_or_default();
        for child in children {
            if child != chain_child
                && objects
                    .component(child)
                    .is_some_and(|component| component.capabilities.transform)
            {
                objects.add_dependent(tip, child);
            }
        }
    }
    Ok(())
}

/// `IKConstraint::buildDependencies` runs after TargetedConstraint's base
/// target-to-tip edge and adds target-to-constraint second
/// (`src/constraints/ik_constraint.cpp:10-21`).
pub(crate) fn build_dependencies(objects: &mut InstanceObjectArena, handle: ComponentHandle) {
    if let Some(target) = objects
        .component(handle)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target())
    {
        objects.add_dependent(target, handle);
    }
}

pub(crate) fn constraint_is_ik_strength_property(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    kind == RuntimeConstraintKind::Ik
        && property_key == super::RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength
}

pub(crate) fn apply_bool_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    kind: Option<RuntimeConstraintKind>,
    property_key: u16,
) -> Option<bool> {
    (property_key == IK_INVERT_DIRECTION_PROPERTY_KEY && kind == Some(RuntimeConstraintKind::Ik))
        .then(|| instance.mark_ik_constraint_dirty(local_id))
}

pub(crate) fn apply_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    kind: Option<RuntimeConstraintKind>,
    property_key: u16,
) -> Option<bool> {
    kind.filter(|kind| constraint_is_ik_strength_property(*kind, property_key))
        .map(|_| instance.mark_ik_constraint_dirty(local_id))
}

fn write_local_transform(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    transform: Mat2D,
) -> bool {
    let local = &mut artboard
        .component_at_mut(component_index)
        .transform
        .local_transform
        .0;
    if *local == transform.0 {
        return false;
    }
    *local = transform.0;
    true
}

fn write_local_world_transform(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    local_transform: Mat2D,
    world_transform: Mat2D,
) -> bool {
    let local_changed = write_local_transform(artboard, component_index, local_transform);
    let world_changed = write_world_transform(artboard, component_index, world_transform);
    local_changed || world_changed
}

fn world_translation(transform: Mat2D) -> (f32, f32) {
    (transform.0[4], transform.0[5])
}

fn tip_world_translation(artboard: &ArtboardInstance, bone_index: ComponentHandle) -> (f32, f32) {
    let bone = artboard.component_at(bone_index);
    let length = artboard.bone_length(bone.local_id).unwrap_or(0.0);
    bone.transform.world_transform.transform_point(length, 0.0)
}

fn point_sub(left: (f32, f32), right: (f32, f32)) -> (f32, f32) {
    (left.0 - right.0, left.1 - right.1)
}

fn point_length(point: (f32, f32)) -> f32 {
    (point.0 * point.0 + point.1 * point.1).sqrt()
}

fn point_atan2(point: (f32, f32)) -> f32 {
    point.1.atan2(point.0)
}

/// Literal `IKConstraint::constrain` owner translation
/// (`src/constraints/ik_constraint.cpp:88-185`).
pub(super) fn apply(artboard: &mut ArtboardInstance, constraint: ComponentHandle) -> bool {
    let constraint_local = artboard.component_at(constraint).local_id;
    let Some(target_index) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target())
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

impl ArtboardInstance {
    /// `IKConstraint::markConstraintDirty` dirties the constrained tip and
    /// every root-to-pre-tip Bone retained by this occurrence
    /// (`src/constraints/ik_constraint.cpp:76-86`).
    pub(crate) fn mark_ik_constraint_dirty(&mut self, constraint_local_id: usize) -> bool {
        let Some(constraint) = self.component_handle(constraint_local_id) else {
            return false;
        };
        let Some(tip) = self
            .objects
            .component(constraint)
            .and_then(|constraint| constraint.parent)
        else {
            return false;
        };
        let mut changed = self.mark_transform_dirty_handle(tip);
        let chain_len = self
            .objects
            .component(constraint)
            .and_then(|component| component.concrete.ik.as_ref())
            .map_or(0, |ik| ik.chain.len());
        for index in 0..chain_len.saturating_sub(1) {
            let bone = self
                .objects
                .component(constraint)
                .and_then(|component| component.concrete.ik.as_ref())
                .and_then(|ik| ik.chain.get(index))
                .map(|link| link.bone);
            if let Some(bone) = bone {
                changed |= self.mark_transform_dirty_handle(bone);
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::point_length;

    #[test]
    fn ik_length_preserves_literal_cpp_operation_order() {
        let coordinate = f32::MAX / 2.0;
        assert!(point_length((coordinate, coordinate)).is_infinite());
        assert!(coordinate.hypot(coordinate).is_finite());
    }
}
