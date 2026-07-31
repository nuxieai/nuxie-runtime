//! Direct owner for pinned C++ `src/event.cpp`.

use crate::ArtboardInstance;
use crate::state_machine::{StateMachineEventContext, StateMachineReportedEvent};

/// Trigger one live Event occurrence into its owning report context.
///
/// Pinned C++ delegates directly to `value.context()->reportEvent(this,
/// value.delaySeconds())`. Rust's context is an owner-safe queue boundary, so
/// this returns the live projection for that queue to retain.
pub(crate) fn trigger_event(
    artboard: &ArtboardInstance,
    event_local_index: usize,
    seconds_delay: f32,
    context: Option<StateMachineEventContext>,
) -> Option<StateMachineReportedEvent> {
    let mut report =
        StateMachineReportedEvent::from_live_artboard_event(artboard, event_local_index)?;
    report.seconds_delay = seconds_delay;
    report.context = context;
    Some(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trigger_owner_is_a_thin_live_projection() {
        let signature: fn(
            &ArtboardInstance,
            usize,
            f32,
            Option<StateMachineEventContext>,
        ) -> Option<StateMachineReportedEvent> = trigger_event;
        let _ = signature;
    }
}
