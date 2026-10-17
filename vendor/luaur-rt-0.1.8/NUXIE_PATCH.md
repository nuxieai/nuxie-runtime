# Nuxie patches for luaur-rt 0.1.8

## Non-executing caller identity (runtime 7c9291ae)

- `Lua::state_identity` exposes an opaque, non-owning integer token for the
  handle's actual calling state, including implicit async coroutines. The
  tools-only runtime write-attribution adapter uses it without creating
  handles, allocating, executing VM instructions, or adding error paths.
- This is observation only: no dereference or state mutation API is exposed.
  Touched file: `src/state.rs`.

## Module lifecycle inspection (runtime c5faa1fa)

- `Lua::create_module_thread` installs the module closure's sandbox as the
  coroutine globals and inherits host thread data. `Thread::resume_module`
  reports nonzero statuses, including yields, while the normalized error and
  original frames are still on the coroutine, before transferring the error.
- `Thread::stack_value` provides non-consuming inspection and retained closure
  handles; the existing raw `Thread::state` supports debug-frame inspection.
- These are pure-Rust VM boundaries for Rive's default-no-op module hooks,
  not a debugger or a compatibility C implementation. Generic coroutine resume
  behavior is unchanged.

## Bytecode-only device runtime (UNIV-1644)

- Source compilation and the `Chunk`/`Compiler` API are behind the default-on
  `compiler` feature. Disabling default features retains the VM, the high-level
  handle API, and `Lua::load_bytecode` without normal target dependencies on
  `luaur-ast`, `luaur-bytecode`, or `luaur-compiler`.
- Runtime-owned Lua helper closures used by bind, userdata dispatch, table
  length, and the optional async bridge are compiled by `build.rs`. The pinned
  compiler is therefore a host build dependency, not device code.
- Touched files: `Cargo.toml`, `Cargo.toml.orig`, `build.rs`, `src/lib.rs`,
  `src/state.rs`, `src/luau_ext.rs`, `src/function.rs`, `src/table.rs`,
  `src/userdata.rs`, and `src/async.rs`.

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

## Translated `luaL_error` callback attribution

- `Error::lua_l_runtime` gives Rust translations of C callbacks an explicit
  `luaL_error` path through the shared callback trampoline. It prepends the Lua
  caller's source location and raises the unwrapped message, matching the C API
  instead of producing a location-free nested `runtime error` string.
- Touched files: `src/error.rs`, `src/callback.rs`, and
  `tests/error_position.rs`; workspace-patched execution is covered by
  `tools/luaur-rt-test-harness/tests/error_position.rs`.

## App-data lookup hashing

- The per-VM app-data store hashes its keys (a VM's global-state pointer and a
  `TypeId`) with a one-word multiplicative hasher instead of SipHash. Hosts
  look app data up on every script call; lookup results, borrow tracking and
  the API are unchanged.
- Touched file: `src/app_data.rs`.
- The interrupt-closure map and the per-VM userdata metatable cache use the
  same hasher: the interrupt trampoline looks its closure up twice per
  interrupt, and userdata creation looks up its metatable by VM and `TypeId`.
  Map contents and lookups are unchanged. Touched files: `src/interrupt.rs`,
  `src/userdata.rs`.

## Shared borrowed `LuaInner`

- `Lua::from_borrowed`, which every callback and interrupt trampoline calls,
  reuses the borrowed `LuaInner` it last built on the thread when the state
  matches instead of allocating one per call. A borrowed inner holds only its
  state pointer and never closes the state, so handles built from a shared
  one behave as before. Without `typecheck` only: there each borrowed inner
  carries its own definition buffer.
- Touched file: `src/state.rs`.

## Unprotected reads of metatable-free tables; `Table::get_field`

- `Table::get` reads a table that has no metatable with `lua_rawget`: with no
  `__index` the raw value is exactly what `lua_gettable` yields, and the read
  cannot raise, so the protected call (and the C closure it allocates) is
  needed only for tables with a metatable, which keep that path unchanged.
- `Table::get_field(&str)` mirrors `lua_getfield`: it pushes the key directly
  instead of first creating a registry-owned `LuaString`, for hosts that look
  up script methods by name on every call.
- Touched file: `src/table.rs`; coverage in
  `tools/luaur-rt-test-harness/tests/table_get_field.rs`.

## Callback arguments and results on the stack

- The callback trampoline no longer collects every argument into a
  `MultiValue` (a heap deque holding a registry reference per table,
  function or userdata argument). A `BoxedCallback` receives the argument
  count and converts from the stack: a userdata method reads `self` from
  argument slot 1 in place (the slot roots it for the call), and the
  arguments through the hidden `FromLuaMulti::from_stack_args`, which converts
  exactly the values `from_lua_multi` would, in order. Zero or one result is
  handed back as `ReturnValues` without a deque and pushed where the deque was.
  This is mlua's `from_stack` fast path; conversions, errors, error messages
  and results are unchanged.
- Scoped userdata/functions and the historical `MultiValue` callback shape keep
  collecting their arguments (`multi_value_callback`).
- Tuples build one pre-sized `MultiValue` (hidden `push_into_multi`) instead of
  one per element.
- Touched files: `src/traits.rs`, `src/conversion.rs`, `src/callback.rs`,
  `src/scope.rs`, `src/state.rs`, `src/userdata.rs`, `src/lib.rs`; coverage in
  `tools/luaur-rt-test-harness/tests/callback_stack_args.rs`.

