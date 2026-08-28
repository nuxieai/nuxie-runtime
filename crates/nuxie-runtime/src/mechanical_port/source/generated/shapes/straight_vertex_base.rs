use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::path_vertex::PathVertex,
    shapes::straight_vertex::StraightVertex,
};

pub trait StraightVertexBaseCallbacks:
    crate::mechanical_port::source::generated::shapes::vertex_base::VertexBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn radius_changed(&mut self) {}
}

pub struct StraightVertexBase {
    pub base: PathVertex,
    radius: f32,
}

impl Default for StraightVertexBase {
    fn default() -> Self {
        Self {
            base: PathVertex::default(),
            radius: 0.0,
        }
    }
}

impl StraightVertexBase {
    pub const TYPE_KEY: u16 = 5;
    pub const RADIUS_PROPERTY_KEY: u16 = 26;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 14 | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn radius(&self) -> f32 {
        self.radius
    }
    pub fn set_radius(&mut self, value: f32, callbacks: &mut impl StraightVertexBaseCallbacks) {
        if !self.set_radius_value(value) {
            return;
        }
        callbacks.radius_changed();
        callbacks.notify_property_changed(Self::RADIUS_PROPERTY_KEY);
    }

    pub(crate) fn set_radius_value(&mut self, value: f32) -> bool {
        if self.radius == value {
            return false;
        }
        self.radius = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl StraightVertexBaseCallbacks) -> StraightVertex {
        let mut cloned = StraightVertex::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl StraightVertexBaseCallbacks) {
        self.radius = object.radius;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl StraightVertexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::RADIUS_PROPERTY_KEY => {
                self.radius = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for StraightVertexBase {
    type Target = PathVertex;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for StraightVertexBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
