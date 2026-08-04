// State-machine instance integration for the C++ `rive_profile.cpp` source.
use super::*;
impl StateMachineInstance {
    pub(super) fn record_bind_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        self.bind_phase_trace.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }
    pub(super) fn record_event_dispatch_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        {
            self.event_dispatch_phase_trace.push(phase);
            if let Some((local, audio, total_order)) = &self.event_total_order_trace {
                match phase {
                    "local-dispatch" => total_order.borrow_mut().push(local),
                    "recorded-audio-seam" => total_order.borrow_mut().push(audio),
                    _ => {}
                }
            }
        }
        #[cfg(not(test))]
        let _ = phase;
    }
    pub(super) fn record_advance_phase(&mut self, phase: &'static str) {
        #[cfg(test)]
        self.advance_phase_trace.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }
    pub(super) fn record_constructor_phase(&mut self, phase: RuntimeConstructorPhase) {
        #[cfg(test)]
        self.constructor_phases.push(phase);
        #[cfg(not(test))]
        let _ = phase;
    }
    pub(super) fn record_drop_phase(&self, phase: &'static str) {
        #[cfg(test)]
        if let Some(receipt) = self.drop_phase_receipt.as_ref() {
            receipt.borrow_mut().push(phase);
        }
        #[cfg(not(test))]
        let _ = phase;
    }
    pub(super) fn profile_name(&self) -> &str {
        &self.profile_name
    }
}
