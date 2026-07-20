use crate::enums::lua_type::lua_Type;
use crate::functions::lua_d_pcall::luaD_pcall;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::functions::lua_replace::lua_replace;
use crate::macros::c_call_yield::C_CALL_YIELD;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::macros::isyielded::isyielded;
use crate::macros::lua_callinfo_handle::LUA_CALLINFO_HANDLE;
use crate::macros::savestack::savestack;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_b_xpcally(L: *mut lua_State) -> i32 {
    lua_l_checktype(L, 2, lua_Type::LUA_TFUNCTION as i32);

    // swap function & error function
    lua_pushvalue(L, 1);
    lua_pushvalue(L, 2);
    lua_replace(L, 1);
    lua_replace(L, 2);
    // at this point the stack looks like err, f, args

    // any errors from this point on are handled by continuation
    (*(*L).ci).flags |= LUA_CALLINFO_HANDLE as u32;

    let errf: StkId = (*L).base;
    let func: StkId = (*L).base.add(1);

    let func_offset = savestack!(L, func);
    let errf_offset = savestack!(L, errf);

    // luaB_pcallrun is the internal runner for pcall/xpcall
    use crate::functions::lua_b_pcallrun::lua_b_pcallrun;
    let status = luaD_pcall(
        L,
        Some(lua_b_pcallrun),
        func as *mut core::ffi::c_void,
        func_offset as isize,
        errf_offset as isize,
    );

    // necessary to accommodate functions that return lots of values
    expandstacklimit!(L, (*L).top);

    // yielding means we need to propagate yield; resume will call continuation function later
    if status == 0 && isyielded(L) {
        return C_CALL_YIELD;
    }

    // immediate return (error or success)
    lua_rawcheckstack(L, 1);
    lua_pushboolean(L, (status == 0) as i32);
    lua_replace(L, 1); // replace error function with status
    lua_gettop(L) // return status + all results
}
