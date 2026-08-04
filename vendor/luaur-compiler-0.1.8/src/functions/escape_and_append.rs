extern crate alloc;

use alloc::vec::Vec;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub(crate) fn escapeAndAppend(buffer: &mut Vec<u8>, r#str: *const c_char, len: usize) {
    let s = unsafe { core::slice::from_raw_parts(r#str as *const u8, len) };

    if s.contains(&b'%') {
        for &character in s {
            buffer.push(character);

            if character == b'%' {
                buffer.push(b'%');
            }
        }
    } else {
        buffer.extend_from_slice(s);
    }
}
