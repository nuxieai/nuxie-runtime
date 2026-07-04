use std::collections::{BTreeMap, BTreeSet};

use anyhow::{Context, Result};
use rive_binary::RuntimeFile;
use rive_graph::ArtboardGraph;

use crate::animation::{
    LinearAnimationInstance, RuntimeJoystick, RuntimeLinearAnimation, build_linear_animations,
    build_runtime_joysticks,
};
use crate::artboard_data_bind::{
    RuntimeArtboardCustomPropertyBindingInstance, RuntimeArtboardListBindingInstance,
    RuntimeArtboardSoloBindingInstance, apply_artboard_unbound_color_data_bind_defaults,
    build_artboard_custom_property_bindings, build_artboard_default_view_model_values,
    build_artboard_list_bindings, build_artboard_solo_bindings,
};
use crate::components::{
    AuthoredTransform, ComponentDirt, Mat2D, RuntimeComponent, RuntimeSolo, TransformProperty,
    UpdateComponentsReport, apply_initial_solo_collapses, build_runtime_solos,
};
use crate::constraints::{
    RuntimeFollowPathConstraint, RuntimeIkConstraint, RuntimeListFollowPathConstraint,
    build_runtime_follow_path_constraints, build_runtime_ik_constraints,
    build_runtime_list_follow_path_constraints,
};
use crate::data_bind_graph::RuntimeDataBindGraphValue;
use crate::objects::{InstanceObjectArena, InstanceSlot};
use crate::properties::{
    JOYSTICK_FLAG_INVERT_X, JOYSTICK_FLAG_INVERT_Y, RuntimeArtboardDimensions,
    joystick_flags_property_key, joystick_x_property_key, joystick_y_property_key,
    property_key_for_name, solo_active_component_id_property_key,
};
use crate::state_machine::{
    RuntimeStateMachine, StateMachineInputKind, StateMachineInstance, StateMachineReportedEvent,
    build_state_machines,
};

#[derive(Debug, Clone)]
pub struct ArtboardInstance {
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
    pub(crate) component_list_item_transforms: BTreeMap<usize, Vec<Mat2D>>,
    pub(crate) ik_constraints: Vec<RuntimeIkConstraint>,
    pub(crate) joysticks_apply_before_update: bool,
    pub(crate) update_order: Vec<usize>,
    pub(crate) linear_animations: Vec<RuntimeLinearAnimation>,
    pub(crate) state_machines: Vec<RuntimeStateMachine>,
    pub(crate) nested_artboards: BTreeMap<usize, RuntimeNestedArtboardInstance>,
    pub(crate) artboard_data_bind_values: BTreeMap<Vec<u32>, RuntimeDataBindGraphValue>,
    pub(crate) artboard_custom_property_bindings: Vec<RuntimeArtboardCustomPropertyBindingInstance>,
    pub(crate) artboard_solo_bindings: Vec<RuntimeArtboardSoloBindingInstance>,
    pub(crate) artboard_list_bindings: Vec<RuntimeArtboardListBindingInstance>,
    pub(crate) dirt: ComponentDirt,
    pub(crate) dirt_depth: usize,
    pub(crate) did_change: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeNestedArtboardInstance {
    pub(crate) child: Box<ArtboardInstance>,
    animations: Vec<RuntimeNestedAnimationInstance>,
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
        Self::from_graph_inner(file, graph, &[], &mut BTreeSet::new())
    }

    pub fn from_graph_with_artboards(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
    ) -> Result<Self> {
        Self::from_graph_inner(file, graph, artboards, &mut BTreeSet::new())
    }

