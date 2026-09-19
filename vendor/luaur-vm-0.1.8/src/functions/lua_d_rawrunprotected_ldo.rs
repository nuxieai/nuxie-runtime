//! Guest failures return LuaResult, without Rust panic payloads. Host-panic
//! isolation retains the external-exception policy on unwind builds only;
//! host panics still abort on abort builds.

use crate::enums::lua_status::lua_Status;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::pfunc::Pfunc;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaD_rawrunprotected(
    L: *mut lua_State,
    f: Pfunc,
    ud: *mut core::ffi::c_void,
) -> core::ffi::c_int {
    let mut status: core::ffi::c_int = 0;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(f) = f {
            f(L, ud)
        } else {
            Ok(())
        }
    }));

    if let Ok(Err(error)) = &result {
        LUAU_ASSERT!(error.getThread() == L as *const lua_State);
        return error.getStatus();
    }

    if let Err(payload) = result {
        {
            // Luau will never throw this, but this can catch panics that
            // escape from Rust implementations of external functions —
            // the C++ `catch (std::exception&)` arm. Push the message so
            // error handling below can proceed.
            let msg: &str = if let Some(s) = payload.downcast_ref::<&str>() {
                s
            } else if let Some(s) = payload.downcast_ref::<alloc::string::String>() {
                s.as_str()
            } else {
                "unknown error"
            };
            let cmsg = std::ffi::CString::new(msg)
                .unwrap_or_else(|_| std::ffi::CString::new("invalid error message").unwrap());
            // A VM allocation failure while reporting a host panic returns
            // its own status. Rust allocator aborts are not recoverable here.
            if let Err(error) = lua_g_pusherror(L, cmsg.as_ptr()) {
                return error.getStatus();
            }
            status = lua_Status::LUA_ERRRUN as core::ffi::c_int;
        }
    }

    status
}

#[allow(unused_imports)]
pub use luaD_rawrunprotected as lua_d_rawrunprotected;
