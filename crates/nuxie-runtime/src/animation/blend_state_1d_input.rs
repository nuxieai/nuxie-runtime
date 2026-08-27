// Mirrors src/animation/blend_state_1d_input.cpp and
// include/rive/animation/blend_state_1d_input.hpp.
pub(crate) struct RuntimeBlendState1DInput;

impl RuntimeBlendState1DInput {
    pub(crate) fn has_valid_input_id(input_index: Option<usize>) -> bool {
        input_index.is_some()
    }

    pub(crate) fn from_imported(object: &RuntimeObject) -> Option<usize> {
        object
            .uint_property("inputId")
            .filter(|input_id| *input_id != u64::from(u32::MAX))
            .and_then(|input_id| usize::try_from(input_id).ok())
    }

    pub(crate) fn input_index(input_index: Option<usize>) -> Option<usize> {
        Self::has_valid_input_id(input_index)
            .then_some(input_index)
            .flatten()
    }
}
