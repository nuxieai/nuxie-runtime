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
        nested_simple_animation::NestedSimpleAnimation, nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    artboard_component_list::ArtboardComponentList,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    generated::{
        constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        core_registry::CoreRegistry,
    },
    nested_artboard::NestedArtboard,
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
        if let Some(changed) = object.with_downcast_mut::<ScrollConstraint, _>(|scroll| {
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
        }) {
            return changed;
        }
    }
    if let Some(changed) = object.with_downcast_mut::<TextInput, _>(TextInput::reset_typed_text) {
        return changed;
    }
    if let Some((instance, animations)) = object.with_downcast::<NestedArtboard, _>(|nested| {
        (
            nested.artboard_instance_default(),
            nested.nested_animations().to_vec(),
        )
    }) {
        let mut changed = false;
        for animation in animations {
            changed |= reset_nested_animation(&animation);
        }
        if let Some(instance) = instance {
            changed |= reset_artboard(&instance, visited);
        }
        return changed;
    }
    if let Some(rows) = object.with_downcast::<ArtboardComponentList, _>(|list| {
        (0..list.artboard_count())
            .filter_map(|index| i32::try_from(index).ok())
            .map(|index| {
                (
                    list.artboard_instance(index),
                    list.state_machine_instance(index),
                )
            })
            .collect::<Vec<_>>()
    }) {
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

fn reset_nested_animation(animation: &CoreHandle) -> bool {
    if let Some(machine) = animation
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
    {
        return reset_machine(&machine);
    }
    animation
        .with_downcast_mut::<NestedSimpleAnimation, _>(|animation| {
            let speed = animation.base.speed();
            animation
                .base
                .base
                .animation_instance_mut()
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
