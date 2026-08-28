use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::{
        state_machine_base::StateMachineBase, state_machine_listener_base::StateMachineListenerBase,
    },
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    listener_type::ListenerType,
    status_code::StatusCode,
};
#[derive(Default)]
pub struct StateMachineListener {
    pub base: StateMachineListenerBase,
    actions: Vec<CoreHandle>,
    listener_input_types: Vec<CoreHandle>,
}
impl StateMachineListener {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn has_listener(&self, kind: ListenerType) -> bool {
        self.listener_input_types.iter().any(|value| {
            value
                .with(|value| value.listener_input_type_value() == Some(kind as u32))
                .unwrap_or(false)
        })
    }
    pub fn has_listeners(&self, kinds: &[ListenerType]) -> bool {
        kinds.iter().copied().any(|kind| self.has_listener(kind))
    }
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }
    pub fn listener_input_type_count(&self) -> usize {
        self.listener_input_types.len()
    }
    pub fn action(&self, index: usize) -> Option<CoreHandle> {
        self.actions.get(index).cloned()
    }
    pub fn listener_input_type(&self, index: usize) -> Option<CoreHandle> {
        self.listener_input_types.get(index).cloned()
    }
    pub(crate) fn add_action(&mut self, value: CoreHandle) {
        self.actions.push(value);
    }
    pub(crate) fn add_listener_input_type(&mut self, value: CoreHandle) {
        self.listener_input_types.push(value);
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_listener(this);
        self.base.base.import(stack)
    }
    pub fn perform_changes(
        &self,
        machine: &mut crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance,
        invocation: &ListenerInvocation,
        mut dispatch: impl FnMut(
            &CoreHandle,
            &mut crate::mechanical_port::source::animation::state_machine_instance::StateMachineInstance,
            &ListenerInvocation,
        ),
    ) {
        for action in &self.actions {
            dispatch(action, machine, invocation);
        }
    }
}
impl std::ops::Deref for StateMachineListener {
    type Target = StateMachineListenerBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineListener {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::state_machine_component_base::StateMachineComponentBaseCallbacks for StateMachineListener { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
impl crate::mechanical_port::source::generated::animation::state_machine_listener_base::StateMachineListenerBaseCallbacks for StateMachineListener { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
