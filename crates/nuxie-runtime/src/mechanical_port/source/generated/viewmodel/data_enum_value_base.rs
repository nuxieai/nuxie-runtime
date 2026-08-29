use crate::mechanical_port::source::{
    core::Core, core::binary_reader::BinaryReader, viewmodel::data_enum_value::DataEnumValue,
};

pub trait DataEnumValueBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn key_changed(&mut self) {}
    fn value_changed(&mut self) {}
}

pub struct DataEnumValueBase {
    pub base: Core,
    key: String,
    value: String,
}

impl Default for DataEnumValueBase {
    fn default() -> Self {
        Self {
            base: Core::default(),
            key: "".to_owned(),
            value: "".to_owned(),
        }
    }
}

impl DataEnumValueBase {
    pub const TYPE_KEY: u16 = 445;
    pub const KEY_PROPERTY_KEY: u16 = 578;
    pub const VALUE_PROPERTY_KEY: u16 = 579;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn key(&self) -> &str {
        &self.key
    }
    pub fn set_key(&mut self, value: String, callbacks: &mut impl DataEnumValueBaseCallbacks) {
        if !self.set_key_value(value) {
            return;
        }
        callbacks.key_changed();
        callbacks.notify_property_changed(Self::KEY_PROPERTY_KEY);
    }

    pub(crate) fn set_key_value(&mut self, value: String) -> bool {
        if self.key == value {
            return false;
        }
        self.key = value;
        true
    }
    pub fn value(&self) -> &str {
        &self.value
    }
    pub fn set_value(&mut self, value: String, callbacks: &mut impl DataEnumValueBaseCallbacks) {
        if !self.set_value_value(value) {
            return;
        }
        callbacks.value_changed();
        callbacks.notify_property_changed(Self::VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_value_value(&mut self, value: String) -> bool {
        if self.value == value {
            return false;
        }
        self.value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl DataEnumValueBaseCallbacks) -> DataEnumValue {
        let mut cloned = DataEnumValue::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataEnumValueBaseCallbacks) {
        self.key.clone_from(&object.key);
        self.value.clone_from(&object.value);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataEnumValueBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::KEY_PROPERTY_KEY => {
                self.key = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            Self::VALUE_PROPERTY_KEY => {
                self.value = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => false,
        }
    }
}

impl std::ops::Deref for DataEnumValueBase {
    type Target = Core;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataEnumValueBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
