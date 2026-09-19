use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::lua_d_throw;
use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::type_aliases::lua_state::lua_State;

#[export_name = "luaur_luaD_checkCstack"]
#[allow(non_snake_case)]
pub unsafe fn luaD_checkCstack(L: *mut lua_State) -> crate::records::lua_exception::LuaResult<()> {
    // allow extra stack space to handle stack overflow in xpcall
    let hardlimit: i32 = LUAI_MAXCCALLS + (LUAI_MAXCCALLS >> 3);

    if (*L).nCcalls as i32 == LUAI_MAXCCALLS {
        return crate::functions::lua_g_runerror_l::lua_g_runerror_l(
            L,
            core::ptr::null(),
            format_args!("C stack overflow"),
        );
    } else if (*L).nCcalls as i32 >= hardlimit {
        return lua_d_throw(L, lua_Status::LUA_ERRERR as i32);
    }
    Ok(())
}

#[allow(non_snake_case)]
pub use luaD_checkCstack as lua_d_check_cstack;
