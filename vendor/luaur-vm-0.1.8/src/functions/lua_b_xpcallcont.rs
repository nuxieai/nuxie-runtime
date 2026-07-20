use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_pcall::luaD_pcall;
use crate::functions::lua_d_seterrorobj::luaD_seterrorobj;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::functions::lua_pushvalue::lua_pushvalue;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::functions::lua_replace::lua_replace;
use crate::macros::restorestack::restorestack;
use crate::macros::savestack::savestack;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_b_xpcallcont(L: *mut lua_State, status: i32) -> i32 {
    if status == 0 {
        lua_rawcheckstack(L, 1);
        lua_pushboolean(L, 1);
        lua_replace(L, 1);
        lua_gettop(L)
    } else {
        lua_rawcheckstack(L, 3);
        lua_pushboolean(L, 0);
        lua_pushvalue(L, 1);
        lua_pushvalue(L, -3);

        let errf: StkId = (*L).top.offset(-2);
        let oldtopoffset = savestack!(L, errf);

        // C: `luaD_pcall(L, luaB_xpcallerr, errf, oldtopoffset, 0)` — run the error
        // HANDLER (errf) over the error object. Passing None never invoked it, so the
        // handler result was lost and the raw error + handler fn leaked out, breaking
        // xpcall's error path whenever it ran via this continuation (i.e. in any coroutine).
        let err = luaD_pcall(
            L,
            Some(crate::functions::lua_b_xpcallerr::luaB_xpcallerr),
            errf as *mut core::ffi::c_void,
            oldtopoffset as isize,
            0,
        );

        if err != 0 {
            let mut errstatus = status;

            if status == lua_Status::LUA_ERRMEM as i32 && err == lua_Status::LUA_ERRMEM as i32 {
                errstatus = lua_Status::LUA_ERRMEM as i32;
            } else {
                errstatus = lua_Status::LUA_ERRERR as i32;
            }

            let oldtop = restorestack!(L, oldtopoffset);
            luaD_seterrorobj(L, errstatus, oldtop);
        }

        2
    }
}
