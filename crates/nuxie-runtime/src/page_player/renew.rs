//! Give a nested artboard host a fresh instance of its own artboard.
//!
//! The host's instance is discarded and a new one is made from the artboard
//! as the file has it, the way a list makes a row: never copied from the
//! instance that leaves, so nothing a script, an animation, a scroll or a
//! write left in it reaches the new one. The new instance runs its scripts'
//! setup as any new instance does, and the instance that leaves disposes its
//! scripts as it goes.
//!
//! What stays is the host: its own records, its nested animations (the
//! machine record a use carries among them), its listeners' place in the
//! machines above it, its focus scope, its place in the accessibility tree
//! and in its parent's layout. Each link those hold to the instance is made
//! again for the new one:
//!
//! | Holder | What happens |
//! | --- | --- |
//! | The host's own instance slot | The old instance is taken out and dropped; the new one is nested as a runtime swap nests one. |
//! | The host's nested animations (its machines, simple and remap animations) | Kept. Each drops its playback of the old instance and makes a new one on the new instance, at its entry, with its authored inputs. |
//! | The machine above, which hears a nested machine's events | Each machine that listened to the old playback listens to the new one, and the new playback names the host, as at load. |
//! | The pointers the machines above hold on the host | Dropped, as a reset drops them: the playback they were pressing is gone. |
//! | Focus | The old instance's focus tree is cleaned up; the host's focus scope is synced again for the new one. |
//! | Accessibility tree | The old instance's semantic subtree is cleaned up; the new one's is built under the host's closest semantic node. |
//! | Layout | The old instance leaves its parent's layout; the host's size overrides and its parent's layout children are synced for the new one, and the new one's style is synced. |
//! | Collapse | The new instance takes the host's collapse. |
//! | Scripts | The old instance's scripted objects dispose their scripts and leave the VM when it drops; the new instance's run their generator, take their inputs and call `init`, as any new instance does. |
//! | Walk lists of the hosted scene | Marked changed for the host. |
//!
//! The new instance then plays its first frame, as a reset plays one, so its
//! starting states apply what they key in the step that replaced it.
//!
//! A host is refused, and nothing changes, when it hosts nothing, when a live
//! move carries instances through it (the live move owns its instance), when
//! a bound view model property chooses its artboard (the bind owns it), or
//! when it carries a view model instance (a model's values are not the
//! file's alone).
use crate::CoreHandle;
use crate::mechanical_port::source::{
    advancing_component::AdvancingComponentHandle,
    animation::{
        nested_state_machine::NestedStateMachine,
        state_machine_instance::{
            RuntimeStateMachineInstanceHandle, RuntimeStateMachineInstanceWeakHandle,
        },
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    focus_data::FocusData,
    layout_component::{LayoutComponent, owning_layout},
    nested_artboard::NestedArtboard,
    nested_artboard_layout::NestedArtboardLayout,
    semantic::semantic_data::SemanticData,
};

/// Why a host's instance cannot be replaced.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RenewRefusal {
    /// The object hosts no nested artboard, or hosts nothing now.
    NotAHost,
    /// A live move, a bound artboard property or a view model owns what the
    /// host holds.
    Owned,
}

/// Replace the instance `host` holds with a fresh one of the same artboard,
/// made as the file has it (module docs). `artboard` is the artboard that
/// holds `host`, and `machines` the machines that play on it. Always a change
/// when it is not refused.
pub(super) fn renew_hosted_instance(
    artboard: &RuntimeArtboardInstanceHandle,
    machines: &[RuntimeStateMachineInstanceHandle],
    host: &CoreHandle,
) -> Result<(), RenewRefusal> {
    let definition = replaceable_definition(host)?;
    let wrappers = host
        .with(|o| {
            o.as_nested_artboard()
                .map(|n| n.nested_animations().to_vec())
        })
        .flatten()
        .unwrap_or_default();
    let listeners: Vec<_> = wrappers.iter().map(wrapper_listeners).collect();

    // Empty the host: the playback of the old instance, its focus and
    // semantic trees, its place in the parent's layout, then the instance.
    for wrapper in &wrappers {
        wrapper.with_mut(|o| o.nested_animation_release_dependencies());
    }
    let old = host
        .with_mut(|o| {
            let nested = o.as_nested_artboard_mut()?;
            let old = nested.take_artboard_instance();
            nested.reset_mesh_raster();
            old
        })
        .flatten();
    if let Some(old) = old {
        old.cleanup_focus_tree();
        old.cleanup_semantic_tree();
        old.with_artboard_mut(|b| b.set_host_state(None, None, None));
        drop(old);
    }
    // The bind the old mount scheduled, if any, goes with it.
    NestedArtboard::flush_pending_stateful_binding_occurrence(host);

    // Fill it from the file, nested and bound as load leaves a host.
    let Some(instance) = NestedArtboard::live_move_refill_occurrence(host, &definition) else {
        // The definition made an instance once; a failure now leaves the
        // host empty, as a bound swap to an artboard that makes none does.
        crate::hosted_scene::membership_changed_for(host);
        return Ok(());
    };
    relink_listeners(host, &wrappers, &listeners);
    host.with_downcast_mut::<NestedArtboardLayout, _>(|l| l.mark_layout_node_dirty(false));
    let fallback = FocusData::find_closest_focus_node_from_parent(
        host.with(|o| o.component_parent_handle()).flatten(),
    );
    NestedArtboard::sync_nested_focus_tree_occurrence(host, fallback, true, true);
    if let Some(parent) = host
        .with(|o| {
            o.as_nested_artboard()
                .and_then(NestedArtboard::parent_artboard_handle)
        })
        .flatten()
    {
        build_semantics(host, &parent, &instance);
    }
    crate::live_move::reconcile_slot_collapse(host);
    if let Some(layout) = owning_layout(host.with(|o| o.component_parent_handle()).flatten()) {
        LayoutComponent::sync_layout_children_occurrence(&layout);
        LayoutComponent::mark_layout_style_dirty_occurrence(&layout);
    }
    LayoutComponent::sync_style_occurrence(&instance.core_handle());
    crate::hosted_scene::membership_changed_for(host);

    super::reset::release_pointers_under(artboard, machines, host);
    AdvancingComponentHandle::classified(host).advance(0.0, super::reset::FIRST_FRAME);
    Ok(())
}

