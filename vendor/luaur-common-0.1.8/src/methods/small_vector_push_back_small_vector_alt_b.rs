use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    /// C++ `void push_back(T&&)` rvalue overload; the hand-ported `SmallVector`
    /// exposes a single by-value `push` (callers move into it), so this
    /// distinct rvalue overload has no call site.
    pub fn push_back_mut(&mut self) {
        unreachable!("C++ SmallVector push_back(T&&) rvalue overload; Rust push takes T by value — no call site")
    }
}
