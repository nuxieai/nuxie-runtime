use crate::records::vec_deque::VecDeque;
use core::marker::PhantomData;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub fn new() -> Self {
        Self {
            buffer: None,
            buffer_capacity: 0,
            head: 0,
            queue_size: 0,
            _marker: PhantomData,
        }
    }
}

impl<T> Default for VecDeque<T> {
    fn default() -> Self {
        Self::new()
    }
}
