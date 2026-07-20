use crate::functions::lua_rawgetfield::lua_rawgetfield;
use crate::functions::lua_toboolean::lua_toboolean;
use crate::macros::lua_pop::lua_pop;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};

pub fn getboolfield(L: *mut lua_State, key: &str) -> i32 {
    let key_bytes = key.as_bytes();

    // lua_rawgetfield expects a null-terminated C string key.
    let mut buf = key_bytes.to_vec();
    buf.push(0);
    let key_c: *const c_char = buf.as_ptr() as *const c_char;

    unsafe {
        lua_rawgetfield(L, -1, key_c);

        // We cannot use the lua_isnil! macro because it calls the 0-arity lua_type stub directly,
        // which causes a compilation error. We manually implement the logic here.
        let is_nil = crate::functions::lua_type::lua_type(L, -1)
            == (crate::enums::lua_type::lua_Type::LUA_TNIL as i32);

        let res: c_int = if is_nil { -1 } else { lua_toboolean(L, -1) };

        lua_pop(L, 1);
        res
    }
}
