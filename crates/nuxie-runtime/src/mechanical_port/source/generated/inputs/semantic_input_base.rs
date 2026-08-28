use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    inputs::{semantic_input::SemanticInput, user_input::UserInput},
};

pub trait SemanticInputBaseCallbacks {
    fn action_type_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct SemanticInputBase {
    pub base: UserInput,
    action_type: u32,
}

impl Default for SemanticInputBase {
    fn default() -> Self {
        Self {
            base: UserInput::default(),
            action_type: 0,
        }
    }
}

impl SemanticInputBase {
    pub const TYPE_KEY: u16 = 670;
    pub const ACTION_TYPE_PROPERTY_KEY: u16 = 1010;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 663)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn action_type(&self) -> u32 {
        self.action_type
    }

    pub fn set_action_type<C: SemanticInputBaseCallbacks>(
        &mut self,
        value: u32,
        callbacks: &mut C,
    ) {
        if !self.set_action_type_value(value) {
            return;
        }
        callbacks.action_type_changed();
        callbacks.notify_property_changed(Self::ACTION_TYPE_PROPERTY_KEY);
    }

    pub(crate) fn set_action_type_value(&mut self, value: u32) -> bool {
        if self.action_type == value {
            return false;
        }
        self.action_type = value;
        true
    }

    pub fn clone_into<C: SemanticInputBaseCallbacks>(&self, callbacks: &mut C) -> SemanticInput {
        let mut cloned = SemanticInput::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy<C: SemanticInputBaseCallbacks>(&mut self, object: &Self, _callbacks: &mut C) {
        self.action_type = object.action_type;
        self.base.base.copy(&object.base.base);
    }

    pub fn deserialize<C: SemanticInputBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        _callbacks: &mut C,
    ) -> bool {
        match key {
            Self::ACTION_TYPE_PROPERTY_KEY => {
                self.action_type = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader),
        }
    }
}

impl std::ops::Deref for SemanticInputBase {
    type Target = UserInput;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for SemanticInputBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
