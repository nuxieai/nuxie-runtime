#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub(crate) struct lua_jmpbuf {
    pub(crate) prev: *mut lua_jmpbuf,
    pub(crate) status: core::ffi::c_int,
    pub(crate) buf: [core::ffi::c_int; 64],
}

// Note: jmp_buf is a platform-specific array type used for non-local jumps.
// In a wasm32-unknown-unknown or portable context where libc is unavailable,
// we provide a sufficiently sized buffer to satisfy the struct layout for the VM's
// internal longjmp-based error recovery pointers.
