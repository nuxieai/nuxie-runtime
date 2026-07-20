//! Generated skeleton item.
//! Node: `cxx:TypeAlias:Luau.VM:VM/include/lua.h:528:lua_callbacks`
//! Source: `VM/include/lua.h`
//! Graph edges:
//! - declared_by: source_file VM/include/lua.h
//! - source_includes:
//!   - includes -> source_file VM/include/luaconf.h
//! - incoming:
//!   - declares <- source_file VM/include/lua.h
//!   - type_ref <- record lua_Callbacks (VM/include/lua.h)
//! - outgoing:
//!   - type_ref -> record lua_Callbacks (VM/include/lua.h)
//!   - translates_to -> rust_item lua_Callbacks

// C forward-decl `typedef struct lua_Callbacks lua_Callbacks;` — transparent
// alias to the struct definition, the same type rather than a second one.
pub type LuaCallbacks = crate::records::lua_callbacks::LuaCallbacks;
