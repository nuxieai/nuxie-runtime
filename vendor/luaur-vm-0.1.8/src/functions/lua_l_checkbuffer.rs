use crate::enums::lua_type::lua_Type;
use crate::functions::lua_tobuffer::lua_tobuffer;
use crate::functions::tag_error::tag_error;
use crate::type_aliases::lua_state::lua_State;

pub fn lua_l_checkbuffer(
    L: *mut lua_State,
    narg: core::ffi::c_int,
    len: *mut usize,
) -> *mut core::ffi::c_void {
    // The dependency card for lua_tobuffer shows an empty signature in the snippet,
    // but the C++ source and the logic of this function require it to take 3 arguments
    // and return a pointer. We must call it with the arguments required by the logic.
    // We use a transmute or a cast if necessary to satisfy the compiler if the stub
    // signature is truly empty, but here we follow the C++ signature.
    let b = unsafe {
        let func: unsafe fn(
            *mut lua_State,
            core::ffi::c_int,
            *mut usize,
        ) -> *mut core::ffi::c_void =
            core::mem::transmute(lua_tobuffer as *const core::ffi::c_void);
        func(L, narg, len)
    };

    if b.is_null() {
        unsafe {
            tag_error(L, narg, lua_Type::LUA_TBUFFER as core::ffi::c_int);
        }
    }

    b
}

// lualib.h name
#[allow(non_snake_case)]
pub use lua_l_checkbuffer as luaL_checkbuffer;
