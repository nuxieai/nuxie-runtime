//! Atomic transport. The authored wrappers and the empty layout leaf belong to
//! their slot. All running state belongs to the single transported occurrence.
use super::values::{FlightValues, TypedText};
use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        animation::{
            nested_state_machine::NestedStateMachine,
            state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
        },
        artboard::ArtboardInstance,
        component_dirt::ComponentDirt,
        data_bind::{data_bind::DataBind, data_context::DataContext},
        generated::{core_registry::CoreRegistry, nested_animation_base::NestedAnimationBase},
        input::{focus_manager::RuntimeFocusManagerHandle, focus_node::FocusNodeRef},
        layout_component::{LayoutComponent, owning_layout},
        math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
        nested_artboard::NestedArtboard,
        semantic::semantic_data::SemanticData,
        viewmodel::viewmodel_instance::ViewModelInstance,
    },
};
use anyhow::{Result, ensure};
use std::collections::HashSet;

#[derive(Default)]
pub(crate) struct SlotState {
    pub size: Option<Vec2D>,
    pub bounds: Option<Aabb>,
    pub local_override: Option<Mat2D>,
    placeholder: Option<RuntimeArtboardInstanceHandle>,
    // The placeholder's root, read once at install, outside any borrow.
    placeholder_root: Option<CoreHandle>,
    source: Option<CoreHandle>,
    mounted: Option<CoreHandle>,
    listeners: Option<Vec<Vec<RuntimeStateMachineInstanceWeakHandle>>>,
    origin_home: Option<CoreHandle>,
    pub(super) away_instance: Option<CoreHandle>,
}
impl SlotState {
    fn allocate() -> Box<Self> {
        #[cfg(feature = "test-support")]
        crate::hosted_scene::bookkeeping_allocation(1, true);
        Box::new(Self::default())
    }
}
impl Drop for SlotState {
    fn drop(&mut self) {
        #[cfg(feature = "test-support")]
        crate::hosted_scene::bookkeeping_allocation(1, false);
    }
}
impl NestedArtboard {
    pub(crate) fn live_move_mut(&mut self) -> &mut SlotState {
        self.live_move.get_or_insert_with(SlotState::allocate)
    }
    pub(crate) fn live_move_layout_instance(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.artboard_instance_default().or_else(|| {
            self.live_move
                .as_ref()
                .and_then(|state| state.placeholder.clone())
        })
    }
    /// The vacated slot's layout leaf, by its root handle. Unlike
    /// `live_move_layout_instance`, this borrows no artboard.
    pub(crate) fn live_move_placeholder_root(&self) -> Option<CoreHandle> {
        self.live_move
            .as_ref()
            .and_then(|state| state.placeholder_root.clone())
    }
}
pub(super) fn read<T>(host: &CoreHandle, f: impl FnOnce(&NestedArtboard) -> T) -> Result<T> {
    host.with(|o| o.as_nested_artboard().map(f))
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("expected a mounted NestedArtboard slot"))
}
pub(super) fn write<T>(host: &CoreHandle, f: impl FnOnce(&mut NestedArtboard) -> T) -> T {
    host.with_mut(|o| f(o.as_nested_artboard_mut().expect("validated slot")))
        .expect("live slot")
}
pub(super) fn child(host: &CoreHandle) -> Result<Option<RuntimeArtboardInstanceHandle>> {
    read(host, NestedArtboard::artboard_instance_default)
}
fn definition(host: &CoreHandle) -> Result<CoreHandle> {
    // An occupied slot's native child always wins over saved empty-slot state.
    if let Some(instance) = child(host)? {
        return instance
            .with_artboard(|b| b.artboard_source_handle())
            .ok_or_else(|| anyhow::anyhow!("slot has no unit definition"));
    }
    read(host, |s| {
        s.live_move.as_ref().and_then(|state| state.source.clone())
    })?
    .ok_or_else(|| anyhow::anyhow!("slot has no unit definition"))
}
/// Native bound replacement can change either the definition or only its prop
/// bindings. Compare occurrence identity, not just the definition's identity.
fn reconcile_native_child(host: &CoreHandle) {
    host.with_mut(|o| {
        let Some(slot) = o.as_nested_artboard_mut() else {
            return;
        };
        if slot.live_move.as_ref().is_some_and(|state| {
            state.mounted != slot.artboard_instance_default().map(|i| i.core_handle())
        }) {
            slot.live_move = None;
        }
    });
}
pub(crate) fn reconcile_slot(
    host: &CoreHandle,
    boards: &HashSet<CoreHandle>,
    hosts: &HashSet<CoreHandle>,
) {
    reconcile_native_child(host);
    host.with_mut(|o| {
        let Some(state) = o
            .as_nested_artboard_mut()
            .and_then(|s| s.live_move.as_mut())
        else {
            return;
        };
        if state
            .away_instance
            .as_ref()
            .is_some_and(|unit| !boards.contains(unit))
        {
            state.away_instance = None;
        }
        if state
            .origin_home
            .as_ref()
            .is_some_and(|slot| !hosts.contains(slot))
        {
            state.origin_home = None;
        }
    });
}
fn root(mut board: CoreHandle) -> Result<CoreHandle> {
    loop {
        let (host, parent) = board
            .with_downcast::<Artboard, _>(|b| (b.host(), b.parent_artboard()))
            .ok_or_else(|| anyhow::anyhow!("retired slot owner"))?;
        let Some(parent) = parent else {
            return Ok(board);
        };
        let host = host.ok_or_else(|| anyhow::anyhow!("unattached slot owner"))?;
        let mounted = host
            .with(|o| {
                o.as_artboard_host().is_some_and(|h| {
                    (0..h.artboard_count()).any(|i| {
                        h.artboard_instance(i as i32)
                            .is_some_and(|b| b.core_handle() == board)
                    })
                })
            })
            .unwrap_or(false);
        ensure!(mounted, "slot's row is no longer mounted");
        board = parent;
    }
}
fn wrappers(host: &CoreHandle) -> Result<Vec<CoreHandle>> {
    read(host, |s| s.nested_animations().to_vec())
}
pub fn validate_move(from: &CoreHandle, to: &CoreHandle) -> Result<()> {
    let scene = crate::hosted_scene::scene_for_operation(from)
        .ok_or_else(|| anyhow::anyhow!("file has no live-move records"))?;
    ensure!(
        crate::hosted_scene::is_live_slot(from) && crate::hosted_scene::is_live_slot(to),
        "live moves require slots in Page.screens Screen artboards"
    );
    scene.reconcile();
    ensure!(from != to, "source and destination are the same slot");
    let parent = |h| {
        read(h, NestedArtboard::parent_artboard_handle)?
            .ok_or_else(|| anyhow::anyhow!("slot has no parent"))
    };
    ensure!(
        root(parent(from)?)? == root(parent(to)?)?,
        "slots belong to different scene occurrences"
    );
    ensure!(
        definition(from)? == definition(to)?,
        "different unit artboards"
    );
    let (a, b) = (wrappers(from)?, wrappers(to)?);
    ensure!(a.len() == b.len(), "different wrapper counts");
    for (a, b) in a.iter().zip(&b) {
        ensure!(a.core_type() == b.core_type(), "different wrapper types");
        let key = i32::from(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY);
        ensure!(
            CoreRegistry::get_uint_handle(a, key) == CoreRegistry::get_uint_handle(b, key),
            "different animation ids"
        );
        let inputs = |h: &CoreHandle| {
            h.with_downcast::<NestedStateMachine, _>(|m| (0..m.input_count()).filter_map(|i| m.input(i)).map(|i| {
            let key = i32::from(crate::source::generated::animation::nested_input_base::NestedInputBase::INPUT_ID_PROPERTY_KEY);
            (i.core_type(), CoreRegistry::get_uint_handle(&i, key))
        }).collect::<Vec<_>>())
        };
        ensure!(inputs(a) == inputs(b), "different nested inputs");
        ensure!(
            inputs(a).is_some()
                || a.with(|o| o.as_nested_linear_animation().is_some())
                    .unwrap_or(false),
            "unsupported wrapper"
        );
    }
    Ok(())
}
pub(super) fn size(host: &CoreHandle, instance: &RuntimeArtboardInstanceHandle) -> Vec2D {
    let layout = host.is_type_of(
        crate::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY,
    );
    instance.with_artboard(|b| {
        if layout {
            Vec2D::new(b.layout_width(), b.layout_height())
        } else {
            Vec2D::new(b.width(), b.height())
        }
    })
}
fn wrapper_listeners(wrapper: &CoreHandle) -> Vec<RuntimeStateMachineInstanceWeakHandle> {
    if let Some(m) = wrapper
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
    {
        m.with_instance(|m| m.nested_event_listeners.clone())
    } else {
        wrapper
            .with(|o| {
                o.as_nested_linear_animation()
                    .and_then(|a| a.animation_instance())
                    .map(|a| a.nested_event_notifier.nested_event_listeners())
            })
            .flatten()
            .unwrap_or_default()
    }
}
fn remember(host: &CoreHandle) -> Result<()> {
    ensure!(
        crate::hosted_scene::is_live_slot(host),
        "live moves require a slot in a Page.screens Screen artboard"
    );
    reconcile_native_child(host);
    if read(host, |s| {
        s.live_move
            .as_ref()
            .is_none_or(|state| state.source.is_none())
    })? {
        let source = definition(host)?;
        let listeners = wrappers(host)?.iter().map(wrapper_listeners).collect();
        let mounted = child(host)?.map(|i| i.core_handle());
        write(host, |s| {
            s.live_move_mut().mounted = mounted;
            s.live_move_mut().source = Some(source);
            s.live_move_mut().listeners = Some(listeners);
        });
    }
    if let Some(instance) = child(host)? {
        let size = size(host, &instance);
        let leaf = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
        leaf.set_size(size.x, size.y);
        let (style, bounds, x, y, w, h, frame, ox, oy) = instance.with_artboard(|b| {
            (
                b.base.base.layout_data.style().clone(),
                b.layout_bounds(),
                b.layout_x(),
                b.layout_y(),
                b.layout_width(),
                b.layout_height(),
                b.frame_origin(),
                b.origin_x(),
                b.origin_y(),
            )
        });
        leaf.with_artboard_mut(|b| {
            b.set_frame_origin(frame);
            b.set_origin_x(ox);
            b.set_origin_y(oy);
            let mut style = style;
            style.size.width = taffy::prelude::Dimension::length(size.x);
            style.size.height = taffy::prelude::Dimension::length(size.y);
            style.min_size.width = taffy::prelude::LengthPercentageAuto::length(size.x);
            style.min_size.height = taffy::prelude::LengthPercentageAuto::length(size.y);
            style.max_size = style.min_size;
            style.flex_grow = 0.0;
            style.flex_shrink = 0.0;
            style.flex_basis = taffy::prelude::Dimension::auto();
            b.take_layout_data().set_style(style);
            b.set_layout(x, y, w, h);
        });
        let leaf_root = leaf.core_handle();
        write(host, |s| {
            s.live_move_mut().size = Some(size);
            s.live_move_mut().bounds = Some(bounds);
            s.live_move_mut().placeholder = Some(leaf);
            s.live_move_mut().placeholder_root = Some(leaf_root);
        });
    }
    Ok(())
}
fn discard(host: &CoreHandle) {
    register_loan(host, false);
    for wrapper in wrappers(host).expect("slot wrappers") {
        wrapper.with_mut(|o| o.nested_animation_release_dependencies());
    }
    if let Some(old) = write(host, |s| s.take_artboard_instance()) {
        let storage = crate::hosted_scene::scene_for(host).and_then(|scene| {
            scene
                .unit_vm_storage
                .borrow_mut()
                .remove(&old.core_handle())
        });
        // The native inline model owns its prop bindings after detachment.
        // Its identity stays in the slot even when its child leaves.
        detach_parent_props(host);
        old.cleanup_focus_tree();
        old.cleanup_semantic_tree();
        old.with_artboard_mut(|b| b.set_host_state(None, None, None));
        // Dispose the unit while its original VM storage still resolves.
        drop(old);
        drop(storage);
    }
    write(host, |s| {
        s.view_model_instance = None;
        s.active_view_model_instance = None;
        s.global_view_model_instances.clear();
        s.owns_active_vmi = false;
        s.mounted_bindable = None;
        s.live_move_mut().origin_home = None;
        s.live_move_mut().mounted = None;
        s.reset_mesh_raster();
        // The mount is gone, so this drops the stateful bind it scheduled.
        // Kept, it would rebuild the next transported unit's data context.
        s.flush_pending_stateful_binding();
    });
}
fn sync_layout(host: &CoreHandle) {
    if let Some(parent) = owning_layout(host.with(|o| o.component_parent_handle()).flatten()) {
        LayoutComponent::sync_layout_children_occurrence(&parent);
        LayoutComponent::mark_layout_style_dirty_occurrence(&parent);
    }
}
pub fn empty_slot(host: &CoreHandle) -> Result<()> {
    let scene = crate::hosted_scene::scene_for_operation(host)
        .ok_or_else(|| anyhow::anyhow!("file has no live-move records"))?;
    ensure!(
        crate::hosted_scene::is_live_slot(host),
        "live moves require a Screen slot"
    );
    scene.reconcile();
    remember(host)?;
    discard(host);
    write(host, |s| {
        s.artboard_referencer.set_referenced_artboard(None)
    });
    sync_layout(host);
    crate::hosted_scene::membership_changed_for(host);
    Ok(())
}

