use crate::mechanical_port::source::{
    animation::{
        listener_action::{ListenerAction, ListenerActionBehavior},
        listener_invocation::ListenerInvocation,
        listener_types::listener_input_type::ListenerInputType,
    },
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
    actions: Vec<Box<ListenerAction>>,
    listener_input_types: Vec<Box<ListenerInputType>>,
}
impl StateMachineListener {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn has_listener(&self, kind: ListenerType) -> bool {
        self.listener_input_types
            .iter()
            .any(|value| value.base.listener_type_value() == kind as u32)
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
    pub fn action(&self, index: usize) -> Option<&ListenerAction> {
        self.actions.get(index).map(Box::as_ref)
    }
    pub fn listener_input_type(&self, index: usize) -> Option<&ListenerInputType> {
        self.listener_input_types.get(index).map(Box::as_ref)
    }
    pub(crate) fn add_action(&mut self, value: Box<ListenerAction>) {
        self.actions.push(value);
    }
    pub(crate) fn add_listener_input_type(&mut self, value: Box<ListenerInputType>) {
        self.listener_input_types.push(value);
    }
    pub fn import(self: Box<Self>, stack: &mut ImportStack) -> StatusCode {
        let raw = Box::into_raw(self);
        let Some(importer) = stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(raw)) };
            return StatusCode::MissingObject;
        };
        importer.add_listener(unsafe { Box::from_raw(raw) });
        unsafe { (*raw).base.base.import(stack) }
    }
    pub fn perform_changes(
        &self,
        machine: *mut (),
        invocation: &ListenerInvocation,
        dispatch: impl Fn(&ListenerAction, *mut (), &ListenerInvocation),
    ) {
        for action in &self.actions {
            dispatch(action, machine, invocation);
        }
    }
}
