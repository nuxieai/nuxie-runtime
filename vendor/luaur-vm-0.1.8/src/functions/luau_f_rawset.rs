use crate::functions::lua_h_set::luaH_set;
use crate::functions::luai_vecisnan::luai_vecisnan;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::luai_numisnan::luai_numisnan;
use crate::macros::nvalue::nvalue;
use crate::macros::setobj::setobj;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttistable::ttistable;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_rawset(
    L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3 && nresults <= 1 && ttistable!(arg0) {
        let key = args;

        if ttisnil!(key) {
            return -1;
        } else if ttisnumber!(key) && luai_numisnan(nvalue!(key) as f64) {
            return -1;
        } else if ttisvector!(key) && luai_vecisnan(vvalue!(key).as_ptr()) {
            return -1;
        }

        let t = hvalue!(arg0);
        if (*t).readonly != 0 {
            return -1;
        }

        setobj!(L, res, arg0);
        let slot = luaH_set(L, t, args);
        setobj!(L, slot, args.add(1));
        luaC_barriert!(L, t, args.add(1));

        return 1;
    }

    -1
}
