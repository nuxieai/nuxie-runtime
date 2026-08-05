use crate::records::vec_deque::VecDeque;
use core::ptr;

impl<T> VecDeque<T> {
    pub fn emplace_front(&mut self, value: T) -> &mut T {
        if self.is_full() {
            self.grow();
        }

        self.head = if self.head == 0 {
            self.capacity() - 1
        } else {
            self.head - 1
        };
        unsafe {
            let slot = self.buffer.unwrap().as_ptr().add(self.head);
            ptr::write(slot, value);
            self.queue_size += 1;
            &mut *slot
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::records::vec_deque::VecDeque;

    #[test]
    fn returns_inserted_front_element() {
        let mut queue = VecDeque::new();
        *queue.emplace_front(3) = 4;
        assert_eq!(*queue.front(), 4);
    }
}
