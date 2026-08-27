use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property::ViewModelProperty,
};

pub trait ViewModelPropertyViewModelBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn view_model_reference_id_changed(&mut self) {}
}

pub struct ViewModelPropertyViewModelBase {
    pub base: ViewModelProperty,
    view_model_reference_id: u32,
}

impl Default for ViewModelPropertyViewModelBase {
    fn default() -> Self {
        Self {
            base: ViewModelProperty::default(),
            view_model_reference_id: 0,
        }
    }
}

impl ViewModelPropertyViewModelBase {
    pub const TYPE_KEY: u16 = 436;
    pub const VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY: u16 = 565;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 0)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn view_model_reference_id(&self) -> u32 {
        self.view_model_reference_id
    }
    pub fn set_view_model_reference_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) {
        if self.view_model_reference_id == value {
            return;
        }
        self.view_model_reference_id = value;
        callbacks.view_model_reference_id_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY);
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) {
        self.view_model_reference_id = object.view_model_reference_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelPropertyViewModelBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY => {
                self.view_model_reference_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
