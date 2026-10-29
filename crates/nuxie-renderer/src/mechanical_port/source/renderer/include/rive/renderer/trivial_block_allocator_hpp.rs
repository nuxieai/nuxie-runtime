//! renderer/include/rive/renderer/trivial_block_allocator.hpp; retained Rust allocation ownership adapter.
#[derive(Default)]
pub struct TrivialBlockAllocator {
    allocations: Vec<(*mut u8, core::alloc::Layout, Option<unsafe fn(*mut u8)>)>,
}

impl TrivialBlockAllocator {
    pub fn with_capacity_in_bytes(_: usize) -> Self {
        Self::default()
    }

    pub fn make<T>(&mut self, value: T) -> *mut T {
        unsafe fn drop_value<T>(ptr: *mut u8) {
            unsafe { core::ptr::drop_in_place(ptr.cast::<T>()) };
        }

        let boxed = Box::new(value);
        let ptr = Box::into_raw(boxed);
        self.allocations.push((
            ptr.cast(),
            core::alloc::Layout::new::<T>(),
            Some(drop_value::<T>),
        ));
        ptr
    }

    pub unsafe fn makePODArray<T>(&mut self, count: usize) -> *mut T {
        if count == 0 {
            return core::ptr::null_mut();
        }
        let layout = core::alloc::Layout::array::<T>(count).expect("arena POD array layout");
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<T>();
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        self.allocations.push((ptr.cast(), layout, None));
        ptr
    }

    pub fn reset(&mut self) {
        for (ptr, layout, drop_value) in self.allocations.drain(..) {
            if let Some(drop_value) = drop_value {
                unsafe { drop_value(ptr) };
            }
            // `Box::into_raw` may return a dangling-but-aligned sentinel for a
            // zero-sized type. The source arena owns no storage in that case.
            if layout.size() != 0 {
                unsafe { std::alloc::dealloc(ptr, layout) };
            }
        }
    }

    pub fn empty(&self) -> bool {
        self.allocations.is_empty()
    }
}

impl Drop for TrivialBlockAllocator {
    fn drop(&mut self) {
        self.reset();
    }
}

pub struct TrivialArrayAllocator<T: bytemuck::Pod, const ALIGN: usize = 1> {
    allocations: Vec<(*mut T, usize, core::alloc::Layout)>,
}

impl<T: bytemuck::Pod, const ALIGN: usize> TrivialArrayAllocator<T, ALIGN> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            allocations: Vec::with_capacity(capacity.max(1)),
        }
    }
    pub fn alloc(&mut self, count: usize) -> *mut T {
        if count == 0 {
            return core::ptr::null_mut();
        }
        let align = ALIGN.max(core::mem::align_of::<T>());
        assert!(align.is_power_of_two());
        let size = core::mem::size_of::<T>()
            .checked_mul(count)
            .expect("trivial array size");
        let layout =
            core::alloc::Layout::from_size_align(size, align).expect("trivial array alignment");
        let ptr = unsafe { std::alloc::alloc_zeroed(layout) }.cast::<T>();
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        debug_assert_eq!((ptr as usize) & (align - 1), 0);
        self.allocations.push((ptr, count, layout));
        ptr
    }
    pub fn rewindLastAllocation(&mut self, rewind_count: usize) {
        let (_, count, _) = self
            .allocations
            .last_mut()
            .expect("rewind requires an allocation");
        assert!(rewind_count <= *count);
        *count -= rewind_count;
    }
    pub fn reset(&mut self) {
        for (ptr, _, layout) in self.allocations.drain(..) {
            if layout.size() != 0 {
                unsafe { std::alloc::dealloc(ptr.cast(), layout) };
            }
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self.allocations.last_mut() {
            Some((ptr, count, _)) => unsafe { core::slice::from_raw_parts_mut(*ptr, *count) },
            None => &mut [],
        }
    }
}

impl<T: bytemuck::Pod, const ALIGN: usize> Default for TrivialArrayAllocator<T, ALIGN> {
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<T: bytemuck::Pod, const ALIGN: usize> Drop for TrivialArrayAllocator<T, ALIGN> {
    fn drop(&mut self) {
        self.reset()
    }
}

pub struct BlockAllocatedLinkedList<T> {
    nodes: Vec<Box<T>>,
}

impl<T> Default for BlockAllocatedLinkedList<T> {
    fn default() -> Self {
        Self { nodes: Vec::new() }
    }
}

impl<T: super::next_ptr_range_hpp::NextPtrNodeMut> BlockAllocatedLinkedList<T> {
    pub fn push_back(&mut self, value: T) -> *mut T {
        let mut node = Box::new(value);
        let ptr = core::ptr::NonNull::from(node.as_mut());
        if let Some(tail) = self.nodes.last_mut() {
            tail.set_next_ptr(Some(ptr));
        }
        self.nodes.push(node);
        ptr.as_ptr()
    }
    pub fn clear(&mut self) {
        self.nodes.clear();
    }
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
    pub fn tail(&mut self) -> *mut T {
        self.nodes
            .last_mut()
            .map_or(core::ptr::null_mut(), |node| node.as_mut())
    }
    /// # Safety
    /// Nodes reachable through NextPtrNode must be valid and immutable for the
    /// returned borrow. The next chain must terminate at null.
    pub unsafe fn begin(&self) -> super::next_ptr_range_hpp::NextPtrIterator<'_, T> {
        unsafe {
            super::next_ptr_range_hpp::NextPtrIterator::new(
                self.nodes
                    .first()
                    .map_or(core::ptr::null(), |node| node.as_ref()),
            )
        }
    }
    pub fn end(&self) -> super::next_ptr_range_hpp::NextPtrIterator<'_, T> {
        unsafe { super::next_ptr_range_hpp::NextPtrIterator::new(core::ptr::null()) }
    }
    /// # Safety
    /// Nodes reachable through NextPtrNode must be valid and immutable for the
    /// returned borrow. The next chain must terminate at null.
    pub unsafe fn iter(&self) -> impl Iterator<Item = &T> {
        unsafe {
            super::next_ptr_range_hpp::NextPtrRange::new(
                self.nodes
                    .first()
                    .map_or(core::ptr::null(), |node| node.as_ref()),
                core::ptr::null(),
            )
        }
        .into_iter()
    }
}