    fn from_graph_inner(
        file: &RuntimeFile,
        graph: &ArtboardGraph,
        artboards: &[ArtboardGraph],
        visiting: &mut BTreeSet<u32>,
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
        apply_artboard_unbound_color_data_bind_defaults(file, graph, &mut objects);

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
        let ik_constraints = build_runtime_ik_constraints(file, graph);
        let state_machines = build_state_machines(file, graph, &linear_animations);
        let artboard_data_bind_values = build_artboard_default_view_model_values(file, graph);
        let artboard_custom_property_bindings =
            build_artboard_custom_property_bindings(file, graph);
        let artboard_solo_bindings = build_artboard_solo_bindings(file, graph);
        let artboard_list_bindings = build_artboard_list_bindings(file, graph);
        apply_initial_solo_collapses(&objects, &solos, &mut components, &component_by_local);
        let nested_artboards = if inserted {
            build_runtime_nested_artboard_instances(file, graph, artboards, visiting)?
        } else {
            BTreeMap::new()
        };
        if inserted {
            visiting.remove(&graph.global_id);
        }

        Ok(Self {
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
            component_list_item_transforms: BTreeMap::new(),
            ik_constraints,
            joysticks_apply_before_update: graph.joysticks_apply_before_update,
            update_order,
            linear_animations,
            state_machines,
            nested_artboards,
            artboard_data_bind_values,
            artboard_custom_property_bindings,
            artboard_solo_bindings,
            artboard_list_bindings,
            dirt: ComponentDirt::COMPONENTS,
            dirt_depth: 0,
            did_change: true,
        })
    }

    pub fn component(&self, local_id: usize) -> Option<&RuntimeComponent> {
        self.component_by_local
            .get(&local_id)
            .map(|index| &self.components[*index])
    }

    pub fn slot(&self, local_id: usize) -> Option<&InstanceSlot> {
        self.slots.get(local_id)
    }

    pub fn slots(&self) -> &[InstanceSlot] {
        &self.slots
    }

    pub fn component_mut(&mut self, local_id: usize) -> Option<&mut RuntimeComponent> {
        let index = *self.component_by_local.get(&local_id)?;
        Some(&mut self.components[index])
    }

    pub fn components(&self) -> &[RuntimeComponent] {
        &self.components
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
        &self.state_machines
    }

    pub fn set_artboard_dimensions(&mut self, width: f32, height: f32) -> bool {
        if self.width == width && self.height == height {
            return false;
        }
        self.width = width;
        self.height = height;
        self.did_change = true;
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

    pub(crate) fn color_property(&self, local_id: usize, property_key: u16) -> Option<u32> {
        self.objects.color_property(local_id, property_key)
    }

    pub(crate) fn set_color_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u32,
    ) -> bool {
        if !self
            .objects
            .set_color_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        true
    }

    pub(crate) fn bool_property(&self, local_id: usize, property_key: u16) -> Option<bool> {
        self.objects.bool_property(local_id, property_key)
    }

