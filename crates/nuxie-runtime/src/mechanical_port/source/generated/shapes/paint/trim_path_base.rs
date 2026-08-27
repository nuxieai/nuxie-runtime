use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, shapes::paint::trim_path::TrimPath,
};

pub trait TrimPathBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn start_changed(&mut self) {}
    fn end_changed(&mut self) {}
    fn offset_changed(&mut self) {}
    fn mode_value_changed(&mut self) {}
}

pub struct TrimPathBase {
    pub base: Component,
    start: f32,
    end: f32,
    offset: f32,
    mode_value: u32,
}

impl Default for TrimPathBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            start: 0.0,
            end: 0.0,
            offset: 0.0,
            mode_value: 0,
        }
    }
}

impl TrimPathBase {
    pub const TYPE_KEY: u16 = 47;
    pub const START_PROPERTY_KEY: u16 = 114;
    pub const END_PROPERTY_KEY: u16 = 115;
    pub const OFFSET_PROPERTY_KEY: u16 = 116;
    pub const MODE_VALUE_PROPERTY_KEY: u16 = 117;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn start(&self) -> f32 {
        self.start
    }
    pub fn set_start(&mut self, value: f32, callbacks: &mut impl TrimPathBaseCallbacks) {
        if self.start == value {
            return;
        }
        self.start = value;
        callbacks.start_changed();
        callbacks.notify_property_changed(Self::START_PROPERTY_KEY);
    }
    pub fn end(&self) -> f32 {
        self.end
    }
    pub fn set_end(&mut self, value: f32, callbacks: &mut impl TrimPathBaseCallbacks) {
        if self.end == value {
            return;
        }
        self.end = value;
        callbacks.end_changed();
        callbacks.notify_property_changed(Self::END_PROPERTY_KEY);
    }
    pub fn offset(&self) -> f32 {
        self.offset
    }
    pub fn set_offset(&mut self, value: f32, callbacks: &mut impl TrimPathBaseCallbacks) {
        if self.offset == value {
            return;
        }
        self.offset = value;
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }
    pub fn mode_value(&self) -> u32 {
        self.mode_value
    }
    pub fn set_mode_value(&mut self, value: u32, callbacks: &mut impl TrimPathBaseCallbacks) {
        if self.mode_value == value {
            return;
        }
        self.mode_value = value;
        callbacks.mode_value_changed();
        callbacks.notify_property_changed(Self::MODE_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl TrimPathBaseCallbacks) -> TrimPath {
        let mut cloned = TrimPath::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TrimPathBaseCallbacks) {
        self.start = object.start;
        self.end = object.end;
        self.offset = object.offset;
        self.mode_value = object.mode_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TrimPathBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::START_PROPERTY_KEY => {
                self.start = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::END_PROPERTY_KEY => {
                self.end = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MODE_VALUE_PROPERTY_KEY => {
                self.mode_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
