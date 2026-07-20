use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    pub fn is_contiguous(&self) -> bool {
        self.head <= self.capacity() - self.queue_size
    }
}
