use crate::mechanical_port::source::{
    generated::animation::scripted_transition_condition_base::ScriptedTransitionConditionBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
};
pub trait ScriptedConditionRuntime {
    fn dispose_script_inputs(&mut self);
    #[cfg(feature = "rive_scripting")]
    fn evaluate(&mut self) -> bool;
    fn stateful_condition(
        &mut self,
        stateless: *const (),
    ) -> Option<&mut dyn ScriptedConditionRuntime>;
    fn register_referencer(&mut self, stack: &mut ImportStack) -> StatusCode;
    fn register_scripted_object(&mut self, object: *mut ());
    fn add_property(&mut self, property: *mut ());
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
    pub fn evaluate_stateful(&mut self, _machine: *const (), _layer: *mut ()) -> bool {
        #[cfg(feature = "rive_scripting")]
        if let Some(runtime) = &mut self.runtime {
            return runtime.evaluate();
        }
        false
    }
    pub fn evaluate(&mut self, _machine: *const (), _layer: *mut ()) -> bool {
        #[cfg(feature = "rive_scripting")]
        if let Some(runtime) = &mut self.runtime {
            if let Some(stateful) = runtime.stateful_condition(self as *const Self as *const ()) {
                return stateful.evaluate();
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
    pub fn component(&self) -> Option<*mut ()> {
        None
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(runtime) = &mut self.runtime else {
            return StatusCode::MissingObject;
        };
        let code = runtime.register_referencer(stack);
        if code != StatusCode::Ok {
            return code;
        }
        runtime.register_scripted_object(self as *mut Self as *mut ());
        self.base.base.import(stack)
    }
    pub fn add_property(&mut self, property: *mut ()) {
        if let Some(runtime) = &mut self.runtime {
            runtime.add_property(property);
        }
    }
}
