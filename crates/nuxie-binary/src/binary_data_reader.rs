/// Cursor-based reader for Rive's primitive binary wire values.
///
/// This is a direct safe-Rust owner for pinned C++ `BinaryDataReader`. Read
/// failures return the same zero/empty sentinels, set a sticky overflow flag,
/// and move the cursor to EOF.
#[derive(Debug, Clone)]
pub struct BinaryDataReader<'a> {
    bytes: &'a [u8],
    position: usize,
    overflowed: bool,
    length: usize,
}

impl<'a> BinaryDataReader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self {
            bytes,
            position: 0,
            overflowed: false,
            length: bytes.len(),
        }
    }

    pub fn length_in_bytes(&self) -> usize {
        self.length
    }

    pub fn did_overflow(&self) -> bool {
        self.overflowed
    }

    pub fn is_eof(&self) -> bool {
        self.position >= self.bytes.len()
    }

    /// Byte offset corresponding to C++ `position()`.
    pub fn position(&self) -> usize {
        self.position
    }

    fn overflow(&mut self) {
        self.overflowed = true;
        self.position = self.bytes.len();
    }

    pub fn read_var_uint(&mut self) -> u64 {
        let mut result = 0u64;
        let mut shift = 0u32;
        loop {
            let Some(&byte) = self.bytes.get(self.position) else {
                self.overflow();
                return 0;
            };
            self.position = self.position.saturating_add(1);
            result |= u64::from(byte & 0x7f).wrapping_shl(shift);
            if byte & 0x80 == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
        }
    }

    pub fn read_var_uint32(&mut self) -> u32 {
        let mut result = 0u32;
        let mut shift = 0u32;
        loop {
            let Some(&byte) = self.bytes.get(self.position) else {
                self.overflow();
                return 0;
            };
            self.position = self.position.saturating_add(1);
            result |= u32::from(byte & 0x7f).wrapping_shl(shift);
            if byte & 0x80 == 0 {
                return result;
            }
            shift = shift.wrapping_add(7);
        }
    }

    pub fn read_float64(&mut self) -> f64 {
        let Some(bytes) = self.read_array::<8>() else {
            return 0.0;
        };
        f64::from_le_bytes(bytes)
    }

    pub fn read_float32(&mut self) -> f32 {
        let Some(bytes) = self.read_array::<4>() else {
            return 0.0;
        };
        f32::from_le_bytes(bytes)
    }

    pub fn read_byte(&mut self) -> u8 {
        let Some(&byte) = self.bytes.get(self.position) else {
            self.overflow();
            return 0;
        };
        self.position = self.position.saturating_add(1);
        byte
    }

    pub fn read_uint32(&mut self) -> u32 {
        let Some(bytes) = self.read_array::<4>() else {
            return 0;
        };
        u32::from_le_bytes(bytes)
    }

    /// Reads the byte-preserving contents of C++ `std::string`.
    pub fn read_string(&mut self) -> Vec<u8> {
        let length = self.read_var_uint();
        if self.did_overflow() {
            return Vec::new();
        }
        let Ok(length) = usize::try_from(length) else {
            self.overflow();
            return Vec::new();
        };
        let Some(end) = self.position.checked_add(length) else {
            self.overflow();
            return Vec::new();
        };
        let Some(value) = self.bytes.get(self.position..end) else {
            self.overflow();
            return Vec::new();
        };
        self.position = end;
        value.to_vec()
    }

    /// Replaces the backing range without clearing the sticky overflow flag,
    /// matching C++ `complete`.
    pub fn complete(&mut self, bytes: &'a [u8]) {
        self.bytes = bytes;
        self.position = 0;
        self.length = bytes.len();
    }

    /// Rewinds within the current backing range without changing its length or
    /// the sticky overflow flag.
    pub fn reset(&mut self) {
        self.position = 0;
    }

    fn read_array<const N: usize>(&mut self) -> Option<[u8; N]> {
        let Some(end) = self.position.checked_add(N) else {
            self.overflow();
            return None;
        };
        let Some(bytes) = self.bytes.get(self.position..end) else {
            self.overflow();
            return None;
        };
        let Ok(bytes) = bytes.try_into() else {
            self.overflow();
            return None;
        };
        self.position = end;
        Some(bytes)
    }
}
