use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn end_mut(&mut self) -> *mut T {
        let slice = self.as_mut_slice();
        unsafe { slice.as_mut_ptr().add(slice.len()) }
    }
}
