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
    pub fn name(&self) -> &str {
        self.base.base.base.name()
    }

    pub fn on_added_dirty(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        StatusCode::Ok
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) =
            import_stack.latest::<StateMachineImporter>(StateMachineBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_input(Some(this));
        self.base.base.import(import_stack)
    }
}
impl std::ops::Deref for StateMachineInput {
    type Target = StateMachineInputBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachineInput {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
