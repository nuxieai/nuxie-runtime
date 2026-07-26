use nuxie_binary::RuntimeObject;

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionInputCondition {
    input_index: usize,
}

impl RuntimeTransitionInputCondition {
    pub(super) fn from_object(
        state_machine_inputs: &[Option<&RuntimeObject>],
        expected_type_name: &str,
        object: &RuntimeObject,
    ) -> Option<Self> {
        let condition = Self {
            input_index: usize::try_from(object.uint_property("inputId")?).ok()?,
        };
        // Pinned C++ validates bounds and the concrete StateMachineInput
        // subclass while importing the condition, before occurrences exist
        // (`transition_input_condition.cpp:10-30`).
        match state_machine_inputs.get(condition.input_index) {
            Some(Some(input)) if input.type_name == expected_type_name => Some(condition),
            // Pinned C++ deliberately accepts a null input slot so an older
            // runtime can limp through a newer input type; all three direct
            // conditions then evaluate it as true
            // (`transition_{bool,number,trigger}_condition.cpp:10-20`).
            Some(None) => Some(condition),
            _ => None,
        }
    }

    #[cfg(test)]
    pub(super) fn new(input_index: usize) -> Self {
        Self { input_index }
    }

    pub(super) fn input_index(self) -> usize {
        self.input_index
    }
}
