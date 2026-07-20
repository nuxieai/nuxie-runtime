use crate::records::vec_deque::VecDeque;

impl<T> VecDeque<T> {
    #[allow(non_snake_case)]
    pub fn with_allocator(_alloc: ()) -> Self {
        Self::new()
    }
}
