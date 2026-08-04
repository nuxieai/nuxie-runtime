use core::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, Default)]
pub struct StringRef {
    pub(crate) data: *const i8,
    pub(crate) length: usize,
}

impl StringRef {
    pub fn new(data: *const i8, length: usize) -> Self {
        Self { data, length }
    }

    #[inline]
    fn as_bytes(&self) -> &[u8] {
        if self.data.is_null() {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.data as *const u8, self.length) }
        }
    }
}

// C++ `BytecodeBuilder::StringRef::operator==`:
//   (data && other.data) ? (length == other.length && memcmp(...) == 0) : (data == other.data)
// i.e. content comparison when both pointers are non-null, otherwise raw-pointer
// equality (so the DenseHashMap's {null, 0} empty-key sentinel still matches itself).
// The derived impl compared the raw pointer, so the same string reached via two
// different buffers (e.g. an AstName key vs a string-literal key, both "f") failed to
// dedup and produced duplicate string-table / constant entries.
impl PartialEq for StringRef {
    fn eq(&self, other: &Self) -> bool {
        if !self.data.is_null() && !other.data.is_null() {
            self.length == other.length && self.as_bytes() == other.as_bytes()
        } else {
            self.data == other.data
        }
    }
}

impl Eq for StringRef {}

// C++ `StringRefHash` hashes the content range (`hashRange(data, length)`); equal
// content must hash equally for the dedup map.
impl Hash for StringRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_bytes().hash(state);
    }
}
