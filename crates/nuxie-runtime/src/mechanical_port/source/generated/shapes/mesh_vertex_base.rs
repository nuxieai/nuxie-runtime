use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, shapes::mesh_vertex::MeshVertex, shapes::vertex::Vertex,
};

pub trait MeshVertexBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn u_changed(&mut self) {}
    fn v_changed(&mut self) {}
}

pub struct MeshVertexBase {
    pub base: Vertex,
    u: f32,
    v: f32,
}

impl Default for MeshVertexBase {
    fn default() -> Self {
        Self {
            base: Vertex::default(),
            u: 0.0,
            v: 0.0,
        }
    }
}

impl MeshVertexBase {
    pub const TYPE_KEY: u16 = 108;
    pub const U_PROPERTY_KEY: u16 = 215;
    pub const V_PROPERTY_KEY: u16 = 216;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 107 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn u(&self) -> f32 {
        self.u
    }
    pub fn set_u(&mut self, value: f32, callbacks: &mut impl MeshVertexBaseCallbacks) {
        if self.u == value {
            return;
        }
        self.u = value;
        callbacks.u_changed();
        callbacks.notify_property_changed(Self::U_PROPERTY_KEY);
    }
    pub fn v(&self) -> f32 {
        self.v
    }
    pub fn set_v(&mut self, value: f32, callbacks: &mut impl MeshVertexBaseCallbacks) {
        if self.v == value {
            return;
        }
        self.v = value;
        callbacks.v_changed();
        callbacks.notify_property_changed(Self::V_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl MeshVertexBaseCallbacks) -> MeshVertex {
        let mut cloned = MeshVertex::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl MeshVertexBaseCallbacks) {
        self.u = object.u;
        self.v = object.v;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl MeshVertexBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::U_PROPERTY_KEY => {
                self.u = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::V_PROPERTY_KEY => {
                self.v = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
