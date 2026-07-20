//! Generated skeleton item.
//! Node: `cxx:TypeAlias:Luau.VM:VM/include/lua.h:457:lua_debug`
//! Source: `VM/include/lua.h`
//! Graph edges:
//! - declared_by: source_file VM/include/lua.h
//! - source_includes:
//!   - includes -> source_file VM/include/luaconf.h
//! - incoming:
//!   - declares <- source_file VM/include/lua.h
//!   - type_ref <- record lua_Debug (VM/include/lua.h)
//! - outgoing:
//!   - type_ref -> record lua_Debug (VM/include/lua.h)
//!   - translates_to -> rust_item lua_Debug

// C forward-decl `typedef struct lua_Debug lua_Debug;` — transparent alias to
// the struct definition, the same type rather than a second one.
pub type LuaDebug = crate::records::lua_debug::LuaDebug;