    pub(crate) fn set_bool_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
    ) -> bool {
        if !self
            .objects
            .set_bool_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        self.apply_bool_property_changed(local_id, property_key, value);
        true
    }

    pub(crate) fn uint_property(&self, local_id: usize, property_key: u16) -> Option<u64> {
        self.objects.uint_property(local_id, property_key)
    }

    pub(crate) fn double_property(&self, local_id: usize, property_key: u16) -> Option<f32> {
        self.objects.double_property(local_id, property_key)
    }

    pub(crate) fn set_double_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        if !self
            .objects
            .set_double_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        self.apply_double_property_changed(local_id, property_key, value);
        true
    }

    pub(crate) fn set_uint_property(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: u64,
    ) -> bool {
        if !self
            .objects
            .set_uint_property(local_id, property_key, value)
        {
            return false;
        }
        self.did_change = true;
        self.apply_uint_property_changed(local_id, property_key);
        true
    }

    pub(crate) fn string_property(&self, local_id: usize, property_key: u16) -> Option<&[u8]> {
        self.objects.string_property(local_id, property_key)
    }

    pub(crate) fn set_string_property(
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
        self.did_change = true;
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
        &self,
        instance: &mut LinearAnimationInstance,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let Some(animation) = self.linear_animation(instance.animation_index) else {
            return false;
        };
        instance.advance_with_events(animation, elapsed_seconds, reported_events)
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

    pub fn advance_state_machine_instance(
        &mut self,
        instance: &mut StateMachineInstance,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(state_machine) = self.state_machine(instance.state_machine_index()).cloned()
        else {
            return false;
        };
        instance.advance(self, &state_machine, elapsed_seconds)
    }

    pub fn advance_nested_artboards(&mut self, elapsed_seconds: f32) -> bool {
        self.nested_artboards
            .values_mut()
            .fold(false, |changed, nested| {
                changed | nested.advance(elapsed_seconds)
            })
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
        if !self.components[index].capabilities.transform {
            return false;
        }

        let Some(current) = self.transform_property(local_id, property) else {
            return false;
        };
        if current == value {
            return false;
        }
        if !self
            .objects
            .set_double_property_by_name(local_id, property.property_name(), value)
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
        self.component(local_id)
            .filter(|component| component.capabilities.transform)?;
        self.objects
            .double_property_by_name(local_id, property.property_name())
            .or_else(|| Some(property.default_value()))
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
        self.on_component_dirty(local_id);

        if recurse {
            let dependents = self.components[index].dependent_locals.clone();
            for dependent in dependents {
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
        }
        self.on_component_dirty(local_id);
        self.apply_component_collapse_changed(local_id);
        true
    }

    pub fn update_components(&mut self) -> UpdateComponentsReport {
        self.update_components_with_hook(|_, _, _| {})
    }

    pub fn update_pass(&mut self) -> bool {
        let mut did_update = false;
        if self.joysticks_apply_before_update {
            did_update |= self.apply_joysticks(true);
        }
        if self.update_components().did_update {
            did_update = true;
        }
        if !self.joysticks_apply_before_update {
            let joysticks = self.joysticks.clone();
            for joystick in joysticks {
                if !joystick.can_apply_before_update && self.update_components().did_update {
                    did_update = true;
                }
                did_update |= self.apply_joystick(&joystick);
            }
            if self.update_components().did_update {
                did_update = true;
            }
        }
        if self.update_nested_artboards() {
            did_update = true;
        }
        did_update
    }

    fn update_nested_artboards(&mut self) -> bool {
        let host_opacities = self
            .nested_artboards
            .keys()
            .filter_map(|host_local_id| {
                let opacity = self
                    .component(*host_local_id)
                    .map(|component| component.transform.render_opacity)?;
                Some((*host_local_id, opacity))
            })
            .collect::<Vec<_>>();

        let mut changed = false;
        for (host_local_id, host_opacity) in host_opacities {
            let Some(nested) = self.nested_artboards.get_mut(&host_local_id) else {
                continue;
            };
            changed |= nested.set_root_opacity(host_opacity);
            changed |= nested.child.update_pass();
        }
        changed
    }

    pub fn update_components_with_hook<F>(&mut self, mut hook: F) -> UpdateComponentsReport
    where
        F: FnMut(&mut Self, usize, ComponentDirt),
    {
        let mut report = UpdateComponentsReport::default();
        if !self.has_dirt(ComponentDirt::COMPONENTS) {
            return report;
        }

        report.did_update = true;
        let max_steps = 100;
        let update_order = self.update_order.clone();

        while self.has_dirt(ComponentDirt::COMPONENTS) && report.steps < max_steps {
            self.dirt &= !ComponentDirt::COMPONENTS;

            for (order_index, local_id) in update_order.iter().copied().enumerate() {
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
                report.updated_locals.push(local_id);
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
        self.did_change = true;
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
            let opacity = self.authored_transform(local_id).opacity;
            let parent_opacity = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.render_opacity)
                .unwrap_or(1.0);
            self.components[component_index].update_render_opacity(opacity, parent_opacity);
        }
    }

    pub(crate) fn apply_joysticks(&mut self, can_apply_before_update: bool) -> bool {
        let joysticks = self.joysticks.clone();
        joysticks
            .iter()
            .filter(|joystick| joystick.can_apply_before_update == can_apply_before_update)
            .fold(false, |changed, joystick| {
                changed | self.apply_joystick(joystick)
            })
    }

    pub(crate) fn apply_joystick(&mut self, joystick: &RuntimeJoystick) -> bool {
        let mut changed = false;
        if let Some(animation_index) = joystick.x_animation_index {
            if let Some(seconds) =
                self.joystick_axis_seconds(joystick.local_id, animation_index, true)
            {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
        }
        if let Some(animation_index) = joystick.y_animation_index {
            if let Some(seconds) =
                self.joystick_axis_seconds(joystick.local_id, animation_index, false)
            {
                changed |= self.apply_linear_animation(animation_index, seconds, 1.0);
            }
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
        if solo_active_component_id_property_key() == Some(property_key) {
            changed |= self.propagate_solo_collapse(local_id);
        }
        changed |= self.apply_nested_trigger_property_changed(local_id, property_key);
        changed
    }

    pub(crate) fn apply_bool_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: bool,
    ) -> bool {
        if self.slot(local_id).and_then(|slot| slot.type_name) != Some("NestedBool")
            || property_key_for_name("NestedBool", "nestedValue") != Some(property_key)
        {
            return false;
        }
        let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id) else {
            return false;
        };
        self.set_nested_state_machine_bool(state_machine_local_id, input_id, value)
    }

    pub(crate) fn apply_double_property_changed(
        &mut self,
        local_id: usize,
        property_key: u16,
        value: f32,
    ) -> bool {
        match self.slot(local_id).and_then(|slot| slot.type_name) {
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
            _ => false,
        }
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
        let changed_here = self.collapse_component(local_id, collapsed);
        let mut changed = changed_here;
        if ancestor_changed && !collapsed {
            changed |= self.add_dirt(local_id, ComponentDirt::FILTHY, false);
        }
        let children = self
            .components
            .iter()
            .filter(|component| component.parent_local == Some(local_id))
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        for child in children {
            changed |= self.collapse_component_tree_with_ancestor(
                child,
                collapsed,
                ancestor_changed || changed_here,
            );
        }
        changed
    }
}

impl RuntimeNestedArtboardInstance {
    fn advance(&mut self, elapsed_seconds: f32) -> bool {
        let mut changed = false;
        for animation in &mut self.animations {
            changed |= animation.advance(&mut self.child, elapsed_seconds);
        }
        changed |= self.child.advance_nested_artboards(elapsed_seconds);
        changed
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
}

impl RuntimeNestedAnimationInstance {
    fn advance(&mut self, child: &mut ArtboardInstance, elapsed_seconds: f32) -> bool {
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
                child.advance_state_machine_instance(state_machine, elapsed_seconds)
            }
        }
    }
}

