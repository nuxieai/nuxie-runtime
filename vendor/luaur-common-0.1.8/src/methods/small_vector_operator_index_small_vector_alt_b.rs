use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn operator_index(&self, index: usize) -> &T {
        LUAU_ASSERT!(index < self.size() as usize);

        // The C++ implementation uses `ptr[index]`.
        // In the Rust `SmallVector` record, `ptr` is not a field; instead, the data
        // is either in `storage` (if `heap` is null) or in the `heap` block.
        // Since the fields are private to the record and this is a method impl
        // in a sibling module, we use the public `as_slice` which provides
        // safe access to the active storage.
        &self.as_slice()[index]
    }
}
