use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::{
        any_state_base::AnyStateBase, entry_state_base::EntryStateBase,
        exit_state_base::ExitStateBase, state_machine_base::StateMachineBase,
        state_machine_layer_base::StateMachineLayerBase,
    },
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    status_code::StatusCode,
};
#[derive(Default)]
pub struct StateMachineLayer {
    pub base: StateMachineLayerBase,
    states: Vec<CoreHandle>,
    any: Option<CoreHandle>,
    entry: Option<CoreHandle>,
    exit: Option<CoreHandle>,
}
impl StateMachineLayer {
    pub fn name(&self) -> &str {
        self.base.base.base.name()
    }

    pub(crate) fn add_state(&mut self, state: CoreHandle) {
        self.states.push(state);
    }
    pub fn any_state(&self) -> Option<CoreHandle> {
        self.any.clone()
    }
    pub fn entry_state(&self) -> Option<CoreHandle> {
        self.entry.clone()
    }
    pub fn exit_state(&self) -> Option<CoreHandle> {
        self.exit.clone()
    }
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
    pub fn state(&self, index: usize) -> Option<CoreHandle> {
        self.states.get(index).cloned()
    }
    pub fn states(&self) -> &[CoreHandle] {
        &self.states
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for state in self.states.iter().cloned() {
            let code = state
                .with_mut(|state| state.layer_state_on_added_dirty(context))
                .flatten()
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
            match state.core_type() {
                Some(AnyStateBase::TYPE_KEY) => self.any = Some(state),
                Some(EntryStateBase::TYPE_KEY) => self.entry = Some(state),
                Some(ExitStateBase::TYPE_KEY) => self.exit = Some(state),
                _ => {}
            }
        }
        if self.any.is_none() || self.entry.is_none() || self.exit.is_none() {
            StatusCode::InvalidObject
        } else {
            StatusCode::Ok
        }
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for state in self.states.iter().cloned() {
            let code = state
                .with_mut(|state| state.layer_state_on_added_clean(context))
                .flatten()
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_layer(this);
        self.base.base.import(stack)
    }
}
