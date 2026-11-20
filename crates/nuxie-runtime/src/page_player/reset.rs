//! Reset what the engine holds under one object of the player's artboard.
//!
//! The contract: after a reset inside a step, everything the engine holds
//! under that object is what a freshly made instance of the same file would
//! hold after its first settle, and the frame drawn after that same step
//! shows it, and from then on the two play the same. Property values a host
//! wrote are the only difference, and they stay.
//!
//! A reset meets it in three parts, each in the engine's own place:
//!
//! 1. Rebuild, or return in place. What the engine can make again from the
//!    file is rebuilt, made as an instance of the file makes it, and
//!    replaces what was there: a nested animation's playback (of a simple or
//!    a remap animation), the layers of a nested machine, the edit state of a
//!    text field, the physics of a scroll view, and a list's rows, each
//!    row's artboard and machine. What other objects link to is returned in
//!    place, each kind by its own `reset_in_place`, which names every field
//!    it holds: a nested machine, which its parent's machine and its host
//!    link to; a scroll view and a text field, which are objects of the
//!    file; and a nested host's artboard, which its host, its layout and its
//!    data context link to. The player's own listeners drop the presses and
//!    hovers they hold on objects under the reset object.
//! 2. First frame. Every nested artboard host the reset reached, of any host
//!    type, and every artboard component list, is advanced by zero as a new
//!    frame through the engine's advancing-component dispatch: the call a
//!    fresh instance's first frame makes on it. That applies the poses a
//!    fresh instance shows. A machine takes its transitions out of entry and
//!    applies its states' animations at time zero, a simple animation
//!    applies its starting pose, and the call recurses into the hosted
//!    artboards. The within-frame settle cannot do this: it advances hosts as
//!    upstream does within a frame, without a new frame, which skips simple
//!    animations, and that rule stays for every other settle.
//! 3. Settle. The step's within-frame settle after the round updates what
//!    the first two parts dirtied, as it does after any write: a scroll's
//!    content transform, reshaped text, layout and paint, for the frame
//!    drawn after the step.
//!
//! What a reset leaves as it is: on a nested machine, the values of its
//! inputs, which its listeners and its host set, and its pointer latches
//! (its scroll latch, drag holders, drag owners, disabled pointer ids and
//! the pointers its hosted artboards hold); on a scroll view, whether a drag
//! holds it; on a text field, whether it has focus; and the inputs of the
//! player's own machine, which is under no object. A rebuild runs only when
//! a driver calls a reset: a player with no driver, or a driver that never
//! resets, runs none of it.
use std::collections::HashSet;

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponentHandle,
    animation::{
        nested_animation, nested_linear_animation::NestedLinearAnimation,
        nested_state_machine::NestedStateMachine,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    artboard_component_list::ArtboardComponentList,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    generated::{
        animation::nested_state_machine_base::NestedStateMachineBase,
        artboard_component_list_base::ArtboardComponentListBase,
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        core_registry::CoreRegistry,
    },
    math::vec2d::Vec2D,
    text::text_input::TextInput,
};
use crate::{ArtboardInstance, CoreHandle, StateMachineInstance};

/// Reset everything under `root` in the player's artboard, `root` included,
/// to what a fresh instance of the file holds after its first settle
/// (module docs). `machine` is the player's own machine, whose listeners on
/// objects under `root` drop the pointers they hold. Whether anything was
/// reset.
pub(super) fn reset_subtree(
    artboard: &ArtboardInstance,
    machine: Option<&StateMachineInstance>,
    root: &CoreHandle,
) -> bool {
    let native = artboard.native_handle();
    let objects = native.with_artboard(|artboard| artboard.base.objects().to_vec());
    let sources = source_objects(&native);
    let under: HashSet<u32> = objects
        .iter()
        .enumerate()
        .filter_map(|(local_id, object)| {
            let object = object.as_ref()?;
            descends_from(object, root)
                .then(|| u32::try_from(local_id).ok())
                .flatten()
        })
        .collect();
    let mut changed = false;
    let mut visited = HashSet::new();
    let mut hosts = Vec::new();
    for (local_id, object) in objects.iter().enumerate() {
        let Some(object) = object else { continue };
        if u32::try_from(local_id).is_ok_and(|id| under.contains(&id)) {
            let source = sources.get(local_id).cloned().flatten();
            changed |= reset_object(object, source.as_ref(), &mut visited);
            if hosts_artboards(object) {
                hosts.push(object.clone());
            }
        }
    }
    if let Some(machine) = machine {
        changed |= machine.native_handle().with_instance_mut(|machine| {
            machine.release_listener_pointers(|id| under.contains(&id))
        });
    }
    // Only the player's artboard's own hosts: the first frame recurses into
    // the artboards they host, so a host inside one is advanced once, by it.
    for host in &hosts {
        AdvancingComponentHandle::classified(host).advance(0.0, FIRST_FRAME);
    }
    changed
}

/// The flags a root artboard's frame hands its advancing components: what a
/// fresh instance's first frame advances a host with.
const FIRST_FRAME: AdvanceFlags = AdvanceFlags(
    AdvanceFlags::IS_ROOT.0
        | AdvanceFlags::ANIMATE.0
        | AdvanceFlags::ADVANCE_NESTED.0
        | AdvanceFlags::NEW_FRAME.0,
);

/// Whether `object` hosts artboards: a nested artboard of any host type, or
/// an artboard component list.
fn hosts_artboards(object: &CoreHandle) -> bool {
    object
        .with_mut(|object| {
            object.as_nested_artboard().is_some()
                || object.as_artboard_component_list_mut().is_some()
        })
        .unwrap_or(false)
}

