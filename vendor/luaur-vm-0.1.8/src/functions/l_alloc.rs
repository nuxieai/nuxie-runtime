pub unsafe extern "C" fn l_alloc(
    ud: *mut core::ffi::c_void,
    ptr: *mut core::ffi::c_void,
    osize: usize,
    nsize: usize,
) -> *mut core::ffi::c_void {
    let _ = ud;
    let _ = osize;

    unsafe {
        if nsize == 0 {
            let _ = ptr;
            if !ptr.is_null() {
                let _ = libc_free(ptr);
            }
            core::ptr::null_mut()
        } else {
            if ptr.is_null() {
                libc_realloc(core::ptr::null_mut(), nsize)
            } else {
                libc_realloc(ptr, nsize)
            }
        }
    }
}

// The C allocator surface. On native targets these `extern "C"` symbols bind
// the platform libc. On `wasm32-unknown-unknown` there is no libc, so they
// resolve at link time to `luaur_common::wasm_libc`'s size-prefixed allocator
// (backed by Rust's global allocator) — the VM allocates real memory in the
// browser rather than the previous null-returning wasm stub.
extern "C" {
    fn free(ptr: *mut core::ffi::c_void);
    fn realloc(ptr: *mut core::ffi::c_void, size: usize) -> *mut core::ffi::c_void;
}

unsafe fn libc_free(ptr: *mut core::ffi::c_void) {
    free(ptr);
}

unsafe fn libc_realloc(ptr: *mut core::ffi::c_void, nsize: usize) -> *mut core::ffi::c_void {
    realloc(ptr, nsize)
}
