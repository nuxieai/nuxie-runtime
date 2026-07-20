use crate::enums::lua_status::lua_Status;
use crate::functions::luau_poscall::luau_poscall;
use crate::functions::luau_precall::luau_precall;
use crate::functions::resume_continue::resume_continue;
use crate::macros::curr_func::curr_func;
use crate::macros::isyielded::isyielded;
use crate::macros::lua_callinfo_return::LUA_CALLINFO_RETURN;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::macros::pcrlua::PCRLUA;
use crate::macros::scheduled_reentry::SCHEDULED_REENTRY;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn resume(l: *mut lua_State, ud: *mut core::ffi::c_void) {
    let mut first_arg = ud as StkId;

    if (*l).status == lua_Status::LUA_OK as u8 {
        LUAU_ASSERT!((*l).ci == (*l).base_ci && first_arg >= (*l).base);
        if first_arg == (*l).base {
            crate::functions::lua_g_pusherror::lua_g_pusherror(
                l,
                c"cannot resume dead coroutine".as_ptr(),
            );
            crate::functions::lua_d_throw_ldo::luaD_throw(l, lua_Status::LUA_ERRRUN as i32);
        }

        let precallresult = luau_precall(l, first_arg.offset(-1), LUA_MULTRET);

        if (*l).status == SCHEDULED_REENTRY as u8 {
            first_arg = (*l).base;
        } else {
            if precallresult != PCRLUA {
                return;
            }

            (*(*l).ci).flags |= LUA_CALLINFO_RETURN as u32;
        }
    }

    if (*l).status != lua_Status::LUA_OK as u8 {
        LUAU_ASSERT!(first_arg >= (*l).base);
        LUAU_ASSERT!(isyielded(l));
        (*l).status = lua_Status::LUA_OK as u8;

        let cl = curr_func!(l);

        if (*cl).isC != 0 {
            let c = core::ptr::addr_of!((*cl).inner.c).cast::<crate::records::closure::CClosure>();
            if (*c).cont.is_none() {
                luau_poscall(l, first_arg);
            } else {
                (*l).base = (*(*l).ci).base;
            }
        } else {
            (*l).base = (*(*l).ci).base;
        }
    }

    resume_continue(l);
}
