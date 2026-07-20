use crate::records::scoped_set_gc_threshold::ScopedSetGcThreshold;

impl ScopedSetGcThreshold {
    /// C++ `ScopedSetGCThreshold& operator=(const ScopedSetGCThreshold&) = delete;`
    ///
    /// In Rust, we represent a deleted copy assignment operator by not implementing `Clone` or `Copy`
    /// for the type, and by not providing a public assignment method.
    ///
    /// This item is a stub to satisfy the translation graph for the deleted C++ operator.
    #[allow(dead_code)]
    fn operator_assign(&mut self, _other: &ScopedSetGcThreshold) {
        panic!("ScopedSetGCThreshold copy assignment operator is deleted");
    }
}
