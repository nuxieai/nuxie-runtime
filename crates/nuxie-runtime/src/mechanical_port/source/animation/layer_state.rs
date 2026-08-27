use crate::mechanical_port::source::{
    animation::{
        state_instance::StateInstance, state_transition::StateTransition,
        system_state_instance::SystemStateInstance,
    },
    core_context::CoreContext,
    generated::animation::{
        layer_state_base::LayerStateBase, state_machine_layer_base::StateMachineLayerBase,
    },
    importers::{
        import_stack::ImportStack, state_machine_layer_importer::StateMachineLayerImporter,
    },
    status_code::StatusCode,
};
use std::ptr::NonNull;
#[derive(Default)]
pub struct LayerState {
    pub base: LayerStateBase,
    transitions: Vec<Box<StateTransition>>,
}
impl LayerState {
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
    pub fn transition(&self, index: usize) -> Option<&StateTransition> {
        self.transitions.get(index).map(Box::as_ref)
    }
    pub(crate) fn add_transition(&mut self, transition: Box<StateTransition>) {
        self.transitions.push(transition);
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for transition in &mut self.transitions {
            let code = transition.on_added_dirty(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for transition in &mut self.transitions {
            let code = transition.on_added_clean(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) =
            stack.latest::<StateMachineLayerImporter>(StateMachineLayerBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        importer.add_state(NonNull::from(&mut *self));
        self.base.base.import(stack)
    }
    pub fn make_instance(&self, artboard: *mut ()) -> Box<SystemStateInstance> {
        Box::new(SystemStateInstance::new(self, artboard))
    }
}
