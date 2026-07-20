use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::macros::getstr::getstr;
use crate::macros::lua_c_needs_gc::luaC_needsGC;
use crate::macros::nvalue::nvalue;
use crate::macros::setsvalue::setsvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_sub(
    l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3
        && nresults <= 1
        && ttisstring!(arg0)
        && ttisnumber!(args)
        && ttisnumber!(args.add(1))
    {
        let ts = tsvalue!(arg0);
        let i = nvalue!(args) as i32;
        let j = nvalue!(args.add(1)) as i32;

        if luaC_needsGC!(l) {
            return -1;
        }

        if i >= 1 && j >= i && ((j - 1) as u32) < (*ts).len {
            let str_ptr = getstr(ts);
            let new_ts = luaS_newlstr(l, str_ptr.add((i - 1) as usize), (j - i + 1) as usize);
            setsvalue!(l, res, new_ts);
            return 1;
        }
    }

    -1
}
