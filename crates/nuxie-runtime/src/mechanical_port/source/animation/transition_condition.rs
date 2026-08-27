use crate::mechanical_port::source::{
    core_context::CoreContext,
    generated::animation::{
        state_transition_base::StateTransitionBase,
        transition_condition_base::TransitionConditionBase,
    },
    importers::{import_stack::ImportStack, state_transition_importer::StateTransitionImporter},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct TransitionCondition {
    pub base: TransitionConditionBase,
}
impl TransitionCondition {
    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<StateTransitionImporter>(StateTransitionBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.add_condition(NonNull::from(&mut *self));
        self.base.base.import(stack)
    }
    pub fn evaluate(&self, _machine: *const (), _layer: *mut ()) -> bool {
        true
    }
    pub fn use_in_layer(&self, _machine: *const (), _layer: *mut ()) {}
    pub fn validate_input_type(&self, _input: *const ()) -> bool {
        true
    }
}
use std::ptr::NonNull;
