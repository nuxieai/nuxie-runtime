use core::alloc::Layout;
use core::cmp;
use core::marker::PhantomData;
use core::ptr::{self, NonNull};

#[allow(non_snake_case)]
pub struct VecDeque<T> {
    pub(crate) buffer: Option<NonNull<T>>,
    pub(crate) buffer_capacity: usize,
    pub(crate) head: usize,
    pub(crate) queue_size: usize,
    pub(crate) _marker: PhantomData<T>,
}

impl<T> VecDeque<T> {
    /// `VecDeque(std::initializer_list<T>)` — construct from a list, reserving
    /// exactly the element count so the buffer is contiguous with
    /// `capacity == size` (no growth), matching the C++ initializer-list ctor.
    pub fn from_init_list(items: Vec<T>) -> Self {
        let mut q = Self::new();
        if !items.is_empty() {
            q.reserve(items.len());
        }
        for item in items {
            q.push_back(item);
        }
        q
    }

    pub(crate) fn allocate(&self, capacity: usize) -> NonNull<T> {
        if capacity == 0 {
            panic!("Zero capacity allocation");
        }
        let layout = Layout::array::<T>(capacity).unwrap();
        unsafe {
            let ptr = std::alloc::alloc(layout);
            NonNull::new(ptr as *mut T).expect("Allocation failed")
        }
    }

    pub(crate) fn deallocate(&self, ptr: Option<NonNull<T>>, capacity: usize) {
        if let Some(p) = ptr {
            if capacity > 0 {
                let layout = Layout::array::<T>(capacity).unwrap();
                unsafe {
                    std::alloc::dealloc(p.as_ptr() as *mut u8, layout);
                }
            }
        }
    }

