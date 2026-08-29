use crate::mechanical_port::source::{
    constraints::ik_constraint::IKConstraint, constraints::targeted_constraint::TargetedConstraint,
    core::binary_reader::BinaryReader,
};

pub trait IKConstraintBaseCallbacks: crate::mechanical_port::source::generated::constraints::targeted_constraint_base::TargetedConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn invert_direction_changed(&mut self) {}
    fn parent_bone_count_changed(&mut self) {}
}

pub struct IKConstraintBase {
    pub base: TargetedConstraint,
    invert_direction: bool,
    parent_bone_count: u32,
}

impl Default for IKConstraintBase {
    fn default() -> Self {
        Self {
            base: TargetedConstraint::default(),
            invert_direction: false,
            parent_bone_count: 0,
        }
    }
}

impl IKConstraintBase {
    pub const TYPE_KEY: u16 = 81;
    pub const INVERT_DIRECTION_PROPERTY_KEY: u16 = 174;
    pub const PARENT_BONE_COUNT_PROPERTY_KEY: u16 = 175;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn invert_direction(&self) -> bool {
        self.invert_direction
    }
    pub fn set_invert_direction(
        &mut self,
        value: bool,
        callbacks: &mut impl IKConstraintBaseCallbacks,
    ) {
        if !self.set_invert_direction_value(value) {
            return;
        }
        callbacks.invert_direction_changed();
        IKConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INVERT_DIRECTION_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_invert_direction_value(&mut self, value: bool) -> bool {
        if self.invert_direction == value {
            return false;
        }
        self.invert_direction = value;
        true
    }
    pub fn parent_bone_count(&self) -> u32 {
        self.parent_bone_count
    }
    pub fn set_parent_bone_count(
        &mut self,
        value: u32,
        callbacks: &mut impl IKConstraintBaseCallbacks,
    ) {
        if !self.set_parent_bone_count_value(value) {
            return;
        }
        callbacks.parent_bone_count_changed();
        IKConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PARENT_BONE_COUNT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_parent_bone_count_value(&mut self, value: u32) -> bool {
        if self.parent_bone_count == value {
            return false;
        }
        self.parent_bone_count = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl IKConstraintBaseCallbacks) -> IKConstraint {
        let mut cloned = IKConstraint::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl IKConstraintBaseCallbacks) {
        self.invert_direction = object.invert_direction;
        self.parent_bone_count = object.parent_bone_count;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl IKConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INVERT_DIRECTION_PROPERTY_KEY => {
                self.invert_direction = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::PARENT_BONE_COUNT_PROPERTY_KEY => {
                self.parent_bone_count = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for IKConstraintBase {
    type Target = TargetedConstraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for IKConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
