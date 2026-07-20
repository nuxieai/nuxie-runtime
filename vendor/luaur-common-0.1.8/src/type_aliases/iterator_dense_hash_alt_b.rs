#[allow(non_camel_case_types)]
pub type iterator<T> = T;

/*
 * Note: In the original C++ source, `iterator` is a member type alias within the `DenseHash` class
 * which points to `Impl::iterator`. In this Rust translation, we provide a generic type alias
 * that represents the iterator type for the corresponding DenseHash implementation.
 *
 * Since this is a type alias for a nested iterator type, downstream Rust code will typically
 * use the iterator type provided by the specific collection implementation (e.g., std::collections::hash_map::Iter).
 */
