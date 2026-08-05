use crate::enums::lua_type::lua_Type;
use crate::macros::lua_c_init::luaC_init;
use crate::records::lua_state::lua_State;
use crate::records::luau_class::LuauClass;
use crate::records::t_string::TString;

#[allow(non_snake_case)]
pub unsafe fn lua_r_newblankclass(L: *mut lua_State, name: *mut TString) -> *mut LuauClass {
    let classobject = crate::functions::lua_m_newgco::luaM_newgco_(
        L,
        core::mem::size_of::<LuauClass>(),
        (*L).activememcat,
    ) as *mut LuauClass;
    luaC_init!(L, classobject, lua_Type::LUA_TCLASS as core::ffi::c_int);
    (*classobject).name = name;
    (*classobject).staticmembers = core::ptr::null_mut();
    (*classobject).memberstooffset = core::ptr::null_mut();
    (*classobject).offsettomember = core::ptr::null_mut();
    (*classobject).metatable = core::ptr::null_mut();
    (*classobject).instancemetatable = core::ptr::null_mut();
    (*classobject).numberofinstancemembers = 0;
    (*classobject).numberofallmembers = 0;
    classobject
}
