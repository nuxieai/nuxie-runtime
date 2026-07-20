extern crate alloc;

use alloc::vec::Vec as StdVec;

#[allow(non_camel_case_types)]
pub type vec<K, V> = StdVec<(K, V)>;
