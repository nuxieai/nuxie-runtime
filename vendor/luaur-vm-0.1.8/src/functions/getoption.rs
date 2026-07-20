use crate::enums::k_option::KOption;
use crate::functions::getnum::getnum;
use crate::functions::getnumlimit::getnumlimit;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::maxalign::MAXALIGN;
use crate::records::header::Header;
use core::ffi::c_char;

pub fn getoption(h: *mut Header, fmt: *mut *const c_char, size: *mut i32) -> KOption {
    let opt = unsafe { **fmt as c_char };
    unsafe {
        *fmt = (*fmt).add(1);
    }
    unsafe {
        *size = 0;
    }

    match opt as u8 as char {
        'b' => {
            unsafe {
                *size = 1;
            }
            KOption::Kint
        }
        'B' => {
            unsafe {
                *size = 1;
            }
            KOption::Kuint
        }
        'h' => {
            unsafe {
                *size = 2;
            }
            KOption::Kint
        }
        'H' => {
            unsafe {
                *size = 2;
            }
            KOption::Kuint
        }
        'l' => {
            unsafe {
                *size = 8;
            }
            KOption::Kint
        }
        'L' => {
            unsafe {
                *size = 8;
            }
            KOption::Kuint
        }
        'j' => {
            unsafe {
                *size = 4;
            }
            KOption::Kint
        }
        'J' => {
            unsafe {
                *size = 4;
            }
            KOption::Kuint
        }
        'T' => {
            unsafe {
                *size = 4;
            }
            KOption::Kuint
        }
        'f' => {
            unsafe {
                *size = 4;
            }
            KOption::Kfloat
        }
        'd' => {
            unsafe {
                *size = 8;
            }
            KOption::Kfloat
        }
        'n' => {
            unsafe {
                *size = 8;
            }
            KOption::Kfloat
        }
        'i' => {
            unsafe {
                *size = getnumlimit(h, fmt, 4);
            }
            KOption::Kint
        }
        'I' => {
            unsafe {
                *size = getnumlimit(h, fmt, 4);
            }
            KOption::Kuint
        }
        's' => {
            unsafe {
                *size = getnumlimit(h, fmt, 4);
            }
            KOption::Kstring
        }
        'c' => {
            unsafe {
                *size = getnum(h, fmt, -1);
            }
            if unsafe { *size } == -1 {
                luaL_error!(unsafe { (*h).L }, "missing size for format option 'c'");
            }
            KOption::Kchar
        }
        'z' => KOption::Kzstr,
        'x' => {
            unsafe {
                *size = 1;
            }
            KOption::Kpadding
        }
        'X' => KOption::Kpaddalign,
        ' ' => KOption::Knop,
        '<' => {
            unsafe {
                (*h).islittle = 1;
            }
            KOption::Knop
        }
        '>' => {
            unsafe {
                (*h).islittle = 0;
            }
            KOption::Knop
        }
        '=' => {
            unsafe {
                (*h).islittle = if cfg!(target_endian = "little") { 1 } else { 0 };
            }
            KOption::Knop
        }
        '!' => {
            unsafe {
                (*h).maxalign = getnumlimit(h, fmt, MAXALIGN);
            }
            KOption::Knop
        }
        _ => {
            luaL_error!(
                unsafe { (*h).L },
                "invalid format option '{}'",
                opt as u8 as char
            );
            unreachable!()
        }
    }
}
