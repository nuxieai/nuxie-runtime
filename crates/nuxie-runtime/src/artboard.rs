use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use anyhow::{Context, Result};
use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;
use nuxie_render_api::Factory as RenderFactory;

use crate::RuntimeOwnedViewModelInstance;
use crate::animation::{
    LinearAnimationInstance, RuntimeJoystick, RuntimeKeyedCallback, RuntimeLinearAnimation,
    build_linear_animations, build_runtime_joysticks,
};
use crate::artboard_data_bind::{
    RuntimeArtboardContextSourceValue, RuntimeArtboardConverterPropertyBindingInstance,
    RuntimeArtboardCustomPropertyBindingInstance, RuntimeArtboardDataBindSourceQueues,
    RuntimeArtboardDataBindTargetQueues, RuntimeArtboardFormulaTokenBindingInstance,
    RuntimeArtboardImageAssetBindingInstance, RuntimeArtboardLayoutComputedBindingInstance,
    RuntimeArtboardListBindingInstance, RuntimeArtboardNestedHostBindingInstance,
    RuntimeArtboardNumericSourceBindingInstance, RuntimeArtboardOwnedContextKey,
    RuntimeArtboardPropertyBindingInstance, RuntimeArtboardSoloBindingInstance,
    RuntimeArtboardSoloSourceBindingInstance, RuntimeArtboardTextListBindingInstance,
    RuntimeNestedChildContextUpdate, apply_artboard_name_based_color_data_bind_defaults,
    build_artboard_converter_property_bindings, build_artboard_custom_property_bindings,
    build_artboard_default_view_model_values, build_artboard_formula_token_bindings,
    build_artboard_image_asset_bindings, build_artboard_layout_computed_bindings,
    build_artboard_list_bindings, build_artboard_nested_host_bindings,
    build_artboard_numeric_source_bindings, build_artboard_property_bindings,
    build_artboard_solo_bindings, build_artboard_solo_source_bindings,
    build_artboard_text_list_bindings, build_nested_host_data_bind_source_local_slots,
    build_nested_host_data_bind_source_locals, build_nested_host_view_model_instance_locals,
};
use crate::components::{
    AuthoredTransform, ComponentDirt, Mat2D, RuntimeComponent, RuntimeSolo, TransformProperty,
    UpdateComponentsReport, apply_initial_solo_collapses, build_runtime_solos,
    retain_runtime_component_layout_topology,
};
use crate::constraints::{
    RuntimeFollowPathConstraint, RuntimeIkConstraint, RuntimeListFollowPathConstraint,
    RuntimeScrollConstraint, build_runtime_follow_path_constraints, build_runtime_ik_constraints,
    build_runtime_list_follow_path_constraints, build_runtime_scroll_constraints,
};
use crate::data_bind_graph::{RuntimeDataBindGraphFormulaRandomSource, RuntimeDataBindGraphValue};
use crate::draw::RuntimeLayoutBounds;
use crate::objects::{InstanceObjectArena, InstanceSlot};
use crate::properties::{
    JOYSTICK_FLAG_INVERT_X, JOYSTICK_FLAG_INVERT_Y, RuntimeArtboardDimensions,
    joystick_flags_property_key, joystick_x_property_key, joystick_y_property_key,
    layout_component_style_display_value_property_key, property_key_for_name,
    solid_color_value_property_key, solo_active_component_id_property_key,
    transform_property_for_key,
};
use crate::scripting::{
    NoopScriptHost, RuntimeScriptInstanceHandle, ScriptArtboard, ScriptError, ScriptInstance,
    ScriptMethod, ScriptValue, ScriptViewModel,
};
use crate::state_machine::{
    RuntimeStateMachine, StateMachineInputKind, StateMachineInstance, StateMachineReportedEvent,
    build_state_machines,
};
use crate::view_model::RuntimeOwnedViewModelListHandle;

#[derive(Debug)]
struct RuntimeArtboardInstanceIdentity(u64);

impl RuntimeArtboardInstanceIdentity {
    fn next() -> Self {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(0);
        Self(NEXT.fetch_add(1, Ordering::Relaxed))
    }
}

impl Clone for RuntimeArtboardInstanceIdentity {
    fn clone(&self) -> Self {
        Self::next()
    }
}

#[derive(Debug, Clone)]
pub struct ArtboardInstance {
    instance_identity: RuntimeArtboardInstanceIdentity,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) origin_x: f32,
    pub(crate) origin_y: f32,
    pub(crate) clip: bool,
    pub(crate) slots: Vec<InstanceSlot>,
    pub(crate) objects: InstanceObjectArena,
    pub(crate) components: Vec<RuntimeComponent>,
    pub(crate) component_by_local: BTreeMap<usize, usize>,
    pub(crate) solos: Vec<RuntimeSolo>,
    pub(crate) joysticks: Vec<RuntimeJoystick>,
    pub(crate) follow_path_constraints: Vec<RuntimeFollowPathConstraint>,
    pub(crate) list_follow_path_constraints: Vec<RuntimeListFollowPathConstraint>,
    pub(crate) scroll_constraints: Vec<RuntimeScrollConstraint>,
    pub(crate) component_list_item_transforms: BTreeMap<usize, Vec<Mat2D>>,
    pub(crate) component_list_items: BTreeMap<usize, Vec<RuntimeComponentListItemInstance>>,
    pub(crate) component_list_sources: BTreeMap<usize, RuntimeOwnedViewModelListHandle>,
    pub(crate) ik_constraints: Vec<RuntimeIkConstraint>,
    pub(crate) joysticks_apply_before_update: bool,
    pub(crate) update_order: Vec<usize>,
    pub(crate) linear_animations: Vec<RuntimeLinearAnimation>,
    pub(crate) state_machines: Arc<Vec<RuntimeStateMachine>>,
    pub(crate) script_instances_by_global: BTreeMap<u32, RuntimeScriptInstanceHandle>,
    pub(crate) scripted_data_converter_instances_by_global:
        BTreeMap<u32, RuntimeScriptInstanceHandle>,
    nested_script_owned_contexts: BTreeMap<u32, RuntimeOwnedViewModelInstance>,
    script_path_effect_globals: BTreeSet<u32>,
    script_advances_active: BTreeSet<u32>,
    script_updates_pending: BTreeSet<u32>,
    pub(crate) nested_artboards: BTreeMap<usize, RuntimeNestedArtboardInstance>,
    pub(crate) nested_artboard_locals: Vec<usize>,
    newly_uncollapsed_nested_artboards: BTreeSet<usize>,
    pub(crate) graph_global_id: u32,
    build_context: Option<RuntimeArtboardBuildContext>,
    nested_layout_bounds: Option<RuntimeNestedLayoutBoundsFrame>,
    pub(crate) artboard_data_bind_values: BTreeMap<Vec<u32>, RuntimeDataBindGraphValue>,
    pub(crate) artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource,
    pub(crate) artboard_owned_context_key: Option<RuntimeArtboardOwnedContextKey>,
    pub(crate) artboard_property_bindings: Vec<RuntimeArtboardPropertyBindingInstance>,
    pub(crate) artboard_image_asset_bindings: Vec<RuntimeArtboardImageAssetBindingInstance>,
    pub(crate) artboard_data_bind_target_queues: RuntimeArtboardDataBindTargetQueues,
    pub(crate) artboard_data_bind_source_queues: RuntimeArtboardDataBindSourceQueues,
    pub(crate) artboard_data_bind_suppressed_target_data_bind: Option<usize>,
    pub(crate) artboard_custom_property_bindings: Vec<RuntimeArtboardCustomPropertyBindingInstance>,
    pub(crate) artboard_layout_computed_bindings: Vec<RuntimeArtboardLayoutComputedBindingInstance>,
    pub(crate) artboard_numeric_source_bindings: Vec<RuntimeArtboardNumericSourceBindingInstance>,
    pub(crate) artboard_formula_token_bindings: Vec<RuntimeArtboardFormulaTokenBindingInstance>,
    pub(crate) artboard_converter_property_bindings:
        Vec<RuntimeArtboardConverterPropertyBindingInstance>,
    pub(crate) artboard_solo_bindings: Vec<RuntimeArtboardSoloBindingInstance>,
    pub(crate) artboard_solo_source_bindings: Vec<RuntimeArtboardSoloSourceBindingInstance>,
    pub(crate) artboard_nested_host_bindings: Vec<RuntimeArtboardNestedHostBindingInstance>,
    pub(crate) artboard_list_bindings: Vec<RuntimeArtboardListBindingInstance>,
    pub(crate) artboard_text_list_bindings: Vec<RuntimeArtboardTextListBindingInstance>,
    pub(crate) artboard_context_source_values_scratch: Vec<RuntimeArtboardContextSourceValue>,
    pub(crate) artboard_nested_child_context_updates_scratch: Vec<RuntimeNestedChildContextUpdate>,
    pub(crate) image_asset_overrides: BTreeMap<usize, Option<u32>>,
    pub(crate) dirt: ComponentDirt,
    pub(crate) dirt_depth: usize,
    pub(crate) cache_epoch: u64,
    pub(crate) prepared_epoch: u64,
    pub(crate) path_epoch: u64,
    pub(crate) layout_epoch: u64,
    pub(crate) draw_order_epoch: u64,
    pub(crate) did_change: bool,
    pub(crate) layout_constraint_bounds_enabled: bool,
    pub(crate) layout_constraint_bounds: Option<Arc<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeNestedArtboardInstance {
    pub(crate) child: Box<ArtboardInstance>,
    pub(crate) render_cache_revision: u64,
    pub(crate) data_bind_path_ids: Option<Vec<u32>>,
    pub(crate) data_bind_path_is_relative: bool,
    pub(crate) stateful_view_model_instance_local: Option<usize>,
    pub(crate) stateful_view_model_context: Option<RuntimeOwnedViewModelInstance>,
    pub(crate) data_bind_property_source_locals: Vec<Option<usize>>,
    pub(crate) data_bind_image_source_locals: Vec<Option<usize>>,
    pub(crate) data_bind_context_source_locals_by_path: BTreeMap<Vec<u32>, usize>,
    animations: Vec<RuntimeNestedAnimationInstance>,
    is_paused: bool,
    speed: f32,
    quantize: f32,
    cumulated_seconds: f32,
}

/// Ported from C++ `src/artboard_component_list.cpp`: one persistent child
/// artboard and state machine for an owned view-model list item.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeComponentListItemInstance {
    pub(crate) child: Box<ArtboardInstance>,
    pub(crate) state_machine: Option<StateMachineInstance>,
    pub(crate) context: RuntimeOwnedViewModelInstance,
    pub(crate) transform: Mat2D,
    pub(crate) render_cache_revision: u64,
}

#[derive(Debug, Clone)]
struct RuntimeArtboardBuildContext {
    file: Arc<RuntimeFile>,
    artboards: Arc<Vec<ArtboardGraph>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RuntimeNestedLayoutBoundsCacheKey {
    graph_global_id: u32,
    layout_epoch: u64,
}

#[derive(Debug, Clone)]
struct RuntimeNestedLayoutBoundsFrame {
    key: RuntimeNestedLayoutBoundsCacheKey,
    bounds: Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>>,
}

#[derive(Debug, Clone)]
enum RuntimeNestedAnimationInstance {
    Simple {
        animation: LinearAnimationInstance,
        is_playing: bool,
        speed: f32,
        mix: f32,
    },
    Remap {
        local_id: usize,
        animation: LinearAnimationInstance,
        mix: f32,
    },
    StateMachine {
        local_id: usize,
        state_machine: StateMachineInstance,
    },
}

impl ArtboardInstance {
    pub fn from_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Result<Self> {
        let artboards = vec![graph.clone()];
        let context = RuntimeArtboardBuildContext {
            file: Arc::new(file.clone()),
            artboards: Arc::new(artboards.clone()),
        };
        Self::from_graph_inner(
            file,
            graph,
            &artboards,
            &mut BTreeSet::new(),
            Some(context),
            true,
        )
    }

    pub fn from_graph_with_artboards(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
    ) -> Result<Self> {
        let context = RuntimeArtboardBuildContext {
            file: Arc::new(file.clone()),
            artboards: Arc::new(artboards.to_vec()),
        };
        Self::from_graph_inner(
            file,
            graph,
            artboards,
            &mut BTreeSet::new(),
            Some(context),
            true,
        )
    }

    fn from_graph_inner(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        visiting: &mut BTreeSet<u32>,
        build_context: Option<RuntimeArtboardBuildContext>,
        layout_constraint_bounds_enabled: bool,
    ) -> Result<Self> {
        let inserted = visiting.insert(graph.global_id);
        let dimensions =
            RuntimeArtboardDimensions::from_object(file.object(graph.global_id as usize));
        let mut slots = Vec::new();
        for local_object in &graph.local_objects {
            let object = file.object(local_object.global_id as usize);
            if local_object.type_name.is_some() && object.is_none() {
                anyhow::bail!(
                    "local object {} global id {} is missing",
                    local_object.local_id,
                    local_object.global_id
                );
            }
            slots.push(InstanceSlot {
                local_id: local_object.local_id,
                source_global_id: local_object.global_id,
                type_name: local_object.type_name,
                name: local_object.name.clone(),
                component_index: None,
            });
        }
        let mut objects = InstanceObjectArena::from_slots(file, &slots);
        apply_artboard_name_based_color_data_bind_defaults(file, graph, &mut objects);

        let mut component_by_local = BTreeMap::new();
        let mut components = Vec::new();

        for component in &graph.components {
            file.object(component.global_id as usize).with_context(|| {
                format!("component global id {} is missing", component.global_id)
            })?;

            component_by_local.insert(component.local_id, components.len());
            components.push(RuntimeComponent::from_graph_component(component));
            let slot = slots
                .get_mut(component.local_id)
                .with_context(|| format!("component local id {} is missing", component.local_id))?;
            slot.component_index = Some(components.len() - 1);
        }

        let mut update_order = graph
            .components
            .iter()
            .filter_map(|component| {
                component
                    .graph_order
                    .map(|order| (order, component.local_id))
            })
            .collect::<Vec<_>>();
        update_order.sort_by_key(|(order, local_id)| (*order, *local_id));
        let update_order = update_order
            .into_iter()
            .map(|(_, local_id)| local_id)
            .collect::<Vec<_>>();
        let solos = build_runtime_solos(file, graph);
        let linear_animations = build_linear_animations(file, graph, &slots);
        let joysticks = build_runtime_joysticks(graph, &linear_animations);
        let follow_path_constraints = build_runtime_follow_path_constraints(file, graph);
        let list_follow_path_constraints = build_runtime_list_follow_path_constraints(file, graph);
        let scroll_constraints = build_runtime_scroll_constraints(file, graph);
        let ik_constraints = build_runtime_ik_constraints(file, graph);
        let state_machines = build_state_machines(file, graph, &linear_animations);
        let artboard_data_bind_values = build_artboard_default_view_model_values(file, graph);
        let artboard_property_bindings = build_artboard_property_bindings(file, graph);
        let artboard_image_asset_bindings = build_artboard_image_asset_bindings(file, graph);
        let artboard_custom_property_bindings =
            build_artboard_custom_property_bindings(file, graph);
        let artboard_layout_computed_bindings =
            build_artboard_layout_computed_bindings(file, graph);
        let artboard_numeric_source_bindings = build_artboard_numeric_source_bindings(file, graph);
        let artboard_formula_token_bindings = build_artboard_formula_token_bindings(file, graph);
        let artboard_converter_property_bindings =
            build_artboard_converter_property_bindings(file, graph);
        let artboard_data_bind_target_queues = RuntimeArtboardDataBindTargetQueues::new(
            &artboard_property_bindings,
            &artboard_image_asset_bindings,
            &artboard_converter_property_bindings,
        );
        let artboard_solo_bindings = build_artboard_solo_bindings(file, graph);
        let artboard_solo_source_bindings = build_artboard_solo_source_bindings(file, graph);
        let artboard_nested_host_bindings = build_artboard_nested_host_bindings(file, graph);
        let artboard_list_bindings = build_artboard_list_bindings(file, graph);
        let artboard_text_list_bindings = build_artboard_text_list_bindings(file, graph);
        let artboard_data_bind_source_queues = RuntimeArtboardDataBindSourceQueues::new(
            &artboard_custom_property_bindings,
            &artboard_layout_computed_bindings,
            &artboard_numeric_source_bindings,
            &artboard_solo_source_bindings,
        );
        apply_initial_solo_collapses(&objects, &solos, &mut components, &component_by_local);
        retain_runtime_component_layout_topology(&mut components, &component_by_local);
        let nested_artboards = if inserted {
            build_runtime_nested_artboard_instances(
                file,
                graph,
                artboards,
                &slots,
                &objects,
                visiting,
                build_context.clone(),
            )?
        } else {
            BTreeMap::new()
        };
        if inserted {
            visiting.remove(&graph.global_id);
        }
        let nested_artboard_locals = nested_artboards.keys().copied().collect::<Vec<_>>();

        let mut instance = Self {
            instance_identity: RuntimeArtboardInstanceIdentity::next(),
            width: dimensions.width,
            height: dimensions.height,
            origin_x: dimensions.origin_x,
            origin_y: dimensions.origin_y,
            clip: dimensions.clip,
            slots,
            objects,
            components,
            component_by_local,
            solos,
            joysticks,
            follow_path_constraints,
            list_follow_path_constraints,
            scroll_constraints,
            component_list_item_transforms: BTreeMap::new(),
            component_list_items: BTreeMap::new(),
            component_list_sources: BTreeMap::new(),
            ik_constraints,
            joysticks_apply_before_update: graph.joysticks_apply_before_update,
            update_order,
            linear_animations,
            state_machines: Arc::new(state_machines),
            script_instances_by_global: BTreeMap::new(),
            scripted_data_converter_instances_by_global: BTreeMap::new(),
            nested_script_owned_contexts: BTreeMap::new(),
            script_path_effect_globals: BTreeSet::new(),
            script_advances_active: BTreeSet::new(),
            script_updates_pending: BTreeSet::new(),
            nested_artboards,
            nested_artboard_locals,
            newly_uncollapsed_nested_artboards: BTreeSet::new(),
            graph_global_id: graph.global_id,
            build_context,
            nested_layout_bounds: None,
            artboard_data_bind_values,
            artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
            artboard_owned_context_key: None,
            artboard_property_bindings,
            artboard_image_asset_bindings,
            artboard_data_bind_target_queues,
            artboard_data_bind_source_queues,
            artboard_data_bind_suppressed_target_data_bind: None,
            artboard_custom_property_bindings,
            artboard_layout_computed_bindings,
            artboard_numeric_source_bindings,
            artboard_formula_token_bindings,
            artboard_converter_property_bindings,
            artboard_solo_bindings,
            artboard_solo_source_bindings,
            artboard_nested_host_bindings,
            artboard_list_bindings,
            artboard_text_list_bindings,
            artboard_context_source_values_scratch: Vec::new(),
            artboard_nested_child_context_updates_scratch: Vec::new(),
            image_asset_overrides: BTreeMap::new(),
            dirt: ComponentDirt::COMPONENTS,
            dirt_depth: 0,
            cache_epoch: 1,
            prepared_epoch: 1,
            path_epoch: 1,
            layout_epoch: 1,
            draw_order_epoch: 1,
            did_change: true,
            layout_constraint_bounds_enabled,
            layout_constraint_bounds: None,
        };
        instance.apply_initial_layout_component_display_collapses();
        let nested_host_locals = instance.nested_artboard_locals.clone();
        for host_local_id in nested_host_locals {
            instance.sync_nested_artboard_root_opacity(host_local_id);
        }

        Ok(instance)
    }

