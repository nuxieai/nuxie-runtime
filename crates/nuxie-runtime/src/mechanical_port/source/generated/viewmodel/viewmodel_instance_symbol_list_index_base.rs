use crate::mechanical_port::source::viewmodel::viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_instance_symbol::ViewModelInstanceSymbol,
};

pub trait ViewModelInstanceSymbolListIndexBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_value_base::ViewModelInstanceValueBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn property_value_changed(&mut self) {}
}

pub struct ViewModelInstanceSymbolListIndexBase {
    pub base: ViewModelInstanceSymbol,
    property_value: u32,
}

impl Default for ViewModelInstanceSymbolListIndexBase {
    fn default() -> Self {
        Self {
            base: ViewModelInstanceSymbol::default(),
            property_value: 0,
        }
    }
}

impl ViewModelInstanceSymbolListIndexBase {
    pub const TYPE_KEY: u16 = 566;
    pub const PROPERTY_VALUE_PROPERTY_KEY: u16 = 814;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 565 | 428 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn property_value(&self) -> u32 {
        self.property_value
    }
    pub fn set_property_value(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelInstanceSymbolListIndexBaseCallbacks,
    ) {
        if !self.set_property_value_value(value) {
            return;
        }
        callbacks.property_value_changed();
        callbacks.notify_property_changed(Self::PROPERTY_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_property_value_value(&mut self, value: u32) -> bool {
        if self.property_value == value {
            return false;
        }
        self.property_value = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelInstanceSymbolListIndexBaseCallbacks,
    ) -> ViewModelInstanceSymbolListIndex {
        let mut cloned = ViewModelInstanceSymbolListIndex::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ViewModelInstanceSymbolListIndexBaseCallbacks,
    ) {
        self.property_value = object.property_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelInstanceSymbolListIndexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PROPERTY_VALUE_PROPERTY_KEY => {
                self.property_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelInstanceSymbolListIndexBase {
    type Target = ViewModelInstanceSymbol;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelInstanceSymbolListIndexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
