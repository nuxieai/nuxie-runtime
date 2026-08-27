use std::ptr::NonNull;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SemanticActionType {
    Tap = 0,
    Increase = 1,
    Decrease = 2,
}
pub trait SemanticListenerGroupHost {
    fn add_semantic_listener(&mut self, listener: NonNull<SemanticListenerGroup>);
    fn remove_semantic_listener(&mut self, listener: NonNull<SemanticListenerGroup>);
}
pub trait SemanticListenerGroupStateMachine {
    fn constraints_met(&self, listener: *const (), action: SemanticActionType) -> bool;
    fn queue_semantic_event(
        &mut self,
        group: NonNull<SemanticListenerGroup>,
        action: SemanticActionType,
    );
}
pub struct SemanticListenerGroup {
    semantic_data: Option<NonNull<dyn SemanticListenerGroupHost>>,
    listener: *const (),
    state_machine_instance: NonNull<dyn SemanticListenerGroupStateMachine>,
}
impl SemanticListenerGroup {
    pub fn new(
        semantic_data: Option<NonNull<dyn SemanticListenerGroupHost>>,
        listener: *const (),
        state_machine_instance: NonNull<dyn SemanticListenerGroupStateMachine>,
    ) -> Box<Self> {
        let mut value = Box::new(Self {
            semantic_data,
            listener,
            state_machine_instance,
        });
        if let Some(mut data) = semantic_data {
            unsafe {
                data.as_mut()
                    .add_semantic_listener(NonNull::from(value.as_mut()))
            };
        }
        value
    }
    pub fn listener(&self) -> *const () {
        self.listener
    }
    pub fn semantic_data(&self) -> Option<NonNull<dyn SemanticListenerGroupHost>> {
        self.semantic_data
    }
    fn queue_if_listening(&mut self, action: SemanticActionType) {
        if !self.listener.is_null()
            && unsafe {
                self.state_machine_instance
                    .as_ref()
                    .constraints_met(self.listener, action)
            }
        {
            let this = NonNull::from(&mut *self);
            unsafe {
                self.state_machine_instance
                    .as_mut()
                    .queue_semantic_event(this, action)
            };
        }
    }
    pub fn on_semantic_tap(&mut self) {
        self.queue_if_listening(SemanticActionType::Tap);
    }
    pub fn on_semantic_increase(&mut self) {
        self.queue_if_listening(SemanticActionType::Increase);
    }
    pub fn on_semantic_decrease(&mut self) {
        self.queue_if_listening(SemanticActionType::Decrease);
    }
}
impl Drop for SemanticListenerGroup {
    fn drop(&mut self) {
        let this = NonNull::from(&mut *self);
        if let Some(mut data) = self.semantic_data {
            unsafe { data.as_mut().remove_semantic_listener(this) };
        }
    }
}