/// The artboard `host` holds an instance of, as the file has it, when the
/// driver may replace that instance.
fn replaceable_definition(host: &CoreHandle) -> Result<CoreHandle, RenewRefusal> {
    let read = host
        .with(|o| {
            let nested = o.as_nested_artboard()?;
            let owned = nested.is_artboard_data_bound()
                || nested.has_view_model_instance()
                || nested.live_move.is_some();
            Some((nested.artboard_instance_default(), owned))
        })
        .flatten();
    let Some((instance, owned)) = read else {
        return Err(RenewRefusal::NotAHost);
    };
    if owned || crate::hosted_scene::is_carry_host(host) {
        return Err(RenewRefusal::Owned);
    }
    instance
        .and_then(|instance| instance.with_artboard(|b| b.artboard_source_handle()))
        .ok_or(RenewRefusal::NotAHost)
}

/// The machines that hear a host's nested animation `wrapper`.
fn wrapper_listeners(wrapper: &CoreHandle) -> Vec<RuntimeStateMachineInstanceWeakHandle> {
    if let Some(machine) = wrapper
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
    {
        return machine.with_instance(|m| m.nested_event_listeners());
    }
    wrapper
        .with(|o| {
            o.as_nested_linear_animation()
                .and_then(|a| a.animation_instance())
                .map(|a| a.nested_event_notifier.nested_event_listeners())
        })
        .flatten()
        .unwrap_or_default()
}

/// Each machine that heard a wrapper's old playback hears its new one, and
/// the new playback names the host, as a machine links itself to its nested
/// artboards' playback when it is made.
fn relink_listeners(
    host: &CoreHandle,
    wrappers: &[CoreHandle],
    listeners: &[Vec<RuntimeStateMachineInstanceWeakHandle>],
) {
    for (wrapper, listeners) in wrappers.iter().zip(listeners) {
        let live = || listeners.iter().filter(|l| l.upgrade().is_some()).cloned();
        if let Some(machine) = wrapper
            .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
            .flatten()
        {
            machine.with_instance_mut(|m| {
                m.set_nested_artboard(host.clone());
                for listener in live() {
                    m.add_nested_event_listener(listener);
                }
            });
        } else {
            wrapper.with_mut(|o| {
                if let Some(a) = o
                    .as_nested_linear_animation_mut()
                    .and_then(|a| a.animation_instance_mut())
                {
                    a.set_nested_artboard(host.clone());
                    for listener in live() {
                        a.add_nested_event_listener(listener);
                    }
                }
            });
        }
    }
}

/// Build `instance`'s semantic subtree under `host`'s closest semantic node,
/// when `parent`, the artboard holding the host, keeps a semantic tree.
fn build_semantics(
    host: &CoreHandle,
    parent: &CoreHandle,
    instance: &RuntimeArtboardInstanceHandle,
) {
    let Some(manager) = parent
        .with_downcast::<Artboard, _>(Artboard::semantic_manager_handle)
        .flatten()
    else {
        return;
    };
    let parent_node =
        SemanticData::find_closest_semantic_node_handle(Some(host.clone())).or_else(|| {
            parent
                .with_downcast::<Artboard, _>(Artboard::semantic_boundary_node)
                .flatten()
        });
    instance.cleanup_semantic_tree();
    instance.build_semantic_tree(Some(manager), parent_node);
}
