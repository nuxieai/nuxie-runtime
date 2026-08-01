/// Storage seam used by [`BinaryWriter`], directly corresponding to C++
/// `BinaryStream`.
pub trait BinaryStream {
    fn write(&mut self, bytes: &[u8]);
    fn flush(&mut self);
    fn clear(&mut self);
}

impl BinaryStream for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }

    fn flush(&mut self) {}

    fn clear(&mut self) {
        Vec::clear(self);
    }
}

/// Byte-compatible writer for Rive primitive binary values.
///
/// All fixed-width values use the little-endian format emitted by the pinned
/// C++ runtime on its supported targets. The stream is flushed when the writer
/// is dropped, matching the C++ destructor.
pub struct BinaryWriter<'a> {
    stream: &'a mut dyn BinaryStream,
}

impl<'a> BinaryWriter<'a> {
    pub fn new(stream: &'a mut dyn BinaryStream) -> Self {
        Self { stream }
    }

    pub fn write_f32(&mut self, value: f32) {
        self.stream.write(&value.to_le_bytes());
    }

    pub fn write_float(&mut self, value: f32) {
        self.write_f32(value);
    }

    pub fn write_f64(&mut self, value: f64) {
        self.stream.write(&value.to_le_bytes());
    }

    pub fn write_double(&mut self, value: f64) {
        self.write_f64(value);
    }

    pub fn write_var_uint64(&mut self, value: u64) {
        self.write_var_uint(value);
    }

    pub fn write_var_uint32(&mut self, value: u32) {
        self.write_var_uint(u64::from(value));
    }

    pub fn write_bytes(&mut self, bytes: &[u8]) {
        if !bytes.is_empty() {
            self.stream.write(bytes);
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.stream.write(&[value]);
    }

    pub fn write_u16(&mut self, value: u16) {
        self.stream.write(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        self.stream.write(&value.to_le_bytes());
    }

    /// Writes a byte-preserving C++ `std::string` value.
    pub fn write_string(&mut self, value: &[u8]) {
        self.write_var_uint64(value.len() as u64);
        self.write_bytes(value);
    }

    pub fn clear(&mut self) {
        self.stream.clear();
    }

    fn write_var_uint(&mut self, mut value: u64) {
        let mut buffer = [0u8; 10];
        let mut length = 0usize;
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            let Some(slot) = buffer.get_mut(length) else {
                // A u64 consumes at most ten LEB128 bytes, so the pinned
                // 16-byte C++ scratch buffer can never reach this branch.
                return;
            };
            *slot = byte;
            length = length.saturating_add(1);
            if value == 0 {
                break;
            }
        }
        self.stream.write(buffer.get(..length).unwrap_or(&buffer));
    }
}

impl Drop for BinaryWriter<'_> {
    fn drop(&mut self) {
        self.stream.flush();
    }
}
