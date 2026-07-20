use crate::functions::luaui_signf::luaui_signf;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectorsign(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    _args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 1 && nresults <= 1 && ttisvector!(arg0) {
        let v = vvalue!(arg0).as_ptr();

        if LUA_VECTOR_SIZE == 4 {
            setvvalue!(
                res,
                luaui_signf(*v.offset(0)),
                luaui_signf(*v.offset(1)),
                luaui_signf(*v.offset(2)),
                luaui_signf(*v.offset(3))
            );
        } else {
            setvvalue!(
                res,
                luaui_signf(*v.offset(0)),
                luaui_signf(*v.offset(1)),
                luaui_signf(*v.offset(2)),
                0.0f32
            );
        }

        return 1;
    }

    -1
}
