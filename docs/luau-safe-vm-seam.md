# Safe Luau VM: the seam inventory (D5)

Companion to `docs/luau-safe-vm-plan.md`, decision D5 and section 4. This
document lists everything the product asks of the Luau VM today, so a
replacement VM can stand behind exactly that surface, and gives every raw
escape hatch a disposition. Inventory taken 2026-10-31 at `d2a4dbe22b`.

## 0. Scope and method

**Who depends on the VM.** `grep -l luaur crates/*/Cargo.toml tools/*/Cargo.toml`:

| Crate | luaur crates | Section | What it means |
|---|---|---|---|
| `crates/nuxie-scripting` | `luaur-rt`, `luaur-vm`, `luaur-common` | `[dependencies]` | The only product crate that links the VM. Everything below is about it. |
| `crates/nuxie-scripting` | `luaur-compiler`, `luaur-common` | `[build-dependencies]`, `[dev-dependencies]` | `build.rs` precompiles three embedded helper chunks; tests compile source. |
| `crates/nuxie`, `crates/nux-capi`, `crates/nuxie-runtime`, `tools/runtime-test-support` | `luaur-compiler`, `luaur-common` | `[dev-dependencies]` only | Tests compile Luau source. No VM dependency. |
| `tools/silver-corpus` | `luaur-rt`, `luaur-vm`, `luaur-compiler` | `[dependencies]` | Oracle tool; uses the handle API plus `lua_gettop`/`lua_gc` (section 5). |
| `tools/luaur-rt-test-harness` | `luaur-rt`, `luaur-vm` | `[dependencies]` | Tests the vendored `luaur-rt` itself. Not a product consumer. |

But the handle API also leaks past `nuxie-scripting` without a Cargo
dependency: `crates/nuxie-scripting/src/lib.rs:38` re-exports
`pub use luaur_rt::{Lua, Value}`, and `ScriptVm::lua()` (`vm.rs:2576`) is
public. Two product files outside the crate call luaur-rt methods through
that door (section 1.3).

**Counting.** Every number here is a grep over source, never an estimate.
The scripts live outside the tree; their rules are:

- *Product vs test.* A line is test code if it is in a file that is only
  compiled under `cfg(test)` (13 files: `vm/upstream_scripting_*_test.rs`,
  `vm/upstream_scripting_gpu_{features,target}.rs`,
  `vm/upstream_scripting_routing.rs`, `vm/*_tests.rs`,
  `vm/view_model/*_tests.rs`, `gpu_canvas_ore/tests.rs`) or inside an inline
  `mod … {` / `fn` whose attributes mention `test`. `crates/nuxie-scripting/tests/`
  is not counted at all.
- Embedded Luau sources in multi-line `r#"…"#` strings are blanked first, so
  `coroutine.resume(` inside the Promise library does not count as a Rust call.
- Comments are skipped.
- *Ambiguous names.* Method names that std types share (`get`, `set`, `len`,
  `push`, `call`, `borrow`, `as_bytes`, `upgrade`, …) cannot be attributed to
  a luaur receiver by grep. Those rows carry a `≤` and are upper bounds. A
  receiver-typed count needs a compiler pass (for example `#[deprecated]` on
  every luaur-rt method and counting warnings); that was not run.

## 1. The handle API surface

### 1.1 What the product calls, by type

Columns: product call sites, test call sites (inside `src/`), the product
modules that use it (most uses first), and what it maps to. "C API" means a
plain Luau C-API concept the new VM gets for free from upstream semantics.
"mlua" means a luaur-rt convenience over plain C API with no VM requirement
of its own. "Patch" means a Nuxie or Rive change carried in
`vendor/luaur-rt-0.1.8/NUXIE_PATCH.md` or the Rive fork, which the new VM must
reproduce on purpose.

**`Lua` (state handle)**

| Method | Prod | Test | Product modules | Maps to |
|---|---:|---:|---|---|
| `Lua::new` | 2 | 115 | `vm`, `lua_artboards` | C API `lua_newstate` + `luaL_openlibs`; **also calls `luaur_common::set_all_flags(true)` (process-global flag profile, section 3.8)** |
| `globals` | 43 | 292 | 27 modules: `vm`, `resource_limits`, `gpu_canvas`, `lua_promise`, … | C API `LUA_GLOBALSINDEX` |
| `create_userdata` | 145 | 56 | 32 modules: `listener_invocation` 19, `gpu_canvas` 17, `lua_audio_processors` 12, `view_model` 11, `lua_artboards` 11, … | C API `lua_newuserdatadtor` + per-`TypeId` metatable; **Patch UNIV-1764**: field dispatchers are Lua closures so they run on whichever thread invokes them |
| `create_function` | 112 | 13 | 31 modules: `view_model` 20, `lua_vec2d` 18, `lua_mat2d` 7, … | C API `lua_pushcclosurek` with a Rust trampoline |
| `create_table` | 61 | 21 | 25 modules | C API `lua_createtable` |
| `create_table_result` | 2 | 0 | `lua_mat4` | C API `lua_createtable` (fallible form) |
| `create_sequence_from` | 1 | 0 | `lua_rive_file` | C API `lua_createtable` + `lua_rawseti` |
| `create_string` | 21 | 3 | `vm` 7, `lua_scriptnet` 7, `view_model` 3, `lua_font` 2, `lua_audio_processors`, `lua_path` | C API `lua_pushlstring` |
| `create_buffer` | 4 | 6 | `lua_image_decode`, `lua_audio_processors`, `lua_scriptnet`, `lua_blob` | C API `lua_newbuffer` |
| `create_buffer_with_capacity` | 1 | 0 | `gpu_canvas` | C API `lua_newbuffer` |
| `create_module_thread` | 1 | 0 | `vm` (`prepare_module`, `vm.rs:2847`) | **Patch c5faa1fa**: new thread with the closure's sandbox env as coroutine globals, inherits host thread data |
| `create_thread` | 0 | 1 | — | C API `lua_newthread` |
| `create_c_function` | 3 | 0 | `vm` (require), `lua_promise` (async thread), `lua_mat4` (20 functions) | Raw hatch: `lua_pushcclosurek` of an `unsafe fn(*mut lua_State)`; every such function is named `"luaur-rt-c-function"` (section 3.7) |
| `create_registry_value` / `registry_value` | 3 / 3 | 7 / 9 | `lua_audio_samples` 2, `lua_image_decode` | C API `lua_ref` / `lua_rawgeti(REGISTRY)` |
| `remove_registry_value` | 0 | 6 | — | C API `lua_unref` |
| `set_named_registry_value` / `named_registry_value` | 13 / 15 | 0 / 1 | `view_model`, `resource_limits`, `native_registration`, `lua_promise`, `lua_vec2d`, `vm`, … (8 modules each) | C API `lua_setfield`/`lua_getfield` on `LUA_REGISTRYINDEX` |
| `set_app_data` / `app_data_ref` | 26 / 57 | 6 / 1 | `lua_audio_processors` 7/21, `lua_image` 6/9, `lua_image_decode`, `lua_scriptnet`, `lua_audio_samples`, `lua_main_ref`, … (13 / 12 modules) | Not a VM concept: a Rust-side per-VM `TypeId` map (Patch: word hasher). This is where Rust keeps what C++ keeps in `lua_getthreaddata` |
| `app_data_mut` | 0 | 0 | — | (unused) |
| `remove_app_data` | 1 | 0 | `lua_scriptnet` | Rust-side map |
| `gc_collect` | **0** | 26 | — | C API `lua_gc(LUA_GCCOLLECT)`. Test-only (section 3.6) |
| `load_bytecode` | 7 | 1 | `vm` 2, `gpu_canvas`, `lua_promise`, `view_model`, `lua_artboards`, `lua_data_value` | C API `luau_load` with chunk name `"=" + name`; **Patch UNIV-1644** (bytecode-only device runtime). Used only for runtime-owned helper chunks; authored bytecode goes through the raw hatch R1 |
| `load` (source, `compiler` feature) | 0 | ≤141 | — | Compiler + `luau_load`. Test-only |
| `current_thread` | 5 | 5 | `vm`, `lua_font`, `lua_main_ref`, `lua_transition`, `lua_input` | Every product use is `.current_thread().state()`: a raw pointer for a hatch |
| `set_interrupt` | 3 | 2 | `vm` 2, `gpu_canvas` | C API `lua_callbacks()->interrupt` (GC-step interrupts filtered out) |
| `scoped_interrupt` | 1 | 1 | `vm` (`start_module_execution`, `vm.rs:2273`) | **Patch** "Scoped module execution interrupt" (upstream 6ec0b04b deadlines) |
| `remove_interrupt` | 0 | 2 | — | C API |
| `set_memory_limit` | 3 | 0 | `vm` 2, `gpu_canvas` | mlua: wraps `global->frealloc` with a byte cap, raising `LUA_ERRMEM` |
| `state_identity` | 1 | 0 | `vm` (`luau_write_scope`, `vm.rs:1158`, `tools` feature only) | **Patch 7c9291ae**: opaque id of the calling thread |
| `exec_raw` | 20 | 2 | 11 modules (section 2) | Raw hatch (luaur-rt API, not a Nuxie patch) |
| `weak` | 3 | 3 | `view_model`, `lua_main_ref`, `lua_scriptnet` | Rust `Rc` weak of the VM owner |
| `try_upgrade` (on `WeakLua`) | 5 | 3 | `lua_main_ref` 3, `view_model`, `lua_scriptnet` | Rust |
| `unpack` / `pack` | 11 / 6 | 0 | `lua_font`, `view_model`, `lua_transition`, `lua_scriptnet`, `lua_data_value`, `lua_input` | mlua `FromLua`/`IntoLua` |
| `coerce_number` | 10 | 0 | `lua_color` 4, `vm`, `lua_canvas`, `lua_font`, `lua_data_value`, `gpu_canvas_ore/shader`, … | C API `lua_tonumberx` (string-to-number coercion) |
| `inspect_stack` | 1 | 0 | `vm` (`caller_chunk_source`, `vm.rs:1767`) | C API `lua_getinfo(level, "s")`; product reads `source` (section 3.7) |
| `sandbox` | 1 | 0 | `vm` (`vm.rs:2655`) | C API `luaL_sandbox` |
| `scope` | 1 | 0 | `resource_limits` | mlua scoped (non-`'static`) callbacks |
| `set_type_metatable::<Vector>` | 1 | 0 | `lua_vec2d` | C API: the global vector metatable |

