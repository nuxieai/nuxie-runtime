use crate::mechanical_port::source::viewmodel::viewmodel_property::ViewModelProperty;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, viewmodel::viewmodel_component::ViewModelComponent,
};

pub trait ViewModelPropertyBaseCallbacks: crate::mechanical_port::source::generated::viewmodel::viewmodel_component_base::ViewModelComponentBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn symbol_type_value_changed(&mut self) {}
    fn component_props_changed(&mut self) {}
}

pub struct ViewModelPropertyBase {
    pub base: ViewModelComponent,
    symbol_type_value: u32,
    component_props: u32,
}

impl Default for ViewModelPropertyBase {
    fn default() -> Self {
        Self {
            base: ViewModelComponent::default(),
            symbol_type_value: 0,
            component_props: 0,
        }
    }
}

impl ViewModelPropertyBase {
    pub const TYPE_KEY: u16 = 430;
    pub const SYMBOL_TYPE_VALUE_PROPERTY_KEY: u16 = 875;
    pub const COMPONENT_PROPS_PROPERTY_KEY: u16 = 957;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn symbol_type_value(&self) -> u32 {
        self.symbol_type_value
    }
    pub fn set_symbol_type_value(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelPropertyBaseCallbacks,
    ) {
        if !self.set_symbol_type_value_value(value) {
            return;
        }
        callbacks.symbol_type_value_changed();
        callbacks.notify_property_changed(Self::SYMBOL_TYPE_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_symbol_type_value_value(&mut self, value: u32) -> bool {
        if self.symbol_type_value == value {
            return false;
        }
        self.symbol_type_value = value;
        true
    }
    pub fn component_props(&self) -> u32 {
        self.component_props
    }
    pub fn set_component_props(
        &mut self,
        value: u32,
        callbacks: &mut impl ViewModelPropertyBaseCallbacks,
    ) {
        if !self.set_component_props_value(value) {
            return;
        }
        callbacks.component_props_changed();
        callbacks.notify_property_changed(Self::COMPONENT_PROPS_PROPERTY_KEY);
    }

    pub(crate) fn set_component_props_value(&mut self, value: u32) -> bool {
        if self.component_props == value {
            return false;
        }
        self.component_props = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ViewModelPropertyBaseCallbacks,
    ) -> ViewModelProperty {
        let mut cloned = ViewModelProperty::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ViewModelPropertyBaseCallbacks) {
        self.symbol_type_value = object.symbol_type_value;
        self.component_props = object.component_props;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ViewModelPropertyBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SYMBOL_TYPE_VALUE_PROPERTY_KEY => {
                self.symbol_type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::COMPONENT_PROPS_PROPERTY_KEY => {
                self.component_props = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ViewModelPropertyBase {
    type Target = ViewModelComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ViewModelPropertyBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
