use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::vec_deque::VecDeque;
use core::ptr::drop_in_place;

impl<T> VecDeque<T> {
    pub(crate) fn pop_back_impl(&mut self) {
        LUAU_ASSERT!(!self.empty());

        self.queue_size -= 1;
        let next_back = self.logicalToPhysical(self.queue_size);

        unsafe {
            if let Some(buffer) = self.buffer {
                drop_in_place(buffer.as_ptr().add(next_back));
            }
        }
    }
}
