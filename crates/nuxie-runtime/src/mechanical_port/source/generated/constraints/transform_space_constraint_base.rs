use crate::mechanical_port::source::{
    constraints::targeted_constraint::TargetedConstraint, core::binary_reader::BinaryReader,
};

pub trait TransformSpaceConstraintBaseCallbacks: crate::mechanical_port::source::generated::constraints::targeted_constraint_base::TargetedConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn source_space_value_changed(&mut self) {}
    fn dest_space_value_changed(&mut self) {}
}

pub struct TransformSpaceConstraintBase {
    pub base: TargetedConstraint,
    source_space_value: u32,
    dest_space_value: u32,
}

impl Default for TransformSpaceConstraintBase {
    fn default() -> Self {
        Self {
            base: TargetedConstraint::default(),
            source_space_value: 0,
            dest_space_value: 0,
        }
    }
}

impl TransformSpaceConstraintBase {
    pub const TYPE_KEY: u16 = 90;
    pub const SOURCE_SPACE_VALUE_PROPERTY_KEY: u16 = 179;
    pub const DEST_SPACE_VALUE_PROPERTY_KEY: u16 = 180;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn source_space_value(&self) -> u32 {
        self.source_space_value
    }
    pub fn set_source_space_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TransformSpaceConstraintBaseCallbacks,
    ) {
        if !self.set_source_space_value_value(value) {
            return;
        }
        callbacks.source_space_value_changed();
        TransformSpaceConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::SOURCE_SPACE_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_source_space_value_value(&mut self, value: u32) -> bool {
        if self.source_space_value == value {
            return false;
        }
        self.source_space_value = value;
        true
    }
    pub fn dest_space_value(&self) -> u32 {
        self.dest_space_value
    }
    pub fn set_dest_space_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TransformSpaceConstraintBaseCallbacks,
    ) {
        if !self.set_dest_space_value_value(value) {
            return;
        }
        callbacks.dest_space_value_changed();
        TransformSpaceConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::DEST_SPACE_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_dest_space_value_value(&mut self, value: u32) -> bool {
        if self.dest_space_value == value {
            return false;
        }
        self.dest_space_value = value;
        true
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransformSpaceConstraintBaseCallbacks,
    ) {
        self.source_space_value = object.source_space_value;
        self.dest_space_value = object.dest_space_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformSpaceConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SOURCE_SPACE_VALUE_PROPERTY_KEY => {
                self.source_space_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::DEST_SPACE_VALUE_PROPERTY_KEY => {
                self.dest_space_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TransformSpaceConstraintBase {
    type Target = TargetedConstraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TransformSpaceConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
