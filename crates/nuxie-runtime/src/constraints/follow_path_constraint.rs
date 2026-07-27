//! Direct Rust home for pinned C++
//! `include/rive/constraints/follow_path_constraint.hpp` and
//! `src/constraints/follow_path_constraint.cpp`.
//!
//! Artboard retains authored-order lifecycle orchestration. This module owns
//! the concrete FollowPath occurrence, generated callbacks, target path flags,
//! dependency construction, retained path measure, and constraint arithmetic.

use anyhow::{Context, Result};
use nuxie_render_api::{Mat2D as RenderMat2D, RawPath};

use crate::components::{ComponentHandle, RuntimeConstraintKind, TransformComponents};
use crate::draw::RuntimePathMeasure;
use crate::objects::InstanceObjectArena;
use crate::shapes::path::RuntimePathState;
use crate::shapes::shape::RuntimeShapeState;
use crate::{ArtboardInstance, Mat2D};

use super::{
    RUNTIME_CONSTRAINT_PROPERTY_KEYS, TransformSpace, invert, parent_world_transform,
    retained_constraint_bool, retained_constraint_double, retained_constraint_space,
    write_world_transform,
};

pub(crate) const FOLLOW_PATH_DISTANCE_PROPERTY_KEY: u16 = 363;
pub(crate) const FOLLOW_PATH_ORIENT_PROPERTY_KEY: u16 = 364;
pub(crate) const FOLLOW_PATH_OFFSET_PROPERTY_KEY: u16 = 365;

/// Occurrence-local members owned by one C++ `FollowPathConstraint`.
///
/// `raw_path` retains its allocations across `rewind`/rebuild just like
/// `FollowPathConstraint::m_rawPath`. `path_measure` is rebuilt from that
/// retained path only during this owner's dependency-ordered `update`
/// (`src/constraints/follow_path_constraint.cpp:122-147`).
#[derive(Debug, Clone)]
pub(crate) struct RuntimeFollowPathState {
    pub(crate) raw_path: RawPath,
    pub(crate) path_measure: RuntimePathMeasure,
    #[cfg(test)]
    pub(crate) measure_rebuilds: usize,
}

impl RuntimeFollowPathState {
    pub(crate) fn new() -> Self {
        Self {
            raw_path: RawPath::new(),
            path_measure: RuntimePathMeasure::from_raw_path(&RawPath::new()),
            #[cfg(test)]
            measure_rebuilds: 0,
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::new()
    }
}

impl Default for RuntimeFollowPathState {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) fn bool_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    matches!(
        kind,
        RuntimeConstraintKind::FollowPath | RuntimeConstraintKind::ListFollowPath
    ) && property_key == FOLLOW_PATH_ORIENT_PROPERTY_KEY
}

pub(crate) fn double_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    matches!(
        kind,
        RuntimeConstraintKind::FollowPath | RuntimeConstraintKind::ListFollowPath
    ) && property_key == FOLLOW_PATH_DISTANCE_PROPERTY_KEY
}

/// `FollowPathConstraint::onAddedClean` marks a Shape or Path target before
/// the first dependency update (`src/constraints/follow_path_constraint.cpp:
/// 149-165`).
pub(crate) fn mark_target_path_flags(
    objects: &mut InstanceObjectArena,
    constraint: ComponentHandle,
) {
    let Some(target) = objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target())
    else {
        return;
    };
    if let Some(shape) = objects
        .component_mut(target)
        .and_then(|component| component.concrete.shape.as_mut())
    {
        shape.add_flags(RuntimeShapeState::FOLLOW_PATH);
    } else if let Some(path) = objects
        .component_mut(target)
        .and_then(|component| component.concrete.path.as_mut())
    {
        path.add_flags(RuntimePathState::FOLLOW_PATH);
    }
}

