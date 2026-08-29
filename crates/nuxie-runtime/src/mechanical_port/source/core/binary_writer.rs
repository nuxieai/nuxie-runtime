use crate::mechanical_port::source::core::binary_stream::BinaryStream;

pub struct BinaryWriter<'a> {
    stream: &'a mut dyn BinaryStream,
}
impl<'a> BinaryWriter<'a> {
    pub fn new(stream: &'a mut dyn BinaryStream) -> Self {
        Self { stream }
    }
    pub fn write_float(&mut self, value: f32) {
        self.stream.write(&value.to_le_bytes());
    }
    pub fn write_double(&mut self, value: f64) {
        self.stream.write(&value.to_le_bytes());
    }
    pub fn write_var_uint(&mut self, mut value: u64) {
        let mut buffer = [0u8; 16];
        let mut index = 0;
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            buffer[index] = byte;
            index += 1;
            if value == 0 {
                break;
            }
        }
        self.stream.write(&buffer[..index]);
    }
    pub fn write_bytes(&mut self, bytes: &[u8]) {
        if !bytes.is_empty() {
            self.stream.write(bytes);
        }
    }
    pub fn write_u8(&mut self, value: u8) {
        self.stream.write(&value.to_ne_bytes());
    }
    pub fn write_u16(&mut self, value: u16) {
        self.stream.write(&value.to_ne_bytes());
    }
    pub fn write_u32(&mut self, value: u32) {
        self.stream.write(&value.to_ne_bytes());
    }
    pub fn write_string(&mut self, value: String) {
        self.write_var_uint(value.len() as u64);
        self.write_bytes(value.as_bytes());
    }
    pub fn clear(&mut self) {
        self.stream.clear();
    }
}
impl Drop for BinaryWriter<'_> {
    fn drop(&mut self) {
        self.stream.flush();
    }
}
