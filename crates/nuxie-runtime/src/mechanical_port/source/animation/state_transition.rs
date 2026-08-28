use crate::mechanical_port::source::{
    animation::{
        layer_state::LayerState,
        state_instance::RuntimeStateInstanceHandle,
        state_machine_instance::{
            RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
        },
    },
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::{
        keyframe_interpolator_base::KeyFrameInterpolatorBase, layer_state_base::LayerStateBase,
        state_transition_base::StateTransitionBase,
    },
    importers::{import_stack::ImportStack, layer_state_importer::LayerStateImporter},
    status_code::StatusCode,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllowTransition {
    No,
    WaitingForExit,
    Yes,
}
pub trait TransitionRuntime {
    fn evaluate_condition(
        &self,
        condition: &CoreHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool;
    fn use_condition_in_layer(
        &self,
        condition: &CoreHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
    );
    fn animation_duration(&self, state: &LayerState) -> Option<f32>;
    fn exit_animation(&self, state: &LayerState) -> Option<(f32, f32)>;
    fn exit_instance_times(
        &self,
        from: &RuntimeStateInstanceHandle,
    ) -> Option<(f32, f32, f32, i32)>;
    fn set_exit_instance_time(&self, from: &RuntimeStateInstanceHandle, time: f32);
}
pub struct StateTransition {
    pub base: StateTransitionBase,
    state_to: Option<CoreHandle>,
    evaluated_random_weight: u32,
    interpolator: Option<CoreHandle>,
    conditions: Vec<CoreHandle>,
}
impl Default for StateTransition {
    fn default() -> Self {
        Self {
            base: StateTransitionBase::default(),
            state_to: None,
            evaluated_random_weight: 1,
            interpolator: None,
            conditions: Vec::new(),
        }
    }
}
impl StateTransition {
    fn flags(&self) -> u32 {
        self.base.flags()
    }
    pub fn state_to(&self) -> Option<CoreHandle> {
        self.state_to.clone()
    }
    pub fn set_state_to(&mut self, state: Option<CoreHandle>) {
        self.state_to = state;
    }
    pub fn interpolator(&self) -> Option<CoreHandle> {
        self.interpolator.clone()
    }
    pub fn evaluated_random_weight(&self) -> u32 {
        self.evaluated_random_weight
    }
    pub fn set_evaluated_random_weight(&mut self, v: u32) {
        self.evaluated_random_weight = v
    }
    pub(crate) fn add_condition(&mut self, c: CoreHandle) {
        self.conditions.push(c)
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        if self.base.interpolator_id() != u32::MAX {
            let Some(interpolator) = context.resolve(self.base.interpolator_id()) else {
                return StatusCode::MissingObject;
            };
            if !interpolator.is_type_of(KeyFrameInterpolatorBase::TYPE_KEY) {
                return StatusCode::MissingObject;
            }
            self.interpolator = Some(interpolator)
        }
        for condition in self.conditions.iter().cloned() {
            let code = condition
                .with_mut(|condition| condition.transition_condition_on_added_dirty(context))
                .flatten()
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for condition in self.conditions.iter().cloned() {
            let code = condition
                .with_mut(|condition| condition.transition_condition_on_added_clean(context))
                .flatten()
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(i) = stack.latest::<LayerStateImporter>(LayerStateBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        i.add_transition(this);
        self.base.base.base.base.import(stack)
    }
    pub fn is_disabled(&self) -> bool {
        self.flags() & 1 != 0
    }
    pub fn pause_on_exit(&self) -> bool {
        self.flags() & 2 != 0
    }
    pub fn enable_exit_time(&self) -> bool {
        self.flags() & 4 != 0
    }
    pub fn enable_early_exit(&self) -> bool {
        self.flags() & 8 != 0
    }
    pub fn duration_is_percentage(&self) -> bool {
        self.flags() & 16 != 0
    }
    pub fn condition_count(&self) -> usize {
        self.conditions.len()
    }
    pub fn condition(&self, i: usize) -> Option<CoreHandle> {
        self.conditions.get(i).cloned()
    }
    pub fn mix_time(&self, from: &LayerState, r: &dyn TransitionRuntime) -> f32 {
        if self.base.duration() == 0 {
            return 0.0;
        }
        if self.duration_is_percentage() {
            self.base.duration() as f32 / 100.0 * r.animation_duration(from).unwrap_or(0.0)
        } else {
            self.base.duration() as f32 / 1000.0
        }
    }
    pub fn exit_time_seconds(
        &self,
        from: &LayerState,
        absolute: bool,
        r: &dyn TransitionRuntime,
    ) -> f32 {
        if self.flags() & 32 != 0 {
            let (start, duration) = r.exit_animation(from).unwrap_or((0.0, 0.0));
            (if absolute { start } else { 0.0 }) + self.base.exit_time() as f32 / 100.0 * duration
        } else {
            self.base.exit_time() as f32 / 1000.0
        }
    }
    pub fn allowed(
        &self,
        from: &RuntimeStateInstanceHandle,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
        r: &dyn TransitionRuntime,
    ) -> AllowTransition {
        if self.is_disabled() {
            return AllowTransition::No;
        }
        for c in &self.conditions {
            if !r.evaluate_condition(c, machine, layer.clone()) {
                return AllowTransition::No;
            }
        }
        if self.enable_exit_time() {
            if let Some((last, total, duration, loop_value)) = r.exit_instance_times(from) {
                let mut exit = if self.flags() & 32 != 0 {
                    self.base.exit_time() as f32 / 100.0 * duration
                } else {
                    self.base.exit_time() as f32 / 1000.0
                };
                if exit <= duration && loop_value != 0 {
                    exit += (last / duration).floor() * duration
                }
                if total < exit {
                    return AllowTransition::WaitingForExit;
                }
            }
        }
        AllowTransition::Yes
    }
    pub fn apply_exit_condition(
        &self,
        from: &RuntimeStateInstanceHandle,
        state: &LayerState,
        r: &dyn TransitionRuntime,
    ) -> bool {
        let use_exit = self.enable_exit_time() && r.animation_duration(state).is_some();
        if self.pause_on_exit() && use_exit {
            r.set_exit_instance_time(from, self.exit_time_seconds(state, true, r));
            return true;
        }
        use_exit
    }
    pub fn use_layer_in_conditions(
        &self,
        machine: &mut StateMachineInstance,
        layer: RuntimeStateMachineLayerInstanceWeakHandle,
        r: &dyn TransitionRuntime,
    ) {
        for c in &self.conditions {
            r.use_condition_in_layer(c, machine, layer.clone())
        }
    }
}
impl std::ops::Deref for StateTransition {
    type Target = StateTransitionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateTransition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::state_transition_base::StateTransitionBaseCallbacks for StateTransition { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
