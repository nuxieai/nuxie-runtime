use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn begin_mut(&mut self) -> *mut T {
        self.begin() as *mut T
    }
}
