use crate::records::temp_vector::TempVector;

impl<'a, T> Drop for TempVector<'a, T> {
    fn drop(&mut self) {
        let storage = unsafe { &mut *self.storage };
        luaur_common::LUAU_ASSERT!(storage.len() == self.offset + self.size_);
        storage.truncate(self.offset);
    }
}

#[allow(non_snake_case)]
pub fn temp_vector_t_temp_vector() {
    // This is a stub for the destructor method item.
    // The actual implementation is provided in the Drop impl above.
}
