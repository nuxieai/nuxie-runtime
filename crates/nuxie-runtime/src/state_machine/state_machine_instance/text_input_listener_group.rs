// State-machine instance integration for the C++ `text_input_listener_group.cpp` source.
use super::*;

impl HitDrawable {
    pub(super) fn add_text_input_listener(&mut self, group_index: usize) {
        self.can_early_out = false;
        self.needs_down_listener = true;
        self.needs_up_listener = true;
        self.listeners.push(group_index);
    }
}

impl StateMachineInstance {
    /// Dispatch owned committed text to the currently focused listener groups.
    pub fn text_input(&mut self, artboard: &mut ArtboardInstance, text: &str) -> bool {
        if self.script_error.is_some() {
            return false;
        }
        if self.scripted_data_context_rebind_pending() {
            return false;
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return false;
        }
        if !self.focus.is_inert() {
            self.focus.drop_hidden_focus_target();
        }
        let owner_identity = self.focus.owner_identity();
        for (owner, _, focus_data_local_id) in self.focus.focused_listener_chain() {
            let handled = if owner == owner_identity {
                self.text_input_at_focus_data(artboard, focus_data_local_id, text)
            } else {
                artboard.dispatch_nested_text_input_at_focus(owner, focus_data_local_id, text)
            };
            if handled.terminal_resource_failure {
                return false;
            }
            if handled.handled {
                return true;
            }
        }
        false
    }
    pub(crate) fn text_input_at_focus_data(
        &mut self,
        artboard: &mut ArtboardInstance,
        focus_data_local_id: usize,
        text: &str,
    ) -> RuntimeInputDispatchOutcome {
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        if self.scripted_data_context_rebind_pending() {
            return RuntimeInputDispatchOutcome::default();
        }
        self.ensure_scripted_input_groups_current(artboard);
        if self.script_error.is_some() {
            return RuntimeInputDispatchOutcome::terminal();
        }
        let groups = self
            .keyboard_listener_groups
            .iter()
            .filter(|group| group.focus_data_local_id == focus_data_local_id)
            .cloned()
            .collect::<Vec<_>>();
        for group in groups {
            let outcome = group.text_input(self, artboard, text);
            if outcome.terminal_resource_failure || outcome.handled {
                return outcome;
            }
        }
        RuntimeInputDispatchOutcome::default()
    }
    pub(super) fn sync_text_input_focus(&self, artboard: &mut ArtboardInstance) -> bool {
        let artboard_identity = artboard.instance_identity();
        let focused_local_id = self.focus.focused_listener_chain().into_iter().find_map(
            |(owner_identity, target_local_id, _)| {
                (owner_identity == artboard_identity
                    && artboard.runtime_object_type_name(target_local_id) == Some("TextInput"))
                .then_some(target_local_id)
            },
        );
        artboard.sync_text_input_focus(focused_local_id)
    }
}
