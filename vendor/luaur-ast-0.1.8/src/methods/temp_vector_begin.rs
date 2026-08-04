use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    pub fn begin(&self) -> core::slice::Iter<'_, T> {
        unsafe { (&*self.storage)[self.offset..].iter() }
    }
}

#[allow(non_snake_case)]
pub fn temp_vector_begin<'s, 'a, T>(vector: &'s TempVector<'a, T>) -> core::slice::Iter<'s, T> {
    vector.begin()
}
