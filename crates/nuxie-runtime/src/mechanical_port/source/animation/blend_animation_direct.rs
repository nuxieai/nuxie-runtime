use std::ptr::NonNull;

use crate::mechanical_port::source::{
    animation::{
        blend_state_direct_instance::BlendAnimationDirectDefinition,
        blend_state_instance::BlendAnimationDefinition, linear_animation::LinearAnimation,
        state_machine::StateMachine, state_machine_number::StateMachineNumber,
    },
    core_context::CoreContext,
    data_bind::bindable_property::BindableProperty,
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
    bindable_property: Option<NonNull<BindableProperty>>,
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
    pub fn on_added_dirty(&mut self, _context: &mut CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut CoreContext) -> StatusCode {
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
                unsafe {
                    if input_id >= state_machine.as_ref().input_count() {
                        return StatusCode::InvalidObject;
                    }
                    let Some(input) = state_machine.as_ref().input(input_id) else {
                        return StatusCode::InvalidObject;
                    };
                    if input.as_ref().core_type() != StateMachineNumber::TYPE_KEY {
                        return StatusCode::InvalidObject;
                    }
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

    pub fn set_bindable_property(&mut self, value: Option<NonNull<BindableProperty>>) {
        self.bindable_property = value;
    }

    pub fn bindable_property(&self) -> Option<NonNull<BindableProperty>> {
        self.bindable_property
    }
}

impl Drop for BlendAnimationDirect {
    fn drop(&mut self) {
        if let Some(property) = self.bindable_property.take() {
            unsafe { drop(Box::from_raw(property.as_ptr())) };
        }
    }
}

impl BlendAnimationDefinition for BlendAnimationDirect {
    type Animation = LinearAnimation;

    fn animation(&self) -> &Self::Animation {
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

    fn bindable_property(&self) -> Option<NonNull<BindableProperty>> {
        self.bindable_property
    }
}
