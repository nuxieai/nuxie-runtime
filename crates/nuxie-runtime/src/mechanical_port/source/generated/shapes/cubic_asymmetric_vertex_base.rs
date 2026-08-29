use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::cubic_asymmetric_vertex::CubicAsymmetricVertex,
    shapes::cubic_vertex::CubicVertex,
};

pub trait CubicAsymmetricVertexBaseCallbacks:
    crate::mechanical_port::source::generated::shapes::vertex_base::VertexBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn rotation_changed(&mut self) {}
    fn in_distance_changed(&mut self) {}
    fn out_distance_changed(&mut self) {}
}

pub struct CubicAsymmetricVertexBase {
    pub base: CubicVertex,
    rotation: f32,
    in_distance: f32,
    out_distance: f32,
}

impl Default for CubicAsymmetricVertexBase {
    fn default() -> Self {
        Self {
            base: CubicVertex::default(),
            rotation: 0.0,
            in_distance: 0.0,
            out_distance: 0.0,
        }
    }
}

impl CubicAsymmetricVertexBase {
    pub const TYPE_KEY: u16 = 34;
    pub const ROTATION_PROPERTY_KEY: u16 = 79;
    pub const IN_DISTANCE_PROPERTY_KEY: u16 = 80;
    pub const OUT_DISTANCE_PROPERTY_KEY: u16 = 81;

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
        callbacks: &mut impl CubicAsymmetricVertexBaseCallbacks,
    ) {
        if !self.set_rotation_value(value) {
            return;
        }
        callbacks.rotation_changed();
        CubicAsymmetricVertexBaseCallbacks::notify_property_changed(
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
    pub fn in_distance(&self) -> f32 {
        self.in_distance
    }
    pub fn set_in_distance(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicAsymmetricVertexBaseCallbacks,
    ) {
        if !self.set_in_distance_value(value) {
            return;
        }
        callbacks.in_distance_changed();
        CubicAsymmetricVertexBaseCallbacks::notify_property_changed(
            callbacks,
            Self::IN_DISTANCE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_in_distance_value(&mut self, value: f32) -> bool {
        if self.in_distance == value {
            return false;
        }
        self.in_distance = value;
        true
    }
    pub fn out_distance(&self) -> f32 {
        self.out_distance
    }
    pub fn set_out_distance(
        &mut self,
        value: f32,
        callbacks: &mut impl CubicAsymmetricVertexBaseCallbacks,
    ) {
        if !self.set_out_distance_value(value) {
            return;
        }
        callbacks.out_distance_changed();
        CubicAsymmetricVertexBaseCallbacks::notify_property_changed(
            callbacks,
            Self::OUT_DISTANCE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_out_distance_value(&mut self, value: f32) -> bool {
        if self.out_distance == value {
            return false;
        }
        self.out_distance = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl CubicAsymmetricVertexBaseCallbacks,
    ) -> CubicAsymmetricVertex {
        let mut cloned = CubicAsymmetricVertex::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl CubicAsymmetricVertexBaseCallbacks) {
        self.rotation = object.rotation;
        self.in_distance = object.in_distance;
        self.out_distance = object.out_distance;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl CubicAsymmetricVertexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ROTATION_PROPERTY_KEY => {
                self.rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::IN_DISTANCE_PROPERTY_KEY => {
                self.in_distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OUT_DISTANCE_PROPERTY_KEY => {
                self.out_distance = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for CubicAsymmetricVertexBase {
    type Target = CubicVertex;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicAsymmetricVertexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
