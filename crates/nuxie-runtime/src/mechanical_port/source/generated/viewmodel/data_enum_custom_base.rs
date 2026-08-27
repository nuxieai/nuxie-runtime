use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::data_enum::DataEnum,
    viewmodel::data_enum_custom::DataEnumCustom,
};

pub trait DataEnumCustomBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn name_changed(&mut self) {}
}

pub struct DataEnumCustomBase {
    pub base: DataEnum,
    name: String,
}

impl Default for DataEnumCustomBase {
    fn default() -> Self {
        Self {
            base: DataEnum::default(),
            name: "".to_owned(),
        }
    }
}

impl DataEnumCustomBase {
    pub const TYPE_KEY: u16 = 438;
    pub const NAME_PROPERTY_KEY: u16 = 572;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 510)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn set_name(&mut self, value: String, callbacks: &mut impl DataEnumCustomBaseCallbacks) {
        if self.name == value {
            return;
        }
        self.name = value;
        callbacks.name_changed();
        callbacks.notify_property_changed(Self::NAME_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl DataEnumCustomBaseCallbacks) -> DataEnumCustom {
        let mut cloned = DataEnumCustom::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DataEnumCustomBaseCallbacks) {
        self.name.clone_from(&object.name);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataEnumCustomBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::NAME_PROPERTY_KEY => {
                self.name = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
