use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_string_trim::DataConverterStringTrim,
};

pub trait DataConverterStringTrimBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn trim_type_changed(&mut self) {}
}

pub struct DataConverterStringTrimBase {
    pub base: DataConverter,
    trim_type: u32,
}

impl Default for DataConverterStringTrimBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            trim_type: 1,
        }
    }
}

impl DataConverterStringTrimBase {
    pub const TYPE_KEY: u16 = 532;
    pub const TRIM_TYPE_PROPERTY_KEY: u16 = 746;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn trim_type(&self) -> u32 {
        self.trim_type
    }
    pub fn set_trim_type(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterStringTrimBaseCallbacks,
    ) {
        if self.trim_type == value {
            return;
        }
        self.trim_type = value;
        callbacks.trim_type_changed();
        callbacks.notify_property_changed(Self::TRIM_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterStringTrimBaseCallbacks,
    ) -> DataConverterStringTrim {
        let mut cloned = DataConverterStringTrim::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterStringTrimBaseCallbacks,
    ) {
        self.trim_type = object.trim_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterStringTrimBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TRIM_TYPE_PROPERTY_KEY => {
                self.trim_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
