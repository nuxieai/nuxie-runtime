//! Qualifies the typed protected-call boundary independently of the still-
//! migrating interpreter. Run with CARGO_PROFILE_DEV_PANIC=abort.
use luaur_vm::enums::lua_status::lua_Status;
use luaur_vm::functions::{
    lua_close::lua_close, lua_d_pcall::luaD_pcall, lua_l_newstate::lua_l_newstate,
    lua_pushstring::lua_pushstring, lua_settop::lua_settop, lua_tolstring::lua_tolstring,
};
use luaur_vm::records::lua_exception::lua_exception;
use luaur_vm::type_aliases::lua_state::lua_State;

unsafe fn fail(state: *mut lua_State, _: *mut core::ffi::c_void) -> Result<(), lua_exception> {
    unsafe {
        lua_pushstring(state, c"returned failure".as_ptr());
        Err(lua_exception::lua_exception(
            state,
            lua_Status::LUA_ERRRUN as i32,
        ))
    }
}

unsafe fn succeed(state: *mut lua_State, _: *mut core::ffi::c_void) -> Result<(), lua_exception> {
    unsafe { lua_pushstring(state, c"recovered".as_ptr()) };
    Ok(())
}

fn main() {
    let state = lua_l_newstate();
    assert!(!state.is_null());
    unsafe {
        let original_ci = (*state).ci;
        let original_calls = (*state).nCcalls;
        for _ in 0..3 {
            let top = luaur_vm::savestack!(state, (*state).top);
            let status = luaD_pcall(state, Some(fail), core::ptr::null_mut(), top, 0);
            assert_eq!(status, lua_Status::LUA_ERRRUN as i32);
            assert_eq!((*state).ci, original_ci);
            assert_eq!((*state).nCcalls, original_calls);
            assert_eq!(
                std::ffi::CStr::from_ptr(lua_tolstring(state, -1, core::ptr::null_mut()))
                    .to_bytes(),
                b"returned failure",
            );
            lua_settop(state, 0);
            let top = luaur_vm::savestack!(state, (*state).top);
            assert_eq!(
                luaD_pcall(state, Some(succeed), core::ptr::null_mut(), top, 0),
                0
            );
            assert_eq!(
                std::ffi::CStr::from_ptr(lua_tolstring(state, -1, core::ptr::null_mut()))
                    .to_bytes(),
                b"recovered",
            );
            lua_settop(state, 0);
        }
        lua_close(state);
    }
    println!("returned-error cleanup and recovery: passed");
}
