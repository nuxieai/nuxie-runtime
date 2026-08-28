use crate::mechanical_port::source::{
    animation::listener_action::ListenerAction, core::binary_reader::BinaryReader,
};

pub trait ListenerInputChangeBaseCallbacks: crate::mechanical_port::source::generated::animation::listener_action_base::ListenerActionBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn input_id_changed(&mut self) {}
    fn nested_input_id_changed(&mut self) {}
}

pub struct ListenerInputChangeBase {
    pub base: ListenerAction,
    input_id: u32,
    nested_input_id: u32,
}

impl Default for ListenerInputChangeBase {
    fn default() -> Self {
        Self {
            base: ListenerAction::default(),
            input_id: u32::MAX,
            nested_input_id: u32::MAX,
        }
    }
}

impl ListenerInputChangeBase {
    pub const TYPE_KEY: u16 = 116;
    pub const INPUT_ID_PROPERTY_KEY: u16 = 227;
    pub const NESTED_INPUT_ID_PROPERTY_KEY: u16 = 400;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 125)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn input_id(&self) -> u32 {
        self.input_id
    }
    pub fn set_input_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ListenerInputChangeBaseCallbacks,
    ) {
        if !self.set_input_id_value(value) {
            return;
        }
        callbacks.input_id_changed();
        callbacks.notify_property_changed(Self::INPUT_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_input_id_value(&mut self, value: u32) -> bool {
        if self.input_id == value {
            return false;
        }
        self.input_id = value;
        true
    }
    pub fn nested_input_id(&self) -> u32 {
        self.nested_input_id
    }
    pub fn set_nested_input_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ListenerInputChangeBaseCallbacks,
    ) {
        if !self.set_nested_input_id_value(value) {
            return;
        }
        callbacks.nested_input_id_changed();
        callbacks.notify_property_changed(Self::NESTED_INPUT_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_nested_input_id_value(&mut self, value: u32) -> bool {
        if self.nested_input_id == value {
            return false;
        }
        self.nested_input_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ListenerInputChangeBaseCallbacks) {
        self.input_id = object.input_id;
        self.nested_input_id = object.nested_input_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerInputChangeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INPUT_ID_PROPERTY_KEY => {
                self.input_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::NESTED_INPUT_ID_PROPERTY_KEY => {
                self.nested_input_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ListenerInputChangeBase {
    type Target = ListenerAction;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerInputChangeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
