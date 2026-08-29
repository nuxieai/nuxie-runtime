use crate::mechanical_port::source::{
    constraints::follow_path_constraint::FollowPathConstraint,
    constraints::transform_space_constraint::TransformSpaceConstraint,
    core::binary_reader::BinaryReader,
};

pub trait FollowPathConstraintBaseCallbacks: crate::mechanical_port::source::generated::constraints::transform_space_constraint_base::TransformSpaceConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn distance_changed(&mut self) {}
    fn orient_changed(&mut self) {}
    fn offset_changed(&mut self) {}
}

pub struct FollowPathConstraintBase {
    pub base: TransformSpaceConstraint,
    distance: f32,
    orient: bool,
    offset: bool,
}

impl Default for FollowPathConstraintBase {
    fn default() -> Self {
        Self {
            base: TransformSpaceConstraint::default(),
            distance: 0.0,
            orient: true,
            offset: false,
        }
    }
}

impl FollowPathConstraintBase {
    pub const TYPE_KEY: u16 = 165;
    pub const DISTANCE_PROPERTY_KEY: u16 = 363;
    pub const ORIENT_PROPERTY_KEY: u16 = 364;
    pub const OFFSET_PROPERTY_KEY: u16 = 365;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 90 | 80 | 79 | 10)
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
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) {
        if !self.set_distance_value(value) {
            return;
        }
        callbacks.distance_changed();
        FollowPathConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::DISTANCE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_distance_value(&mut self, value: f32) -> bool {
        if self.distance == value {
            return false;
        }
        self.distance = value;
        true
    }
    pub fn orient(&self) -> bool {
        self.orient
    }
    pub fn set_orient(
        &mut self,
        value: bool,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) {
        if !self.set_orient_value(value) {
            return;
        }
        callbacks.orient_changed();
        FollowPathConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ORIENT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_orient_value(&mut self, value: bool) -> bool {
        if self.orient == value {
            return false;
        }
        self.orient = value;
        true
    }
    pub fn offset(&self) -> bool {
        self.offset
    }
    pub fn set_offset(
        &mut self,
        value: bool,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) {
        if !self.set_offset_value(value) {
            return;
        }
        callbacks.offset_changed();
        FollowPathConstraintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::OFFSET_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_offset_value(&mut self, value: bool) -> bool {
        if self.offset == value {
            return false;
        }
        self.offset = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) -> FollowPathConstraint {
        let mut cloned = FollowPathConstraint::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FollowPathConstraintBaseCallbacks) {
        self.distance = object.distance;
        self.orient = object.orient;
        self.offset = object.offset;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DISTANCE_PROPERTY_KEY => {
                self.distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIENT_PROPERTY_KEY => {
                self.orient = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for FollowPathConstraintBase {
    type Target = TransformSpaceConstraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FollowPathConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
