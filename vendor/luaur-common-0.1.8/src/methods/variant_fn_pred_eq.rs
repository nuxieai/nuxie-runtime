/// A type-erased equality predicate function pointer type, matching the C++ `bool(*fnPredEq)(const void*, const void*)`.
/// This is used by `Variant` to compare two alternatives of the same type.
pub type FnPredEq =
    unsafe extern "C" fn(*const core::ffi::c_void, *const core::ffi::c_void) -> bool;

/// Returns a function pointer that compares two values of type `T` for equality.
/// This mirrors `Variant::fnPredEq<T>` in C++.
///
/// # Safety
/// - `lhs` and `rhs` must be valid pointers to instances of `T`.
/// - `T` must implement `PartialEq`.
pub unsafe fn variant_fn_pred_eq<T: PartialEq>() -> FnPredEq {
    unsafe extern "C" fn pred_eq<T: PartialEq>(
        lhs: *const core::ffi::c_void,
        rhs: *const core::ffi::c_void,
    ) -> bool {
        let lhs = &*(lhs as *const T);
        let rhs = &*(rhs as *const T);
        lhs == rhs
    }
    pred_eq::<T>
}
