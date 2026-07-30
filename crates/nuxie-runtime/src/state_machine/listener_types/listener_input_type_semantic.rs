use super::semantic_input::RuntimeSemanticInput;
use nuxie_binary::RuntimeObject;

/// Authored ListenerInputTypeSemantic definition shared by occurrences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeListenerInputTypeSemantic {
    pub(crate) global_id: u32,
    semantic_inputs: Vec<RuntimeSemanticInput>,
}

impl RuntimeListenerInputTypeSemantic {
    pub(in crate::state_machine) fn from_imported(
        input_type: &RuntimeObject,
        inputs: &[&RuntimeObject],
    ) -> Self {
        Self {
            global_id: input_type.id,
            semantic_inputs: inputs
                .iter()
                .filter(|input| input.type_name == "SemanticInput")
                .map(|input| RuntimeSemanticInput::from_imported(input))
                .collect(),
        }
    }

    pub(crate) fn semantic_input_count(&self) -> usize {
        self.semantic_inputs.len()
    }

    pub(crate) fn semantic_input(&self, index: usize) -> Option<&RuntimeSemanticInput> {
        self.semantic_inputs.get(index)
    }

    pub(crate) fn constraints_met(input_types: &[Self], action_type: u32) -> bool {
        for input_type in input_types {
            if input_type.semantic_inputs.is_empty() {
                return true;
            }
            if input_type
                .semantic_inputs
                .iter()
                .any(|input| input.action_type == action_type)
            {
                return true;
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn semantic_input(global_id: u32, action_type: u32) -> RuntimeSemanticInput {
        RuntimeSemanticInput {
            global_id,
            action_type,
        }
    }

    #[test]
    fn semantic_constraints_match_exact_action_and_authored_order() {
        let first = RuntimeListenerInputTypeSemantic {
            global_id: 1,
            semantic_inputs: vec![semantic_input(2, 0)],
        };
        let second = RuntimeListenerInputTypeSemantic {
            global_id: 3,
            semantic_inputs: vec![semantic_input(4, 2)],
        };
        assert!(RuntimeListenerInputTypeSemantic::constraints_met(
            &[first.clone(), second],
            2,
        ));
        assert!(!RuntimeListenerInputTypeSemantic::constraints_met(
            &[first],
            1,
        ));
    }

    #[test]
    fn empty_semantic_type_is_catch_all_but_no_typed_owner_is_not() {
        let catch_all = RuntimeListenerInputTypeSemantic {
            global_id: 1,
            semantic_inputs: Vec::new(),
        };
        assert!(RuntimeListenerInputTypeSemantic::constraints_met(
            &[catch_all],
            99,
        ));
        assert!(!RuntimeListenerInputTypeSemantic::constraints_met(&[], 0));
    }
}
