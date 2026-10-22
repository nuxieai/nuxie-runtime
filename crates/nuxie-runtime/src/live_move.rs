//! Nuxie live nested-instance moves. No source animation or arena object moves.
//!
//! Call through one player between frames. `after_advance` is an update-only
//! second pass: it never advances a machine or consumes elapsed time.
use std::collections::{HashMap, HashSet};

use crate::{
    Artboard, CoreHandle, RuntimeArtboardInstanceHandle,
    source::{
        animation::{
            nested_state_machine::NestedStateMachine,
            state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
        },
        artboard::ArtboardInstance,
        artboard_component_list::ArtboardComponentList,
        component_dirt::ComponentDirt,
        data_bind::data_context::{DataContext, RuntimeDataContextHandle},
        focus_data::FocusData,
        generated::{
            component_base::ComponentBase, core_registry::CoreRegistry,
            nested_animation_base::NestedAnimationBase, node_base::NodeBase,
            transform_component_base::TransformComponentBase,
            world_transform_component_base::WorldTransformComponentBase,
        },
        input::{focus_manager::RuntimeFocusManagerHandle, focus_node::FocusNodeRef},
        layout_component::{LayoutComponent, owning_layout},
        math::{aabb::Aabb, mat2d::Mat2D, vec2d::Vec2D},
        nested_artboard::NestedArtboard,
        semantic::semantic_data::SemanticData,
    },
};
use anyhow::{Result, bail, ensure};
use serde::Deserialize;

/// State exists only on slots touched by the Nuxie API. Ordinary Rive hosts
/// keep the upstream empty-host and mount behaviour.
#[derive(Default)]
pub(crate) struct SlotState {
    pub size: Option<Vec2D>,
    pub bounds: Option<Aabb>,
    placeholder: Option<RuntimeArtboardInstanceHandle>,
    definition: Option<CoreHandle>,
    listeners: Vec<Vec<RuntimeStateMachineInstanceWeakHandle>>,
    props: Vec<(u32, CoreHandle)>,
    local_override: Option<Mat2D>,
}

impl NestedArtboard {
    pub(crate) fn live_move_layout_instance(&self) -> Option<RuntimeArtboardInstanceHandle> {
        self.artboard_instance_default()
            .or_else(|| self.live_move.placeholder.clone())
    }
}

fn slot<T>(host: &CoreHandle, f: impl FnOnce(&NestedArtboard) -> T) -> Result<T> {
    host.with(|o| o.as_nested_artboard().map(f))
        .flatten()
        .ok_or_else(|| anyhow::anyhow!("expected a live NestedArtboard slot"))
}
fn slot_mut<T>(host: &CoreHandle, f: impl FnOnce(&mut NestedArtboard) -> T) -> T {
    host.with_mut(|o| f(o.as_nested_artboard_mut().expect("validated slot")))
        .expect("retained live slot")
}
fn top(mut board: CoreHandle) -> CoreHandle {
    while let Some(parent) = board
        .with_downcast::<Artboard, _>(Artboard::parent_artboard)
        .flatten()
    {
        board = parent;
    }
    board
}
fn definition(host: &CoreHandle) -> Result<CoreHandle> {
    slot(host, |s| {
        s.live_move.definition.clone().or_else(|| {
            s.source_artboard().and_then(|b| {
                b.with_downcast::<Artboard, _>(Artboard::artboard_source_handle)
                    .flatten()
            })
        })
    })?
    .ok_or_else(|| anyhow::anyhow!("slot has no unit definition"))
}
fn wrappers(host: &CoreHandle) -> Result<Vec<CoreHandle>> {
    slot(host, |s| s.nested_animations().to_vec())
}

/// Validate the entire transport before touching either host.
pub fn validate_move(from: &CoreHandle, to: &CoreHandle) -> Result<()> {
    ensure!(from != to, "source and destination must differ");
    let a = slot(from, NestedArtboard::parent_artboard_handle)?
        .ok_or_else(|| anyhow::anyhow!("source has no player"))?;
    let b = slot(to, NestedArtboard::parent_artboard_handle)?
        .ok_or_else(|| anyhow::anyhow!("destination has no player"))?;
    ensure!(top(a) == top(b), "slots belong to different players");
    ensure!(
        definition(from)? == definition(to)?,
        "slots reference different unit artboards"
    );
    let a = wrappers(from)?;
    let b = wrappers(to)?;
    ensure!(a.len() == b.len(), "nested animation wrapper counts differ");
    for (a, b) in a.iter().zip(&b) {
        ensure!(
            a.core_type() == b.core_type(),
            "nested animation wrapper types differ"
        );
        let key = i32::from(NestedAnimationBase::ANIMATION_ID_PROPERTY_KEY);
        ensure!(
            CoreRegistry::get_uint_handle(a, key) == CoreRegistry::get_uint_handle(b, key),
            "nested animation wrapper ids differ"
        );
        ensure!(
            a.with_downcast::<NestedStateMachine, _>(|_| ()).is_some()
                || a.with(|o| o.as_nested_linear_animation().is_some())
                    .unwrap_or(false),
            "unsupported nested animation wrapper"
        );
        if let Some(ai) = a.with_downcast::<NestedStateMachine, _>(|m| {
            (0..m.input_count())
                .filter_map(|i| m.input(i))
                .collect::<Vec<_>>()
        }) {
            let bi = b
                .with_downcast::<NestedStateMachine, _>(|m| {
                    (0..m.input_count())
                        .filter_map(|i| m.input(i))
                        .collect::<Vec<_>>()
                })
                .unwrap();
            ensure!(ai.len() == bi.len(), "nested machine input counts differ");
            for (a, b) in ai.iter().zip(&bi) {
                let key = i32::from(crate::source::generated::animation::nested_input_base::NestedInputBase::INPUT_ID_PROPERTY_KEY);
                ensure!(
                    a.core_type() == b.core_type()
                        && CoreRegistry::get_uint_handle(a, key)
                            == CoreRegistry::get_uint_handle(b, key),
                    "nested machine input structure differs"
                );
            }
        }
    }
    Ok(())
}

