use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    pub fn size(&self) -> usize {
        self.size_
    }
}
