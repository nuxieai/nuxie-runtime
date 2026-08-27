use crate::mechanical_port::source::viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem;

use crate::mechanical_port::source::{core::Core, core::binary_reader::BinaryReader};

pub trait ViewModelInstanceListItemBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn view_model_id_changed(&mut self) {}
    fn view_model_instance_id_changed(&mut self) {}
}

pub struct ViewModelInstanceListItemBase {
    pub base: Core,
    view_model_id: u32,
    view_model_instance_id: u32,
}

impl Default for ViewModelInstanceListItemBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            view_model_id: u32::MAX,
            view_model_instance_id: u32::MAX,
        }
    }
}

impl ViewModelInstanceListItemBase {
    pub const TYPE_KEY: u16 = 427;
    pub const VIEW_MODEL_ID_PROPERTY_KEY: u16 = 549;
    pub const VIEW_MODEL_INSTANCE_ID_PROPERTY_KEY: u16 = 550;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn view_model_id(&self) -> u32 {
        self.view_model_id
    }
    pub fn set_view_model_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelInstanceListItemBaseCallbacks,
    ) {
        if self.view_model_id == value {
            return;
        }
        self.view_model_id = value;
        callbacks.view_model_id_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_ID_PROPERTY_KEY);
    }
    pub fn view_model_instance_id(&self) -> u32 {
        self.view_model_instance_id
    }
    pub fn set_view_model_instance_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelInstanceListItemBaseCallbacks,
    ) {
        if self.view_model_instance_id == value {
            return;
        }
        self.view_model_instance_id = value;
        callbacks.view_model_instance_id_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_INSTANCE_ID_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelInstanceListItemBaseCallbacks,
    ) -> ViewModelInstanceListItem {
        let mut cloned = ViewModelInstanceListItem::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelInstanceListItemBaseCallbacks,
    ) {
        self.view_model_id = object.view_model_id;
        self.view_model_instance_id = object.view_model_instance_id;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceListItemBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_ID_PROPERTY_KEY => {
                self.view_model_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VIEW_MODEL_INSTANCE_ID_PROPERTY_KEY => {
                self.view_model_instance_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}
