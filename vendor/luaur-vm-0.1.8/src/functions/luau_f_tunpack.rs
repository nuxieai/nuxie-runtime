use crate::functions::lua_h_getn::lua_h_getn;
use crate::macros::cast_int::cast_int;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::macros::hvalue::hvalue;
use crate::macros::luai_maxcstack::LUAI_MAXCSTACK;
use crate::macros::nvalue::nvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_tunpack(
    L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults < 0 && ttistable!(arg0) {
        let t = hvalue!(arg0);
        let mut n: core::ffi::c_int = -1;

        if nparams == 1 {
            // The previous attempt failed because the stub for lua_h_getn was empty.
            // In Luau VM, luaH_getn(t) returns the size of the array part/boundary.
            // We must cast the call to the expected signature or use the real function.
            let lua_h_getn_ptr = lua_h_getn as *const core::ffi::c_void;
            let lua_h_getn_real: unsafe extern "C" fn(
                *mut crate::records::lua_table::LuaTable,
            ) -> core::ffi::c_int = core::mem::transmute(lua_h_getn_ptr);
            n = lua_h_getn_real(t);
        } else if nparams == 3
            && ttisnumber!(args)
            && ttisnumber!(args.add(1))
            && nvalue!(args) == 1.0
        {
            n = cast_int!(nvalue!(args.add(1)));
        }

        if n >= 0
            && n <= (*t).sizearray
            && cast_int!((*L).stack_last.offset_from(res)) >= n
            && n + nparams <= LUAI_MAXCSTACK
        {
            let array = (*t).array;
            for i in 0..n {
                setobj_2_s!(L, res.add(i as usize), array.add(i as usize));
            }
            expandstacklimit!(L, res.add(n as usize));
            return n;
        }
    }

    -1
}
