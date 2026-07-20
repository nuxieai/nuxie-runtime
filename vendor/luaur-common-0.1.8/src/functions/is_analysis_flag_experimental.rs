use core::ffi::CStr;

#[allow(non_snake_case)]
pub fn isAnalysisFlagExperimental(flag: *const core::ffi::c_char) -> bool {
    if flag.is_null() {
        return false;
    }

    // Flags in this list are disabled by default in various command-line tools. They may have behavior that is not fully final,
    // or critical bugs that are found after the code has been submitted. This list is intended _only_ for flags that affect
    // Luau's type checking. Flags that may change runtime behavior (e.g.: parser or VM flags) are not appropriate for this list.
    static K_LIST: &[&[u8]] = &[
        b"LuauInstantiateInSubtyping\0",
        b"LuauFixIndexerSubtypingOrdering\0",
        b"LuauSolverV2\0",
        b"UseNewLuauTypeSolverDefaultEnabled\0",
    ];

    let flag_cstr = unsafe { CStr::from_ptr(flag) };
    let flag_bytes = flag_cstr.to_bytes_with_nul();

    for &item in K_LIST {
        if item == flag_bytes {
            return true;
        }
    }

    false
}
