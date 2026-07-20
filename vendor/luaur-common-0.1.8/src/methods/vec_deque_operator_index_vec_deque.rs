use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub fn operator_index_mut(&mut self, pos: usize) -> &mut T {
        LUAU_ASSERT!(pos < self.queue_size);

        let physical_index = self.logicalToPhysical(pos);
        unsafe {
            let ptr = self
                .buffer
                .expect("buffer must be allocated if queue_size > 0")
                .as_ptr();
            &mut *ptr.add(physical_index)
        }
    }
}
