use crate::enums::lua_type::lua_Type;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_h_resizearray::lua_h_resizearray;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::moveelements::moveelements;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_isnoneornil::lua_isnoneornil;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn tmove(L: *mut lua_State) -> core::ffi::c_int {
    lua_l_checktype(L, 1, lua_Type::LUA_TTABLE as core::ffi::c_int);
    let f = lua_l_checkinteger(L, 2);
    let e = lua_l_checkinteger(L, 3);
    let t = lua_l_checkinteger(L, 4);
    let tt = if !lua_isnoneornil!(L, 5) { 5 } else { 1 };

    lua_l_checktype(L, tt, lua_Type::LUA_TTABLE as core::ffi::c_int);

    if e >= f {
        luaL_argcheck!(
            L,
            f > 0 || e < core::ffi::c_int::MAX + f,
            3,
            "too many elements to move"
        );
        let n = e - f + 1;
        luaL_argcheck!(
            L,
            t <= core::ffi::c_int::MAX - n + 1,
            4,
            "destination wrap around"
        );

        let dst = hvalue!((*L).base.offset((tt - 1) as isize));

        if (*dst).readonly != 0 {
            lua_g_readonlyerror(L);
        }

        if t > 0 && (t - 1) <= (*dst).sizearray && (t - 1 + n) > (*dst).sizearray {
            lua_h_resizearray(L, dst, t - 1 + n);
        }

        moveelements(L, 1, tt, f, e, t);
    }

    lua_pushvalue(L, tt);
    1
}
