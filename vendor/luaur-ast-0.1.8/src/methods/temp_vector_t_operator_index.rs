use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    pub fn operator_index(&self, index: usize) -> &T {
        luaur_common::LUAU_ASSERT!(index < self.size_);
        unsafe { &*(*self.storage).as_ptr().add(self.offset + index) }
    }
}

impl<'a, T> core::ops::Index<usize> for TempVector<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.operator_index(index)
    }
}
