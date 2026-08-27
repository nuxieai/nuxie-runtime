use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, focus_data::FocusData,
};

pub trait FocusDataBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn focus_flags_changed(&mut self) {}
    fn edge_behavior_value_changed(&mut self) {}
}

pub struct FocusDataBase {
    pub base: Component,
    focus_flags: u32,
    edge_behavior_value: u32,
}

impl Default for FocusDataBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            focus_flags: 7,
            edge_behavior_value: 0,
        }
    }
}

impl FocusDataBase {
    pub const TYPE_KEY: u16 = 653;
    pub const FOCUS_FLAGS_PROPERTY_KEY: u16 = 1033;
    pub const CAN_FOCUS_PROPERTY_KEY: u16 = 953;
    pub const CAN_FOCUS_BITMASK: u32 = 1 << 0;
    pub const CAN_TOUCH_PROPERTY_KEY: u16 = 954;
    pub const CAN_TOUCH_BITMASK: u32 = 1 << 1;
    pub const CAN_TRAVERSE_PROPERTY_KEY: u16 = 955;
    pub const CAN_TRAVERSE_BITMASK: u32 = 1 << 2;
    pub const EDGE_BEHAVIOR_VALUE_PROPERTY_KEY: u16 = 956;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn focus_flags(&self) -> u32 {
        self.focus_flags
    }
    pub fn set_focus_flags(&mut self, value: u32, callbacks: &mut impl FocusDataBaseCallbacks) {
        if self.focus_flags == value {
            return;
        }
        self.focus_flags = value;
        callbacks.focus_flags_changed();
        callbacks.notify_property_changed(Self::FOCUS_FLAGS_PROPERTY_KEY);
    }
    pub fn edge_behavior_value(&self) -> u32 {
        self.edge_behavior_value
    }
    pub fn set_edge_behavior_value(
        &mut self,
        value: u32,
        callbacks: &mut impl FocusDataBaseCallbacks,
    ) {
        if self.edge_behavior_value == value {
            return;
        }
        self.edge_behavior_value = value;
        callbacks.edge_behavior_value_changed();
        callbacks.notify_property_changed(Self::EDGE_BEHAVIOR_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl FocusDataBaseCallbacks) -> FocusData {
        let mut cloned = FocusData::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FocusDataBaseCallbacks) {
        self.focus_flags = object.focus_flags;
        self.edge_behavior_value = object.edge_behavior_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FocusDataBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FOCUS_FLAGS_PROPERTY_KEY => {
                self.focus_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::EDGE_BEHAVIOR_VALUE_PROPERTY_KEY => {
                self.edge_behavior_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