    pub fn component(&self, local_id: usize) -> Option<&RuntimeComponent> {
        self.slots
            .get(local_id)
            .and_then(|slot| slot.component_index)
            .and_then(|index| self.components.get(index))
    }

    /// Attach a VM-owned script instance to a scripted object global id.
    ///
    /// Ported toward C++ `src/scripted/scripted_drawable.cpp`: the runtime draw
    /// path owns the `ScriptedDrawable` envelope, while the backend VM owns the
    /// instance table and `draw(self, renderer)` method.
    pub fn set_script_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) {
        self.script_advances_active.remove(&global_id);
        if instance.has_method(ScriptMethod::Advance).unwrap_or(false) {
            self.script_advances_active.insert(global_id);
        }
        self.script_instances_by_global
            .insert(global_id, RuntimeScriptInstanceHandle::new(instance));
        self.script_updates_pending.insert(global_id);
    }

    pub fn set_script_path_effect_instance_for_global(
        &mut self,
        global_id: u32,
        instance: Box<dyn ScriptInstance>,
    ) {
        self.script_path_effect_globals.insert(global_id);
        self.set_script_instance_for_global(global_id, instance);
    }

    /// Runs the C++ `ScriptedDrawable::update` phase for scripts dirtied by
    /// initialization or input hydration.
    pub fn update_script_instances(&mut self) -> Result<bool, ScriptError> {
        let pending = std::mem::take(&mut self.script_updates_pending);
        let mut did_update = false;
        let mut host = NoopScriptHost;
        for global_id in pending {
            if self.script_path_effect_globals.contains(&global_id) {
                continue;
            }
            let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
                continue;
            };
            let mut instance = handle.borrow_mut();
            if !instance.has_method(ScriptMethod::Update)? {
                continue;
            }
            instance.call_method(ScriptMethod::Update, &[], &mut host)?;
            did_update = true;
        }
        did_update |= self.refresh_component_list_items();
        Ok(did_update)
    }

    pub fn advance_script_instances(&mut self, seconds: f32) -> Result<bool, ScriptError> {
        if seconds == 0.0 {
            return Ok(false);
        }
        let active = std::mem::take(&mut self.script_advances_active);
        let mut did_advance = false;
        let mut host = NoopScriptHost;
        for global_id in active {
            let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
                continue;
            };
            let result = handle.borrow_mut().call_method(
                ScriptMethod::Advance,
                &[ScriptValue::Number(f64::from(seconds))],
                &mut host,
            )?;
            if result == ScriptValue::Bool(true) {
                self.script_advances_active.insert(global_id);
                self.script_updates_pending.insert(global_id);
                did_advance = true;
            }
        }
        Ok(did_advance)
    }

    /// Re-runs user `init` after C++ clears a scripted object's data context.
    pub fn reinitialize_script_instances(&mut self) -> Result<bool, ScriptError> {
        let mut did_initialize = false;
        let mut host = NoopScriptHost;
        for (global_id, handle) in &self.script_instances_by_global {
            let mut instance = handle.borrow_mut();
            if !instance.has_method(ScriptMethod::Init)? {
                continue;
            }
            instance.call_method(ScriptMethod::Init, &[], &mut host)?;
            if instance.has_method(ScriptMethod::Advance).unwrap_or(false) {
                self.script_advances_active.insert(*global_id);
            }
            self.script_updates_pending.insert(*global_id);
            did_initialize = true;
        }
        Ok(did_initialize)
    }

    pub fn reinitialize_script_instances_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let mut did_initialize = false;
        let mut host = NoopScriptHost;
        for (global_id, handle) in &self.script_instances_by_global {
            let mut instance = handle.borrow_mut();
            if !instance.has_method(ScriptMethod::Init)? {
                continue;
            }
            instance.call_method_with_factory(ScriptMethod::Init, &[], &mut host, factory)?;
            if instance.has_method(ScriptMethod::Advance).unwrap_or(false) {
                self.script_advances_active.insert(*global_id);
            }
            self.script_updates_pending.insert(*global_id);
            did_initialize = true;
        }
        Ok(did_initialize)
    }

    pub fn reinitialize_script_instance_with_factory(
        &mut self,
        global_id: u32,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let Some(handle) = self.script_instances_by_global.get(&global_id).cloned() else {
            return Ok(false);
        };
        let mut instance = handle.borrow_mut();
        if !instance.has_method(ScriptMethod::Init)? {
            return Ok(false);
        }
        instance.call_method_with_factory(ScriptMethod::Init, &[], &mut NoopScriptHost, factory)?;
        if instance.has_method(ScriptMethod::Advance).unwrap_or(false) {
            self.script_advances_active.insert(global_id);
        }
        self.script_updates_pending.insert(global_id);
        Ok(true)
    }

    pub fn set_script_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        value: ScriptValue,
    ) -> Result<(), ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        handle.borrow_mut().set_input(name, value)?;
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.script_advances_active.insert(global_id);
        }
        self.script_updates_pending.insert(global_id);
        Ok(())
    }

    pub fn set_script_artboard_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        artboard: Box<dyn ScriptArtboard>,
    ) -> Result<(), ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        handle.borrow_mut().set_artboard_input(name, artboard)?;
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.script_advances_active.insert(global_id);
        }
        self.script_updates_pending.insert(global_id);
        Ok(())
    }

    pub fn set_script_view_model_input_for_global(
        &mut self,
        global_id: u32,
        name: &str,
        view_model: ScriptViewModel,
    ) -> Result<(), ScriptError> {
        let handle = self
            .script_instances_by_global
            .get(&global_id)
            .cloned()
            .ok_or_else(|| ScriptError::new(format!("missing script instance {global_id}")))?;
        handle.borrow_mut().set_view_model_input(name, view_model)?;
        if handle
            .borrow_mut()
            .has_method(ScriptMethod::Advance)
            .unwrap_or(false)
        {
            self.script_advances_active.insert(global_id);
        }
        self.script_updates_pending.insert(global_id);
        Ok(())
    }

    pub fn set_script_context_view_model(
        &mut self,
        view_model: Option<ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        for handle in self.script_instances_by_global.values() {
            handle
                .borrow_mut()
                .set_context_view_model(view_model.clone())?;
        }
        Ok(())
    }

    pub fn mark_script_update_for_global(&mut self, global_id: u32) -> bool {
        if !self.script_instances_by_global.contains_key(&global_id) {
            return false;
        }
        self.script_updates_pending.insert(global_id)
    }

    pub(crate) fn script_instance_for_global(
        &self,
        global_id: u32,
    ) -> Option<RuntimeScriptInstanceHandle> {
        self.script_instances_by_global.get(&global_id).cloned()
    }

    pub fn slot(&self, local_id: usize) -> Option<&InstanceSlot> {
        self.slots.get(local_id)
    }

    pub fn slots(&self) -> &[InstanceSlot] {
        &self.slots
    }

    pub fn component_mut(&mut self, local_id: usize) -> Option<&mut RuntimeComponent> {
        let index = self.slots.get(local_id)?.component_index?;
        Some(&mut self.components[index])
    }

    pub fn components(&self) -> &[RuntimeComponent] {
        &self.components
    }

    pub(crate) fn runtime_file(&self) -> Option<&RuntimeFile> {
        self.build_context
            .as_ref()
            .map(|context| context.file.as_ref())
    }

    pub(crate) fn runtime_file_arc(&self) -> Option<Arc<RuntimeFile>> {
        self.build_context
            .as_ref()
            .map(|context| Arc::clone(&context.file))
    }

    pub(crate) fn runtime_graph(&self) -> Option<&ArtboardGraph> {
        let graph_global_id = self.graph_global_id;
        self.build_context
            .as_ref()?
            .artboards
            .iter()
            .find(|graph| graph.global_id == graph_global_id)
    }

    pub fn update_order(&self) -> &[usize] {
        &self.update_order
    }

    pub fn linear_animation(&self, index: usize) -> Option<&RuntimeLinearAnimation> {
        self.linear_animations.get(index)
    }

    pub fn linear_animations(&self) -> &[RuntimeLinearAnimation] {
        &self.linear_animations
    }

    pub fn state_machine(&self, index: usize) -> Option<&RuntimeStateMachine> {
        self.state_machines.get(index)
    }

    pub fn state_machines(&self) -> &[RuntimeStateMachine] {
        self.state_machines.as_slice()
    }

    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        self.mark_changed();
        self.mark_layout_changed();
        true
    }

    pub(crate) fn artboard_property_value(&self, property_type: u64) -> f32 {
        match property_type {
            0 => self.width,
            1 => self.height,
            2 => self.width / self.height,
            _ => 0.0,
        }
    }

    /// Reads one typed color property from the live object arena.
    ///
    /// Returns `None` when either the local object or a color property with
    /// this key does not exist. Schema defaults are already materialized in
    /// the object arena, so a matching property returns its current value
    /// even when the source record omitted that default.
    pub fn color_property(&self, local_id: usize, property_key: u16) -> Option<u32> {
        self.objects.color_property(local_id, property_key)
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_color_property(&mut self, local_id: usize, property_key: u16, value: u32) -> bool {
        let previous = self.color_property(local_id, property_key);
        if !self
            .objects
            .set_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_color_property_set(local_id, property_key, previous, value)
    }

    pub(crate) fn set_keyed_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        let previous = self.color_property(local_id, property_key);
        if !self
            .objects
            .set_generated_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_color_property_set(local_id, property_key, previous, value)
    }

    fn after_color_property_set(
        &mut self,
        local_id: usize,
        property_key: u16,
        previous: Option<u32>,
        value: u32,
    ) -> bool {
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_changed();
        self.mark_prepared_changed_for_color_property(local_id, property_key, previous, value);
        self.apply_color_property_changed(local_id, property_key);
        true
    }

    pub(crate) fn bool_property(&self, local_id: usize, property_key: u16) -> Option<bool> {
        self.objects.bool_property(local_id, property_key)
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_bool_property(&mut self, local_id: usize, property_key: u16, value: bool) -> bool {
        if !self
            .objects
            .set_bool_property(local_id, property_key, value)
        {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_changed();
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        if property_affects_effect_path_epoch(
            self.slot(local_id).and_then(|slot| slot.type_name),
            property_key,
        ) {
            self.mark_path_changed();
        }
        self.apply_bool_property_changed(local_id, property_key, value);
        true
    }

    pub(crate) fn uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        self.objects.uint_property(local_id, property_key)
    }

    pub(crate) fn resolved_image_asset_global(
        &self,
        local_id: Option<usize>,
        authored_asset_global: Option<u32>,
    ) -> Option<u32> {
        local_id
            .and_then(|local_id| self.image_asset_overrides.get(&local_id))
            .copied()
            .unwrap_or(authored_asset_global)
    }

    pub(crate) fn set_image_asset_override(
        &mut self,
        local_id: usize,
        asset_global: Option<u32>,
    ) -> bool {
        if self.image_asset_overrides.get(&local_id) == Some(&asset_global) {
            return false;
        }
        self.image_asset_overrides.insert(local_id, asset_global);
        self.mark_changed();
        self.mark_prepared_changed();
        true
    }

    /// Reads one typed double property from the live object arena.
    ///
    /// Returns `None` when either the local object or a double property with
    /// this key does not exist. Schema defaults are already materialized in
    /// the object arena, so a matching property returns its current value
    /// even when the source record omitted that default.
    pub fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.objects.double_property(local_id, property_key)
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_double_property(&mut self, local_id: usize, property_key: u16, value: f32) -> bool {
        if !self
            .objects
            .set_double_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    pub(crate) fn set_keyed_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        if !self
            .objects
            .set_generated_double_property(local_id, property_key, value)
        {
            return false;
        }
        self.after_double_property_set(local_id, property_key, value)
    }

    fn after_double_property_set(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_changed();
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        if property_affects_effect_path_epoch(
            self.slot(local_id).and_then(|slot| slot.type_name),
            property_key,
        ) {
            self.mark_path_changed();
        }
        self.apply_double_property_changed(local_id, property_key, value);
        true
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_uint_property(&mut self, local_id: usize, property_key: u16, value: u64) -> bool {
        if !self
            .objects
            .set_uint_property(local_id, property_key, value)
        {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_changed();
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        if property_affects_effect_path_epoch(
            self.slot(local_id).and_then(|slot| slot.type_name),
            property_key,
        ) {
            self.mark_path_changed();
        }
        self.apply_uint_property_changed(local_id, property_key);
        true
    }

    pub(crate) fn string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        self.objects.string_property(local_id, property_key)
    }

    pub(crate) fn text_list_runs(&self, text_local: usize) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.artboard_text_list_bindings
            .iter()
            .find(|binding| binding.target_local_id() == text_local)
            .map(RuntimeArtboardTextListBindingInstance::text_runs)
            .unwrap_or_default()
    }

    /// Typed property write with dirt propagation — the write path the
    /// data-bind pipeline uses. Public for authoring hosts (editors, FFI
    /// embeddings): returns whether a matching property existed and its
    /// value changed; invalidation is handled internally.
    pub fn set_string_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: Vec<u8>,
    ) -> bool {
        if !self
            .objects
            .set_string_property(local_id, property_key, value)
        {
            return false;
        }
        self.notify_artboard_data_bind_target_property_changed(local_id, property_key);
        self.mark_changed();
        self.mark_prepared_changed_for_property(local_id, property_key);
        self.mark_layout_changed_for_property(local_id, property_key);
        self.apply_string_property_changed(local_id, property_key);
        true
    }

    pub fn apply_linear_animation(&mut self, index: usize, seconds: f32, mix: f32) -> bool {
        let Some(animation) = self.linear_animations.get(index).cloned() else {
            return false;
        };
        animation.apply(self, seconds, mix)
    }

    pub fn linear_animation_instance(&self, index: usize) -> Option<LinearAnimationInstance> {
        self.linear_animation_instance_with_speed(index, 1.0)
    }

    pub fn linear_animation_instance_with_speed(
        &self,
        index: usize,
        speed_multiplier: f32,
    ) -> Option<LinearAnimationInstance> {
        let animation = self.linear_animation(index)?;
        Some(LinearAnimationInstance::new(
            index,
            animation,
            speed_multiplier,
        ))
    }

    pub fn advance_linear_animation_instance(
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(animation) = self.linear_animation(instance.animation_index) else {
            return false;
        };
        instance.advance(animation, elapsed_seconds)
    }

    pub fn advance_linear_animation_instance_with_events(
        &mut self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let (mut changed, keyed_callbacks) = {
            let Some(animation) = self.linear_animation(instance.animation_index) else {
                return false;
            };
            let mut keyed_callbacks = Vec::new();
            let changed = instance.advance_with_events(
                animation,
                elapsed_seconds,
                reported_events,
                &mut keyed_callbacks,
            );
            (changed, keyed_callbacks)
        };
        for callback in keyed_callbacks {
            changed |= self.apply_keyed_callback(callback);
        }
        changed
    }

    pub fn apply_linear_animation_instance(
        &mut self,
        instance: &LinearAnimationInstance,
        mix: f32,
    ) -> bool {
        self.apply_linear_animation(instance.animation_index, instance.time, mix)
    }

    pub fn linear_animation_instance_keep_going(&self, instance: &LinearAnimationInstance) -> bool {
        let Some(animation) = self.linear_animation(instance.animation_index) else {
            return false;
        };
        instance.keep_going(animation)
    }

    pub fn state_machine_instance(&self, index: usize) -> Option<StateMachineInstance> {
        let state_machine = self.state_machine(index)?;
        Some(StateMachineInstance::new(index, state_machine, self))
    }

    /// Ported from C++ `src/artboard_component_list.cpp::updateList`,
    /// `findArtboard`, and `createArtboardAt`.
    pub(crate) fn sync_component_list_items(
        &mut self,
        file: &RuntimeFile,
        list_local_id: usize,
        contexts: Vec<RuntimeOwnedViewModelInstance>,
    ) -> bool {
        if self
            .component_list_items
            .get(&list_local_id)
            .is_some_and(|existing| {
                existing.len() == contexts.len()
                    && existing.iter().zip(&contexts).all(|(item, context)| {
                        item.context.instance_identity() == context.instance_identity()
                    })
            })
        {
            return false;
        }
        let Some(build_context) = self.build_context.clone() else {
            return false;
        };
        let Some(parent_graph) = build_context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)
        else {
            return false;
        };
        let Some(component_list) = parent_graph
            .component_lists
            .iter()
            .find(|list| list.local_id == list_local_id)
        else {
            return false;
        };

        let mut items = Vec::with_capacity(contexts.len());
        for context in contexts {
            let mapped_index = component_list
                .map_rules
                .iter()
                .find(|rule| rule.view_model_id == context.view_model_index() as i64)
                .and_then(|rule| usize::try_from(rule.artboard_id).ok());
            let child_graph = mapped_index
                .and_then(|index| build_context.artboards.get(index))
                .or_else(|| {
                    build_context.artboards.iter().find(|graph| {
                        file.object(graph.global_id as usize)
                            .and_then(|artboard| artboard.uint_property("viewModelId"))
                            .and_then(|value| usize::try_from(value).ok())
                            == Some(context.view_model_index())
                    })
                });
            let Some(child_graph) = child_graph else {
                continue;
            };
            let mut visiting = BTreeSet::from([self.graph_global_id]);
            let Ok(mut child) = ArtboardInstance::from_graph_inner(
                file,
                child_graph,
                &build_context.artboards,
                &mut visiting,
                Some(build_context.clone()),
                false,
            ) else {
                continue;
            };
            child.bind_owned_view_model_artboard_context(file, &context);
            let mut state_machine = child.state_machine_instance(0);
            if let Some(state_machine) = state_machine.as_mut() {
                state_machine.bind_owned_view_model_context(&context);
                child.advance_state_machine_instance(state_machine, 0.0);
            }
            child.advance_artboard_data_binds_with_elapsed(0.0);
            child.update_pass();
            items.push(RuntimeComponentListItemInstance {
                child: Box::new(child),
                state_machine,
                context,
                transform: Mat2D::IDENTITY,
                render_cache_revision: 0,
            });
        }

        let changed = self
            .component_list_items
            .get(&list_local_id)
            .is_none_or(|existing| existing.len() != items.len());
        self.component_list_item_transforms.insert(
            list_local_id,
            items.iter().map(|item| item.transform).collect(),
        );
        self.component_list_items.insert(list_local_id, items);
        if changed {
            self.mark_layout_changed();
            self.mark_prepared_changed();
        }
        changed
    }

    pub(crate) fn refresh_component_list_items(&mut self) -> bool {
        let Some(file) = self
            .build_context
            .as_ref()
            .map(|context| Arc::clone(&context.file))
        else {
            return false;
        };
        let updates = self
            .component_list_sources
            .iter()
            .map(|(&local_id, source)| (local_id, source.items()))
            .collect::<Vec<_>>();
        updates
            .into_iter()
            .fold(false, |changed, (local_id, items)| {
                self.sync_component_list_items(&file, local_id, items) || changed
            })
    }

    pub fn advance_state_machine_instance(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let state_machines = Arc::clone(&self.state_machines);
        let Some(state_machine) = state_machines.get(instance.state_machine_index()) else {
            return false;
        };
        instance.advance(self, state_machine, elapsed_seconds)
    }

    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.advance_nested_artboards_collect_events(elapsed_seconds, None)
    }

    pub fn try_visit_nested_artboard_instances_mut<E>(
        &mut self,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        self.try_visit_nested_artboard_instances_mut_at_depth(1, visitor)
    }

    fn try_visit_nested_artboard_instances_mut_at_depth<E>(
        &mut self,
        depth: usize,
        visitor: &mut impl FnMut(usize, u32, &mut ArtboardInstance) -> Result<(), E>,
    ) -> Result<(), E> {
        for nested in self.nested_artboards.values_mut() {
            visitor(depth, nested.child.graph_global_id, nested.child.as_mut())?;
            nested
                .child
                .try_visit_nested_artboard_instances_mut_at_depth(depth + 1, visitor)?;
        }
        Ok(())
    }

    pub fn bind_nested_artboard_owned_context_for_graph(
        &mut self,
        file: &RuntimeFile,
        graph_global_id: u32,
        context: &RuntimeOwnedViewModelInstance,
    ) -> bool {
        let mut changed = false;
        for nested in self.nested_artboards.values_mut() {
            if nested.child.graph_global_id == graph_global_id {
                let context_chain: [&[usize]; 1] = [&[]];
                changed |=
                    nested.bind_owned_view_model_animation_contexts(file, context, &context_chain);
                changed |= nested
                    .child
                    .bind_owned_view_model_artboard_context(file, context);
            } else {
                changed |= nested.child.bind_nested_artboard_owned_context_for_graph(
                    file,
                    graph_global_id,
                    context,
                );
            }
        }
        changed
    }

    pub fn set_nested_script_owned_context_for_graph(
        &mut self,
        graph_global_id: u32,
        context: RuntimeOwnedViewModelInstance,
    ) {
        self.nested_script_owned_contexts
            .insert(graph_global_id, context);
    }

    pub fn rebind_nested_script_owned_contexts(&mut self, file: &RuntimeFile) -> bool {
        let contexts = self
            .nested_script_owned_contexts
            .iter()
            .map(|(graph_global_id, context)| (*graph_global_id, context.clone()))
            .collect::<Vec<_>>();
        let mut changed = false;
        for (graph_global_id, context) in contexts {
            changed |=
                self.bind_nested_artboard_owned_context_for_graph(file, graph_global_id, &context);
        }
        changed
    }

    pub fn advance_nested_artboards_with_state_machine(
        &mut self,
        elapsed_seconds: f32,
        state_machine: &mut StateMachineInstance,
    ) -> bool {
        let mut nested_events = Vec::new();
        self.advance_nested_artboards_collect_events(elapsed_seconds, Some(&mut nested_events));
        let mut notified_state_machine = false;
        for (host_local, events) in nested_events {
            notified_state_machine |= state_machine.notify_events(self, Some(host_local), &events);
        }
        notified_state_machine
    }

    fn advance_nested_artboards_collect_events(
        &mut self,
        elapsed_seconds: f32,
        mut nested_events: Option<&mut Vec<(usize, Vec<StateMachineReportedEvent>)>>,
    ) -> bool {
        let layout_bounds = self.runtime_nested_artboard_layout_bounds();
        let mut changed = self.refresh_component_list_items();
        for index in 0..self.nested_artboard_locals.len() {
            let host_local = self.nested_artboard_locals[index];
            if self
                .component(host_local)
                .is_some_and(RuntimeComponent::is_collapsed)
            {
                continue;
            }
            changed |= self
                .apply_nested_artboard_layout_bounds(host_local, layout_bounds.as_ref().as_ref());
            let nested_needs_update = match nested_events.as_mut() {
                Some(nested_events) => {
                    let mut reported_events = Vec::new();
                    let nested_needs_update = self
                        .nested_artboards
                        .get_mut(&host_local)
                        .map(|nested| {
                            nested.advance(elapsed_seconds, Some(&mut reported_events))
                                || nested.child.has_dirt(ComponentDirt::COMPONENTS)
                        })
                        .unwrap_or(false);
                    if !reported_events.is_empty() {
                        (**nested_events).push((host_local, reported_events));
                    }
                    nested_needs_update
                }
                None => self
                    .nested_artboards
                    .get_mut(&host_local)
                    .map(|nested| {
                        nested.advance(elapsed_seconds, None)
                            || nested.child.has_dirt(ComponentDirt::COMPONENTS)
                    })
                    .unwrap_or(false),
            };
            if nested_needs_update {
                changed = true;
                self.add_dirt(host_local, ComponentDirt::COMPONENTS, false);
            }
        }
        for items in self.component_list_items.values_mut() {
            for item in items {
                if let Some(state_machine) = item.state_machine.as_mut() {
                    changed |= item
                        .child
                        .advance_state_machine_instance(state_machine, elapsed_seconds);
                }
                changed |= item
                    .child
                    .advance_artboard_data_binds_with_elapsed(elapsed_seconds);
                changed |= item.child.advance_nested_artboards(elapsed_seconds);
                changed |= item.child.update_pass();
            }
        }
        changed
    }

    fn runtime_nested_artboard_layout_bounds(
        &mut self,
    ) -> Arc<Option<BTreeMap<usize, RuntimeLayoutBounds>>> {
        let key = RuntimeNestedLayoutBoundsCacheKey {
            graph_global_id: self.graph_global_id,
            layout_epoch: self.layout_epoch,
        };
        if self
            .nested_layout_bounds
            .as_ref()
            .is_none_or(|frame| frame.key != key)
        {
            self.nested_layout_bounds = Some(RuntimeNestedLayoutBoundsFrame {
                key,
                bounds: Arc::new(self.compute_runtime_nested_artboard_layout_bounds()),
            });
        }

        self.nested_layout_bounds
            .as_ref()
            .expect("nested layout bounds frame was just populated")
            .bounds
            .clone()
    }

    fn compute_runtime_nested_artboard_layout_bounds(
        &self,
    ) -> Option<BTreeMap<usize, RuntimeLayoutBounds>> {
        if !self.nested_artboard_locals.iter().any(|local_id| {
            self.component(*local_id)
                .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        }) {
            return None;
        }
        let context = self.build_context.as_ref()?;
        let runtime = context.file.clone();
        let graph = context
            .artboards
            .iter()
            .find(|graph| graph.global_id == self.graph_global_id)?
            .clone();
        self.runtime_taffy_layout_bounds(&graph, Some(runtime.as_ref()))
    }

    fn apply_nested_artboard_layout_bounds(
        &mut self,
        host_local_id: usize,
        layout_bounds: Option<&BTreeMap<usize, RuntimeLayoutBounds>>,
    ) -> bool {
        if !self
            .component(host_local_id)
            .is_some_and(|component| component.type_name == "NestedArtboardLayout")
        {
            return false;
        }
        let Some(bounds) = layout_bounds.and_then(|bounds| bounds.get(&host_local_id).copied())
        else {
            return false;
        };
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };
        let mut changed = nested
            .child
            .set_artboard_dimensions(bounds.width, bounds.height);
        if let Some(width_key) = property_key_for_name("LayoutComponent", "width") {
            changed |= nested.child.set_double_property(0, width_key, bounds.width);
        }
        if let Some(height_key) = property_key_for_name("LayoutComponent", "height") {
            changed |= nested
                .child
                .set_double_property(0, height_key, bounds.height);
        }
        if changed {
            nested.child.update_pass();
        }
        changed
    }

    pub fn set_transform_property(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        value: f32,
    ) -> bool {
        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };
        let property_key = self.components[index].transform_property_key(property);
        let Some(property_key) = property_key else {
            return false;
        };
        self.set_transform_property_with_key(local_id, property, property_key, value)
    }

    pub(crate) fn set_transform_property_with_key(
        &mut self,
        local_id: usize,
        property: TransformProperty,
        property_key: u16,
        value: f32,
    ) -> bool {
        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };
        if !self.components[index].capabilities.transform {
            return false;
        }

        let Some(current) = self.transform_property_with_key(local_id, property, property_key)
        else {
            return false;
        };
        if current == value {
            return false;
        }
        if !self
            .objects
            .set_generated_double_property(local_id, property_key, value)
        {
            return false;
        }

        match property {
            TransformProperty::Opacity => {
                self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true);
            }
            TransformProperty::X
            | TransformProperty::Y
            | TransformProperty::Rotation
            | TransformProperty::ScaleX
            | TransformProperty::ScaleY => {
                self.add_dirt(local_id, ComponentDirt::TRANSFORM, false);
                self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
            }
        }
        true
    }

    pub fn transform_property(&self, local_id: usize, property: TransformProperty) -> Option<f32> {
        let component = self
            .component(local_id)
            .filter(|component| component.capabilities.transform)?;
        let property_key = component.transform_property_key(property)?;
        self.transform_property_with_key(local_id, property, property_key)
    }

    pub(crate) fn transform_property_with_key(
        &self,
        local_id: usize,
        property: TransformProperty,
        property_key: u16,
    ) -> Option<f32> {
        self.component(local_id)
            .filter(|component| component.capabilities.transform)?;
        Some(
            self.objects
                .double_property(local_id, property_key)
                .unwrap_or_else(|| property.default_value()),
        )
    }

    pub(crate) fn authored_transform(&self, local_id: usize) -> AuthoredTransform {
        let component = self.component(local_id);
        let (x, y) = if component.is_some_and(|component| component.type_name == "Bone") {
            (
                component
                    .and_then(|component| component.parent_local)
                    .and_then(|parent_local| self.bone_length(parent_local))
                    .unwrap_or(0.0),
                0.0,
            )
        } else {
            (
                self.transform_property(local_id, TransformProperty::X)
                    .unwrap_or_else(|| TransformProperty::X.default_value()),
                self.transform_property(local_id, TransformProperty::Y)
                    .unwrap_or_else(|| TransformProperty::Y.default_value()),
            )
        };

        AuthoredTransform {
            x,
            y,
            rotation: self
                .transform_property(local_id, TransformProperty::Rotation)
                .unwrap_or_else(|| TransformProperty::Rotation.default_value()),
            scale_x: self
                .transform_property(local_id, TransformProperty::ScaleX)
                .unwrap_or_else(|| TransformProperty::ScaleX.default_value()),
            scale_y: self
                .transform_property(local_id, TransformProperty::ScaleY)
                .unwrap_or_else(|| TransformProperty::ScaleY.default_value()),
            opacity: self
                .transform_property(local_id, TransformProperty::Opacity)
                .unwrap_or_else(|| TransformProperty::Opacity.default_value()),
        }
    }

    pub(crate) fn bone_length(&self, local_id: usize) -> Option<f32> {
        self.component(local_id).filter(|component| {
            component.type_name == "Bone" || component.type_name == "RootBone"
        })?;
        self.objects
            .double_property_by_name(local_id, "length")
            .or(Some(0.0))
    }

    pub fn has_dirt(&self, dirt: ComponentDirt) -> bool {
        self.dirt.contains(dirt)
    }

    pub fn did_change(&self) -> bool {
        self.did_change
    }

    pub(crate) fn cache_epoch(&self) -> u64 {
        self.cache_epoch
    }

    pub(crate) fn instance_identity(&self) -> u64 {
        self.instance_identity.0
    }

    pub(crate) fn prepared_epoch(&self) -> u64 {
        self.prepared_epoch
    }

    pub(crate) fn path_epoch(&self) -> u64 {
        self.path_epoch
    }

    pub(crate) fn layout_epoch(&self) -> u64 {
        self.layout_epoch
    }

    pub(crate) fn draw_order_epoch(&self) -> u64 {
        self.draw_order_epoch
    }

    fn mark_changed(&mut self) {
        self.did_change = true;
        self.cache_epoch = self.cache_epoch.wrapping_add(1);
    }

    pub(crate) fn mark_prepared_changed(&mut self) {
        self.prepared_epoch = self.prepared_epoch.wrapping_add(1);
    }

    pub(crate) fn enable_layout_constraint_bounds(&mut self) {
        if self.layout_constraint_bounds_enabled {
            return;
        }
        self.layout_constraint_bounds_enabled = true;
        self.layout_constraint_bounds = self.runtime_graph().and_then(|graph| {
            self.runtime_taffy_layout_bounds(graph, self.runtime_file())
                .map(Arc::new)
        });
        self.enqueue_artboard_parametric_layout_control_sources();
        let layout_locals = self
            .components
            .iter()
            .filter(|component| component.type_name == "LayoutComponent")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        for local_id in layout_locals {
            self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
        }
    }

    pub(crate) fn mark_layout_changed(&mut self) {
        self.layout_epoch = self.layout_epoch.wrapping_add(1);
        self.mark_prepared_changed();
    }

    pub(crate) fn mark_path_changed(&mut self) {
        self.path_epoch = self.path_epoch.wrapping_add(1);
        self.mark_prepared_changed();
    }

    fn mark_draw_order_changed(&mut self) {
        self.draw_order_epoch = self.draw_order_epoch.wrapping_add(1);
        self.mark_prepared_changed();
    }

    fn mark_render_opacity_changed(&mut self) {
        self.mark_prepared_changed();
    }

    fn mark_prepared_changed_for_property(&mut self, local_id: usize, property_key: u16) {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        if property_may_affect_prepared_frame(type_name, property_key) {
            self.mark_prepared_changed();
        }
    }

    fn mark_prepared_changed_for_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        previous: Option<u32>,
        next: u32,
    ) {
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("SolidColor")
            && solid_color_value_property_key() == Some(property_key)
        {
            self.mark_prepared_changed_for_solid_color_visibility(previous, next);
        } else {
            self.mark_prepared_changed_for_property(local_id, property_key);
        }
    }

    fn mark_prepared_changed_for_solid_color_visibility(
        &mut self,
        previous: Option<u32>,
        next: u32,
    ) {
        let next_visible = (next >> 24) != 0;
        if previous.is_none_or(|previous| ((previous >> 24) != 0) != next_visible) {
            self.mark_prepared_changed();
        }
    }

    fn mark_layout_changed_for_property(&mut self, local_id: usize, property_key: u16) {
        if self.property_affects_layout(local_id, property_key) {
            self.mark_layout_changed();
        }
    }

    fn property_affects_layout(&self, local_id: usize, property_key: u16) -> bool {
        let type_name = self.slot(local_id).and_then(|slot| slot.type_name);
        if matches!(
            type_name,
            Some("LayoutComponentStyle" | "NestedArtboardLayout")
        ) {
            return true;
        }

        if matches!(type_name, Some("Text")) {
            return [
                "alignValue",
                "sizingValue",
                "overflowValue",
                "width",
                "height",
                "verticalTrimValue",
            ]
            .into_iter()
            .any(|name| property_key_for_name("Text", name) == Some(property_key));
        }

        if matches!(type_name, Some("TextStyle" | "TextStylePaint")) {
            return ["fontSize", "lineHeight", "letterSpacing"]
                .into_iter()
                .any(|name| property_key_for_name("TextStyle", name) == Some(property_key));
        }

        if matches!(type_name, Some("TextValueRun")) {
            return property_key_for_name("TextValueRun", "text") == Some(property_key)
                || property_key_for_name("TextValueRun", "styleId") == Some(property_key);
        }

        if matches!(type_name, Some("TextInput")) {
            return property_key_for_name("TextInput", "text") == Some(property_key)
                || property_key_for_name("TextInput", "multiline") == Some(property_key);
        }

        matches!(
            type_name,
            Some("Artboard" | "LayoutComponent" | "NestedArtboard")
        ) && (property_key_for_name("Artboard", "width") == Some(property_key)
            || property_key_for_name("Artboard", "height") == Some(property_key)
            || property_key_for_name("LayoutComponent", "width") == Some(property_key)
            || property_key_for_name("LayoutComponent", "height") == Some(property_key)
            || property_key_for_name("LayoutComponent", "styleId") == Some(property_key)
            || property_key_for_name("LayoutComponent", "fractionalWidth") == Some(property_key)
            || property_key_for_name("LayoutComponent", "fractionalHeight") == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceWidthScaleType")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceHeightScaleType")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceWidthUnitsValue")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceHeightUnitsValue")
                == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceWidth") == Some(property_key)
            || property_key_for_name("NestedArtboardLayout", "instanceHeight")
                == Some(property_key))
    }

    pub fn clear_component_dirt(&mut self, local_id: usize) {
        if let Some(component) = self.component_mut(local_id) {
            component.dirt = ComponentDirt::NONE;
        }
    }

    pub fn add_dirt(&mut self, local_id: usize, dirt: ComponentDirt, recurse: bool) -> bool {
        if dirt.is_empty() {
            return false;
        }

        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };

        if self.components[index].dirt.contains(dirt) {
            return false;
        }

        self.components[index].dirt |= dirt;
        if dirt.contains(ComponentDirt::LAYOUT_STYLE) {
            self.mark_layout_changed();
        }
        if component_dirt_affects_path_epoch(dirt) {
            self.mark_path_changed();
        } else if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
            self.mark_prepared_changed();
        }
        if dirt.contains(ComponentDirt::DRAW_ORDER) {
            self.mark_draw_order_changed();
        }
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            self.mark_render_opacity_changed();
        }
        self.on_component_dirty(local_id);

        // Ported from C++ `src/bones/skin.cpp::Skin::onDirty` and
        // `src/shapes/points_path.cpp::PointsPath::markSkinDirty`.
        if self.components[index].type_name == "Skin" {
            if !dirt.contains(ComponentDirt::SKIN) {
                self.add_dirt(local_id, ComponentDirt::SKIN, false);
            }
            let skinnable_count = self.components[index].dependent_locals.len();
            for dependent_index in 0..skinnable_count {
                let skinnable_local = self.components[index].dependent_locals[dependent_index];
                self.add_dirt(skinnable_local, ComponentDirt::PATH, true);
            }
        }

        if recurse {
            // Mirrors C++ DependencyHelper::addDirtToDependents: dependency
            // edges are stable after import, so cascade without cloning.
            let dependent_count = self.components[index].dependent_locals.len();
            for dependent_index in 0..dependent_count {
                let dependent = self.components[index].dependent_locals[dependent_index];
                self.add_dirt(dependent, dirt, true);
            }
        }

        true
    }

    pub fn collapse_component(&mut self, local_id: usize, collapsed: bool) -> bool {
        let Some(index) = self.component_by_local.get(&local_id).copied() else {
            return false;
        };

        if self.components[index].is_collapsed() == collapsed {
            return false;
        }

        if collapsed {
            self.components[index].dirt |= ComponentDirt::COLLAPSED;
        } else {
            self.components[index].dirt &= !ComponentDirt::COLLAPSED;
            if self.nested_artboards.contains_key(&local_id) {
                self.newly_uncollapsed_nested_artboards.insert(local_id);
            }
        }
        self.mark_path_changed();
        self.mark_layout_changed();
        self.on_component_dirty(local_id);
        self.apply_component_collapse_changed(local_id);
        true
    }

    pub fn update_components(&mut self) -> UpdateComponentsReport {
        self.update_components_with_hook(|_, _, _| {})
    }

    pub fn update_pass(&mut self) -> bool {
        // Mirrors C++ src/artboard.cpp Artboard::updatePass: data binds run
        // before components, with artboard-host children publishing first.
        self.update_nested_artboard_data_binds_from_hosts();
        self.advance_artboard_data_binds();
        let mut did_update = false;
        if self.joysticks_apply_before_update {
            did_update |= self.apply_joysticks(true);
        }
        let mut nested_did_update = false;
        if self
            .update_components_with_hook_recording(false, |instance, local_id, dirt| {
                nested_did_update |= instance.update_nested_artboard_from_host_dirt(local_id, dirt);
            })
            .did_update
        {
            did_update = true;
        }
        did_update |= nested_did_update;
        if !self.joysticks_apply_before_update {
            let joystick_count = self.joysticks.len();
            for joystick_index in 0..joystick_count {
                let mut nested_did_update = false;
                if !self.joysticks[joystick_index].can_apply_before_update
                    && self
                        .update_components_with_hook_recording(false, |instance, local_id, dirt| {
                            nested_did_update |=
                                instance.update_nested_artboard_from_host_dirt(local_id, dirt);
                        })
                        .did_update
                {
                    did_update = true;
                }
                did_update |= nested_did_update;
                did_update |= self.apply_joystick_at(joystick_index);
            }
            let mut nested_did_update = false;
            if self
                .update_components_with_hook_recording(false, |instance, local_id, dirt| {
                    nested_did_update |=
                        instance.update_nested_artboard_from_host_dirt(local_id, dirt);
                })
                .did_update
            {
                did_update = true;
            }
            did_update |= nested_did_update;
        }
        if did_update {
            self.update_nested_artboard_data_binds_from_hosts();
            self.advance_artboard_data_binds();
        }
        did_update
    }

    fn update_nested_artboard_from_host_dirt(
        &mut self,
        host_local_id: usize,
        dirt: ComponentDirt,
    ) -> bool {
        if !dirt.contains(ComponentDirt::RENDER_OPACITY)
            && !dirt.contains(ComponentDirt::COMPONENTS)
        {
            return false;
        }
        let mut changed = false;
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            changed |= self.sync_nested_artboard_root_opacity(host_local_id);
        }
        if dirt.contains(ComponentDirt::COMPONENTS) {
            let newly_uncollapsed = self
                .newly_uncollapsed_nested_artboards
                .remove(&host_local_id);
            let is_remap_host = self
                .nested_artboards
                .get(&host_local_id)
                .is_some_and(|nested| {
                    nested.animations.iter().any(|animation| {
                        matches!(animation, RuntimeNestedAnimationInstance::Remap { .. })
                    })
                });
            let child_has_component_dirt = self
                .nested_artboards
                .get(&host_local_id)
                .is_some_and(|nested| nested.child.has_dirt(ComponentDirt::COMPONENTS));
            let host_has_data_bindings = self.has_artboard_data_bindings();
            if newly_uncollapsed
                && is_remap_host
                && dirt.contains(ComponentDirt::RENDER_OPACITY)
                && (!child_has_component_dirt || !host_has_data_bindings)
            {
                return changed;
            }
            if let Some(nested) = self.nested_artboards.get_mut(&host_local_id) {
                changed |= nested.child.update_pass();
            }
        }
        changed
    }

    fn has_artboard_data_bindings(&self) -> bool {
        !self.artboard_property_bindings.is_empty()
            || !self.artboard_image_asset_bindings.is_empty()
            || !self.artboard_custom_property_bindings.is_empty()
            || !self.artboard_layout_computed_bindings.is_empty()
            || !self.artboard_numeric_source_bindings.is_empty()
            || !self.artboard_formula_token_bindings.is_empty()
            || !self.artboard_converter_property_bindings.is_empty()
            || !self.artboard_solo_bindings.is_empty()
            || !self.artboard_solo_source_bindings.is_empty()
            || !self.artboard_nested_host_bindings.is_empty()
            || !self.artboard_list_bindings.is_empty()
    }

    fn sync_nested_artboard_root_opacity(&mut self, host_local_id: usize) -> bool {
        let Some(host_opacity) = self
            .component(host_local_id)
            .map(|component| component.transform.render_opacity)
        else {
            return false;
        };
        let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
            return false;
        };
        nested.set_root_opacity(host_opacity)
    }

    pub fn update_components_with_hook<F>(&mut self, mut hook: F) -> UpdateComponentsReport
    where
        F: FnMut(&mut Self, usize, ComponentDirt),
    {
        self.update_components_with_hook_recording(true, |instance, local_id, dirt| {
            hook(instance, local_id, dirt);
        })
    }

    fn update_components_with_hook_recording<F>(
        &mut self,
        record_updated_locals: bool,
        mut hook: F,
    ) -> UpdateComponentsReport
    where
        F: FnMut(&mut Self, usize, ComponentDirt),
    {
        let mut report = UpdateComponentsReport::default();
        if !self.has_dirt(ComponentDirt::COMPONENTS) {
            return report;
        }

        report.did_update = true;
        let max_steps = 100;
        let update_order_len = self.update_order.len();

        while self.has_dirt(ComponentDirt::COMPONENTS) && report.steps < max_steps {
            self.dirt &= !ComponentDirt::COMPONENTS;

            for order_index in 0..update_order_len {
                let local_id = self.update_order[order_index];
                self.dirt_depth = order_index;
                let Some(component_index) = self.component_by_local.get(&local_id).copied() else {
                    continue;
                };
                let dirt = self.components[component_index].dirt;
                if dirt.is_empty() || dirt.contains(ComponentDirt::COLLAPSED) {
                    continue;
                }

                self.components[component_index].dirt = ComponentDirt::NONE;
                self.update_component(component_index, dirt);
                if record_updated_locals {
                    report.updated_locals.push(local_id);
                }
                hook(self, local_id, dirt);

                if self.dirt_depth < order_index {
                    break;
                }
            }

            report.steps += 1;
        }

        report.max_steps_reached = self.has_dirt(ComponentDirt::COMPONENTS);
        report
    }

    pub(crate) fn on_component_dirty(&mut self, local_id: usize) {
        self.mark_changed();
        self.dirt |= ComponentDirt::COMPONENTS;

        let Some(component) = self.component(local_id) else {
            return;
        };
        if component.graph_order < self.dirt_depth {
            self.dirt_depth = component.graph_order;
        }
    }

    pub(crate) fn update_component(&mut self, component_index: usize, dirt: ComponentDirt) {
        let local_id = self.components[component_index].local_id;
        if dirt.contains(ComponentDirt::TRANSFORM) {
            let authored = self.authored_transform(local_id);
            self.components[component_index].update_transform(authored);
        }
        if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
            let parent_world = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.world_transform);
            self.components[component_index].update_world_transform(parent_world);
            crate::constraints::apply_constraints(self, component_index);
            crate::constraints::apply_list_constraints(self, component_index);
        }
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            let previous_opacity = self.components[component_index].transform.render_opacity;
            let opacity = self.authored_transform(local_id).opacity;
            let parent_opacity = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.render_opacity)
                .unwrap_or(1.0);
            self.components[component_index].update_render_opacity(opacity, parent_opacity);
            if self.components[component_index].transform.render_opacity != previous_opacity {
                self.mark_render_opacity_changed();
            }
        }
    }

    pub(crate) fn apply_joysticks(&mut self, can_apply_before_update: bool) -> bool {
        let mut changed = false;
        let joystick_count = self.joysticks.len();
        for joystick_index in 0..joystick_count {
            if self.joysticks[joystick_index].can_apply_before_update == can_apply_before_update {
                changed |= self.apply_joystick_at(joystick_index);
            }
        }
        changed
    }

    fn apply_joystick_at(&mut self, joystick_index: usize) -> bool {
        // Mirrors C++ Artboard::updatePass / Joystick::apply: iterate retained
        // joystick entries instead of cloning the joystick list per pass.
        let Some(joystick) = self.joysticks.get(joystick_index) else {
            return false;
        };
        let local_id = joystick.local_id;
        let x_animation_index = joystick.x_animation_index;
        let y_animation_index = joystick.y_animation_index;
        let nested_remap_dependents_len = joystick.nested_remap_dependents.len();

        let mut changed = false;
        if let Some(animation_index) = x_animation_index {
            if let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, true) {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        if let Some(animation_index) = y_animation_index {
            if let Some(seconds) = self.joystick_axis_seconds(local_id, animation_index, false) {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        for dependent_index in 0..nested_remap_dependents_len {
            let remap_local_id =
                self.joysticks[joystick_index].nested_remap_dependents[dependent_index];
            changed |= self.advance_nested_remap_animation(remap_local_id);
        }
        changed
    }

    pub(crate) fn joystick_axis_seconds(
        &self,
        local_id: usize,
        animation_index: usize,
        is_x_axis: bool,
    ) -> Option<f32> {
        let animation = self.linear_animation(animation_index)?;
        let axis_key = if is_x_axis {
            joystick_x_property_key()
        } else {
            joystick_y_property_key()
        }?;
        let flag = if is_x_axis {
            JOYSTICK_FLAG_INVERT_X
        } else {
            JOYSTICK_FLAG_INVERT_Y
        };
        let mut axis = self.double_property(local_id, axis_key).unwrap_or(0.0);
        let flags = joystick_flags_property_key()
            .and_then(|key| self.uint_property(local_id, key))
            .unwrap_or(0);
        if flags & flag != 0 {
            axis = -axis;
        }
        Some(((axis + 1.0) / 2.0) * animation.duration_seconds())
    }

    pub(crate) fn apply_uint_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        let mut changed = false;
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("NestedArtboard")
            && property_key_for_name("NestedArtboard", "artboardId") == Some(property_key)
            && let Some(value) = self.uint_property(local_id, property_key)
        {
            changed |= self.set_nested_artboard_artboard_id(local_id, value);
        }
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("DrawRules")
            && property_key_for_name("DrawRules", "drawTargetId") == Some(property_key)
        {
            changed |= self.add_dirt(local_id, ComponentDirt::DRAW_ORDER, false);
        }
        if self.slot(local_id).and_then(|slot| slot.type_name) == Some("DrawTarget")
            && property_key_for_name("DrawTarget", "placementValue") == Some(property_key)
        {
            changed |= self.add_dirt(local_id, ComponentDirt::DRAW_ORDER, false);
        }
        if solo_active_component_id_property_key() == Some(property_key) {
            changed |= self.propagate_solo_collapse(local_id);
        }
        if layout_component_style_display_value_property_key() == Some(property_key) {
            changed |= self.propagate_layout_component_display_changed(local_id);
        }
        changed |= self.apply_nested_trigger_property_changed(local_id, property_key);
        changed
    }

    fn apply_keyed_callback(&mut self, callback: RuntimeKeyedCallback) -> bool {
        let _seconds_delay = callback.seconds_delay;
        match self
            .slot(callback.target_local_id)
            .and_then(|slot| slot.type_name)
        {
            Some("CustomPropertyTrigger")
                if property_key_for_name("CustomPropertyTrigger", "fire")
                    == Some(callback.property_key) =>
            {
                let Some(property_value_key) =
                    property_key_for_name("CustomPropertyTrigger", "propertyValue")
                else {
                    return false;
                };
                let value = self
                    .uint_property(callback.target_local_id, property_value_key)
                    .unwrap_or(0)
                    + 1;
                self.set_uint_property(callback.target_local_id, property_value_key, value)
            }
            _ => false,
        }
    }

    pub(crate) fn apply_bool_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
    ) -> bool {
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("Artboard") if property_key_for_name("Artboard", "clip") == Some(property_key) => {
                if self.clip == value {
                    return false;
                }
                self.clip = value;
                true
            }
            Some("NestedArtboard")
                if property_key_for_name("NestedArtboard", "isPaused") == Some(property_key) =>
            {
                self.set_nested_artboard_is_paused(local_id, value)
            }
            Some("NestedBool")
                if property_key_for_name("NestedBool", "nestedValue") == Some(property_key) =>
            {
                let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id)
                else {
                    return false;
                };
                self.set_nested_state_machine_bool(state_machine_local_id, input_id, value)
            }
            Some("FollowPathConstraint")
                if property_key_for_name("FollowPathConstraint", "orient")
                    == Some(property_key) =>
            {
                self.mark_constraint_parent_transform_dirty(local_id)
            }
            _ => false,
        }
    }

    pub(crate) fn apply_string_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("TextValueRun")
                if property_key_for_name("TextValueRun", "text") == Some(property_key) =>
            {
                self.mark_text_value_run_shape_dirty(local_id)
            }
            _ => false,
        }
    }

    pub(crate) fn apply_color_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("GradientStop")
                if property_key_for_name("GradientStop", "colorValue") == Some(property_key) =>
            {
                self.mark_parent_gradient_stops_dirty(local_id)
            }
            _ => false,
        }
    }

    fn mark_text_value_run_shape_dirty(&mut self, run_local_id: usize) -> bool {
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return false;
        };
        let Some(text_local) = self
            .uint_property(run_local_id, parent_key)
            .and_then(|parent_id| usize::try_from(parent_id).ok())
        else {
            return false;
        };
        if self.slot(text_local).and_then(|slot| slot.type_name) != Some("Text") {
            return false;
        }

        let mut changed = false;
        changed |= self.add_dirt(text_local, ComponentDirt::TEXT_SHAPE, false);
        changed |= self.add_dirt(text_local, ComponentDirt::WORLD_TRANSFORM, true);
        changed
    }

    fn propagate_layout_component_display_changed(&mut self, style_local_id: usize) -> bool {
        if self.slot(style_local_id).and_then(|slot| slot.type_name) != Some("LayoutComponentStyle")
        {
            return false;
        }
        let Some(style_id_key) = property_key_for_name("LayoutComponent", "styleId") else {
            return false;
        };
        let layout_locals = self
            .components
            .iter()
            .filter(|component| matches!(component.type_name, "Artboard" | "LayoutComponent"))
            .filter(|component| {
                self.uint_property(component.local_id, style_id_key) == Some(style_local_id as u64)
            })
            .map(|component| component.local_id)
            .collect::<Vec<_>>();

        let mut changed = false;
        for layout_local in layout_locals {
            changed |= self.propagate_layout_component_display_collapse(layout_local);
            changed |= self.add_dirt(layout_local, ComponentDirt::LAYOUT_STYLE, false);
        }
        changed
    }

    fn apply_initial_layout_component_display_collapses(&mut self) -> bool {
        let layout_locals = self
            .components
            .iter()
            .filter(|component| matches!(component.type_name, "Artboard" | "LayoutComponent"))
            .map(|component| component.local_id)
            .collect::<Vec<_>>();

        let mut changed = false;
        for layout_local in layout_locals {
            changed |= self.propagate_layout_component_display_collapse(layout_local);
        }
        changed
    }

    fn propagate_layout_component_display_collapse(&mut self, layout_local: usize) -> bool {
        self.propagate_layout_component_display_collapse_with_ancestor(layout_local, false)
    }

    // Mirrors C++ src/layout_component.cpp LayoutComponent::propagateCollapse:
    // the propagated value folds in the local display:none state, and each
    // child receives a full-subtree collapse (ContainerComponent::collapse).
    fn propagate_layout_component_display_collapse_with_ancestor(
        &mut self,
        layout_local: usize,
        ancestor_changed: bool,
    ) -> bool {
        // Cycle guard: this and collapse_component_tree_with_ancestor recurse
        // mutually over parent_local-derived children, which a malformed-but-
        // accepted file can make cyclic -> unbounded recursion. Thread a visited
        // set (C++'s DependencySorter::visit idiom, src/dependency_sorter.cpp);
        // on a valid file every component has one parent, so each local is
        // visited at most once and the guard is a no-op.
        let mut visited = BTreeSet::new();
        self.propagate_layout_component_display_collapse_with_ancestor_guarded(
            layout_local,
            ancestor_changed,
            &mut visited,
        )
    }

    fn propagate_layout_component_display_collapse_with_ancestor_guarded(
        &mut self,
        layout_local: usize,
        ancestor_changed: bool,
        visited: &mut BTreeSet<usize>,
    ) -> bool {
        let display_hidden =
            self.layout_component_style_local(layout_local)
                .and_then(|style_local| {
                    layout_component_style_display_value_property_key()
                        .and_then(|key| self.uint_property(style_local, key))
                })
                == Some(1);
        let collapsed = display_hidden
            || self
                .component(layout_local)
                .is_some_and(RuntimeComponent::is_collapsed);
        let children = self
            .components
            .iter()
            .filter(|component| component.parent_local == Some(layout_local))
            .map(|component| component.local_id)
            .collect::<Vec<_>>();

        let mut changed = false;
        for child_local in children {
            changed |= self.collapse_component_tree_with_ancestor_guarded(
                child_local,
                collapsed,
                ancestor_changed,
                visited,
            );
        }
        changed
    }

    fn layout_component_style_local(&self, layout_local: usize) -> Option<usize> {
        property_key_for_name("LayoutComponent", "styleId")
            .and_then(|key| self.uint_property(layout_local, key))
            .and_then(|style| usize::try_from(style).ok())
    }

    pub(crate) fn apply_double_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        if let Some(property) = transform_property_for_key(property_key) {
            match property {
                TransformProperty::Opacity => {
                    self.add_dirt(local_id, ComponentDirt::RENDER_OPACITY, true);
                }
                TransformProperty::X
                | TransformProperty::Y
                | TransformProperty::Rotation
                | TransformProperty::ScaleX
                | TransformProperty::ScaleY => {
                    self.add_dirt(local_id, ComponentDirt::TRANSFORM, false);
                    self.add_dirt(local_id, ComponentDirt::WORLD_TRANSFORM, true);
                }
            }
            return true;
        }

        match self.slot(local_id).and_then(|slot| slot.type_name) {
            Some("LinearGradient" | "RadialGradient")
                if property_key_for_name("LinearGradient", "startX") == Some(property_key)
                    || property_key_for_name("LinearGradient", "startY") == Some(property_key)
                    || property_key_for_name("LinearGradient", "endX") == Some(property_key)
                    || property_key_for_name("LinearGradient", "endY") == Some(property_key) =>
            {
                self.add_dirt(local_id, ComponentDirt::TRANSFORM, false)
            }
            Some("LinearGradient" | "RadialGradient")
                if property_key_for_name("LinearGradient", "opacity") == Some(property_key) =>
            {
                self.add_dirt(local_id, ComponentDirt::PAINT, false)
            }
            Some("GradientStop")
                if property_key_for_name("GradientStop", "position") == Some(property_key) =>
            {
                self.mark_parent_gradient_stops_dirty(local_id)
            }
            Some("NestedArtboard")
                if property_key_for_name("NestedArtboard", "speed") == Some(property_key) =>
            {
                self.set_nested_artboard_speed(local_id, value)
            }
            Some("NestedArtboard")
                if property_key_for_name("NestedArtboard", "quantize") == Some(property_key) =>
            {
                self.set_nested_artboard_quantize(local_id, value)
            }
            Some("NestedNumber")
                if property_key_for_name("NestedNumber", "nestedValue") == Some(property_key) =>
            {
                let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id)
                else {
                    return false;
                };
                self.set_nested_state_machine_number(state_machine_local_id, input_id, value)
            }
            Some("NestedRemapAnimation")
                if property_key_for_name("NestedRemapAnimation", "time") == Some(property_key) =>
            {
                self.set_nested_remap_time(local_id, value)
            }
            Some("FollowPathConstraint")
                if property_key_for_name("FollowPathConstraint", "distance")
                    == Some(property_key)
                    || property_key_for_name("Constraint", "strength") == Some(property_key) =>
            {
                self.mark_constraint_parent_transform_dirty(local_id)
            }
            _ => false,
        }
    }

    fn mark_constraint_parent_transform_dirty(&mut self, constraint_local_id: usize) -> bool {
        let parent_local = self
            .component(constraint_local_id)
            .and_then(|component| component.parent_local)
            .or_else(|| {
                let parent_key = property_key_for_name("Component", "parentId")?;
                usize::try_from(self.uint_property(constraint_local_id, parent_key)?).ok()
            });
        let Some(parent_local) = parent_local else {
            return false;
        };
        let mut changed = self.add_dirt(parent_local, ComponentDirt::TRANSFORM, false);
        changed |= self.add_dirt(parent_local, ComponentDirt::WORLD_TRANSFORM, true);
        changed
    }

    fn mark_parent_gradient_stops_dirty(&mut self, stop_local_id: usize) -> bool {
        let Some(parent_key) = property_key_for_name("Component", "parentId") else {
            return false;
        };
        let Some(gradient_local_id) = self
            .uint_property(stop_local_id, parent_key)
            .and_then(|parent_id| usize::try_from(parent_id).ok())
        else {
            return false;
        };
        if !matches!(
            self.slot(gradient_local_id).and_then(|slot| slot.type_name),
            Some("LinearGradient" | "RadialGradient")
        ) {
            return false;
        }
        self.add_dirt(
            gradient_local_id,
            ComponentDirt::PAINT | ComponentDirt::STOPS,
            false,
        )
    }

    fn set_nested_artboard_is_paused(&mut self, local_id: usize, value: bool) -> bool {
        let Some(nested) = self.nested_artboards.get_mut(&local_id) else {
            return false;
        };
        nested.set_is_paused(value)
    }

    fn set_nested_artboard_speed(&mut self, local_id: usize, value: f32) -> bool {
        let Some(nested) = self.nested_artboards.get_mut(&local_id) else {
            return false;
        };
        nested.set_speed(value)
    }

    fn set_nested_artboard_quantize(&mut self, local_id: usize, value: f32) -> bool {
        let Some(nested) = self.nested_artboards.get_mut(&local_id) else {
            return false;
        };
        nested.set_quantize(value)
    }

    fn insert_nested_artboard_local(&mut self, local_id: usize) {
        if let Err(index) = self.nested_artboard_locals.binary_search(&local_id) {
            self.nested_artboard_locals.insert(index, local_id);
        }
    }

    fn remove_nested_artboard_local(&mut self, local_id: usize) {
        if let Ok(index) = self.nested_artboard_locals.binary_search(&local_id) {
            self.nested_artboard_locals.remove(index);
        }
    }

    pub(crate) fn set_nested_artboard_artboard_id(&mut self, local_id: usize, value: u64) -> bool {
        self.set_nested_artboard_artboard_id_with_force(local_id, value, false)
    }

    pub(crate) fn replace_nested_artboard_artboard_id(
        &mut self,
        local_id: usize,
        value: u64,
    ) -> bool {
        self.set_nested_artboard_artboard_id_with_force(local_id, value, true)
    }

    fn set_nested_artboard_artboard_id_with_force(
        &mut self,
        local_id: usize,
        value: u64,
        force: bool,
    ) -> bool {
        let Some(mut nested) = self.runtime_nested_artboard_instance_for_id(local_id, value) else {
            let changed = self.nested_artboards.remove(&local_id).is_some();
            if changed {
                self.remove_nested_artboard_local(local_id);
                self.artboard_owned_context_key = None;
                self.mark_changed();
                self.mark_prepared_changed();
            }
            return changed;
        };
        if !force
            && self
                .nested_artboards
                .get(&local_id)
                .is_some_and(|existing| {
                    existing.child.graph_global_id == nested.child.graph_global_id
                })
        {
            return false;
        }
        nested.render_cache_revision = self.nested_artboards.get(&local_id).map_or(0, |existing| {
            if existing.child.graph_global_id == nested.child.graph_global_id {
                existing.render_cache_revision.saturating_add(1)
            } else {
                0
            }
        });
        self.nested_artboards.insert(local_id, nested);
        self.insert_nested_artboard_local(local_id);
        self.artboard_owned_context_key = None;
        self.sync_nested_artboard_root_opacity(local_id);
        self.mark_changed();
        self.mark_prepared_changed();
        true
    }

    fn runtime_nested_artboard_instance_for_id(
        &self,
        host_local_id: usize,
        artboard_id: u64,
    ) -> Option<RuntimeNestedArtboardInstance> {
        if artboard_id == u64::from(u32::MAX) {
            return None;
        }
        let context = self.build_context.as_ref()?;
        let artboard_index = usize::try_from(artboard_id).ok()?;
        let referenced = context.file.artboard(artboard_index)?;
        let child_graph = context
            .artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced.id)?;
        if child_graph.global_id == self.graph_global_id {
            return None;
        }
        let parent_graph = context
            .artboards
            .iter()
            .find(|artboard| artboard.global_id == self.graph_global_id)?;
        let data_bind_path = self
            .slot(host_local_id)
            .and_then(|host| context.file.object(host.source_global_id as usize))
            .and_then(|host_object| {
                context
                    .file
                    .data_bind_path_for_referencer_object(host_object)
            });
        let data_bind_path_is_relative = data_bind_path
            .as_ref()
            .and_then(|path| path.object)
            .and_then(|path| path.bool_property("isRelative"))
            .unwrap_or(false);
        let data_bind_path_ids = data_bind_path.map(|path| {
            if data_bind_path_is_relative {
                path.path_ids
            } else {
                path.resolved_path_ids
            }
        });
        let mut visiting = BTreeSet::new();
        visiting.insert(self.graph_global_id);
        build_runtime_nested_artboard_instance(
            &context.file,
            parent_graph,
            context.artboards.as_slice(),
            &self.slots,
            &self.objects,
            host_local_id,
            child_graph,
            &mut visiting,
            Some(context.clone()),
            data_bind_path_ids,
            data_bind_path_is_relative,
            self.bool_property(
                host_local_id,
                property_key_for_name("NestedArtboard", "isPaused")?,
            )
            .unwrap_or(false),
            self.double_property(
                host_local_id,
                property_key_for_name("NestedArtboard", "speed")?,
            )
            .unwrap_or(1.0),
            self.double_property(
                host_local_id,
                property_key_for_name("NestedArtboard", "quantize")?,
            )
            .unwrap_or(-1.0),
        )
        .ok()
    }

    fn apply_nested_trigger_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
    ) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) != Some("NestedTrigger")
            || property_key_for_name("NestedTrigger", "fire") != Some(property_key)
        {
            return false;
        }
        let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id) else {
            return false;
        };
        self.fire_nested_state_machine_trigger(state_machine_local_id, input_id)
    }

    fn nested_input_target(&self, local_id: usize) -> Option<(usize, usize)> {
        let parent_key = property_key_for_name("Component", "parentId")?;
        let input_key = property_key_for_name("NestedInput", "inputId")?;
        let state_machine_local_id =
            usize::try_from(self.uint_property(local_id, parent_key)?).ok()?;
        let input_id = usize::try_from(self.uint_property(local_id, input_key)?).ok()?;
        Some((state_machine_local_id, input_id))
    }

    fn nested_state_machine_mut(
        &mut self,
        state_machine_local_id: usize,
    ) -> Option<&mut StateMachineInstance> {
        for nested in self.nested_artboards.values_mut() {
            for animation in &mut nested.animations {
                if let RuntimeNestedAnimationInstance::StateMachine {
                    local_id,
                    state_machine,
                } = animation
                    && *local_id == state_machine_local_id
                {
                    return Some(state_machine);
                }
            }
        }
        None
    }

    fn set_nested_state_machine_bool(
        &mut self,
        state_machine_local_id: usize,
        input_id: usize,
        value: bool,
    ) -> bool {
        self.nested_state_machine_mut(state_machine_local_id)
            .is_some_and(|state_machine| state_machine.set_bool(input_id, value))
    }

    fn set_nested_state_machine_number(
        &mut self,
        state_machine_local_id: usize,
        input_id: usize,
        value: f32,
    ) -> bool {
        self.nested_state_machine_mut(state_machine_local_id)
            .is_some_and(|state_machine| state_machine.set_number(input_id, value))
    }

    fn fire_nested_state_machine_trigger(
        &mut self,
        state_machine_local_id: usize,
        input_id: usize,
    ) -> bool {
        self.nested_state_machine_mut(state_machine_local_id)
            .is_some_and(|state_machine| state_machine.fire_trigger(input_id))
    }

    fn set_nested_remap_time(&mut self, remap_local_id: usize, time: f32) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.set_remap_time(remap_local_id, time))
    }

    fn advance_nested_remap_animation(&mut self, remap_local_id: usize) -> bool {
        self.nested_artboards
            .values_mut()
            .any(|nested| nested.advance_remap(remap_local_id))
    }

    pub(crate) fn apply_component_collapse_changed(&mut self, local_id: usize) -> bool {
        self.propagate_solo_collapse(local_id)
    }

    pub(crate) fn set_solo_active_child_by_index(
        &mut self,
        solo_local_id: usize,
        value: f32,
    ) -> bool {
        let rounded = value.round();
        if rounded < 0.0 || !rounded.is_finite() {
            return false;
        }
        let Some(solo) = self
            .solos
            .iter()
            .find(|solo| solo.local_id == solo_local_id)
            .cloned()
        else {
            return false;
        };
        let Some(child) = solo.children.get(rounded as usize) else {
            return false;
        };
        self.set_solo_active_child(&solo, child.local_id)
    }

    pub(crate) fn set_solo_active_child_by_name(
        &mut self,
        solo_local_id: usize,
        value: &[u8],
    ) -> bool {
        let Some(solo) = self
            .solos
            .iter()
            .find(|solo| solo.local_id == solo_local_id)
            .cloned()
        else {
            return false;
        };
        let Some(child) = solo.children.iter().find(|child| {
            self.slot(child.local_id)
                .and_then(|slot| slot.name.as_deref())
                .is_some_and(|name| name.as_bytes() == value)
        }) else {
            return false;
        };
        self.set_solo_active_child(&solo, child.local_id)
    }

    pub(crate) fn set_solo_active_child(
        &mut self,
        solo: &RuntimeSolo,
        child_local_id: usize,
    ) -> bool {
        let Some(cpp_local_id) =
            solo.runtime_local_by_cpp_local
                .iter()
                .find_map(|(cpp_local_id, runtime_local_id)| {
                    (*runtime_local_id == child_local_id).then_some(*cpp_local_id)
                })
        else {
            return false;
        };
        let Ok(cpp_local_id) = u64::try_from(cpp_local_id) else {
            return false;
        };
        self.set_uint_property(
            solo.local_id,
            solo.active_component_property_key,
            cpp_local_id,
        )
    }

    pub(crate) fn propagate_solo_collapse(&mut self, solo_local_id: usize) -> bool {
        let Some(solo) = self
            .solos
            .iter()
            .find(|solo| solo.local_id == solo_local_id)
            .cloned()
        else {
            return false;
        };

        let solo_collapsed = self
            .component(solo.local_id)
            .is_some_and(RuntimeComponent::is_collapsed);
        let active_local = self
            .uint_property(solo.local_id, solo.active_component_property_key)
            .and_then(|id| usize::try_from(id).ok())
            .and_then(|id| solo.runtime_local_by_cpp_local.get(&id).copied());

        let mut changed = false;
        for child in solo.children {
            let collapsed = if child.participates {
                solo_collapsed || Some(child.local_id) != active_local
            } else {
                solo_collapsed
            };
            changed |= self.collapse_component_tree(child.local_id, collapsed);
        }
        changed
    }

    pub(crate) fn collapse_component_tree(&mut self, local_id: usize, collapsed: bool) -> bool {
        self.collapse_component_tree_with_ancestor(local_id, collapsed, false)
    }

    pub(crate) fn collapse_component_tree_with_ancestor(
        &mut self,
        local_id: usize,
        collapsed: bool,
        ancestor_changed: bool,
    ) -> bool {
        // Cycle guard entry point: see
        // propagate_layout_component_display_collapse_with_ancestor.
        let mut visited = BTreeSet::new();
        self.collapse_component_tree_with_ancestor_guarded(
            local_id,
            collapsed,
            ancestor_changed,
            &mut visited,
        )
    }

    fn collapse_component_tree_with_ancestor_guarded(
        &mut self,
        local_id: usize,
        collapsed: bool,
        ancestor_changed: bool,
        visited: &mut BTreeSet<usize>,
    ) -> bool {
        // Cycle guard: see propagate_layout_component_display_collapse_with_
        // ancestor. Skip a local already visited on this propagation walk.
        if !visited.insert(local_id) {
            return false;
        }
        let changed_here = self.collapse_component(local_id, collapsed);
        let mut changed = changed_here;
        if ancestor_changed && !collapsed {
            changed |= self.add_dirt(local_id, ComponentDirt::FILTHY, false);
        }
        match self.slot(local_id).and_then(|slot| slot.type_name) {
            // C++ Solo::collapse (src/solo.cpp) intentionally skips the blind
            // ContainerComponent child walk: Solo::propagateCollapse (already
            // triggered on change via collapse_component ->
            // apply_component_collapse_changed) re-collapses inactive children
            // even while the solo itself becomes visible.
            Some("Solo") => changed,
            // C++ LayoutComponent::collapse routes through
            // LayoutComponent::propagateCollapse, folding the local
            // display:none state into the value pushed onto children.
            Some("Artboard" | "LayoutComponent") => {
                changed
                    | self.propagate_layout_component_display_collapse_with_ancestor_guarded(
                        local_id,
                        ancestor_changed || changed_here,
                        visited,
                    )
            }
            _ => {
                let children = self
                    .components
                    .iter()
                    .filter(|component| component.parent_local == Some(local_id))
                    .map(|component| component.local_id)
                    .collect::<Vec<_>>();
                for child in children {
                    changed |= self.collapse_component_tree_with_ancestor_guarded(
                        child,
                        collapsed,
                        ancestor_changed || changed_here,
                        visited,
                    );
                }
                changed
            }
        }
    }
}