/// Validate before taking anything. Call between frames under the shared
/// occurrence guard. A stateful bind the source slot has scheduled runs
/// first, as its next advance would; otherwise no mount, machine
/// initialization, or user callback runs.
pub fn move_live(from: &CoreHandle, to: &CoreHandle) -> Result<()> {
    move_with_props(from, to, to, None)
}

pub(super) fn move_to_flight(
    from: &CoreHandle,
    to: &CoreHandle,
    landing: &CoreHandle,
    values: &mut FlightValues,
) -> Result<()> {
    move_with_props(from, to, landing, Some(values))
}

fn move_with_props(
    from: &CoreHandle,
    to: &CoreHandle,
    prop_slot: &CoreHandle,
    values: Option<&mut FlightValues>,
) -> Result<()> {
    validate_move(from, to)?;
    // A unit whose slot never advanced has not had the bind its mount
    // scheduled. It leaves bound, as that advance would have left it.
    write(from, NestedArtboard::flush_pending_stateful_binding);
    let instance = child(from)?.ok_or_else(|| anyhow::anyhow!("source slot is empty"))?;
    remember(from)?;
    remember(to)?;
    // Read the outgoing slot's current native props before detaching them.
    // Already transported props are also recognized by TypedText::capture.
    let bound = slot_props(from)?
        .iter()
        .map(|(id, _)| *id)
        .collect::<Vec<_>>();
    let typed = TypedText::capture(&instance, &bound);
    let parent = read(to, NestedArtboard::parent_artboard_handle)?.expect("validated parent");
    let scene = crate::hosted_scene::scene_for(from).expect("validated live scene");
    let owner = scene
        .unit_vm_storage
        .borrow()
        .get(&instance.core_handle())
        .cloned()
        .or_else(|| {
            instance
                .data_context()
                .and_then(|context| context.with_context(DataContext::main_view_model_instance))
                .and_then(|vm| vm.runtime_arena_owner())
        });
    let inherited = instance.data_context().is_some_and(|c| {
        read(from, |s| {
            s.data_context.as_ref().is_some_and(|p| p.ptr_eq(&c))
        })
        .unwrap_or(false)
    });
    let home = read(from, |s| {
        s.live_move
            .as_ref()
            .and_then(|state| state.origin_home.clone())
    })?
    .unwrap_or_else(|| from.clone());
    discard(to);
    detach_parent_props(from);
    for (a, b) in wrappers(from)?.iter().zip(wrappers(to)?) {
        if a.with_downcast::<NestedStateMachine, _>(|_| ()).is_some() {
            let playback = a
                .with_downcast_mut::<NestedStateMachine, _>(|a| a.instance.take())
                .flatten();
            b.with_downcast_mut::<NestedStateMachine, _>(|b| b.instance = playback);
        } else {
            let playback = a
                .with_mut(|o| {
                    o.as_nested_linear_animation_mut()
                        .unwrap()
                        .animation_instance
                        .take()
                })
                .flatten();
            b.with_mut(|o| {
                o.as_nested_linear_animation_mut()
                    .unwrap()
                    .animation_instance = playback
            });
        }
    }
    let (vm, active, globals, owns, seconds, scope, scope_manager, mounted) = write(from, |s| {
        s.take_artboard_instance();
        s.artboard_referencer.set_referenced_artboard(None);
        s.reset_mesh_raster();
        s.live_move_mut().local_override = None;
        s.live_move_mut().away_instance = Some(instance.core_handle());
        s.live_move_mut().mounted = None;
        (
            s.view_model_instance.take(),
            s.active_view_model_instance.take(),
            std::mem::take(&mut s.global_view_model_instances),
            std::mem::take(&mut s.owns_active_vmi),
            std::mem::take(&mut s.cumulated_seconds),
            s.focus_scope.take(),
            s.focus_scope_manager.take(),
            s.mounted_bindable.take(),
        )
    });
    let (old_scope, old_manager, context, mark, opacity, destination_size, leaf) = write(to, |s| {
        (
            s.focus_scope.take(),
            s.focus_scope_manager.take(),
            s.data_context.clone(),
            s.inactive_change_sequence.clone(),
            s.base.base.render_opacity(),
            s.live_move.as_ref().and_then(|state| state.size),
            s.live_move
                .as_ref()
                .and_then(|state| state.placeholder.clone()),
        )
    });
    if let (Some(scope), Some(manager)) = (old_scope, old_manager) {
        manager.with_focus_manager_mut(|m| m.remove_child(&scope));
    }
    reparent_focus(to, &instance, scope.clone(), scope_manager);
    if let Some(own) = instance.data_context() {
        if inherited {
            // An inherited Screen context cannot be reparented to itself. Keep
            // it untouched and give this context-less occurrence a local bridge.
            let bridge = crate::source::data_bind::data_context::RuntimeDataContextHandle::new(
                DataContext::new(None),
            );
            bridge.with_context_mut(|c| c.set_parent(context));
            instance.with_artboard(|b| b.data_bind_container.set_data_bind_context(Some(bridge)));
        } else {
            own.with_context_mut(|c| c.set_parent(context));
        }
    }
    if let Some(size) = destination_size {
        instance.set_size(size.x, size.y);
    }
    if let Some(leaf) = leaf {
        let (x, y, w, h) = leaf.with_artboard(|b| {
            (
                b.layout_x(),
                b.layout_y(),
                b.layout_width(),
                b.layout_height(),
            )
        });
        instance.with_artboard_mut(|b| b.set_layout(x, y, w, h));
    }
    instance.with_artboard_mut(|b| {
        b.set_frame_origin(false);
        b.set_host_opacity(opacity);
        b.set_host_state(Some(to.clone()), Some(parent.clone()), Some(mark));
        // Every move starts unstretched; a flight's pose stretches it again.
        b.set_flight_stretch(Vec2D::new(1.0, 1.0));
        // Every move starts with the unit's own corners; a flight's frames
        // set them again.
        b.set_flight_corners(None);
    });
    write(to, |s| {
        s.instance = Some(instance.clone());
        s.artboard_referencer
            .set_referenced_artboard(Some(instance.core_handle()));
        s.view_model_instance = vm;
        s.active_view_model_instance = active;
        s.global_view_model_instances = globals;
        s.owns_active_vmi = owns;
        s.cumulated_seconds = seconds;
        s.focus_scope = scope;
        s.focus_scope_manager = parent
            .with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
            .flatten();
        s.mounted_bindable = mounted;
        s.live_move_mut().away_instance = None;
        s.live_move_mut().mounted = Some(instance.core_handle());
        s.live_move_mut().origin_home = (home != *to).then_some(home);
        s.apply_origin_override();
        s.base.base.base.add_dirt(ComponentDirt::FILTHY, false);
    });
    // Release the old host's native sizing caches, then use ordinary
    // hosting's complete native/CSS setup. In flight, the follower sizes the
    // flight host's own box, so that setup is the only sizing route.
    LayoutComponent::release_hosted_item_sizing_occurrence(&instance.core_handle());
    if to
        .with_downcast_mut::<crate::source::nested_artboard_layout::NestedArtboardLayout, _>(
            |host| host.mark_layout_node_dirty(false),
        )
        .is_none()
    {
        // A plain host has no layout item. Clear it through the same
        // ownership setter used by StyleOverrider for record-free hosts.
        LayoutComponent::set_hosted_css_item_occurrence(&instance.core_handle(), None, None);
    }
    let owner = owner.filter(|board| {
        let owner = board.core_handle();
        owner != parent && owner != instance.core_handle() && !scene.is_root(&owner)
    });
    // Ownership is a scene lease, never a reference from one row to another.
    // Replacement and return release just this unit's storage, without history.
    let previous = if let Some(owner) = owner {
        scene
            .unit_vm_storage
            .borrow_mut()
            .insert(instance.core_handle(), owner)
    } else {
        scene
            .unit_vm_storage
            .borrow_mut()
            .remove(&instance.core_handle())
    };
    drop(previous);
    register_loan(from, false);
    register_loan(
        to,
        read(to, |s| {
            s.live_move
                .as_ref()
                .is_some_and(|state| state.origin_home.is_some())
        })?,
    );
    let listeners = read(to, |s| {
        s.live_move
            .as_ref()
            .and_then(|state| state.listeners.clone())
            .unwrap_or_default()
    })?;
    for (i, wrapper) in wrappers(to)?.iter().enumerate() {
        let listeners = listeners.get(i).cloned().unwrap_or_default();
        if let Some(machine) = wrapper
            .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
            .flatten()
        {
            machine.with_instance_mut(|m| {
                m.nested_event_listeners = listeners.clone();
                m.set_nested_artboard(to.clone());
                m.set_parent_nested_artboard(to.clone());
                m.set_parent_state_machine_instance(listeners.first().cloned().unwrap_or_default());
                if let Some(context) = instance.data_context()
                    && !m
                        .data_context()
                        .as_ref()
                        .is_some_and(|old| old.ptr_eq(&context))
                {
                    m.data_bind_container.set_data_bind_context(Some(context));
                }
            });
        } else {
            wrapper.with_mut(|o| {
                if let Some(a) = o
                    .as_nested_linear_animation_mut()
                    .and_then(|a| a.animation_instance_mut())
                {
                    for old in a.nested_event_notifier.nested_event_listeners() {
                        a.remove_nested_event_listener(old);
                    }
                    a.set_nested_artboard(to.clone());
                    for listener in listeners {
                        a.add_nested_event_listener(listener);
                    }
                }
            });
        }
    }
    if let Some(manager) = parent
        .with_downcast::<Artboard, _>(Artboard::semantic_manager_handle)
        .flatten()
    {
        let parent_node = SemanticData::find_closest_semantic_node_handle(Some(to.clone()))
            .or_else(|| {
                parent
                    .with_downcast::<Artboard, _>(Artboard::semantic_boundary_node)
                    .flatten()
            });
        let (previous, boundary) =
            instance.with_artboard(|b| (b.semantic_manager(), b.semantic_boundary_node()));
        if let Some(boundary) =
            boundary.filter(|_| previous.as_ref().is_some_and(|p| p.ptr_eq(&manager)))
        {
            manager.remove_child(&boundary);
            manager.add_child(parent_node, boundary);
            instance.with_artboard_mut(|b| b.mark_semantic_boundary_transform_dirty());
        } else {
            instance.cleanup_semantic_tree();
            instance.build_semantic_tree(Some(manager), parent_node);
        }
    }
    super::reconcile_slot_collapse(to);
    // Attach the landing slot's props directly, never the flight placeholder's.
    // Its native inline model remains the owner of the authored bindings.
    if prop_slot != to {
        let context = read(prop_slot, |s| s.data_context.clone())?;
        if let Some(own) = instance.data_context() {
            own.with_context_mut(|c| c.set_parent(context));
        }
    }
    adopt_props(prop_slot, &instance, values, &typed)?;
    sync_layout(from);
    sync_layout(to);
    // The instance may change layout trees. Publish its adopted dimensions
    // and CSS item style before the new tree takes its node.
    LayoutComponent::sync_style_occurrence(&instance.core_handle());
    crate::hosted_scene::membership_changed_for(from);
    Ok(())
}
fn slot_model(slot: &CoreHandle) -> Result<Option<CoreHandle>> {
    let model_id = definition(slot)?.with_downcast::<Artboard, _>(|b| b.base.view_model_id());
    read(slot, |s| {
        s.children()
            .iter()
            .find(|child| {
                child
                    .with(|o| {
                        o.as_view_model_instance()
                            .is_some_and(|v| Some(v.base.view_model_id()) == model_id)
                    })
                    .unwrap_or(false)
            })
            .cloned()
    })
}
fn detach_parent_props(from: &CoreHandle) {
    // Props belong to the slot's authored inline model, including when the
    // transported unit is using a shared or global model from another owner.
    let Some(model) = slot_model(from).ok().flatten() else {
        return;
    };
    let values = model
        .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
        .unwrap_or_default();
    if let Some(container) = read(from, NestedArtboard::parent_artboard_handle)
        .ok()
        .flatten()
        .and_then(|b| b.data_bind_container())
    {
        let binds = container.data_binds().to_vec();
        for bind in binds {
            if bind
                .with(|o| o.as_data_bind().and_then(|b| b.target()))
                .flatten()
                .is_some_and(|t| values.contains(&t))
            {
                container.remove_data_bind(bind.clone());
                DataBind::unbind_handle(&bind);
                // Transfer the original binding, including its owned converter,
                // to its native VM owner. No hidden copies or template lifetime.
                model.with_downcast_mut::<ViewModelInstance, _>(|v| {
                    if !v.value_data_binds().contains(&bind) {
                        // Do not rerun inline import's source-first normalization.
                        // Arrival derives bindings with this authored direction/order.
                        v.retain_existing_value_data_bind(bind);
                    }
                });
            }
        }
    }
}
fn slot_props(slot: &CoreHandle) -> Result<Vec<(u32, CoreHandle)>> {
    let parent = read(slot, NestedArtboard::parent_artboard_handle)?.expect("slot row");
    let model = slot_model(slot)?;
    let Some(model) = model else {
        return Ok(Vec::new());
    };
    let values = model
        .with_downcast::<ViewModelInstance, _>(|v| v.property_values().to_vec())
        .unwrap_or_default();
    let mut binds = parent
        .with_downcast::<Artboard, _>(Artboard::data_bind_handles)
        .unwrap_or_default();
    binds.extend(
        model
            .with_downcast::<ViewModelInstance, _>(|v| v.value_data_binds().to_vec())
            .unwrap_or_default(),
    );
    let mut seen = HashSet::new();
    Ok(binds
        .into_iter()
        .filter_map(|bind| {
            let target = bind
                .with(|o| {
                    o.as_data_bind()
                        .filter(|b| !b.is_instance_value_bind())
                        .and_then(DataBind::target)
                })
                .flatten()?;
            if !values.contains(&target) {
                return None;
            }
            let id = target
                .with(|o| {
                    o.as_view_model_instance_value()
                        .map(|v| v.view_model_property_id())
                })
                .flatten()?;
            seen.insert(id).then_some((id, bind))
        })
        .collect())
}
fn adopt_props(
    slot: &CoreHandle,
    instance: &RuntimeArtboardInstanceHandle,
    mut values: Option<&mut FlightValues>,
    typed: &TypedText,
) -> Result<()> {
    let props = slot_props(slot)?;
    let Some(vm) = instance
        .data_context()
        .and_then(|c| c.with_context(DataContext::main_view_model_instance))
    else {
        return Ok(());
    };
    let container = instance
        .core_handle()
        .data_bind_container()
        .expect("unit bind container");
    let binds = container.data_binds().to_vec();
    for bind in binds {
        if bind
            .with(|o| {
                o.as_data_bind()
                    .is_some_and(DataBind::is_instance_value_bind)
            })
            .unwrap_or(false)
        {
            container.remove_and_delete_data_bind(&bind);
        }
    }
    let arena = instance.core_handle().retain_arena().expect("unit arena");
    let mut attached = Vec::new();
    for (id, template) in props {
        let target = vm
            .with_downcast::<ViewModelInstance, _>(|v| {
                v.property_values()
                    .iter()
                    .find(|p| {
                        p.with(|o| {
                            o.as_view_model_instance_value()
                                .is_some_and(|v| v.view_model_property_id() == id)
                        })
                        .unwrap_or(false)
                    })
                    .cloned()
            })
            .flatten();
        if let Some(target) = target
            && let Some(bind) =
                DataBind::clone_with_target_into(&template, Some(target.clone()), &arena)
        {
            bind.with_mut(|o| o.as_data_bind_mut().unwrap().mark_instance_value_bind());
            if let Some(values) = values.as_deref_mut() {
                values.attach(&bind, &target);
            }
            attached.push((id, bind, target));
        }
    }
    // Capture every start before switched props or text notify dependencies.
    for (id, bind, target) in attached {
        typed.write_back(id, &bind, &target, instance);
        // Never write a previous Screen's presentation value back on arrival.
        bind.with_mut(|o| o.as_data_bind_mut().unwrap().require_source_adoption());
        container.add_data_bind(bind);
    }
    Ok(())
}
fn reparent_focus(
    to: &CoreHandle,
    instance: &RuntimeArtboardInstanceHandle,
    scope: Option<FocusNodeRef>,
    previous: Option<RuntimeFocusManagerHandle>,
) {
    use crate::source::focus_data::FocusData;
    let manager = read(to, NestedArtboard::parent_artboard_handle)
        .ok()
        .flatten()
        .and_then(|b| {
            b.with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
                .flatten()
        });
    let (Some(scope), Some(manager)) = (scope, manager) else {
        return;
    };
    let focused = previous
        .as_ref()
        .and_then(|m| m.with_focus_manager(|m| m.primary_focus()));
    let inside = focused.as_ref().is_some_and(|leaf| {
        let mut current = Some(leaf.clone());
        while let Some(node) = current {
            if std::rc::Rc::ptr_eq(&node, &scope) {
                return true;
            }
            current = node.borrow().parent();
        }
        false
    });
    if inside {
        let mut current = scope.borrow().parent();
        while let Some(node) = current {
            node.borrow_mut().set_has_focus(false);
            current = node.borrow().parent();
        }
    }
    if let Some(old) = previous.filter(|m| !m.ptr_eq(&manager)) {
        if inside {
            old.with_focus_manager_mut(|m| {
                m.primary_focus.take();
            });
            manager.with_focus_manager_mut(|m| {
                m.clear_focus();
                m.primary_focus = focused;
            });
        }
        old.with_focus_manager_mut(|m| m.detach_child(&scope));
    }
    let parent = FocusData::find_closest_focus_node_from_parent(
        to.with(|o| o.component_parent_handle()).flatten(),
    );
    manager.with_focus_manager_mut(|m| m.add_child(parent, scope.clone(), None));
    if inside {
        let mut current = scope.borrow().parent();
        while let Some(node) = current {
            node.borrow_mut().set_has_focus(true);
            current = node.borrow().parent();
        }
    }
    instance.with_artboard_mut(|b| b.set_active_focus_manager(Some(manager)));
}

