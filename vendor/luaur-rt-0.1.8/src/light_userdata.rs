//! [`LightUserData`] — a raw pointer carried as a first-class Lua value.
//!
//! Mirrors `mlua::LightUserData`. A light userdata is just a `void *` stored
//! directly in a Lua value (no GC, no metatable storage of its own beyond the
//! per-type metatable). luaur's VM represents it as the `LUA_TLIGHTUSERDATA`
//! tag; we push it via `lua_pushlightuserdatatagged` (tag 0) and read it back
//! with `lua_tolightuserdata`.

use std::os::raw::c_void;

/// A Lua light userdata: an opaque raw pointer value. Mirrors
/// `mlua::LightUserData`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LightUserData(pub *mut c_void);

// Under the `send` feature the whole VM (and every `Value` it can hold) is
// `Send` so it can be *moved* across threads. A light userdata is an opaque
// pointer value with no thread affinity of its own — moving it with the VM is
// sound under luaur-rt's move-not-share contract (see `crate::sync`). mlua makes
// the same documented `unsafe impl Send` for its `LightUserData`.
#[cfg(feature = "send")]
unsafe impl Send for LightUserData {}
#[cfg(feature = "send")]
unsafe impl Sync for LightUserData {}
