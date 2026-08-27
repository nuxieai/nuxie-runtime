use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    generated::animation::scripted_listener_action_base::ScriptedListenerActionBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
};
pub trait ScriptedListenerRuntime {
    fn dispose_script_inputs(&mut self);
    fn perform_action_or_legacy(&mut self, invocation: &ListenerInvocation);
    fn stateful_listener(
        &mut self,
        stateless: *const (),
    ) -> Option<&mut dyn ScriptedListenerRuntime>;
    fn register_referencer(&mut self, stack: &mut ImportStack) -> StatusCode;
    fn register_scripted_object(&mut self, object: *mut ());
    fn clone_properties_to(&self, target: &mut dyn ScriptedListenerRuntime);
    fn reinit(&mut self);
    fn add_property(&mut self, property: *mut ());
}
#[derive(Default)]
pub struct ScriptedListenerAction {
    pub base: ScriptedListenerActionBase,
    runtime: Option<Box<dyn ScriptedListenerRuntime>>,
}
impl Drop for ScriptedListenerAction {
    fn drop(&mut self) {
        self.dispose_script_inputs();
    }
}
impl ScriptedListenerAction {
    pub fn dispose_script_inputs(&mut self) {
        if let Some(runtime) = &mut self.runtime {
            runtime.dispose_script_inputs();
        }
    }
    pub fn perform_stateful(&mut self, _machine: *mut (), invocation: &ListenerInvocation) {
        #[cfg(feature = "rive_scripting")]
        if let Some(runtime) = &mut self.runtime {
            runtime.perform_action_or_legacy(invocation);
        }
    }
    pub fn perform(&mut self, machine: *mut (), invocation: &ListenerInvocation) {
        #[cfg(feature = "rive_scripting")]
        if let Some(runtime) = &mut self.runtime {
            if let Some(stateful) = runtime.stateful_listener(self as *const Self as *const ()) {
                stateful.perform_action_or_legacy(invocation);
            }
        }
        let _ = machine;
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
