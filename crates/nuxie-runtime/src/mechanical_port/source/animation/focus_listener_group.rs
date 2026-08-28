use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    core::CoreHandle,
    focus_data::{FocusData, RuntimeFocusListenerHandle},
    listener_type::ListenerType,
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

#[derive(Clone)]
pub struct RuntimeFocusListenerGroupHandle(Rc<RefCell<FocusListenerGroup>>);

#[derive(Clone, Default)]
pub struct RuntimeFocusListenerGroupWeakHandle(Weak<RefCell<FocusListenerGroup>>);

impl RuntimeFocusListenerGroupHandle {
    pub fn new(
        focus_data: CoreHandle,
        listener: CoreHandle,
        state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    ) -> Self {
        let is_focus_listener = listener
            .with(|listener| listener.state_machine_listener_has(ListenerType::Focus))
            .flatten()
            .unwrap_or(false);
        let is_blur_listener = listener
            .with(|listener| listener.state_machine_listener_has(ListenerType::Blur))
            .flatten()
            .unwrap_or(false);
        let handle = Self(Rc::new(RefCell::new(FocusListenerGroup {
            occurrence: RuntimeFocusListenerGroupWeakHandle::default(),
            focus_data,
            listener,
            state_machine_instance,
            is_focus_listener,
            is_blur_listener,
        })));
        let occurrence = handle.downgrade();
        handle.0.borrow_mut().occurrence = occurrence.clone();
        let focus_data = handle.0.borrow().focus_data.clone();
        focus_data.with_downcast_mut::<FocusData, _>(|focus_data| {
            focus_data.add_focus_listener(RuntimeFocusListenerHandle::new(occurrence));
        });
        handle
    }

    pub fn downgrade(&self) -> RuntimeFocusListenerGroupWeakHandle {
        RuntimeFocusListenerGroupWeakHandle(Rc::downgrade(&self.0))
    }

    pub fn with_group<R>(&self, use_group: impl FnOnce(&FocusListenerGroup) -> R) -> R {
        use_group(&self.0.borrow())
    }

    pub fn with_group_mut<R>(&self, use_group: impl FnOnce(&mut FocusListenerGroup) -> R) -> R {
        use_group(&mut self.0.borrow_mut())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl RuntimeFocusListenerGroupWeakHandle {
    pub fn upgrade(&self) -> Option<RuntimeFocusListenerGroupHandle> {
        self.0.upgrade().map(RuntimeFocusListenerGroupHandle)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0)
    }
}

pub struct FocusListenerGroup {
    occurrence: RuntimeFocusListenerGroupWeakHandle,
    focus_data: CoreHandle,
    listener: CoreHandle,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    is_focus_listener: bool,
    is_blur_listener: bool,
}

impl FocusListenerGroup {
    pub fn listener(&self) -> CoreHandle {
        self.listener.clone()
    }

    pub fn focus_data(&self) -> CoreHandle {
        self.focus_data.clone()
    }

    pub fn is_focus_listener(&self) -> bool {
        self.is_focus_listener
    }

    pub fn is_blur_listener(&self) -> bool {
        self.is_blur_listener
    }

    pub fn on_focused(&mut self) {
        if self.is_focus_listener
            && let Some(group) = self.occurrence.upgrade()
        {
            self.state_machine_instance
                .with_instance_mut(|machine| machine.queue_focus_event(group, true));
        }
    }

    pub fn on_blurred(&mut self) {
        if self.is_blur_listener
            && let Some(group) = self.occurrence.upgrade()
        {
            self.state_machine_instance
                .with_instance_mut(|machine| machine.queue_focus_event(group, false));
        }
    }
}

impl Drop for FocusListenerGroup {
    fn drop(&mut self) {
        self.focus_data
            .with_downcast_mut::<FocusData, _>(|focus_data| {
                focus_data.remove_focus_listener(RuntimeFocusListenerHandle::new(
                    self.occurrence.clone(),
                ));
            });
    }
}
