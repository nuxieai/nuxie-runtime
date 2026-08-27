pub struct BinaryReader<'a> {
    bytes: &'a [u8],
    position: usize,
    overflowed: bool,
    int_range_error: bool,
}

impl<'a> BinaryReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            overflowed: false,
            int_range_error: false,
        }
    }
    pub fn reached_end(&self) -> bool {
        self.position == self.bytes.len() || self.has_error()
    }
    pub fn length_in_bytes(&self) -> usize {
        self.bytes.len()
    }
    pub fn position(&self) -> &[u8] {
        &self.bytes[self.position..]
    }
    pub fn did_overflow(&self) -> bool {
        self.overflowed
    }
    pub fn did_int_range_error(&self) -> bool {
        self.int_range_error
    }
    pub fn has_error(&self) -> bool {
        self.overflowed || self.int_range_error
    }
    fn overflow(&mut self) {
        self.overflowed = true;
        self.position = self.bytes.len();
    }
    fn int_range_error(&mut self) {
        self.int_range_error = true;
        self.position = self.bytes.len();
    }
    pub fn read_var_uint64(&mut self) -> u64 {
        let mut value = 0u64;
        let mut shift = 0u8;
        loop {
            if self.position >= self.bytes.len() {
                self.overflow();
                return 0;
            }
            let byte = self.bytes[self.position];
            self.position += 1;
            value |= u64::from(byte & 0x7f).wrapping_shl(u32::from(shift));
            if byte & 0x80 == 0 {
                return value;
            }
            shift = shift.wrapping_add(7);
        }
    }
    pub fn read_var_uint_as<T>(&mut self) -> T
    where
        T: TryFrom<u64> + Default,
    {
        match T::try_from(self.read_var_uint64()) {
            Ok(value) => value,
            Err(_) => {
                self.int_range_error();
                T::default()
            }
        }
    }
    pub fn read_string(&mut self) -> String {
        let length = self.read_var_uint64() as usize;
        if self.did_overflow() {
            String::new()
        } else {
            self.read_string_length(length)
        }
    }
    pub fn read_string_length(&mut self, length: usize) -> String {
        let bytes = self.read_bytes_length(length);
        if self.did_overflow() {
            String::new()
        } else {
            String::from_utf8_lossy(bytes).into_owned()
        }
    }
    pub fn read_bytes(&mut self) -> &'a [u8] {
        let length = self.read_var_uint64() as usize;
        if self.did_overflow() {
            &self.bytes[self.position..self.position]
        } else {
            self.read_bytes_length(length)
        }
    }
    pub fn read_bytes_length(&mut self, length: usize) -> &'a [u8] {
        if length > self.bytes.len().saturating_sub(self.position) {
            self.overflow();
            return &self.bytes[self.bytes.len()..];
        }
        let start = self.position;
        self.position += length;
        &self.bytes[start..self.position]
    }
    pub fn read_float32(&mut self) -> f32 {
        f32::from_le_bytes(self.read_array())
    }
    #[cfg(feature = "tools")]
    pub fn read_float64(&mut self) -> f64 {
        f64::from_le_bytes(self.read_array())
    }
    pub fn read_byte(&mut self) -> u8 {
        if self.position >= self.bytes.len() {
            self.overflow();
            0
        } else {
            let value = self.bytes[self.position];
            self.position += 1;
            value
        }
    }
    pub fn read_uint16(&mut self) -> u16 {
        u16::from_le_bytes(self.read_array())
    }
    pub fn read_uint32(&mut self) -> u32 {
        u32::from_le_bytes(self.read_array())
    }
    fn read_array<const N: usize>(&mut self) -> [u8; N] {
        if N > self.bytes.len().saturating_sub(self.position) {
            self.overflow();
            return [0; N];
        }
        let mut value = [0; N];
        value.copy_from_slice(&self.bytes[self.position..self.position + N]);
        self.position += N;
        value
    }
    pub fn reset(&mut self) {
        self.position = 0;
    }
}
