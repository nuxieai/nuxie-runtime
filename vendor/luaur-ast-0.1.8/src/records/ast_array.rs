#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstArray<T> {
    pub data: *mut T,
    pub size: usize,
}

impl<T> AstArray<T> {
    /// The elements as a slice. C++ `AstArray` exposes `begin()`/`end()` over
    /// `[data, data + size)`; the arena keeps the backing storage alive.
    ///
    /// # Safety note
    /// `data`/`size` come from the arena allocator and are always a valid region
    /// (or `data == null` with `size == 0`), so this is sound for live nodes.
    pub fn as_slice(&self) -> &[T] {
        if self.data.is_null() {
            &[]
        } else {
            unsafe { core::slice::from_raw_parts(self.data, self.size) }
        }
    }

    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

impl<'a, T> IntoIterator for &'a AstArray<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

// An empty array (`{nullptr, 0}`) for every `T`, mirroring C++ `AstArray<T>{}`.
// Manual (not derived) so it does not require `T: Default`.
impl<T> Default for AstArray<T> {
    fn default() -> Self {
        AstArray {
            data: core::ptr::null_mut(),
            size: 0,
        }
    }
}
