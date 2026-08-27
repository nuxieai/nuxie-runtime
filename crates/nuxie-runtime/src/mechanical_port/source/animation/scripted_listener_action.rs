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
    fn add_scripted_object(&mut self, stack: &mut ImportStack, object: *mut ()) -> StatusCode;
    fn clone_base(&self, base: &ScriptedListenerActionBase) -> ScriptedListenerActionBase;
    fn clone_runtime(&self) -> Box<dyn ScriptedListenerRuntime>;
    fn clone_properties_to(
        &self,
        target: &mut dyn ScriptedListenerRuntime,
        data_bind_container: *mut (),
    );
    fn reinit(&mut self);
    fn set_script_input_owner(&mut self, property: *mut (), owner: *mut ());
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
    pub fn script_protocol(&self) -> u8 {
        5
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
