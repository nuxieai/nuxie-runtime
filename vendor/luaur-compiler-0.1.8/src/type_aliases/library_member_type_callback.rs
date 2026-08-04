pub type LibraryMemberTypeCallback = Option<
    unsafe extern "C" fn(
        library: *const core::ffi::c_char,
        member: *const core::ffi::c_char,
    ) -> core::ffi::c_int,
>;
