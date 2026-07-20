use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub(crate) fn logicalToPhysical(&self, pos: usize) -> usize {
        let cap = self.capacity();
        if cap == 0 {
            0
        } else {
            (self.head + pos) % cap
        }
    }
}
