use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation,
        state_machine_fire_action::StateMachineFireOccurance,
    },
    generated::animation::{
        listener_action_base::ListenerActionBase,
        state_machine_layer_component_base::StateMachineLayerComponentBase,
        state_machine_listener_base::StateMachineListenerBase,
    },
    importers::{
        import_stack::ImportStack,
        state_machine_layer_component_importer::StateMachineLayerComponentImporter,
        state_machine_listener_importer::StateMachineListenerImporter,
    },
    status_code::StatusCode,
};
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ParentKind {
    Listener = 0,
    Transition = 1,
    State = 2,
}
#[derive(Default)]
pub struct ListenerAction {
    pub base: ListenerActionBase,
}
pub trait ListenerActionBehavior {
    fn perform(&self, state_machine_instance: *mut (), invocation: &ListenerInvocation);
}
impl ListenerAction {
    pub fn matches_scheduled_occurrence(&self, occurs: StateMachineFireOccurance) -> bool {
        (self.base.flags() & 1) == occurs.0 as u32
    }
    pub fn parent_kind(&self) -> ParentKind {
        match (self.base.flags() >> 1) & 3 {
            1 => ParentKind::Transition,
            2 => ParentKind::State,
            _ => ParentKind::Listener,
        }
    }
    pub fn import(self: Box<Self>, stack: &mut ImportStack) -> StatusCode {
        let raw = Box::into_raw(self);
        match unsafe { (*raw).parent_kind() } {
            ParentKind::Listener => {
                let Some(importer) = stack
                    .latest::<StateMachineListenerImporter>(StateMachineListenerBase::TYPE_KEY)
                else {
                    unsafe { drop(Box::from_raw(raw)) };
                    return StatusCode::MissingObject;
                };
                importer.add_action(unsafe { Box::from_raw(raw) });
            }
            ParentKind::Transition | ParentKind::State => {
                let Some(importer) = stack.latest::<StateMachineLayerComponentImporter>(
                    StateMachineLayerComponentBase::TYPE_KEY,
                ) else {
                    unsafe { drop(Box::from_raw(raw)) };
                    return StatusCode::MissingObject;
                };
                importer.add_listener_action(unsafe { Box::from_raw(raw) });
            }
        }
        unsafe { (*raw).base.base.import(stack) }
    }
}
