use crate::mechanical_port::source::{
    constraints::constraint::Constraint, core::binary_reader::BinaryReader,
};

pub trait DraggableConstraintBaseCallbacks:
    crate::mechanical_port::source::generated::constraints::constraint_base::ConstraintBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn direction_value_changed(&mut self) {}
}

pub struct DraggableConstraintBase {
    pub base: Constraint,
    direction_value: u32,
}

impl Default for DraggableConstraintBase {
    fn default() -> Self {
        Self {
            base: Constraint::default(),
            direction_value: 1,
        }
    }
}

impl DraggableConstraintBase {
    pub const TYPE_KEY: u16 = 520;
    pub const DIRECTION_VALUE_PROPERTY_KEY: u16 = 722;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn direction_value(&self) -> u32 {
        self.direction_value
    }
    pub fn set_direction_value(
        &mut self,
        value: u32,
        callbacks: &mut impl DraggableConstraintBaseCallbacks,
    ) {
        if !self.set_direction_value_value(value) {
            return;
        }
        callbacks.direction_value_changed();
        DraggableConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::DIRECTION_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_direction_value_value(&mut self, value: u32) -> bool {
        if self.direction_value == value {
            return false;
        }
        self.direction_value = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DraggableConstraintBaseCallbacks) {
        self.direction_value = object.direction_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DraggableConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DIRECTION_VALUE_PROPERTY_KEY => {
                self.direction_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DraggableConstraintBase {
    type Target = Constraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DraggableConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
