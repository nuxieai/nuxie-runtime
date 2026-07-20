use crate::functions::lua_l_checklstring::lua_l_checklstring;
use crate::functions::lua_l_checkvector::lua_l_checkvector;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn vector_index(L: *mut lua_State) -> core::ffi::c_int {
    let v = lua_l_checkvector(L, 1);
    let mut len = 0usize;
    let name = lua_l_checklstring(L, 2, &mut len);

    if len == 1 {
        let ic = (unsafe { *name } as i32 | 0x20) - 'x' as i32;

        #[allow(non_upper_case_globals)]
        const W_OFFSET: i32 = 'w' as i32 - 'x' as i32;

        #[allow(non_upper_case_globals)]
        const X_OFFSET: i32 = 'x' as i32 - 'x' as i32;

        #[allow(non_upper_case_globals)]
        const Y_OFFSET: i32 = 'y' as i32 - 'x' as i32;

        #[allow(non_upper_case_globals)]
        const Z_OFFSET: i32 = 'z' as i32 - 'x' as i32;

        let ic = if ic == W_OFFSET { 3 } else { ic as usize };

        if ic < LUA_VECTOR_SIZE as usize {
            lua_pushnumber(L, (*v.add(ic)) as f64);
            return 1;
        }
    }

    let name_bytes = core::slice::from_raw_parts(name as *const u8, len);
    let name = String::from_utf8_lossy(name_bytes);
    luaL_error!(L, "attempt to index vector with '{}'", name);
    0
}
