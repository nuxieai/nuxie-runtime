#[allow(non_upper_case_globals)]
const SPECIALS: &[u8] = b"^$*+?.([%-";

pub fn nospecials(p: *const core::ffi::c_char, l: usize) -> core::ffi::c_int {
    let mut upto: usize = 0;
    loop {
        unsafe {
            let current_p = p.add(upto);
            let s = core::ffi::CStr::from_ptr(current_p);
            let bytes = s.to_bytes();

            // Check if any character in the current null-terminated segment is a special character
            for &b in bytes {
                for &spec in SPECIALS {
                    if b == spec {
                        return 0;
                    }
                }
            }

            upto += bytes.len() + 1; // Move past the segment and its null terminator
        }

        if upto > l {
            break;
        }
    }
    1
}