**`Table`**

| Method | Prod | Test | Product modules | Maps to |
|---|---:|---:|---|---|
| `get` | ≤328 | ≤359 | 41 modules; `gpu_canvas` 96, `view_model` 46, `vm` 33 | C API `lua_gettable` (protected); **Patch**: `lua_rawget` when the table has no metatable |
| `set` | ≤243 | ≤219 | 38 modules; `vm` 48, `view_model` 34 | C API `lua_settable` (protected) |
| `get_field` | 14 | 0 | `vm` 12, `lua_path`, `lua_renderer` | **Patch** "Table::get_field": `lua_getfield` without a registry-owned key string |
| `raw_get` / `raw_set` | 30 / 35 | 9 / 8 | `lua_text`, `native_registration`, `gpu_canvas_ore/pipeline`, `lua_audio_processors` 16 (set), … | C API `lua_rawget` / `lua_rawset` |
| `raw_len` | 23 | 7 | `gpu_canvas_ore/pipeline` 11, `gpu_canvas` 4, `gpu_canvas_ore/pass` 3, … | C API `lua_objlen` (tables only) |
| `raw_push` | 1 | 0 | `lua_artboards` | C API `lua_rawseti(n+1)` |
| `len`, `push`, `pop`, `clear`, `contains_key` | ≤219, ≤102, ≤6, ≤17, ≤5 | | (mostly `Vec`/`HashMap`; not separable by grep) | `len` honors `__len` via the build-time `TABLE_LEN` helper chunk |
| `pairs` | 14 | 0 | `native_registration` 4, `lua_audio_processors` 3, `gpu_canvas`, `lua_scriptnet`, `host_commands`, `lua_paint`, … | C API `lua_next` |
| `sequence_values` | 10 | 0 | `gpu_canvas` | C API `lua_rawgeti` loop |
| `set_metatable` | 6 | 1 | `lua_mesh` 2, `view_model` 2, `vm`, `lua_vec2d` | C API `lua_setmetatable` |
| `set_readonly` / `is_readonly` | 28 / 6 | 2 / 2 | 18 / 5 modules | C API `lua_setreadonly` / `lua_getreadonly` (Luau) |
| `to_pointer` (all handle types) | 9 | 13 | `view_model` 5, `lua_path` 2, `host_commands`, `lua_audio_processors` | C API `lua_topointer` (identity keys) |

**`Function`, `Thread`, `AnyUserData`, userdata registration**

| Method | Prod | Test | Product modules | Maps to |
|---|---:|---:|---|---|
| `Function::call` | ≤49 | ≤69 | `view_model`, `gpu_canvas`, `lua_promise`, `lua_font`, `resource_limits`, … (`luau_audio_shared_web` 17 are not Lua) | C API `lua_pcall`; wrapped by the product's `ProtectedScriptCall` (`vm.rs:1134`) |
| `Function::environment` / `set_environment` | 4 / 1 | 0 | `vm` | C API `lua_getfenv` / `lua_setfenv` (module sandbox) |
| `Function::bind`, `wrap` | 0 | ≤5 / 0 | — | (test-only / unused) |
| `Function::call_async` | **0** | 0 | — | **Patch**: thread-data inheritance into the implicit coroutine. Product never calls it; `tests/default_features.rs` only asserts the API exists |
| `Thread::resume_module` | 1 | 0 | `vm` (`execute_module`, `vm.rs:2858`) | **Patch c5faa1fa**: reports error with frames still on the coroutine, drops the execution guard before hooks |
| `Thread::state` (raw `*mut lua_State`) | 5 | 13 | `vm`, `lua_font`, `lua_main_ref`, `lua_transition`, `lua_input` | Raw pointer for hatches R4, R6, R21–R23 |
| `Thread::resume`, `reset`, `status`, `stack_value` | 0 | ≤1 | — | (test-only or unused; `stack_value` is **Patch c5faa1fa**, unused) |
| `AnyUserData::borrow` / `borrow_mut` | ≤129 / ≤34 | ≤42 / 0 | 26 / 11 modules; `lua_renderer` 22/9 | Typed `RefCell` payload inside the userdata block |
| `AnyUserData::is` | 13 | 2 | `lua_audio_processors` 4, `gpu_canvas_ore/shader` 3, `lua_paint` 2, `lua_renderer` 2, … | `TypeId` check |
| `UserDataMethods::add_method` / `add_method_mut` / `add_function` / `add_meta_method` | 195 / 67 / 10 / 9 | 0 | 21 / 9 / 4 / 7 modules | Metatable built once per `TypeId`; method and field lookup through the `USERDATA_DISPATCH` Lua closure (`vendor/luaur-rt-0.1.8/build.rs:16`). No `__namecall` is installed, so `obj:m()` goes `NAMECALL` → `__index` closure |
| `UserDataFields::add_field_method_get` / `_set` / `add_field_function_get` / `add_field_index_method_get` / `_set` | 183 / 45 / 9 / 1 / 1 | 0 | 20 / 8 / 2 / 1 / 1 modules | Getter/setter tables read by the same dispatcher; unknown-field set raises `attempt to set unknown field '%s' on userdata` |
| `add_meta_method` names used | `__len` 3, `__namecall` 2, `__eq` 2 (one via `MetaMethod::Eq`), `__iter` 1, `__mul` 1 | | | |
| `impl UserData for` | 97 types | | | |

