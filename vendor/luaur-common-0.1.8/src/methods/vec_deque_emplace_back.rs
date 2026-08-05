use crate::records::vec_deque::VecDeque;
use core::ptr;

impl<T> VecDeque<T> {
    pub fn emplace_back(&mut self, value: T) -> &mut T {
        if self.is_full() {
            self.grow();
        }

        let next_back = self.logicalToPhysical(self.queue_size);
        unsafe {
            let slot = self.buffer.unwrap().as_ptr().add(next_back);
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
    fn returns_inserted_back_element() {
        let mut queue = VecDeque::new();
        *queue.emplace_back(3) = 4;
        assert_eq!(*queue.back(), 4);
    }
}