fn listeners(wrapper: &CoreHandle) -> Vec<RuntimeStateMachineInstanceWeakHandle> {
    if let Some(machine) = wrapper
        .with_downcast::<NestedStateMachine, _>(NestedStateMachine::state_machine_instance)
        .flatten()
    {
        return machine.with_instance(|m| m.nested_event_listeners.clone());
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

fn instance_size(host: &CoreHandle, instance: &RuntimeArtboardInstanceHandle) -> Vec2D {
    let layout_host = host.is_type_of(
        crate::source::generated::nested_artboard_layout_base::NestedArtboardLayoutBase::TYPE_KEY,
    );
    instance.with_artboard(|b| {
        if layout_host && b.layout_width().is_finite() && b.layout_height().is_finite() {
            Vec2D::new(b.layout_width(), b.layout_height())
        } else {
            Vec2D::new(b.width(), b.height())
        }
    })
}

fn remember_slot(host: &CoreHandle) -> Result<()> {
    let def = definition(host)?;
    let ws = wrappers(host)?;
    let saved_listeners = ws.iter().map(listeners).collect();
    let instance = slot(host, NestedArtboard::artboard_instance_default)?;
    let first = slot(host, |s| s.live_move.definition.is_none())?;
    let mut props = Vec::new();
    if first && let Some(instance) = &instance {
        use crate::source::{
            data_bind::data_bind::DataBind, viewmodel::viewmodel_instance::ViewModelInstance,
        };
        let vm = instance
            .data_context()
            .and_then(|c| c.with_context(DataContext::main_view_model_instance));
        let values = vm
            .as_ref()
            .and_then(|vm| {
                vm.with_downcast::<ViewModelInstance, _>(|vm| vm.property_values().to_vec())
            })
            .unwrap_or_default();
        // Explicit occurrence VMs live in their Screen arena. Their prop binds
        // belong to that Screen's container, rather than the child's container.
        let mut candidates = slot(host, NestedArtboard::parent_artboard_handle)?
            .and_then(|b| b.with_downcast::<Artboard, _>(Artboard::data_bind_handles))
            .unwrap_or_default();
        if let Some(vm) = vm {
            candidates.extend(
                vm.with_downcast::<ViewModelInstance, _>(|vm| vm.value_data_binds().to_vec())
                    .unwrap_or_default(),
            );
        }
        candidates.extend(instance.with_artboard(|b| b.data_bind_handles()));
        let mut seen = HashSet::new();
        for bind in candidates {
            let target = bind
                .with(|o| o.as_data_bind().and_then(|b| b.target()))
                .flatten();
            if let Some(target) = target.filter(|t| values.contains(t)) {
                let id = target
                    .with(|o| {
                        o.as_view_model_instance_value()
                            .unwrap()
                            .view_model_property_id()
                    })
                    .unwrap();
                if !seen.insert(id) {
                    continue;
                }
                let arena = host.retain_arena().expect("slot arena");
                if let Some(template) = DataBind::clone_with_target_into(&bind, None, &arena) {
                    props.push((id, template));
                }
            }
        }
    }
    slot_mut(host, |s| {
        if s.live_move.definition.is_none() {
            s.live_move.definition = Some(def);
            s.live_move.listeners = saved_listeners;
            s.live_move.props = props;
        }
    });
    if let Some(instance) = instance {
        let size = instance_size(host, &instance);
        let (bounds, layout) = instance.with_artboard(|b| {
            (
                b.layout_bounds(),
                (
                    b.layout_x(),
                    b.layout_y(),
                    b.layout_width(),
                    b.layout_height(),
                ),
            )
        });
        let (frozen_style, frame_origin, origin_x, origin_y) = instance.with_artboard(|b| {
            (
                b.base.base.layout_data.style().clone(),
                b.frame_origin(),
                b.origin_x(),
                b.origin_y(),
            )
        });
        // A tiny, object-free layout owner replaces the moved root layout node.
        // It never participates in draw, advance, VM enumeration or hit tests.
        let placeholder = RuntimeArtboardInstanceHandle::new(ArtboardInstance::default());
        placeholder.set_size(size.x, size.y);
        placeholder.with_artboard_mut(|b| {
            // A frozen leaf has no authored style applier. Give its solver node
            // the exact retained box instead of the default auto/zero style.
            b.set_frame_origin(frame_origin);
            b.set_origin_x(origin_x);
            b.set_origin_y(origin_y);
            let data = b.take_layout_data();
            let mut style = frozen_style;
            style.size.width = taffy::prelude::Dimension::length(size.x);
            style.size.height = taffy::prelude::Dimension::length(size.y);
            style.flex_shrink = 0.0;
            style.flex_grow = 0.0;
            style.flex_basis = taffy::prelude::Dimension::auto();
            style.min_size.width = taffy::prelude::LengthPercentageAuto::length(size.x);
            style.min_size.height = taffy::prelude::LengthPercentageAuto::length(size.y);
            style.max_size = style.min_size;
            data.set_style(style);
            b.set_layout(layout.0, layout.1, layout.2, layout.3);
        });
        slot_mut(host, |s| {
            s.live_move.size = Some(size);
            s.live_move.bounds = Some(bounds);
            s.live_move.placeholder = Some(placeholder);
        });
    }
    Ok(())
}

/// Discard a fresh flight instance while retaining its wrappers and layout.
/// Useful when the file format mounts a unit by default in every slot.
pub fn empty_slot(host: &CoreHandle) -> Result<()> {
    remember_slot(host)?;
    discard_instance(host);
    slot_mut(host, |s| {
        s.artboard_referencer.set_referenced_artboard(None);
    });
    sync_layout(host);
    Ok(())
}

fn discard_instance(host: &CoreHandle) {
    // Release wrapper dependencies first, exactly as NestedArtboard::drop does.
    for wrapper in wrappers(host).expect("slot") {
        wrapper.with_mut(|o| o.nested_animation_release_dependencies());
    }
    if let Some(instance) = slot_mut(host, |s| s.take_artboard_instance()) {
        instance.cleanup_focus_tree();
        instance.cleanup_semantic_tree();
        instance.with_artboard_mut(|b| {
            b.set_host_state(None, None, None);
        });
    }
    slot_mut(host, |s| {
        s.view_model_instance = None;
        s.active_view_model_instance = None;
        s.global_view_model_instances.clear();
        s.owns_active_vmi = false;
        s.reset_mesh_raster();
    });
}

fn sync_layout(host: &CoreHandle) {
    let parent = host
        .with(|o| o.as_component().and_then(|c| c.parent_handle()))
        .flatten();
    if let Some(layout) = owning_layout(parent) {
        LayoutComponent::sync_layout_children_occurrence(&layout);
        LayoutComponent::mark_layout_style_dirty_occurrence(&layout);
    }
}

/// Atomically transport a child graph and its running playback to another slot.
/// No callback that draws, advances or enumerates media runs within the move.
pub fn move_instance(from: &CoreHandle, to: &CoreHandle) -> Result<()> {
    validate_move(from, to)?;
    ensure!(
        slot(from, NestedArtboard::artboard_instance_default)?.is_some(),
        "source slot is empty"
    );
    remember_slot(from)?;
    remember_slot(to)?;
    let a = wrappers(from)?;
    let b = wrappers(to)?;
    let destination_layout = slot(to, |s| s.live_move.placeholder.clone())?;
    discard_instance(to);
    detach_parent_props(from);
    // Wrapper objects stay in their authored arenas. Only playback moves.
    for (a, b) in a.iter().zip(&b) {
        if a.with_downcast::<NestedStateMachine, _>(|_| ()).is_some() {
            let machine = a
                .with_downcast_mut::<NestedStateMachine, _>(|a| a.instance.take())
                .flatten();
            b.with_downcast_mut::<NestedStateMachine, _>(|b| b.instance = machine);
        } else {
            let animation = a
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
                    .animation_instance = animation
            });
        }
    }
    let (instance, vm, active, globals, owns, seconds, scope, scope_manager, mounted) =
        slot_mut(from, |s| {
            s.artboard_referencer.set_referenced_artboard(None);
            s.reset_mesh_raster();
            s.live_move.local_override = None;
            (
                s.take_artboard_instance().unwrap(),
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
    // Dispose the destination structural scope, then reparent the source scope.
    let (old_scope, old_manager, parent_context, mark, parent_board, host_opacity) =
        slot_mut(to, |s| {
            (
                s.focus_scope.take(),
                s.focus_scope_manager.take(),
                s.data_context.clone(),
                s.inactive_change_sequence.clone(),
                s.parent_artboard_handle().unwrap(),
                s.base.base.render_opacity(),
            )
        });
    if let (Some(scope), Some(manager)) = (old_scope, old_manager) {
        manager.with_focus_manager_mut(|m| m.remove_child(&scope));
    }
    rehome_focus(to, &instance, scope.clone(), scope_manager);
    if let Some(context) = instance.data_context() {
        // Own contexts retain VM identities and private values. Only the parent
        // link changes. Never call clear_data_context or bind_stateful here.
        let inherited = slot(from, |s| {
            s.data_context.as_ref().is_some_and(|p| p.ptr_eq(&context))
        })?;
        if inherited {
            let context = RuntimeDataContextHandle::new(DataContext::new(None));
            context.with_context_mut(|c| c.set_parent(parent_context.clone()));
            instance.with_artboard(|b| b.data_bind_container.set_data_bind_context(Some(context)));
        } else {
            context.with_context_mut(|c| c.set_parent(parent_context.clone()));
        }
    }
    if let Some(layout) = destination_layout {
        let (x, y, w, h) = layout.with_artboard(|b| {
            (
                b.layout_x(),
                b.layout_y(),
                b.layout_width(),
                b.layout_height(),
            )
        });
        let size = slot(to, |s| s.live_move.size)?.unwrap_or(Vec2D::new(w, h));
        instance.set_size(size.x, size.y);
        instance.with_artboard_mut(|b| b.set_layout(x, y, w, h));
    }
    instance.with_artboard_mut(|i| {
        i.set_frame_origin(false);
        i.set_host_opacity(host_opacity);
        i.set_host_state(Some(to.clone()), Some(parent_board.clone()), Some(mark));
    });
    slot_mut(to, |s| {
        s.instance = Some(instance.clone());
        s.artboard_referencer
            .set_referenced_artboard(Some(instance.core_handle()));
        s.view_model_instance = vm;
        s.active_view_model_instance = active;
        s.global_view_model_instances = globals;
        s.owns_active_vmi = owns;
        s.cumulated_seconds = seconds;
        s.focus_scope = scope;
        s.focus_scope_manager = parent_board
            .with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
            .flatten();
        s.mounted_bindable = mounted;
        s.apply_origin_override();
        s.base.base.base.add_dirt(ComponentDirt::FILTHY, false);
    });
    let destination_listeners = slot(to, |s| s.live_move.listeners.clone())?;
    for (index, wrapper) in b.iter().enumerate() {
        let listeners = destination_listeners
            .get(index)
            .cloned()
            .unwrap_or_default();
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
                if let Some(animation) = o
                    .as_nested_linear_animation_mut()
                    .and_then(|a| a.animation_instance_mut())
                {
                    let old = animation.nested_event_notifier.nested_event_listeners();
                    for listener in old {
                        animation.remove_nested_event_listener(listener);
                    }
                    animation.set_nested_artboard(to.clone());
                    for listener in listeners {
                        animation.add_nested_event_listener(listener);
                    }
                }
            });
        }
    }
    if let Some(manager) = parent_board
        .with_downcast::<Artboard, _>(Artboard::semantic_manager_handle)
        .flatten()
    {
        let parent_node = SemanticData::find_closest_semantic_node_handle(Some(to.clone()))
            .or_else(|| {
                // Match the normal mount path when no slot ancestor has
                // SemanticData. The unit still belongs to its Screen boundary.
                parent_board
                    .with_downcast::<Artboard, _>(Artboard::semantic_boundary_node)
                    .flatten()
            });
        let (previous_manager, boundary) =
            instance.with_artboard(|b| (b.semantic_manager(), b.semantic_boundary_node()));
        if let Some(boundary) = boundary.filter(|_| {
            previous_manager
                .as_ref()
                .is_some_and(|previous| previous.ptr_eq(&manager))
        }) {
            // A live move within one player keeps the whole semantic subtree.
            // Building it again appends the same nodes to both hosts and to
            // their existing parents. Detach only the boundary, then attach
            // it once. Descendant registrations and SDK field ids stay intact.
            manager.remove_child(&boundary);
            manager.add_child(parent_node, boundary);
            instance.with_artboard_mut(|b| b.mark_semantic_boundary_transform_dirty());
        } else {
            // First semantic mount, or a separately enabled nested manager.
            // Use the ordinary cleanup/build path when registrations cannot
            // stay in the destination manager.
            instance.cleanup_semantic_tree();
            instance.build_semantic_tree(Some(manager), parent_node);
        }
    }
    adopt_props(to, &instance);
    // add_data_bind binds only the newly adopted prop templates. Do not
    // rebind existing unit binds: scripted converter binding calls Lua reinit.
    sync_layout(from);
    sync_layout(to);
    Ok(())
}

