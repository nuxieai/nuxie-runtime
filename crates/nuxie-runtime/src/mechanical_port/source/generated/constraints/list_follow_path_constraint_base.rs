use crate::mechanical_port::source::{
    constraints::follow_path_constraint::FollowPathConstraint,
    constraints::list_follow_path_constraint::ListFollowPathConstraint,
    core::binary_reader::BinaryReader,
};

pub trait ListFollowPathConstraintBaseCallbacks: crate::mechanical_port::source::generated::constraints::follow_path_constraint_base::FollowPathConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn distance_end_changed(&mut self) {}
    fn distance_offset_changed(&mut self) {}
}

pub struct ListFollowPathConstraintBase {
    pub base: FollowPathConstraint,
    distance_end: f32,
    distance_offset: f32,
}

impl Default for ListFollowPathConstraintBase {
    fn default() -> Self {
        Self {
            base: FollowPathConstraint::default(),
            distance_end: 1.0,
            distance_offset: 0.0,
        }
    }
}

impl ListFollowPathConstraintBase {
    pub const TYPE_KEY: u16 = 625;
    pub const DISTANCE_END_PROPERTY_KEY: u16 = 888;
    pub const DISTANCE_OFFSET_PROPERTY_KEY: u16 = 889;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 165 | 90 | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn distance_end(&self) -> f32 {
        self.distance_end
    }
    pub fn set_distance_end(
        &mut self,
        value: f32,
        callbacks: &mut impl ListFollowPathConstraintBaseCallbacks,
    ) {
        if !self.set_distance_end_value(value) {
            return;
        }
        callbacks.distance_end_changed();
        callbacks.notify_property_changed(Self::DISTANCE_END_PROPERTY_KEY);
    }

    pub(crate) fn set_distance_end_value(&mut self, value: f32) -> bool {
        if self.distance_end == value {
            return false;
        }
        self.distance_end = value;
        true
    }
    pub fn distance_offset(&self) -> f32 {
        self.distance_offset
    }
    pub fn set_distance_offset(
        &mut self,
        value: f32,
        callbacks: &mut impl ListFollowPathConstraintBaseCallbacks,
    ) {
        if !self.set_distance_offset_value(value) {
            return;
        }
        callbacks.distance_offset_changed();
        callbacks.notify_property_changed(Self::DISTANCE_OFFSET_PROPERTY_KEY);
    }

    pub(crate) fn set_distance_offset_value(&mut self, value: f32) -> bool {
        if self.distance_offset == value {
            return false;
        }
        self.distance_offset = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl ListFollowPathConstraintBaseCallbacks,
    ) -> ListFollowPathConstraint {
        let mut cloned = ListFollowPathConstraint::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl ListFollowPathConstraintBaseCallbacks,
    ) {
        self.distance_end = object.distance_end;
        self.distance_offset = object.distance_offset;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ListFollowPathConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::DISTANCE_END_PROPERTY_KEY => {
                self.distance_end = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::DISTANCE_OFFSET_PROPERTY_KEY => {
                self.distance_offset = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ListFollowPathConstraintBase {
    type Target = FollowPathConstraint;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ListFollowPathConstraintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
