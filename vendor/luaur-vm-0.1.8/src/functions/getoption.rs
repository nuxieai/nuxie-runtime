use crate::enums::k_option::KOption;
use crate::functions::getnum::getnum;
use crate::functions::getnumlimit::getnumlimit;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::maxalign::MAXALIGN;
use crate::records::header::Header;
use core::ffi::c_char;

pub fn getoption(
    h: *mut Header,
    fmt: *mut *const c_char,
    size: *mut i32,
) -> crate::records::lua_exception::LuaResult<KOption> {
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
            Ok(KOption::Kint)
        }
        'B' => {
            unsafe {
                *size = 1;
            }
            Ok(KOption::Kuint)
        }
        'h' => {
            unsafe {
                *size = 2;
            }
            Ok(KOption::Kint)
        }
        'H' => {
            unsafe {
                *size = 2;
            }
            Ok(KOption::Kuint)
        }
        'l' => {
            unsafe {
                *size = 8;
            }
            Ok(KOption::Kint)
        }
        'L' => {
            unsafe {
                *size = 8;
            }
            Ok(KOption::Kuint)
        }
        'j' => {
            unsafe {
                *size = 4;
            }
            Ok(KOption::Kint)
        }
        'J' => {
            unsafe {
                *size = 4;
            }
            Ok(KOption::Kuint)
        }
        'T' => {
            unsafe {
                *size = 4;
            }
            Ok(KOption::Kuint)
        }
        'f' => {
            unsafe {
                *size = 4;
            }
            Ok(KOption::Kfloat)
        }
        'd' => {
            unsafe {
                *size = 8;
            }
            Ok(KOption::Kfloat)
        }
        'n' => {
            unsafe {
                *size = 8;
            }
            Ok(KOption::Kfloat)
        }
        'i' => {
            unsafe {
                *size = getnumlimit(h, fmt, 4)?;
            }
            Ok(KOption::Kint)
        }
        'I' => {
            unsafe {
                *size = getnumlimit(h, fmt, 4)?;
            }
            Ok(KOption::Kuint)
        }
        's' => {
            unsafe {
                *size = getnumlimit(h, fmt, 4)?;
            }
            Ok(KOption::Kstring)
        }
        'c' => {
            unsafe {
                *size = getnum(h, fmt, -1)?;
            }
            if unsafe { *size } == -1 {
                luaL_error!(unsafe { (*h).L }, "missing size for format option 'c'");
            }
            Ok(KOption::Kchar)
        }
        'z' => Ok(KOption::Kzstr),
        'x' => {
            unsafe {
                *size = 1;
            }
            Ok(KOption::Kpadding)
        }
        'X' => Ok(KOption::Kpaddalign),
        ' ' => Ok(KOption::Knop),
        '<' => {
            unsafe {
                (*h).islittle = 1;
            }
            Ok(KOption::Knop)
        }
        '>' => {
            unsafe {
                (*h).islittle = 0;
            }
            Ok(KOption::Knop)
        }
        '=' => {
            unsafe {
                (*h).islittle = if cfg!(target_endian = "little") { 1 } else { 0 };
            }
            Ok(KOption::Knop)
        }
        '!' => {
            unsafe {
                (*h).maxalign = getnumlimit(h, fmt, MAXALIGN)?;
            }
            Ok(KOption::Knop)
        }
        _ => {
            luaL_error!(
                unsafe { (*h).L },
                "invalid format option '{}'",
                opt as u8 as char
            );
        }
    }
}
