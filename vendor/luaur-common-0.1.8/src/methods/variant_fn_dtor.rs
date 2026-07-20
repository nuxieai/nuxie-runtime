/// A type-erased destructor function pointer type, matching the C++ `void(*fnDtor)(void*)`.
/// This is used by `Variant` to destroy its active alternative.
pub type FnDtor = unsafe extern "C" fn(*mut core::ffi::c_void);

/// Returns a function pointer that destroys the active alternative of a `Variant` by
/// invoking its destructor. This mirrors `Variant::fnDtor<T>` in C++.
///
/// # Safety
/// - `dst` must be a valid pointer to a `T` currently held by a `Variant`.
/// - The caller must ensure `dst` is not null and points to a live `T`.
pub unsafe fn variant_fn_dtor<T>() -> FnDtor {
    unsafe extern "C" fn dtor<T>(dst: *mut core::ffi::c_void) {
        let ptr = dst as *mut T;
        core::ptr::drop_in_place(ptr);
    }
    dtor::<T>
}
