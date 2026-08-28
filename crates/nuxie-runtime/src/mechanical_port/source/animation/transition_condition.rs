use crate::mechanical_port::source::{
    animation::state_machine_instance::{
        RuntimeStateMachineLayerInstanceWeakHandle, StateMachineInstance,
    },
    core::CoreHandle,
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
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_condition(this);
        self.base.base.import(stack)
    }
    pub fn evaluate(
        &self,
        _machine: &StateMachineInstance,
        _layer: &RuntimeStateMachineLayerInstanceWeakHandle,
    ) -> bool {
        true
    }
    pub fn use_in_layer(
        &self,
        _machine: &mut StateMachineInstance,
        _layer: RuntimeStateMachineLayerInstanceWeakHandle,
    ) {
    }
    pub fn validate_input_type(&self, _input: Option<&CoreHandle>) -> bool {
        true
    }
}
impl std::ops::Deref for TransitionCondition {
    type Target = TransitionConditionBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TransitionCondition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
