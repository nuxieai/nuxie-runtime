#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub union Ftypes {
    pub f: core::ffi::c_float,
    pub d: core::ffi::c_double,
    pub n: core::ffi::c_double,
    pub buff: [core::ffi::c_char; 5 * core::mem::size_of::<core::ffi::c_double>()],
}

impl core::fmt::Debug for Ftypes {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Ftypes").finish_non_exhaustive()
    }
}