impl RuntimeNestedArtboardInstance {
    pub(crate) fn bind_owned_view_model_animation_contexts(
        &mut self,
        file: &RuntimeFile,
        context: &RuntimeOwnedViewModelInstance,
        context_chain: &[&[usize]],
    ) -> bool {
        let mut changed = false;
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::StateMachine { state_machine, .. } = animation
            else {
                continue;
            };
            if state_machine.bind_owned_view_model_context_chain(file, context, context_chain) {
                changed = true;
                changed |= state_machine.advance_data_context();
            }
        }
        changed
    }

    fn advance(
        &mut self,
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        if self.is_paused {
            return false;
        }

        let local_elapsed_seconds = self.calculate_local_elapsed_seconds(elapsed_seconds);
        if local_elapsed_seconds == 0.0 && self.quantize >= 0.0 {
            return true;
        }

        let mut changed = false;
        for animation in &mut self.animations {
            changed |= animation.advance(
                &mut self.child,
                local_elapsed_seconds,
                reported_events.as_mut().map(|events| &mut **events),
            );
        }
        // Mirrors C++ src/nested_artboard.cpp NestedArtboard::updateDataBinds.
        changed |= self
            .child
            .advance_artboard_data_binds_with_elapsed(local_elapsed_seconds);
        changed |= self.child.advance_nested_artboards(local_elapsed_seconds);
        changed
    }

    // Mirrors src/nested_artboard.cpp NestedArtboard::calculateLocalElapsedSeconds.
    fn calculate_local_elapsed_seconds(&mut self, elapsed_seconds: f32) -> f32 {
        let mut local_elapsed_seconds =
            elapsed_seconds * if self.speed >= 0.0 { self.speed } else { 1.0 };
        if self.quantize >= 0.0 {
            self.cumulated_seconds += local_elapsed_seconds;
            let quantized_seconds = 1.0 / self.quantize;
            if self.cumulated_seconds > quantized_seconds {
                local_elapsed_seconds =
                    (self.cumulated_seconds / quantized_seconds).floor() * quantized_seconds;
                self.cumulated_seconds -= local_elapsed_seconds;
            } else {
                local_elapsed_seconds = 0.0;
            }
        }
        local_elapsed_seconds
    }

    fn set_root_opacity(&mut self, opacity: f32) -> bool {
        let Some(opacity_key) = property_key_for_name("Artboard", "opacity") else {
            return false;
        };
        self.child.set_double_property(0, opacity_key, opacity)
    }

    fn set_remap_time(&mut self, remap_local_id: usize, time: f32) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Remap {
                local_id,
                animation,
                ..
            } = animation
            else {
                continue;
            };
            if *local_id != remap_local_id {
                continue;
            }
            let Some(linear_animation) = self.child.linear_animation(animation.animation_index)
            else {
                return false;
            };
            let seconds = linear_animation
                .global_to_local_seconds(linear_animation.duration_seconds() * time);
            animation.set_time(linear_animation, seconds);
            return true;
        }
        false
    }

    fn advance_remap(&mut self, remap_local_id: usize) -> bool {
        for animation in &mut self.animations {
            let RuntimeNestedAnimationInstance::Remap {
                local_id,
                animation,
                mix,
            } = animation
            else {
                continue;
            };
            if *local_id != remap_local_id || *mix == 0.0 {
                continue;
            }
            return self.child.apply_linear_animation_instance(animation, *mix);
        }
        false
    }

    fn set_is_paused(&mut self, value: bool) -> bool {
        if self.is_paused == value {
            return false;
        }
        self.is_paused = value;
        true
    }

    fn set_speed(&mut self, value: f32) -> bool {
        if self.speed == value {
            return false;
        }
        self.speed = value;
        true
    }

    fn set_quantize(&mut self, value: f32) -> bool {
        if self.quantize == value {
            return false;
        }
        self.quantize = value;
        true
    }
}

