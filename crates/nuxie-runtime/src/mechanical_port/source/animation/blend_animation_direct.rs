use crate::mechanical_port::source::{
    animation::{
        blend_state_direct_instance::BlendAnimationDirectDefinition,
        blend_state_instance::BlendAnimationDefinition, state_machine::StateMachine,
        state_machine_number::StateMachineNumber,
    },
    core::CoreHandle,
    core_context::CoreContext,
    generated::{
        animation::blend_animation_direct_base::BlendAnimationDirectBase,
        data_bind::bindable_property_base::BindablePropertyBase,
    },
    importers::{
        bindable_property_importer::BindablePropertyImporter, import_stack::ImportStack,
        state_machine_importer::StateMachineImporter,
    },
    status_code::StatusCode,
};

#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectBlendSource {
    InputId = 0,
    MixValue = 1,
    DataBindId = 2,
}

pub struct BlendAnimationDirect {
    pub base: BlendAnimationDirectBase,
    bindable_property: Option<CoreHandle>,
}

impl Default for BlendAnimationDirect {
    fn default() -> Self {
        Self {
            base: BlendAnimationDirectBase::default(),
            bindable_property: None,
        }
    }
}

impl BlendAnimationDirect {
    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(state_machine_importer) =
            import_stack.latest::<StateMachineImporter>(StateMachine::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };

        match self.base.blend_source() {
            value if value == DirectBlendSource::InputId as u32 => {
                let state_machine = state_machine_importer.state_machine();
                let input_id = self.base.input_id() as usize;
                let valid = state_machine
                    .with_downcast::<StateMachine, _>(|state_machine| {
                        if input_id >= state_machine.input_count() {
                            return false;
                        }
                        state_machine
                            .input(input_id)
                            .is_some_and(|input| input.is_type_of(StateMachineNumber::TYPE_KEY))
                    })
                    .unwrap_or(false);
                if !valid {
                    return StatusCode::InvalidObject;
                }
            }
            value if value == DirectBlendSource::DataBindId as u32 => {
                let Some(bindable_importer) =
                    import_stack.latest::<BindablePropertyImporter>(BindablePropertyBase::TYPE_KEY)
                else {
                    return StatusCode::MissingObject;
                };
                self.bindable_property = bindable_importer.bindable_property();
            }
            _ => {}
        }

        self.base.base.import(import_stack)
    }

    pub fn set_bindable_property(&mut self, value: Option<CoreHandle>) {
        self.bindable_property = value;
    }

    pub fn bindable_property(&self) -> Option<CoreHandle> {
        self.bindable_property.clone()
    }
}

impl BlendAnimationDefinition for BlendAnimationDirect {
    fn animation(&self) -> Option<CoreHandle> {
        self.base.base.animation()
    }
}

impl BlendAnimationDirectDefinition for BlendAnimationDirect {
    fn blend_source(&self) -> u32 {
        self.base.blend_source()
    }

    fn mix_value(&self) -> f32 {
        self.base.mix_value()
    }

    fn input_id(&self) -> u32 {
        self.base.input_id()
    }

    fn bindable_property(&self) -> Option<CoreHandle> {
        self.bindable_property.clone()
    }
}
