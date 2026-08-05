use crate::enums::lua_type::lua_Type;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

// Kept outside LUAU_FASTMATH so the narrowing to f32 is not optimized away.
#[allow(non_snake_case)]
pub unsafe fn luau_f_fround(
    _l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisnumber!(arg0) {
        let value = nvalue!(arg0);
        setnvalue!(res, (value as f32) as f64);
        return 1;
    }

    -1
}
