use crate::enums::k_option::KOption;
use crate::functions::getoption::getoption;
use crate::macros::lua_l_argerror::luaL_argerror;
use crate::records::header::Header;
use core::ffi::{c_char, c_int};

pub(crate) unsafe fn getdetails(
    h: *mut Header,
    totalsize: usize,
    fmt: *mut *const c_char,
    psize: *mut c_int,
    ntoalign: *mut c_int,
) -> KOption {
    let opt = getoption(h, fmt, psize);
    let mut align = *psize;

    if opt == KOption::Kpaddalign {
        if **fmt == b'\0' as c_char || getoption(h, fmt, &mut align) == KOption::Kchar || align == 0
        {
            luaL_argerror!((*h).L, 1, "invalid next option for option 'X'");
        }
    }

    if align <= 1 || opt == KOption::Kchar {
        *ntoalign = 0;
    } else {
        if align > (*h).maxalign {
            align = (*h).maxalign;
        }

        if (align & (align - 1)) != 0 {
            luaL_argerror!((*h).L, 1, "format asks for alignment not power of 2");
        }

        *ntoalign = (align - (totalsize as c_int & (align - 1))) & (align - 1);
    }

    opt
}
