use crate::functions::andaux::andaux;
use crate::functions::lua_pushboolean::lua_pushboolean;
use crate::type_aliases::lua_state::lua_State;

pub fn b_test(l: *mut lua_State) -> crate::records::lua_exception::LuaResult<core::ffi::c_int> {
    let r = andaux(l)?;
    unsafe {
        lua_pushboolean(l, if r != 0 { 1 } else { 0 })?;
    }
    Ok(1)
}