impl RuntimeNestedAnimationInstance {
    fn advance(
        &mut self,
        child: &mut ArtboardInstance,
        elapsed_seconds: f32,
        mut reported_events: Option<&mut Vec<StateMachineReportedEvent>>,
    ) -> bool {
        match self {
            Self::Simple {
                animation,
                is_playing,
                speed,
                mix,
            } => {
                let mut changed = false;
                if *is_playing {
                    changed |= child
                        .advance_linear_animation_instance(animation, elapsed_seconds * *speed);
                }
                if *mix != 0.0 {
                    changed |= child.apply_linear_animation_instance(animation, *mix);
                }
                changed
            }
            Self::Remap { animation, mix, .. } => {
                if *mix == 0.0 {
                    return false;
                }
                child.apply_linear_animation_instance(animation, *mix)
            }
            Self::StateMachine { state_machine, .. } => {
                let changed = child.advance_state_machine_instance(state_machine, elapsed_seconds);
                if let Some(reported_events) = reported_events.as_mut() {
                    for index in 0..state_machine.reported_event_count() {
                        if let Some(event) = state_machine.reported_event(index) {
                            (**reported_events).push(event.clone());
                        }
                    }
                }
                changed
            }
        }
    }
}

fn build_runtime_nested_artboard_instances(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    slots: &[InstanceSlot],
    objects: &InstanceObjectArena,
    visiting: &mut BTreeSet<u32>,
    build_context: Option<RuntimeArtboardBuildContext>,
) -> Result<BTreeMap<usize, RuntimeNestedArtboardInstance>> {
    if artboards.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut nested_artboards = BTreeMap::new();
    for host in &graph.nested_artboards {
        if !matches!(
            host.type_name,
            "NestedArtboard" | "NestedArtboardLayout" | "NestedArtboardLeaf"
        ) {
            continue;
        }

        let Some(host_object) = file.object(host.global_id as usize) else {
            continue;
        };
        let Some(referenced) = file.resolved_artboard_for_referencer_object(host_object) else {
            continue;
        };
        let data_bind_path = file.data_bind_path_for_referencer_object(host_object);
        let data_bind_path_is_relative = data_bind_path
            .as_ref()
            .and_then(|path| path.object)
            .and_then(|path| path.bool_property("isRelative"))
            .unwrap_or(false);
        let data_bind_path_ids = data_bind_path.map(|path| {
            if data_bind_path_is_relative {
                path.path_ids
            } else {
                path.resolved_path_ids
            }
        });
        let Some(child_graph) = artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced.id)
        else {
            continue;
        };
        if visiting.contains(&child_graph.global_id) {
            continue;
        }

        let instance = build_runtime_nested_artboard_instance(
            file,
            graph,
            artboards,
            slots,
            objects,
            host.local_id,
            child_graph,
            visiting,
            build_context.clone(),
            data_bind_path_ids,
            data_bind_path_is_relative,
            host_object.bool_property("isPaused").unwrap_or(false),
            host_object.double_property("speed").unwrap_or(1.0),
            host_object.double_property("quantize").unwrap_or(-1.0),
        )?;
        nested_artboards.insert(host.local_id, instance);
    }

    Ok(nested_artboards)
}

