use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, core::Core,
    data_bind::converters::data_converter_group_item::DataConverterGroupItem,
};

pub trait DataConverterGroupItemBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn converter_id_changed(&mut self) {}
}

pub struct DataConverterGroupItemBase {
    pub base: Core,
    converter_id: u32,
}

impl Default for DataConverterGroupItemBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            converter_id: u32::MAX,
        }
    }
}

impl DataConverterGroupItemBase {
    pub const TYPE_KEY: u16 = 498;
    pub const CONVERTER_ID_PROPERTY_KEY: u16 = 679;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn converter_id(&self) -> u32 {
        self.converter_id
    }
    pub fn set_converter_id(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterGroupItemBaseCallbacks,
    ) {
        if !self.set_converter_id_value(value) {
            return;
        }
        callbacks.converter_id_changed();
        callbacks.notify_property_changed(Self::CONVERTER_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_converter_id_value(&mut self, value: u32) -> bool {
        if self.converter_id == value {
            return false;
        }
        self.converter_id = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterGroupItemBaseCallbacks,
    ) -> DataConverterGroupItem {
        let mut cloned = DataConverterGroupItem::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterGroupItemBaseCallbacks,
    ) {
        self.converter_id = object.converter_id;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterGroupItemBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::CONVERTER_ID_PROPERTY_KEY => {
                self.converter_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for DataConverterGroupItemBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterGroupItemBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
