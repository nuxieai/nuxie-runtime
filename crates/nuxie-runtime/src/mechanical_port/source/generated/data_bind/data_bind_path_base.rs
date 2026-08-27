use crate::mechanical_port::source::{
    core::Core, core::binary_reader::BinaryReader, data_bind::data_bind_path::DataBindPath,
};

pub trait DataBindPathBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn path_changed(&mut self) {}
    fn is_relative_changed(&mut self) {}
    fn decode_path(&mut self, value: &[u8]);
    fn copy_path(&mut self, object: &DataBindPathBase);
}

pub struct DataBindPathBase {
    pub base: Core,
    is_relative: bool,
}

impl Default for DataBindPathBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            is_relative: false,
        }
    }
}

impl DataBindPathBase {
    pub const TYPE_KEY: u16 = 643;
    pub const PATH_PROPERTY_KEY: u16 = 920;
    pub const IS_RELATIVE_PROPERTY_KEY: u16 = 921;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn is_relative(&self) -> bool {
        self.is_relative
    }
    pub fn set_is_relative(&mut self, value: bool, callbacks: &mut impl DataBindPathBaseCallbacks) {
        if self.is_relative == value {
            return;
        }
        self.is_relative = value;
        callbacks.is_relative_changed();
        callbacks.notify_property_changed(Self::IS_RELATIVE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl DataBindPathBaseCallbacks) -> DataBindPath {
        let mut cloned = DataBindPath::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataBindPathBaseCallbacks) {
        callbacks.copy_path(object);
        self.is_relative = object.is_relative;
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataBindPathBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::IS_RELATIVE_PROPERTY_KEY => {
                self.is_relative = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::PATH_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_path(value.as_slice());
                true
            }
            _ => false,
        }
    }
}
