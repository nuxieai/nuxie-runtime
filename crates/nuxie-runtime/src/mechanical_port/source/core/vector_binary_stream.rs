use crate::mechanical_port::source::core::binary_stream::BinaryStream;

#[derive(Default)]
pub struct VectorBinaryStream {
    memory: Vec<u8>,
}

impl VectorBinaryStream {
    pub fn memory(&mut self) -> &mut [u8] {
        &mut self.memory
    }

    pub fn data(&self) -> &[u8] {
        &self.memory
    }
}

impl BinaryStream for VectorBinaryStream {
    fn write(&mut self, bytes: &[u8]) {
        self.memory.extend_from_slice(bytes);
    }

    fn flush(&mut self) {}

    fn clear(&mut self) {
        self.memory.clear();
    }
}
