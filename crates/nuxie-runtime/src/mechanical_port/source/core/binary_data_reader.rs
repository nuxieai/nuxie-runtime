pub struct BinaryDataReader<'a> {
    bytes: &'a mut [u8],
    position: usize,
    end: usize,
    overflowed: bool,
    length: usize,
}
impl<'a> BinaryDataReader<'a> {
    pub fn new(bytes: &'a mut [u8]) -> Self {
        let length = bytes.len();
        Self {
            bytes,
            position: 0,
            end: length,
            overflowed: false,
            length,
        }
    }
    pub fn length_in_bytes(&self) -> usize {
        self.length
    }
    pub fn did_overflow(&self) -> bool {
        self.overflowed
    }
    pub fn is_eof(&self) -> bool {
        self.position >= self.end
    }
    pub fn position(&self) -> &[u8] {
        &self.bytes[self.position..self.end]
    }
    fn overflow(&mut self) {
        self.overflowed = true;
        self.position = self.end;
    }
    pub fn read_var_uint(&mut self) -> u64 {
        let mut value = 0u64;
        for shift in (0..=63).step_by(7) {
            let byte = self.read_byte();
            if self.overflowed {
                return 0;
            }
            if shift == 63 && byte > 1 {
                self.overflow();
                return 0;
            }
            value |= u64::from(byte & 0x7f) << shift;
            if byte & 0x80 == 0 {
                return value;
            }
        }
        self.overflow();
        0
    }
    pub fn read_var_uint32(&mut self) -> u32 {
        let value = self.read_var_uint();
        if value > u32::MAX as u64 {
            self.overflow();
            0
        } else {
            value as u32
        }
    }
    pub fn read_float64(&mut self) -> f64 {
        f64::from_le_bytes(self.read_array())
    }
    pub fn read_float32(&mut self) -> f32 {
        f32::from_le_bytes(self.read_array())
    }
    pub fn read_byte(&mut self) -> u8 {
        if self.position >= self.end {
            self.overflow();
            0
        } else {
            let value = self.bytes[self.position];
            self.position += 1;
            value
        }
    }
    pub fn read_uint32(&mut self) -> u32 {
        u32::from_le_bytes(self.read_array())
    }
    pub fn read_string(&mut self) -> String {
        let length = self.read_var_uint() as usize;
        if self.overflowed || length > self.end.saturating_sub(self.position) {
            self.overflow();
            return String::new();
        }
        let start = self.position;
        self.position += length;
        String::from_utf8_lossy(&self.bytes[start..self.position]).into_owned()
    }
    fn read_array<const N: usize>(&mut self) -> [u8; N] {
        if N > self.end.saturating_sub(self.position) {
            self.overflow();
            return [0; N];
        }
        let mut value = [0; N];
        value.copy_from_slice(&self.bytes[self.position..self.position + N]);
        self.position += N;
        value
    }
    pub fn complete(&mut self, length: usize) {
        self.position = 0;
        self.end = length.min(self.bytes.len());
        self.length = length;
    }
    pub fn reset(&mut self, position: usize) {
        self.position = position.min(self.end);
    }
}
