use crate::records::vec_deque::VecDeque;
use core::ptr::NonNull;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub fn operator_assign(&mut self, other: &VecDeque<T>) -> &mut Self {
        if core::ptr::eq(self, other) {
            return self;
        }

        // destroy all of the existing elements
        self.destroyElements();

        if self.buffer_capacity < other.queue_size {
            // free the current buffer
            self.deallocate(self.buffer, self.buffer_capacity);

            self.buffer = Some(self.allocate(other.buffer_capacity));
            self.buffer_capacity = other.buffer_capacity;
        }

        let head_size = core::cmp::min(other.queue_size, other.buffer_capacity - other.head);
        let tail_size = other.queue_size - head_size;

        // Assignment doesn't try to match the capacity of 'other' and thus makes the buffer contiguous
        self.head = 0;
        self.queue_size = other.queue_size;

        if let Some(dst_ptr) = self.buffer {
            if let Some(src_ptr) = other.buffer {
                unsafe {
                    if head_size != 0 {
                        core::ptr::copy_nonoverlapping(
                            src_ptr.as_ptr().add(other.head),
                            dst_ptr.as_ptr(),
                            head_size,
                        );
                    }

                    if tail_size != 0 {
                        core::ptr::copy_nonoverlapping(
                            src_ptr.as_ptr(),
                            dst_ptr.as_ptr().add(head_size),
                            tail_size,
                        );
                    }
                }
            }
        }

        self
    }
}