**Values, strings, buffers, errors**

| Item | Prod | Test | Product modules | Maps to |
|---|---:|---:|---|---|
| `Buffer::read_bytes` / `write_bytes` | 11 / 10 | 4 / 2 | `shader_asset` 10, `buffer_ext` 5, `lua_audio_processors` 3, … | C API `lua_tobuffer` + copy |
| `Buffer::to_vec` | ≤41 | ≤54 | | C API `lua_tobuffer` + copy |
| `LuaString::as_bytes` / `to_str` / `to_string_lossy` | ≤40 / ≤11 / ≤16 | | 17 / 8 / 7 modules | C API `lua_tolstring` |
| `Value::type_name` / `as_table` | 5 / 1 | 0 | `vm`, `lua_data_value`, `host_commands` | C API `lua_typename` |
| `Vector::new` | 32 | 0 | `lua_vec2d` 10, `lua_path` 8, `listener_invocation` 4, `lua_artboards` 4, … | C API `lua_pushvector` (3 floats) |
| `MultiValue::from_vec` | 12 | 0 | `view_model`, `lua_audio`, `lua_text`, `gpu_canvas`, `vm` | mlua |
| `RegistryKey` / `LightUserData` (types named) | 5 / 2 | 0 | `lua_audio_samples`, `lua_image_decode` / `view_model` | C API `lua_ref` / `lua_pushlightuserdata` |
| `Error::runtime` | 505 | 36 | 40 modules; `gpu_canvas` 140, `shader_asset` 51 | Host error raised as Lua error (`runtime error:` wrapping, section 3.7) |
| `Error::lua_l_runtime` | 2 | 0 | `lua_renderer` | **Patch** "Translated `luaL_error` callback attribution": prepends the Lua caller's `source:line:` |
| `Error::RuntimeError` / `MemoryError` / `CallbackError` / `SyntaxError` matched | 12 / 3 / 2 / 1 | 0 | `lua_video`, `view_model`, `resource_limits`, `logging_scripting_context`, `gpu_canvas` | Error classification: `MemoryError` drives `ScriptResourceLimit::Memory`; `CallbackError` is unwrapped to its cause |
| `VmState::Continue` | 4 | 3 | `vm` 3, `gpu_canvas` | Interrupt return value |

### 1.2 Product-level APIs that are not VM concepts

The plan's section 4 lists `set_log_sink` and `set_timeout_ms`. Both are
`ScriptVm` methods (`vm.rs:2239`, `vm.rs:2251`), not luaur-rt: the log sink
is a Rust `print` installed with `create_function`
(`vm/lua_rive_base.rs:8-30`, it resolves `tostring` from the invoking thread
on purpose), and the timeout is a field read by the interrupt closure
(section 3.1). They need nothing from the VM beyond `create_function`,
`globals` and the interrupt.

### 1.3 Handle API used outside `nuxie-scripting`

Through `pub use luaur_rt::{Lua, Value}` and `ScriptVm::lua()`:

| File | Calls |
|---|---|
| `crates/nuxie/src/checked_value_script.rs` (product) | `Value::as_string` 4, `as_bytes` 2, `to_str` 2, `as_table` 2, `Table::pairs` 2, `is_nil`, `as_boolean`, `as_number`, `as_integer`, `Lua::create_table_result`, `Table::raw_set` 5, `Lua::create_string` (lines 63–227) |
| `crates/nux-capi/src/value_rule_script.rs` (product) | `ScriptVm::lua()`, `Value::as_table`, `Table::set_readonly` 2, `Lua::create_function` 3 with `FromLua` argument tuples, `Table::set` 3 (lines 49–101) |
| `crates/nux-capi/src/value_rule_script_tests.rs`, `tools/silver-corpus`, `tools/luau-audio-web/src/probe.rs` | tests and tools: `create_function`, `globals`, `create_buffer`, `load_bytecode`, `current_thread().state()` |

A replacement VM therefore has to keep `Lua`, `Value`, `Table`,
`Function` and the `FromLua`/`IntoLua` conversions source-compatible for two
more crates, or the re-export has to be narrowed first.

## 2. Raw escape hatches

### 2.1 Totals

- **`exec_raw` call sites: 22** (20 product, 2 test) in 11 product files.
  The plan's "19" missed the turbofish form `exec_raw::<(), _>(` at
  `vm.rs:2925` (plus the two test sites).
- **Direct raw symbols: 75 distinct names**, counted from `luaur_vm::…`
  paths and `use` groups plus `luaur_rt::ffi::{lua_error, lua_CFunction}`
  and `luaur_rt::lua_State`:
  - 73 are used by product code, in 13 product files.
  - 2 are test-only (`lua_tostringatom`, `lua_pop`), and 3 more files touch
    raw symbols only in tests (`vm/lua_data_value.rs` test module,
    `vm/artboard_context_8f06_tests.rs`, `vm/property_lifetime_a637_tests.rs`).
  - **24 are exclusive to `vm/luau_audio_processor_engine.rs`**:
    `lua_newstate`, `lua_close`, `l_alloc`, `lua_pcall`, `lua_gc`,
    `lua_l_openlibs`, `lua_l_sandbox`, `lua_pushcclosurek`, `lua_createtable`,
    `lua_newbuffer`, `lua_tobuffer`, `lua_next`, `lua_rawset`, `lua_rawseti`,
    `lua_setmetatable`, `lua_toboolean`, `lua_tonumberx`, `lua_topointer`,
    `lua_pushlstring`, `lua_gettop`, `lua_error`, `lua_exception`,
    `LUA_GLOBALSINDEX`, `lua_upvalueindex`.
  - **49 are used in the 12 product files outside the audio engine.** That
    is the population the plan's "53 symbols across 12 files" described; the
    audio engine is a second embedding the plan did not list (row group J).
- Raw functions installed with `create_c_function`: 22 (1 `require`,
  1 `newAsyncThread`, 20 in `Mat4`). The audio engine installs its own 29
  with `lua_pushcclosurek` (2 globals, 3 `Audio.*`, 24 channel methods and
  field handlers).

### 2.2 Rows, grouped by the capability they need

Dispositions: **lift** = add a safe method to the embedding API (signature
in section 6); **reimplement** = the behavior belongs in the new VM's
userdata/namecall/embedding model; **drop** = dead or already replaceable,
with the evidence.

**A. Load authored bytecode under a verbatim chunk name**

| # | Site | Raw operations | Why the handle API was not enough | Disposition |
|---|---|---|---|---|
| R1 | `vm.rs:2706-2734` (`exec_raw` at 2716), `ScriptVm::load_bytecode` | `luau_load(state, name, bytes, len, env=0)`; on failure `luaur_rt::ffi::lua_error` | `Lua::load_bytecode` prefixes `"="` to the chunk name (`vendor/luaur-rt-0.1.8/src/state.rs:585`). Rive's `luaL_where` prints the raw `ar.source` (`vendor/luaur-vm-0.1.8/src/functions/lua_l_where.rs`), so the chunk name appears verbatim in every error (`"test_source:1: require could not find…"`, `vm.rs:5666`). Chunk names also carry library scope that `lua_blob.rs:133` parses (`label@version/path`) and that R2 uses as a registry key | **lift** `load_bytecode_as` |