/// Concrete `FollowPathConstraint::buildDependencies` replaces the inherited
/// TargetedConstraint edge: path-composer/path before constraint, then
/// constraint before its constrained parent
/// (`src/constraints/follow_path_constraint.cpp:167-190`).
pub(crate) fn build_dependencies(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> Result<()> {
    let target = objects
        .component(handle)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target())
        .context("FollowPathConstraint is missing its retained target")?;
    let source = objects
        .component(target)
        .and_then(|component| component.concrete.shape.as_ref())
        .and_then(|_| {
            objects
                .component_local_id(target)
                .and_then(|local| objects.path_composer_handle(local))
        })
        .or_else(|| {
            objects
                .component(target)
                .and_then(|component| component.concrete.path.as_ref())
                .and_then(|path| path.shape)
                .and_then(|shape| objects.component_local_id(shape))
                .and_then(|shape_local| objects.path_composer_handle(shape_local))
                .or_else(|| {
                    objects
                        .component(target)
                        .and_then(|component| component.concrete.path.as_ref())
                        .map(|_| target)
                })
        });
    if let Some(source) = source {
        objects.add_dependent(source, handle);
    }
    if let Some(parent) = objects
        .component(handle)
        .and_then(|component| component.parent)
    {
        objects.add_dependent(handle, parent);
    }
    Ok(())
}

pub(crate) fn update(artboard: &mut ArtboardInstance, constraint: ComponentHandle) -> bool {
    let Some(target) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target())
    else {
        return false;
    };
    let path_handles = artboard
        .objects
        .component(target)
        .and_then(|component| component.concrete.shape.as_ref())
        .map(|shape| shape.paths.clone())
        .or_else(|| {
            artboard
                .objects
                .component(target)
                .and_then(|component| component.concrete.path.as_ref())
                .map(|_| vec![target])
        })
        .unwrap_or_default();

    // C++ preserves the previous RawPath/PathMeasure when a Shape currently
    // has no paths (`follow_path_constraint.cpp:122-147`).
    if path_handles.is_empty() {
        return false;
    }

    // C++ materializes only a local vector of retained Path pointers, then
    // rewinds and appends their RawPaths directly into the constraint owner.
    // Arc clones are the Rust pointer references; geometry is never lowered
    // into a temporary command buffer (`follow_path_constraint.cpp:122-145`).
    let mut sources = Vec::with_capacity(path_handles.len());
    for path_handle in path_handles {
        let Some(path_local) = artboard.objects.component_local_id(path_handle) else {
            continue;
        };
        let Some((raw_path, has_weighted_context)) =
            crate::shapes::path::retained_follow_path_source(&artboard.runtime_shapes, path_local)
        else {
            continue;
        };
        let transform = if has_weighted_context {
            Mat2D::IDENTITY
        } else {
            artboard.component_at(path_handle).transform.world_transform
        };
        sources.push((raw_path, transform));
    }
    let retained = artboard
        .objects
        .component_mut(constraint)
        .and_then(|component| component.concrete.follow_path.as_mut())
        .expect("FollowPathConstraint update requires its concrete owner");
    let verb_count = sources.iter().map(|(path, _)| path.verbs().len()).sum();
    let point_count = sources.iter().map(|(path, _)| path.points().len()).sum();
    retained.raw_path.rewind();
    retained.raw_path.reserve(verb_count, point_count);
    for (source, transform) in &sources {
        retained.raw_path.add_path(source, RenderMat2D(transform.0));
    }
    retained.path_measure = RuntimePathMeasure::from_raw_path(&retained.raw_path);
    #[cfg(test)]
    {
        retained.measure_rebuilds += 1;
    }
    true
}

pub(super) fn apply(
    artboard: &mut ArtboardInstance,
    component_index: ComponentHandle,
    constraint: ComponentHandle,
) -> bool {
    let Some(target) = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.constraint)
        .and_then(|constraint| constraint.target())
    else {
        return false;
    };
    if artboard.component_at(target).is_collapsed() {
        return false;
    }
    let constraint_local = artboard.component_at(constraint).local_id;
    let distance = retained_constraint_double(
        artboard,
        constraint_local,
        FOLLOW_PATH_DISTANCE_PROPERTY_KEY,
        0.0,
    );
    let transform_b =
        target_transform_at_distance(artboard, constraint, target, component_index, distance);
    let components = constrain_components(
        artboard,
        constraint_local,
        target,
        artboard
            .component_at(component_index)
            .transform
            .world_transform,
        transform_b,
        parent_world_transform(artboard, component_index),
    );
    write_world_transform(artboard, component_index, Mat2D::compose(components))
}

