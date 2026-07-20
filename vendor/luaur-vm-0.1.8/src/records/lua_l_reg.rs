//! Generated skeleton item.
//! Node: `cxx:Record:Luau.VM:VM/include/lualib.h:11:lua_l_reg`
//! Source: `VM/include/lualib.h`
//! Graph edges:
//! - declared_by: source_file VM/include/lualib.h
//! - source_includes:
//!   - includes -> source_file VM/include/lua.h
//! - incoming:
//!   - declares <- source_file VM/include/lualib.h
//!   - type_ref <- type_alias luaL_Reg (VM/include/lualib.h)
//! - outgoing:
//!   - type_ref -> type_alias luaL_Reg (VM/include/lualib.h)
//!   - type_ref -> type_alias lua_CFunction (VM/include/lua.h)
//!   - translates_to -> rust_item luaL_Reg

#[derive(Debug, Clone, Copy)]
pub struct LuaLReg {
    pub name: *const core::ffi::c_char,
    pub func: crate::type_aliases::lua_c_function::lua_CFunction,
}
