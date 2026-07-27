//! Direct Rust owner for pinned C++
//! `include/rive/constraints/targeted_constraint.hpp` and
//! `src/constraints/targeted_constraint.cpp`.
//!
//! Constraint construction remains authored-order Artboard orchestration;
//! this module owns the nullable target occurrence, required-target policy,
//! generated target key, validation/retention, and base dependency edge.

use anyhow::Context;

use crate::components::{ComponentHandle, RuntimeConstraintKind};
use crate::objects::InstanceObjectArena;

pub(crate) const TARGET_ID_PROPERTY_KEY: u16 = 173;

#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeTargetedConstraintState {
    target: Option<ComponentHandle>,
    requires_target: bool,
}

impl RuntimeTargetedConstraintState {
    pub(crate) const fn new(requires_target: bool) -> Self {
        Self {
            target: None,
            requires_target,
        }
    }

    pub(crate) const fn target(self) -> Option<ComponentHandle> {
        self.target
    }

    pub(crate) fn set_target(&mut self, target: Option<ComponentHandle>) {
        self.target = target;
    }

    pub(crate) const fn requires_target(self) -> bool {
        self.requires_target
    }

    pub(crate) const fn clone_for_occurrence(self) -> Self {
        Self::new(self.requires_target)
    }
}

/// `TargetedConstraint::onAddedDirty` resolves one live target after the
/// Constraint base has registered on its parent
/// (`src/constraints/targeted_constraint.cpp:23-39`).
pub(crate) fn on_added_dirty(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
    local_id: usize,
) -> anyhow::Result<()> {
    let (requires_target, is_targeted) = objects
        .component(handle)
        .and_then(|component| component.concrete.constraint)
        .map(|constraint| {
            (
                constraint
                    .targeted()
                    .is_some_and(RuntimeTargetedConstraintState::requires_target),
                constraint.targeted().is_some(),
            )
        })
        .context("Constraint occurrence is missing its concrete state")?;
    if !is_targeted {
        return Ok(());
    }

    let target_local = objects
        .uint_property(local_id, TARGET_ID_PROPERTY_KEY)
        .and_then(|target| usize::try_from(target).ok());
    let target = target_local.and_then(|local| objects.component_handle(local));
    if target_local.is_some_and(|local| objects.contains_object(local))
        && target.is_none_or(|target| {
            !objects
                .component(target)
                .is_some_and(|target| target.capabilities.transform)
        })
    {
        anyhow::bail!("TargetedConstraint targetId does not resolve to a TransformComponent");
    }
    if requires_target && target.is_none() {
        anyhow::bail!("TargetedConstraint is missing its required target");
    }
    objects
        .component_mut(handle)
        .expect("Constraint handle was validated")
        .concrete
        .constraint
        .as_mut()
        .expect("Constraint occurrence owns Constraint state")
        .set_target(target);
    Ok(())
}

/// Base `TargetedConstraint::buildDependencies`: target before constrained
/// parent. FollowPath subclasses replace this base implementation with their
/// concrete dependency builder (`src/constraints/targeted_constraint.cpp:
/// 41-49`; `src/constraints/follow_path_constraint.cpp:167-190`).
pub(crate) fn build_dependencies(objects: &mut InstanceObjectArena, handle: ComponentHandle) {
    let Some((kind, target)) = objects
        .component(handle)
        .and_then(|component| component.concrete.constraint)
        .map(|constraint| (constraint.kind, constraint.target()))
    else {
        return;
    };
    if matches!(
        kind,
        RuntimeConstraintKind::FollowPath
            | RuntimeConstraintKind::ListFollowPath
            | RuntimeConstraintKind::Other
            | RuntimeConstraintKind::Scroll
            | RuntimeConstraintKind::ScrollBar
    ) {
        return;
    }
    let Some((target, parent)) = target.zip(
        objects
            .component(handle)
            .and_then(|component| component.parent),
    ) else {
        return;
    };
    objects.add_dependent(target, parent);
}

pub(crate) fn optional_target_for_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "RotationConstraint" | "ScaleConstraint" | "TranslationConstraint"
    )
}

pub(crate) fn state_for_type(type_name: &str) -> Option<RuntimeTargetedConstraintState> {
    nuxie_schema::definition_by_name(type_name)
        .is_some_and(|definition| definition.is_a("TargetedConstraint"))
        .then(|| RuntimeTargetedConstraintState::new(!optional_target_for_type(type_name)))
}
