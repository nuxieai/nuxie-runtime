use crate::macros::luai_str_2_long::strtoll;

pub fn lua_o_str_2_l(s: *const core::ffi::c_char, result: *mut i64, base: i32) -> i32 {
    unsafe {
        let mut endptr: *mut core::ffi::c_char = core::ptr::null_mut();

        if base == 10 {
            *result = strtoll(s, &mut endptr, base) as i64;
            if (endptr as *const core::ffi::c_char) == s {
                return 0; // conversion failed
            }
            if *endptr == b'x' as core::ffi::c_char || *endptr == b'X' as core::ffi::c_char {
                // maybe an hexadecimal constant?
                *result = strtoull(s, &mut endptr, 16) as i64;
            }
        } else {
            // unsigned parse in other bases
            *result = strtoull(s, &mut endptr, base as u32) as i64;
            if (endptr as *const core::ffi::c_char) == s {
                return 0;
            }
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

        1
    }
}

// Helper for isspace: check if a u32 value corresponds to an ASCII whitespace character
#[inline]
unsafe fn isspace(c: u8) -> bool {
    match c {
        b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r' => true,
        _ => false,
    }
}

// Helper function for strtoull-like behavior via libc-compatible symbol
unsafe fn strtoull(
    s: *const core::ffi::c_char,
    endptr: &mut *mut core::ffi::c_char,
    base: u32,
) -> u64 {
    extern "C" {
        fn strtoull(
            s: *const core::ffi::c_char,
            endptr: *mut *mut core::ffi::c_char,
            base: u32,
        ) -> u64;
    }
    strtoull(s, endptr as *mut *mut core::ffi::c_char, base)
}
