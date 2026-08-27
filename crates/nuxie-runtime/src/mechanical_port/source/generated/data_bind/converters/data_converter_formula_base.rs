use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_formula::DataConverterFormula,
};

pub trait DataConverterFormulaBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn random_mode_value_changed(&mut self) {}
}

pub struct DataConverterFormulaBase {
    pub base: DataConverter,
    random_mode_value: u32,
}

impl Default for DataConverterFormulaBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            random_mode_value: 0,
        }
    }
}

impl DataConverterFormulaBase {
    pub const TYPE_KEY: u16 = 536;
    pub const RANDOM_MODE_VALUE_PROPERTY_KEY: u16 = 887;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn random_mode_value(&self) -> u32 {
        self.random_mode_value
    }
    pub fn set_random_mode_value(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterFormulaBaseCallbacks,
    ) {
        if self.random_mode_value == value {
            return;
        }
        self.random_mode_value = value;
        callbacks.random_mode_value_changed();
        callbacks.notify_property_changed(Self::RANDOM_MODE_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterFormulaBaseCallbacks,
    ) -> DataConverterFormula {
        let mut cloned = DataConverterFormula::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataConverterFormulaBaseCallbacks) {
        self.random_mode_value = object.random_mode_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterFormulaBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::RANDOM_MODE_VALUE_PROPERTY_KEY => {
                self.random_mode_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
