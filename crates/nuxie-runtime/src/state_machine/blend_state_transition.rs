use super::state_instance::RuntimeStateInstanceKind;
use super::*;

// Mirrors src/animation/blend_state_transition.cpp and its primary header.

/// `LayerStateImporter` resolves the authored BlendAnimation occurrence to its
/// insertion-order slot. Rust retains that slot instead of the C++ pointer.
pub(super) fn from_imported(
    transition: &nuxie_binary::RuntimeStateTransition<'_>,
) -> Option<usize> {
    transition.exit_blend_animation_index
}

/// Mirrors `BlendStateTransition::exitTimeAnimationInstance`.
pub(super) fn exit_time_animation_instance<'a>(
    from: &'a RuntimeStateInstance,
    exit_blend_animation_index: Option<usize>,
) -> Option<&'a LinearAnimationInstance> {
    let exit_blend_animation_index = exit_blend_animation_index?;
    match &from.kind {
        RuntimeStateInstanceKind::Blend1D(instance) => {
            instance.animation_instance(exit_blend_animation_index)
        }
        RuntimeStateInstanceKind::BlendDirect(instance) => {
            instance.animation_instance(exit_blend_animation_index)
        }
        RuntimeStateInstanceKind::System(_) | RuntimeStateInstanceKind::Animation(_) => None,
    }
}

/// Mirrors `BlendStateTransition::exitTimeAnimation`. Rust obtains the
/// immutable LinearAnimation through the occurrence's retained handle rather
/// than the C++ BlendAnimation pointer.
pub(super) fn exit_time_animation<'a>(
    from: &'a RuntimeStateInstance,
    exit_blend_animation_index: Option<usize>,
    artboard: &'a ArtboardInstance,
) -> Option<RuntimeTransitionAnimationRef<'a>> {
    let instance = exit_time_animation_instance(from, exit_blend_animation_index)?;
    let animation = artboard.linear_animation_instance_definition(instance)?;
    Some(RuntimeTransitionAnimationRef {
        instance,
        animation,
    })
}
