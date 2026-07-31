use super::StateMachineReportedEvent;
use crate::ArtboardInstance;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerFireEvent {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeListenerFireEvent {
    #[cfg(test)]
    pub(crate) fn for_test(flags: u64, event_local_id: Option<usize>) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("ListenerFireEvent");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        action_owner.set_uint(
            super::listener_action_owner::LISTENER_FIRE_EVENT_ID_KEY,
            event_local_id
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(u64::from(u32::MAX)),
        );
        Self { action_owner }
    }

    /// Resolve the authored id against the live occurrence at perform time,
    /// matching pinned C++ `ListenerFireEvent::perform`.
    pub(crate) fn perform(&self, artboard: &ArtboardInstance) -> Option<StateMachineReportedEvent> {
        // `ListenerFireEvent::perform` explicitly ignores its invocation.
        // The new report therefore resolves the live Event payload but never
        // inherits the pointer/event context that caused the listener.
        let event_local_id = self
            .action_owner
            .uint(super::listener_action_owner::LISTENER_FIRE_EVENT_ID_KEY);
        crate::event::trigger_event(artboard, usize::try_from(event_local_id).ok()?, 0.0, None)
    }
}
