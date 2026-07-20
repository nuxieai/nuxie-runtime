#[allow(non_snake_case)]
pub unsafe fn fnVisitR<Visitor, Result, T>(
    vis: &mut Visitor,
    dst: &mut Result,
    src: *mut core::ffi::c_void,
) where
    Visitor: FnMut(&T) -> Result,
{
    // In C++, this function is used by the Variant implementation to invoke a visitor
    // on a specific type T stored within the variant's storage.
    // The src pointer is cast to T* and dereferenced.
    *dst = vis(&*(src as *const T));
}
