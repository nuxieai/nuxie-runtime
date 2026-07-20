use crate::records::variant::Variant1;

impl<T0> Variant1<T0> {
    /// Port of `Variant::fnCopy<T>`.
    /// In the C++ implementation, this is used as a type-erased function pointer to perform
    /// a placement-new copy of a specific type `T` from `src` to `dst`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it performs raw pointer dereferencing and writes to
    /// uninitialized memory.
    /// - `dst` must be valid for writes and properly aligned for `T`.
    /// - `src` must be valid for reads, properly aligned for `T`, and contain a valid instance of `T`.
    pub unsafe fn variant_fn_copy<T>(dst: *mut core::ffi::c_void, src: *const core::ffi::c_void)
    where
        T: Clone,
    {
        let src_val = &*(src as *const T);
        let dst_ptr = dst as *mut T;
        core::ptr::write(dst_ptr, src_val.clone());
    }
}
