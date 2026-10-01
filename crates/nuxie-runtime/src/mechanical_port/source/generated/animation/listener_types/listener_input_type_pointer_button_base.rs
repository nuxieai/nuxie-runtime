use crate::mechanical_port::source::{
    animation::listener_types::{
        listener_input_type::ListenerInputType,
        listener_input_type_pointer_button::ListenerInputTypePointerButton,
    },
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    generated::animation::listener_types::listener_input_type_base::ListenerInputTypeBaseCallbacks,
};

pub trait ListenerInputTypePointerButtonBaseCallbacks: ListenerInputTypeBaseCallbacks {
    fn pointer_button_value_changed(&mut self) {}
}

#[derive(Default)]
pub struct ListenerInputTypePointerButtonBase {
    pub base: ListenerInputType,
    pointer_button_value: u8,
}

impl ListenerInputTypePointerButtonBase {
    pub const TYPE_KEY: u16 = 155;
    pub const POINTER_BUTTON_VALUE_PROPERTY_KEY: u16 = 468;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 658)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn pointer_button_value(&self) -> u8 {
        self.pointer_button_value
    }
    pub fn set_pointer_button_value(
        &mut self,
        value: u8,
        callbacks: &mut impl ListenerInputTypePointerButtonBaseCallbacks,
    ) {
        if self.set_pointer_button_value_value(value) {
            callbacks.pointer_button_value_changed();
            callbacks.notify_property_changed(Self::POINTER_BUTTON_VALUE_PROPERTY_KEY);
        }
    }
    pub(crate) fn set_pointer_button_value_value(&mut self, value: u8) -> bool {
        if self.pointer_button_value == value {
            return false;
        }
        self.pointer_button_value = value;
        true
    }
    pub fn clone_into(&self) -> ListenerInputTypePointerButton {
        let mut cloned = ListenerInputTypePointerButton::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ListenerInputTypePointerButtonBaseCallbacks,
    ) {
        self.pointer_button_value = object.pointer_button_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerInputTypePointerButtonBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::POINTER_BUTTON_VALUE_PROPERTY_KEY => {
                self.pointer_button_value = CoreUintType::deserialize(reader) as u8;
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ListenerInputTypePointerButtonBase {
    type Target = ListenerInputType;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ListenerInputTypePointerButtonBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
