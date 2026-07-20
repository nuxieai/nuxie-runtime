#[allow(non_snake_case)]
pub fn match_class(c: core::ffi::c_int, cl: core::ffi::c_int) -> core::ffi::c_int {
    let lower = (cl as u8).to_ascii_lowercase();

    let res = match lower {
        b'a' => (c as u8).is_ascii_alphabetic(),
        b'c' => (c as u8).is_ascii_control(),
        b'd' => (c as u8).is_ascii_digit(),
        b'g' => (c as u8).is_ascii_graphic(),
        b'l' => (c as u8).is_ascii_lowercase(),
        b'p' => (c as u8).is_ascii_punctuation(),
        b's' => (c as u8).is_ascii_whitespace(),
        b'u' => (c as u8).is_ascii_uppercase(),
        b'w' => (c as u8).is_ascii_alphanumeric(),
        b'x' => (c as u8).is_ascii_hexdigit(),
        b'z' => c == 0, // deprecated option
        _ => return if cl == c { 1 } else { 0 },
    };

    if (cl as u8).is_ascii_lowercase() {
        if res {
            1
        } else {
            0
        }
    } else {
        if !res {
            1
        } else {
            0
        }
    }
}
