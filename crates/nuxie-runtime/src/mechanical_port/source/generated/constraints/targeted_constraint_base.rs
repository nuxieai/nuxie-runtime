use crate::mechanical_port::source::{
    constraints::constraint::Constraint, core::binary_reader::BinaryReader,
};

pub trait TargetedConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn target_id_changed(&mut self) {}
}

pub struct TargetedConstraintBase {
    pub base: Constraint,
    target_id: u32,
}

impl Default for TargetedConstraintBase {
    fn default() -> Self {
        Self {
            base: Constraint::default(),
            target_id: u32::MAX,
        }
    }
}

impl TargetedConstraintBase {
    pub const TYPE_KEY: u16 = 80;
    pub const TARGET_ID_PROPERTY_KEY: u16 = 173;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn target_id(&self) -> u32 {
        self.target_id
    }
    pub fn set_target_id(
        &mut self,
        value: u32,
        callbacks: &mut impl TargetedConstraintBaseCallbacks,
    ) {
        if self.target_id == value {
            return;
        }
        self.target_id = value;
        callbacks.target_id_changed();
        callbacks.notify_property_changed(Self::TARGET_ID_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TargetedConstraintBaseCallbacks) {
        self.target_id = object.target_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TargetedConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TARGET_ID_PROPERTY_KEY => {
                self.target_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
