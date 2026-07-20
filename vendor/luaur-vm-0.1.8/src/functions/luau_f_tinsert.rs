use crate::functions::lua_h_getn::lua_h_getn;
use crate::functions::lua_h_setnum::luaH_setnum;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_tinsert(
    L: *mut lua_State,
    _res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams == 2 && nresults <= 0 && ttistable!(arg0) {
        let t = hvalue!(arg0);
        if (*t).readonly != 0 {
            return -1;
        }

        let pos = lua_h_getn(t) + 1;
        let slot = luaH_setnum(L, t, pos);
        setobj2t!(L, slot, args);
        luaC_barriert!(L, t, args);
        return 0;
    }

    -1
}
