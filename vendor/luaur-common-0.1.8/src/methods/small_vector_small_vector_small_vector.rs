use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn small_vector(&mut self) {
        unsafe {
            core::ptr::write(self, Self::new());
        }
    }
}
