//! Reset what the engine holds under one object of the player's artboard.
//!
//! Engine-held state is what the engine changes on its own while it plays:
//! a scroll position, typed text, the playheads of nested artboards' machines
//! and animations, and the presses and hovers listeners track. Other
//! properties are not engine-held state: whatever the file, a bind or a host
//! last wrote stays. A scroll position lives in its constraint's scroll
//! offset properties, which scrolling writes; a reset puts them back to the
//! values the file gave them.
use std::collections::HashSet;

use crate::mechanical_port::source::{
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

/// Reset everything under `root` in the player's artboard, `root` included.
/// `machine` is the player's own machine, whose listeners on objects under
/// `root` drop the pointers they hold. Whether anything was reset.
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
    for (local_id, object) in objects.iter().enumerate() {
        let Some(object) = object else { continue };
        if u32::try_from(local_id).is_ok_and(|id| under.contains(&id)) {
            let source = sources.get(local_id).cloned().flatten();
            changed |= reset_object(object, source.as_ref(), &mut visited);
        }
    }
    if let Some(machine) = machine {
        changed |= machine.native_handle().with_instance_mut(|machine| {
            machine.release_listener_pointers(|id| under.contains(&id))
        });
    }
    changed
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

/// Reset one of a host's nested animations: a state machine back to its
/// entry, and a simple animation's playhead to its start. A simple
/// animation's playhead is engine-held, since it advances with time; a remap
/// animation's playhead follows its time property, which the file, a bind or
/// a host wrote, so a reset leaves it.
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
