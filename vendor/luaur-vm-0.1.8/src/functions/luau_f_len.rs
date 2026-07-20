use crate::enums::lua_type::lua_Type;
use crate::macros::setnvalue::setnvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisstring::ttisstring;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_len(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    // The macros ttisstring, tsvalue, and setnvalue depend on lua_Type being in scope
    let _ = lua_Type::LUA_TNIL;

    if nparams >= 1 && nresults <= 1 && ttisstring!(arg0) {
        let ts = tsvalue!(arg0);

        setnvalue!(res, (*ts).len as f64);
        return 1;
    }

    -1
}
