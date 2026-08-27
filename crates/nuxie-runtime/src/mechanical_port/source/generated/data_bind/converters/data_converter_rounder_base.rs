use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_rounder::DataConverterRounder,
};

pub trait DataConverterRounderBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn decimals_changed(&mut self) {}
}

pub struct DataConverterRounderBase {
    pub base: DataConverter,
    decimals: u32,
}

impl Default for DataConverterRounderBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            decimals: 0,
        }
    }
}

impl DataConverterRounderBase {
    pub const TYPE_KEY: u16 = 489;
    pub const DECIMALS_PROPERTY_KEY: u16 = 669;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn decimals(&self) -> u32 {
        self.decimals
    }
    pub fn set_decimals(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterRounderBaseCallbacks,
    ) {
        if self.decimals == value {
            return;
        }
        self.decimals = value;
        callbacks.decimals_changed();
        callbacks.notify_property_changed(Self::DECIMALS_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterRounderBaseCallbacks,
    ) -> DataConverterRounder {
        let mut cloned = DataConverterRounder::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataConverterRounderBaseCallbacks) {
        self.decimals = object.decimals;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterRounderBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DECIMALS_PROPERTY_KEY => {
                self.decimals = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
