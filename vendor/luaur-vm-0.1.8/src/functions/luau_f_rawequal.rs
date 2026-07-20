use crate::functions::lua_o_rawequal_obj::luaO_rawequalObj;
use crate::macros::luau_fastmath_end::LUAU_FASTMATH_END;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luauF_rawequal(
    _L: *mut LuaState,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    LUAU_FASTMATH_END!();

    if nparams >= 2 && nresults <= 1 {
        // setbvalue(res, luaO_rawequalObj(arg0, args));
        let b = luaO_rawequalObj(arg0 as *const TValue, args as *const TValue);
        let i_o: *mut TValue = res;
        (*i_o).value.b = b;
        (*i_o).tt = crate::enums::lua_type::lua_Type::LUA_TBOOLEAN as i32;
        return 1;
    }

    -1
}
