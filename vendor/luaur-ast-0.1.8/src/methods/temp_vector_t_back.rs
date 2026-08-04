use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    pub fn back(&self) -> &T {
        luaur_common::LUAU_ASSERT!(self.size_ > 0);
        unsafe { &*(*self.storage).as_ptr().add(self.offset + self.size_ - 1) }
    }
}
