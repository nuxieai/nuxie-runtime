use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::vec_deque::VecDeque;
use core::cmp;
use core::ptr;

impl<T> VecDeque<T> {
    #[allow(dead_code)]
    pub(crate) fn vec_deque_grow_impl(&mut self) {
        let old_capacity = self.capacity();

        // we use a growth factor of 1.5x (plus a constant) here in order to enable the
        // previous memory to be reused after a certain number of calls to grow.
        // see: https://github.com/facebook/folly/blob/main/folly/docs/FBVector.md#memory-handling
        let new_capacity = if old_capacity > 0 {
            old_capacity * 3 / 2 + 1
        } else {
            4
        };

        // check that it's a legal allocation
        if new_capacity > self.max_size() {
            panic!("bad_array_new_length");
        }

        // allocate a new backing buffer
        let new_buffer = self.allocate(new_capacity);

        // we should not be growing if the capacity is not the current size
        LUAU_ASSERT!(old_capacity == self.queue_size);

        // how many elements are in the head portion (i.e. from the head to the end of the buffer)
        let head_size = cmp::min(self.queue_size, old_capacity - self.head);
        // how many elements are in the tail portion (i.e. any portion that wrapped to the front)
        let tail_size = self.queue_size - head_size;

        if let Some(old_ptr) = self.buffer {
            unsafe {
                // move the head into the new buffer
                if head_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_ptr.as_ptr().add(self.head),
                        new_buffer.as_ptr(),
                        head_size,
                    );
                }

                // move the tail into the new buffer immediately after
                if tail_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_ptr.as_ptr(),
                        new_buffer.as_ptr().add(head_size),
                        tail_size,
                    );
                }
            }
        }

        // destroy the old elements
        self.destroyElements();

        // deallocate the old buffer
        self.deallocate(self.buffer, old_capacity);

        // set up the queue to be backed by the new buffer
        self.buffer = Some(new_buffer);
        self.buffer_capacity = new_capacity;
        self.head = 0;
    }
}

impl<T> VecDeque<T> {
    // The record file already contains a stub for grow() that calls vec_deque_grow_impl.
    // To avoid E0592 (duplicate definitions), we do not redefine grow() here.
}
