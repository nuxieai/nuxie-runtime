use crate::mechanical_port::source::{
    generated::animation::{
        listener_types::listener_input_type_base::ListenerInputTypeBase,
        state_machine_listener_base::StateMachineListenerBase,
    },
    importers::{
        import_stack::ImportStack, state_machine_listener_importer::StateMachineListenerImporter,
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct ListenerInputType {
    pub base: ListenerInputTypeBase,
}

impl ListenerInputType {
    pub fn import(self: Box<Self>, import_stack: &mut ImportStack) -> StatusCode {
        let object = Box::into_raw(self);
        let Some(importer) =
            import_stack.latest::<StateMachineListenerImporter>(StateMachineListenerBase::TYPE_KEY)
        else {
            unsafe { drop(Box::from_raw(object)) };
            return StatusCode::MissingObject;
        };
        importer.add_listener_input_type(unsafe { Box::from_raw(object) });
        unsafe { (*object).base.base.import(import_stack) }
    }
}
