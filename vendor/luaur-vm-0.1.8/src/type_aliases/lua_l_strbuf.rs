//! Generated skeleton item.
//! Node: `cxx:TypeAlias:Luau.VM:VM/include/lualib.h:94:lua_l_strbuf`
//! Source: `VM/include/lualib.h`
//! Graph edges:
//! - declared_by: source_file VM/include/lualib.h
//! - source_includes:
//!   - includes -> source_file VM/include/lua.h
//! - incoming:
//!   - declares <- source_file VM/include/lualib.h
//!   - type_ref <- record luaL_Strbuf (VM/include/lualib.h)
//! - outgoing:
//!   - type_ref -> record luaL_Strbuf (VM/include/lualib.h)
//!   - translates_to -> rust_item luaL_Strbuf

// C forward-decl `typedef struct luaL_Strbuf luaL_Strbuf;` — transparent alias
// to the struct definition, the same type rather than a second one.
pub type LuaLStrbuf = crate::records::lua_l_strbuf::LuaLStrbuf;
