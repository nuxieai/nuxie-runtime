use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_number_to_list::DataConverterNumberToList,
};

pub trait DataConverterNumberToListBaseCallbacks: crate::mechanical_port::source::generated::data_bind::converters::data_converter_base::DataConverterBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn view_model_id_changed(&mut self) {}
}

pub struct DataConverterNumberToListBase {
    pub base: DataConverter,
    view_model_id: u32,
}

impl Default for DataConverterNumberToListBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            view_model_id: u32::MAX,
        }
    }
}

impl DataConverterNumberToListBase {
    pub const TYPE_KEY: u16 = 568;
    pub const VIEW_MODEL_ID_PROPERTY_KEY: u16 = 816;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
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
        callbacks: &mut impl DataConverterNumberToListBaseCallbacks,
    ) {
        if !self.set_view_model_id_value(value) {
            return;
        }
        callbacks.view_model_id_changed();
        callbacks.notify_property_changed(Self::VIEW_MODEL_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_view_model_id_value(&mut self, value: u32) -> bool {
        if self.view_model_id == value {
            return false;
        }
        self.view_model_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterNumberToListBaseCallbacks,
    ) -> DataConverterNumberToList {
        let mut cloned = DataConverterNumberToList::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterNumberToListBaseCallbacks,
    ) {
        self.view_model_id = object.view_model_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterNumberToListBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::VIEW_MODEL_ID_PROPERTY_KEY => {
                self.view_model_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DataConverterNumberToListBase {
    type Target = DataConverter;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterNumberToListBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
