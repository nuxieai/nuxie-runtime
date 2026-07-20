use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    /// C++ `SmallVector& operator=(SmallVector&&)` move-assignment; the Rust port
    /// uses ordinary move/`Clone` assignment, so this special member has no call site.
    pub fn operator_assign_mut(&mut self) {
        unreachable!("C++ SmallVector move-assign; Rust uses move / Clone — no call site")
    }
}
