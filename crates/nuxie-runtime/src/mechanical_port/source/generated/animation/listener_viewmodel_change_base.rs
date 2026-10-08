use crate::mechanical_port::source::animation::listener_viewmodel_change::ListenerViewModelChange;

use crate::mechanical_port::source::{
    animation::listener_action::ListenerAction, core::binary_reader::BinaryReader,
};

pub trait ListenerViewModelChangeBaseCallbacks:
    crate::mechanical_port::source::generated::animation::listener_action_base::ListenerActionBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct ListenerViewModelChangeBase {
    pub base: ListenerAction,
    input_value: u8,
    input_value_index: u8,
}

impl Default for ListenerViewModelChangeBase {
    fn default() -> Self {
        Self {
            base: ListenerAction::default(),
            input_value: 0,
            input_value_index: 0,
        }
    }
}

impl ListenerViewModelChangeBase {
    pub const TYPE_KEY: u16 = 487;
    pub const INPUT_VALUE_PROPERTY_KEY: u16 = 453;
    pub const INPUT_VALUE_INDEX_PROPERTY_KEY: u16 = 454;

    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn input_value(&self) -> u8 {
        self.input_value
    }
    pub fn input_value_index(&self) -> u8 {
        self.input_value_index
    }
    pub fn set_input_value(
        &mut self,
        value: u8,
        callbacks: &mut impl ListenerViewModelChangeBaseCallbacks,
    ) {
        if !self.set_input_value_value(value) {
            return;
        }
        ListenerViewModelChangeBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INPUT_VALUE_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_input_value_value(&mut self, value: u8) -> bool {
        if self.input_value == value {
            return false;
        }
        self.input_value = value;
        true
    }
    pub fn set_input_value_index(
        &mut self,
        value: u8,
        callbacks: &mut impl ListenerViewModelChangeBaseCallbacks,
    ) {
        if !self.set_input_value_index_value(value) {
            return;
        }
        ListenerViewModelChangeBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INPUT_VALUE_INDEX_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_input_value_index_value(&mut self, value: u8) -> bool {
        if self.input_value_index == value {
            return false;
        }
        self.input_value_index = value;
        true
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ListenerViewModelChangeBaseCallbacks,
    ) {
        self.input_value = object.input_value;
        self.input_value_index = object.input_value_index;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListenerViewModelChangeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INPUT_VALUE_PROPERTY_KEY => {
                self.input_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::INPUT_VALUE_INDEX_PROPERTY_KEY => {
                self.input_value_index = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
    pub fn clone_into(&self) -> ListenerViewModelChange {
        let mut cloned = ListenerViewModelChange::default();
        let mut base = std::mem::take(&mut cloned.base);
        base.copy(self, &mut cloned);
        cloned.base = base;
        cloned
    }
}

impl std::ops::Deref for ListenerViewModelChangeBase {
    type Target = ListenerAction;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListenerViewModelChangeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
