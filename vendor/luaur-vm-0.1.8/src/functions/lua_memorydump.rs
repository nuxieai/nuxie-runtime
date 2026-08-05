use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_category_name::lua_CategoryName;

pub unsafe fn lua_memorydump(
    l: *mut lua_State,
    file: *mut core::ffi::c_void,
    category_name: lua_CategoryName,
) {
    crate::macros::api_check::api_check!(l, !file.is_null());
    crate::functions::lua_c_dump::luaC_dump(l, file, category_name);
}
