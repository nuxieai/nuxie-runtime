use crate::macros::cast_num::cast_num;
use crate::macros::luai_str_2_num::luai_str2num;

pub fn lua_o_str_2_d(s: *const core::ffi::c_char, result: *mut f64) -> i32 {
    let mut endptr: *mut core::ffi::c_char = core::ptr::null_mut();
    unsafe {
        *result = luai_str2num!(s, &mut endptr);
    }
    if (endptr as *const core::ffi::c_char) == s {
        return 0; // conversion failed
    }
    unsafe {
        if *endptr == b'x' as core::ffi::c_char || *endptr == b'X' as core::ffi::c_char {
            // maybe an hexadecimal constant?
            *result = cast_num!(strtoul(s, &mut endptr, 16));
        }
        if *endptr == b'\0' as core::ffi::c_char {
            return 1; // most common case
        }
        while isspace(*endptr as u8) {
            endptr = endptr.add(1);
        }
        if *endptr != b'\0' as core::ffi::c_char {
            return 0; // invalid trailing characters?
        }
    }
    1
}

// Helper functions for C standard library functions not directly available in core
unsafe fn strtoul(
    s: *const core::ffi::c_char,
    endptr: *mut *mut core::ffi::c_char,
    base: core::ffi::c_int,
) -> core::ffi::c_ulong {
    extern "C" {
        fn strtoul(
            s: *const core::ffi::c_char,
            endptr: *mut *mut core::ffi::c_char,
            base: core::ffi::c_int,
        ) -> core::ffi::c_ulong;
    }
    strtoul(s, endptr, base)
}

// Helper for isspace: check if a u8 value corresponds to an ASCII whitespace character
#[inline]
unsafe fn isspace(c: u8) -> bool {
    match c {
        b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r' => true,
        _ => false,
    }
}
