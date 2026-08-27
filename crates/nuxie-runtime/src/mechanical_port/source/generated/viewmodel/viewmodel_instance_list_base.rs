use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_value::ViewModelInstanceValue,
};

pub trait ViewModelInstanceListBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn list_source_changed(&mut self) {}
}

pub struct ViewModelInstanceListBase {
    pub base: ViewModelInstanceValue,
    list_source: u32,
}

impl Default for ViewModelInstanceListBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceValue::default(),
            list_source: u32::MAX,
        }
    }
}

impl ViewModelInstanceListBase {
    pub const TYPE_KEY: u16 = 441;
    pub const LIST_SOURCE_PROPERTY_KEY: u16 = 966;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 0 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn list_source(&self) -> u32 {
        self.list_source
    }
    pub fn set_list_source(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelInstanceListBaseCallbacks,
    ) {
        if self.list_source == value {
            return;
        }
        self.list_source = value;
        callbacks.list_source_changed();
        callbacks.notify_property_changed(Self::LIST_SOURCE_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ViewModelInstanceListBaseCallbacks) {
        self.list_source = object.list_source;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceListBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LIST_SOURCE_PROPERTY_KEY => {
                self.list_source = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