fn detach_parent_props(host: &CoreHandle) {
    use crate::source::{
        data_bind::data_bind::DataBind, viewmodel::viewmodel_instance::ViewModelInstance,
    };
    let instance = slot(host, NestedArtboard::artboard_instance_default)
        .expect("slot")
        .expect("source");
    let values = instance
        .data_context()
        .and_then(|c| c.with_context(DataContext::main_view_model_instance))
        .and_then(|vm| vm.with_downcast::<ViewModelInstance, _>(|vm| vm.property_values().to_vec()))
        .unwrap_or_default();
    if let Some(container) = slot(host, NestedArtboard::parent_artboard_handle)
        .expect("slot")
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
            }
        }
    }
}

fn adopt_props(host: &CoreHandle, instance: &RuntimeArtboardInstanceHandle) {
    use crate::source::{
        data_bind::data_bind::DataBind, viewmodel::viewmodel_instance::ViewModelInstance,
    };
    let Some(vm) = instance
        .data_context()
        .and_then(|c| c.with_context(DataContext::main_view_model_instance))
    else {
        return;
    };
    let container = instance
        .core_handle()
        .data_bind_container()
        .expect("unit binds");
    // Only occurrence-prop bindings change. Unbound private values and graph
    // bindings stay on the same objects. Saved templates also serve empty homes.
    let old = container.data_binds().to_vec();
    for bind in old {
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
    let templates = slot(host, |s| s.live_move.props.clone()).expect("slot");
    let arena = instance.core_handle().retain_arena().expect("unit arena");
    for (id, template) in templates {
        let target = vm
            .with_downcast::<ViewModelInstance, _>(|vm| {
                vm.property_values()
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
        if let Some(bind) = DataBind::clone_with_target_into(&template, target, &arena) {
            bind.with_mut(|o| o.as_data_bind_mut().unwrap().mark_instance_value_bind());
            container.add_data_bind(bind);
        }
    }
}

fn rehome_focus(
    to: &CoreHandle,
    instance: &RuntimeArtboardInstanceHandle,
    scope: Option<FocusNodeRef>,
    previous: Option<RuntimeFocusManagerHandle>,
) {
    let manager = slot(to, NestedArtboard::parent_artboard_handle)
        .ok()
        .flatten()
        .and_then(|b| {
            b.with_downcast::<Artboard, _>(Artboard::focus_manager_handle)
                .flatten()
        });
    if let (Some(scope), Some(manager)) = (scope, manager) {
        // A player's nested artboards normally share one manager. add_child
        // reparents without remove_child's blur. Keep the focused leaf intact.
        let parent = FocusData::find_closest_focus_node_from_parent(
            to.with(|o| o.as_component().and_then(|c| c.parent_handle()))
                .flatten(),
        );
        let focused = previous
            .as_ref()
            .and_then(|m| m.with_focus_manager(|m| m.primary_focus()));
        let focused_inside = focused.as_ref().is_some_and(|leaf| {
            let mut node = Some(leaf.clone());
            while let Some(current) = node {
                if std::rc::Rc::ptr_eq(&current, &scope) {
                    return true;
                }
                node = current.borrow().parent();
            }
            false
        });
        if focused_inside {
            let mut ancestor = scope.borrow().parent();
            while let Some(node) = ancestor {
                node.borrow_mut().set_has_focus(false);
                ancestor = node.borrow().parent();
            }
        }
        if let Some(previous) = previous.as_ref().filter(|p| !p.ptr_eq(&manager)) {
            if focused_inside {
                previous.with_focus_manager_mut(|m| {
                    m.primary_focus.take();
                });
                manager.with_focus_manager_mut(|m| {
                    m.clear_focus();
                    m.primary_focus = focused;
                });
            }
            previous.with_focus_manager_mut(|m| m.detach_child(&scope));
        }
        manager.with_focus_manager_mut(|m| m.add_child(parent, scope.clone(), None));
        if focused_inside {
            let mut ancestor = scope.borrow().parent();
            while let Some(node) = ancestor {
                node.borrow_mut().set_has_focus(true);
                ancestor = node.borrow().parent();
            }
        }
        instance.with_artboard_mut(|i| i.set_active_focus_manager(Some(manager)));
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SizeMode {
    #[default]
    Scale,
    Layout,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlotPath {
    pub screen: String,
    pub path: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnitMove {
    pub unit: String,
    pub from: SlotPath,
    pub to: SlotPath,
    pub flight: SlotPath,
    #[serde(default)]
    pub size: SizeMode,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    pub id: String,
    pub units: Vec<UnitMove>,
}
#[derive(Default, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Moves {
    pub transitions: Vec<Transition>,
}

pub struct Flight {
    pub flight: CoreHandle,
    pub from: CoreHandle,
    pub to: CoreHandle,
    pub size_mode: SizeMode,
    pub progress: f32,
    unit_size: Vec2D,
}
impl Flight {
    pub fn new(
        flight: CoreHandle,
        from: CoreHandle,
        to: CoreHandle,
        size_mode: SizeMode,
    ) -> Result<Self> {
        validate_move(&from, &flight)?;
        validate_move(&from, &to)?;
        let unit_size = slot(&from, |s| {
            s.artboard_instance_default()
                .map(|i| instance_size(&from, &i))
                .or(s.live_move.size)
                .or_else(|| s.natural_size_for_layout())
        })?
        .ok_or_else(|| anyhow::anyhow!("source has no size"))?;
        Ok(Self {
            flight,
            from,
            to,
            size_mode,
            progress: 0.0,
            unit_size,
        })
    }
}

/// Per-player state. Keeps homes by instance identity, never by unit name.
#[derive(Default)]
pub struct LiveMoves {
    moves: Moves,
    pub flights: Vec<Flight>,
    homes: Vec<(CoreHandle, CoreHandle)>,
    active_step: Option<(String, Option<String>)>,
    // Occurrence VMs can originate in a Screen arena. Retain those arenas in
    // the player controller, outside the scene graph, to avoid arena cycles.
    retained_vm_arenas: HashMap<usize, crate::source::core::CoreArena>,
    retained_vm_artboards: HashMap<CoreHandle, RuntimeArtboardInstanceHandle>,
    prepared_version: Option<(CoreHandle, u32)>,
}
impl LiveMoves {
    pub fn set_moves(&mut self, json: &str) -> Result<()> {
        ensure!(
            self.active_step.is_none(),
            "cannot replace moves during a step"
        );
        let moves: Moves = serde_json::from_str(json)?;
        let mut ids = HashSet::new();
        for t in &moves.transitions {
            ensure!(
                !t.id.is_empty() && ids.insert(&t.id),
                "duplicate or empty transition id"
            );
            let mut units = HashSet::new();
            for u in &t.units {
                ensure!(
                    !u.unit.is_empty() && units.insert(&u.unit),
                    "duplicate or empty unit id"
                );
                for p in [&u.from, &u.to, &u.flight] {
                    ensure!(
                        !p.screen.is_empty()
                            && !p.path.is_empty()
                            && p.path.split('/').all(|n| !n.is_empty()),
                        "empty slot path"
                    );
                }
            }
        }
        self.moves = moves;
        self.prepared_version = None;
        Ok(())
    }
    /// Empty compiler-emitted flight slots as soon as their rows mount.
    /// The player calls this before a frame can draw or enumerate media.
    pub fn prepare(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<()> {
        if self.moves.transitions.is_empty() {
            return Ok(());
        }
        let list = root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .ok_or_else(|| anyhow::anyhow!("root has no screens list"))?;
        let version = list
            .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::items_version)
            .unwrap();
        if self
            .prepared_version
            .as_ref()
            .is_some_and(|(l, v)| *l == list && *v == version)
        {
            return Ok(());
        }
        let rows = screens(root)?;
        let mut seen = HashSet::new();
        for t in &self.moves.transitions {
            for u in &t.units {
                if !rows.contains_key(&u.flight.screen) {
                    continue;
                }
                let host = resolve(&rows, &u.flight)?;
                if !seen.insert(host.clone()) || self.flights.iter().any(|f| f.flight == host) {
                    continue;
                }
                if let Some(instance) = slot(&host, NestedArtboard::artboard_instance_default)? {
                    ensure!(
                        !self
                            .homes
                            .iter()
                            .any(|(id, _)| *id == instance.core_handle()),
                        "inactive flight holds a transported unit"
                    );
                    empty_slot(&host)?;
                }
            }
        }
        self.prepared_version = Some((list, version));
        Ok(())
    }
    pub fn step_begin(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        transition: Option<&str>,
    ) -> Result<()> {
        ensure!(self.active_step.is_none(), "a step is already active");
        let rows = screens(root)?;
        ensure!(rows.contains_key(incoming), "incoming Screen not found");
        let transition = transition
            .map(|id| {
                self.moves
                    .transitions
                    .iter()
                    .find(|t| t.id == id)
                    .ok_or_else(|| anyhow::anyhow!("transition not found"))
            })
            .transpose()?;
        let mut plan = Vec::new();
        let mut used = HashSet::new();
        if let Some(t) = transition {
            for u in &t.units {
                ensure!(
                    u.to.screen == incoming,
                    "transition destination differs from incoming Screen"
                );
                let from = resolve(&rows, &u.from)?;
                let to = resolve(&rows, &u.to)?;
                let flight = resolve(&rows, &u.flight)?;
                for h in [&from, &to, &flight] {
                    ensure!(used.insert(h.clone()), "overlapping move slots");
                }
                let flight = Flight::new(flight, from, to, u.size)?;
                ensure!(
                    slot(&flight.from, NestedArtboard::artboard_instance_default)?.is_some(),
                    "flight source empty"
                );
                plan.push(flight);
            }
        }
        // Verify that every flight can be represented before transporting any
        // unit, including singular/transparent parents and non-finite controls.
        for f in &plan {
            follow(f, false)?;
        }
        // Plan all automatic returns before executing any transport.
        let incoming_board = rows.get(incoming).unwrap();
        let mut returns = Vec::new();
        for (instance, home) in &self.homes {
            if slot(home, NestedArtboard::parent_artboard_handle)?.as_ref()
                != Some(&incoming_board.core_handle())
                || slot(home, NestedArtboard::artboard_instance_default)?.is_some()
                || plan.iter().any(|f| &f.to == home)
            {
                continue;
            }
            if let Some(holder) = find_holder(root, instance) {
                validate_move(&holder, home)?;
                ensure!(
                    !used.contains(&holder) && !used.contains(home),
                    "automatic return overlaps a flight"
                );
                returns.push((holder, home.clone()));
            } else {
                bail!("home instance is no longer reachable in this player");
            }
        }
        for (holder, home) in returns {
            let instance = slot(&holder, NestedArtboard::artboard_instance_default)?.unwrap();
            if !self
                .homes
                .iter()
                .any(|(id, slot)| *id == instance.core_handle() && *slot == holder)
            {
                self.homes.push((instance.core_handle(), holder.clone()));
            }
            move_instance(&holder, &home)?;
        }
        for flight in plan {
            let instance = slot(&flight.from, NestedArtboard::artboard_instance_default)?.unwrap();
            if !self
                .homes
                .iter()
                .any(|(id, home)| *id == instance.core_handle() && *home == flight.from)
            {
                self.homes
                    .push((instance.core_handle(), flight.from.clone()));
            }
            if let Some(vm) = instance
                .data_context()
                .and_then(|c| c.with_context(DataContext::main_view_model_instance))
            {
                // Artboard::drop explicitly retires its objects even when the
                // arena is retained. Keep an explicit occurrence VM's owning
                // artboard too, outside the graph, until the player is dropped.
                if let Some(owner) = vm
                    .with(|o| o.as_component().and_then(|c| c.artboard_handle()))
                    .flatten()
                    && let Some(artboard) = owner.runtime_artboard_instance()
                {
                    self.retained_vm_artboards.entry(owner).or_insert(artboard);
                }
                if let Some(arena) = vm.retain_arena() {
                    self.retained_vm_arenas
                        .entry(vm.identity_key().0)
                        .or_insert(arena);
                }
            }
            move_instance(&flight.from, &flight.flight)?;
            self.flights.push(flight);
        }
        self.active_step = Some((incoming.to_owned(), transition.map(|t| t.id.clone())));
        self.after_advance(root)?;
        Ok(())
    }
    pub fn step_end(
        &mut self,
        root: &RuntimeArtboardInstanceHandle,
        incoming: &str,
        transition: Option<&str>,
    ) -> Result<()> {
        ensure!(
            self.active_step
                .as_ref()
                .is_some_and(|(s, t)| s == incoming && t.as_deref() == transition),
            "step_end does not match step_begin"
        );
        for f in &self.flights {
            validate_move(&f.flight, &f.to)?;
        }
        for f in &self.flights {
            move_instance(&f.flight, &f.to)?;
        }
        self.flights.clear();
        self.active_step = None;
        root.update_pass(true);
        Ok(())
    }
    /// Hook after the player's existing advance and layout, before drawing.
    pub fn after_advance(&mut self, root: &RuntimeArtboardInstanceHandle) -> Result<bool> {
        self.prepare(root)?;
        let mut updated = false;
        for flight in &self.flights {
            let board = slot(&flight.flight, NestedArtboard::parent_artboard_handle)?;
            if board
                .as_ref()
                .is_some_and(|board| !crate::hosted_scene::is_active_artboard(board))
            {
                continue;
            }
            follow(flight, true)?;
            updated = true;
            // A row can have settled before the post-advance flight write.
            // Update its component graph directly, never its playback.
            if let Some(board) = board {
                Artboard::update_pass_handle(&board, false);
                if let Some(instance) =
                    slot(&flight.flight, NestedArtboard::artboard_instance_default)?
                {
                    // Host transform dirt alone does not update its child's
                    // layout or paints. Settle the actual transported subtree.
                    instance.update_pass(false);
                    Artboard::update_pass_handle(&board, false);
                }
            }
        }
        if updated {
            root.update_pass(true);
        }
        Ok(updated)
    }
}

pub fn screens(
    root: &RuntimeArtboardInstanceHandle,
) -> Result<HashMap<String, RuntimeArtboardInstanceHandle>> {
    let list = root
        .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
        .ok_or_else(|| anyhow::anyhow!("root has no screens list"))?;
    list.with_downcast::<ArtboardComponentList, _>(|l| {
        let mut rows = HashMap::new();
        for i in 0..l.artboard_count() {
            if let Some(row) = l.artboard_instance(i as i32) {
                let name = row.with_artboard(|b| b.name().to_owned());
                ensure!(
                    rows.insert(name, row).is_none(),
                    "duplicate Screen artboard name"
                );
            }
        }
        Ok(rows)
    })
    .unwrap()
}
fn name(h: &CoreHandle) -> Option<String> {
    CoreRegistry::get_string_handle(h, i32::from(ComponentBase::NAME_PROPERTY_KEY))
}
fn resolve(
    rows: &HashMap<String, RuntimeArtboardInstanceHandle>,
    path: &SlotPath,
) -> Result<CoreHandle> {
    let board = rows
        .get(&path.screen)
        .ok_or_else(|| anyhow::anyhow!("Screen {} not found", path.screen))?;
    let objects =
        board.with_artboard(|b| b.objects().iter().flatten().cloned().collect::<Vec<_>>());
    let mut parent = board.core_handle();
    for part in path.path.split('/') {
        let matches = objects
            .iter()
            .filter(|h| {
                name(h).as_deref() == Some(part)
                    && h.with(|o| o.as_component().and_then(|c| c.parent_handle()))
                        .flatten()
                        .as_ref()
                        == Some(&parent)
            })
            .cloned()
            .collect::<Vec<_>>();
        ensure!(
            matches.len() == 1,
            "slot path {} is missing or ambiguous",
            path.path
        );
        parent = matches[0].clone();
    }
    slot(&parent, |_| ())?;
    Ok(parent)
}
fn find_holder(board: &RuntimeArtboardInstanceHandle, id: &CoreHandle) -> Option<CoreHandle> {
    for host in board.with_artboard(|b| b.nested_artboards()) {
        if let Some(child) = slot(&host, NestedArtboard::artboard_instance_default)
            .ok()
            .flatten()
        {
            if child.core_handle() == *id {
                return Some(host);
            }
            if let Some(found) = find_holder(&child, id) {
                return Some(found);
            }
        }
    }
    for list in board.with_artboard(|b| b.artboard_component_lists()) {
        let children = list
            .with_downcast::<ArtboardComponentList, _>(|l| {
                (0..l.artboard_count())
                    .filter_map(|i| l.artboard_instance(i as i32))
                    .collect::<Vec<_>>()
            })
            .unwrap();
        for child in children {
            if let Some(found) = find_holder(&child, id) {
                return Some(found);
            }
        }
    }
    None
}

/// Transform from an artboard's draw space into its player's draw space.
fn board_to_root(board: &CoreHandle) -> Mat2D {
    let (host, origin) = board
        .with_downcast::<Artboard, _>(|b| (b.host(), b.origin()))
        .unwrap();
    let Some(host) = host else {
        return Mat2D::identity();
    };
    let (local, parent) = host
        .with(|o| {
            let host = o.as_artboard_host().expect("artboard host");
            let runtime = board.runtime_artboard_instance().unwrap();
            (
                host.world_transform_for_artboard(runtime.downgrade()),
                host.parent_artboard(),
            )
        })
        .unwrap();
    parent.map_or(Mat2D::identity(), |p| board_to_root(&p))
        * local
        * Mat2D::from_translate(-origin.x, -origin.y)
}
#[derive(Clone, Copy, Debug)]
pub struct Pose {
    pub transform: Mat2D,
    pub size: Vec2D,
    pub opacity: f32,
}
pub fn slot_pose(host: &CoreHandle) -> Result<Pose> {
    let (parent, world, opacity, size) = slot(host, |s| {
        (
            s.parent_artboard_handle(),
            s.base.base.world_transform(),
            s.base.base.render_opacity(),
            s.live_move.size.or_else(|| s.natural_size_for_layout()),
        )
    })?;
    let size = slot(host, |s| {
        s.instance.as_ref().map(|i| instance_size(host, i))
    })?
    .or(size)
    .unwrap_or_default();
    Ok(Pose {
        transform: parent.map_or(Mat2D::identity(), |b| board_to_root(&b)) * world,
        size,
        opacity,
    })
}
fn child_control(host: &CoreHandle, child_name: &str) -> Option<Vec2D> {
    let parent = slot(host, NestedArtboard::parent_artboard_handle)
        .ok()
        .flatten()?;
    let objects = parent.with_downcast::<Artboard, _>(|b| b.objects().to_vec())?;
    objects.iter().flatten().find_map(|h| {
        if name(h).as_deref() != Some(child_name) {
            return None;
        }
        h.with(|o| {
            let node = o.as_node()?;
            (node.parent_handle().as_ref() == Some(host)).then(|| Vec2D::new(node.x(), node.y()))
        })
        .flatten()
    })
}
fn follow(flight: &Flight, apply: bool) -> Result<()> {
    let a = slot_pose(&flight.from)?;
    let b = slot_pose(&flight.to)?;
    let t = child_control(&flight.flight, "progress")
        .map_or(flight.progress, |p| p.x)
        .clamp(0.0, 1.0);
    ensure!(t.is_finite(), "non-finite flight progress");
    let mix = |a: f32, b: f32| a + (b - a) * t;
    let ac = a.transform.decompose();
    let bc = b.transform.decompose();
    let mut c = ac;
    let delta = (bc.rotation() - ac.rotation() + std::f32::consts::PI)
        .rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    c.set_rotation(ac.rotation() + delta * t);
    c.set_scale_x(mix(ac.scale_x(), bc.scale_x()));
    c.set_scale_y(mix(ac.scale_y(), bc.scale_y()));
    c.set_skew(mix(ac.skew(), bc.skew()));
    let offset = child_control(&flight.flight, "path").unwrap_or_default();
    c.set_x(mix(ac.x(), bc.x()) + offset.x);
    c.set_y(mix(ac.y(), bc.y()) + offset.y);
    let size = Vec2D::new(mix(a.size.x, b.size.x), mix(a.size.y, b.size.y));
    if flight.size_mode == SizeMode::Scale {
        ensure!(
            flight.unit_size.x > 0.0 && flight.unit_size.y > 0.0,
            "unit has zero natural size"
        );
        c.set_scale_x(c.scale_x() * size.x / flight.unit_size.x);
        c.set_scale_y(c.scale_y() * size.y / flight.unit_size.y);
    }
    let (parent_board, parent_node) = flight
        .flight
        .with(|o| {
            let c = o.as_component().unwrap();
            (c.artboard_handle(), c.parent_handle())
        })
        .unwrap();
    let (parent_world, parent_opacity) = parent_node
        .and_then(|p| {
            p.with(|o| {
                o.as_world_transform_component()
                    .map(|p| (p.world_transform(), p.child_opacity()))
            })
            .flatten()
        })
        .unwrap_or((Mat2D::identity(), 1.0));
    let parent_root = parent_board.map_or(Mat2D::identity(), |b| board_to_root(&b)) * parent_world;
    let mut inverse = Mat2D::identity();
    ensure!(
        parent_root.invert(&mut inverse),
        "flight parent transform is singular"
    );
    let opacity = mix(a.opacity, b.opacity);
    ensure!(
        parent_opacity != 0.0 || opacity == 0.0,
        "nonzero flight under a transparent parent"
    );
    ensure!(
        c.x().is_finite() && c.y().is_finite() && size.x.is_finite() && size.y.is_finite(),
        "non-finite flight pose"
    );
    if !apply {
        return Ok(());
    }
    if let Some(instance) = slot(&flight.flight, NestedArtboard::artboard_instance_default)? {
        let size = if flight.size_mode == SizeMode::Scale {
            flight.unit_size
        } else {
            size
        };
        instance.set_size(size.x, size.y);
    }
    let local_matrix = inverse * Mat2D::compose(&c);
    let local = local_matrix.decompose();
    slot_mut(&flight.flight, |s| {
        s.live_move.local_override = Some(local_matrix);
        s.base.base.mark_transform_dirty();
    });
    for (key, value) in [
        (NodeBase::X_PROPERTY_KEY, local.x()),
        (NodeBase::Y_PROPERTY_KEY, local.y()),
        (
            TransformComponentBase::ROTATION_PROPERTY_KEY,
            local.rotation(),
        ),
        (
            TransformComponentBase::SCALE_X_PROPERTY_KEY,
            local.scale_x(),
        ),
        (
            TransformComponentBase::SCALE_Y_PROPERTY_KEY,
            local.scale_y(),
        ),
    ] {
        CoreRegistry::set_double_handle(&flight.flight, i32::from(key), value);
    }
    CoreRegistry::set_double_handle(
        &flight.flight,
        i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
        if parent_opacity == 0.0 {
            0.0
        } else {
            opacity / parent_opacity
        },
    );
    Ok(())
}

impl ArtboardComponentList {
    pub(crate) fn live_move_rebuild_collapse_mask(&mut self) {
        if self.live_move_collapsed_rows.is_empty() {
            self.live_move_collapsed_mask.clear();
            return;
        }
        self.live_move_collapsed_mask = vec![0; self.artboard_count().div_ceil(64)];
        for row in 0..self.artboard_count() {
            if self.live_move_row_collapsed(row as i32) {
                self.live_move_collapsed_mask[row >> 6] |= 1 << (row & 63);
            }
        }
    }
    pub(crate) fn live_move_row_collapsed(&self, index: i32) -> bool {
        self.list_item(index)
            .is_some_and(|item| self.live_move_collapsed_rows.contains(&item))
    }
}

/// Collapse a mounted list row without releasing the row, its VM or playback.
/// Calls are idempotent. A collapsed row remains discoverable for coming home.
pub fn set_row_collapsed(list: &CoreHandle, index: i32, collapsed: bool) -> Result<()> {
    let (row, parent, changed) = list
        .with_downcast_mut::<ArtboardComponentList, _>(|l| -> Result<_> {
            ensure!(
                !l.virtualization_enabled(),
                "row collapse requires a non-virtual list"
            );
            let item = l
                .list_item(index)
                .ok_or_else(|| anyhow::anyhow!("row not found"))?;
            let changed = if collapsed {
                l.live_move_collapsed_rows.insert(item)
            } else {
                l.live_move_collapsed_rows.remove(&item)
            };
            if changed {
                l.reset_quiet_row(index as usize);
                l.invalidate_ordered_list_indices_cache();
                l.mark_host_transform_dirty();
            }
            Ok((
                l.artboard_instance(index),
                l.layout_parent_handle(),
                changed,
            ))
        })
        .ok_or_else(|| anyhow::anyhow!("expected ArtboardComponentList"))??;
    if !changed {
        return Ok(());
    }
    if let Some(row) = row {
        if collapsed {
            row.collapse_semantic_boundary(true);
        } else {
            // The list's inactive sequence was marked every skipped frame.
            row.update_data_binds(true);
            crate::hosted_scene::resume_semantics(&row);
        }
    }
    if let Some(parent) = parent {
        LayoutComponent::sync_layout_children_occurrence(&parent);
        LayoutComponent::mark_layout_style_dirty_occurrence(&parent);
    }
    Ok(())
}

thread_local! {
    static MEASURE_ADVANCES: std::cell::Cell<Option<u64>> = const { std::cell::Cell::new(None) };
}
/// Spike work measurement. Disabled unless explicitly enabled on this thread.
#[doc(hidden)]
pub fn measure_advances(enabled: bool) -> u64 {
    MEASURE_ADVANCES.with(|n| n.replace(enabled.then_some(0)).unwrap_or(0))
}
pub(crate) fn count_advance(flags: crate::AdvanceFlags) {
    if flags.contains(crate::AdvanceFlags::NEW_FRAME) {
        MEASURE_ADVANCES.with(|n| {
            if let Some(count) = n.get() {
                n.set(Some(count + 1));
            }
        });
    }
}

// Small virtual-composition hook in both NestedArtboard concrete owners.
// Ordinary hosts have no override and use the upstream implementation.
pub(crate) fn compose_slot_transform<T: crate::source::transform_component::TransformUpdate>(
    object: &mut T,
) {
    let matrix = object
        .as_nested_artboard()
        .and_then(|s| s.live_move.local_override);
    if let Some(matrix) = matrix {
        object
            .as_transform_component_mut()
            .expect("slot transform")
            .cells()
            .set_local(matrix);
    }
    crate::source::transform_component::compose_world_transform(object);
}
