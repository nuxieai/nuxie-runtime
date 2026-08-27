use crate::mechanical_port::source::{
    animation::{layer_state::LayerState, transition_condition::TransitionCondition},
    generated::animation::{
        layer_state_base::LayerStateBase, state_transition_base::StateTransitionBase,
    },
    importers::{import_stack::ImportStack, layer_state_importer::LayerStateImporter},
    status_code::StatusCode,
};
use std::ptr::NonNull;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AllowTransition {
    No,
    WaitingForExit,
    Yes,
}
pub trait TransitionRuntime {
    fn resolve_interpolator(&self, id: u32) -> Option<*mut ()>;
    fn condition_added_dirty(&mut self, c: &mut TransitionCondition) -> StatusCode;
    fn condition_added_clean(&mut self, c: &mut TransitionCondition) -> StatusCode;
    fn evaluate_condition(&self, c: &TransitionCondition, machine: *mut (), layer: *mut ())
    -> bool;
    fn use_condition_in_layer(&self, c: &TransitionCondition, machine: *mut (), layer: *mut ());
    fn animation_duration(&self, state: &LayerState) -> Option<f32>;
    fn exit_animation(&self, state: &LayerState) -> Option<(f32, f32)>;
    fn exit_instance_times(&self, from: *mut ()) -> Option<(f32, f32, f32, i32)>;
    fn set_exit_instance_time(&self, from: *mut (), time: f32);
}
pub struct StateTransition {
    pub base: StateTransitionBase,
    state_to: Option<NonNull<LayerState>>,
    evaluated_random_weight: u32,
    interpolator: Option<*mut ()>,
    conditions: Vec<Box<TransitionCondition>>,
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
    pub fn state_to(&self) -> Option<&LayerState> {
        self.state_to.map(|v| unsafe { v.as_ref() })
    }
    pub fn interpolator(&self) -> Option<*mut ()> {
        self.interpolator
    }
    pub fn evaluated_random_weight(&self) -> u32 {
        self.evaluated_random_weight
    }
    pub fn set_evaluated_random_weight(&mut self, v: u32) {
        self.evaluated_random_weight = v
    }
    pub(crate) fn add_condition(&mut self, c: Box<TransitionCondition>) {
        self.conditions.push(c)
    }
    pub fn on_added_dirty(&mut self, r: &mut dyn TransitionRuntime) -> StatusCode {
        if self.base.interpolator_id() != u32::MAX {
            let Some(i) = r.resolve_interpolator(self.base.interpolator_id()) else {
                return StatusCode::MissingObject;
            };
            self.interpolator = Some(i)
        }
        for c in &mut self.conditions {
            let s = r.condition_added_dirty(c);
            if s != StatusCode::Ok {
                return s;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, r: &mut dyn TransitionRuntime) -> StatusCode {
        for c in &mut self.conditions {
            let s = r.condition_added_clean(c);
            if s != StatusCode::Ok {
                return s;
            }
        }
        StatusCode::Ok
    }
    pub fn import(self: Box<Self>, stack: &mut ImportStack) -> StatusCode {
        let raw = Box::into_raw(self);
        let Some(i) = stack.latest::<LayerStateImporter>(LayerStateBase::TYPE_KEY) else {
            unsafe { drop(Box::from_raw(raw)) };
            return StatusCode::MissingObject;
        };
        i.add_transition(unsafe { Box::from_raw(raw) });
        unsafe { (*raw).base.base.import(stack) }
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
    pub fn condition(&self, i: usize) -> Option<&TransitionCondition> {
        self.conditions.get(i).map(Box::as_ref)
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
        from: *mut (),
        machine: *mut (),
        layer: *mut (),
        r: &dyn TransitionRuntime,
    ) -> AllowTransition {
        if self.is_disabled() {
            return AllowTransition::No;
        }
        for c in &self.conditions {
            if !r.evaluate_condition(c, machine, layer) {
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
        from: *mut (),
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
    pub fn use_layer_in_conditions(&self, m: *mut (), l: *mut (), r: &dyn TransitionRuntime) {
        for c in &self.conditions {
            r.use_condition_in_layer(c, m, l)
        }
    }
}