fn build_runtime_nested_artboard_instance(
    file: &RuntimeFile,
    parent_graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    parent_slots: &[InstanceSlot],
    parent_objects: &InstanceObjectArena,
    host_local_id: usize,
    child_graph: &ArtboardGraph,
    visiting: &mut BTreeSet<u32>,
    build_context: Option<RuntimeArtboardBuildContext>,
    data_bind_path_ids: Option<Vec<u32>>,
    data_bind_path_is_relative: bool,
    is_paused: bool,
    speed: f32,
    quantize: f32,
) -> Result<RuntimeNestedArtboardInstance> {
    let mut child = Box::new(ArtboardInstance::from_graph_inner(
        file,
        child_graph,
        artboards,
        visiting,
        build_context,
        false,
    )?);
    child.bind_default_view_model_artboard_list_context(file);
    if !child_has_state_machine_data_binds(file, child_graph) {
        child.clear_default_text_property_context();
    }
    let animations = runtime_nested_animation_instances(file, parent_graph, host_local_id, &child);
    let data_bind_view_model_instance_locals_by_id =
        build_nested_host_view_model_instance_locals(parent_slots, parent_objects, host_local_id);
    let stateful_view_model_instance_local = file
        .object(child_graph.global_id as usize)
        .and_then(|artboard| artboard.uint_property("viewModelId"))
        .and_then(|view_model_id| u32::try_from(view_model_id).ok())
        .and_then(|view_model_id| {
            data_bind_view_model_instance_locals_by_id
                .get(&view_model_id)
                .copied()
        });
    let stateful_view_model_context = stateful_view_model_instance_local.and_then(|local_id| {
        let slot = parent_slots.iter().find(|slot| slot.local_id == local_id)?;
        let instance = file.object(slot.source_global_id as usize)?;
        let view_model_index = usize::try_from(instance.uint_property("viewModelId")?).ok()?;
        RuntimeOwnedViewModelInstance::from_instance_object(file, view_model_index, instance)
    });
    let data_bind_source_locals_by_path = build_nested_host_data_bind_source_locals(
        parent_slots,
        parent_objects,
        host_local_id,
        &data_bind_view_model_instance_locals_by_id,
        &child,
    );
    let (data_bind_property_source_locals, data_bind_image_source_locals) =
        build_nested_host_data_bind_source_local_slots(&child, &data_bind_source_locals_by_path);
    Ok(RuntimeNestedArtboardInstance {
        child,
        render_cache_revision: 0,
        data_bind_path_ids,
        data_bind_path_is_relative,
        stateful_view_model_instance_local,
        stateful_view_model_context,
        data_bind_property_source_locals,
        data_bind_image_source_locals,
        data_bind_context_source_locals_by_path: data_bind_source_locals_by_path,
        animations,
        is_paused,
        speed,
        quantize,
        cumulated_seconds: 0.0,
    })
}

