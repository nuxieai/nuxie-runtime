use crate::functions::lua_pushlightuserdatatagged::lua_pushlightuserdatatagged;

#[allow(non_upper_case_globals)]
pub const lua_pushlightuserdata: unsafe fn(*mut core::ffi::c_void, *mut core::ffi::c_void) =
    |l, p| unsafe {
        // The dependency signature in the context was a stub; the real function in Luau
        // takes (lua_State* L, void* p, int tag).
        // We cast the function pointer to the correct signature to call it.
        let func: unsafe fn(*mut core::ffi::c_void, *mut core::ffi::c_void, i32) =
            core::mem::transmute(lua_pushlightuserdatatagged as *const core::ffi::c_void);
        func(l, p, 0)
    };
