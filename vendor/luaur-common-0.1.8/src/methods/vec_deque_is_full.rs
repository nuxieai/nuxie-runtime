use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    pub fn is_full(&self) -> bool {
        self.queue_size == self.capacity()
    }
}
