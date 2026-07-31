use super::StateMachineReportedEvent;
use crate::ArtboardInstance;

/// One authored pinned-C++ `StateMachineFireEvent` occurrence.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineFireEvent {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeStateMachineFireEvent {
    #[cfg(test)]
    pub(crate) fn for_test(occurs_value: u64, event_local_id: Option<usize>) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("StateMachineFireEvent");
        action_owner.set_uint(
            super::listener_action_owner::FIRE_OCCURS_VALUE_KEY,
            occurs_value,
        );
        action_owner.set_uint(
            super::listener_action_owner::FIRE_EVENT_ID_KEY,
            event_local_id
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(u64::from(u32::MAX)),
        );
        Self { action_owner }
    }

    /// Resolve and report the live Event when the action performs.
    ///
    /// Pinned C++ deliberately resolves `eventId` from the live Artboard at
    /// perform time, validates the concrete Event type, and reports that
    /// occurrence (`state_machine_fire_event.cpp:13-22`).
    pub(crate) fn perform(
        &self,
        artboard: &ArtboardInstance,
        reported_events: &mut Vec<StateMachineReportedEvent>,
    ) {
        let event_local_id = self
            .action_owner
            .uint(super::listener_action_owner::FIRE_EVENT_ID_KEY);
        let Ok(event_local_id) = usize::try_from(event_local_id) else {
            return;
        };
        let Some(event) = crate::event::trigger_event(artboard, event_local_id, 0.0, None) else {
            return;
        };
        reported_events.push(event);
    }
}
