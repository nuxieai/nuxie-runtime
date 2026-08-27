use crate::mechanical_port::source::{
    core_context::CoreContext,
    generated::animation::{
        state_machine_base::StateMachineBase, state_machine_input_base::StateMachineInputBase,
    },
    importers::{import_stack::ImportStack, state_machine_importer::StateMachineImporter},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct StateMachineInput {
    pub base: StateMachineInputBase,
}

impl StateMachineInput {
    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn import(self: Box<Self>, import_stack: &mut ImportStack) -> StatusCode {
        let object = Box::into_raw(self);
        let Some(importer) =
            import_stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(object)) };
            return StatusCode::MissingObject;
        };
        importer.add_input(Some(unsafe { Box::from_raw(object) }));
        unsafe { (*object).base.base.import(import_stack) }
    }
}