fn build_runtime_nested_artboard_instances(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboards: &[ArtboardGraph],
    visiting: &mut BTreeSet<u32>,
) -> Result<BTreeMap<usize, RuntimeNestedArtboardInstance>> {
    if artboards.is_empty() {
        return Ok(BTreeMap::new());
    }

    let mut nested_artboards = BTreeMap::new();
    for host in &graph.nested_artboards {
        if host.type_name != "NestedArtboard" {
            continue;
        }

        let Some(host_object) = file.object(host.global_id as usize) else {
            continue;
        };
        let Some(referenced) = file.resolved_artboard_for_referencer_object(host_object) else {
            continue;
        };
        let Some(child_graph) = artboards
            .iter()
            .find(|artboard| artboard.global_id == referenced.id)
        else {
            continue;
        };
        if visiting.contains(&child_graph.global_id) {
            continue;
        }

        let mut child = Box::new(ArtboardInstance::from_graph_inner(
            file,
            child_graph,
            artboards,
            visiting,
        )?);
        child.bind_default_view_model_artboard_list_context(file);
        let animations = runtime_nested_animation_instances(file, graph, host.local_id, &child);
        nested_artboards.insert(
            host.local_id,
            RuntimeNestedArtboardInstance { child, animations },
        );
    }

    Ok(nested_artboards)
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

fn nested_simple_animation_instance(
    object: &rive_binary::RuntimeObject,
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
    object: &rive_binary::RuntimeObject,
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
    object: &rive_binary::RuntimeObject,
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
    use rive_binary::{BytesValue, FieldValue, RuntimeObject, RuntimeProperty, read_runtime_file};
    use rive_graph::GraphFile;
    use rive_schema::definition_by_name;

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
            component_list_item_transforms: BTreeMap::new(),
            ik_constraints: Vec::new(),
            joysticks_apply_before_update: true,
            update_order,
            linear_animations: Vec::new(),
            state_machines: Vec::new(),
            nested_artboards: BTreeMap::new(),
            artboard_data_bind_values: BTreeMap::new(),
            artboard_custom_property_bindings: Vec::new(),
            artboard_solo_bindings: Vec::new(),
            artboard_list_bindings: Vec::new(),
            dirt: ComponentDirt::COMPONENTS,
            dirt_depth: 0,
            did_change: true,
        }
    }

    fn synthetic_component(local_id: usize, graph_order: usize) -> RuntimeComponent {
        RuntimeComponent {
            local_id,
            global_id: local_id as u32,
            type_name: "Node",
            capabilities: RuntimeComponentCapabilities {
                world_transform: true,
                transform: true,
            },
            parent_local: None,
            constraint_locals: Vec::new(),
            dependent_locals: Vec::new(),
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
}
