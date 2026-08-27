// Mirrors src/animation/blend_state_1d_input.cpp and
// include/rive/animation/blend_state_1d_input.hpp.
pub(crate) struct RuntimeBlendState1DInput;

impl RuntimeBlendState1DInput {
    pub(crate) fn has_valid_input_id(input_index: Option<usize>) -> bool {
        input_index.is_some()
    }

    pub(crate) fn from_imported(object: &RuntimeObject) -> Option<usize> {
        let input_id = u32::try_from(object.uint_property("inputId")?).ok()?;
        (input_id != u32::MAX).then_some(input_id as usize)
    }

    pub(crate) fn input_index(input_index: Option<usize>) -> Option<usize> {
        Self::has_valid_input_id(input_index)
            .then_some(input_index)
            .flatten()
    }
}
