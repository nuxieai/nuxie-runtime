use crate::enums::lua_status::lua_Status;
use crate::macros::incr_top::incr_top;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setsvalue::setsvalue;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;
use core::ffi::c_int;

pub fn resume_error(l: *mut lua_State, msg: *const c_char, narg: c_int) -> c_int {
    unsafe {
        // L->top -= narg;
        (*l).top = (*l).top.sub(narg as usize);

        // setsvalue(L, L->top, luaS_new(L, msg));
        // Note: setsvalue! macro expects a pointer to the TValue.
        // (*l).top is a StkId (which is a *mut TValue).
        setsvalue!(l, (*l).top, luaS_new(l, msg));

        // incr_top(L) expands to: { luaD_checkstack(L, 1); L->top++; }
        // We manually perform the incr_top logic here to match the C++ source.

        // stacklimitreached check (simplified for the error-prone macro environment)
        let stack_last = (*l).stack_last as *mut u8;
        let top = (*l).top as *mut u8;
        let limit_reached = (stack_last as usize).wrapping_sub(top as usize)
            <= (1 * core::mem::size_of::<crate::type_aliases::t_value::TValue>());

        if limit_reached {
            crate::functions::lua_d_growstack::lua_d_growstack(l, 1);
        }

        (*l).top = (*l).top.add(1);

        lua_Status::LUA_ERRRUN as c_int
    }
}
