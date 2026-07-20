//! Generated skeleton item.
//! Node: `cxx:TypeAlias:Luau.VM:VM/include/lualib.h:16:lua_l_reg`
//! Source: `VM/include/lualib.h`
//! Graph edges:
//! - declared_by: source_file VM/include/lualib.h
//! - source_includes:
//!   - includes -> source_file VM/include/lua.h
//! - incoming:
//!   - declares <- source_file VM/include/lualib.h
//!   - type_ref <- record luaL_Reg (VM/include/lualib.h)
//! - outgoing:
//!   - type_ref -> record luaL_Reg (VM/include/lualib.h)
//!   - translates_to -> rust_item luaL_Reg

// C forward-decl `typedef struct luaL_Reg luaL_Reg;` — transparent alias to the
// struct definition, the same type rather than a second one.
pub type LuaLReg = crate::records::lua_l_reg::LuaLReg;
