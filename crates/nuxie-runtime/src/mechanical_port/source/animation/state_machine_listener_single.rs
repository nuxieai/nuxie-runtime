use crate::mechanical_port::source::{
    data_bind_path_referencer::DataBindPathReferencer,
    generated::animation::state_machine_listener_single_base::StateMachineListenerSingleBase,
    importers::import_stack::ImportStack, listener_type::ListenerType, status_code::StatusCode,
};

#[derive(Default)]
pub struct StateMachineListenerSingle {
    pub base: StateMachineListenerSingleBase,
    pub data_bind_path_referencer: DataBindPathReferencer,
}

impl StateMachineListenerSingle {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        self.data_bind_path_referencer
            .import_data_bind_path(import_stack);
        self.base.base.import(import_stack)
    }

    pub fn has_listener(&self, listener_type: ListenerType) -> bool {
        self.base.listener_type_value() == listener_type as u32
    }

    pub fn decode_view_model_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn copy_view_model_path_ids(&mut self, object: &Self) {
        self.data_bind_path_referencer
            .copy_data_bind_path(object.data_bind_path_referencer.data_bind_path());
    }

    pub fn view_model_path_ids_buffer(&self) -> Vec<u32> {
        let path = self
            .data_bind_path_referencer
            .data_bind_path()
            .expect("state machine listener data-bind path must exist");
        let mut path = path.clone();
        path.path().clone()
    }
}
