//! Reset what the engine holds under one object of the player's artboard.
//!
//! The contract: after a reset inside a step, everything the engine holds
//! under that object is what a freshly made instance of the same file would
//! hold after its first settle, and the frame drawn after that same step
//! shows it. Property values a host wrote are the only difference, and they
//! stay.
//!
//! A reset meets it in three parts, each in the engine's own place:
//!
//! 1. Return. Each engine-held value goes back to the value a fresh instance
//!    starts from: a scroll position to the offsets the file gave its
//!    constraint, with its physics stopped; typed text to the text last
//!    written to the field; a nested or list-row machine to its entry state,
//!    its listeners holding no pointer; a simple animation's playhead to its
//!    start. The player's own listeners drop the presses and hovers they
//!    hold on objects under the reset object.
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
//! A remap animation's playhead is not returned. It is its time property,
//! which the file, a bind or a host wrote, so a fresh instance takes the
//! same value, and the first frame applies it at the same time.
use std::collections::HashSet;

use crate::mechanical_port::source::{
    advance_flags::AdvanceFlags,
    advancing_component::AdvancingComponentHandle,
    animation::{
        nested_linear_animation::NestedLinearAnimation, nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    generated::{
        animation::{
            nested_simple_animation_base::NestedSimpleAnimationBase,
            nested_state_machine_base::NestedStateMachineBase,
        },
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        core_registry::CoreRegistry,
    },
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
        let authored = |key: u16| {
            source
                .filter(|source| source.is_type_of(ScrollConstraintBase::TYPE_KEY))
                .and_then(|source| CoreRegistry::get_double_handle(source, i32::from(key)))
                .unwrap_or(0.0)
        };
        let x = authored(ScrollConstraintBase::SCROLL_OFFSET_X_PROPERTY_KEY);
        let y = authored(ScrollConstraintBase::SCROLL_OFFSET_Y_PROPERTY_KEY);
        let reset = object.with_mut(|object| {
            object.as_scroll_constraint_mut().map(|scroll| {
                let moved = scroll.offset_x() != x
                    || scroll.offset_y() != y
                    || scroll.authored_scroll_offset_x() != x
                    || scroll.authored_scroll_offset_y() != y;
                scroll.stop_physics();
                scroll.set_authored_scroll_offset_x(x);
                scroll.set_authored_scroll_offset_y(y);
                scroll.set_offset_x(x);
                scroll.set_offset_y(y);
                moved
            })
        });
        if let Some(changed) = reset.flatten() {
            return changed;
        }
    }
    let typed =
        object.with_mut(|object| object.as_text_input_mut().map(TextInput::reset_typed_text));
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
        let mut changed = false;
        for animation in animations {
            changed |= reset_nested_animation(&animation);
        }
        if let Some(instance) = instance {
            changed |= reset_artboard(&instance, visited);
        }
        return changed;
    }
    let rows = object.with_mut(|object| {
        object.as_artboard_component_list_mut().map(|list| {
            (0..list.artboard_count())
                .filter_map(|index| i32::try_from(index).ok())
                .map(|index| {
                    (
                        list.artboard_instance(index),
                        list.state_machine_instance(index),
                    )
                })
                .collect::<Vec<_>>()
        })
    });
    if let Some(rows) = rows.flatten() {
        let mut changed = false;
        for (instance, machine) in rows {
            if let Some(machine) = machine {
                changed |= reset_machine(&machine);
            }
            if let Some(instance) = instance {
                changed |= reset_artboard(&instance, visited);
            }
        }
        return changed;
    }
    false
}

/// Return one of a host's nested animations to where a fresh instance
/// starts it: a state machine at its entry, and a simple animation's
/// playhead at its start. The first frame (module docs) applies them. A
/// remap animation's playhead is its time property, which is not returned.
fn reset_nested_animation(animation: &CoreHandle) -> bool {
    if animation.is_type_of(NestedStateMachineBase::TYPE_KEY) {
        // The registry has no subtype of NestedStateMachine and no shared
        // capability for it: every nested machine is this one type.
        return animation
            .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
            .flatten()
            .is_some_and(|machine| reset_machine(&machine));
    }
    if !animation.is_type_of(NestedSimpleAnimationBase::TYPE_KEY) {
        return false;
    }
    let speed = CoreRegistry::get_double_handle(
        animation,
        i32::from(NestedSimpleAnimationBase::SPEED_PROPERTY_KEY),
    )
    .unwrap_or(1.0);
    animation
        .with_mut(|animation| {
            animation
                .as_nested_linear_animation_mut()
                .and_then(NestedLinearAnimation::animation_instance_mut)
                .is_some_and(|instance| {
                    let before = instance.time();
                    instance.reset(speed);
                    instance.time() != before
                })
        })
        .unwrap_or(false)
}

fn reset_machine(machine: &RuntimeStateMachineInstanceHandle) -> bool {
    machine.with_instance_mut(|machine| {
        machine.reset_state();
        machine.mark_needs_advance();
    });
    true
}

/// Everything in a hosted artboard is under the object that hosts it.
fn reset_artboard(
    artboard: &RuntimeArtboardInstanceHandle,
    visited: &mut HashSet<(usize, usize, u64)>,
) -> bool {
    if !visited.insert(artboard.core_handle().identity_key()) {
        return false;
    }
    let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    let sources = source_objects(artboard);
    let mut changed = false;
    for (local_id, object) in objects.iter().enumerate() {
        let Some(object) = object else { continue };
        let source = sources.get(local_id).cloned().flatten();
        changed |= reset_object(object, source.as_ref(), visited);
    }
    changed
}
