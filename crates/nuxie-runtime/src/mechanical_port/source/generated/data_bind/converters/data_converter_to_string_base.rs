use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_to_string::DataConverterToString,
};

pub trait DataConverterToStringBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn flags_changed(&mut self) {}
    fn decimals_changed(&mut self) {}
    fn color_format_changed(&mut self) {}
}

pub struct DataConverterToStringBase {
    pub base: DataConverter,
    flags: u32,
    decimals: u32,
    color_format: String,
}

impl Default for DataConverterToStringBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            flags: 0,
            decimals: 0,
            color_format: "".to_owned(),
        }
    }
}

impl DataConverterToStringBase {
    pub const TYPE_KEY: u16 = 490;
    pub const FLAGS_PROPERTY_KEY: u16 = 764;
    pub const DECIMALS_PROPERTY_KEY: u16 = 765;
    pub const COLOR_FORMAT_PROPERTY_KEY: u16 = 766;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterToStringBaseCallbacks,
    ) {
        if self.flags == value {
            return;
        }
        self.flags = value;
        callbacks.flags_changed();
        callbacks.notify_property_changed(Self::FLAGS_PROPERTY_KEY);
    }
    pub fn decimals(&self) -> u32 {
        self.decimals
    }
    pub fn set_decimals(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterToStringBaseCallbacks,
    ) {
        if self.decimals == value {
            return;
        }
        self.decimals = value;
        callbacks.decimals_changed();
        callbacks.notify_property_changed(Self::DECIMALS_PROPERTY_KEY);
    }
    pub fn color_format(&self) -> &str {
        &self.color_format
    }
    pub fn set_color_format(
        &mut self,
        value: String,
        callbacks: &mut impl DataConverterToStringBaseCallbacks,
    ) {
        if self.color_format == value {
            return;
        }
        self.color_format = value;
        callbacks.color_format_changed();
        callbacks.notify_property_changed(Self::COLOR_FORMAT_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterToStringBaseCallbacks,
    ) -> DataConverterToString {
        let mut cloned = DataConverterToString::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataConverterToStringBaseCallbacks) {
        self.flags = object.flags;
        self.decimals = object.decimals;
        self.color_format.clone_from(&object.color_format);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterToStringBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FLAGS_PROPERTY_KEY => {
                self.flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::DECIMALS_PROPERTY_KEY => {
                self.decimals = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::COLOR_FORMAT_PROPERTY_KEY => {
                self.color_format = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
