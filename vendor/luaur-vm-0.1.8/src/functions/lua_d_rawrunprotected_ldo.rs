//! Node: `cxx:Function:Luau.VM:VM/src/ldo.cpp:124:lua_d_rawrunprotected`
//! Source: `VM/src/ldo.cpp:124-159` (hand-ported; C++-exceptions build
//! flavor — `luaD_throw` is `panic_any(lua_exception)`, this is the matching
//! `catch_unwind` boundary; see translation/design-cards/lvmexecute.md)

use crate::enums::lua_status::lua_Status;
use crate::functions::lua_g_pusherror::lua_g_pusherror;
use crate::records::lua_exception::lua_exception;
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

    // Silence the default panic-hook noise for the VM's longjmp-emulation
    // unwinds (a caught `lua_exception` is a normal Lua error, not a crash).
    crate::functions::install_lua_exception_panic_hook::install_lua_exception_panic_hook();

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        if let Some(f) = f {
            f(L, ud);
        }
    }));

    if let Err(payload) = result {
        if let Some(e) = payload.downcast_ref::<lua_exception>() {
            // It is assumed/required that the exception caught here was
            // thrown from the same Luau state (see C++ comment).
            LUAU_ASSERT!(e.getThread() == L as *const lua_State);
            status = e.getStatus();
        } else {
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
            // C++ nests a second try/catch for OOM while pushing; a Rust
            // allocation failure aborts, so the LUA_ERRMEM arm has no analog.
            lua_g_pusherror(L, cmsg.as_ptr());
            status = lua_Status::LUA_ERRRUN as core::ffi::c_int;
        }
    }

    status
}

#[allow(unused_imports)]
pub use luaD_rawrunprotected as lua_d_rawrunprotected;
