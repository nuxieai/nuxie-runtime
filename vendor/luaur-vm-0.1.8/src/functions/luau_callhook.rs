//! Node: `cxx:Function:Luau.VM:VM/src/lvmexecute.cpp:147:luau_callhook`
//! Source: `VM/src/lvmexecute.cpp:147-200` (hand-ported)

use crate::enums::lua_status::lua_Status;
use crate::functions::lua_g_getline::luaG_getline;
use crate::macros::clvalue::clvalue;
use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::pc_rel::pcRel;
use crate::macros::restorestack::restorestack;
use crate::macros::savestack::savestack;
use crate::records::closure::Closure;
use crate::records::lua_debug::lua_Debug;
use crate::type_aliases::lua_hook::LuaHook;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

/// C++ `LUAU_NOINLINE void luau_callhook(lua_State* L, lua_Hook hook, void* userdata)`.
#[allow(non_snake_case)]
#[inline(never)]
pub unsafe fn luau_callhook(L: *mut lua_State, hook: LuaHook, userdata: *mut core::ffi::c_void) {
    let base = savestack!(L, (*L).base);
    let top = savestack!(L, (*L).top);
    let ci_top = savestack!(L, (*(*L).ci).top);
    let status = (*L).status;

    // if the hook is called externally on a paused thread, we need to make
    // sure the paused thread can emit Luau calls
    if status == lua_Status::LUA_YIELD as u8 || status == lua_Status::LUA_BREAK as u8 {
        (*L).status = 0;
        (*L).base = (*(*L).ci).base;
    }

    let cl = clvalue!((*(*L).ci).func);

    // note: the pc expectations of the hook are matching the general "pc
    // points to next instruction"; however, for the hook to be able to
    // continue execution from the same point, this is called with savedpc at
    // the *current* instruction. this needs to be called before
    // luaD_checkstack in case it fails to reallocate stack
    let oldsavedpc = (*(*L).ci).savedpc;

    if !(*(*L).ci).savedpc.is_null() {
        let code_end = {
            let l = &(*cl).inner.l;
            (*l.p).code.add((*l.p).sizecode as usize)
        };
        if (*(*L).ci).savedpc != code_end {
            (*(*L).ci).savedpc = (*(*L).ci).savedpc.add(1);
        }
    }

    luaD_checkstack!(L, LUA_MINSTACK); // ensure minimum stack size
    (*(*L).ci).top = (*L).top.add(LUA_MINSTACK as usize);
    LUAU_ASSERT!((*(*L).ci).top <= (*L).stack_last);

    let mut ar: lua_Debug = core::mem::zeroed();
    ar.currentline = if (*cl).isC != 0 {
        -1
    } else {
        let p = {
            let l = &(*cl).inner.l;
            l.p
        };
        luaG_getline(p, pcRel!((*(*L).ci).savedpc, p))
    };
    ar.userdata = userdata;

    if let Some(hook) = hook {
        hook(L, &mut ar);
    }

    (*(*L).ci).savedpc = oldsavedpc;

    (*(*L).ci).top = restorestack!(L, ci_top);
    (*L).top = restorestack!(L, top);

    // note that we only restore the paused state if the hook hasn't yielded by itself
    if status == lua_Status::LUA_YIELD as u8 && (*L).status != lua_Status::LUA_YIELD as u8 {
        (*L).status = lua_Status::LUA_YIELD as u8;
        (*L).base = restorestack!(L, base);
    } else if status == lua_Status::LUA_BREAK as u8 {
        LUAU_ASSERT!((*L).status != lua_Status::LUA_BREAK as u8); // hook shouldn't break again

        (*L).status = lua_Status::LUA_BREAK as u8;
        (*L).base = restorestack!(L, base);
    }
}
