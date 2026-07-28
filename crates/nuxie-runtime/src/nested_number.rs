//! Live `NestedNumber` input forwarding.
//!
//! The serialized `m_NestedValue` seeds construction only. Runtime reads and
//! writes use the child `SMINumber` occurrence and do not rewrite or dirty the
//! parent Core property (`src/animation/nested_number.cpp:9-48`).

use crate::ArtboardInstance;

impl ArtboardInstance {
    pub(crate) fn nested_number_value(&self, local_id: usize) -> Option<f32> {
        let (state_machine_local_id, input_id) = self.nested_input_target(local_id)?;
        self.nested_state_machine(state_machine_local_id)?
            .input(input_id)?
            .number_value()
    }

    pub(crate) fn set_nested_number_value(&mut self, local_id: usize, value: f32) -> bool {
        let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id) else {
            return false;
        };
        self.set_nested_state_machine_number(state_machine_local_id, input_id, value)
    }
}
