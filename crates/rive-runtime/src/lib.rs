use anyhow::{Context, Result};
use rive_binary::{RuntimeFile, RuntimeImportStatus, RuntimeObject};
use rive_graph::{ArtboardGraph, ComponentNode};
use rive_schema::{definition_by_name, object_supports_property};
use std::collections::BTreeMap;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

#[derive(Debug, Clone)]
pub struct ArtboardInstance {
    slots: Vec<InstanceSlot>,
    components: Vec<RuntimeComponent>,
    component_by_local: BTreeMap<usize, usize>,
    update_order: Vec<usize>,
    linear_animations: Vec<RuntimeLinearAnimation>,
    state_machines: Vec<RuntimeStateMachine>,
    dirt: ComponentDirt,
    dirt_depth: usize,
    did_change: bool,
}

impl ArtboardInstance {
    pub fn from_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Result<Self> {
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

        let mut component_by_local = BTreeMap::new();
        let mut components = Vec::new();

        for component in &graph.components {
            let object = file.object(component.global_id as usize).with_context(|| {
                format!("component global id {} is missing", component.global_id)
            })?;

            component_by_local.insert(component.local_id, components.len());
            components.push(RuntimeComponent::from_graph_component(component, object));
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
        let linear_animations = build_linear_animations(file, graph, &slots);
        let state_machines = build_state_machines(file, graph, &linear_animations);

        Ok(Self {
            slots,
            components,
            component_by_local,
            update_order,
            linear_animations,
            state_machines,
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
        let Some(state_machine) = self.state_machine(instance.state_machine_index).cloned() else {
            return false;
        };
        instance.advance(self, &state_machine, elapsed_seconds)
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

        if !self.components[index]
            .transform
            .set_property(property, value)
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

    fn transform_property(&self, local_id: usize, property: TransformProperty) -> Option<f32> {
        self.component(local_id)
            .filter(|component| component.capabilities.transform)
            .map(|component| component.transform.property(property))
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
        true
    }

    pub fn update_components(&mut self) -> UpdateComponentsReport {
        self.update_components_with_hook(|_, _, _| {})
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

    fn on_component_dirty(&mut self, local_id: usize) {
        self.did_change = true;
        self.dirt |= ComponentDirt::COMPONENTS;

        let Some(component) = self.component(local_id) else {
            return;
        };
        if component.graph_order < self.dirt_depth {
            self.dirt_depth = component.graph_order;
        }
    }

    fn update_component(&mut self, component_index: usize, dirt: ComponentDirt) {
        if dirt.contains(ComponentDirt::TRANSFORM) {
            self.components[component_index].update_transform();
        }
        if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
            let parent_world = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.world_transform);
            self.components[component_index].update_world_transform(parent_world);
        }
        if dirt.contains(ComponentDirt::RENDER_OPACITY) {
            let parent_opacity = self.components[component_index]
                .parent_local
                .and_then(|parent_local| self.component(parent_local))
                .filter(|parent| parent.capabilities.world_transform)
                .map(|parent| parent.transform.render_opacity)
                .unwrap_or(1.0);
            self.components[component_index].update_render_opacity(parent_opacity);
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstanceSlot {
    pub local_id: usize,
    pub source_global_id: u32,
    pub type_name: Option<&'static str>,
    pub name: Option<String>,
    pub component_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLinearAnimation {
    pub global_id: u32,
    pub name: Option<String>,
    pub fps: u64,
    pub duration: u64,
    pub speed: f32,
    pub loop_value: u64,
    pub work_start: u64,
    pub work_end: u64,
    pub enable_work_area: bool,
    pub quantize: bool,
    pub keyed_objects: Vec<RuntimeKeyedObject>,
}

impl RuntimeLinearAnimation {
    fn apply(&self, instance: &mut ArtboardInstance, seconds: f32, mix: f32) -> bool {
        let seconds = if self.quantize && self.fps != 0 {
            let fps = self.fps as f32;
            (seconds * fps).floor() / fps
        } else {
            seconds
        };

        let mut changed = false;
        for keyed_object in &self.keyed_objects {
            for keyed_property in &keyed_object.keyed_properties {
                let Some(property) = keyed_property.transform_property else {
                    continue;
                };
                let Some(current) =
                    instance.transform_property(keyed_object.target_local_id, property)
                else {
                    continue;
                };
                let Some(value) = keyed_property.value_at(seconds, self.fps, current, mix) else {
                    continue;
                };
                changed |=
                    instance.set_transform_property(keyed_object.target_local_id, property, value);
            }
        }
        changed
    }

    fn start_seconds(&self) -> f32 {
        self.frame_to_seconds(self.start_frame())
    }

    fn end_seconds(&self) -> f32 {
        self.frame_to_seconds(self.end_frame())
    }

    fn duration_seconds(&self) -> f32 {
        self.frame_to_seconds(self.duration as f32)
    }

    fn start_time_with_speed(&self, speed_multiplier: f32) -> f32 {
        if self.speed * speed_multiplier >= 0.0 {
            self.start_seconds()
        } else {
            self.end_seconds()
        }
    }

    fn fps_as_f32(&self) -> f32 {
        self.fps as f32
    }

    fn start_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_start as f32
        } else {
            0.0
        }
    }

    fn end_frame(&self) -> f32 {
        if self.enable_work_area {
            self.work_end as f32
        } else {
            self.duration as f32
        }
    }

    fn frame_to_seconds(&self, frame: f32) -> f32 {
        if self.fps == 0 {
            return 0.0;
        }
        frame / self.fps_as_f32()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AnimationLoop {
    OneShot,
    Loop,
    PingPong,
}

impl AnimationLoop {
    fn from_loop_value(value: u64) -> Self {
        match value {
            1 => Self::Loop,
            2 => Self::PingPong,
            _ => Self::OneShot,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinearAnimationInstance {
    animation_index: usize,
    time: f32,
    speed_direction: f32,
    total_time: f32,
    last_total_time: f32,
    spilled_time: f32,
    direction: f32,
    did_loop: bool,
    loop_value: Option<u64>,
}

impl LinearAnimationInstance {
    fn new(
        animation_index: usize,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> Self {
        Self {
            animation_index,
            time: animation.start_time_with_speed(speed_multiplier),
            speed_direction: if speed_multiplier >= 0.0 { 1.0 } else { -1.0 },
            total_time: 0.0,
            last_total_time: 0.0,
            spilled_time: 0.0,
            direction: 1.0,
            did_loop: false,
            loop_value: None,
        }
    }

    pub fn animation_index(&self) -> usize {
        self.animation_index
    }

    pub fn time(&self) -> f32 {
        self.time
    }

    pub fn speed_direction(&self) -> f32 {
        self.speed_direction
    }

    pub fn total_time(&self) -> f32 {
        self.total_time
    }

    pub fn last_total_time(&self) -> f32 {
        self.last_total_time
    }

    pub fn spilled_time(&self) -> f32 {
        self.spilled_time
    }

    pub fn direction(&self) -> f32 {
        self.direction
    }

    pub fn did_loop(&self) -> bool {
        self.did_loop
    }

    pub fn loop_value(&self) -> Option<u64> {
        self.loop_value
    }

    fn set_time(&mut self, animation: &RuntimeLinearAnimation, value: f32) {
        if self.time == value {
            return;
        }
        self.time = value;
        let diff = self.total_time - self.last_total_time;
        self.total_time = value - animation.start_seconds();
        self.last_total_time = self.total_time - diff;
        self.direction = 1.0;
    }

    pub fn directed_speed(&self, animation: &RuntimeLinearAnimation) -> f32 {
        self.direction * animation.speed
    }

    fn resolved_loop_kind(&self, animation: &RuntimeLinearAnimation) -> AnimationLoop {
        AnimationLoop::from_loop_value(self.loop_value.unwrap_or(animation.loop_value))
    }

    fn keep_going(&self, animation: &RuntimeLinearAnimation) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) > 0.0 && self.time < animation.end_seconds())
            || (self.directed_speed(animation) < 0.0 && self.time > animation.start_seconds())
    }

    fn keep_going_with_speed_multiplier(
        &self,
        animation: &RuntimeLinearAnimation,
        speed_multiplier: f32,
    ) -> bool {
        self.resolved_loop_kind(animation) != AnimationLoop::OneShot
            || (self.directed_speed(animation) * speed_multiplier > 0.0
                && self.time < animation.end_seconds())
            || (self.directed_speed(animation) * speed_multiplier < 0.0
                && self.time > animation.start_seconds())
    }

    fn advance(&mut self, animation: &RuntimeLinearAnimation, elapsed_seconds: f32) -> bool {
        let delta_seconds = elapsed_seconds * animation.speed * self.direction;
        self.spilled_time = 0.0;
        if delta_seconds == 0.0 {
            self.did_loop = false;
            return false;
        }

        self.last_total_time = self.total_time;
        self.total_time += delta_seconds.abs();
        let kill_spilled_time = !self.keep_going_with_speed_multiplier(animation, elapsed_seconds);

        self.time += delta_seconds;
        let fps = animation.fps_as_f32();
        if fps == 0.0 {
            self.did_loop = false;
            return self.keep_going_with_speed_multiplier(animation, elapsed_seconds);
        }

        let mut frames = self.time * fps;
        let start = animation.start_frame();
        let end = animation.end_frame();
        let range = end - start;
        let mut did_loop = false;
        let mut direction = if delta_seconds < 0.0 { -1 } else { 1 };

        match self.resolved_loop_kind(animation) {
            AnimationLoop::OneShot => {
                if direction == 1 && frames > end {
                    let delta_frames = delta_seconds * fps;
                    let spilled_frames_ratio = (frames - end) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end;
                    self.time = frames / fps;
                    did_loop = true;
                } else if direction == -1 && frames < start {
                    let delta_frames = (delta_seconds * fps).abs();
                    let spilled_frames_ratio = (start - frames) / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::Loop => {
                if range != 0.0 && direction == 1 && frames >= end {
                    let delta_frames = delta_seconds * fps;
                    let remainder = (frames - start) % range;
                    let spilled_frames_ratio = remainder / delta_frames;
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = start + remainder;
                    self.time = frames / fps;
                    did_loop = true;
                } else if range != 0.0 && direction == -1 && frames <= start {
                    let delta_frames = delta_seconds * fps;
                    let remainder = ((start - frames) % range).abs();
                    let spilled_frames_ratio = (remainder / delta_frames).abs();
                    self.spilled_time = spilled_frames_ratio * elapsed_seconds;
                    frames = end - remainder;
                    self.time = frames / fps;
                    did_loop = true;
                }
            }
            AnimationLoop::PingPong => loop {
                if direction == 1 && frames >= end {
                    self.spilled_time = (frames - end) / fps;
                    frames = end + (end - frames);
                } else if direction == -1 && frames < start {
                    self.spilled_time = (start - frames) / fps;
                    frames = start + (start - frames);
                } else {
                    break;
                }
                self.time = frames / fps;
                self.direction *= -1.0;
                direction *= -1;
                did_loop = true;
            },
        }

        if kill_spilled_time {
            self.spilled_time = 0.0;
        }
        self.did_loop = did_loop;
        self.keep_going_with_speed_multiplier(animation, elapsed_seconds)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachine {
    pub global_id: u32,
    pub name: Option<String>,
    pub inputs: Vec<RuntimeStateMachineInput>,
    pub layers: Vec<RuntimeStateMachineLayer>,
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineInput {
    pub global_id: u32,
    pub name: Option<String>,
    pub kind: StateMachineInputKind,
    value: StateMachineInputValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateMachineInputKind {
    Bool,
    Number,
    Trigger,
}

#[derive(Debug, Clone)]
enum StateMachineInputValue {
    Bool(bool),
    Number(f32),
    Trigger {
        fired: bool,
        used_layers: Vec<usize>,
    },
}

#[derive(Debug, Clone)]
pub struct RuntimeStateMachineLayer {
    pub global_id: u32,
    pub name: Option<String>,
    pub states: Vec<RuntimeLayerState>,
    entry_state_index: Option<usize>,
    any_state_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct RuntimeLayerState {
    pub global_id: Option<u32>,
    pub type_name: Option<&'static str>,
    animation_index: Option<usize>,
    speed: f32,
    flags: u64,
    fire_actions: Vec<RuntimeStateMachineFireEvent>,
    transitions: Vec<RuntimeStateTransition>,
}

impl RuntimeLayerState {
    const RANDOM: u64 = 1 << 0;

    fn uses_random_transition_selection(&self) -> bool {
        self.flags & Self::RANDOM == Self::RANDOM
    }

    fn fire_events(
        &self,
        occurrence: StateMachineFireOccurrence,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        fire_state_machine_events(&self.fire_actions, occurrence, reported_events);
    }
}

#[derive(Debug, Clone)]
struct RuntimeStateTransition {
    state_to_index: Option<usize>,
    duration: u64,
    exit_time: u64,
    flags: u64,
    random_weight: u64,
    condition_count: usize,
    conditions: Vec<RuntimeTransitionCondition>,
    fire_actions: Vec<RuntimeStateMachineFireEvent>,
    interpolator: Option<RuntimeTransitionInterpolator>,
    has_unsupported_interpolator: bool,
}

impl RuntimeStateTransition {
    const DISABLED: u64 = 1 << 0;
    const DURATION_IS_PERCENTAGE: u64 = 1 << 1;
    const ENABLE_EXIT_TIME: u64 = 1 << 2;
    const EXIT_TIME_IS_PERCENTAGE: u64 = 1 << 3;
    const PAUSE_ON_EXIT: u64 = 1 << 4;
    const ENABLE_EARLY_EXIT: u64 = 1 << 5;

    fn is_simple_supported(&self) -> bool {
        self.state_to_index.is_some()
            && self.condition_count == self.conditions.len()
            && !self.has_unsupported_interpolator
            && self.flags & Self::DISABLED == 0
    }

    fn allow(
        &self,
        inputs: &[StateMachineInputInstance],
        layer_index: usize,
        animation_from: Option<(&LinearAnimationInstance, &RuntimeLinearAnimation)>,
    ) -> TransitionAllowance {
        if !self
            .conditions
            .iter()
            .all(|condition| condition.evaluate(inputs, layer_index))
        {
            return TransitionAllowance::No;
        }

        if self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
            && let Some((animation_instance, animation)) = animation_from
        {
            let mut exit_time = self.exit_time_seconds(Some(animation), false);
            let duration = animation.duration_seconds();
            if exit_time <= duration
                && AnimationLoop::from_loop_value(animation.loop_value) != AnimationLoop::OneShot
                && duration != 0.0
            {
                exit_time += (animation_instance.last_total_time / duration).floor() * duration;
            }
            if animation_instance.total_time < exit_time {
                return TransitionAllowance::WaitingForExit;
            }
        }

        TransitionAllowance::Yes
    }

    fn has_exit_time(&self) -> bool {
        self.flags & Self::ENABLE_EXIT_TIME == Self::ENABLE_EXIT_TIME
    }

    fn pause_on_exit(&self) -> bool {
        self.flags & Self::PAUSE_ON_EXIT == Self::PAUSE_ON_EXIT
    }

    fn enable_early_exit(&self) -> bool {
        self.flags & Self::ENABLE_EARLY_EXIT == Self::ENABLE_EARLY_EXIT
    }

    fn fire_events(
        &self,
        occurrence: StateMachineFireOccurrence,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        fire_state_machine_events(&self.fire_actions, occurrence, reported_events);
    }

    fn transition_duration_seconds(&self, animation_from: Option<&RuntimeLinearAnimation>) -> f32 {
        if self.duration == 0 {
            return 0.0;
        }
        if self.flags & Self::DURATION_IS_PERCENTAGE == Self::DURATION_IS_PERCENTAGE {
            return animation_from
                .map(|animation| self.duration as f32 / 100.0 * animation.duration_seconds())
                .unwrap_or(0.0);
        }
        self.duration as f32 / 1000.0
    }

    fn exit_time_seconds(
        &self,
        animation_from: Option<&RuntimeLinearAnimation>,
        absolute: bool,
    ) -> f32 {
        if self.flags & Self::EXIT_TIME_IS_PERCENTAGE == Self::EXIT_TIME_IS_PERCENTAGE {
            return animation_from
                .map(|animation| {
                    let start = if absolute {
                        animation.start_seconds()
                    } else {
                        0.0
                    };
                    start + self.exit_time as f32 / 100.0 * animation.duration_seconds()
                })
                .unwrap_or(0.0);
        }
        self.exit_time as f32 / 1000.0
    }

    fn use_inputs(&self, inputs: &mut [StateMachineInputInstance], layer_index: usize) {
        for condition in &self.conditions {
            condition.use_input(inputs, layer_index);
        }
    }
}

#[derive(Debug, Clone)]
struct RuntimeStateMachineFireEvent {
    occurs_value: u64,
    event: StateMachineReportedEvent,
}

impl RuntimeStateMachineFireEvent {
    fn from_imported(action: &rive_binary::RuntimeStateMachineFireAction<'_>) -> Option<Self> {
        let event = action.event?;
        Some(Self {
            occurs_value: action.object.uint_property("occursValue").unwrap_or(0),
            event: StateMachineReportedEvent {
                event_local_index: action.event_local_index?,
                event_core_type: u32::from(event.type_key),
                name: event.string_property("name").map(ToOwned::to_owned),
                seconds_delay: 0.0,
            },
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StateMachineFireOccurrence {
    AtStart,
    AtEnd,
}

impl StateMachineFireOccurrence {
    fn value(self) -> u64 {
        match self {
            Self::AtStart => 0,
            Self::AtEnd => 1,
        }
    }
}

fn fire_state_machine_events(
    fire_actions: &[RuntimeStateMachineFireEvent],
    occurrence: StateMachineFireOccurrence,
    reported_events: &mut Vec<StateMachineReportedEvent>,
) {
    for action in fire_actions {
        if action.occurs_value == occurrence.value() {
            reported_events.push(action.event.clone());
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeTransitionInterpolator {
    CubicEase {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    Elastic {
        amplitude: f32,
        period: f32,
        easing_value: u64,
    },
}

impl RuntimeTransitionInterpolator {
    fn from_object(object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "CubicEaseInterpolator" => Some(Self::CubicEase {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    fn transform(self, factor: f32) -> f32 {
        match self {
            Self::CubicEase { x1, y1, x2, y2 } => {
                let t = cubic_interpolator_get_t(factor, x1, x2);
                cubic_interpolator_calc_bezier(t, y1, y2)
            }
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => elastic_interpolator_transform(factor, amplitude, period, easing_value),
        }
    }
}

fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
}

fn cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    const SPLINE_TABLE_SIZE: usize = 11;
    const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
    const NEWTON_ITERATIONS: usize = 4;
    const NEWTON_MIN_SLOPE: f32 = 0.001;
    const SUBDIVISION_PRECISION: f32 = 0.0000001;
    const SUBDIVISION_MAX_ITERATIONS: usize = 10;

    let mut values = [0.0; SPLINE_TABLE_SIZE];
    for (i, value) in values.iter_mut().enumerate() {
        *value = cubic_interpolator_calc_bezier(i as f32 * SAMPLE_STEP_SIZE, x1, x2);
    }

    let mut interval_start = 0.0;
    let mut current_sample = 1;
    let last_sample = SPLINE_TABLE_SIZE - 1;
    while current_sample != last_sample && values[current_sample] <= x {
        interval_start += SAMPLE_STEP_SIZE;
        current_sample += 1;
    }
    current_sample -= 1;

    let dist = (x - values[current_sample]) / (values[current_sample + 1] - values[current_sample]);
    let mut guess_for_t = interval_start + dist * SAMPLE_STEP_SIZE;
    let initial_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
    if initial_slope >= NEWTON_MIN_SLOPE {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
            if current_slope == 0.0 {
                return guess_for_t;
            }
            let current_x = cubic_interpolator_calc_bezier(guess_for_t, x1, x2) - x;
            guess_for_t -= current_x / current_slope;
        }
        guess_for_t
    } else if initial_slope == 0.0 {
        guess_for_t
    } else {
        let mut upper_bound = interval_start + SAMPLE_STEP_SIZE;
        let mut iterations = 0;
        loop {
            let current_t = interval_start + (upper_bound - interval_start) / 2.0;
            let current_x = cubic_interpolator_calc_bezier(current_t, x1, x2) - x;
            if current_x > 0.0 {
                upper_bound = current_t;
            } else {
                interval_start = current_t;
            }
            iterations += 1;
            if current_x.abs() <= SUBDIVISION_PRECISION || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}

fn elastic_interpolator_transform(
    factor: f32,
    amplitude: f32,
    serialized_period: f32,
    easing_value: u64,
) -> f32 {
    let period = if serialized_period == 0.0 {
        0.5
    } else {
        serialized_period
    };
    let shift = if amplitude < 1.0 {
        period / 4.0
    } else {
        period / (2.0 * std::f32::consts::PI) * (1.0 / amplitude).asin()
    };

    match easing_value {
        0 => elastic_ease_in(factor, amplitude, period, shift),
        1 => elastic_ease_out(factor, amplitude, period, shift),
        2 => elastic_ease_in_out(factor, amplitude, period, shift),
        _ => factor,
    }
}

fn elastic_actual_amplitude(time: f32, amplitude: f32, shift: f32) -> f32 {
    if amplitude < 1.0 {
        let shift_abs = shift.abs();
        let time_abs = time.abs();
        if time_abs < shift_abs {
            let l = time_abs / shift_abs;
            return (amplitude * l) + (1.0 - l);
        }
    }

    amplitude
}

fn elastic_ease_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    actual_amplitude
        * 2.0_f32.powf(10.0 * -time)
        * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
        + 1.0
}

fn elastic_ease_in(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    -(actual_amplitude
        * 2.0_f32.powf(10.0 * time)
        * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
}

fn elastic_ease_in_out(factor: f32, amplitude: f32, period: f32, shift: f32) -> f32 {
    let time = factor * 2.0 - 1.0;
    let actual_amplitude = elastic_actual_amplitude(time, amplitude, shift);
    if time < 0.0 {
        -0.5 * actual_amplitude
            * 2.0_f32.powf(10.0 * time)
            * ((-time - shift) * (2.0 * std::f32::consts::PI) / period).sin()
    } else {
        0.5 * (actual_amplitude
            * 2.0_f32.powf(10.0 * -time)
            * ((time - shift) * (2.0 * std::f32::consts::PI) / period).sin())
            + 1.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionAllowance {
    No,
    WaitingForExit,
    Yes,
}

#[derive(Debug, Clone)]
enum RuntimeTransitionCondition {
    Bool {
        input_index: usize,
        op: TransitionConditionOp,
    },
    Number {
        input_index: usize,
        op: TransitionConditionOp,
        value: f32,
    },
    Trigger {
        input_index: usize,
    },
}

impl RuntimeTransitionCondition {
    fn from_object(object: &RuntimeObject) -> Option<Self> {
        let input_index = usize::try_from(object.uint_property("inputId")?).ok()?;
        match object.type_name {
            "TransitionBoolCondition" => Some(Self::Bool {
                input_index,
                op: TransitionConditionOp::from_value(object.uint_property("opValue").unwrap_or(0)),
            }),
            "TransitionNumberCondition" => Some(Self::Number {
                input_index,
                op: TransitionConditionOp::from_value(object.uint_property("opValue").unwrap_or(0)),
                value: object.double_property("value").unwrap_or(0.0),
            }),
            "TransitionTriggerCondition" => Some(Self::Trigger { input_index }),
            _ => None,
        }
    }

    fn evaluate(&self, inputs: &[StateMachineInputInstance], layer_index: usize) -> bool {
        match self {
            Self::Bool { input_index, op } => {
                let Some(value) = inputs
                    .get(*input_index)
                    .and_then(StateMachineInputInstance::bool_value)
                else {
                    return true;
                };
                (value && *op == TransitionConditionOp::Equal)
                    || (!value && *op == TransitionConditionOp::NotEqual)
            }
            Self::Number {
                input_index,
                op,
                value,
            } => {
                let Some(input_value) = inputs
                    .get(*input_index)
                    .and_then(StateMachineInputInstance::number_value)
                else {
                    return true;
                };
                op.compare(input_value, *value)
            }
            Self::Trigger { input_index } => {
                let Some(input) = inputs.get(*input_index) else {
                    return true;
                };
                input
                    .trigger_is_fireable_for_layer(layer_index)
                    .unwrap_or(true)
            }
        }
    }

    fn use_input(&self, inputs: &mut [StateMachineInputInstance], layer_index: usize) {
        if let Self::Trigger { input_index } = self
            && let Some(input) = inputs.get_mut(*input_index)
        {
            input.use_trigger_in_layer(layer_index);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TransitionConditionOp {
    Equal,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LessThan,
    GreaterThan,
}

impl TransitionConditionOp {
    fn from_value(value: u64) -> Self {
        match value {
            1 => Self::NotEqual,
            2 => Self::LessThanOrEqual,
            3 => Self::GreaterThanOrEqual,
            4 => Self::LessThan,
            5 => Self::GreaterThan,
            _ => Self::Equal,
        }
    }

    fn compare(self, input_value: f32, value: f32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            Self::LessThanOrEqual => input_value <= value,
            Self::GreaterThanOrEqual => input_value >= value,
            Self::LessThan => input_value < value,
            Self::GreaterThan => input_value > value,
        }
    }
}

#[derive(Debug, Clone)]
pub struct StateMachineInstance {
    state_machine_index: usize,
    inputs: Vec<StateMachineInputInstance>,
    layers: Vec<StateMachineLayerInstance>,
    reported_events: Vec<StateMachineReportedEvent>,
    changed_state_count: usize,
    needs_advance: bool,
}

#[derive(Debug, Clone)]
pub struct StateMachineReportedEvent {
    event_local_index: usize,
    event_core_type: u32,
    name: Option<String>,
    seconds_delay: f32,
}

impl StateMachineReportedEvent {
    pub fn event_local_index(&self) -> usize {
        self.event_local_index
    }

    pub fn event_core_type(&self) -> u32 {
        self.event_core_type
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }
}

impl StateMachineInstance {
    fn new(
        state_machine_index: usize,
        state_machine: &RuntimeStateMachine,
        artboard: &ArtboardInstance,
    ) -> Self {
        Self {
            state_machine_index,
            inputs: state_machine
                .inputs
                .iter()
                .enumerate()
                .map(|(index, input)| StateMachineInputInstance::new(index, input))
                .collect(),
            layers: state_machine
                .layers
                .iter()
                .map(|layer| StateMachineLayerInstance::new(layer, artboard))
                .collect(),
            reported_events: Vec::new(),
            changed_state_count: 0,
            needs_advance: false,
        }
    }

    pub fn state_machine_index(&self) -> usize {
        self.state_machine_index
    }

    pub fn changed_state_count(&self) -> usize {
        self.changed_state_count
    }

    pub fn needs_advance(&self) -> bool {
        self.needs_advance
    }

    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }

    pub fn input(&self, index: usize) -> Option<&StateMachineInputInstance> {
        self.inputs.get(index)
    }

    pub fn input_named(&self, name: &str) -> Option<&StateMachineInputInstance> {
        self.inputs
            .iter()
            .find(|input| input.name.as_deref() == Some(name))
    }

    pub fn input_index_named(&self, name: &str) -> Option<usize> {
        self.inputs
            .iter()
            .position(|input| input.name.as_deref() == Some(name))
    }

    pub fn set_bool(&mut self, index: usize, value: bool) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_bool(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn set_number(&mut self, index: usize, value: f32) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.set_number(value) {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn fire_trigger(&mut self, index: usize) -> bool {
        let Some(input) = self.inputs.get_mut(index) else {
            return false;
        };
        if !input.fire_trigger() {
            return false;
        }
        self.needs_advance = true;
        true
    }

    pub fn current_animation_count(&self) -> usize {
        self.layers
            .iter()
            .filter(|layer| layer.current_animation.is_some())
            .count()
    }

    pub fn current_animation(&self, index: usize) -> Option<&LinearAnimationInstance> {
        self.layers
            .iter()
            .filter_map(|layer| layer.current_animation.as_ref())
            .nth(index)
    }

    pub fn reported_event_count(&self) -> usize {
        self.reported_events.len()
    }

    pub fn reported_event(&self, index: usize) -> Option<&StateMachineReportedEvent> {
        self.reported_events.get(index)
    }

    fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        state_machine: &RuntimeStateMachine,
        elapsed_seconds: f32,
    ) -> bool {
        self.reported_events.clear();
        self.changed_state_count = 0;
        self.needs_advance = false;
        let mut keep_going = false;
        for (layer_index, (layer_instance, layer)) in self
            .layers
            .iter_mut()
            .zip(&state_machine.layers)
            .enumerate()
        {
            let layer_result = layer_instance.advance(
                artboard,
                layer,
                elapsed_seconds,
                layer_index,
                &mut self.inputs,
                &mut self.reported_events,
            );
            if layer_result.changed_state {
                self.changed_state_count += 1;
            }
            keep_going |= layer_result.keep_going;
        }
        for input in &mut self.inputs {
            input.advanced();
        }
        self.needs_advance = keep_going || !self.reported_events.is_empty();
        self.needs_advance
    }
}

#[derive(Debug, Clone)]
pub struct StateMachineInputInstance {
    index: usize,
    global_id: u32,
    name: Option<String>,
    kind: StateMachineInputKind,
    value: StateMachineInputValue,
}

impl StateMachineInputInstance {
    fn new(index: usize, input: &RuntimeStateMachineInput) -> Self {
        Self {
            index,
            global_id: input.global_id,
            name: input.name.clone(),
            kind: input.kind,
            value: input.value.clone(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn global_id(&self) -> u32 {
        self.global_id
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn kind(&self) -> StateMachineInputKind {
        self.kind
    }

    pub fn bool_value(&self) -> Option<bool> {
        match self.value {
            StateMachineInputValue::Bool(value) => Some(value),
            _ => None,
        }
    }

    pub fn number_value(&self) -> Option<f32> {
        match self.value {
            StateMachineInputValue::Number(value) => Some(value),
            _ => None,
        }
    }

    pub fn trigger_fired(&self) -> Option<bool> {
        match self.value {
            StateMachineInputValue::Trigger { fired, .. } => Some(fired),
            _ => None,
        }
    }

    fn set_bool(&mut self, value: bool) -> bool {
        match &mut self.value {
            StateMachineInputValue::Bool(current) => {
                if *current == value {
                    return false;
                }
                *current = value;
                true
            }
            _ => false,
        }
    }

    fn set_number(&mut self, value: f32) -> bool {
        match &mut self.value {
            StateMachineInputValue::Number(current) => {
                if *current == value {
                    return false;
                }
                *current = value;
                true
            }
            _ => false,
        }
    }

    fn fire_trigger(&mut self) -> bool {
        match &mut self.value {
            StateMachineInputValue::Trigger { fired, .. } => {
                if *fired {
                    return false;
                }
                *fired = true;
                true
            }
            _ => false,
        }
    }

    fn trigger_is_fireable_for_layer(&self, layer_index: usize) -> Option<bool> {
        match &self.value {
            StateMachineInputValue::Trigger { fired, used_layers } => {
                Some(*fired && !used_layers.contains(&layer_index))
            }
            _ => None,
        }
    }

    fn use_trigger_in_layer(&mut self, layer_index: usize) {
        if let StateMachineInputValue::Trigger { used_layers, .. } = &mut self.value
            && !used_layers.contains(&layer_index)
        {
            used_layers.push(layer_index);
        }
    }

    fn advanced(&mut self) {
        if let StateMachineInputValue::Trigger { fired, used_layers } = &mut self.value {
            *fired = false;
            used_layers.clear();
        }
    }
}

#[derive(Debug, Clone)]
struct StateMachineLayerInstance {
    current_state_index: Option<usize>,
    current_animation: Option<LinearAnimationInstance>,
    current_animation_keep_going: bool,
    transition_source_state_index: Option<usize>,
    transition_source_animation: Option<LinearAnimationInstance>,
    transition_duration_seconds: f32,
    transition_mix: f32,
    transition_mix_from: f32,
    transition_source_paused: bool,
    transition_interpolator: Option<RuntimeTransitionInterpolator>,
    transition_enable_early_exit: bool,
    transition_fire_actions: Vec<RuntimeStateMachineFireEvent>,
    transition_completed: bool,
    waiting_for_exit: bool,
}

#[derive(Debug, Clone, Copy)]
struct StateMachineLayerAdvance {
    changed_state: bool,
    keep_going: bool,
}

impl StateMachineLayerInstance {
    fn new(layer: &RuntimeStateMachineLayer, artboard: &ArtboardInstance) -> Self {
        let mut instance = Self {
            current_state_index: layer.entry_state_index,
            current_animation: None,
            current_animation_keep_going: false,
            transition_source_state_index: None,
            transition_source_animation: None,
            transition_duration_seconds: 0.0,
            transition_mix: 1.0,
            transition_mix_from: 1.0,
            transition_source_paused: false,
            transition_interpolator: None,
            transition_enable_early_exit: false,
            transition_fire_actions: Vec::new(),
            transition_completed: false,
            waiting_for_exit: false,
        };
        instance.refresh_current_animation(artboard, layer);
        instance
    }

    fn advance(
        &mut self,
        artboard: &mut ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> StateMachineLayerAdvance {
        self.advance_current_animation(artboard, layer, elapsed_seconds);
        self.update_transition_mix(elapsed_seconds, reported_events);
        self.advance_transition_source_animation(artboard, layer, elapsed_seconds);
        self.apply_animations(artboard);

        let mut changed_state = false;
        for _ in 0..100 {
            if !self.update_state(artboard, layer, layer_index, inputs, reported_events) {
                break;
            }
            changed_state = true;
            self.apply_animations(artboard);
        }

        StateMachineLayerAdvance {
            changed_state,
            keep_going: changed_state
                || self.is_transitioning()
                || self.waiting_for_exit
                || self.current_animation_keep_going,
        }
    }

    fn update_state(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        if self.is_transitioning() && !self.transition_enable_early_exit {
            return false;
        }
        self.waiting_for_exit = false;
        if self.try_change_state(
            artboard,
            layer,
            layer.any_state_index,
            layer_index,
            inputs,
            reported_events,
        ) {
            return true;
        }
        self.try_change_state(
            artboard,
            layer,
            self.current_state_index,
            layer_index,
            inputs,
            reported_events,
        )
    }

    fn try_change_state(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        state_index: Option<usize>,
        layer_index: usize,
        inputs: &mut [StateMachineInputInstance],
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) -> bool {
        let Some(state_index) = state_index else {
            return false;
        };
        let Some(state) = layer.states.get(state_index) else {
            return false;
        };

        if state.uses_random_transition_selection() {
            let Some((transition_index, state_to_index)) =
                self.find_random_transition(artboard, state, state_index, layer_index, inputs)
            else {
                return false;
            };
            let transition = &state.transitions[transition_index];
            transition.use_inputs(inputs, layer_index);
            self.change_state(artboard, layer, transition, state_to_index, reported_events);
            return true;
        }

        for transition in &state.transitions {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self.current_state_index == Some(state_to_index) {
                continue;
            }
            let animation_from = (self.current_state_index == Some(state_index))
                .then(|| {
                    self.current_animation
                        .as_ref()
                        .and_then(|animation_instance| {
                            artboard
                                .linear_animation(animation_instance.animation_index)
                                .map(|animation| (animation_instance, animation))
                        })
                })
                .flatten();
            match transition.allow(inputs, layer_index, animation_from) {
                TransitionAllowance::No => continue,
                TransitionAllowance::WaitingForExit => {
                    self.waiting_for_exit = true;
                    continue;
                }
                TransitionAllowance::Yes => {
                    self.waiting_for_exit = false;
                }
            }
            transition.use_inputs(inputs, layer_index);
            self.change_state(artboard, layer, transition, state_to_index, reported_events);
            return true;
        }
        false
    }

    fn find_random_transition(
        &mut self,
        artboard: &ArtboardInstance,
        state: &RuntimeLayerState,
        state_index: usize,
        layer_index: usize,
        inputs: &[StateMachineInputInstance],
    ) -> Option<(usize, usize)> {
        let animation_from = (self.current_state_index == Some(state_index))
            .then(|| {
                self.current_animation
                    .as_ref()
                    .and_then(|animation_instance| {
                        artboard
                            .linear_animation(animation_instance.animation_index)
                            .map(|animation| (animation_instance, animation))
                    })
            })
            .flatten();
        let mut weighted_candidates = Vec::new();
        let mut total_weight = 0_u64;
        let mut waiting_for_exit = false;

        for (transition_index, transition) in state.transitions.iter().enumerate() {
            if !transition.is_simple_supported() {
                continue;
            }
            let Some(state_to_index) = transition.state_to_index else {
                continue;
            };
            if self.current_state_index == Some(state_to_index) {
                continue;
            }

            match transition.allow(inputs, layer_index, animation_from) {
                TransitionAllowance::No => {}
                TransitionAllowance::WaitingForExit => {
                    waiting_for_exit = true;
                }
                TransitionAllowance::Yes => {
                    total_weight = total_weight.saturating_add(transition.random_weight);
                    weighted_candidates.push((
                        transition_index,
                        state_to_index,
                        transition.random_weight,
                    ));
                }
            }
        }

        if total_weight == 0 {
            self.waiting_for_exit = waiting_for_exit;
            return None;
        }

        let random_weight = Self::random_transition_value() * total_weight as f64;
        let mut current_weight = 0.0_f64;
        for (transition_index, state_to_index, transition_weight) in weighted_candidates {
            current_weight += transition_weight as f64;
            if current_weight > random_weight {
                self.waiting_for_exit = false;
                return Some((transition_index, state_to_index));
            }
        }

        self.waiting_for_exit = waiting_for_exit;
        None
    }

    fn random_transition_value() -> f64 {
        0.0
    }

    fn change_state(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        transition: &RuntimeStateTransition,
        state_to_index: usize,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        let previous_state_index = self.current_state_index;
        let mut previous_animation = self.current_animation.clone();
        let previous_spilled_time = previous_animation
            .as_ref()
            .map(LinearAnimationInstance::spilled_time)
            .unwrap_or(0.0);
        let previous_mix = self.transition_mix;
        if let Some(previous_state) =
            previous_state_index.and_then(|state_index| layer.states.get(state_index))
        {
            previous_state.fire_events(StateMachineFireOccurrence::AtEnd, reported_events);
        }

        if transition.pause_on_exit()
            && transition.has_exit_time()
            && let Some(animation_instance) = previous_animation.as_mut()
            && let Some(animation) = artboard.linear_animation(animation_instance.animation_index)
        {
            animation_instance.set_time(
                animation,
                transition.exit_time_seconds(Some(animation), true),
            );
        }
        let previous_runtime_animation =
            previous_animation.as_ref().and_then(|animation_instance| {
                artboard.linear_animation(animation_instance.animation_index)
            });
        let transition_duration_seconds =
            transition.transition_duration_seconds(previous_runtime_animation);

        self.current_state_index = Some(state_to_index);
        self.refresh_current_animation(artboard, layer);
        if let Some(current_state) = layer.states.get(state_to_index) {
            current_state.fire_events(StateMachineFireOccurrence::AtStart, reported_events);
        }
        transition.fire_events(StateMachineFireOccurrence::AtStart, reported_events);
        if previous_spilled_time != 0.0 {
            self.advance_current_animation(artboard, layer, previous_spilled_time);
        }

        if transition_duration_seconds == 0.0 {
            transition.fire_events(StateMachineFireOccurrence::AtEnd, reported_events);
            self.clear_transition_source();
            return;
        }

        if let (Some(source_state_index), Some(source_animation)) =
            (previous_state_index, previous_animation)
        {
            self.transition_source_state_index = Some(source_state_index);
            self.transition_source_animation = Some(source_animation);
            self.transition_duration_seconds = transition_duration_seconds;
            self.transition_mix_from = previous_mix;
            self.transition_mix = 0.0;
            self.transition_source_paused = transition.pause_on_exit();
            self.transition_interpolator = transition.interpolator;
            self.transition_enable_early_exit = transition.enable_early_exit();
            self.transition_fire_actions = transition.fire_actions.clone();
            self.transition_completed = false;
            self.update_transition_mix(0.0, reported_events);
        } else {
            self.clear_transition_source();
        }
    }

    fn clear_transition_source(&mut self) {
        self.transition_source_state_index = None;
        self.transition_source_animation = None;
        self.transition_duration_seconds = 0.0;
        self.transition_mix = 1.0;
        self.transition_mix_from = 1.0;
        self.transition_source_paused = false;
        self.transition_interpolator = None;
        self.transition_enable_early_exit = false;
        self.transition_fire_actions.clear();
        self.transition_completed = false;
    }

    fn is_transitioning(&self) -> bool {
        self.transition_source_animation.is_some()
            && self.transition_duration_seconds != 0.0
            && self.transition_mix < 1.0
    }

    fn update_transition_mix(
        &mut self,
        elapsed_seconds: f32,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        if self.transition_source_animation.is_none() || self.transition_duration_seconds == 0.0 {
            self.transition_mix = 1.0;
            return;
        }
        self.transition_mix = (self.transition_mix
            + elapsed_seconds / self.transition_duration_seconds)
            .clamp(0.0, 1.0);
        if self.transition_mix == 1.0 && !self.transition_completed {
            self.transition_completed = true;
            fire_state_machine_events(
                &self.transition_fire_actions,
                StateMachineFireOccurrence::AtEnd,
                reported_events,
            );
        }
    }

    fn refresh_current_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
    ) {
        let Some(state) = self
            .current_state_index
            .and_then(|index| layer.states.get(index))
        else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let Some(animation_index) = state.animation_index else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let Some(animation) = artboard.linear_animation(animation_index) else {
            self.current_animation = None;
            self.current_animation_keep_going = false;
            return;
        };
        let mut animation_instance =
            LinearAnimationInstance::new(animation_index, animation, state.speed);
        self.current_animation_keep_going = animation_instance.advance(animation, 0.0);
        self.current_animation = Some(animation_instance);
    }

    fn advance_current_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
    ) -> bool {
        let Some(animation_instance) = self.current_animation.as_mut() else {
            self.current_animation_keep_going = false;
            return false;
        };
        let Some(state) = self
            .current_state_index
            .and_then(|index| layer.states.get(index))
        else {
            self.current_animation_keep_going = false;
            return false;
        };
        let Some(animation) = artboard.linear_animation(animation_instance.animation_index) else {
            self.current_animation_keep_going = false;
            return false;
        };
        self.current_animation_keep_going =
            animation_instance.advance(animation, elapsed_seconds * state.speed);
        self.current_animation_keep_going
    }

    fn advance_transition_source_animation(
        &mut self,
        artboard: &ArtboardInstance,
        layer: &RuntimeStateMachineLayer,
        elapsed_seconds: f32,
    ) -> bool {
        if !self.is_transitioning() {
            return false;
        }
        if self.transition_source_paused {
            return false;
        }
        let Some(animation_instance) = self.transition_source_animation.as_mut() else {
            return false;
        };
        let Some(state) = self
            .transition_source_state_index
            .and_then(|index| layer.states.get(index))
        else {
            return false;
        };
        let Some(animation) = artboard.linear_animation(animation_instance.animation_index) else {
            return false;
        };
        animation_instance.advance(animation, elapsed_seconds * state.speed)
    }

    fn apply_animations(&self, artboard: &mut ArtboardInstance) -> bool {
        let mut changed = false;
        if self.is_transitioning()
            && let Some(source_animation) = self.transition_source_animation.as_ref()
        {
            let mix_from = self
                .transition_interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix_from))
                .unwrap_or(self.transition_mix_from);
            changed |= artboard.apply_linear_animation_instance(source_animation, mix_from);
        }
        let Some(animation_instance) = self.current_animation.as_ref() else {
            return changed;
        };
        let mix = if self.transition_source_animation.is_some() {
            self.transition_interpolator
                .map(|interpolator| interpolator.transform(self.transition_mix))
                .unwrap_or(self.transition_mix)
        } else {
            1.0
        };
        changed | artboard.apply_linear_animation_instance(animation_instance, mix)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedObject {
    pub global_id: u32,
    pub object_id: usize,
    pub target_local_id: usize,
    pub keyed_properties: Vec<RuntimeKeyedProperty>,
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyedProperty {
    pub global_id: u32,
    pub property_key: u16,
    pub transform_property: Option<TransformProperty>,
    pub key_frames: Vec<RuntimeKeyFrameDouble>,
}

impl RuntimeKeyedProperty {
    fn value_at(&self, seconds: f32, fps: u64, current: f32, mix: f32) -> Option<f32> {
        if self.key_frames.is_empty() {
            return None;
        }

        let idx = self.closest_frame_index(seconds, fps);
        let value = if idx == 0 {
            self.key_frames[0].value
        } else if idx < self.key_frames.len() {
            let from = &self.key_frames[idx - 1];
            let to = &self.key_frames[idx];
            if seconds == to.seconds(fps) {
                to.value
            } else if from.interpolation_type == 0 {
                from.value
            } else if from.interpolator_id.is_some() {
                return None;
            } else {
                let from_seconds = from.seconds(fps);
                let to_seconds = to.seconds(fps);
                let frame_mix = if to_seconds == from_seconds {
                    1.0
                } else {
                    (seconds - from_seconds) / (to_seconds - from_seconds)
                };
                from.value + (to.value - from.value) * frame_mix
            }
        } else {
            self.key_frames.last()?.value
        };

        Some(mix_value(current, value, mix))
    }

    fn closest_frame_index(&self, seconds: f32, fps: u64) -> usize {
        let last = self.key_frames.len() - 1;
        if seconds > self.key_frames[last].seconds(fps) {
            return self.key_frames.len();
        }

        let mut start = 0;
        let mut end = last;
        while start <= end {
            let mid = (start + end) >> 1;
            let closest = self.key_frames[mid].seconds(fps);
            if closest < seconds {
                start = mid + 1;
            } else if closest > seconds {
                if mid == 0 {
                    break;
                }
                end = mid - 1;
            } else {
                return mid;
            }
        }
        start
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeKeyFrameDouble {
    pub global_id: u32,
    pub frame: u64,
    pub interpolation_type: u64,
    pub interpolator_id: Option<u64>,
    pub value: f32,
}

impl RuntimeKeyFrameDouble {
    fn seconds(&self, fps: u64) -> f32 {
        if fps == 0 {
            return 0.0;
        }
        self.frame as f32 / fps as f32
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeComponent {
    pub local_id: usize,
    pub global_id: u32,
    pub type_name: &'static str,
    pub capabilities: RuntimeComponentCapabilities,
    pub parent_local: Option<usize>,
    pub dependent_locals: Vec<usize>,
    pub graph_order: usize,
    pub dirt: ComponentDirt,
    pub transform: TransformRuntimeState,
}

impl RuntimeComponent {
    fn from_graph_component(component: &ComponentNode, object: &RuntimeObject) -> Self {
        Self {
            local_id: component.local_id,
            global_id: component.global_id,
            type_name: component.type_name,
            capabilities: RuntimeComponentCapabilities {
                world_transform: component.capabilities.world_transform,
                transform: component.capabilities.transform,
            },
            parent_local: component.parent_local,
            dependent_locals: component.dependent_locals.clone(),
            graph_order: component.graph_order.unwrap_or(0),
            dirt: ComponentDirt::FILTHY,
            transform: TransformRuntimeState::from_object(object),
        }
    }

    pub fn is_collapsed(&self) -> bool {
        self.dirt.contains(ComponentDirt::COLLAPSED)
    }
}

fn build_linear_animations(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    slots: &[InstanceSlot],
) -> Vec<RuntimeLinearAnimation> {
    let Some((start, end)) = artboard_object_range(file, graph.global_id as usize) else {
        return Vec::new();
    };

    let mut animations = Vec::<RuntimeLinearAnimation>::new();
    let mut current_animation = None;
    let mut current_keyed_object = None;
    let mut current_keyed_property = None;

    for global_id in start..end {
        let Some(object) = file.object(global_id) else {
            continue;
        };
        if file.import_status(global_id) != Some(RuntimeImportStatus::Imported) {
            continue;
        }

        if object.type_name == "LinearAnimation" {
            animations.push(RuntimeLinearAnimation {
                global_id: global_id as u32,
                name: object.string_property("name").map(ToOwned::to_owned),
                fps: object.uint_property("fps").unwrap_or(60),
                duration: object.uint_property("duration").unwrap_or(60),
                speed: object.double_property("speed").unwrap_or(1.0),
                loop_value: object.uint_property("loopValue").unwrap_or(0),
                work_start: object.uint_property("workStart").unwrap_or(0),
                work_end: object.uint_property("workEnd").unwrap_or(0),
                enable_work_area: object.bool_property("enableWorkArea").unwrap_or(false),
                quantize: object.bool_property("quantize").unwrap_or(false),
                keyed_objects: Vec::new(),
            });
            current_animation = Some(animations.len() - 1);
            current_keyed_object = None;
            current_keyed_property = None;
            continue;
        }

        let Some(animation_index) = current_animation else {
            continue;
        };

        if object.type_name == "KeyedObject" {
            let Some((object_id, target_local_id, _target)) =
                keyed_object_target(file, slots, object)
            else {
                current_keyed_object = None;
                current_keyed_property = None;
                continue;
            };

            animations[animation_index]
                .keyed_objects
                .push(RuntimeKeyedObject {
                    global_id: global_id as u32,
                    object_id,
                    target_local_id,
                    keyed_properties: Vec::new(),
                });
            current_keyed_object = Some(animations[animation_index].keyed_objects.len() - 1);
            current_keyed_property = None;
            continue;
        }

        if object.type_name == "KeyedProperty" {
            let Some(keyed_object_index) = current_keyed_object else {
                continue;
            };
            let Some(property_key) = object
                .uint_property("propertyKey")
                .and_then(|key| u16::try_from(key).ok())
            else {
                current_keyed_property = None;
                continue;
            };
            let target_local_id =
                animations[animation_index].keyed_objects[keyed_object_index].target_local_id;
            let Some(target) = slots
                .get(target_local_id)
                .and_then(|slot| file.object(slot.source_global_id as usize))
            else {
                current_keyed_property = None;
                continue;
            };
            if !object_supports_property(target.type_key, property_key) {
                current_keyed_property = None;
                continue;
            }

            animations[animation_index].keyed_objects[keyed_object_index]
                .keyed_properties
                .push(RuntimeKeyedProperty {
                    global_id: global_id as u32,
                    property_key,
                    transform_property: transform_property_for_key(property_key),
                    key_frames: Vec::new(),
                });
            current_keyed_property = Some((
                keyed_object_index,
                animations[animation_index].keyed_objects[keyed_object_index]
                    .keyed_properties
                    .len()
                    - 1,
            ));
            continue;
        }

        if object.type_name == "KeyFrameDouble" {
            let Some((keyed_object_index, keyed_property_index)) = current_keyed_property else {
                continue;
            };
            animations[animation_index].keyed_objects[keyed_object_index].keyed_properties
                [keyed_property_index]
                .key_frames
                .push(RuntimeKeyFrameDouble {
                    global_id: global_id as u32,
                    frame: object.uint_property("frame").unwrap_or(0),
                    interpolation_type: object.uint_property("interpolationType").unwrap_or(0),
                    interpolator_id: normalized_interpolator_id(object),
                    value: object.double_property("value").unwrap_or(0.0),
                });
        }
    }

    animations
}

fn build_state_machines(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    linear_animations: &[RuntimeLinearAnimation],
) -> Vec<RuntimeStateMachine> {
    let Some(artboard_index) = artboard_index_for_graph(file, graph) else {
        return Vec::new();
    };
    let animation_index_by_global = linear_animations
        .iter()
        .enumerate()
        .map(|(index, animation)| (animation.global_id, index))
        .collect::<BTreeMap<_, _>>();

    file.artboard_state_machine_graphs(artboard_index)
        .into_iter()
        .map(|state_machine| RuntimeStateMachine {
            global_id: state_machine.object.id,
            name: state_machine
                .object
                .string_property("name")
                .map(ToOwned::to_owned),
            inputs: state_machine
                .inputs
                .into_iter()
                .filter_map(runtime_state_machine_input)
                .collect(),
            layers: state_machine
                .layers
                .into_iter()
                .map(|layer| {
                    let states = layer
                        .states
                        .into_iter()
                        .map(|state| {
                            let animation_index = state.animation.and_then(|animation| {
                                animation_index_by_global.get(&animation.id).copied()
                            });
                            RuntimeLayerState {
                                global_id: state.object.map(|object| object.id),
                                type_name: state.object.map(|object| object.type_name),
                                animation_index,
                                speed: state
                                    .object
                                    .and_then(|object| object.double_property("speed"))
                                    .unwrap_or(1.0),
                                flags: state
                                    .object
                                    .and_then(|object| object.uint_property("flags"))
                                    .unwrap_or(0),
                                fire_actions: state
                                    .fire_actions
                                    .iter()
                                    .filter_map(RuntimeStateMachineFireEvent::from_imported)
                                    .collect(),
                                transitions: state
                                    .transitions
                                    .into_iter()
                                    .map(|transition| {
                                        let interpolator = transition
                                            .interpolator
                                            .and_then(RuntimeTransitionInterpolator::from_object);
                                        RuntimeStateTransition {
                                            state_to_index: transition.state_to_index,
                                            duration: transition
                                                .object
                                                .uint_property("duration")
                                                .unwrap_or(0),
                                            exit_time: transition
                                                .object
                                                .uint_property("exitTime")
                                                .unwrap_or(0),
                                            flags: transition
                                                .object
                                                .uint_property("flags")
                                                .unwrap_or(0),
                                            random_weight: transition
                                                .object
                                                .uint_property("randomWeight")
                                                .unwrap_or(1),
                                            condition_count: transition.conditions.len(),
                                            conditions: transition
                                                .conditions
                                                .iter()
                                                .filter_map(|condition| {
                                                    RuntimeTransitionCondition::from_object(
                                                        condition,
                                                    )
                                                })
                                                .collect(),
                                            fire_actions: transition
                                                .fire_actions
                                                .iter()
                                                .filter_map(
                                                    RuntimeStateMachineFireEvent::from_imported,
                                                )
                                                .collect(),
                                            interpolator,
                                            has_unsupported_interpolator: transition
                                                .interpolator
                                                .is_some()
                                                && interpolator.is_none(),
                                        }
                                    })
                                    .collect(),
                            }
                        })
                        .collect::<Vec<_>>();
                    let entry_state_index = states
                        .iter()
                        .position(|state| state.type_name == Some("EntryState"));
                    let any_state_index = states
                        .iter()
                        .position(|state| state.type_name == Some("AnyState"));
                    RuntimeStateMachineLayer {
                        global_id: layer.object.id,
                        name: layer.object.string_property("name").map(ToOwned::to_owned),
                        states,
                        entry_state_index,
                        any_state_index,
                    }
                })
                .collect(),
        })
        .collect()
}

fn runtime_state_machine_input(object: &RuntimeObject) -> Option<RuntimeStateMachineInput> {
    let (kind, value) = match object.type_name {
        "StateMachineBool" => (
            StateMachineInputKind::Bool,
            StateMachineInputValue::Bool(object.bool_property("value").unwrap_or(false)),
        ),
        "StateMachineNumber" => (
            StateMachineInputKind::Number,
            StateMachineInputValue::Number(object.double_property("value").unwrap_or(0.0)),
        ),
        "StateMachineTrigger" => (
            StateMachineInputKind::Trigger,
            StateMachineInputValue::Trigger {
                fired: false,
                used_layers: Vec::new(),
            },
        ),
        _ => return None,
    };

    Some(RuntimeStateMachineInput {
        global_id: object.id,
        name: object.string_property("name").map(ToOwned::to_owned),
        kind,
        value,
    })
}

fn artboard_index_for_graph(file: &RuntimeFile, graph: &ArtboardGraph) -> Option<usize> {
    file.artboards()
        .into_iter()
        .position(|artboard| artboard.id == graph.global_id)
}

fn artboard_object_range(file: &RuntimeFile, start: usize) -> Option<(usize, usize)> {
    let artboard = file.object(start)?;
    if artboard.type_name != "Artboard" {
        return None;
    }
    let end = ((start + 1)..file.objects.len())
        .find(|index| {
            file.object(*index)
                .is_some_and(|object| object.type_name == "Artboard")
        })
        .unwrap_or(file.objects.len());
    Some((start, end))
}

fn keyed_object_target<'a>(
    file: &'a RuntimeFile,
    slots: &[InstanceSlot],
    keyed_object: &RuntimeObject,
) -> Option<(usize, usize, &'a RuntimeObject)> {
    let object_id = usize::try_from(keyed_object.uint_property("objectId")?).ok()?;
    let slot = slots.get(object_id)?;
    let target = file.object(slot.source_global_id as usize)?;
    Some((object_id, slot.local_id, target))
}

fn transform_property_for_key(property_key: u16) -> Option<TransformProperty> {
    [
        ("Node", "x", TransformProperty::X),
        ("Node", "y", TransformProperty::Y),
        ("Node", "rotation", TransformProperty::Rotation),
        ("Node", "scaleX", TransformProperty::ScaleX),
        ("Node", "scaleY", TransformProperty::ScaleY),
        ("Node", "opacity", TransformProperty::Opacity),
    ]
    .into_iter()
    .find_map(|(type_name, property_name, property)| {
        (property_key_for_name(type_name, property_name) == Some(property_key)).then_some(property)
    })
}

fn property_key_for_name(type_name: &str, property_name: &str) -> Option<u16> {
    let definition = definition_by_name(type_name)?;
    if let Some(property) = definition
        .properties
        .iter()
        .find(|property| property.name == property_name)
    {
        return Some(property.key.int);
    }

    for ancestor in definition.ancestors {
        let ancestor = definition_by_name(ancestor)?;
        if let Some(property) = ancestor
            .properties
            .iter()
            .find(|property| property.name == property_name)
        {
            return Some(property.key.int);
        }
    }

    None
}

fn mix_value(current: f32, value: f32, mix: f32) -> f32 {
    if mix == 1.0 {
        value
    } else {
        current * (1.0 - mix) + value * mix
    }
}

fn normalized_interpolator_id(object: &RuntimeObject) -> Option<u64> {
    object
        .uint_property("interpolatorId")
        .filter(|id| *id != u64::from(u32::MAX) && *id != u64::MAX)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransformProperty {
    X,
    Y,
    Rotation,
    ScaleX,
    ScaleY,
    Opacity,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct RuntimeComponentCapabilities {
    pub world_transform: bool,
    pub transform: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TransformRuntimeState {
    pub x: f32,
    pub y: f32,
    pub rotation: f32,
    pub scale_x: f32,
    pub scale_y: f32,
    pub opacity: f32,
    pub local_transform: Mat2D,
    pub world_transform: Mat2D,
    pub render_opacity: f32,
}

impl TransformRuntimeState {
    fn from_object(object: &RuntimeObject) -> Self {
        Self {
            x: object.double_property("x").unwrap_or(0.0),
            y: object.double_property("y").unwrap_or(0.0),
            rotation: object.double_property("rotation").unwrap_or(0.0),
            scale_x: object.double_property("scaleX").unwrap_or(1.0),
            scale_y: object.double_property("scaleY").unwrap_or(1.0),
            opacity: object.double_property("opacity").unwrap_or(1.0),
            local_transform: Mat2D::IDENTITY,
            world_transform: Mat2D::IDENTITY,
            render_opacity: 0.0,
        }
    }

    fn property(&self, property: TransformProperty) -> f32 {
        match property {
            TransformProperty::X => self.x,
            TransformProperty::Y => self.y,
            TransformProperty::Rotation => self.rotation,
            TransformProperty::ScaleX => self.scale_x,
            TransformProperty::ScaleY => self.scale_y,
            TransformProperty::Opacity => self.opacity,
        }
    }

    fn set_property(&mut self, property: TransformProperty, value: f32) -> bool {
        let current = match property {
            TransformProperty::X => &mut self.x,
            TransformProperty::Y => &mut self.y,
            TransformProperty::Rotation => &mut self.rotation,
            TransformProperty::ScaleX => &mut self.scale_x,
            TransformProperty::ScaleY => &mut self.scale_y,
            TransformProperty::Opacity => &mut self.opacity,
        };
        if *current == value {
            return false;
        }
        *current = value;
        true
    }
}

impl Default for TransformRuntimeState {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
            opacity: 1.0,
            local_transform: Mat2D::IDENTITY,
            world_transform: Mat2D::IDENTITY,
            render_opacity: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2D(pub [f32; 6]);

impl Mat2D {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn from_rotation(radians: f32) -> Self {
        let (sin, cos) = if radians == 0.0 {
            (0.0, 1.0)
        } else {
            radians.sin_cos()
        };
        Self([cos, sin, -sin, cos, 0.0, 0.0])
    }

    pub fn multiply(self, rhs: Self) -> Self {
        let a = self.0;
        let b = rhs.0;
        Self([
            a[0] * b[0] + a[2] * b[1],
            a[1] * b[0] + a[3] * b[1],
            a[0] * b[2] + a[2] * b[3],
            a[1] * b[2] + a[3] * b[3],
            a[0] * b[4] + a[2] * b[5] + a[4],
            a[1] * b[4] + a[3] * b[5] + a[5],
        ])
    }

    pub fn scale_by_values(&mut self, scale_x: f32, scale_y: f32) {
        self.0[0] *= scale_x;
        self.0[1] *= scale_x;
        self.0[2] *= scale_y;
        self.0[3] *= scale_y;
    }
}

impl Default for Mat2D {
    fn default() -> Self {
        Self::IDENTITY
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ComponentDirt(pub u16);

impl ComponentDirt {
    pub const NONE: Self = Self(0);
    pub const COLLAPSED: Self = Self(1 << 0);
    pub const DEPENDENTS: Self = Self(1 << 1);
    pub const COMPONENTS: Self = Self(1 << 2);
    pub const DRAW_ORDER: Self = Self(1 << 3);
    pub const PATH: Self = Self(1 << 4);
    pub const TEXT_SHAPE: Self = Self(1 << 4);
    pub const SKIN: Self = Self(1 << 4);
    pub const VERTICES: Self = Self(1 << 5);
    pub const TEXT_COVERAGE: Self = Self(1 << 5);
    pub const TRANSFORM: Self = Self(1 << 6);
    pub const WORLD_TRANSFORM: Self = Self(1 << 7);
    pub const RENDER_OPACITY: Self = Self(1 << 8);
    pub const PAINT: Self = Self(1 << 9);
    pub const STOPS: Self = Self(1 << 10);
    pub const LAYOUT_STYLE: Self = Self(1 << 11);
    pub const BINDINGS: Self = Self(1 << 12);
    pub const N_SLICER: Self = Self(1 << 13);
    pub const SCRIPT_UPDATE: Self = Self(1 << 14);
    pub const CLIPPING: Self = Self(1 << 15);
    pub const FILTHY: Self = Self(0xFFFE);

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) == flag.0
    }
}

impl Default for ComponentDirt {
    fn default() -> Self {
        Self::NONE
    }
}

impl BitOr for ComponentDirt {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitOrAssign for ComponentDirt {
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for ComponentDirt {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl BitAndAssign for ComponentDirt {
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl Not for ComponentDirt {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UpdateComponentsReport {
    pub did_update: bool,
    pub steps: usize,
    pub updated_locals: Vec<usize>,
    pub max_steps_reached: bool,
}

impl RuntimeComponent {
    fn update_transform(&mut self) {
        if !self.capabilities.transform {
            return;
        }

        let mut transform = Mat2D::from_rotation(self.transform.rotation);
        transform.0[4] = self.transform.x;
        transform.0[5] = self.transform.y;
        transform.scale_by_values(self.transform.scale_x, self.transform.scale_y);
        self.transform.local_transform = transform;
    }

    fn update_world_transform(&mut self, parent_world: Option<Mat2D>) {
        if self.type_name == "Artboard" || !self.capabilities.transform {
            return;
        }

        self.transform.world_transform = match parent_world {
            Some(parent_world) => parent_world.multiply(self.transform.local_transform),
            None => self.transform.local_transform,
        };
    }

    fn update_render_opacity(&mut self, parent_opacity: f32) {
        if !self.capabilities.transform {
            return;
        }

        self.transform.render_opacity = self.transform.opacity * parent_opacity;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rive_binary::read_runtime_file;
    use rive_graph::GraphFile;

    fn synthetic_instance(
        components: Vec<RuntimeComponent>,
        update_order: Vec<usize>,
    ) -> ArtboardInstance {
        let component_by_local = components
            .iter()
            .enumerate()
            .map(|(index, component)| (component.local_id, index))
            .collect::<BTreeMap<_, _>>();

        ArtboardInstance {
            slots: components
                .iter()
                .enumerate()
                .map(|(index, component)| InstanceSlot {
                    local_id: component.local_id,
                    source_global_id: component.global_id,
                    type_name: Some(component.type_name),
                    name: None,
                    component_index: Some(index),
                })
                .collect(),
            components,
            component_by_local,
            update_order,
            linear_animations: Vec::new(),
            state_machines: Vec::new(),
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

    #[test]
    fn transform_update_matches_basic_cpp_order() {
        let mut root = synthetic_component(0, 0);
        root.type_name = "Artboard";
        root.transform.render_opacity = 0.5;
        let mut child = synthetic_component(1, 1);
        child.parent_local = Some(0);
        child.transform.x = 2.0;
        child.transform.y = 3.0;
        child.transform.scale_x = 4.0;
        child.transform.scale_y = 5.0;
        child.transform.opacity = 0.25;
        child.dirt = ComponentDirt::TRANSFORM
            | ComponentDirt::WORLD_TRANSFORM
            | ComponentDirt::RENDER_OPACITY;
        let mut instance = synthetic_instance(vec![root, child], vec![0, 1]);

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
        let mut component = synthetic_component(0, 0);
        component.dirt = ComponentDirt::NONE;
        let mut instance = synthetic_instance(vec![component], vec![0]);
        instance.dirt = ComponentDirt::NONE;
        instance.did_change = false;

        assert!(instance.set_transform_property(0, TransformProperty::X, 12.0));
        let component = instance.component(0).unwrap();
        assert_eq!(component.transform.x, 12.0);
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
        assert_eq!(component.transform.opacity, 0.35);
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