fn child_has_state_machine_data_binds(file: &RuntimeFile, graph: &ArtboardGraph) -> bool {
    crate::properties::artboard_index_for_graph(file, graph).is_some_and(|artboard_index| {
        file.artboard_state_machine_graphs(artboard_index)
            .into_iter()
            .any(|state_machine| !state_machine.data_binds.is_empty())
    })
}

fn runtime_nested_animation_instances(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    host_local_id: usize,
    child: &ArtboardInstance,
) -> Vec<RuntimeNestedAnimationInstance> {
    let mut animations = Vec::new();
    for local_object in &graph.local_objects {
        let Some(object) = file.object(local_object.global_id as usize) else {
            continue;
        };
        if object.uint_property("parentId") != Some(host_local_id as u64) {
            continue;
        }

        match object.type_name {
            "NestedSimpleAnimation" => {
                let Some(animation) = nested_simple_animation_instance(object, child) else {
                    continue;
                };
                animations.push(animation);
            }
            "NestedRemapAnimation" => {
                let Some(animation) =
                    nested_remap_animation_instance(local_object.local_id, object, child)
                else {
                    continue;
                };
                animations.push(animation);
            }
            "NestedStateMachine" => {
                let Some(animation) = nested_state_machine_instance(
                    file,
                    graph,
                    local_object.local_id,
                    object,
                    child,
                ) else {
                    continue;
                };
                animations.push(animation);
            }
            _ => {}
        }
    }
    animations
}

fn component_dirt_affects_path_epoch(dirt: ComponentDirt) -> bool {
    // C++ `src/shapes/path.cpp::Path::update` rebuilds raw path geometry for
    // path/nslicer dirt, and only for world-transform dirt when a deformer is
    // present. Plain transform animation is applied at draw time through the
    // shape/world transform and must not churn retained path-command storage.
    !(dirt
        & (ComponentDirt::PATH
            | ComponentDirt::VERTICES
            | ComponentDirt::LAYOUT_STYLE
            | ComponentDirt::N_SLICER))
        .is_empty()
}

fn property_affects_effect_path_epoch(type_name: Option<&str>, property_key: u16) -> bool {
    match type_name {
        Some("TrimPath") => ["start", "end", "offset", "modeValue"]
            .iter()
            .any(|name| property_key_for_name("TrimPath", name) == Some(property_key)),
        Some("DashPath") => ["offset", "offsetIsPercentage"]
            .iter()
            .any(|name| property_key_for_name("DashPath", name) == Some(property_key)),
        Some("Dash") => ["length", "lengthIsPercentage"]
            .iter()
            .any(|name| property_key_for_name("Dash", name) == Some(property_key)),
        Some("Feather") => ["spaceValue", "strength", "offsetX", "offsetY", "inner"]
            .iter()
            .any(|name| property_key_for_name("Feather", name) == Some(property_key)),
        _ => false,
    }
}

fn property_may_affect_prepared_frame(type_name: Option<&str>, property_key: u16) -> bool {
    let Some(type_name) = type_name else {
        return true;
    };

    if matches!(
        type_name,
        "NestedNumber"
            | "NestedBool"
            | "NestedTrigger"
            | "NestedInput"
            | "NestedRemapAnimation"
            | "NestedSimpleAnimation"
            | "NestedStateMachine"
            | "StateMachine"
            | "StateMachineLayer"
            | "StateMachineNumber"
            | "StateMachineBool"
            | "StateMachineTrigger"
            | "AnimationState"
            | "AnyState"
            | "EntryState"
            | "ExitState"
            | "StateTransition"
            | "TransitionNumberCondition"
            | "TransitionBoolCondition"
            | "TransitionTriggerCondition"
            | "TransitionValueNumberComparator"
            | "TransitionValueBooleanComparator"
            | "TransitionPropertyArtboardComparator"
            | "TransitionArtboardCondition"
            | "BlendStateDirect"
            | "BlendState1D"
            | "BlendAnimationDirect"
            | "BlendAnimation1D"
            | "BlendStateTransition"
            | "BlendState1DInput"
            | "LinearAnimation"
            | "KeyedObject"
            | "KeyedProperty"
            | "KeyFrameDouble"
            | "KeyFrameColor"
            | "KeyFrameBool"
            | "KeyFrameString"
            | "KeyFrameId"
            | "ListenerTriggerChange"
            | "ListenerAlignTarget"
            | "StateMachineListener"
            | "StateMachineListenerSingle"
            | "FileAssetContents"
            | "FontAsset"
            | "ScriptAsset"
            | "ScriptedDrawable"
            | "ScriptedTransitionCondition"
    ) {
        return false;
    }

    if type_name.starts_with("ViewModel")
        || type_name.starts_with("DataBind")
        || type_name.starts_with("DataConverter")
        || type_name.starts_with("DataEnum")
        || type_name.starts_with("BindableProperty")
        || type_name.starts_with("CustomProperty")
    {
        return false;
    }

    if type_name == "NestedArtboard" {
        return property_key_for_name("NestedArtboard", "artboardId") == Some(property_key);
    }

    // C++ src/shapes/paint/solid_color.cpp updates the retained RenderPaint.
    if type_name == "SolidColor" {
        return solid_color_value_property_key() != Some(property_key);
    }

    true
}

fn nested_simple_animation_instance(
    object: &nuxie_binary::RuntimeObject,
    child: &ArtboardInstance,
) -> Option<RuntimeNestedAnimationInstance> {
    let animation_index = usize::try_from(object.uint_property("animationId")?).ok()?;
    Some(RuntimeNestedAnimationInstance::Simple {
        animation: child.linear_animation_instance(animation_index)?,
        is_playing: object.bool_property("isPlaying").unwrap_or(false),
        speed: object.double_property("speed").unwrap_or(1.0),
        mix: object.double_property("mix").unwrap_or(1.0),
    })
}

fn nested_remap_animation_instance(
    local_id: usize,
    object: &nuxie_binary::RuntimeObject,
    child: &ArtboardInstance,
) -> Option<RuntimeNestedAnimationInstance> {
    let animation_index = usize::try_from(object.uint_property("animationId")?).ok()?;
    let linear_animation = child.linear_animation(animation_index)?;
    let mut animation = child.linear_animation_instance(animation_index)?;
    let time = object.double_property("time").unwrap_or(0.0);
    let seconds =
        linear_animation.global_to_local_seconds(linear_animation.duration_seconds() * time);
    animation.set_time(linear_animation, seconds);
    Some(RuntimeNestedAnimationInstance::Remap {
        local_id,
        animation,
        mix: object.double_property("mix").unwrap_or(1.0),
    })
}

fn nested_state_machine_instance(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
    object: &nuxie_binary::RuntimeObject,
    child: &ArtboardInstance,
) -> Option<RuntimeNestedAnimationInstance> {
    let state_machine_index = usize::try_from(object.uint_property("animationId")?).ok()?;
    let mut state_machine = child.state_machine_instance(state_machine_index)?;
    state_machine.bind_default_view_model_context();
    state_machine.advance_data_context();
    apply_authored_nested_input_values(file, graph, local_id, &mut state_machine);
    Some(RuntimeNestedAnimationInstance::StateMachine {
        local_id,
        state_machine,
    })
}

