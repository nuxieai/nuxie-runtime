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
    pub fn listener_type_value(&self) -> u32 {
        self.base.listener_type_value()
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) =
            import_stack.latest::<StateMachineListenerImporter>(StateMachineListenerBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_listener_input_type(this);
        self.base.base.import(import_stack)
    }
}
impl std::ops::Deref for ListenerInputType {
    type Target = ListenerInputTypeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerInputType {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_base::ListenerInputTypeBaseCallbacks for ListenerInputType { fn notify_property_changed(&mut self, key: u16) { self.base.notify_property_changed(key); } }