pub(super) fn target_transform_at_distance(
    artboard: &ArtboardInstance,
    constraint: ComponentHandle,
    target: ComponentHandle,
    offset_component: ComponentHandle,
    distance: f32,
) -> Mat2D {
    let constraint_local = artboard.component_at(constraint).local_id;
    let target_component = artboard.component_at(target);
    if target_component.concrete.shape.is_none() && target_component.concrete.path.is_none() {
        return target_component.transform.world_transform;
    }

    let sample = artboard
        .objects
        .component(constraint)
        .and_then(|component| component.concrete.follow_path.as_ref())
        .expect("FollowPathConstraint targetTransform requires retained measure")
        .path_measure
        .at_percentage(distance);
    let mut transform_b = target_component.transform.world_transform;

    if retained_constraint_bool(
        artboard,
        constraint_local,
        FOLLOW_PATH_ORIENT_PROPERTY_KEY,
        true,
    ) {
        let components_b = transform_b.decompose();
        let tangent_rotation = sample.tan.1.atan2(sample.tan.0);
        let two_pi = std::f32::consts::PI * 2.0;
        let angle_b = components_b.rotation % two_pi;
        let mut diff = tangent_rotation - angle_b;
        if diff > std::f32::consts::PI {
            diff -= two_pi;
        } else if diff < -std::f32::consts::PI {
            diff += two_pi;
        }
        transform_b = Mat2D::from_rotation(
            angle_b
                + diff
                    * retained_constraint_double(
                        artboard,
                        constraint_local,
                        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
                        1.0,
                    ),
        );
    }
    let offset_position = if retained_constraint_bool(
        artboard,
        constraint_local,
        FOLLOW_PATH_OFFSET_PROPERTY_KEY,
        false,
    ) {
        let local = artboard
            .component_at(offset_component)
            .transform
            .local_transform
            .0;
        (local[4], local[5])
    } else {
        (0.0, 0.0)
    };
    transform_b.0[4] = sample.pos.0 + offset_position.0;
    transform_b.0[5] = sample.pos.1 + offset_position.1;
    transform_b
}

pub(super) fn constrain_components(
    artboard: &ArtboardInstance,
    constraint_local: usize,
    target_index: ComponentHandle,
    component_transform: Mat2D,
    mut transform_b: Mat2D,
    component_parent_world: Mat2D,
) -> TransformComponents {
    if retained_constraint_space(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.source_space,
    ) == TransformSpace::Local
    {
        let target_parent_world = parent_world_transform(artboard, target_index);
        let Some(inverse) = invert(target_parent_world) else {
            return TransformComponents::default();
        };
        transform_b = inverse.multiply(transform_b);
    }
    if retained_constraint_space(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.dest_space,
    ) == TransformSpace::Local
    {
        transform_b = component_parent_world.multiply(transform_b);
    }

    let components_a = component_transform.decompose();
    let mut components_b = transform_b.decompose();
    let t = retained_constraint_double(
        artboard,
        constraint_local,
        RUNTIME_CONSTRAINT_PROPERTY_KEYS.strength,
        1.0,
    );
    let ti = 1.0 - t;

    if !retained_constraint_bool(
        artboard,
        constraint_local,
        FOLLOW_PATH_ORIENT_PROPERTY_KEY,
        true,
    ) {
        components_b.rotation = components_a.rotation % (std::f32::consts::PI * 2.0);
    }
    components_b.x = components_a.x * ti + components_b.x * t;
    components_b.y = components_a.y * ti + components_b.y * t;
    components_b.scale_x = components_a.scale_x;
    components_b.scale_y = components_a.scale_y;
    components_b.skew = components_a.skew;
    components_b
}
