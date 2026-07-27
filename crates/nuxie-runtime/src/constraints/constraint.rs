//! Direct Rust home for pinned C++ `include/rive/constraints/constraint.hpp`
//! and `src/constraints/constraint.cpp`.
//!
//! Artboard retains authored-order lifecycle orchestration. This module owns
//! the base Constraint occurrence, parent validation/registration, base
//! strength callback, and unconditional dirty propagation.

use anyhow::Context;

use crate::ArtboardInstance;
use crate::components::{ComponentHandle, RuntimeConstraintScratch};
use crate::constraints::targeted_constraint::{
    RuntimeTargetedConstraintState, state_for_type as targeted_constraint_state_for_type,
};
use crate::objects::InstanceObjectArena;

pub(crate) const CONSTRAINT_STRENGTH_PROPERTY_KEY: u16 = 172;

/// Rust dispatch tag for the concrete C++ Constraint subclass retained by an
/// occurrence. The tag is an adapter only; concrete behavior remains in the
/// corresponding source-named module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeConstraintKind {
    Distance,
    FollowPath,
    ListFollowPath,
    Ik,
    Rotation,
    Scale,
    Scroll,
    ScrollBar,
    Transform,
    Translation,
    Other,
}

/// Runtime-only relations retained by one C++ `Constraint` occurrence.
///
/// Generated values remain in the occurrence's sole generated backing store.
/// The nullable targeted payload is the embedded TargetedConstraint base for
/// concrete subclasses; scratch preserves the existing allocation-free
/// transform-constraint adapter until those leaf owners are split.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RuntimeConstraintState {
    pub(crate) kind: RuntimeConstraintKind,
    targeted: Option<RuntimeTargetedConstraintState>,
    pub(crate) scratch: RuntimeConstraintScratch,
}

impl RuntimeConstraintKind {
    fn for_type(type_name: &'static str) -> Self {
        match type_name {
            "DistanceConstraint" => Self::Distance,
            "FollowPathConstraint" => Self::FollowPath,
            "ListFollowPathConstraint" => Self::ListFollowPath,
            "IKConstraint" => Self::Ik,
            "RotationConstraint" => Self::Rotation,
            "ScaleConstraint" => Self::Scale,
            "ScrollConstraint" => Self::Scroll,
            "ScrollBarConstraint" => Self::ScrollBar,
            "TransformConstraint" => Self::Transform,
            "TranslationConstraint" => Self::Translation,
            _ => Self::Other,
        }
    }
}

impl RuntimeConstraintState {
    pub(crate) fn new(type_name: &'static str) -> Self {
        let kind = RuntimeConstraintKind::for_type(type_name);
        Self {
            kind,
            targeted: targeted_constraint_state_for_type(type_name),
            scratch: RuntimeConstraintScratch::for_kind(kind),
        }
    }

    pub(crate) const fn targeted(self) -> Option<RuntimeTargetedConstraintState> {
        self.targeted
    }

    pub(crate) fn target(self) -> Option<ComponentHandle> {
        self.targeted
            .and_then(RuntimeTargetedConstraintState::target)
    }

    pub(crate) fn set_target(&mut self, target: Option<ComponentHandle>) {
        if let Some(targeted) = self.targeted.as_mut() {
            targeted.set_target(target);
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self {
            kind: self.kind,
            targeted: self
                .targeted
                .map(RuntimeTargetedConstraintState::clone_for_occurrence),
            scratch: RuntimeConstraintScratch::for_kind(self.kind),
        }
    }
}

/// `Constraint::onAddedDirty` validates its TransformComponent parent and
/// appends this occurrence to that parent's constraint list in authored order
/// (`src/constraints/constraint.cpp:10-23`).
pub(crate) fn on_added_dirty(
    objects: &mut InstanceObjectArena,
    handle: ComponentHandle,
) -> anyhow::Result<()> {
    let parent = objects
        .component(handle)
        .and_then(|component| component.parent)
        .context("Constraint is missing its parent Component")?;
    if !objects
        .component(parent)
        .is_some_and(|parent| parent.capabilities.transform)
    {
        anyhow::bail!("Constraint parent is not a TransformComponent");
    }
    objects.add_constraint(parent, handle);
    Ok(())
}

/// Base `Constraint::markConstraintDirty` marks the retained constrained
/// parent's transform (`src/constraints/constraint.cpp:25-29`).
pub(crate) fn mark_constraint_dirty(
    artboard: &mut ArtboardInstance,
    constraint_local_id: usize,
) -> bool {
    let Some(parent) = artboard
        .component_handle(constraint_local_id)
        .and_then(|constraint| artboard.objects.component(constraint))
        .and_then(|constraint| constraint.parent)
    else {
        return false;
    };
    artboard.mark_transform_dirty_handle(parent)
}

/// `Constraint::onDirty` unconditionally calls the virtual
/// `markConstraintDirty`. IK overrides that virtual owner; every other
/// current concrete constraint uses the base parent-transform path
/// (`src/constraints/constraint.cpp:37-42`;
/// `src/constraints/ik_constraint.cpp:76-86`).
pub(crate) fn on_dirty(artboard: &mut ArtboardInstance, constraint: ComponentHandle) -> bool {
    let Some((local_id, kind)) = artboard
        .objects
        .component(constraint)
        .and_then(|component| {
            component
                .concrete
                .constraint
                .map(|constraint| (component.local_id, constraint.kind))
        })
    else {
        return false;
    };
    if kind == RuntimeConstraintKind::Ik {
        artboard.mark_ik_constraint_dirty(local_id)
    } else {
        mark_constraint_dirty(artboard, local_id)
    }
}

pub(crate) fn double_change_marks_parent_dirty(
    kind: RuntimeConstraintKind,
    property_key: u16,
) -> bool {
    property_key == CONSTRAINT_STRENGTH_PROPERTY_KEY && kind != RuntimeConstraintKind::Ik
}
