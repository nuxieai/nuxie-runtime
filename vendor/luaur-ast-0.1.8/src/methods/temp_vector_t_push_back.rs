use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    #[allow(non_snake_case)]
    pub fn push_back(&mut self, item: T) {
        let storage = unsafe { &mut *self.storage };
        luaur_common::LUAU_ASSERT!(storage.len() == self.offset + self.size_);
        storage.push(item);
        self.size_ += 1;
    }
}
