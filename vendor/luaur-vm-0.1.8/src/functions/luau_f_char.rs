use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::macros::lua_c_needs_gc::luaC_needsGC;
use crate::macros::nvalue::nvalue;
use crate::macros::setsvalue::setsvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_char(
    L: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    let mut buffer: [core::ffi::c_char; 8] = [0; 8];

    if nparams < 8 && nresults <= 1 {
        if luaC_needsGC!(L) {
            return -1;
        }

        if nparams >= 1 {
            if !ttisnumber!(arg0) {
                return -1;
            }

            let ch = nvalue!(arg0) as core::ffi::c_int;

            if (ch as u8 as core::ffi::c_int) != ch {
                return -1;
            }

            buffer[0] = ch as core::ffi::c_char;
        }

        for i in 2..=nparams {
            let arg_ptr = args.add((i - 2) as usize);

            if !ttisnumber!(arg_ptr) {
                return -1;
            }

            let ch = nvalue!(arg_ptr) as core::ffi::c_int;

            if (ch as u8 as core::ffi::c_int) != ch {
                return -1;
            }

            buffer[(i - 1) as usize] = ch as core::ffi::c_char;
        }

        buffer[nparams as usize] = 0;

        setsvalue!(L, res, luaS_newlstr(L, buffer.as_ptr(), nparams as usize));
        1
    } else {
        -1
    }
}
