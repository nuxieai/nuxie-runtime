use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    #[inline]
    pub fn operator_index(&self, pos: usize) -> &T {
        LUAU_ASSERT!(pos < self.queue_size);

        unsafe {
            let physical_pos = self.logicalToPhysical(pos);
            // buffer is Option<NonNull<T>>, so we unwrap and offset
            &*self
                .buffer
                .expect("VecDeque buffer is null")
                .as_ptr()
                .add(physical_pos)
        }
    }
}
