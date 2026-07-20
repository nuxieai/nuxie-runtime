use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn end(&self) -> *const T {
        let slice = self.as_slice();
        unsafe { slice.as_ptr().add(slice.len()) }
    }
}