**B. Host functions that raise `luaL_error` with caller attribution and read debug info**

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R2 | `vm.rs:251-321` `lua_require_registered_module`, installed by `create_c_function` at `vm.rs:2900-2904` | `lua_getinfo(level, "s")` walking up to the first non-C frame for its `source`; `lua_settop`; `luaL_checkstring(1)`; `lua_getfield(REGISTRY, "_MODULES")`, `lua_getfield`, `lua_type`; on a miss, `registry._RIVE_REGISTRATION_DEPENDENCIES[source][path] = true` (`lua_pushboolean`, `lua_setfield`); `luaL_error("require could not find a script named %s")`; `lua_remove` | The doc comment says a Rust callback returning `Error::runtime` would lose the caller location and add a second `runtime error:` prefix. That is now solved in the handle API: `Error::lua_l_runtime` (Patch, used at `lua_renderer.rs` ×2) prepends the caller location, `Lua::inspect_stack(level).source()` gives the requiring chunk (already used at `vm.rs:1767`), and the registry tables are reachable with `named_registry_value` | **drop**: rewrite on `create_function` + `inspect_stack` + `named_registry_value` + `Error::lua_l_runtime`. Acceptance: `tests/upstream_scripting_require_check.rs` and the `scope_probe` corpus row |

**C. Coroutine creation with host thread data**

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R3 | `vm/lua_promise.rs:20-40` `create_async_thread`, installed at `:103` as the Promise library's `newAsyncThread` | `luaL_checktype(1, function)`, `lua_newthread`, `lua_setthreaddata(child, lua_getthreaddata(parent))`, `lua_xpush(fn)` | Mirrors C++ `lua_promise.cpp:1102`, which copies the `ScriptingContext` thread data. **No product code ever stores non-null thread data**: the only `lua_setthreaddata` writes are this copy and a test that writes `null` (`artboard_context_8f06_tests.rs:52`); luaur-rt only copies (`thread.rs:617`, `function.rs:192`). Rust keeps that context in app data instead | **drop**: replace with `Lua::create_thread` (existing) or `coroutine.create` inside the embedded library. Acceptance: `lua_promise.rs` `async_coroutine_inherits_thread_data_print_works` and `tests/promise_scenarios.rs` |

**D. VM callbacks and luaur-rt workarounds**

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R4 | `vm.rs:2164-2166` in `ScriptVm::new` | `(*lua_callbacks(current_thread().state())).useratom = resolve_rive_lua_atom`; table at `vm.rs:329-693`, resolver at `vm.rs:801-810` | luaur-rt exposes no `useratom`. The table is a port of C++ `lua_atoms.cpp`. **No product code reads an atom**: all four namecall sites (R21–R23, R28) call `lua_namecallatom(state, &mut atom)` and then compare the returned *string*; the only reader of the atom value is the test `vm.rs:4481-4494` | **drop** (the new VM may still intern method names for speed; that is section 3.2, not a hatch) |
| R5 | `vm.rs:2919-2927` `reserve_parent_stack_headroom` (`exec_raw` at 2925) | no-op `exec_raw` with eight `nil` arguments | luaur-rt bug: after a failed large chunk, the parent C frame can sit exactly at `ci->top`, and the next handle push fails; `exec_raw` calls `lua_checkstack` first. Called before every module-cache access and `eval_bytecode` | **drop**: a handle API without a shared parent C stack cannot have this state. Acceptance: the "failed large bytecode chunk then registry read" regression |

**E. VM-wide registry references that outlive the minting coroutine**

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R6 | `vm/lua_main_ref.rs:10-15` `install_main_lua` | `lua_mainthread(current_thread().state())`, stored as `*mut lua_State` in app data | A luaur-rt handle is bound to the `lua_State` that created it (see the UNIV-1764 note in `NUXIE_PATCH.md`, and `lua_rive_base.rs:15-16`: "a handle captured from the main thread cannot be called safely from an async coroutine"). Values minted inside a coroutine must outlive it | **lift** `RegistryRef` |
| R7 | `vm/lua_main_ref.rs:22-33` `Drop for MainReference` | `lua_unref(main, id)` | Must run from a userdata finalizer without allocating or entering Lua, after the minting coroutine may be dead; `RegistryKey` drop needs a live handle | **lift** `RegistryRef` (drop is allocation-free, thread-free) |
| R8 | `vm/lua_main_ref.rs:46-52` `MainThreadRef::new` (`exec_raw` at 50) | `lua_ref(state, 1)` | Same | **lift** `RegistryRef` |
| R9 | `vm/lua_main_ref.rs:63-73` `MainThreadRef::get` (`exec_raw` at 66) | `lua_rawgeti(REGISTRY, id)` onto the *calling* thread | Same | **lift** `RegistryRef` |

**F. Reading a userdata's metatable**

All eleven rows do the same thing: `exec_raw(userdata, |s| lua_getmetatable(s, 1))`
to get the metatable luaur-rt built, then layer behavior over it. luaur-rt
keeps userdata metatables private (`lua_path.rs:152-154` says so) and its
synthesized `__index` dispatcher overwrites any `__index` a `UserData` impl
registers (`view_model.rs:2126-2129`). The product needs that table to (a)
replace `__index` with strict C++ error texts such as
`'x' is not a valid index of Font`, (b) add integer indexing, (c) stash the
original dispatcher as `__rive_source_index`, and (d) freeze it with
`set_readonly(true)`.

| # | Site | Layered behavior | Disposition |
|---|---|---|---|
| R10 | `vm/lua_font.rs:259-266` `source_dispatcher` (`exec_raw` 261) | read stashed `__rive_source_index` | **lift** `AnyUserData::metatable` |
| R11 | `vm/lua_font.rs:269-283` `source_metatable` (`exec_raw` 279) | strict field set + `__namecall` method set for Font-family types | **lift** |
| R12 | `vm/lua_rive_file.rs:23-34` `c_prefix_index` (`exec_raw` 30) | strict fields, error text truncated at the first NUL (C-string diagnostics) | **lift** |
| R13 | `vm/lua_rive_file.rs:48-53` (`exec_raw` 49, inside the `__index` closure) | re-fetch to reach `__rive_source_index` | **lift** |
| R14 | `vm/lua_rive_file.rs:76-81` `ScriptedBindableArtboard::create` (`exec_raw` 77) | strict `name`/`data` | **lift** |
| R15 | `vm/lua_rive_file.rs:97-102` (`exec_raw` 98, inside `__index`) | same as R13 | **lift** |
| R16 | `vm/lua_transition.rs:38-46` `prepare_metatable` (`exec_raw` 42) | strict `width`/`height` | **lift** |
| R17 | `vm/lua_input.rs:50-58` `prepare_metatable` (`exec_raw` 54) | strict ScrollEvent fields returning `Vector` | **lift** |
| R18 | `vm/lua_path.rs:151-161` `userdata_metatable` (`exec_raw` 156) | numeric `PathCommand` indexing over the method table | **lift** |
| R19 | `vm/view_model.rs:1591-1598` `create_scripted_enum_values` (`exec_raw` 1594) | 1-based integer `__index` over enum value strings | **lift** |
| R20 | `vm/view_model.rs:2131-2137` property list (`exec_raw` 2132) | numeric list indexing in front of the dispatcher, keyed `__riveListIndexPatched` | **lift** |

