use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationResetTarget,
        animation_reset_factory::{ResetArtboard, ResetLinearAnimation},
        blend_animation_1d::BlendAnimation1D,
        blend_state_1d_instance::{
            BlendState1DDefinition, BlendState1DInstance, BlendState1DValueSource,
        },
        blend_state_instance::BlendStateDefinition,
    },
    core::Core,
    generated::animation::{
        blend_state_1d_input_base::BlendState1DInputBase, state_machine_base::StateMachineBase,
        state_machine_number_base::StateMachineNumberBase,
    },
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct BlendState1DInput {
    pub base: BlendState1DInputBase,
}

impl BlendState1DInput {
    pub fn has_valid_input_id(&self) -> bool {
        self.base.input_id() != Core::EMPTY_ID
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(state_machine_importer) =
            import_stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };

        if self.has_valid_input_id() {
            let state_machine = state_machine_importer.state_machine();
            let input_id = self.base.input_id() as usize;
            unsafe {
                if input_id >= state_machine.as_ref().input_count() {
                    return StatusCode::InvalidObject;
                }
                let Some(input) = state_machine.as_ref().input(input_id) else {
                    return StatusCode::InvalidObject;
                };
                if input.as_ref().core_type() != StateMachineNumberBase::TYPE_KEY {
                    return StatusCode::InvalidObject;
                }
            }
        }
        self.base.base.import(import_stack)
    }

    pub fn make_instance<R>(
        &self,
        instance: &mut R,
    ) -> BlendState1DInstance<'_, Self, BlendAnimation1D>
    where
        R: ResetArtboard + AnimationResetTarget,
        <BlendAnimation1D as crate::mechanical_port::source::animation::blend_state_instance::BlendAnimationDefinition>::Animation: ResetLinearAnimation,
    {
        BlendState1DInstance::new(self, instance)
    }
}

impl BlendStateDefinition<BlendAnimation1D> for BlendState1DInput {
    fn animations(&self) -> Vec<&BlendAnimation1D> {
        <crate::mechanical_port::source::animation::blend_state_1d::BlendState1D as BlendStateDefinition<BlendAnimation1D>>::animations(&self.base.base)
    }

    fn flags(&self) -> u8 {
        <crate::mechanical_port::source::animation::blend_state_1d::BlendState1D as BlendStateDefinition<BlendAnimation1D>>::flags(&self.base.base)
    }
}

impl BlendState1DDefinition<BlendAnimation1D> for BlendState1DInput {
    fn value_source(&self) -> BlendState1DValueSource {
        BlendState1DValueSource::Input(self.base.input_id())
    }
}
