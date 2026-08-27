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
    fn add_scripted_object(&mut self, stack: &mut ImportStack, object: *mut ()) -> StatusCode;
    fn clone_base(&self, base: &ScriptedTransitionConditionBase)
        -> ScriptedTransitionConditionBase;
    fn clone_runtime(&self) -> Box<dyn ScriptedConditionRuntime>;
    fn clone_properties_to(
        &self,
        target: &mut dyn ScriptedConditionRuntime,
        data_bind_container: *mut (),
    );
    fn reinit(&mut self);
    fn set_script_input_owner(&mut self, property: *mut (), owner: *mut ());
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
        let code = runtime.add_scripted_object(stack, self as *mut Self as *mut ());
        if code != StatusCode::Ok {
            return code;
        }
        self.base.base.import(stack)
    }
    pub fn add_property(&mut self, property: *mut ()) {
        if let Some(runtime) = &mut self.runtime {
            runtime.set_script_input_owner(property, self as *mut Self as *mut ());
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
    pub fn clone_scripted_object(&self, data_bind_container: *mut ()) -> Self {
        let mut clone = self.clone_definition();
        if let (Some(source), Some(target)) = (&self.runtime, &mut clone.runtime) {
            source.clone_properties_to(target.as_mut(), data_bind_container);
            target.reinit();
        }
        clone
    }
}
