use crate::functions::luaui_clampf::luaui_clampf;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectorclamp(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3
        && nresults <= 1
        && ttisvector!(arg0)
        && ttisvector!(args)
        && ttisvector!(args.offset(1))
    {
        let v = vvalue!(arg0).as_ptr();
        let min = vvalue!(args).as_ptr();
        let max = vvalue!(args.offset(1)).as_ptr();

        if (*min.offset(0) <= *max.offset(0))
            && (*min.offset(1) <= *max.offset(1))
            && (*min.offset(2) <= *max.offset(2))
        {
            if LUA_VECTOR_SIZE == 4 {
                setvvalue!(
                    res,
                    luaui_clampf(*v.offset(0), *min.offset(0), *max.offset(0)),
                    luaui_clampf(*v.offset(1), *min.offset(1), *max.offset(1)),
                    luaui_clampf(*v.offset(2), *min.offset(2), *max.offset(2)),
                    luaui_clampf(*v.offset(3), *min.offset(3), *max.offset(3))
                );
            } else {
                setvvalue!(
                    res,
                    luaui_clampf(*v.offset(0), *min.offset(0), *max.offset(0)),
                    luaui_clampf(*v.offset(1), *min.offset(1), *max.offset(1)),
                    luaui_clampf(*v.offset(2), *min.offset(2), *max.offset(2)),
                    0.0f32
                );
            }

            return 1;
        }
    }

    -1
}