R10–R20 could later be **reimplemented** in the new userdata model (a
`UserData` impl declaring its strict field set, missing-key error text,
integer index handler and namecall method set), which would delete all
eleven. Lifting first keeps the product unchanged at cutover.

**G. The current namecall method name**

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R21 | `vm/lua_font.rs:306-318` (`__namecall` Rust closure) | `lua_namecallatom(current_thread().state(), &mut atom)`, copy C string | luaur-rt has no accessor for the active `NAMECALL` method name, and installs no `__namecall` of its own | **lift** `Lua::namecall_method` |
| R22 | `vm/lua_transition.rs:101-114` (`add_meta_method("__namecall")`) | same; only `draw` is valid | same | **lift** |
| R23 | `vm/lua_input.rs:88-101` (`add_meta_method("__namecall")`) | same; only `hit` is valid | same | **lift** |

**H. `Mat4`: tagged plain-data userdata with direct fields (`vm/lua_mat4.rs`)**

One module, written entirely against the raw API. It mirrors
`ScriptedMat4` in C++: a 64-byte inline `[f32; 16]` payload, one shared
metatable per userdata tag (`MAT4_USERDATA_TAG = LUA_T_COUNT + 62`,
`lua_mat4.rs:30`, kept in sync with `rive_lua_libs.hpp`), and direct-field
getters for the interpreter's `GETTABLEKS` fast path.

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R24 | `:56-109` `install_mat4_global`, `set_c_function` (`exec_raw` at 67) | 20 × `create_c_function` (`__index`, `__newindex`, `__mul`, `__eq`, `__namecall`, 15 statics); `lua_setuserdatametatable(tag)` | luaur-rt userdata are boxed `RefCell<T>` with per-`TypeId` metatables; it has no per-tag metatable and no plain-data payload | **reimplement** (tagged POD userdata) |
| R25 | `:111-130` `push_mat4`, `check_mat4` | `lua_newuserdatataggedwithmetatable(64, tag)`, `lua_touserdatatagged`, `luaL_typeerror(…, "Mat4")` | Allocation-only construction with the shared metatable; C++ type-error text | **reimplement** |
| R26 | `:132-525` statics and methods (`identity`, `values`, `fromTranslation`, …, `lookAt`, `transformPoint`, `writeToBuffer`) | `luaL_checknumber`, `luaL_checkinteger`, `luaL_checkvector`, `luaL_checkbuffer`, `lua_isnumber`, `lua_pushnumber`, `lua_pushboolean`, `lua_pushnil`, `lua_pushvalue`, `lua_pushvector(f32×3)`, `luaL_error` | Exact `luaL_check*` error texts; in-place buffer writes | **reimplement** (needs the arg-check helpers of row group K) |
| R27 | `:388-424` `mat4_index`, `mat4_newindex`, `invalid_index` | `luaL_checklstring(2)`, parse `m11`…`m44`, `luaL_error("'%s' is not a valid index of Mat4")` | Strict keys with C++ text | **reimplement** |
| R28 | `:530-557` `mat4_namecall` | `lua_namecallatom`, match on the string (`invert`, `invertAffine`, `transpose`, `transformPoint`, `transformVec4`, `writeToBuffer`), else `luaL_error("%s is not a valid method of Mat4")` | Namecall without a method table | **reimplement** |
| R29 | `:559-613` `direct_field_getter!`, `register_direct_fields` | `lua_registeruserdatadirectfieldget(tag, "m11"…"m44", fn)`, getter writes with `lua_userdatadirectfield_setnumber` | Rive bytecode/VM extension (section 3.4); not reachable through luaur-rt at all | **reimplement** |

**K. Lua coercion and argument-check semantics**

| # | Site | Raw operations | Why | Disposition |
|---|---|---|---|---|
| R30 | `vm/view_model.rs:2500-2516` `lua_check_c_string` (`exec_raw` at 2513, 2514) plus `:2491-2498` | `luaL_checklstring(2)`, `lua_pushvalue(2)` under a protected call with a placeholder `self` | Needs the exact `luaL_checkstring` error, including missing-vs-explicit-nil. The trampoline's debug name leaks into that error, so `:2494` rewrites `'luaur-rt-exec-raw'` to `'globalViewModel'` in the message text | **lift** `Lua::check_arg` + named host functions |
| R31 | `gpu_canvas_ore/mod.rs:75-96` `string_value` (`exec_raw` at 87) | `lua_tolstring(1)` (converts numbers in place), `lua_pushvalue(1)` | `FromLua<String>` formats numbers with Rust's formatter; the C++ source uses `lua_tostring` (`%.14g`) | **lift** `Lua::coerce_string` |
| R32 | `gpu_canvas_ore/pipeline.rs:255-268` (`exec_raw` at 264) | `lua_objlen(1)`, `lua_pushinteger` | C++ reads `lua_objlen` of any value (string, buffer and userdata byte lengths count) before its 32-attribute budget check; `Table::raw_len` only takes tables | **lift** `Lua::objlen` |

**J. The audio processor engine: a second, complete raw embedding**

`vm/luau_audio_processor_engine.rs` (product lines 1–1626) does not use
luaur-rt at all. Its module doc says why: "Only this private raw VM crosses
threads: no luaur-rt handle, host callback capture, registry TLS or scene
object does." luaur-rt's `Lua` is `!Send` and keeps per-VM state in
thread-locals (`INTERRUPTS` in `interrupt.rs`, `MEMORY_CONTROLS` in
`memory.rs`, app data), so a VM built with it cannot be moved to the audio
render thread. The engine makes 171 raw calls from 33 raw functions and
declares `unsafe impl Send for RawProcessor` (`:715`).

| # | Site | Raw operations | Disposition |
|---|---|---|---|
| R33 | `:715-760`, `:794-950` lifecycle: `Drop` (`lua_close`), `allocate` (custom `lua_Alloc` with a byte budget and an `oom` flag over `l_alloc`), `lua_newstate(allocate, ctx)`, `lua_callbacks()->userdata = ctx`, `lua_callbacks()->interrupt` (cancel flag and deadline, raising preallocated registry error strings), `luaL_openlibs`, `print`/`require`/`Audio` globals via `lua_pushcclosurek`, `luaL_sandbox` | **reimplement** on a `Send` VM with a per-VM allocation budget (section 6, items 9–10) |
| R34 | `:1106-1131` `load_module` | `luau_load(name verbatim)`, `lua_pcall`, `lua_ref` cache, cycle detection | **reimplement** (uses `load_bytecode_as`) |
| R35 | `:953-1010` `process_block` | registry refs via `lua_rawgeti`; `lua_tobuffer` raw pointer zeroed with `write_bytes` before the call and read as `f32` after; `lua_pcall`; **`lua_gc(l, 6 /*LUA_GCSTEP*/, 0)` after every block** | **reimplement** (needs `Buffer` byte access and an explicit GC step) |
| R36 | `:1058-1105` `print`/`require`; `:1135-1210` `push(SharedValue)`; `:1211-1626` 22 C functions (`streamReader`, `processorSampleRate`, ring `send`/`receive`/`receiveInto`, snapshot `publish`/`latest`, blob `read`/`toBuffer`, region `atomicLoad`/`Store`/`Add`/`Cas`/`exclusive`, field handlers) | `lua_createtable`, `lua_rawset(i)`, `lua_setmetatable`, closures with one upvalue (`lua_upvalueindex(1)` → channel index), `lua_next` to decode tables, `lua_topointer` as table identity, `lua_tonumberx`, `lua_toboolean`, `lua_newbuffer`/`lua_tobuffer` | **reimplement** on the handle API (`create_function` with captured index, `Table::pairs`, `to_pointer`) |

