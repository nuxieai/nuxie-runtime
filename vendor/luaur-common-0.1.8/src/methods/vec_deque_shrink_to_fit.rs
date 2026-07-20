use crate::records::vec_deque::VecDeque;
use core::cmp;
use core::ptr;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub(crate) fn shrink_to_fit_impl(&mut self) {
        let old_capacity = self.capacity();
        let new_capacity = self.queue_size;

        if old_capacity == new_capacity {
            return;
        }

        let head_size = cmp::min(self.queue_size, old_capacity - self.head);
        let tail_size = self.queue_size - head_size;

        let new_buffer = self.allocate(new_capacity);

        if let Some(old_buf) = self.buffer {
            unsafe {
                if head_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr().add(self.head),
                        new_buffer.as_ptr(),
                        head_size,
                    );
                }

                if tail_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr(),
                        new_buffer.as_ptr().add(head_size),
                        tail_size,
                    );
                }
            }
        }

        self.destroyElements();
        self.deallocate(self.buffer, old_capacity);

        self.buffer = Some(new_buffer);
        self.buffer_capacity = new_capacity;
        self.head = 0;
    }
}
