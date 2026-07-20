use crate::macros::getstr::getstr;
use crate::type_aliases::t_string::TString;

#[allow(non_snake_case)]
pub fn lua_v_strcmp(ls: *const TString, rs: *const TString) -> core::ffi::c_int {
    if ls == rs {
        return 0;
    }

    unsafe {
        let l = getstr(ls);
        let r = getstr(rs);

        // always safe to read one character because even empty strings are nul terminated
        let bl = *l as u8;
        let br = *r as u8;

        if bl != br {
            return (bl as core::ffi::c_int) - (br as core::ffi::c_int);
        }

        let ll = (*ls).len as usize;
        let lr = (*rs).len as usize;
        let lmin = if ll < lr { ll } else { lr };

        let res = libc::memcmp(
            l as *const core::ffi::c_void,
            r as *const core::ffi::c_void,
            lmin,
        );

        if res != 0 {
            return res;
        }

        if ll == lr {
            0
        } else if ll < lr {
            -1
        } else {
            1
        }
    }
}

mod libc {
    extern "C" {
        pub fn memcmp(
            s1: *const core::ffi::c_void,
            s2: *const core::ffi::c_void,
            n: usize,
        ) -> core::ffi::c_int;
    }
}

#[allow(non_snake_case)]
pub use lua_v_strcmp as luaV_strcmp;