fn apply_authored_nested_input_values(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    state_machine_local_id: usize,
    state_machine: &mut StateMachineInstance,
) -> bool {
    let mut changed = false;
    for local_object in &graph.local_objects {
        let Some(object) = file.object(local_object.global_id as usize) else {
            continue;
        };
        if object.uint_property("parentId") != Some(state_machine_local_id as u64) {
            continue;
        }
        let Some(input_id) = object
            .uint_property("inputId")
            .and_then(|input_id| usize::try_from(input_id).ok())
        else {
            continue;
        };
        match object.type_name {
            "NestedBool" => {
                if state_machine
                    .input(input_id)
                    .is_some_and(|input| input.kind() == StateMachineInputKind::Bool)
                {
                    changed |= state_machine.set_bool(
                        input_id,
                        object.bool_property("nestedValue").unwrap_or(false),
                    );
                }
            }
            "NestedNumber" => {
                if state_machine
                    .input(input_id)
                    .is_some_and(|input| input.kind() == StateMachineInputKind::Number)
                {
                    changed |= state_machine.set_number(
                        input_id,
                        object.double_property("nestedValue").unwrap_or(0.0),
                    );
                }
            }
            _ => {}
        }
    }
    changed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Mat2D;
    use crate::components::{RuntimeComponentCapabilities, TransformRuntimeState};
    use crate::data_bind_graph::{
        RuntimeDataBindGraphConverter, runtime_data_bind_graph_reverse_convert_value,
    };
    use crate::properties::property_key_for_name;
    use nuxie_binary::{BytesValue, FieldValue, RuntimeObject, RuntimeProperty, read_runtime_file};
    use nuxie_graph::GraphFile;
    use nuxie_schema::definition_by_name;
    use std::cell::Cell;
    use std::rc::Rc;

    struct UpdateScriptInstance {
        inits: Rc<Cell<usize>>,
        updates: Rc<Cell<usize>>,
    }

    struct AdvanceScriptInstance {
        advances: Rc<Cell<usize>>,
    }

    impl ScriptInstance for AdvanceScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(method == ScriptMethod::Advance)
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            assert_eq!(method, ScriptMethod::Advance);
            assert_eq!(args.len(), 1);
            assert_eq!(args[0].as_number().map(|value| value as f32), Some(0.1));
            let count = self.advances.get() + 1;
            self.advances.set(count);
            Ok(ScriptValue::Bool(count != 2))
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    impl ScriptInstance for UpdateScriptInstance {
        fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
            Ok(matches!(method, ScriptMethod::Init | ScriptMethod::Update))
        }

        fn call_method(
            &mut self,
            method: ScriptMethod,
            _args: &[ScriptValue],
            _host: &mut dyn crate::ScriptHost,
        ) -> Result<ScriptValue, ScriptError> {
            match method {
                ScriptMethod::Init => self.inits.set(self.inits.get() + 1),
                ScriptMethod::Update => self.updates.set(self.updates.get() + 1),
                _ => {}
            }
            Ok(ScriptValue::Nil)
        }

        fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
            Ok(ScriptValue::Nil)
        }

        fn set_input(&mut self, _name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
            Ok(())
        }
    }

    fn synthetic_instance(
        components: Vec<RuntimeComponent>,
        update_order: Vec<usize>,
    ) -> ArtboardInstance {
        let component_by_local = components
            .iter()
            .enumerate()
            .map(|(index, component)| (component.local_id, index))
            .collect::<BTreeMap<_, _>>();

        let slots = components
            .iter()
            .enumerate()
            .map(|(index, component)| InstanceSlot {
                local_id: component.local_id,
                source_global_id: component.global_id,
                type_name: Some(component.type_name),
                name: None,
                component_index: Some(index),
            })
            .collect::<Vec<_>>();
        let mut runtime_objects = vec![None; slots.len()];
        for component in &components {
            if component.local_id >= runtime_objects.len() {
                runtime_objects.resize(component.local_id + 1, None);
            }
            runtime_objects[component.local_id] = Some(synthetic_runtime_object(
                component.global_id,
                component.type_name,
                Vec::new(),
            ));
        }
        let objects = InstanceObjectArena::from_runtime_objects(runtime_objects);

        ArtboardInstance {
            instance_identity: RuntimeArtboardInstanceIdentity::next(),
            width: 0.0,
            height: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
            clip: true,
            slots,
            objects,
            components,
            component_by_local,
            solos: Vec::new(),
            joysticks: Vec::new(),
            follow_path_constraints: Vec::new(),
            list_follow_path_constraints: Vec::new(),
            scroll_constraints: Vec::new(),
            component_list_item_transforms: BTreeMap::new(),
            component_list_items: BTreeMap::new(),
            component_list_sources: BTreeMap::new(),
            ik_constraints: Vec::new(),
            joysticks_apply_before_update: true,
            update_order,
            linear_animations: Vec::new(),
            state_machines: Arc::new(Vec::new()),
            script_instances_by_global: BTreeMap::new(),
            scripted_data_converter_instances_by_global: BTreeMap::new(),
            nested_script_owned_contexts: BTreeMap::new(),
            script_path_effect_globals: BTreeSet::new(),
            script_advances_active: BTreeSet::new(),
            script_updates_pending: BTreeSet::new(),
            nested_artboards: BTreeMap::new(),
            nested_artboard_locals: Vec::new(),
            newly_uncollapsed_nested_artboards: BTreeSet::new(),
            graph_global_id: 0,
            build_context: None,
            nested_layout_bounds: None,
            artboard_data_bind_values: BTreeMap::new(),
            artboard_formula_random_source: RuntimeDataBindGraphFormulaRandomSource::default(),
            artboard_owned_context_key: None,
            artboard_property_bindings: Vec::new(),
            artboard_image_asset_bindings: Vec::new(),
            artboard_data_bind_target_queues: RuntimeArtboardDataBindTargetQueues::default(),
            artboard_data_bind_source_queues: RuntimeArtboardDataBindSourceQueues::default(),
            artboard_data_bind_suppressed_target_data_bind: None,
            artboard_custom_property_bindings: Vec::new(),
            artboard_layout_computed_bindings: Vec::new(),
            artboard_numeric_source_bindings: Vec::new(),
            artboard_formula_token_bindings: Vec::new(),
            artboard_converter_property_bindings: Vec::new(),
            artboard_solo_bindings: Vec::new(),
            artboard_solo_source_bindings: Vec::new(),
            artboard_nested_host_bindings: Vec::new(),
            artboard_list_bindings: Vec::new(),
            artboard_text_list_bindings: Vec::new(),
            artboard_context_source_values_scratch: Vec::new(),
            artboard_nested_child_context_updates_scratch: Vec::new(),
            image_asset_overrides: BTreeMap::new(),
            dirt: ComponentDirt::COMPONENTS,
            dirt_depth: 0,
            cache_epoch: 1,
            prepared_epoch: 1,
            path_epoch: 1,
            layout_epoch: 1,
            draw_order_epoch: 1,
            did_change: true,
            layout_constraint_bounds_enabled: false,
            layout_constraint_bounds: None,
        }
    }

    #[test]
    fn scripted_updates_run_once_per_attach_or_input_change() {
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        let inits = Rc::new(Cell::new(0));
        let updates = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            7,
            Box::new(UpdateScriptInstance {
                inits: Rc::clone(&inits),
                updates: Rc::clone(&updates),
            }),
        );

        assert!(instance.update_script_instances().expect("initial update"));
        assert_eq!(updates.get(), 1);
        assert!(!instance.update_script_instances().expect("clean update"));

        instance
            .set_script_input_for_global(7, "value", ScriptValue::Number(2.0))
            .expect("input update");
        assert!(instance.update_script_instances().expect("dirty update"));
        assert_eq!(updates.get(), 2);

        assert!(
            instance
                .reinitialize_script_instances()
                .expect("reinitialize")
        );
        assert_eq!(inits.get(), 1);
        assert!(
            instance
                .update_script_instances()
                .expect("post-init update")
        );
        assert_eq!(updates.get(), 3);
    }

    #[test]
    fn scripted_advances_stop_on_false_and_reactivate_on_input_change() {
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        let advances = Rc::new(Cell::new(0));
        instance.set_script_instance_for_global(
            7,
            Box::new(AdvanceScriptInstance {
                advances: Rc::clone(&advances),
            }),
        );

        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("first advance")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("second advance")
        );
        assert!(
            !instance
                .advance_script_instances(0.1)
                .expect("inactive advance")
        );
        assert_eq!(advances.get(), 2);

        instance
            .set_script_input_for_global(7, "value", ScriptValue::Number(2.0))
            .expect("input update");
        assert!(
            instance
                .advance_script_instances(0.1)
                .expect("reactivated advance")
        );
        assert_eq!(advances.get(), 3);
    }

    #[test]
    fn render_opacity_update_invalidates_a_prepared_zero_opacity_frame() {
        let mut instance = synthetic_instance(vec![synthetic_component(0, 0)], vec![0]);
        assert_eq!(instance.components[0].transform.render_opacity, 0.0);
        let prepared_epoch = instance.prepared_epoch;

        instance.update_component(0, ComponentDirt::RENDER_OPACITY);

        assert_eq!(instance.components[0].transform.render_opacity, 1.0);
        assert!(instance.prepared_epoch > prepared_epoch);
    }

    fn synthetic_component(local_id: usize, graph_order: usize) -> RuntimeComponent {
        RuntimeComponent {
            local_id,
            global_id: local_id as u32,
            type_name: "Node",
            transform_property_keys: crate::components::TransformPropertyKeys::for_type("Node"),
            capabilities: RuntimeComponentCapabilities {
                world_transform: true,
                transform: true,
            },
            parent_local: None,
            constraint_locals: Vec::new(),
            dependent_locals: Vec::new(),
            layout_chain_has_layout_component: false,
            constrained_layout_ancestor: None,
            graph_order,
            dirt: ComponentDirt::NONE,
            transform: TransformRuntimeState::default(),
        }
    }

    #[test]
    fn component_dirt_bits_match_cpp_layout() {
        assert_eq!(ComponentDirt::NONE.0, 0);
        assert_eq!(ComponentDirt::COLLAPSED.0, 1 << 0);
        assert_eq!(ComponentDirt::DEPENDENTS.0, 1 << 1);
        assert_eq!(ComponentDirt::COMPONENTS.0, 1 << 2);
        assert_eq!(ComponentDirt::DRAW_ORDER.0, 1 << 3);
        assert_eq!(ComponentDirt::PATH.0, 1 << 4);
        assert_eq!(ComponentDirt::TEXT_SHAPE.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::SKIN.0, ComponentDirt::PATH.0);
        assert_eq!(ComponentDirt::VERTICES.0, 1 << 5);
        assert_eq!(ComponentDirt::TEXT_COVERAGE.0, ComponentDirt::VERTICES.0);
        assert_eq!(ComponentDirt::TRANSFORM.0, 1 << 6);
        assert_eq!(ComponentDirt::WORLD_TRANSFORM.0, 1 << 7);
        assert_eq!(ComponentDirt::RENDER_OPACITY.0, 1 << 8);
        assert_eq!(ComponentDirt::PAINT.0, 1 << 9);
        assert_eq!(ComponentDirt::STOPS.0, 1 << 10);
        assert_eq!(ComponentDirt::LAYOUT_STYLE.0, 1 << 11);
        assert_eq!(ComponentDirt::BINDINGS.0, 1 << 12);
        assert_eq!(ComponentDirt::N_SLICER.0, 1 << 13);
        assert_eq!(ComponentDirt::SCRIPT_UPDATE.0, 1 << 14);
        assert_eq!(ComponentDirt::CLIPPING.0, 1 << 15);
        assert_eq!(ComponentDirt::FILTHY.0, 0xFFFE);
    }

    #[test]
    fn range_mapper_reverse_conversion_swaps_input_and_output_ranges() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            global_id: 0,
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 0,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 6.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 6, got {value}"
        );
    }

    #[test]
    fn range_mapper_reverse_conversion_preserves_reverse_flag() {
        let converter = RuntimeDataBindGraphConverter::RangeMapper {
            global_id: 0,
            min_input: 0.0,
            max_input: 10.0,
            min_output: 100.0,
            max_output: 200.0,
            flags: 1 << 3,
            interpolation_type: 1,
            interpolator: None,
        };

        let Some(RuntimeDataBindGraphValue::Number(value)) =
            runtime_data_bind_graph_reverse_convert_value(
                &converter,
                &RuntimeDataBindGraphValue::Number(160.0),
            )
        else {
            panic!("range mapper reverse conversion did not return a number");
        };

        assert!(
            (value - 4.0).abs() <= 0.0001,
            "range mapper reverse conversion mismatch: expected 4, got {value}"
        );
    }

    #[test]
    fn add_dirt_recurses_to_graph_dependents() {
        let mut source = synthetic_component(0, 0);
        source.dependent_locals.push(1);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);

        assert!(instance.add_dirt(0, ComponentDirt::PATH, true));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));

        assert!(!instance.add_dirt(0, ComponentDirt::PATH, true));
    }

    #[test]
    fn enabling_layout_constraint_bounds_dirties_layout_dependents() {
        let mut layout = synthetic_component(0, 0);
        layout.type_name = "LayoutComponent";
        layout.dependent_locals.push(1);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![layout, dependent], vec![0, 1]);
        instance.dirt = ComponentDirt::NONE;
        let prepared_epoch = instance.prepared_epoch();

        instance.enable_layout_constraint_bounds();

        assert!(instance.layout_constraint_bounds_enabled);
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(instance.prepared_epoch() > prepared_epoch);

        let cache_epoch = instance.cache_epoch();
        instance.enable_layout_constraint_bounds();
        assert_eq!(instance.cache_epoch(), cache_epoch);
    }

    #[test]
    fn path_epoch_tracks_path_dirt_separately_from_draw_cache_epoch() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_path_epoch = instance.path_epoch();
        let initial_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert!(instance.cache_epoch() > initial_cache_epoch);

        let paint_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PATH, false));
        assert!(instance.path_epoch() > initial_path_epoch);
        assert!(instance.cache_epoch() > paint_cache_epoch);

        let path_epoch = instance.path_epoch();
        assert!(!instance.add_dirt(0, ComponentDirt::PATH, false));
        assert_eq!(instance.path_epoch(), path_epoch);

        assert!(instance.collapse_component(0, true));
        assert!(instance.path_epoch() > path_epoch);
    }

    #[test]
    fn world_transform_dirt_invalidates_prepared_frame_without_rebuilding_paths() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_path_epoch = instance.path_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.add_dirt(0, ComponentDirt::WORLD_TRANSFORM, false));

        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert!(instance.prepared_epoch() > initial_prepared_epoch);
    }

    #[test]
    fn path_epoch_tracks_effect_path_property_changes() {
        let mut trim = synthetic_component(0, 0);
        trim.type_name = "TrimPath";
        let mut instance = synthetic_instance(vec![trim], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "TrimPath", Vec::new()),
        )]);
        let trim_start = property_key_for_name("TrimPath", "start").expect("TrimPath.start");
        let trim_mode = property_key_for_name("TrimPath", "modeValue").expect("TrimPath.modeValue");

        let mut path_epoch = instance.path_epoch();
        assert!(instance.set_double_property(0, trim_start, 0.25));
        assert!(instance.path_epoch() > path_epoch);

        path_epoch = instance.path_epoch();
        assert!(instance.set_uint_property(0, trim_mode, 2));
        assert!(instance.path_epoch() > path_epoch);

        let mut dash_path = synthetic_component(0, 0);
        dash_path.type_name = "DashPath";
        let mut instance = synthetic_instance(vec![dash_path], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "DashPath", Vec::new()),
        )]);
        let offset_is_percentage = property_key_for_name("DashPath", "offsetIsPercentage")
            .expect("DashPath.offsetIsPercentage");

        path_epoch = instance.path_epoch();
        assert!(instance.set_bool_property(0, offset_is_percentage, true));
        assert!(instance.path_epoch() > path_epoch);

        let mut dash = synthetic_component(0, 0);
        dash.type_name = "Dash";
        let mut instance = synthetic_instance(vec![dash], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "Dash", Vec::new()),
        )]);
        let length = property_key_for_name("Dash", "length").expect("Dash.length");

        path_epoch = instance.path_epoch();
        assert!(instance.set_double_property(0, length, 4.0));
        assert!(instance.path_epoch() > path_epoch);

        let mut feather = synthetic_component(0, 0);
        feather.type_name = "Feather";
        let mut instance = synthetic_instance(vec![feather], vec![0]);
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![Some(
            synthetic_runtime_object(0, "Feather", Vec::new()),
        )]);
        let inner = property_key_for_name("Feather", "inner").expect("Feather.inner");
        let space_value =
            property_key_for_name("Feather", "spaceValue").expect("Feather.spaceValue");

        path_epoch = instance.path_epoch();
        assert!(instance.set_bool_property(0, inner, true));
        assert!(instance.path_epoch() > path_epoch);

        path_epoch = instance.path_epoch();
        assert!(instance.set_uint_property(0, space_value, 1));
        assert!(instance.path_epoch() > path_epoch);
    }

    #[test]
    fn layout_epoch_tracks_layout_dirt_separately_from_draw_cache_epoch() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_layout_epoch = instance.layout_epoch();
        let initial_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.layout_epoch(), initial_layout_epoch);
        assert!(instance.cache_epoch() > initial_cache_epoch);

        let paint_cache_epoch = instance.cache_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        assert!(instance.layout_epoch() > initial_layout_epoch);
        assert!(instance.cache_epoch() > paint_cache_epoch);

        let layout_epoch = instance.layout_epoch();
        assert!(!instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        assert_eq!(instance.layout_epoch(), layout_epoch);
    }

    #[test]
    fn draw_order_epoch_tracks_draw_order_dirt() {
        let component = synthetic_component(0, 0);
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let initial_draw_order_epoch = instance.draw_order_epoch();
        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        assert_eq!(instance.draw_order_epoch(), initial_draw_order_epoch);

        assert!(instance.add_dirt(0, ComponentDirt::DRAW_ORDER, false));
        assert!(instance.draw_order_epoch() > initial_draw_order_epoch);

        let draw_order_epoch = instance.draw_order_epoch();
        assert!(!instance.add_dirt(0, ComponentDirt::DRAW_ORDER, false));
        assert_eq!(instance.draw_order_epoch(), draw_order_epoch);

        instance.clear_component_dirt(0);
        assert!(instance.add_dirt(0, ComponentDirt::DRAW_ORDER, false));
        assert!(instance.draw_order_epoch() > draw_order_epoch);
    }

    #[test]
    fn solid_color_changes_keep_prepared_topology_epoch_stable() {
        let mut solid = synthetic_component(0, 0);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![solid], vec![0]);
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        instance.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "SolidColor",
                vec![RuntimeProperty {
                    key: color_key,
                    name: "colorValue",
                    owner: "SolidColor",
                    value: FieldValue::Color(0xffff_ffff),
                }],
            ))]);

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();
        let initial_path_epoch = instance.path_epoch();
        let initial_layout_epoch = instance.layout_epoch();
        let initial_draw_order_epoch = instance.draw_order_epoch();

        assert!(instance.set_color_property(0, color_key, 0xff00_ff00));

        assert!(instance.cache_epoch() > initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
        assert_eq!(instance.path_epoch(), initial_path_epoch);
        assert_eq!(instance.layout_epoch(), initial_layout_epoch);
        assert_eq!(instance.draw_order_epoch(), initial_draw_order_epoch);
    }

    #[test]
    fn solid_color_visibility_changes_invalidate_prepared_topology() {
        let mut solid = synthetic_component(0, 0);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![solid], vec![0]);
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        instance.objects =
            InstanceObjectArena::from_runtime_objects(vec![Some(synthetic_runtime_object(
                0,
                "SolidColor",
                vec![RuntimeProperty {
                    key: color_key,
                    name: "colorValue",
                    owner: "SolidColor",
                    value: FieldValue::Color(0xffff_ffff),
                }],
            ))]);

        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_color_property(0, color_key, 0x00ff_ffff));

        assert!(instance.prepared_epoch() > initial_prepared_epoch);
    }

    #[test]
    fn prepared_epoch_ignores_nested_input_proxy_value_changes() {
        let mut nested_number = synthetic_component(0, 0);
        nested_number.type_name = "NestedNumber";
        let mut instance = synthetic_instance(vec![nested_number], vec![0]);
        let nested_value =
            property_key_for_name("NestedNumber", "nestedValue").expect("NestedNumber.nestedValue");

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_double_property(0, nested_value, 1.0));

        assert!(instance.cache_epoch() > initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
    }

    #[test]
    fn prepared_epoch_ignores_nested_artboard_animation_knobs() {
        let mut nested_artboard = synthetic_component(0, 0);
        nested_artboard.type_name = "NestedArtboard";
        let mut instance = synthetic_instance(vec![nested_artboard], vec![0]);
        let speed_key = property_key_for_name("NestedArtboard", "speed").expect("speed");

        let initial_cache_epoch = instance.cache_epoch();
        let initial_prepared_epoch = instance.prepared_epoch();

        assert!(instance.set_double_property(0, speed_key, 2.0));

        assert!(instance.cache_epoch() > initial_cache_epoch);
        assert_eq!(instance.prepared_epoch(), initial_prepared_epoch);
    }

    #[test]
    fn nested_layout_bounds_cache_tracks_layout_epoch() {
        let mut host = synthetic_component(0, 0);
        host.type_name = "NestedArtboardLayout";
        let mut instance = synthetic_instance(vec![host], vec![0]);
        instance.nested_artboards.insert(
            0,
            RuntimeNestedArtboardInstance {
                child: Box::new(synthetic_instance(
                    vec![synthetic_component(10, 0)],
                    vec![10],
                )),
                render_cache_revision: 0,
                data_bind_path_ids: None,
                data_bind_path_is_relative: false,
                stateful_view_model_instance_local: None,
                stateful_view_model_context: None,
                data_bind_property_source_locals: Vec::new(),
                data_bind_image_source_locals: Vec::new(),
                data_bind_context_source_locals_by_path: BTreeMap::new(),
                animations: Vec::new(),
                is_paused: false,
                speed: 1.0,
                quantize: 0.0,
                cumulated_seconds: 0.0,
            },
        );
        instance.nested_artboard_locals.push(0);

        let first_bounds = instance.runtime_nested_artboard_layout_bounds();
        let first_frame = instance
            .nested_layout_bounds
            .as_ref()
            .expect("nested layout bounds frame")
            .clone();
        assert_eq!(first_frame.key.layout_epoch, instance.layout_epoch());
        assert!(Arc::ptr_eq(&first_bounds, &first_frame.bounds));

        assert!(instance.add_dirt(0, ComponentDirt::PAINT, false));
        let after_paint = instance.runtime_nested_artboard_layout_bounds();
        assert_eq!(
            instance
                .nested_layout_bounds
                .as_ref()
                .expect("nested layout bounds frame")
                .key
                .layout_epoch,
            instance.layout_epoch()
        );
        assert!(Arc::ptr_eq(&first_bounds, &after_paint));

        assert!(instance.add_dirt(0, ComponentDirt::LAYOUT_STYLE, false));
        let after_layout = instance.runtime_nested_artboard_layout_bounds();
        assert_eq!(
            instance
                .nested_layout_bounds
                .as_ref()
                .expect("nested layout bounds frame")
                .key
                .layout_epoch,
            instance.layout_epoch()
        );
        assert!(!Arc::ptr_eq(&first_bounds, &after_layout));
    }

    #[test]
    fn layout_epoch_tracks_cpp_layout_property_changes() {
        let mut layout = synthetic_component(0, 0);
        layout.type_name = "LayoutComponent";
        let mut text_run = synthetic_component(1, 1);
        text_run.type_name = "TextValueRun";
        let mut solid = synthetic_component(2, 2);
        solid.type_name = "SolidColor";
        let mut instance = synthetic_instance(vec![layout, text_run, solid], vec![0, 1, 2]);

        let fractional_width =
            property_key_for_name("LayoutComponent", "fractionalWidth").expect("fractional width");
        let text = property_key_for_name("TextValueRun", "text").expect("text run text");
        let color = property_key_for_name("SolidColor", "colorValue").expect("solid color");

        let mut layout_epoch = instance.layout_epoch();
        assert!(instance.set_double_property(0, fractional_width, 0.5));
        assert!(instance.layout_epoch() > layout_epoch);

        layout_epoch = instance.layout_epoch();
        assert!(instance.set_string_property(1, text, b"hello".to_vec()));
        assert!(instance.layout_epoch() > layout_epoch);

        layout_epoch = instance.layout_epoch();
        assert!(instance.set_color_property(2, color, 0xff00ff00));
        assert_eq!(instance.layout_epoch(), layout_epoch);
    }

    #[test]
    fn gradient_property_changes_mark_cpp_dirty_bits() {
        let mut gradient = synthetic_component(0, 0);
        gradient.type_name = "LinearGradient";
        let mut stop = synthetic_component(1, 1);
        stop.type_name = "GradientStop";
        let mut instance = synthetic_instance(vec![gradient, stop], vec![0, 1]);
        let parent_key = property_key_for_name("Component", "parentId").expect("parentId key");
        let start_x_key = property_key_for_name("LinearGradient", "startX").expect("startX key");
        let opacity_key = property_key_for_name("LinearGradient", "opacity").expect("opacity key");
        let stop_color_key =
            property_key_for_name("GradientStop", "colorValue").expect("stop color key");
        let stop_position_key =
            property_key_for_name("GradientStop", "position").expect("stop position key");
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "LinearGradient", Vec::new())),
            Some(synthetic_runtime_object(
                1,
                "GradientStop",
                vec![RuntimeProperty {
                    key: parent_key,
                    name: "parentId",
                    owner: "Component",
                    value: FieldValue::Uint(0),
                }],
            )),
        ]);

        instance.dirt = ComponentDirt::NONE;
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(0, start_x_key, 10.0));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::TRANSFORM)
        );

        instance.dirt = ComponentDirt::NONE;
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(0, opacity_key, 0.5));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT)
        );

        instance.dirt = ComponentDirt::NONE;
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_color_property(1, stop_color_key, 0xff00_ff00));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT | ComponentDirt::STOPS)
        );

        instance.dirt = ComponentDirt::NONE;
        instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        assert!(instance.set_double_property(1, stop_position_key, 0.25));
        assert!(
            instance
                .component(0)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PAINT | ComponentDirt::STOPS)
        );
    }

    #[test]
    fn follow_path_property_changes_dirty_the_constrained_parent_transform() {
        let mut parent = synthetic_component(0, 0);
        parent.type_name = "Node";
        let mut constraint = synthetic_component(1, 1);
        constraint.type_name = "FollowPathConstraint";
        constraint.parent_local = Some(0);
        let mut instance = synthetic_instance(vec![parent, constraint], vec![0, 1]);
        let distance_key = property_key_for_name("FollowPathConstraint", "distance")
            .expect("FollowPathConstraint.distance key");
        let orient_key = property_key_for_name("FollowPathConstraint", "orient")
            .expect("FollowPathConstraint.orient key");
        let strength_key =
            property_key_for_name("Constraint", "strength").expect("Constraint.strength key");

        fn assert_parent_transform_dirty(instance: &mut ArtboardInstance, changed: bool) {
            assert!(changed);
            assert!(
                instance
                    .component(0)
                    .unwrap()
                    .dirt
                    .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
            );
            instance.dirt = ComponentDirt::NONE;
            instance.component_mut(0).unwrap().dirt = ComponentDirt::NONE;
        }

        let changed = instance.set_double_property(1, distance_key, 0.5);
        assert_parent_transform_dirty(&mut instance, changed);
        let changed = instance.set_bool_property(1, orient_key, false);
        assert_parent_transform_dirty(&mut instance, changed);
        let changed = instance.set_double_property(1, strength_key, 0.5);
        assert_parent_transform_dirty(&mut instance, changed);
    }

    #[test]
    fn artboard_clip_property_updates_draw_cache() {
        let mut artboard = synthetic_component(0, 0);
        artboard.type_name = "Artboard";
        let mut instance = synthetic_instance(vec![artboard], vec![0]);
        let clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip key");

        assert!(instance.clip);
        assert!(instance.set_bool_property(0, clip_key, false));
        assert!(!instance.clip);
        assert!(instance.set_bool_property(0, clip_key, true));
        assert!(instance.clip);
    }

    #[test]
    fn update_components_skips_collapsed_components_without_clearing_dirt() {
        let mut first = synthetic_component(0, 0);
        first.dirt = ComponentDirt::PATH;
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH | ComponentDirt::COLLAPSED;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);

        let report = instance.update_components();

        assert!(report.did_update);
        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.component(0).unwrap().dirt, ComponentDirt::NONE);
        assert!(
            instance
                .component(1)
                .unwrap()
                .dirt
                .contains(ComponentDirt::PATH)
        );
        assert!(instance.component(1).unwrap().is_collapsed());
    }

    #[test]
    fn update_components_restarts_when_update_dirties_earlier_graph_order() {
        let first = synthetic_component(0, 0);
        let mut second = synthetic_component(1, 1);
        second.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![first, second], vec![0, 1]);
        let mut dirtied_earlier = false;

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            if local_id == 1 && !dirtied_earlier {
                dirtied_earlier = true;
                instance.add_dirt(0, ComponentDirt::PATH, false);
            }
        });

        assert_eq!(report.steps, 2);
        assert_eq!(report.updated_locals, vec![1, 0]);
        assert!(!report.max_steps_reached);
    }

    #[test]
    fn update_components_surfaces_cpp_max_pass_guard() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::PATH;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        let report = instance.update_components_with_hook(|instance, local_id, _| {
            instance.add_dirt(local_id, ComponentDirt::PATH, false);
        });

        assert_eq!(report.steps, 100);
        assert_eq!(report.updated_locals.len(), 100);
        assert!(report.max_steps_reached);
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
    }

    fn synthetic_runtime_object(
        id: u32,
        type_name: &'static str,
        properties: Vec<RuntimeProperty>,
    ) -> RuntimeObject {
        let definition = definition_by_name(type_name).expect("synthetic runtime object type");
        RuntimeObject {
            id,
            type_key: definition.type_key.int,
            type_name: definition.name,
            rust_variant: definition.rust_variant,
            properties,
            skipped_properties: Vec::new(),
        }
    }

    #[test]
    fn instance_object_arena_uses_generated_core_registry_setter_families() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Node", Vec::new())),
            Some(synthetic_runtime_object(1, "FileAssetContents", Vec::new())),
        ]);

        assert!(arena.set_double_property(0, node_x_key, 12.5));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_uint_property(0, node_x_key, 12));
        assert_eq!(arena.double_property(0, node_x_key), Some(12.5));

        assert!(!arena.set_string_property(1, bytes_key, vec![1, 2, 3]));
        assert_eq!(arena.string_property(1, bytes_key), None);
    }

    #[test]
    fn instance_object_arena_keeps_mutable_properties_in_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let source = synthetic_runtime_object(0, "Node", Vec::new());
        let mut arena = InstanceObjectArena::from_runtime_objects(vec![Some(source.clone())]);

        assert!(arena.set_double_property(0, node_x_key, 42.0));

        assert!(source.properties.is_empty());
        assert_eq!(arena.double_property(0, node_x_key), Some(42.0));
    }

    #[test]
    fn instance_object_arena_reads_generated_defaults_and_imported_fields() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let artboard_clip_key = property_key_for_name("Artboard", "clip").expect("Artboard.clip");
        let bytes_key =
            property_key_for_name("FileAssetContents", "bytes").expect("FileAssetContents.bytes");
        let arena = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(
                0,
                "Node",
                vec![RuntimeProperty {
                    key: node_x_key,
                    name: "x",
                    owner: "Node",
                    value: FieldValue::Double(7.5),
                }],
            )),
            Some(synthetic_runtime_object(1, "Artboard", Vec::new())),
            Some(synthetic_runtime_object(
                2,
                "FileAssetContents",
                vec![RuntimeProperty {
                    key: bytes_key,
                    name: "bytes",
                    owner: "FileAssetContents",
                    value: FieldValue::Bytes(BytesValue::new(vec![1, 2, 3])),
                }],
            )),
        ]);

        assert_eq!(arena.double_property(0, node_x_key), Some(7.5));
        assert_eq!(arena.bool_property(1, artboard_clip_key), Some(true));
        assert_eq!(arena.string_property(2, bytes_key), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn artboard_typed_property_reads_surface_defaults_and_reject_wrong_value_kinds() {
        let opacity_key = property_key_for_name("Shape", "opacity").expect("Shape.opacity");
        let color_key =
            property_key_for_name("SolidColor", "colorValue").expect("SolidColor.colorValue");
        let mut instance = synthetic_instance(Vec::new(), Vec::new());
        instance.objects = InstanceObjectArena::from_runtime_objects(vec![
            Some(synthetic_runtime_object(0, "Shape", Vec::new())),
            Some(synthetic_runtime_object(1, "SolidColor", Vec::new())),
        ]);

        assert_eq!(instance.double_property(0, opacity_key), Some(1.0));
        assert_eq!(instance.color_property(1, color_key), Some(0xff74_7474));
        assert_eq!(instance.color_property(0, opacity_key), None);
        assert_eq!(instance.double_property(1, color_key), None);
    }

    #[test]
    fn update_transform_reads_generated_instance_storage() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let node_scale_x_key = property_key_for_name("Node", "scaleX").expect("Node.scaleX key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::TRANSFORM;
        let mut instance = synthetic_instance(vec![component], vec![0]);

        assert!(instance.objects.set_double_property_by_name(0, "x", 8.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(0, "scaleX", 2.5)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(instance.double_property(0, node_x_key), Some(8.0));
        assert_eq!(instance.double_property(0, node_scale_x_key), Some(2.5));
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([2.5, 0.0, -0.0, 1.0, 8.0, 0.0])
        );
    }

    #[test]
    fn transform_update_matches_basic_cpp_order() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.transform.render_opacity = 0.5;
        let mut child = synthetic_component(1, 1);
        child.parent_local = Some(0);
        child.dirt = ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
            | ComponentDirt::RENDER_OPACITY;
        let mut instance = synthetic_instance(vec![root, child], vec![0, 1]);
        assert!(instance.objects.set_double_property_by_name(1, "x", 2.0));
        assert!(instance.objects.set_double_property_by_name(1, "y", 3.0));
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleX", 4.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "scaleY", 5.0)
        );
        assert!(
            instance
                .objects
                .set_double_property_by_name(1, "opacity", 0.25)
        );

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![1]);
        let child = instance.component(1).unwrap();
        assert_eq!(
            child.transform.local_transform,
            Mat2D([4.0, 0.0, -0.0, 5.0, 2.0, 3.0])
        );
        assert_eq!(
            child.transform.world_transform,
            child.transform.local_transform
        );
        assert_eq!(child.transform.render_opacity, 0.125);
    }

    #[test]
    fn transform_property_mutation_marks_instance_dirty() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;
        instance.did_change = false;

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(12.0)
        );
        assert_eq!(instance.double_property(0, node_x_key), Some(12.0));
        assert!(
            component
                .dirt
                .contains(ComponentDirt::TRANSFORM | ComponentDirt::WORLD_TRANSFORM)
        );
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(instance.did_change());

        assert!(!instance.set_transform_property(0, TransformProperty::X, 12.0));
    }

    #[test]
    fn transform_property_mutation_writes_generated_storage_by_concrete_type() {
        let node_x_key = property_key_for_name("Node", "x").expect("Node.x key");
        let vertex_x_key = property_key_for_name("StraightVertex", "x").expect("StraightVertex.x");
        let mut vertex = synthetic_component(0, 0);
        vertex.type_name = "StraightVertex";
        let mut instance = synthetic_instance(vec![vertex], vec![0]);

        assert!(instance.set_transform_property(0, TransformProperty::X, 14.0));

        assert_eq!(
            instance.transform_property(0, TransformProperty::X),
            Some(14.0)
        );
        assert_eq!(instance.double_property(0, vertex_x_key), Some(14.0));
        assert_eq!(instance.double_property(0, node_x_key), None);
    }

    #[test]
    fn transform_property_mutation_only_recurses_world_transform_to_dependents() {
        let mut source = synthetic_component(0, 0);
        source.dependent_locals.push(1);
        let dependent = synthetic_component(1, 1);
        let mut instance = synthetic_instance(vec![source, dependent], vec![0, 1]);
        instance.clear_component_dirt(0);
        instance.clear_component_dirt(1);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));

        let source = instance.component(0).unwrap();
        assert!(source.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(source.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
        let dependent = instance.component(1).unwrap();
        assert!(!dependent.dirt.contains(ComponentDirt::TRANSFORM));
        assert!(dependent.dirt.contains(ComponentDirt::WORLD_TRANSFORM));
    }

    #[test]
    fn opacity_mutation_marks_render_opacity_dirty() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_transform_property(0, TransformProperty::Opacity, 0.35));
        let component = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.35)
        );
        assert!(component.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!component.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn generic_artboard_opacity_mutation_marks_render_opacity_dirty() {
        let artboard_opacity_key =
            property_key_for_name("Artboard", "opacity").expect("Artboard.opacity key");
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![root], vec![0]);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_double_property(0, artboard_opacity_key, 0.25));

        let root = instance.component(0).unwrap();
        assert_eq!(
            instance.transform_property(0, TransformProperty::Opacity),
            Some(0.25)
        );
        assert!(root.dirt.contains(ComponentDirt::RENDER_OPACITY));
        assert!(!root.dirt.contains(ComponentDirt::TRANSFORM));
    }

    #[test]
    fn update_reads_mutated_instance_transform_state() {
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;

        assert!(instance.set_transform_property(0, TransformProperty::X, 9.0));
        assert!(instance.set_transform_property(0, TransformProperty::Y, 4.0));

        let report = instance.update_components();

        assert_eq!(report.updated_locals, vec![0]);
        assert_eq!(
            instance.component(0).unwrap().transform.local_transform,
            Mat2D([1.0, 0.0, -0.0, 1.0, 9.0, 4.0])
        );
    }

    #[test]
    fn builds_instance_from_graph_fixture() {
        let bytes = include_bytes!("../../../fixtures/graph/dependency_test.riv");
        let file = read_runtime_file(bytes).expect("fixture should import");
        let graph = GraphFile::from_runtime_file(&file).expect("fixture should graph");
        let artboard = graph.artboards.first().expect("fixture has artboard");
        let instance = ArtboardInstance::from_graph(&file, artboard).expect("instance builds");

        assert_eq!(instance.slots().len(), artboard.local_objects.len());
        assert_eq!(
            instance
                .slots()
                .iter()
                .map(|slot| (slot.local_id, slot.source_global_id, slot.type_name))
                .collect::<Vec<_>>(),
            artboard
                .local_objects
                .iter()
                .map(|object| (object.local_id, object.global_id, object.type_name))
                .collect::<Vec<_>>()
        );
        assert_eq!(instance.components().len(), artboard.components.len());
        let graph_ordered_components = artboard
            .components
            .iter()
            .filter(|component| component.graph_order.is_some())
            .count();
        assert_eq!(instance.update_order().len(), graph_ordered_components);
        assert!(instance.has_dirt(ComponentDirt::COMPONENTS));
        assert!(
            instance
                .components()
                .iter()
                .all(|component| component.dirt == ComponentDirt::FILTHY)
        );
    }

    fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn schema_type_key(type_name: &str) -> u16 {
        definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int
    }

    fn schema_property_key(type_name: &str, property_name: &str) -> u16 {
        property_key_for_name(type_name, property_name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
    }

    fn push_synthetic_object(bytes: &mut Vec<u8>, type_name: &str, properties: &[(&str, u64)]) {
        push_var_uint(bytes, u64::from(schema_type_key(type_name)));
        for (property_name, value) in properties {
            push_var_uint(
                bytes,
                u64::from(schema_property_key(type_name, property_name)),
            );
            push_var_uint(bytes, *value);
        }
        push_var_uint(bytes, 0);
    }

    fn synthetic_riv(file_id: u64, object_stream: impl FnOnce(&mut Vec<u8>)) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"RIVE");
        push_var_uint(&mut bytes, 7);
        push_var_uint(&mut bytes, 0);
        push_var_uint(&mut bytes, file_id);
        push_var_uint(&mut bytes, 0);
        object_stream(&mut bytes);
        bytes
    }

    fn instance_from_riv(bytes: &[u8]) -> ArtboardInstance {
        let file = read_runtime_file(bytes).expect("synthetic riv should import");
        let graph = GraphFile::from_runtime_file(&file).expect("synthetic riv should graph");
        let artboard = graph.artboards.first().expect("synthetic riv has artboard");
        ArtboardInstance::from_graph(&file, artboard).expect("instance builds")
    }

    fn assert_collapsed(instance: &ArtboardInstance, local_id: usize, collapsed: bool) {
        assert_eq!(
            instance
                .component(local_id)
                .unwrap_or_else(|| panic!("missing component {local_id}"))
                .is_collapsed(),
            collapsed,
            "component {local_id} collapse mismatch"
        );
    }

    // Regression for the M8 audit finding: apply_initial_solo_collapses only
    // flagged DIRECT solo children, so Solo -> Group -> Shape left the Shape
    // un-collapsed (and drawing) on a fresh instance without a state machine.
    // C++ Solo::onAddedClean recurses the full subtree (src/solo.cpp).
    #[test]
    fn initial_solo_collapse_propagates_to_deep_descendants() {
        let bytes = synthetic_riv(9601, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: solo with the first group active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            // Local 2/3: active branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 2)]);
            // Local 4/5: statically-inactive branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 4)]);
        });
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 2, false);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 4, true);
        // The deep descendant was left un-collapsed before the fix.
        assert_collapsed(&instance, 5, true);
    }

    // Regression for the M8 audit finding: collapse propagation from a
    // display:none layout recursed only into Artboard|LayoutComponent
    // children, so display:none -> Node -> Shape still drew. C++
    // LayoutComponent::propagateCollapse recurses through
    // ContainerComponent::collapse (src/layout_component.cpp).
    #[test]
    fn initial_display_none_collapse_propagates_to_deep_descendants() {
        let bytes = synthetic_riv(9602, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: hidden layout; local 2: its style with display:none.
            push_synthetic_object(bytes, "LayoutComponent", &[("parentId", 0), ("styleId", 2)]);
            push_synthetic_object(bytes, "LayoutComponentStyle", &[("displayValue", 1)]);
            // Local 3/4: plain-node chain under the hidden layout.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 3)]);
        });
        let instance = instance_from_riv(&bytes);

        assert_collapsed(&instance, 3, true);
        // The deep descendant was left un-collapsed before the fix.
        assert_collapsed(&instance, 4, true);
    }

    // Regression for the M8 audit finding: collapse_component_tree_with_ancestor
    // blindly un-collapsed descendants, clobbering a nested solo's
    // re-collapsed inactive children. C++ Solo::collapse skips the blind
    // container child walk (src/solo.cpp).
    #[test]
    fn solo_switch_preserves_nested_solo_inactive_collapse() {
        let bytes = synthetic_riv(9603, |bytes| {
            push_synthetic_object(bytes, "Backboard", &[]);
            push_synthetic_object(bytes, "Artboard", &[]);
            // Local 1: outer solo, group A (local 2) active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 0), ("activeComponentId", 2)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 3: inactive group B holding a nested solo.
            push_synthetic_object(bytes, "Node", &[("parentId", 1)]);
            // Local 4: inner solo, group C (local 5) active.
            push_synthetic_object(bytes, "Solo", &[("parentId", 3), ("activeComponentId", 5)]);
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 5)]);
            // Local 7/8: inner solo's inactive branch.
            push_synthetic_object(bytes, "Node", &[("parentId", 4)]);
            push_synthetic_object(bytes, "Shape", &[("parentId", 7)]);
        });
        let mut instance = instance_from_riv(&bytes);

        // Fresh instance: the whole inactive outer branch is collapsed.
        for local_id in 3..=8 {
            assert_collapsed(&instance, local_id, true);
        }

        // Switch the outer solo to group B (child index 1).
        assert!(instance.set_solo_active_child_by_index(1, 1.0));

        assert_collapsed(&instance, 2, true);
        assert_collapsed(&instance, 3, false);
        assert_collapsed(&instance, 4, false);
        assert_collapsed(&instance, 5, false);
        assert_collapsed(&instance, 6, false);
        // The nested solo's inactive branch must stay collapsed; the blind
        // descendant walk un-collapsed it before the fix.
        assert_collapsed(&instance, 7, true);
        assert_collapsed(&instance, 8, true);
    }
}
