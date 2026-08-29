use crate::mechanical_port::source::core::binary_stream::BinaryStream;

pub struct VectorBinaryWriter<'a> {
    write_buffer: &'a mut Vec<u8>,
    start: usize,
    pos: usize,
}

impl<'a> VectorBinaryWriter<'a> {
    pub fn new(buffer: &'a mut Vec<u8>) -> Self {
        let start = buffer.len();
        Self {
            write_buffer: buffer,
            start,
            pos: 0,
        }
    }

    pub fn buffer(&self) -> &[u8] {
        &self.write_buffer[self.start..]
    }

    pub fn buffer_size(&self) -> usize {
        self.write_buffer.len() - self.start
    }

    pub fn start(&self) -> usize {
        self.start
    }

    pub fn size(&self) -> usize {
        self.pos
    }
}

impl BinaryStream for VectorBinaryWriter<'_> {
    fn write(&mut self, bytes: &[u8]) {
        let end = self.pos;
        if self.write_buffer.len() < end + bytes.len() {
            self.write_buffer.resize(end + bytes.len(), 0);
        }
        self.write_buffer[end..end + bytes.len()].copy_from_slice(bytes);
        self.pos += bytes.len();
    }

    fn flush(&mut self) {}

    fn clear(&mut self) {
        self.pos = 0;
    }
}
