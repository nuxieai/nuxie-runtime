//! Live `NestedTrigger` callback forwarding.
//!
//! `NestedTrigger::fire` is a callback rather than a stored uint property. It
//! ignores the callback payload and fires the retained child `SMITrigger` on
//! every invocation (`src/animation/nested_trigger.cpp:9-20`).

use crate::ArtboardInstance;

impl ArtboardInstance {
    pub(crate) fn fire_nested_trigger_input(&mut self, local_id: usize) -> bool {
        let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id) else {
            return false;
        };
        self.fire_nested_state_machine_trigger(state_machine_local_id, input_id)
    }
}
