use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_rawrunprotected_ldo::luaD_rawrunprotected;
use crate::functions::lua_d_seterrorobj::luaD_seterrorobj;
use crate::functions::lua_isyieldable::lua_isyieldable;
use crate::functions::resume_findhandler::resume_findhandler;
use crate::functions::resume_handle::resume_handle;
use crate::macros::expandstacklimit::expandstacklimit;
use crate::records::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub unsafe fn resume_finish(l: *mut lua_State, mut status: c_int, oldnCcalls: c_int) -> c_int {
    let mut ch: *mut CallInfo;

    loop {
        ch = resume_findhandler(l);
        if status == lua_Status::LUA_OK as c_int || ch.is_null() {
            break;
        }

        if lua_isyieldable(l) != 0 {
            if let Some(debugprotectederror) = (*(*l).global).cb.debugprotectederror {
                debugprotectederror(l);

                if (*l).status == lua_Status::LUA_BREAK as u8 {
                    status = lua_Status::LUA_OK as c_int;
                    break;
                }
            }
        }

        if luaur_common::FFlag::LuauResumeRestoreCcalls.get() {
            (*l).nCcalls = oldnCcalls as u16;
            (*l).baseCcalls = (*l).nCcalls;
        }

        (*l).status = status as u8;
        status = luaD_rawrunprotected(l, Some(resume_handle), ch as *mut core::ffi::c_void);
    }

    if luaur_common::FFlag::LuauResumeRestoreCcalls.get() {
        (*l).nCcalls = (oldnCcalls - 1) as u16;
    } else {
        (*l).baseCcalls = (*l).baseCcalls.wrapping_sub(1);
        (*l).nCcalls = (*l).baseCcalls;
    }

    (*l).baseCcalls = (*l).nCcalls;
    (*l).isactive = false;

    if status != lua_Status::LUA_OK as c_int {
        (*l).status = status as u8;
        luaD_seterrorobj(l, status, (*l).top);
        (*(*l).ci).top = (*l).top;
    } else if (*l).status == lua_Status::LUA_OK as u8 {
        expandstacklimit!(l, (*l).top);
    }

    (*l).status as c_int
}
