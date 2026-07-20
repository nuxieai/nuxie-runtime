use crate::records::vec_deque::VecDeque;
use core::ptr;

impl<T: Clone> From<&[T]> for VecDeque<T> {
    #[allow(non_snake_case)]
    fn from(init: &[T]) -> Self {
        let mut dq = VecDeque::<T>::new();
        if !init.is_empty() {
            let buf = dq.allocate(init.len());
            dq.buffer = Some(buf);
            dq.buffer_capacity = init.len();
            dq.queue_size = init.len();
            for (i, item) in init.iter().enumerate() {
                unsafe {
                    ptr::write(buf.as_ptr().add(i), item.clone());
                }
            }
        }
        dq
    }
}
