use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_string_pad::DataConverterStringPad,
};

pub trait DataConverterStringPadBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn length_changed(&mut self) {}
    fn text_changed(&mut self) {}
    fn pad_type_changed(&mut self) {}
}

pub struct DataConverterStringPadBase {
    pub base: DataConverter,
    length: u32,
    text: String,
    pad_type: u32,
}

impl Default for DataConverterStringPadBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            length: 1,
            text: "".to_owned(),
            pad_type: 0,
        }
    }
}

impl DataConverterStringPadBase {
    pub const TYPE_KEY: u16 = 530;
    pub const LENGTH_PROPERTY_KEY: u16 = 743;
    pub const TEXT_PROPERTY_KEY: u16 = 744;
    pub const PAD_TYPE_PROPERTY_KEY: u16 = 745;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn length(&self) -> u32 {
        self.length
    }
    pub fn set_length(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterStringPadBaseCallbacks,
    ) {
        if self.length == value {
            return;
        }
        self.length = value;
        callbacks.length_changed();
        callbacks.notify_property_changed(Self::LENGTH_PROPERTY_KEY);
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn set_text(
        &mut self,
        value: String,
        callbacks: &mut impl DataConverterStringPadBaseCallbacks,
    ) {
        if self.text == value {
            return;
        }
        self.text = value;
        callbacks.text_changed();
        callbacks.notify_property_changed(Self::TEXT_PROPERTY_KEY);
    }
    pub fn pad_type(&self) -> u32 {
        self.pad_type
    }
    pub fn set_pad_type(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterStringPadBaseCallbacks,
    ) {
        if self.pad_type == value {
            return;
        }
        self.pad_type = value;
        callbacks.pad_type_changed();
        callbacks.notify_property_changed(Self::PAD_TYPE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterStringPadBaseCallbacks,
    ) -> DataConverterStringPad {
        let mut cloned = DataConverterStringPad::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterStringPadBaseCallbacks,
    ) {
        self.length = object.length;
        self.text.clone_from(&object.text);
        self.pad_type = object.pad_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterStringPadBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LENGTH_PROPERTY_KEY => {
                self.length = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::TEXT_PROPERTY_KEY => {
                self.text = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            Self::PAD_TYPE_PROPERTY_KEY => {
                self.pad_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
