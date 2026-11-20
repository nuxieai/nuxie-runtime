//! Reset what the engine holds under one object of the player's artboard.
//!
//! The contract: a reset returns the engine's state holders under the
//! object to their start and leaves every property value alone, whoever
//! wrote it. So a property that an animation or a state wrote, and that the
//! starting state does not key, keeps its value until something writes it.
//!
//! A state holder is what the engine keeps of its own while it plays, beyond
//! the properties: an animation's playback, a machine's layers, a scroll, a
//! text field's edit state, a list's rows, the artboard a paint paints with,
//! and a layout's motion in flight. The table below lists every kind, the
//! ones a reset leaves included.
//!
//! A reset works in three parts, each in the engine's own place:
//!
//! 1. Rebuild, or return in place. A state holder the engine can make again
//!    from the file is rebuilt, made as an instance of the file makes it,
//!    and replaces what was there. One that other objects link to is
//!    returned in place, each kind by its own `reset_in_place`, which names
//!    every field it holds, so a field added later does not compile until
//!    someone decides what a reset does to it. The player's own listeners
//!    drop the presses and hovers they hold on objects under the reset
//!    object.
//! 2. First frame. Every nested artboard host the reset reached, of any host
//!    type, and every artboard component list, is advanced by zero as a new
//!    frame through the engine's advancing-component dispatch: the call an
//!    instance's first frame makes on it. A machine takes its transitions
//!    out of entry and applies its starting states' animations at time zero,
//!    a simple animation applies its starting pose, and the call recurses
//!    into the hosted artboards. This is the playback writing what it keys,
//!    as it does on any frame; what it does not key keeps its value. The
//!    within-frame settle cannot do this: it advances hosts as upstream does
//!    within a frame, without a new frame, which skips simple animations, and
//!    that rule stays for every other settle.
//! 3. Settle. The step's within-frame settle after the round updates what
//!    the first two parts dirtied, as it does after any write: a scroll's
//!    content transform, reshaped text, layout and paint, for the frame
//!    drawn after the step.
//!
//! Where a state holder keeps its state in a property, returning it writes
//! that property, and nothing else is written:
//!
//! - A scroll's position lives in its constraint's scroll offset
//!   properties, which scrolling writes. Returning the scroll to its start
//!   writes them back to the offsets the file gives it.
//! - A text field's text is its edit state's working copy, which typing
//!   writes. The edit state starts from the text last written to the field
//!   (the file's text until a host writes one), so the field's text is put
//!   back to that.
//! - A layout's motion in flight ends where it was going. The layout it
//!   lands on is the one its properties give, so no property is written.
//! - A list's rows and a paint's artboard are rebuilt whole, so their
//!   objects hold the file's values.
//!
//! What a reset does to each kind of object that holds state of its own
//! while it plays:
//!
//! | Kind | The reset | Why |
//! | --- | --- | --- |
//! | Nested simple or remap animation's playback | Rebuilt | Made as a host makes it when it mounts an instance; the parent machine's links to hear its events move to it. |
//! | Nested state machine | Returned in place, its layers rebuilt | Its parent's machine, its host and hit testing hold it. Its layers start at their entry states. Its inputs' values and its pointer latches are left as they are. A rebuilt layer draws from the shared random source where it stands; the source is not restarted. |
//! | Scroll constraint | Returned in place, its physics rebuilt | An object of the file. Its offsets go back to the file's (above); whether a drag holds it is left as it is. |
//! | Text input | Returned in place, its edit state rebuilt | An object of the file. Its text goes back to the text last written to it (above); its focus is left as it is. |
//! | Artboard component list | Its rows rebuilt, each row's artboard and machine | Made as the list makes a row; the rows it kept pooled are dropped. |
//! | Hosted artboard | Returned in place, its objects reset by kind | Its host and its host's layout, focus and data context hold it. Its objects keep their property values. |
//! | Layout (a layout component, an artboard, a layout host) | Returned in place | Its motion in flight ends where it was going (above). |
//! | Layout participant | Returned in place | As a layout. |
//! | Paint that paints with an artboard | Returned in place, its artboard rebuilt | Made as the paint makes it when it is added, its machine or animation from the start. |
//! | Nested host whose frames skip (paused, collapsed, or playing at a fixed rate) | Its playback returned as above | A host that skips a frame skips the reset's first frame too, so what its playback keys holds its values from before the reset until the host next advances. |
//! | Script state (scripted drawables, layouts, path effects, transitions and their list rows, converters, interpolators, listener actions and conditions) | Left as it is | It lives in the script's own state, which only the script's init makes. |
//! | Audio | Left as it is | A sound an audio event started plays on. |
//! | Video | Left as it is | Its playback position, play state and frame are the host's. |
//! | Text style artboards | Left as they are | Texts can share one through the file's shared styles, so one object's reset cannot rebuild it alone. |
//! | Bound values (a converter that eases a value, a formula's drawn random numbers, a bind's last seen values) | Left as they are | A bind carries the view model's value, which the reset leaves as it is. |
//! | Inverse kinematics | Left as it is | It holds only the solve it wrote last frame, which the next update replaces when anything moves the bone. |
//! | Focus, a text field's click count, a hosted mesh's last pointer, a scripted drawable's scroll latch | Left as they are | Pointer and focus state, like the latches above. |
//! | A custom trigger's change stamp | Left as it is | The machine's reset moves its baselines past it, so a trigger written before the reset does not fire after it. |
//! | Live move flight | Left as it is | The live move that set it ends it. |
//! | Semantic retries, layer masks, cached rasters | Left as they are | They hold nothing drawn or reported differently. |
//! | Joystick | Nothing to reset | Its time comes from its position each frame. |
//!
//! The inputs of the player's own machine, which is under no object, are
//! left as they are. A rebuild runs only when a driver calls a reset: a
//! player with no driver, or a driver that never resets, runs none of it.
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
        core_registry::CoreRegistry, layout::layout_participant_base::LayoutParticipantBase,
        shapes::paint::paint_image_base::PaintImageBase,
    },
    layout::layout_participant::LayoutParticipant,
    layout_component::LayoutComponent,
    math::vec2d::Vec2D,
    shapes::paint::paint_image::PaintImage,
    text::text_input::TextInput,
};
use crate::{ArtboardInstance, CoreHandle, StateMachineInstance};

