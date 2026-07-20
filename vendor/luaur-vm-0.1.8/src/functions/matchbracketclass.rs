use crate::functions::match_class::match_class;
use crate::macros::l_esc::L_ESC;
use crate::macros::uchar::uchar;

pub fn matchbracketclass(
    c: i32,
    mut p: *const core::ffi::c_char,
    ec: *const core::ffi::c_char,
) -> i32 {
    let mut sig: i32 = 1;

    unsafe {
        if *p.add(1) == b'^' as core::ffi::c_char {
            sig = 0;
            p = p.add(1); // skip the `^'
        }

        while p.add(1) < ec {
            p = p.add(1);

            if *p == L_ESC {
                p = p.add(1);
                if match_class(c, uchar(*p as i32) as i32) != 0 {
                    return sig;
                }
            } else if *p.add(1) == b'-' as core::ffi::c_char && p.add(2) < ec {
                p = p.add(2);
                if (uchar(*(p.offset(-2)) as i32) as i32) <= c && c <= (uchar(*p as i32) as i32) {
                    return sig;
                }
            } else if (uchar(*p as i32) as i32) == c {
                return sig;
            }
        }

        if sig == 0 {
            1
        } else {
            0
        }
    }
}
