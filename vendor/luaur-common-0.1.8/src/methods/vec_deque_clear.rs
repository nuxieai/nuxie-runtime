use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    pub(crate) fn clear_impl(&mut self) {
        self.destroyElements();
        self.head = 0;
        self.queue_size = 0;
    }
}
