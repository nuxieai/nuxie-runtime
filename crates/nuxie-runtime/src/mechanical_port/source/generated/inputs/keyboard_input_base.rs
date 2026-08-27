use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    inputs::{keyboard_input::KeyboardInput, user_input::UserInput},
};

pub trait KeyboardInputBaseCallbacks {
    fn key_type_changed(&mut self) {}
    fn key_phase_changed(&mut self) {}
    fn modifiers_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct KeyboardInputBase {
    pub base: UserInput,
    key_type: u32,
    key_phase: u32,
    modifiers: u32,
}

impl Default for KeyboardInputBase {
    fn default() -> Self {
        Self {
            base: UserInput::default(),
            key_type: u32::MAX,
            key_phase: 0,
            modifiers: 0,
        }
    }
}

impl KeyboardInputBase {
    pub const TYPE_KEY: u16 = 664;
    pub const KEY_TYPE_PROPERTY_KEY: u16 = 971;
    pub const KEY_PHASE_PROPERTY_KEY: u16 = 972;
    pub const MODIFIERS_PROPERTY_KEY: u16 = 973;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 663)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn key_type(&self) -> u32 {
        self.key_type
    }
    pub fn key_phase(&self) -> u32 {
        self.key_phase
    }
    pub fn modifiers(&self) -> u32 {
        self.modifiers
    }

    pub fn set_key_type<C: KeyboardInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.key_type == value {
            return;
        }
        self.key_type = value;
        c.key_type_changed();
        c.notify_property_changed(Self::KEY_TYPE_PROPERTY_KEY);
    }
    pub fn set_key_phase<C: KeyboardInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.key_phase == value {
            return;
        }
        self.key_phase = value;
        c.key_phase_changed();
        c.notify_property_changed(Self::KEY_PHASE_PROPERTY_KEY);
    }
    pub fn set_modifiers<C: KeyboardInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if self.modifiers == value {
            return;
        }
        self.modifiers = value;
        c.modifiers_changed();
        c.notify_property_changed(Self::MODIFIERS_PROPERTY_KEY);
    }

    pub fn clone_into<C: KeyboardInputBaseCallbacks>(&self, c: &mut C) -> KeyboardInput {
        let mut cloned = KeyboardInput::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: KeyboardInputBaseCallbacks>(&mut self, object: &Self, _c: &mut C) {
        self.key_type = object.key_type;
        self.key_phase = object.key_phase;
        self.modifiers = object.modifiers;
        self.base.base.copy(&object.base.base);
    }
    pub fn deserialize<C: KeyboardInputBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        _c: &mut C,
    ) -> bool {
        match key {
            Self::KEY_TYPE_PROPERTY_KEY => {
                self.key_type = CoreUintType::deserialize(reader);
                true
            }
            Self::KEY_PHASE_PROPERTY_KEY => {
                self.key_phase = CoreUintType::deserialize(reader);
                true
            }
            Self::MODIFIERS_PROPERTY_KEY => {
                self.modifiers = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader),
        }
    }
}
