use crate::records::vec_deque::VecDeque;
use core::mem;

impl<T> VecDeque<T> {
    pub fn max_size(&self) -> usize {
        usize::MAX / mem::size_of::<T>()
    }
}