fn descends_from(object: &CoreHandle, root: &CoreHandle) -> bool {
    let mut current = Some(object.clone());
    // An artboard's parent chain is acyclic; the bound only guards a
    // malformed import.
    for _ in 0..4096 {
        let Some(node) = current else { return false };
        if &node == root {
            return true;
        }
        current = node
            .with(|object| object.component_parent_handle())
            .flatten();
    }
    false
}

/// The objects of the artboard `artboard` was instanced from, by local id:
/// what the file gave each object.
fn source_objects(artboard: &RuntimeArtboardInstanceHandle) -> Vec<Option<CoreHandle>> {
    artboard
        .with_artboard(|artboard| artboard.artboard_source_handle())
        .and_then(|source| source.with_downcast::<Artboard, _>(|source| source.objects().to_vec()))
        .unwrap_or_default()
}

/// Reset one object's own engine state, and everything inside it when it
/// hosts artboards. `source` is the object the file holds at the same local
/// id, when there is one.
///
/// Each kind is reached through the engine's shared capability, so every
/// type of that kind is: a scroll constraint, a text input, a nested
/// artboard of any host type (`NestedArtboard`, `NestedArtboardLayout`,
/// `NestedArtboardLeaf`) and an artboard component list.
fn reset_object(
    object: &CoreHandle,
    source: Option<&CoreHandle>,
    visited: &mut HashSet<(usize, usize, u64)>,
) -> bool {
    if object.is_type_of(ScrollConstraintBase::TYPE_KEY) {
        let source = source.filter(|source| source.is_type_of(ScrollConstraintBase::TYPE_KEY));
        let authored = |key: u16| {
            source
                .and_then(|source| CoreRegistry::get_double_handle(source, i32::from(key)))
                .unwrap_or(0.0)
        };
        let authored = Vec2D::new(
            authored(ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY),
            authored(ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY),
        );
        // A copy of the file's physics, as an instance of the file copies it.
        let physics = source
            .and_then(|source| {
                source
                    .with(|source| source.as_scroll_constraint().map(ScrollConstraint::physics))
                    .flatten()
                    .flatten()
            })
            .map(|physics| {
                physics
                    .clone_occurrence()
                    .expect("live ScrollPhysics is cloneable")
            });
        let reset = object.with_mut(|object| {
            object
                .as_scroll_constraint_mut()
                .map(|scroll| scroll.reset_in_place(authored, physics))
        });
        if let Some(changed) = reset.flatten() {
            return changed;
        }
    }
    let typed = object.with_mut(|object| object.as_text_input_mut().map(TextInput::reset_in_place));
    if let Some(changed) = typed.flatten() {
        return changed;
    }
    let hosted = object.with(|object| {
        object.as_nested_artboard().map(|nested| {
            (
                nested.artboard_instance_default(),
                nested.nested_animations().to_vec(),
            )
        })
    });
    if let Some((instance, animations)) = hosted.flatten() {
        let Some(instance) = instance else {
            return false;
        };
        for animation in animations {
            reset_nested_animation(&animation, &instance);
        }
        reset_artboard(&instance, visited);
        return true;
    }
    if object.is_type_of(ArtboardComponentListBase::TYPE_KEY) {
        return ArtboardComponentList::rebuild_rows_occurrence(object);
    }
    false
}

/// Reset one of a host's nested animations, which plays on `hosted`, the
/// artboard the host holds: a state machine is returned in place
/// (`StateMachineInstance::reset_in_place`), and a simple or a remap
/// animation's playback is rebuilt.
fn reset_nested_animation(animation: &CoreHandle, hosted: &RuntimeArtboardInstanceHandle) {
    if animation.is_type_of(NestedStateMachineBase::TYPE_KEY) {
        // The registry has no subtype of NestedStateMachine and no shared
        // capability for it: every nested machine is this one type.
        let machine = animation
            .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
            .flatten();
        if let Some(machine) = machine {
            machine.with_instance_mut(
                crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance::reset_in_place,
            );
        }
        return;
    }
    // The machine that hears the playback's events linked itself to it when
    // that machine was made; the link moves to the rebuilt playback.
    let links = animation
        .with_mut(|animation| {
            animation
                .as_nested_linear_animation_mut()
                .and_then(NestedLinearAnimation::animation_instance_mut)
                .map(|instance| std::mem::take(&mut instance.nested_event_notifier))
        })
        .flatten();
    let Some(links) = links else { return };
    // How a host makes its animations' playback when it mounts an instance:
    // a remap animation's then takes the time its property holds.
    nested_animation::initialize_animation(animation, hosted.downgrade());
    animation.with_mut(|animation| {
        if let Some(instance) = animation
            .as_nested_linear_animation_mut()
            .and_then(NestedLinearAnimation::animation_instance_mut)
        {
            instance.nested_event_notifier = links;
        }
    });
}

/// Everything in a hosted artboard is under the object that hosts it: the
/// artboard itself is returned in place (`Artboard::reset_in_place`), and
/// each of its objects is reset by kind.
fn reset_artboard(
    artboard: &RuntimeArtboardInstanceHandle,
    visited: &mut HashSet<(usize, usize, u64)>,
) {
    if !visited.insert(artboard.core_handle().identity_key()) {
        return;
    }
    artboard.with_artboard_mut(|artboard| artboard.base.reset_in_place());
    let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    let sources = source_objects(artboard);
    for (local_id, object) in objects.iter().enumerate() {
        let Some(object) = object else { continue };
        let source = sources.get(local_id).cloned().flatten();
        reset_object(object, source.as_ref(), visited);
    }
}
