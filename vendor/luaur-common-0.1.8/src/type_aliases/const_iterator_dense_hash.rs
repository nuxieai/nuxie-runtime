#[allow(non_camel_case_types)]
pub type const_iterator<T> = T;

/// In the context of Luau's DenseHash, const_iterator is an alias for the underlying
/// implementation's iterator type. In Rust, this is typically represented by the
/// iterator type provided by the collection (e.g., std::collections::hash_map::Iter).
///
/// Since this is a type alias for a template-dependent type in C++, the Rust
/// equivalent is parameterized over the implementation's iterator type.
pub type ConstIterator<'a, I> = I;
