use core::ffi::c_char;

#[allow(non_snake_case)]
pub unsafe fn lua_o_chunkid(
    buf: *mut c_char,
    buflen: usize,
    source: *const c_char,
    srclen: usize,
) -> *mut c_char {
    debug_assert!(!buf.is_null());
    debug_assert!(!source.is_null());

    if *source == b'=' as c_char {
        if srclen <= buflen {
            return source.add(1) as *mut c_char;
        }
        // truncate the part after '='
        if buflen == 0 {
            return buf;
        }

        let n = buflen.saturating_sub(1);
        core::ptr::copy_nonoverlapping(source.add(1) as *const u8, buf as *mut u8, n);
        *buf.add(buflen - 1) = b'\0' as c_char;
    } else if *source == b'@' as c_char {
        if srclen <= buflen {
            return source.add(1) as *mut c_char;
        }
        // truncate the part after '@'
        if buflen == 0 {
            return buf;
        }

        // memcpy(buf, "...", 3);
        core::ptr::copy_nonoverlapping(b"..." as *const u8, buf as *mut u8, 3);

        // memcpy(buf + 3, source + srclen - (buflen - 4), buflen - 4);
        let tail_len = buflen.saturating_sub(4);
        let src_start = source.add(srclen.saturating_sub(tail_len));
        core::ptr::copy_nonoverlapping(src_start as *const u8, buf.add(3) as *mut u8, tail_len);

        *buf.add(buflen - 1) = b'\0' as c_char;
    } else {
        // buf = [string "string"]
        let mut len = 0usize;

        // strcspn(source, "\n\r"); // stop at first newline or carriage return
        loop {
            let ch = *source.add(len);
            if ch == b'\n' as c_char || ch == b'\r' as c_char || ch == 0 {
                break;
            }
            len += 1;
        }

        // buflen -= sizeof("[string \"...\"]");
        // In the C++ source, this literal size (excluding NUL terminator) behaves like subtracting 15.
        let mut inner_buflen = buflen.saturating_sub(15);
        if len > inner_buflen {
            len = inner_buflen;
        }

        // strcpy(buf, "[string \"");
        let prefix_bytes = b"[string \"";
        core::ptr::copy_nonoverlapping(prefix_bytes.as_ptr(), buf as *mut u8, prefix_bytes.len());

        let source_end = *source.add(len);
        if source_end != 0 {
            // strncat(buf, source, len);
            if len > 0 {
                core::ptr::copy_nonoverlapping(
                    source as *const u8,
                    buf.add(prefix_bytes.len()) as *mut u8,
                    len,
                );
            }
            // strcat(buf, "...");
            let ell = b"...";
            core::ptr::copy_nonoverlapping(
                ell.as_ptr(),
                buf.add(prefix_bytes.len() + len) as *mut u8,
                3,
            );
            // strcat(buf, "\"]");
            let suffix = b"\"]";
            core::ptr::copy_nonoverlapping(
                suffix.as_ptr(),
                buf.add(prefix_bytes.len() + len + 3) as *mut u8,
                suffix.len(),
            );
            *buf.add(prefix_bytes.len() + len + 3 + suffix.len()) = b'\0' as c_char;
        } else {
            // strcat(buf, source);
            if len > 0 {
                core::ptr::copy_nonoverlapping(
                    source as *const u8,
                    buf.add(prefix_bytes.len()) as *mut u8,
                    len,
                );
            }
            // strcat(buf, "\"]");
            let suffix = b"\"]";
            core::ptr::copy_nonoverlapping(
                suffix.as_ptr(),
                buf.add(prefix_bytes.len() + len) as *mut u8,
                suffix.len(),
            );
            *buf.add(prefix_bytes.len() + len + suffix.len()) = b'\0' as c_char;
        }
    }

    buf
}
