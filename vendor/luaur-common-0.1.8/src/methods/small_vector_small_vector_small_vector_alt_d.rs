use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    /// C++ `SmallVector(SmallVector&&)` move-ctor; the Rust port moves by value
    /// and derives `Clone`, so this special member has no call site.
    pub fn small_vector_small_vector_mut(&mut self) {
        unreachable!("C++ SmallVector move-ctor; Rust moves by value / uses Clone — no call site")
    }
}
