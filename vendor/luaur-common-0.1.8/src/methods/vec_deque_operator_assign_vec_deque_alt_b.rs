use crate::records::vec_deque::VecDeque;
use core::mem;

impl<T> VecDeque<T> {
    pub fn operator_assign_mut(&mut self, mut other: VecDeque<T>) -> &mut Self {
        // In Rust, the move assignment operator is typically handled by the language,
        // but since this is a manual translation of a C++ move assignment operator
        // that performs explicit resource management, we implement the logic as provided.

        // C++: if (this == &other) return *this;
        // In Rust, we compare the pointers to the struct instances.
        if core::ptr::eq(self, &other) {
            return self;
        }

        // destroy all of the existing elements
        self.destroyElements();

        // free the current buffer
        self.deallocate(self.buffer, self.buffer_capacity);

        // buffer = std::exchange(other.buffer, nullptr);
        self.buffer = mem::replace(&mut other.buffer, None);

        // buffer_capacity = std::exchange(other.buffer_capacity, 0);
        self.buffer_capacity = mem::replace(&mut other.buffer_capacity, 0);

        // head = std::exchange(other.head, 0);
        self.head = mem::replace(&mut other.head, 0);

        // queue_size = std::exchange(other.queue_size, 0);
        self.queue_size = mem::replace(&mut other.queue_size, 0);

        self
    }
}
