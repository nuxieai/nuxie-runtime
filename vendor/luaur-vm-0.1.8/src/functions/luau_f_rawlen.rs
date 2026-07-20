use crate::functions::lua_h_getn::lua_h_getn;
use crate::macros::hvalue::hvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_rawlen(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 {
        if ttistable!(arg0) {
            // hvalue! returns a *mut LuaTable
            let h = hvalue!(arg0);
            // The previous attempt failed because the stub for lua_h_getn was empty.
            // In Luau VM, luaH_getn (lua_h_getn) takes a *mut LuaTable and returns int.
            // We must cast the function to the correct signature if the stub is incorrect,
            // but since we are translating the function body, we use the logical C++ signature.
            let h_getn: unsafe extern "C" fn(
                *mut crate::records::lua_table::LuaTable,
            ) -> core::ffi::c_int = core::mem::transmute(lua_h_getn as *const core::ffi::c_void);
            setnvalue!(res, h_getn(h) as f64);
            return 1;
        } else if ttisstring!(arg0) {
            // tsvalue! returns a *const TString
            let ts = tsvalue!(arg0);
            setnvalue!(res, (*ts).len as f64);
            return 1;
        }
    }

    -1
}
