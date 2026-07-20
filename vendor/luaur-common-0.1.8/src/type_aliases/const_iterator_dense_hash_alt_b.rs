#[allow(non_camel_case_types)]
pub type const_iterator<T> = T;

/*
 * Note: In the original C++ implementation, this is a member type alias within DenseHash<K, V, Hash, Eq>:
 * typedef typename Impl::const_iterator const_iterator;
 *
 * In Rust, this is represented as a generic type alias. Downstream code using DenseHash
 * will reference this via the internal implementation's iterator type.
 */
