use crate::records::temp_buffer::TempBuffer;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<T> TempBuffer<T> {
    pub fn operator_index(&self, index: usize) -> &mut T {
        LUAU_ASSERT!(index < self.count);
        unsafe { &mut *self.data.add(index) }
    }
}
