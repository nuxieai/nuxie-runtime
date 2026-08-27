// Mirrors src/animation/blend_state_1d_viewmodel.cpp and
// include/rive/animation/blend_state_1d_viewmodel.hpp.
pub(crate) struct RuntimeBlendState1DViewModel;

impl RuntimeBlendState1DViewModel {
    pub(crate) fn drop_source(source: &mut crate::state_machine::RuntimeBlendState1DSource) {
        if let crate::state_machine::RuntimeBlendState1DSource::BindableProperty { global_id } =
            source
        {
            if global_id.is_some() {
                *global_id = None;
            }
        }
    }

    pub(crate) fn from_imported(file: &RuntimeFile, object: &RuntimeObject) -> Option<u32> {
        file.latest_bindable_property_for_object(object)
            .map(|property| property.id as u32)
    }

    pub(crate) fn bindable_property(global_id: Option<u32>) -> Option<u32> {
        global_id
    }

    pub(crate) fn value(
        global_id: Option<u32>,
        bindable_numbers: &[crate::state_machine::StateMachineBindableNumberInstance],
    ) -> f32 {
        Self::bindable_property(global_id)
            .and_then(|global_id| {
                crate::state_machine::bindable_number_value(bindable_numbers, global_id)
            })
            .unwrap_or(0.0)
    }
}