/// Return the state holders under `root` in the player's artboard, `root`
/// included, to their start, leaving every property value alone (module
/// docs). `machine` is the player's own machine, whose listeners on
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

/// The flags a root artboard's frame hands its advancing components: what an
/// instance's first frame advances a host with.
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
/// type of that kind is: a layout of any type (an artboard, a layout
/// component, a layout host), a layout participant, a scroll constraint, a
/// text input, a nested artboard of any host type (`NestedArtboard`,
/// `NestedArtboardLayout`, `NestedArtboardLeaf`), an artboard component
/// list and a paint that paints with an artboard. A layout's interpolation
/// ends whatever else the object is.
fn reset_object(
    object: &CoreHandle,
    source: Option<&CoreHandle>,
    visited: &mut HashSet<(usize, usize, u64)>,
) -> bool {
    let is_layout = object
        .with(|object| object.as_layout_component().is_some())
        .unwrap_or(false);
    let laid = is_layout && LayoutComponent::reset_in_place_occurrence(object);
    laid | reset_kind(object, source, visited)
}

/// `reset_object` for what an object is besides a layout.
fn reset_kind(
    object: &CoreHandle,
    source: Option<&CoreHandle>,
    visited: &mut HashSet<(usize, usize, u64)>,
) -> bool {
    if object.is_type_of(LayoutParticipantBase::TYPE_KEY) {
        return LayoutParticipant::reset_in_place_occurrence(object);
    }
    if object.is_type_of(PaintImageBase::TYPE_KEY) {
        return PaintImage::reset_in_place_occurrence(object);
    }
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