### 2.3 Test-only raw sites

| # | Site | Raw operations | Disposition |
|---|---|---|---|
| T1 | `vm.rs:4382-4494` | `lua_pushlstring`, `lua_tostringatom`, `lua_pop` (asserts the atom resolver returns 237) | **drop** with R4 |
| T2 | `vm/lua_path.rs:485-540` (`exec_raw` 488) | capture `lua_State*`, `lua_gettop` before/after | **drop**: stack-balance assertions have no meaning without a C stack |
| T3 | `vm/lua_data_value.rs:28-62` (`exec_raw` 31) | same | **drop** |
| T4 | `vm/view_model.rs:4196`, `:4231` | `lua_gettop(thread.state())` | **drop** |
| T5 | `vm/property_lifetime_a637_tests.rs:225` | `lua_gettop` | **drop** |
| T6 | `vm/artboard_context_8f06_tests.rs:15-84` | `lua_getthreaddata`, `lua_setthreaddata(null)`, `lua_settop`, `thread.state()` | **drop** with R3 |

### 2.4 Disposition counts

| | lift | reimplement | drop | total |
|---|---:|---:|---:|---:|
| Product rows | 22 (R1, R6–R23, R30–R32) | 10 (R24–R29, R33–R36) | 4 (R2–R5) | 36 |
| Test-only rows | 0 | 0 | 6 | 6 |

The 22 lift rows collapse to six API additions (section 6, items 1–6).

## 3. Host callbacks and VM-level contracts

### 3.1 Interrupts, timeouts and the safepoint budget

- The main VM installs one interrupt in `ScriptVm::new` (`vm.rs:2184-2193`)
  and swaps in a scoped one for module execution
  (`start_module_execution`, `vm.rs:2273-2296`). `gpu_canvas.rs:2338` and
  `vm.rs:2318` install variants for limit-configured VMs.
