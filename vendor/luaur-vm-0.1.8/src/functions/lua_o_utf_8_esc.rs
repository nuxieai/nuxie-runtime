#![allow(non_snake_case)]

use crate::macros::cast_to::cast_to;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

/// Number of bytes in a UTF-8 buffer (C++ `#define UTF8BUFFSZ 8`).
const UTF8BUFFSZ: usize = 8;

/// Encodes a Unicode code point into a UTF-8 byte sequence stored in the
/// buffer, starting from the end. Returns the number of bytes used.
pub fn lua_o_utf_8_esc(buff: &mut [core::ffi::c_char; UTF8BUFFSZ], mut x: u32) -> i32 {
    let mut n: i32 = 1; // number of bytes put in buffer (backwards)
    LUAU_ASSERT!(x <= 0x10FFFF);
    if x < 0x80 {
        // ascii?
        buff[UTF8BUFFSZ - 1] = cast_to!(core::ffi::c_char, x);
    } else {
        // need continuation bytes
        let mut mfb: u32 = 0x3f; // maximum that fits in first byte
        loop {
            // add continuation bytes
            buff[UTF8BUFFSZ - (n as usize)] = cast_to!(core::ffi::c_char, 0x80 | (x & 0x3f));
            n += 1;
            x >>= 6; // remove added bits
            mfb >>= 1; // now there is one less bit available in first byte
            if !(x > mfb) {
                break;
            }
        }
        buff[UTF8BUFFSZ - (n as usize)] = cast_to!(core::ffi::c_char, (!mfb << 1) | x);
        // add first byte
    }
    n
}