fn containing_list(slot: &CoreHandle) -> Option<CoreHandle> {
    let board = read(slot, NestedArtboard::parent_artboard_handle)
        .ok()
        .flatten()?;
    let host = board
        .with_downcast::<Artboard, _>(Artboard::host)
        .flatten()?;
    crate::hosted_scene::is_screen_list(&host).then_some(host)
}
fn register_loan(slot: &CoreHandle, occupied: bool) {
    let Some(scene) = crate::hosted_scene::scene_for(slot) else {
        return;
    };
    let Some(list) = containing_list(slot) else {
        return;
    };
    {
        let mut loans = scene.loans.borrow_mut();
        loans.retain(|owner, holders| {
            holders.retain(CoreHandle::is_alive);
            owner.is_alive() && !holders.is_empty()
        });
        if occupied {
            loans.entry(list).or_default().insert(slot.clone());
        } else if let Some(holders) = loans.get_mut(&list) {
            holders.remove(slot);
            if holders.is_empty() {
                loans.remove(&list);
            }
        }
    }
}

/// Structural unmount hook. A disappearing row first returns every borrowed
/// occurrence whose original home is still mounted in this list.
pub(crate) fn return_row_loans(owner: &CoreHandle, next: &[CoreHandle]) -> Result<()> {
    let Some(scene) = crate::hosted_scene::scene_for(owner) else {
        return Ok(());
    };
    if !scene
        .loans
        .borrow()
        .get(owner)
        .is_some_and(|holders| holders.iter().any(CoreHandle::is_alive))
    {
        return Ok(());
    }
    let next_members = next.iter().cloned().collect::<HashSet<_>>();
    let returns = owner
        .with_downcast::<crate::source::artboard_component_list::ArtboardComponentList, _>(
            |list| -> Result<Vec<_>> {
                if list.artboard_count() == next.len()
                    && next
                        .iter()
                        .enumerate()
                        .all(|(i, item)| list.list_item(i as i32).as_ref() == Some(item))
                {
                    return Ok(Vec::new());
                }
                let mut homes = Vec::new();
                let mut leaving = Vec::new();
                for i in 0..list.artboard_count() {
                    if let Some(row) = list.artboard_instance(i as i32) {
                        #[cfg(feature = "test-support")]
                        crate::hosted_scene::record_walk(crate::hosted_scene::Walk::LoanRow);
                        if list
                            .list_item(i as i32)
                            .is_some_and(|item| next_members.contains(&item))
                        {
                            let flight_nodes = row
                                .with_artboard(|b| b.find_all_handles::<super::records::LiveSlot>())
                                .iter()
                                .filter_map(|r| {
                                    r.with_downcast::<super::records::LiveSlot, _>(|r| {
                                        (r.values.role != 0).then(|| r.parent()).flatten()
                                    })
                                    .flatten()
                                })
                                .collect::<HashSet<_>>();
                            for home in row.with_artboard(|b| b.nested_artboards().to_vec()) {
                                if !flight_nodes.contains(&home)
                                    && child(&home)?.is_none()
                                    && let Some(id) = read(&home, |s| {
                                        s.live_move
                                            .as_ref()
                                            .and_then(|state| state.away_instance.clone())
                                    })?
                                {
                                    homes.push((id, home));
                                }
                            }
                        } else {
                            leaving.push(row);
                        }
                    }
                }
                let mut returns = Vec::new();
                for row in leaving {
                    for holder in row.with_artboard(|b| b.nested_artboards().to_vec()) {
                        let Some(unit) = child(&holder)? else {
                            continue;
                        };
                        let origin = read(&holder, |s| {
                            s.live_move
                                .as_ref()
                                .and_then(|state| state.origin_home.clone())
                        })?;
                        let destination = homes
                            .iter()
                            .filter(|(id, _)| *id == unit.core_handle())
                            .min_by_key(|(_, home)| origin.as_ref() != Some(home));
                        if let Some((_, home)) = destination {
                            returns.push((holder, home.clone()));
                        }
                    }
                }
                Ok(returns)
            },
        )
        .ok_or_else(|| anyhow::anyhow!("not a list"))??;
    // Release the list's borrow before any move can resolve host membership,
    // repair semantic/focus ancestry, or sync a layout owned by that list.
    for (holder, home) in &returns {
        validate_move(holder, home)?;
    }
    for (holder, home) in returns {
        move_live(&holder, &home)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "transport_tests.rs"]
mod tests;
