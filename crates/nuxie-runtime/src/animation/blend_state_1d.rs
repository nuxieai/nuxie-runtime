// Mirrors src/animation/blend_state_1d.cpp.
impl crate::state_machine::RuntimeBlendState1D {
    pub(crate) fn make_instance(
        &self,
        artboard: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
        reset_blend_values: bool,
    ) -> crate::state_machine::BlendState1DInstance {
        crate::state_machine::BlendState1DInstance::new(
            self,
            artboard,
            animation_definitions,
            empty_animation_definition,
            reset_blend_values,
        )
    }
}
