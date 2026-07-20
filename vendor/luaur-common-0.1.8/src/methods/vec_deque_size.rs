use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    pub fn size(&self) -> usize {
        self.queue_size
    }
}
