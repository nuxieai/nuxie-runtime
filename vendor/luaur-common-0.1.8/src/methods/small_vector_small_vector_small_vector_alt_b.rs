use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    /// C++ `SmallVector(std::initializer_list<T>)` ctor; Rust constructs from an
    /// array/iterator (`from_iter`/`vec!`-style) instead, so this ctor overload
    /// has no call site.
    pub fn small_vector_initializer_list_t(&mut self) {
        unreachable!(
            "C++ SmallVector initializer_list ctor; Rust builds from iterator/array — no call site"
        )
    }
}
