use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader, shapes::mesh::Mesh,
};

pub trait MeshBaseCallbacks {
    fn triangle_index_bytes_changed(&mut self) {}
    fn decode_triangle_index_bytes(&mut self, value: &[u8]);
    fn copy_triangle_index_bytes(&mut self, object: &MeshBase);
}

pub struct MeshBase {
    pub base: ContainerComponent,
}

impl Default for MeshBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
        }
    }
}

impl MeshBase {
    pub const TYPE_KEY: u16 = 109;
    pub const TRIANGLE_INDEX_BYTES_PROPERTY_KEY: u16 = 223;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self, callbacks: &mut impl MeshBaseCallbacks) -> Mesh {
        let mut cloned = Mesh::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl MeshBaseCallbacks) {
        callbacks.copy_triangle_index_bytes(object);
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl MeshBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::TRIANGLE_INDEX_BYTES_PROPERTY_KEY => {
                let value = crate::mechanical_port::source::core::field_types::core_bytes_type::CoreBytesType::deserialize(reader);
                callbacks.decode_triangle_index_bytes(value.as_slice());
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
