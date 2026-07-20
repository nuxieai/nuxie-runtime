use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_rawrunprotected_ldo::luaD_rawrunprotected;
use crate::functions::lua_pushstring::lua_pushstring;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::methods::load_context_run::LoadContextRun;
use crate::records::load_context::LoadContext;
use crate::records::scoped_set_gc_threshold::ScopedSetGcThreshold;
use crate::records::temp_buffer::TempBuffer;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luau_load(
    L: *mut lua_State,
    chunkname: *const c_char,
    data: *const c_char,
    size: usize,
    env: c_int,
) -> c_int {
    // we will allocate a fair amount of memory so check GC before we do
    luaC_checkGC!(L);

    // pause GC for the duration of deserialization - some objects we're creating aren't rooted
    let mut pause_gc = ScopedSetGcThreshold {
        global: core::ptr::null_mut(),
        original_threshold: 0,
    };
    pause_gc.scoped_set_gc_threshold_global_state_usize((*L).global, usize::MAX);

    let mut ctx = LoadContext {
        strings: TempBuffer::temp_buffer(),
        protos: TempBuffer::temp_buffer(),
        chunkname,
        data,
        size,
        env,
        result: 0,
    };

    let status = luaD_rawrunprotected(
        L,
        Some(<LoadContext as LoadContextRun>::run),
        &mut ctx as *mut LoadContext as *mut core::ffi::c_void,
    );

    // load can either succeed or get an OOM error, any other errors should be handled internally
    LUAU_ASSERT!(
        status == lua_Status::LUA_OK as c_int || status == lua_Status::LUA_ERRMEM as c_int
    );

    let result = if status == lua_Status::LUA_ERRMEM as c_int {
        lua_pushstring(L, b"not enough memory\0".as_ptr() as *const c_char);
        1
    } else {
        ctx.result
    };

    pause_gc.drop();

    result
}
