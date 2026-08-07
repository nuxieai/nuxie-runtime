# Nuxie patches for luaur-rt 0.1.8

## Async coroutine host-data inheritance

- `Function::call_async` copies the invoking thread's
  `lua_getthreaddata` pointer to its implicit child coroutine before the
  child can resume. This matches Rive's
  `lua_setthreaddata(co, lua_getthreaddata(L))` on promise coroutines
  (`src/lua/lua_promise.cpp:1102`) and module threads
  (`src/lua/rive_lua_libs.cpp:693`) and keeps the host-owned scripting
  context available across the generic async bridge.
- Touched files: `src/function.rs`, plus `src/ffi.rs`/`src/ffi_public.rs`
  re-exports of `lua_getthreaddata`.
- A focused regression covers both the initial and post-yield resumes
  (`tests/mlua_async.rs::call_async_inherits_parent_thread_data_across_yield`).

## Userdata field dispatchers are Lua closures (UNIV-1764)

- `create_userdata`/`create_scoped_userdata` build the `__index`/`__newindex`
  field dispatchers as Lua closures (via a shared `create_field_dispatchers`)
  instead of Rust closures capturing `Table` handles. A luaur-rt handle is
  bound to the `lua_State` that created it; a userdata created inside a
  callback running on an implicit `call_async` coroutine outlives that
  coroutine, and its Rust dispatchers then manipulated the dead coroutine's
  stack (native: `index2addr` assert; wasm release: `lua_g_indexerror`
  abort). A Lua closure is a VM heap object whose dispatch runs on whichever
  live thread invokes the metamethod, matching mlua's current-state dispatch
  and C++ Luau's C-function metamethods.
- Touched files: `src/userdata.rs` (shared `create_field_dispatchers`, used
  by both `create_userdata` and `create_scoped_userdata`).
- Regressions: `call_async_closure_captures_userdata_across_pending_await`
  (this package) plus nuxie-scripting's
  `async_shader_instantiation` production-path tests.

## Luau fork rung 7

- Ported official Luau 0.731 delta (upstream e8ae48c4..f8ca77ac).
- Touched areas: compiler configuration for float or double vector constant
  precision, runtime vector push/read conversion, and feature-sensitive VM type
  tag mapping.
