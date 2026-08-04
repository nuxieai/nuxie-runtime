use crate::records::allocator::Allocator;

impl Allocator {
    #[allow(non_snake_case)]
    pub fn alloc<T>(&mut self, value: T) -> *mut T {
        // The C++ implementation uses a static_assert to ensure T is trivially destructible
        // because the allocator never calls destructors. In Rust, we don't have an exact
        // equivalent to std::is_trivially_destructible as a stable trait bound, but
        // the contract remains: the caller must be aware that the memory is managed
        // by the Allocator and won't be dropped automatically.

        let ptr = self.allocate(core::mem::size_of::<T>()) as *mut T;
        if !ptr.is_null() {
            unsafe {
                core::ptr::write(ptr, value);
            }
        }
        ptr
    }
}
