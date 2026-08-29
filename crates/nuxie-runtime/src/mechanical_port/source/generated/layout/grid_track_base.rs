use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, layout::grid_track::GridTrack,
};

pub trait GridTrackBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn track_value_changed(&mut self) {}
    fn track_max_value_changed(&mut self) {}
    fn collection_changed(&mut self) {}
    fn track_type_changed(&mut self) {}
    fn track_max_type_changed(&mut self) {}
}

pub struct GridTrackBase {
    pub base: Component,
    track_value: f32,
    track_max_value: f32,
    collection: u8,
    track_type: u8,
    track_max_type: u8,
}

impl Default for GridTrackBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            track_value: 0.0,
            track_max_value: 0.0,
            collection: 0,
            track_type: 0,
            track_max_type: 0,
        }
    }
}

impl GridTrackBase {
    pub const TYPE_KEY: u16 = 1058;
    pub const TRACK_VALUE_PROPERTY_KEY: u16 = 1063;
    pub const TRACK_MAX_VALUE_PROPERTY_KEY: u16 = 1065;
    pub const COLLECTION_PROPERTY_KEY: u16 = 1061;
    pub const TRACK_TYPE_PROPERTY_KEY: u16 = 1062;
    pub const TRACK_MAX_TYPE_PROPERTY_KEY: u16 = 1064;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn track_value(&self) -> f32 {
        self.track_value
    }
    pub fn set_track_value(&mut self, value: f32, callbacks: &mut impl GridTrackBaseCallbacks) {
        if !self.set_track_value_value(value) {
            return;
        }
        callbacks.track_value_changed();
        GridTrackBaseCallbacks::notify_property_changed(callbacks, Self::TRACK_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_track_value_value(&mut self, value: f32) -> bool {
        if self.track_value == value {
            return false;
        }
        self.track_value = value;
        true
    }
    pub fn track_max_value(&self) -> f32 {
        self.track_max_value
    }
    pub fn set_track_max_value(&mut self, value: f32, callbacks: &mut impl GridTrackBaseCallbacks) {
        if !self.set_track_max_value_value(value) {
            return;
        }
        callbacks.track_max_value_changed();
        GridTrackBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRACK_MAX_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_track_max_value_value(&mut self, value: f32) -> bool {
        if self.track_max_value == value {
            return false;
        }
        self.track_max_value = value;
        true
    }
    pub fn collection(&self) -> u8 {
        self.collection
    }
    pub fn set_collection(&mut self, value: u8, callbacks: &mut impl GridTrackBaseCallbacks) {
        if !self.set_collection_value(value) {
            return;
        }
        callbacks.collection_changed();
        GridTrackBaseCallbacks::notify_property_changed(callbacks, Self::COLLECTION_PROPERTY_KEY);
    }

    pub(crate) fn set_collection_value(&mut self, value: u8) -> bool {
        if self.collection == value {
            return false;
        }
        self.collection = value;
        true
    }
    pub fn track_type(&self) -> u8 {
        self.track_type
    }
    pub fn set_track_type(&mut self, value: u8, callbacks: &mut impl GridTrackBaseCallbacks) {
        if !self.set_track_type_value(value) {
            return;
        }
        callbacks.track_type_changed();
        GridTrackBaseCallbacks::notify_property_changed(callbacks, Self::TRACK_TYPE_PROPERTY_KEY);
    }

    pub(crate) fn set_track_type_value(&mut self, value: u8) -> bool {
        if self.track_type == value {
            return false;
        }
        self.track_type = value;
        true
    }
    pub fn track_max_type(&self) -> u8 {
        self.track_max_type
    }
    pub fn set_track_max_type(&mut self, value: u8, callbacks: &mut impl GridTrackBaseCallbacks) {
        if !self.set_track_max_type_value(value) {
            return;
        }
        callbacks.track_max_type_changed();
        GridTrackBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TRACK_MAX_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_track_max_type_value(&mut self, value: u8) -> bool {
        if self.track_max_type == value {
            return false;
        }
        self.track_max_type = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl GridTrackBaseCallbacks) -> GridTrack {
        let mut cloned = GridTrack::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl GridTrackBaseCallbacks) {
        self.track_value = object.track_value;
        self.track_max_value = object.track_max_value;
        self.collection = object.collection;
        self.track_type = object.track_type;
        self.track_max_type = object.track_max_type;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl GridTrackBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TRACK_VALUE_PROPERTY_KEY => {
                self.track_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::TRACK_MAX_VALUE_PROPERTY_KEY => {
                self.track_max_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::COLLECTION_PROPERTY_KEY => {
                self.collection = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::TRACK_TYPE_PROPERTY_KEY => {
                self.track_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::TRACK_MAX_TYPE_PROPERTY_KEY => {
                self.track_max_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for GridTrackBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GridTrackBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
