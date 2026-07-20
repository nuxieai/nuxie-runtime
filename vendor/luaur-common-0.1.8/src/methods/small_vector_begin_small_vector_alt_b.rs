use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn begin(&self) -> *const T {
        self.as_slice().as_ptr()
    }
}
