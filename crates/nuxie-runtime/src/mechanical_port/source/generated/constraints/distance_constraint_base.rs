use crate::mechanical_port::source::{
    constraints::distance_constraint::DistanceConstraint,
    constraints::targeted_constraint::TargetedConstraint, core::binary_reader::BinaryReader,
};

pub trait DistanceConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn distance_changed(&mut self) {}
    fn mode_value_changed(&mut self) {}
}

pub struct DistanceConstraintBase {
    pub base: TargetedConstraint,
    distance: f32,
    mode_value: u32,
}

impl Default for DistanceConstraintBase {
    fn default() -> Self {
        Self {
            base: TargetedConstraint::default(),
            distance: 100.0,
            mode_value: 0,
        }
    }
}

impl DistanceConstraintBase {
    pub const TYPE_KEY: u16 = 82;
    pub const DISTANCE_PROPERTY_KEY: u16 = 177;
    pub const MODE_VALUE_PROPERTY_KEY: u16 = 178;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn distance(&self) -> f32 {
        self.distance
    }
    pub fn set_distance(
        &mut self,
        value: f32,
        callbacks: &mut impl DistanceConstraintBaseCallbacks,
    ) {
        if self.distance == value {
            return;
        }
        self.distance = value;
        callbacks.distance_changed();
        callbacks.notify_property_changed(Self::DISTANCE_PROPERTY_KEY);
    }
    pub fn mode_value(&self) -> u32 {
        self.mode_value
    }
    pub fn set_mode_value(
        &mut self,
        value: u32,
        callbacks: &mut impl DistanceConstraintBaseCallbacks,
    ) {
        if self.mode_value == value {
            return;
        }
        self.mode_value = value;
        callbacks.mode_value_changed();
        callbacks.notify_property_changed(Self::MODE_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DistanceConstraintBaseCallbacks,
    ) -> DistanceConstraint {
        let mut cloned = DistanceConstraint::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DistanceConstraintBaseCallbacks) {
        self.distance = object.distance;
        self.mode_value = object.mode_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DistanceConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DISTANCE_PROPERTY_KEY => {
                self.distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
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
