use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    #[inline]
    pub fn empty(&self) -> bool {
        self.size_ == 0
    }
}
