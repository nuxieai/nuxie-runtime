use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    pub fn end(&self) -> *const T {
        unsafe { (*self.storage).as_ptr().add(self.offset + self.size_) }
    }
}

#[allow(non_snake_case)]
pub fn temp_vector_end<'a, T>(vector: &TempVector<'a, T>) -> *const T {
    vector.end()
}
