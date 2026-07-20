pub(crate) unsafe fn lmemfind(
    mut s1: *const core::ffi::c_char,
    mut l1: usize,
    s2: *const core::ffi::c_char,
    mut l2: usize,
) -> *const core::ffi::c_char {
    if l2 == 0 {
        s1 // empty strings are everywhere
    } else if l2 > l1 {
        core::ptr::null() // avoids a negative `l1'
    } else {
        let mut init: *const core::ffi::c_char; // to search for a `*s2' inside `s1'
        l2 -= 1; // 1st char will be checked by `memchr'
        l1 = l1 - l2; // `s2' cannot be found after that
        while l1 > 0 {
            init = libc_memchr(s1 as *const core::ffi::c_void, *s2 as core::ffi::c_int, l1)
                as *const core::ffi::c_char;
            if init.is_null() {
                break;
            }
            init = init.add(1); // 1st char is already checked
            if libc_memcmp(
                init as *const core::ffi::c_void,
                s2.add(1) as *const core::ffi::c_void,
                l2,
            ) == 0
            {
                return init.sub(1);
            } else {
                // correct `l1' and `s1' to try again
                l1 -= init.offset_from(s1) as usize;
                s1 = init;
            }
        }
        core::ptr::null() // not found
    }
}

unsafe fn libc_memchr(
    s: *const core::ffi::c_void,
    c: core::ffi::c_int,
    n: usize,
) -> *mut core::ffi::c_void {
    #[cfg(target_arch = "wasm32")]
    {
        let slice = core::slice::from_raw_parts(s as *const u8, n);
        for i in 0..n {
            if slice[i] == c as u8 {
                return s.add(i) as *mut core::ffi::c_void;
            }
        }
        core::ptr::null_mut()
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        extern "C" {
            fn memchr(
                s: *const core::ffi::c_void,
                c: core::ffi::c_int,
                n: usize,
            ) -> *mut core::ffi::c_void;
        }
        memchr(s, c, n)
    }
}

unsafe fn libc_memcmp(
    s1: *const core::ffi::c_void,
    s2: *const core::ffi::c_void,
    n: usize,
) -> core::ffi::c_int {
    #[cfg(target_arch = "wasm32")]
    {
        let slice1 = core::slice::from_raw_parts(s1 as *const u8, n);
        let slice2 = core::slice::from_raw_parts(s2 as *const u8, n);
        for i in 0..n {
            if slice1[i] != slice2[i] {
                return if slice1[i] < slice2[i] { -1 } else { 1 };
            }
        }
        0
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        extern "C" {
            fn memcmp(
                s1: *const core::ffi::c_void,
                s2: *const core::ffi::c_void,
                n: usize,
            ) -> core::ffi::c_int;
        }
        memcmp(s1, s2, n)
    }
}
