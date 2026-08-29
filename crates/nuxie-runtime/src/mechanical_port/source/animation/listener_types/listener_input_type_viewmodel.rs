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
            .copy_data_bind_path(&object.data_bind_path_referencer);
    }

    pub fn view_model_path_ids_buffer(&self) -> Vec<u32> {
        self.data_bind_path_referencer
            .with_data_bind_path(|path| path.path().to_vec())
            .expect("listener input type data-bind path must exist")
    }
}

impl std::ops::Deref for ListenerInputTypeViewModel {
    type Target = ListenerInputTypeViewModelBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerInputTypeViewModel {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
