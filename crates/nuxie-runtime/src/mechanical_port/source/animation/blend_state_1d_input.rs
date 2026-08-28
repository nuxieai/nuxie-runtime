use crate::mechanical_port::source::{
    animation::{
        animation_reset::AnimationResetTarget,
        animation_reset_factory::ResetArtboard,
        blend_animation_1d::BlendAnimation1D,
        blend_state_1d_instance::{
            BlendState1DDefinition, BlendState1DInstance, BlendState1DValueSource,
        },
        blend_state_instance::BlendStateDefinition,
    },
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::{Core, CoreHandle},
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
            let valid = state_machine
                .with_downcast::<crate::mechanical_port::source::animation::state_machine::StateMachine, _>(|state_machine| {
                    input_id < state_machine.input_count()
                        && state_machine.input(input_id).is_some_and(|input| input.is_type_of(StateMachineNumberBase::TYPE_KEY))
                })
                .unwrap_or(false);
            if !valid {
                return StatusCode::InvalidObject;
            }
        }
        self.base.base.import(import_stack)
    }

    pub fn make_instance<R>(
        &self,
        instance: &mut R,
        artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> BlendState1DInstance<Self, BlendAnimation1D>
    where
        R: ResetArtboard + AnimationResetTarget,
    {
        let state = self
            .base
            .base
            .base
            .base
            .base
            .base
            .handle()
            .expect("an imported BlendState1DInput has arena identity before instancing");
        BlendState1DInstance::new(state, instance, artboard)
    }
}

impl BlendStateDefinition<BlendAnimation1D> for BlendState1DInput {
    fn animations(&self) -> Vec<CoreHandle> {
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
