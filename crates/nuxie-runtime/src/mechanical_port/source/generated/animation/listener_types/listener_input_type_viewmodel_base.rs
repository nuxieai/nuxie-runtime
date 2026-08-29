use crate::mechanical_port::source::animation::listener_types::listener_input_type_viewmodel::ListenerInputTypeViewModel;

use crate::mechanical_port::source::{
    animation::listener_types::listener_input_type::ListenerInputType,
    core::binary_reader::BinaryReader,
};

pub trait ListenerInputTypeViewModelBaseCallbacks: crate::mechanical_port::source::generated::animation::listener_types::listener_input_type_base::ListenerInputTypeBaseCallbacks {
    fn view_model_path_ids_changed(&mut self) {}
    fn decode_view_model_path_ids(&mut self, value: &[u8]);
    fn copy_view_model_path_ids(&mut self, object: &ListenerInputTypeViewModel);
}

pub struct ListenerInputTypeViewModelBase {
    pub base: ListenerInputType,
}

impl Default for ListenerInputTypeViewModelBase {
    fn default() -> Self {
        Self {
            base: ListenerInputType::default(),
        }
    }
}

impl ListenerInputTypeViewModelBase {
    pub const TYPE_KEY: u16 = 660;
    pub const VIEW_MODEL_PATH_IDS_PROPERTY_KEY: u16 = 963;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(source: &ListenerInputTypeViewModel) -> ListenerInputTypeViewModel {
        let mut cloned = ListenerInputTypeViewModel::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(source, &mut cloned);
        cloned.base = base;
        cloned
    }
    pub fn copy(
        &mut self,
        object: &ListenerInputTypeViewModel,
        callbacks: &mut impl ListenerInputTypeViewModelBaseCallbacks,
    ) {
        callbacks.copy_view_model_path_ids(object);
        self.base.copy(&object.base.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerInputTypeViewModelBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_PATH_IDS_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_view_model_path_ids(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ListenerInputTypeViewModelBase {
    type Target = ListenerInputType;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputTypeViewModelBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
