use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    pub fn capacity(&self) -> usize {
        self.buffer_capacity
    }
}
