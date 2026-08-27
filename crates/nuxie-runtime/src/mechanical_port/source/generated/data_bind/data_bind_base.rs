use crate::mechanical_port::source::{
    core::Core, core::binary_reader::BinaryReader, data_bind::data_bind::DataBind,
};

pub trait DataBindBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_key_changed(&mut self) {}
    fn flags_changed(&mut self) {}
    fn converter_id_changed(&mut self) {}
}

pub struct DataBindBase {
    pub base: Core,
    property_key: u32,
    flags: u32,
    converter_id: u32,
}

impl Default for DataBindBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            property_key: Core::invalidPropertyKey,
            flags: 0,
            converter_id: u32::MAX,
        }
    }
}

impl DataBindBase {
    pub const TYPE_KEY: u16 = 446;
    pub const PROPERTY_KEY_PROPERTY_KEY: u16 = 586;
    pub const FLAGS_PROPERTY_KEY: u16 = 587;
    pub const CONVERTER_ID_PROPERTY_KEY: u16 = 660;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_key(&self) -> u32 {
        self.property_key
    }
    pub fn set_property_key(&mut self, value: u32, callbacks: &mut impl DataBindBaseCallbacks) {
        if self.property_key == value {
            return;
        }
        self.property_key = value;
        callbacks.property_key_changed();
        callbacks.notify_property_changed(Self::PROPERTY_KEY_PROPERTY_KEY);
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(&mut self, value: u32, callbacks: &mut impl DataBindBaseCallbacks) {
        if self.flags == value {
            return;
        }
        self.flags = value;
        callbacks.flags_changed();
        callbacks.notify_property_changed(Self::FLAGS_PROPERTY_KEY);
    }
    pub fn converter_id(&self) -> u32 {
        self.converter_id
    }
    pub fn set_converter_id(&mut self, value: u32, callbacks: &mut impl DataBindBaseCallbacks) {
        if self.converter_id == value {
            return;
        }
        self.converter_id = value;
        callbacks.converter_id_changed();
        callbacks.notify_property_changed(Self::CONVERTER_ID_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl DataBindBaseCallbacks) -> DataBind {
        let mut cloned = DataBind::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataBindBaseCallbacks) {
        self.property_key = object.property_key;
        self.flags = object.flags;
        self.converter_id = object.converter_id;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataBindBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_KEY_PROPERTY_KEY => {
                self.property_key = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::FLAGS_PROPERTY_KEY => {
                self.flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::CONVERTER_ID_PROPERTY_KEY => {
                self.converter_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}