- Each interrupt checks the module deadline
  (`ModuleExecutionTiming::check`, `vm.rs:863`, strict "elapsed whole
  milliseconds > timeout", text `execution exceeded N.N second(s) timeout`),
  then **counts safepoints**: 100,000 per cycle (`SCRIPT_SAFEPOINTS_PER_CYCLE`,
  `vm.rs:325`), plus an optional trusted-callback budget. Exceeding either
  raises a resource-limit error.
- That makes **the number of interrupt invocations observable**: which
  script runs out of safepoints, and where, depends on where the VM fires
  interrupts. luaur-vm fires them at `CALL`, `CALLFB`, `RETURN`, `FORNLOOP`,
  `FORGLOOP`, `JUMPBACK` and `JUMPX` (`luau_execute.rs:1171, 1306, 1450,
  3142, 3322, 3711, 3738`) and inside string pattern matching
  (`functions/match.rs:29-31`). GC-step interrupts (`gc >= 0`, from
  `lua_c_step.rs`) are filtered out by luaur-rt's trampoline
  (`interrupt.rs`, `interrupt_trampoline`). The new VM must fire the same
  safepoints, not merely "support interrupts".
- Errors returned from the interrupt are raised as Lua errors on the
  running thread. `VmState::Yield` is never returned by product code.
- The audio engine's interrupt (`luau_audio_processor_engine.rs:759-777`)
  is called with the raw `gc` argument and does not filter GC steps; it also
  calls itself explicitly between phases.

### 3.2 Atoms and namecall dispatch

- `useratom` is installed (R4) with Rive's 300-entry compile-time table
  (`RIVE_LUA_ATOMS`, `vm.rs:329-693`, FNV-1a open addressing at
  `vm.rs:720-799`; ids 283–363 must exist, `vm.rs:716`). Atoms are assigned
  when strings are interned.
- **Nothing in product code dispatches on an atom.** Method calls on
  luaur-rt userdata compile to `NAMECALL`, find no `__namecall`, and fall
  back to the `__index` Lua closure (`USERDATA_DISPATCH`,
  `vendor/luaur-rt-0.1.8/build.rs:16-33`), which looks the name up in the
  getters table and then the methods table. The four hand-written
  `__namecall`s (R21–R23, R28) compare strings.
- So the contract is only the *string-level* behavior: method lookup by
  name, `X is not a valid method of T` texts where a type defines a strict
  `__namecall`, and `attempt to set unknown field '%s' on userdata` from the
  dispatcher. An atom-based fast path in the new VM is a free perf choice.

### 3.3 Thread data and coroutine inheritance

- C++ stores the `ScriptingContext` in `lua_setthreaddata` and copies it to
  promise coroutines and module threads. Rust stores the equivalent in app
  data (`set_app_data`/`app_data_ref`, 26/57 product sites), which is per-VM,
  not per-thread.
- The luaur-rt patch that copies thread data across `call_async`
  (`function.rs:192`) and `create_module_thread` (`thread.rs:617`), and R3's
  copy, always copy `null` in product: no product code writes a non-null
  thread data pointer. **The inheritance contract is vacuous today.** What
  *is* required: values and handles created on one coroutine stay usable from
  any other thread of the same VM (section 3.9), and module threads run with
  their sandbox environment as globals (`create_module_thread`).
- `call_async` itself has no product caller.

### 3.4 Userdata direct-field access

- The only registration is Mat4 (R29): 16 number getters `m11`…`m44` on tag
  `LUA_T_COUNT + 62`. C++ also registers direct fields for Paint and Mat2D;
  the Rust port does not, and `tests/upstream_scripting_direct_field_flag.rs`
  keeps the related benchmark expected-red for that reason.
- Mechanism: under `FFlag::LuauDirectFieldGet` (on in the product profile,
  toggled by `luaur_rt::set_luau_direct_field_get` in tests), `GETTABLEKS` on
  a userdata whose tag has a direct-field table calls the registered C getter
  with the payload pointer and writes the result into the destination
  register, skipping `__index` (`luau_execute.rs:504-540`). It needs no
  special bytecode: any bytecode version takes this path.
- The upstream direct-access opcodes `GETUDATAKS`, `SETUDATAKS` and
  `NAMECALLUDATA` exist in the opcode enum and the interpreter, but the
  compiler never emits them (no match in `vendor/luaur-compiler-0.1.8/src`)
  and the product never calls `lua_registeruserdatadirectaccess`.

### 3.5 Memory

- Main VM: `Lua::set_memory_limit` with 16 MiB (`SCRIPT_VM_MEMORY_LIMIT_BYTES`,
  `vm.rs:324`) or the limits-configured value (`vm.rs:2308`), and
  `MAX_LUAU_VM_MEMORY_BYTES` for GPU canvas VMs (`gpu_canvas.rs:2335`).
  `Error::MemoryError` is classified as `ScriptResourceLimit::Memory`
  (`resource_limits.rs:134`).
- Audio VMs: a custom `lua_Alloc` with a per-VM budget and an out-of-memory
  flag (R33). **So an allocation hook is needed**, contrary to the plan's
  section 4 item 4; it can be a per-VM budget rather than a callback.
- Memory categories (`set_memory_category`, `lua_setmemcat`) are not used
  by the product.

### 3.6 Garbage collection

- **No product code forces a collection.** `gc_collect` has 26 call sites,
  all in tests: they prove userdata finalization (`view_model.rs:3716`,
  `:3735`, `:4475`), VM teardown with a live `File`
  (`artboard_context_8f06_tests.rs:85-92`), and scripted property lifetime
  (`property_lifetime_a637_tests.rs`, 13 sites).
- The audio engine runs `lua_gc(LUA_GCSTEP, 0)` after every rendered block
  (`luau_audio_processor_engine.rs:1000`) to keep collection work bounded on
  the audio thread.
- `tools/silver-corpus/src/scripted_misc_replay.rs:104` runs a full
  `LUA_GCCOLLECT` per replayed frame.
- Collection semantics the product relies on: weak-key tables (the Promise
  engine's `states`, `lua_promise.rs:138`, `__mode = "k"`); userdata
  destructors running Rust `Drop` at collection or `lua_close`, without
  re-entering Lua (`lua_main_ref.rs:22-33`); and no handle cycle keeping the
  VM alive after `ScriptVm` drops (asserted with `WeakLua::try_upgrade` at
  `vm.rs:1065-1070`, `:1119`).

### 3.7 Error text and chunk names

The error *text* is part of the observable surface: scripts can `pcall` and
print it, and the log sink records it.

- Chunk names are passed verbatim (R1, R34) and printed raw by Rive's
  `luaL_where` as `chunkname:line: message`.
- The product parses chunk names back out of debug info:
  `normalize_chunk_source` strips a leading `=` or `@` (`vm.rs:1755-1760`),
  `caller_chunk_source` walks `inspect_stack` past `[C]` frames
  (`vm.rs:1762-1775`), and `lua_blob.rs:133-141` takes the library scope
  from a `label@version/path` chunk name. R2 indexes the dependency registry
  by the raw `source`.
- String matching on VM error text in product code: one site,
  `view_model.rs:2494`, replaces `'luaur-rt-exec-raw'` with
  `'globalViewModel'`. Every other `contains(...)` on error strings is in a
  test.
- Host function debug names reach `luaL_argerror` texts. luaur-rt names
  every `create_c_function` `"luaur-rt-c-function"` and every `exec_raw`
  trampoline `"luaur-rt-exec-raw"` (`exec_raw.rs:130`, `:182`), so the 20
  Mat4 functions report argument errors under that name today. The new VM
  should let host functions carry their C++ names.
- `Error::runtime` is raised as a Lua error and surfaces through
  `CallbackError`; `Error::lua_l_runtime` matches `luaL_error`. The
  `logging_scripting_context.rs:17-20` printer reads `RuntimeError`,
  `SyntaxError` and `MemoryError` messages and unwraps `CallbackError`.
- Base-library text and behavior is the baked-on `RIVE_LUAU` profile from
  fork rung 9: no `print` (the host installs one), no `newproxy`, raw
  source prefixes, level-0 `luaL_where` retry.

### 3.8 The process-global flag profile

`Lua::new` calls `luaur_common::set_all_flags(true)` (`state.rs:180`), and
the validator scopes the same profile (`validate_executable_luau_bytecode`,
`vm.rs:3129`). The VM-visible part of that profile is a behavior contract
the new VM hardwires:

| Flag | Value | Effect |
|---|---|---|
| `LuauCallFeedback` | on | v11 bytecode loads (validator rejects v11 without it, `vm/bytecode.rs:254-259`) |
| `LuauDirectFieldGet` | on | Mat4 direct-field fast path |
| `LuauIntegerLibrary` | on | `integer` library and buffer integer functions are opened (`lua_l_openlibs.rs:130`, `luaopen_buffer.rs:258`) |
| `LuauIntegerFastcalls`, `LuauIntegerBufferFastcalls` | off | integer builtin fastcalls are never emitted |
| `LuauMathRoundNegZero`, `LuauGcTraceUdata`, `LuauBackedgeHeapCheck`, `LuauXpcallFixMessageYieldPath`, `LuauGcMarkUdataAccess` | off | upstream behaviors kept dark |

### 3.9 Handle lifetime across threads

luaur-rt handles are bound to the `lua_State` that created them; using one
from another coroutine is the UNIV-1764 bug class. The product works around
it in three ways: R6–R9 (`MainThreadRef`), Lua-closure dispatchers
(the UNIV-1764 patch), and resolving `tostring` from the invoking thread in
`print`. The new VM's handles should be VM-scoped by construction, which
retires all three.

## 4. Bytecode: what is emitted versus admitted

**Admitted.** `LBC_VERSION_MIN = 3`, `LBC_VERSION_MAX = 13`,
`LBC_VERSION_TARGET = 7`, `LBC_VERSION_CLASSES = 100`; type versions 1–3
(`vendor/luaur-common-0.1.8/src/enums/luau_bytecode_tag.rs:15-25`). The
product validator (`vm/bytecode.rs:57-90`) admits 3–13 and rejects 100; the
VM loader (`luaur-vm/src/functions/loadsafe.rs`) also admits 100. Format
gates: type info from v4, feedback slots from v11 (`loadsafe.rs:579`), proto
size and cost from v12 (`:210`, `:601-605`), `LBC_CONSTANT_VECTORD` with v13.
Note that versions 7–13 are upstream Luau versions carried in the Rive fork,
not Rive inventions; the Rive-specific extensions are listed below.

**What picks the version.** `BytecodeBuilder::get_version`
(`luaur-bytecode/src/methods/bytecode_builder_get_version.rs`): 100 if
`DebugLuauUserDefinedClasses`, else 13 if `LuauCompileEmitVectorDouble`, else
12 if `LuauBytecodeCostModel`, else 11 if `LuauEmitCallFeedback`, else 7. With
`set_all_flags(true)` the first three are off and `LuauEmitCallFeedback` is
on, so **every compile in this repo emits v11**. With all flags off it emits
v7.

**Compiler options in this repo.** No code uses luaur-rt's `Compiler`
builder (`set_optimization_level`, `set_vector_ctor`, `set_userdata_types`,
`libraries_with_known_members`: zero matches outside the vendored crate).
Every compile calls `luau_compile(src, len, null, …)`, which uses
`CompileOptions::default()`: optimization 1, debug 1, type info 0, no vector
library or constructor, no userdata types
(`luaur-compiler/src/records/compile_options.rs:26-28`).

**Who emits what.**

| Producer | Where | Flags | Version |
|---|---|---|---|
| Nuxie Editor (`scripted-resource-compiler` via `editor-publisher-wasm`) | **nuxie-dev**, not this repo | all Luau flags forced off (`DeviceLuauFlagsGuard`), per the matrix in `docs/luau-fork.md:311-370` | **7** (row `editor_scripted_vector_v7`) |
| Historical editor assets | `script_artboard_test.riv`, `script_dependency_test.riv` | — | 6 |
| Real Rive Editor assets | `fixtures/sync/data_bind_blob_test.riv`, `fixtures/sync/scope_probe.riv` (149 scripts) | — | 7 |
| Runtime-owned helper chunks, compiled at **build time** | `vendor/luaur-rt-0.1.8/build.rs` (`bind`, `table-len`, `userdata-dispatch`, `async-poller`) and `crates/nuxie-scripting/build.rs` (`promise-library`, `data-value-metatable`, `property-metatable`) | `set_all_flags(true)` | **11** |
| Tests, silver corpus producers, `nux-capi`/`nuxie` dev tests | `luau_compile` + `set_all_flags(true)` | | 11 |

The version bytes for the four historical rows are pinned by
`crates/nuxie-scripting/tests/corpus_scripts.rs:269`
(`editor_bytecode_matrix_rows_extract_pin_version_and_load`).

**Rive extensions, emitted vs admitted.**

| Extension | Emitted today? | Evidence |
|---|---|---|
| `CALLFB` / call feedback (v11) | Only in this repo's build-time helpers and tests. Not by the editor (v7, flags off). Emitted only for calls inside nested functions, not depth-0 chunks (`compiler_compile_expr_call.rs:266-271`) | the device must load v11 for its own seven helper chunks, unless they are recompiled at v7 |
| Rive builtin fastcalls: `math.fround` = 243, `Vector.*` = 245–255 | Yes, whenever authored code calls them: recognition is not flag-gated (`get_builtin_function_id.rs:95`, `:448-483`) and needs only optimization level ≥ 1 (the default). Whether the editor compiles at O1 is set in nuxie-dev | VM slots wired in fork rung 9 and "Fast-call dispatch enablement" |
| Integer builtins (`integer.*`, `buffer.readinteger`/`writeinteger` fastcalls) | **No**: both gating flags are forced off even under `set_all_flags(true)` (`luaur-common/src/lib.rs:107-109`) | the `integer` library itself is still opened (section 3.8), and `LBC_CONSTANT_INTEGER` is admitted |
| Userdata direct access | No bytecode form is emitted (section 3.4); the direct-field getter path is a VM-side dispatch on `GETTABLEKS` | Mat4 only |
| User-defined classes (v100, `NEWCLASS`) | No; rejected by product admission | `docs/luau-fork.md:45` |
| Double vectors (v13, `VECTORD`) | No (flag off) | |
| Cost model (v12) | No (flag off) | |

Implication for D5: the device needs v6 and v7 for authored content and v11
for runtime-owned helpers. Compiling the seven helpers with the
flags-off profile would let the new VM's loader start at v3–v7 plus the
FASTCALL builtin slots, and add v11 only if the editor moves.

## 5. Observable streams used as oracles

`make scripted-golden-compare` runs `tools/golden-compare` over
`corpus.toml` with `--side-channel --verify-unsupported-cpp
--verify-divergent-rust --verify-scripted-diagnostics`, comparing the C++
scripted runner (`make scripted-golden-runner`, `tools/golden-runner` built
with `RIVE_GOLDEN_WITH_SCRIPTING=1`) against `tools/rust-golden-runner`
(`make scripted-rust-golden-runner`). 37 corpus rows carry a scripted status
(22 `scripted-status:exact`, 15 `scripted-status:diverges`). The recorded
stream is the `RecordingFactory` command stream (paths, paints, transforms,
frames) plus the side channel (`crates/nuxie-render-api/src/lib.rs:4215`;
`tools/rust-golden-runner/src/main.rs:1776-1870`): every advance with its
keep-going/settled flag and state-changed count, every reported event with
its custom properties, semantics diffs, hit results, input events and
view-model writes. Scripts are observed only through those effects: log
text is not in the stream, and `--verify-scripted-diagnostics` checks only
the `unsupported:` diagnostic marker on stderr. `make golden-compare` and
`make e2e-composed-compare` use the same tool and side channel.
`make silver-corpus` (`silver-corpus-validate`) replays 48 `scripted`-lane
cases in `silver-corpus.toml`, literal Luau producers from upstream unit
tests compiled at v11, through `ScriptVm`, with a full GC per frame in the
misc replay. The editor-bytecode matrix test
(`tests/corpus_scripts.rs:269`) and `crates/nuxie/tests/editor_bytecode_matrix.rs`
pin v6/v7 load and render. These are the differential oracles at cutover;
none of them records error text, so error-text parity (section 3.7) needs
its own tests.

## 6. Proposed embedding API additions

Signatures are in luaur-rt's style so the product compiles against either VM
during the flag-guarded cutover.

1. **Load under a verbatim name** (R1, R34):
   `fn load_bytecode_as(&self, chunk_name: &[u8], bytecode: &[u8]) -> Result<Function>`.
   No `=` prefix; validates per D7 before loading; globals as environment.
2. **Thread-independent registry references** (R6–R9):
   `fn create_registry_ref(&self, value: impl IntoLua) -> Result<RegistryRef>`,
   `impl RegistryRef { fn get<T: FromLua>(&self, lua: &Lua) -> Result<T> }`,
   `RegistryRef: Clone`, and its `Drop` neither allocates nor enters Lua and
   is valid from a userdata destructor or after the minting coroutine died.
   (If every handle in the new VM is VM-scoped, this is `RegistryKey` with
   those guarantees.)
3. **Userdata metatable access** (R10–R20):
   `fn AnyUserData::metatable(&self) -> Result<Table>` returning the shared
   per-type metatable, writable until `set_readonly(true)`.
4. **Current namecall name** (R21–R23, R28):
   `fn namecall_method(&self) -> Option<LuaString>`, `Some` only inside a
   `__namecall` invocation.
5. **`luaL` argument checks with exact texts** (R25–R27, R30):
   `fn check_arg<T: CheckArg>(&self, args: &MultiValue, n: usize) -> Result<T>`
   producing `luaL_typeerror`/`luaL_argerror` messages, with `CheckArg` for
   numbers, integers, strings (missing ≠ explicit nil), vectors, buffers and
   tagged userdata; plus **named host functions**
   `fn create_named_function<F>(&self, name: &str, f: F) -> Result<Function>`
   so the function name in those messages is the C++ one (section 3.7).
6. **Lua coercions** (R31, R32):
   `fn coerce_string(&self, value: Value) -> Result<Option<LuaString>>`
   (`lua_tolstring`, `%.14g` numbers) and
   `fn objlen(&self, value: &Value) -> usize` (`lua_objlen` over strings,
   tables, buffers, userdata).
7. **Tagged plain-data userdata with direct fields** (R24–R29):
   `fn register_tagged_userdata<T: Copy + 'static>(&self, tag: u8, metatable: Table, direct_fields: &[(&'static str, fn(&T) -> f64)]) -> Result<()>`,
   `fn create_tagged_userdata<T: Copy + 'static>(&self, value: T) -> Result<AnyUserData>`,
   `fn AnyUserData::tagged<T: Copy + 'static>(&self) -> Result<T>` and
   `fn AnyUserData::set_tagged<T: Copy + 'static>(&self, value: T) -> Result<()>`.
   Also the route by which Paint and Mat2D get direct fields later.
8. **Interrupt placement** (contract, existing API): `set_interrupt` and
   `scoped_interrupt` stay, and the VM fires them at exactly the luaur-vm
   safepoints in section 3.1, excluding GC steps.
9. **A `Send` VM for isolated lanes** (R33–R36):
   `fn Lua::new_isolated(options: IsolatedOptions) -> Result<IsolatedLua>`
   where `IsolatedLua: Send`, with no thread-local registries (interrupt
   closure, memory accounting and app data live in the VM), and
   `IsolatedOptions { memory_limit: usize, … }`.
10. **Memory and GC controls** (R33, R35):
    `fn memory_exhausted(&self) -> bool` after a protected call,
    `fn gc_step(&self, kbytes: i32) -> Result<bool>` (`LUA_GCSTEP`; luaur-rt
    already has `gc_step`), and
    `fn Buffer::with_bytes_mut<R>(&self, f: impl FnOnce(&mut [u8]) -> R) -> R`
    for the in-place output buffer.

Items 1–6 retire every lift row. Items 7 and 9–10 are what the new VM's
userdata and embedding models must provide to reimplement Mat4 and the audio
lanes without `unsafe`. The four drop rows need nothing new.

## 7. What this inventory could not determine

- Receiver-typed counts for the methods marked `≤` (section 0).
- The compile options the Nuxie Editor passes (optimization level, vector
  library): they live in nuxie-dev, so whether authored scripts carry the
  Rive `Vector`/`math.fround` fastcall ids is inferred, not observed.
- The bytecode version of every script in the 37 scripted corpus rows
  beyond the four pinned matrix rows: no tool in this repo scans them.
