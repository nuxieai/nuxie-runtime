use crate::records::lua_state::lua_State;
use crate::type_aliases::lua_embedder_gc::lua_EmbedderGc;

pub unsafe fn lua_setembeddergc(l: *mut lua_State, callback: lua_EmbedderGc) {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    (*(*l).global).embeddergc = callback;
}
