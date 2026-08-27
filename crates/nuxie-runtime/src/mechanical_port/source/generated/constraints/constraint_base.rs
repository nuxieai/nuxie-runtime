use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait ConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn strength_changed(&mut self) {}
}

pub struct ConstraintBase {
    pub base: Component,
    strength: f32,
}

impl Default for ConstraintBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            strength: 1.0,
        }
    }
}

impl ConstraintBase {
    pub const TYPE_KEY: u16 = 79;
    pub const STRENGTH_PROPERTY_KEY: u16 = 172;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn strength(&self) -> f32 {
        self.strength
    }
    pub fn set_strength(&mut self, value: f32, callbacks: &mut impl ConstraintBaseCallbacks) {
        if self.strength == value {
            return;
        }
        self.strength = value;
        callbacks.strength_changed();
        callbacks.notify_property_changed(Self::STRENGTH_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ConstraintBaseCallbacks) {
        self.strength = object.strength;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::STRENGTH_PROPERTY_KEY => {
                self.strength = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
