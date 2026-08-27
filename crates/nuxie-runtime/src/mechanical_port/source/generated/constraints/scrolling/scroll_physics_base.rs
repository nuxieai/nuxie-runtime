use crate::mechanical_port::source::{component::Component, core::binary_reader::BinaryReader};

pub trait ScrollPhysicsBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn constraint_id_changed(&mut self) {}
}

pub struct ScrollPhysicsBase {
    pub base: Component,
    constraint_id: u32,
}

impl Default for ScrollPhysicsBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            constraint_id: u32::MAX,
        }
    }
}

impl ScrollPhysicsBase {
    pub const TYPE_KEY: u16 = 523;
    pub const CONSTRAINT_ID_PROPERTY_KEY: u16 = 731;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn constraint_id(&self) -> u32 {
        self.constraint_id
    }
    pub fn set_constraint_id(
        &mut self,
        value: u32,
        callbacks: &mut impl ScrollPhysicsBaseCallbacks,
    ) {
        if self.constraint_id == value {
            return;
        }
        self.constraint_id = value;
        callbacks.constraint_id_changed();
        callbacks.notify_property_changed(Self::CONSTRAINT_ID_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ScrollPhysicsBaseCallbacks) {
        self.constraint_id = object.constraint_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ScrollPhysicsBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::CONSTRAINT_ID_PROPERTY_KEY => {
                self.constraint_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
