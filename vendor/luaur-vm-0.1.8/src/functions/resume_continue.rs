use crate::enums::lua_status::lua_Status;
use crate::functions::luau_execute::luau_execute;
use crate::functions::luau_finishop::luau_finishop;
use crate::functions::luau_poscall::luau_poscall;
use crate::macros::curr_func::curr_func;
use crate::macros::lua_callinfo_opyield::LUA_CALLINFO_OPYIELD;
use crate::macros::scheduled_reentry::SCHEDULED_REENTRY;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn resume_continue(L: *mut lua_State) {
    // unroll Luau/C combined stack, processing continuations
    while ((*L).status == lua_Status::LUA_OK as u8 || (*L).status == SCHEDULED_REENTRY as u8)
        && (*L).ci > (*L).base_ci
    {
        LUAU_ASSERT!((*L).baseCcalls == (*L).nCcalls);

        (*L).status = lua_Status::LUA_OK as u8;

        let cl = curr_func!(L);

        if (*cl).isC != 0 {
            // C continuation; we expect this to be followed by Lua continuations
            let c = core::ptr::addr_of!((*cl).inner.c).cast::<crate::records::closure::CClosure>();
            let cont_opt = (*c).cont;
            LUAU_ASSERT!(cont_opt.is_some());

            if let Some(cont) = cont_opt {
                let n = cont(L, 0);

                // continuation can break or yield again
                if (*L).status == lua_Status::LUA_BREAK as u8
                    || (*L).status == lua_Status::LUA_YIELD as u8
                {
                    break;
                }

                luau_poscall(L, (*L).top.offset(-(n as isize)));
            }
        } else {
            if luaur_common::FFlag::LuauYieldIter2.get()
                && ((*(*L).ci).flags & LUA_CALLINFO_OPYIELD as u32) != 0
            {
                luau_finishop(L);
            }

            // Luau continuation; it terminates at the end of the stack or at another C continuation
            luau_execute(L);
        }
    }
}
