use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::cubic_mirrored_vertex::CubicMirroredVertex,
    shapes::cubic_vertex::CubicVertex,
};

pub trait CubicMirroredVertexBaseCallbacks:
    crate::mechanical_port::source::generated::shapes::vertex_base::VertexBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn rotation_changed(&mut self) {}
    fn distance_changed(&mut self) {}
}

pub struct CubicMirroredVertexBase {
    pub base: CubicVertex,
    rotation: f32,
    distance: f32,
}

impl Default for CubicMirroredVertexBase {
    fn default() -> Self {
        Self {
            base: CubicVertex::default(),
            rotation: 0.0,
            distance: 0.0,
        }
    }
}

impl CubicMirroredVertexBase {
    pub const TYPE_KEY: u16 = 35;
    pub const ROTATION_PROPERTY_KEY: u16 = 82;
    pub const DISTANCE_PROPERTY_KEY: u16 = 83;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 36 | 14 | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    pub fn set_rotation(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicMirroredVertexBaseCallbacks,
    ) {
        if !self.set_rotation_value(value) {
            return;
        }
        callbacks.rotation_changed();
        CubicMirroredVertexBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ROTATION_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_rotation_value(&mut self, value: f32) -> bool {
        if self.rotation == value {
            return false;
        }
        self.rotation = value;
        true
    }
    pub fn distance(&self) -> f32 {
        self.distance
    }
    pub fn set_distance(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicMirroredVertexBaseCallbacks,
    ) {
        if !self.set_distance_value(value) {
            return;
        }
        callbacks.distance_changed();
        CubicMirroredVertexBaseCallbacks::notify_property_changed(
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
    pub fn clone_into(
        &self,
        callbacks: &mut impl CubicMirroredVertexBaseCallbacks,
    ) -> CubicMirroredVertex {
        let mut cloned = CubicMirroredVertex::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl CubicMirroredVertexBaseCallbacks) {
        self.rotation = object.rotation;
        self.distance = object.distance;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl CubicMirroredVertexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ROTATION_PROPERTY_KEY => {
                self.rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::DISTANCE_PROPERTY_KEY => {
                self.distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for CubicMirroredVertexBase {
    type Target = CubicVertex;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicMirroredVertexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
