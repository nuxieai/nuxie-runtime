use crate::mechanical_port::source::{
    data_bind_path_referencer::DataBindPathReferencer,
    generated::animation::listener_types::listener_input_type_viewmodel_base::ListenerInputTypeViewModelBase,
};

#[derive(Default)]
pub struct ListenerInputTypeViewModel {
    pub base: ListenerInputTypeViewModelBase,
    pub data_bind_path_referencer: DataBindPathReferencer,
}

impl ListenerInputTypeViewModel {
    pub fn decode_view_model_path_ids(&mut self, value: &[u8]) {
        self.data_bind_path_referencer.decode_data_bind_path(value);
    }

    pub fn copy_view_model_path_ids(&mut self, object: &Self) {
        self.data_bind_path_referencer
            .copy_data_bind_path(object.data_bind_path_referencer.data_bind_path());
    }

    pub fn view_model_path_ids_buffer(&self) -> Vec<u32> {
        let Some(path) = self.data_bind_path_referencer.data_bind_path() else {
            return Vec::new();
        };
        let mut path = path.clone();
        path.path().clone()
    }
}