## Interrupt closure held, not re-inserted

- The interrupt trampoline used to remove its closure from the per-VM map and
  re-insert it around every call (two hash-map mutations per safepoint). The
  map now holds an `Rc` entry with a `running` flag: the trampoline clones the
  entry, skips a safepoint reached while the closure runs (as the removed
  closure was skipped), and a closure installed meanwhile replaces the map's
  entry and runs at once, as before.
- Touched file: `src/interrupt.rs`; coverage in
  `tools/luaur-rt-test-harness/tests/interrupt_reentry.rs`.

## Field functions called in place; call results from the stack

- `Table::with_field` reads `self[field]` exactly as `Table::get_field` does
  (raw without a metatable, protected `lua_gettable` with one) and hands the
  callback a `FieldValue` held in its stack slot; `with_field_function` runs
  the callback only for a function (`LUA_TFUNCTION`). Hosts that call a
  script method by name on every frame no longer mint a registry-owned
  `Function` per call; `FieldValue::to_value` gives the value when it is not a
  function. `FieldFunction::call` is `Function::call` for the held function.
- `Function::call` pushes its arguments directly (hidden
  `IntoLuaMulti::push_call_args`, the same conversions as `into_lua_multi`)
  and converts its results with `from_stack_args`, so unused results take no
  registry references. The stack-overflow checks and errors are unchanged.
- Touched files: `src/table.rs`, `src/function.rs`, `src/traits.rs`,
  `src/conversion.rs`, `src/lib.rs`; coverage in
  `tools/luaur-rt-test-harness/tests/table_field_calls.rs`.

## Interrupt budgets

- `Lua::set_interrupt_with_budget` installs an interrupt closure with an
  `InterruptBudget`: safepoint counters (a count up to a limit, optionally a
  countdown) that the trampoline settles itself while both allow, without
  building a `Lua` handle or calling the closure. The closure runs only when
  a counter is exhausted and stays the authority for that case; it must do
  exactly what the budget does while the budget allows a safepoint. Hosts
  whose interrupt only counts safepoints against a ceiling pay a cell update
  per safepoint instead of a closure call.
- Touched files: `src/interrupt.rs`, `src/lib.rs`; coverage in
  `tools/luaur-rt-test-harness/tests/interrupt_reentry.rs`.

## Userdata held on the stack

- A callback that returns a new userdata (`T: UserData` by value) creates it
  where the trampoline would push the returned value (`ReturnValues::Pushed`):
  the same userdata, metatable and destructor as `create_userdata`, without
  the registry reference and `Rc` the handle needed only to be pushed and
  dropped. Without stack room it falls back to the handle, so the trampoline
  reports the overflow as before. Returning an `AnyUserData` handle is
  unchanged.
- `Lua::with_stack_userdata` holds a new userdata in a stack slot for a
  closure: `StackUserData::arg` passes it as a call argument (`StackArg`,
  pushed from the slot; through a handle onto another thread's stack) and
  `borrow` reads it as `AnyUserData::borrow` does. It is collected like any
  unreferenced userdata afterwards unless Lua kept it, as the dropped handle
  was before.
- `create_userdata` and both paths share `push_userdata`; destructed-value
  errors in the userdata collector are built only on that path.
- Touched files: `src/userdata.rs`, `src/table.rs`, `src/traits.rs`,
  `src/callback.rs`, `src/lib.rs`; coverage in
  `tools/luaur-rt-test-harness/tests/stack_userdata.rs`.

## Userdata arguments borrowed in place

- `UserDataMethods::add_method_with_args` registers a method that reads its
  arguments through a call-scoped `CallArgs`: `get` converts an argument as
  `FromLua` does, and `userdata(i)` yields an `ArgUserData` held in its stack
  slot, failing exactly as an `AnyUserData` argument does when the value is
  not a userdata; `borrow`/`borrow_mut`/`is` behave as `AnyUserData`'s. A
  userdata argument therefore needs no registry reference or `Rc` handle.
  Lifetimes keep both inside the call. Arguments are read when the method
  body asks, after `self` is borrowed; hosts use it where that order cannot
  change which error is reported.
- Touched files: `src/userdata.rs`, `src/lib.rs`; parity coverage against
  handle arguments in `tools/luaur-rt-test-harness/tests/stack_userdata.rs`.

## Stack-held handles are bound to their call frame; counts are checked

- A positive stack index names a slot of the call frame current when it is
  used. `FieldValue`/`FieldFunction`, `StackUserData`/`StackArg` and
  `CallArgs`/`ArgUserData` record the frame they were taken in (the state and
  its `CallInfo` depth, `stack_slot::FrameSlot`) and fail with an error when
  used from another frame, e.g. from a scoped callback Lua calls while the
  handle is held, instead of reading whatever that frame's slot holds.
  `FieldValue::as_function` therefore returns a `Result`.
- `ReturnValues::Pushed` carries a `PushedCount` only luaur-rt can make, and
  the trampoline checks it against the values above the callback's arguments
  before the VM reads them. `Function::call` checks the count
  `push_call_args` reports against the values actually pushed.
- Touched files: `src/stack_slot.rs` (new), `src/table.rs`, `src/userdata.rs`,
  `src/traits.rs`, `src/callback.rs`, `src/function.rs`, `src/lib.rs`;
  coverage in `tools/luaur-rt-test-harness/tests/stack_userdata.rs`.
