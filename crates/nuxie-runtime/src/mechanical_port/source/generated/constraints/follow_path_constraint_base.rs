use crate::mechanical_port::source::{
    constraints::follow_path_constraint::FollowPathConstraint,
    constraints::transform_space_constraint::TransformSpaceConstraint,
    core::binary_reader::BinaryReader,
};

pub trait FollowPathConstraintBaseCallbacks {
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
        if self.distance == value {
            return;
        }
        self.distance = value;
        callbacks.distance_changed();
        callbacks.notify_property_changed(Self::DISTANCE_PROPERTY_KEY);
    }
    pub fn orient(&self) -> bool {
        self.orient
    }
    pub fn set_orient(
        &mut self,
        value: bool,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) {
        if self.orient == value {
            return;
        }
        self.orient = value;
        callbacks.orient_changed();
        callbacks.notify_property_changed(Self::ORIENT_PROPERTY_KEY);
    }
    pub fn offset(&self) -> bool {
        self.offset
    }
    pub fn set_offset(
        &mut self,
        value: bool,
        callbacks: &mut impl FollowPathConstraintBaseCallbacks,
    ) {
        if self.offset == value {
            return;
        }
        self.offset = value;
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
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
