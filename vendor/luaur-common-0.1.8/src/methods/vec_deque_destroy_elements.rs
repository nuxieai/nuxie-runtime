use crate::records::vec_deque::VecDeque;
use core::cmp;
use core::ptr;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub(crate) fn destroyElements(&mut self) {
        if let Some(buf) = self.buffer {
            let head_size = cmp::min(self.queue_size, self.capacity().saturating_sub(self.head));
            let tail_size = self.queue_size - head_size;

            unsafe {
                for i in 0..head_size {
                    ptr::drop_in_place(buf.as_ptr().add(self.head + i));
                }
                for i in 0..tail_size {
                    ptr::drop_in_place(buf.as_ptr().add(i));
                }
            }
        }
    }
}
