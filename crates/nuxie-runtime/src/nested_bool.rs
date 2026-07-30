//! Live `NestedBool` input forwarding.
//!
//! Pinned C++ uses the generated `m_NestedValue` only while initializing the
//! nested occurrence. Afterwards the virtual getter and setter address the
//! child `SMIBool` directly; parent-artboard property storage is not mutated
//! (`src/animation/nested_bool.cpp:9-48`).

use crate::ArtboardInstance;

impl ArtboardInstance {
    pub(crate) fn nested_bool_value(&self, local_id: usize) -> Option<bool> {
        let (state_machine_local_id, input_id) = self.nested_input_target(local_id)?;
        self.nested_state_machine(state_machine_local_id)?
            .input(input_id)?
            .bool_value()
    }

    pub(crate) fn set_nested_bool_value(&mut self, local_id: usize, value: bool) -> bool {
        let Some((state_machine_local_id, input_id)) = self.nested_input_target(local_id) else {
            return false;
        };
        self.set_nested_state_machine_bool(state_machine_local_id, input_id, value)
    }

    pub(crate) fn apply_listener_nested_bool_change(
        &mut self,
        local_id: usize,
        authored_value: u64,
    ) -> bool {
        let current = self.nested_bool_value(local_id).unwrap_or(false);
        let value = match authored_value {
            0 => false,
            1 => true,
            _ => !current,
        };
        self.set_nested_bool_value(local_id, value)
    }
}
