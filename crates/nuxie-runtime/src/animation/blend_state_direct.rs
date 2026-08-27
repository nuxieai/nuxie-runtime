// Mirrors src/animation/blend_state_direct.cpp.
impl crate::state_machine::RuntimeBlendStateDirect {
    pub(crate) fn make_instance(
        &self,
        _instance: &ArtboardInstance,
        animation_definitions: &Arc<Vec<RuntimeLinearAnimation>>,
        empty_animation_definition: &Arc<RuntimeLinearAnimation>,
    ) -> crate::state_machine::BlendStateDirectInstance {
        crate::state_machine::BlendStateDirectInstance::new(
            self,
            animation_definitions,
            empty_animation_definition,
        )
    }
}
