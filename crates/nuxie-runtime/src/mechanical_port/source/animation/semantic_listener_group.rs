use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    core::CoreHandle,
    semantic::{
        semantic_data::{SemanticData, SemanticListenerRef},
        semantic_listener::SemanticListener,
    },
};
use std::{
    cell::RefCell,
    fmt,
    rc::{Rc, Weak},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SemanticActionType {
    Tap = 0,
    Increase = 1,
    Decrease = 2,
}

impl SemanticActionType {
    pub const fn from_raw(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::Tap,
            1 => Self::Increase,
            2 => Self::Decrease,
            _ => return None,
        })
    }
}

#[derive(Clone)]
pub struct RuntimeSemanticListenerGroupHandle(Rc<RefCell<SemanticListenerGroup>>);

#[derive(Clone, Default)]
struct RuntimeSemanticListenerGroupWeakHandle(Weak<RefCell<SemanticListenerGroup>>);

impl RuntimeSemanticListenerGroupHandle {
    pub fn new(
        semantic_data: CoreHandle,
        listener: CoreHandle,
        state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    ) -> Self {
        let handle = Self(Rc::new(RefCell::new(SemanticListenerGroup {
            occurrence: RuntimeSemanticListenerGroupWeakHandle::default(),
            semantic_data,
            listener,
            state_machine_instance,
            registration: None,
        })));
        let occurrence = handle.downgrade();
        let registration: SemanticListenerRef = Rc::new(SemanticGroupListener {
            group: occurrence.clone(),
        });
        {
            let mut group = handle.0.borrow_mut();
            group.occurrence = occurrence;
            group.registration = Some(registration.clone());
            group
                .semantic_data
                .with_downcast_mut::<SemanticData, _>(|semantic_data| {
                    semantic_data.add_semantic_listener(registration);
                });
        }
        handle
    }

    fn downgrade(&self) -> RuntimeSemanticListenerGroupWeakHandle {
        RuntimeSemanticListenerGroupWeakHandle(Rc::downgrade(&self.0))
    }

    pub fn with_group<R>(&self, use_group: impl FnOnce(&SemanticListenerGroup) -> R) -> R {
        use_group(&self.0.borrow())
    }

    pub fn with_group_mut<R>(&self, use_group: impl FnOnce(&mut SemanticListenerGroup) -> R) -> R {
        use_group(&mut self.0.borrow_mut())
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl RuntimeSemanticListenerGroupWeakHandle {
    fn upgrade(&self) -> Option<RuntimeSemanticListenerGroupHandle> {
        self.0.upgrade().map(RuntimeSemanticListenerGroupHandle)
    }
}

struct SemanticGroupListener {
    group: RuntimeSemanticListenerGroupWeakHandle,
}

impl fmt::Debug for SemanticGroupListener {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SemanticGroupListener")
    }
}

impl SemanticListener for SemanticGroupListener {
    fn on_semantic_tap(&self) {
        if let Some(group) = self.group.upgrade() {
            group.with_group_mut(SemanticListenerGroup::on_semantic_tap);
        }
    }

    fn on_semantic_increase(&self) {
        if let Some(group) = self.group.upgrade() {
            group.with_group_mut(SemanticListenerGroup::on_semantic_increase);
        }
    }

    fn on_semantic_decrease(&self) {
        if let Some(group) = self.group.upgrade() {
            group.with_group_mut(SemanticListenerGroup::on_semantic_decrease);
        }
    }
}

pub struct SemanticListenerGroup {
    occurrence: RuntimeSemanticListenerGroupWeakHandle,
    semantic_data: CoreHandle,
    listener: CoreHandle,
    state_machine_instance: RuntimeStateMachineInstanceWeakHandle,
    registration: Option<SemanticListenerRef>,
}

impl SemanticListenerGroup {
    pub fn listener(&self) -> CoreHandle {
        self.listener.clone()
    }

    pub fn semantic_data(&self) -> CoreHandle {
        self.semantic_data.clone()
    }

    fn queue_if_listening(&mut self, action: SemanticActionType) {
        let Some(group) = self.occurrence.upgrade() else {
            return;
        };
        self.state_machine_instance.with_instance_mut(|machine| {
            if machine.semantic_constraints_met(&self.listener, action) {
                machine.queue_semantic_event(group, action);
            }
        });
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
        let Some(registration) = self.registration.take() else {
            return;
        };
        self.semantic_data
            .with_downcast_mut::<SemanticData, _>(|semantic_data| {
                semantic_data.remove_semantic_listener(&registration);
            });
    }
}
