use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    pub fn empty(&self) -> bool {
        self.queue_size == 0
    }
}