    pub(crate) fn grow(&mut self) {
        let old_capacity = self.capacity();
        let new_capacity = if old_capacity > 0 {
            old_capacity * 3 / 2 + 1
        } else {
            4
        };

        if new_capacity > self.max_size() {
            panic!("bad_array_new_length");
        }

        let new_buffer = self.allocate(new_capacity);
        let head_size = cmp::min(self.queue_size, old_capacity - self.head);
        let tail_size = self.queue_size - head_size;

        unsafe {
            if let Some(old_buf) = self.buffer {
                if head_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr().add(self.head),
                        new_buffer.as_ptr(),
                        head_size,
                    );
                }
                if tail_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr(),
                        new_buffer.as_ptr().add(head_size),
                        tail_size,
                    );
                }
            }
        }

        self.deallocate(self.buffer, old_capacity);

        self.buffer = Some(new_buffer);
        self.buffer_capacity = new_capacity;
        self.head = 0;
    }

    pub fn push_back(&mut self, value: T) {
        if self.is_full() {
            self.grow();
        }
        let next_back = self.logicalToPhysical(self.queue_size);
        unsafe {
            ptr::write(self.buffer.unwrap().as_ptr().add(next_back), value);
        }
        self.queue_size += 1;
    }

    pub fn pop_back(&mut self) {
        assert!(!self.empty());
        self.queue_size -= 1;
        let next_back = self.logicalToPhysical(self.queue_size);
        unsafe {
            ptr::drop_in_place(self.buffer.unwrap().as_ptr().add(next_back));
        }
    }

    pub fn push_front(&mut self, value: T) {
        if self.is_full() {
            self.grow();
        }
        self.head = if self.head == 0 {
            self.capacity() - 1
        } else {
            self.head - 1
        };
        unsafe {
            ptr::write(self.buffer.unwrap().as_ptr().add(self.head), value);
        }
        self.queue_size += 1;
    }

    pub fn pop_front(&mut self) {
        assert!(!self.empty());
        unsafe {
            ptr::drop_in_place(self.buffer.unwrap().as_ptr().add(self.head));
        }
        self.head += 1;
        self.queue_size -= 1;
        if self.head == self.capacity() {
            self.head = 0;
        }
    }

    pub fn clear(&mut self) {
        self.destroyElements();
        self.head = 0;
        self.queue_size = 0;
    }

    pub fn reserve(&mut self, new_capacity: usize) {
        if new_capacity > self.max_size() {
            panic!("too large");
        }
        let old_capacity = self.capacity();
        if new_capacity <= old_capacity {
            return;
        }

        let head_size = cmp::min(self.queue_size, old_capacity - self.head);
        let tail_size = self.queue_size - head_size;
        let new_buffer = self.allocate(new_capacity);

        unsafe {
            if let Some(old_buf) = self.buffer {
                if head_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr().add(self.head),
                        new_buffer.as_ptr(),
                        head_size,
                    );
                }
                if tail_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr(),
                        new_buffer.as_ptr().add(head_size),
                        tail_size,
                    );
                }
            }
        }

        self.deallocate(self.buffer, old_capacity);
        self.buffer = Some(new_buffer);
        self.buffer_capacity = new_capacity;
        self.head = 0;
    }

    pub fn shrink_to_fit(&mut self) {
        let old_capacity = self.capacity();
        let new_capacity = self.queue_size;
        if old_capacity == new_capacity {
            return;
        }
        if new_capacity == 0 {
            self.destroyElements();
            self.deallocate(self.buffer, old_capacity);
            self.buffer = None;
            self.buffer_capacity = 0;
            self.head = 0;
            return;
        }

        let head_size = cmp::min(self.queue_size, old_capacity - self.head);
        let tail_size = self.queue_size - head_size;
        let new_buffer = self.allocate(new_capacity);

        unsafe {
            if let Some(old_buf) = self.buffer {
                if head_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr().add(self.head),
                        new_buffer.as_ptr(),
                        head_size,
                    );
                }
                if tail_size != 0 {
                    ptr::copy_nonoverlapping(
                        old_buf.as_ptr(),
                        new_buffer.as_ptr().add(head_size),
                        tail_size,
                    );
                }
            }
        }

        self.deallocate(self.buffer, old_capacity);
        self.buffer = Some(new_buffer);
        self.buffer_capacity = new_capacity;
        self.head = 0;
    }

    pub fn at(&self, pos: usize) -> &T {
        if pos >= self.queue_size {
            panic!("VecDeque out of range");
        }
        unsafe {
            &*self
                .buffer
                .unwrap()
                .as_ptr()
                .add(self.logicalToPhysical(pos))
        }
    }

    pub fn at_mut(&mut self, pos: usize) -> &mut T {
        if pos >= self.queue_size {
            panic!("VecDeque out of range");
        }
        unsafe {
            &mut *self
                .buffer
                .unwrap()
                .as_ptr()
                .add(self.logicalToPhysical(pos))
        }
    }

    pub fn front(&self) -> &T {
        assert!(!self.empty());
        unsafe { &*self.buffer.unwrap().as_ptr().add(self.head) }
    }

    pub fn front_mut(&mut self) -> &mut T {
        assert!(!self.empty());
        unsafe { &mut *self.buffer.unwrap().as_ptr().add(self.head) }
    }

    pub fn back(&self) -> &T {
        assert!(!self.empty());
        let back_idx = self.logicalToPhysical(self.queue_size - 1);
        unsafe { &*self.buffer.unwrap().as_ptr().add(back_idx) }
    }

    pub fn back_mut(&mut self) -> &mut T {
        assert!(!self.empty());
        let back_idx = self.logicalToPhysical(self.queue_size - 1);
        unsafe { &mut *self.buffer.unwrap().as_ptr().add(back_idx) }
    }
}

impl<T> Drop for VecDeque<T> {
    fn drop(&mut self) {
        self.destroyElements();
        self.deallocate(self.buffer, self.buffer_capacity);
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for VecDeque<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "VecDeque[")?;
        let mut first = true;
        for i in 0..self.queue_size {
            if !first {
                write!(f, ", ")?;
            }
            first = false;
            if let Some(buf) = self.buffer {
                let idx = (self.head + i) % self.buffer_capacity;
                unsafe { write!(f, "{:?}", &*buf.as_ptr().add(idx))? };
            }
        }
        write!(f, "]")
    }
}

impl<T: Clone> Clone for VecDeque<T> {
    fn clone(&self) -> Self {
        let mut new_deque = VecDeque::<T>::new();
        if self.buffer_capacity > 0 {
            let new_buffer = new_deque.allocate(self.buffer_capacity);
            new_deque.buffer = Some(new_buffer);
            new_deque.buffer_capacity = self.buffer_capacity;
            new_deque.head = self.head;
            new_deque.queue_size = self.queue_size;

            let head_size = cmp::min(self.queue_size, self.buffer_capacity - self.head);
            let tail_size = self.queue_size - head_size;

            unsafe {
                if let Some(old_buf) = self.buffer {
                    for i in 0..head_size {
                        let val = (&*old_buf.as_ptr().add(self.head + i)).clone();
                        ptr::write(new_buffer.as_ptr().add(self.head + i), val);
                    }
                    for i in 0..tail_size {
                        let val = (&*old_buf.as_ptr().add(i)).clone();
                        ptr::write(new_buffer.as_ptr().add(i), val);
                    }
                }
            }
        }
        new_deque
    }
}
