use crate::mechanical_port::source::{
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    inputs::{gamepad_input::GamepadInput, user_input::UserInput},
};

pub trait GamepadInputBaseCallbacks {
    fn kind_changed(&mut self) {}
    fn mapping_changed(&mut self) {}
    fn input_index_changed(&mut self) {}
    fn button_phase_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct GamepadInputBase {
    pub base: UserInput,
    kind: u32,
    mapping: u32,
    input_index: u32,
    button_phase: u32,
}

impl Default for GamepadInputBase {
    fn default() -> Self {
        Self {
            base: UserInput::default(),
            kind: 0,
            mapping: 0,
            input_index: 0,
            button_phase: 1,
        }
    }
}

impl GamepadInputBase {
    pub const TYPE_KEY: u16 = 974;
    pub const KIND_PROPERTY_KEY: u16 = 1021;
    pub const MAPPING_PROPERTY_KEY: u16 = 1018;
    pub const INPUT_INDEX_PROPERTY_KEY: u16 = 1019;
    pub const BUTTON_PHASE_PROPERTY_KEY: u16 = 1020;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 663)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn kind(&self) -> u32 {
        self.kind
    }
    pub fn mapping(&self) -> u32 {
        self.mapping
    }
    pub fn input_index(&self) -> u32 {
        self.input_index
    }
    pub fn button_phase(&self) -> u32 {
        self.button_phase
    }

    pub fn set_kind<C: GamepadInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if !self.set_kind_value(value) {
            return;
        }
        c.kind_changed();
        c.notify_property_changed(Self::KIND_PROPERTY_KEY);
    }

    pub(crate) fn set_kind_value(&mut self, value: u32) -> bool {
        if self.kind == value {
            return false;
        }
        self.kind = value;
        true
    }
    pub fn set_mapping<C: GamepadInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if !self.set_mapping_value(value) {
            return;
        }
        c.mapping_changed();
        c.notify_property_changed(Self::MAPPING_PROPERTY_KEY);
    }

    pub(crate) fn set_mapping_value(&mut self, value: u32) -> bool {
        if self.mapping == value {
            return false;
        }
        self.mapping = value;
        true
    }
    pub fn set_input_index<C: GamepadInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if !self.set_input_index_value(value) {
            return;
        }
        c.input_index_changed();
        c.notify_property_changed(Self::INPUT_INDEX_PROPERTY_KEY);
    }

    pub(crate) fn set_input_index_value(&mut self, value: u32) -> bool {
        if self.input_index == value {
            return false;
        }
        self.input_index = value;
        true
    }
    pub fn set_button_phase<C: GamepadInputBaseCallbacks>(&mut self, value: u32, c: &mut C) {
        if !self.set_button_phase_value(value) {
            return;
        }
        c.button_phase_changed();
        c.notify_property_changed(Self::BUTTON_PHASE_PROPERTY_KEY);
    }

    pub(crate) fn set_button_phase_value(&mut self, value: u32) -> bool {
        if self.button_phase == value {
            return false;
        }
        self.button_phase = value;
        true
    }

    pub fn clone_into<C: GamepadInputBaseCallbacks>(&self, c: &mut C) -> GamepadInput {
        let mut cloned = GamepadInput::default();
        cloned.base.copy(self, c);
        cloned
    }
    pub fn copy<C: GamepadInputBaseCallbacks>(&mut self, object: &Self, _c: &mut C) {
        self.kind = object.kind;
        self.mapping = object.mapping;
        self.input_index = object.input_index;
        self.button_phase = object.button_phase;
        self.base.base.copy(&object.base.base);
    }
    pub fn deserialize<C: GamepadInputBaseCallbacks>(
        &mut self,
        key: u16,
        reader: &mut BinaryReader<'_>,
        _c: &mut C,
    ) -> bool {
        match key {
            Self::KIND_PROPERTY_KEY => {
                self.kind = CoreUintType::deserialize(reader);
                true
            }
            Self::MAPPING_PROPERTY_KEY => {
                self.mapping = CoreUintType::deserialize(reader);
                true
            }
            Self::INPUT_INDEX_PROPERTY_KEY => {
                self.input_index = CoreUintType::deserialize(reader);
                true
            }
            Self::BUTTON_PHASE_PROPERTY_KEY => {
                self.button_phase = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(key, reader),
        }
    }
}

impl std::ops::Deref for GamepadInputBase {
    type Target = UserInput;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GamepadInputBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
