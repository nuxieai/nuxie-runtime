use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
    },
    artboard::RuntimeArtboardInstanceHandle,
    core::CoreHandle,
    generated::animation::scripted_transition_condition_base::ScriptedTransitionConditionBase,
    importers::import_stack::ImportStack,
    status_code::StatusCode,
};
pub trait ScriptedConditionRuntime {
    fn dispose_script_inputs(&mut self);
    fn evaluate(&mut self) -> bool;
    fn evaluate_stateful(
        &mut self,
        stateful: &CoreHandle,
        machine: &StateMachineInstance,
        layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool;
    fn stateful_condition(&mut self, stateless: &CoreHandle) -> Option<CoreHandle>;
    fn register_referencer(&mut self, stack: &mut ImportStack) -> StatusCode;
    fn add_scripted_object(&mut self, stack: &mut ImportStack, object: CoreHandle) -> StatusCode;
    fn clone_base(&self, base: &ScriptedTransitionConditionBase)
    -> ScriptedTransitionConditionBase;
    fn clone_runtime(&self) -> Box<dyn ScriptedConditionRuntime>;
    fn clone_properties_to(
        &self,
        target: &mut dyn ScriptedConditionRuntime,
        data_bind_container: &RuntimeArtboardInstanceHandle,
    );
    fn reinit(&mut self);
    fn set_script_input_owner(&mut self, property: &CoreHandle, owner: CoreHandle);
    fn add_property(&mut self, property: CoreHandle);
}
#[derive(Default)]
pub struct ScriptedTransitionCondition {
    pub base: ScriptedTransitionConditionBase,
    runtime: Option<Box<dyn ScriptedConditionRuntime>>,
}
impl Drop for ScriptedTransitionCondition {
    fn drop(&mut self) {
        self.dispose_script_inputs();
    }
}
impl ScriptedTransitionCondition {
    pub fn dispose_script_inputs(&mut self) {
        if let Some(runtime) = &mut self.runtime {
            runtime.dispose_script_inputs();
        }
    }
    pub fn evaluate_stateful(
        &mut self,
        machine: &StateMachineInstance,
        layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(this) = self.base.base.base.handle() else {
            return false;
        };
        self.runtime
            .as_mut()
            .is_some_and(|runtime| runtime.evaluate_stateful(&this, machine, layer))
    }
    pub fn evaluate(
        &mut self,
        machine: &StateMachineInstance,
        layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        let Some(this) = self.base.base.base.handle() else {
            return false;
        };
        if let Some(runtime) = &mut self.runtime {
            if let Some(stateful) = runtime.stateful_condition(&this) {
                return runtime.evaluate_stateful(&stateful, machine, layer);
            }
        }
        false
    }
    pub fn asset_id(&self) -> u32 {
        self.base.script_asset_id()
    }
    pub fn add_scripted_dirt(&mut self, _value: u32, _recurse: bool) -> bool {
        false
    }
    pub fn component(&self) -> Option<CoreHandle> {
        None
    }
    pub fn script_protocol(&self) -> u8 {
        6
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(runtime) = &mut self.runtime else {
            return StatusCode::MissingObject;
        };
        let code = runtime.register_referencer(stack);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        let code = runtime.add_scripted_object(stack, this);
        if code != StatusCode::Ok {
            return code;
        }
        self.base.base.import(stack)
    }
    pub fn add_property(&mut self, property: CoreHandle) {
        let Some(this) = self.base.base.base.handle() else {
            return;
        };
        if let Some(runtime) = &mut self.runtime {
            runtime.set_script_input_owner(&property, this);
            runtime.add_property(property);
        }
    }
    pub fn clone_definition(&self) -> Self {
        let runtime = self.runtime.as_ref().map(|runtime| runtime.clone_runtime());
        let base = self
            .runtime
            .as_ref()
            .map(|runtime| runtime.clone_base(&self.base))
            .unwrap_or_default();
        Self { base, runtime }
    }
    pub fn clone_scripted_object(
        &self,
        data_bind_container: &RuntimeArtboardInstanceHandle,
    ) -> Self {
        let mut clone = self.clone_definition();
        if let (Some(source), Some(target)) = (&self.runtime, &mut clone.runtime) {
            source.clone_properties_to(target.as_mut(), data_bind_container);
            target.reinit();
        }
        clone
    }
}
