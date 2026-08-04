//! The public, mlua-style raw C API surface (re-exported at `luaur_rt::ffi`).
//!
//! Mirrors `mlua::ffi`. Only the commonly used subset of the luaur C API is
//! surfaced here, for callers that need to drop down to the stack-machine level
//! (e.g. inside [`Lua::exec_raw`](crate::Lua::exec_raw) or
//! [`Lua::create_c_function`](crate::Lua::create_c_function)). The full luaur C
//! API lives in the `luaur-vm` crate.
//!
//! Everything here is `unsafe` to use and offers no safety guarantees beyond the
//! underlying VM — it is the same low-level interface the safe wrappers sit on
//! top of. luaur is a pure-Rust VM, so these are plain Rust `fn`s, not a C ABI
//! boundary (see [`lua_CFunction`], which is a Rust `unsafe fn`, not an
//! `extern "C-unwind" fn`).

pub use luaur_vm::type_aliases::lua_c_function::lua_CFunction;
pub use luaur_vm::type_aliases::lua_state::lua_State;

pub use luaur_vm::functions::lua_call::lua_call;
pub use luaur_vm::functions::lua_error::lua_error;
pub use luaur_vm::functions::lua_getfield::lua_getfield;
pub use luaur_vm::functions::lua_pcall::lua_pcall;
pub use luaur_vm::functions::lua_pushboolean::lua_pushboolean;
pub use luaur_vm::functions::lua_pushinteger::lua_pushinteger;
pub use luaur_vm::functions::lua_pushnumber::lua_pushnumber;
pub use luaur_vm::functions::lua_setfield::lua_setfield;
pub use luaur_vm::functions::lua_tointegerx::lua_tointegerx;
pub use luaur_vm::functions::lua_tonumberx::lua_tonumberx;
pub use luaur_vm::functions::lua_type::lua_type;

pub use luaur_vm::macros::lua_getglobal::lua_getglobal;
pub use luaur_vm::macros::lua_setglobal::lua_setglobal;
