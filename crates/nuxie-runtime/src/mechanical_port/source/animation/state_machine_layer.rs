use crate::mechanical_port::source::{
    animation::layer_state::LayerState,
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
    states: Vec<Box<LayerState>>,
    any: Option<*const LayerState>,
    entry: Option<*const LayerState>,
    exit: Option<*const LayerState>,
}
impl StateMachineLayer {
    pub(crate) fn add_state(&mut self, state: Box<LayerState>) {
        self.states.push(state);
    }
    pub fn any_state(&self) -> Option<&LayerState> {
        self.any.map(|v| unsafe { &*v })
    }
    pub fn entry_state(&self) -> Option<&LayerState> {
        self.entry.map(|v| unsafe { &*v })
    }
    pub fn exit_state(&self) -> Option<&LayerState> {
        self.exit.map(|v| unsafe { &*v })
    }
    pub fn state_count(&self) -> usize {
        self.states.len()
    }
    pub fn state(&self, index: usize) -> Option<&LayerState> {
        self.states.get(index).map(Box::as_ref)
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for state in &mut self.states {
            let code = state.on_added_dirty(context);
            if code != StatusCode::Ok {
                return code;
            }
            let pointer = state.as_ref() as *const LayerState;
            match state.base.core_type() {
                AnyStateBase::TYPE_KEY => self.any = Some(pointer),
                EntryStateBase::TYPE_KEY => self.entry = Some(pointer),
                ExitStateBase::TYPE_KEY => self.exit = Some(pointer),
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
        for state in &mut self.states {
            let code = state.on_added_clean(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn import(self: Box<Self>, stack: &mut ImportStack) -> StatusCode {
        let raw = Box::into_raw(self);
        let Some(importer) = stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(raw)) };
            return StatusCode::MissingObject;
        };
        importer.add_layer(unsafe { Box::from_raw(raw) });
        unsafe { (*raw).base.base.import(stack) }
    }
}
