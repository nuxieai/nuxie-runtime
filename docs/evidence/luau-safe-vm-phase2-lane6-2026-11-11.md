# Safe Luau VM, Phase 2 lane 6: the embedding API (2026-11-11)

Lane 6 of Phase 2 in `docs/luau-safe-vm-plan.md` ("How Phase 2 runs"): the
embedding API of `crates/nuxie-luau-vm`, built against the seam inventory
(`docs/luau-safe-vm-seam.md`, D5) so lane 7 can port `nuxie-scripting`
with zero raw hatches. Branch `claude/luau-vm-lane6-embedding-f599cf`,
rebased onto `origin/main` `e8c3cb91d5`; not pushed, no PR.

**Outcome.**

- **The embedding API exists and is safe.** It has:
  - rooted, VM-scoped handles for tables, functions, strings, userdata,
    threads and buffers;
  - typed conversions (`IntoLua`/`FromLua` and the multi-value forms);
  - host functions in three shapes: typed closures, raw stack functions,
    and stack functions with a continuation;
  - yields, `callyieldable` and `pcallyieldable` from host functions;
  - typed userdata with per-type and per-tag metatables, `luaur-rt`-style
    field and method dispatch, and direct fields;
  - the interrupt, named and keyed registry values, app data, and a
    settable memory limit with a sticky exhaustion flag.

  The crate keeps `#![forbid(unsafe_code)]` and the seven forced panic
  lints denied, with no `allow`.
- **Conformance on nuxie: 36 of 55 required** (31 before). The runner now
  installs upstream's host helpers through the embedding API. New:
  `cyield`, `userdata`, `udata_direct`, `native_userdata`, `vector`.
  `tables` now stops only at a D4 hash-order site.
- **Differential host surface.** The same host functions, userdata type,
  interrupt and memory limit are registered on both VMs, through
  `luaur-rt` and through the new API. A corpus case drives them:
  174 runs, 0 disagreements, 0 drift.
- **Two VM fixes found by the conformance helpers.** Each matches C++:
  - GETTABLEKS reads vector `x`/`y`/`z` before any metatable;
  - `namecall_method` does not depend on the callback's debug name.

  A third came from checking a comment: the unknown-field store text
  renders its key with `tostring`, as `luaur-rt`'s dispatcher does.

## 1. Commits

| Commit | Concern |
|---|---|
| `a1beab3833` | full userdata as an arena object with a typed host payload (`Rc<RefCell<T>>`), tag, metatable and size charge |
| `2d9b2f7ad3` | settable memory limit and a sticky exhaustion flag in `budget.rs` |
| `99f10e636c` | host functions with continuations, yields and error values (`HostK`, `HostCont`, `Request::Yield`) |
| `17c48163da` | the interrupt, direct fields and embedding roots in the VM (GC roots for handles and type/tag metatables; the interrupt inside `string` pattern matching) |
| `b1dcfae80b` | the embedding API: `handle.rs`, `convert.rs`, `embed.rs` |
| `4b63dd0c35` | crate unit tests for the API (`embed_tests.rs`, 19 tests; 23 after review) |
| `e3bbbe4e4b` | `luau-vm-diff`: the shared host surface and `host_surface.luau` |
| `834ab1d037` | `luau-vm-diff`: the API under real scripts (`host_api_tests.rs`) |
| `836e1ad4b9` | fix: GETTABLEKS vector components before any metatable |
| `4e82afb2fe` | fix: `namecall_method` independent of the debug name |
| `d7e2953529` | `luau-conformance`: upstream host helpers for `--vm nuxie` |
| `35c1d607b0` | `scope-nuxie.txt`: five scripts to required |
| `7251775a0a` | fix: unknown-field store renders the key with `tostring` (as luaur-rt's dispatcher) |
| `a0ef30c8cf` | clippy type aliases in the conformance helpers |
| `3b09a5a525` | review fix: converted keys and values stay rooted until stored |
| `572538c32e` | review fix: `call_yieldable` without a continuation returns the call's results |
| `cc55ab9115` | review fix: exact, range-checked integer conversions |
| `3eadfcaa3e` | review fix: `Table::pairs` roots its snapshot before converting |
| `a48deaa79d` | plan doc: Phase 2 notes for lanes 2 to 6 |

This evidence doc is the last commit.

## 2. API map

All of it is in `crates/nuxie-luau-vm/src/{handle,convert,embed,host}.rs`
and exported from `lib.rs`.

### Handles and values

- `Table`, `Function`, `LuaString`, `AnyUserData`, `Thread`, `Buffer`:
  each is an `Rc` marker into the VM's `HandleTable`.
  - A live slot is a GC root.
  - A dropped handle frees its slot without allocating; dead slots are
    reclaimed at the next root scan or when the table has doubled.
  - Handles are VM-scoped, not thread-scoped. A handle minted on a
    coroutine works on any thread of the same VM, which is the
    UNIV-1764 class (seam 3.9).
  - A handle from another VM is `Error::Host`.
- `Value` (public) holds handles. `LightUserData { ptr, tag }`.
  `MultiValue(Vec<Value>)`. `RegistryKey`.
- Conversions:
  - `IntoLua` and `FromLua` for `Value`, `bool`, the float and integer
    types (integers truncate, with a range check), `&str`, `String`,
    `&[u8]`, `[f32; 3]` (vector), `LightUserData`, the handles,
    `Option<T>`, `Vec<T>` (sequence table) and `HostValue`;
  - `IntoLuaMulti` and `FromLuaMulti` for `()`, tuples up to 8 and
    `MultiValue`, and as a blanket impl over single values.
  - A failed conversion is `Error::Conversion { from, to, position }`.
    Inside a typed host function it becomes Rive's
    `invalid argument #n to 'name' (X expected, got Y)`.

### `Vm`

| Area | Methods |
|---|---|
| Creation | `globals`, `create_table`, `create_table_with_capacity`, `create_sequence_from`, `create_string`, `create_buffer`, `create_buffer_from` |
| Functions | `create_function(name, Fn(&mut HostCtx, A) -> Result<R, HostError>)`, `create_host_function(name, f)` (stack API), `create_host_function_k(name, f, k)` (with continuation) |
| Loading | `load(chunk_name, bytecode)`: the name verbatim, as seam item 1's `load_bytecode_as`. `load_with_env(name, bytecode, env)` |
| Threads | `create_thread(f)`, `create_thread_with_env(f, env)` (module threads), `current_thread`, `main_thread`, `state_identity` |
| Userdata | `create_userdata`, `create_userdata_with_size`, `create_userdata_tagged`, `set_userdata_metatable::<T>`, `set_tag_metatable(tag)`, `register_userdata_type(UserDataType<T>) -> Table`, `register_direct_fields::<T>(tag, &[("m11", fn(&T) -> f64)])` |
| Registry | `set_named_registry_value`, `named_registry_value`, `unset_named_registry_value`, `create_registry_value -> RegistryKey`, `registry_value`, `remove_registry_value` |
| App data | `set_app_data`, `app_data` (`Rc<T>`), `remove_app_data` |
| Interrupt | `set_interrupt(FnMut() -> Result<(), HostError>)`, `remove_interrupt`, `replace_interrupt` (the scoped swap); each returns the previous one |
| Memory | `set_memory_limit(Option<usize>)`, `memory_limit`, `used_memory`, `memory_exhausted`, `clear_memory_exhausted`, `gc_step_kb`, `collect` |
| Values | `coerce_string`, `coerce_number`, `objlen`, `raw_equal`, `to_pointer`, `tostring`, `namecall_method`, `inspect_stack -> StackInfo`, `set_vector_metatable` |

### `HostCtx` (inside a host function; derefs to `Vm`)

- **Stack:** `top`, `set_top`, `pop`, `value(i)`, `get::<T>(i)`,
  `args::<A>()`, `args_from::<A>(first)`, `push`, `push_results`,
  `push_copy`, `replace`, `insert`, `remove`. Negative indices count from
  the top.
- **Calls and yields:**
  - `call_yieldable(nargs, nresults)` is `lua_callyieldable`;
  - `pcall_yieldable(nargs, nresults)` is `lua_pcallyieldable` without a
    handler;
  - `yield_values(n)` is `lua_yield`;
  - `is_yieldable`.

  The continuation gets `CallStatus::Ok` or `CallStatus::Error(kind)`.
- **Checks:** `check_number`, `check_integer`, `check_unsigned`,
  `check_boolean`, `check_vector`, `check_any`, `check_buffer`,
  `check_userdata::<T>`, `to_userdata_tagged::<T>`.
- **Errors:** `error(message)` is a positioned `luaL_error`. `arg_error`
  and `type_error` give the `luaL` texts.
- **`HostError`:**
  - `new` is `luaL_error` (positioned);
  - `raw` raises the bare text;
  - `raw_bytes`, `memory` (an out-of-memory error);
  - `from_value` raises any Lua value;
  - `From<Error>` carries a nested Lua error's text and kind.

### Handle methods

- **`Table`:** `get`, `get_field`, `set` (metamethods), `raw_get`,
  `raw_set` (refuses a read-only table), `raw_len`, `len` (`__len`),
  `raw_push`, `contains_key`, `pairs`, `sequence_values`, `clear`,
  `metatable`, `set_metatable`, `set_readonly`, `is_readonly`,
  `set_safeenv`, `to_pointer`.
- **`Function`:** `call` (`lua_pcall`), `environment`,
  `set_environment`.
- **`LuaString`:** `as_bytes`, `to_str`, `to_string_lossy`.
- **`Buffer`:** `len`, `to_vec`, `read_bytes`, `write_bytes`,
  `with_bytes_mut`.
- **`AnyUserData`:** `is`, `cell`, `borrow`, `borrow_mut`, `get_copy`,
  `set_copy`, `tag`, `metatable`, `set_metatable`. A conflicting borrow
  is an error, not a panic.
- **`Thread`:** `resume` goes through the pinned `coroutine.resume`
  builtin, so the error text and kind are the library's. Also `status`
  (returns `ThreadState`) and `reset`.
- **`UserDataType<T>`** mirrors `luaur-rt`'s registration:
  - `type_name` sets `__type`;
  - `add_field_method_get`, `add_field_function_get`,
    `add_field_method_set`;
  - `add_method`, `add_method_mut`, `add_function`;
  - `add_meta_method`, `add_meta_function`, `add_meta_host_function`.

  Fields and methods dispatch through two native closures (getters then
  methods; setters, else the raw `attempt to set unknown field 'k' on
  userdata`). They are VM functions, not Rust closures holding handles,
  so like the UNIV-1764 patch they run on whichever thread invokes them.

### Protection and limits

- Every API entry runs under `Vm::protect`, which unwinds the stack and
  the saved counters on error and renders the error. The result is an
  `Error`, never a panic. At top level (no frames, main thread) it then
  runs a safepoint.
- While the main thread is parked in a yield (`Vm::start` returned
  `Yielded`), API entries outside a host function return
  `Error::Host("... suspended in a yield")` until `resume` finishes it.
- Re-entry from a host function into the VM (`Function::call` and the
  rest) is bounded by `Limits::max_reentry`. Past it: `C stack overflow`.
- Userdata payloads are charged through `budget.rs` at their
  `size_of::<T>()` (or the given size) and credited when swept.
- Destructors run when the collector sweeps the userdata, and at VM drop.
  They run as Rust `Drop` of the payload: they never enter Lua and run
  exactly once.

## 3. Seam coverage

Section numbers are `docs/luau-safe-vm-seam.md`'s. "Covered" means an API
exists and a test exercises it. Where the seam doc and the product code
disagree, the product code wins, and the row says so.

### 3.1 Section 1.1, the handle surface

| Seam row | New API | Status |
|---|---|---|
| `Lua::new` | `Vm::new` + `sandbox`. The flag profile is hardwired (lane 1) | covered |
| `globals` | `Vm::globals` (the running thread's, the sandbox proxy after `sandbox`) | covered |
| `create_userdata` | `create_userdata` with a per-`TypeId` metatable from `register_userdata_type`; dispatch runs on the invoking thread | covered |
| `create_function` | `create_function` (typed), named | covered |
| `create_table`, `create_table_result`, `create_sequence_from` | same names; all fallible | covered |
| `create_string` | `create_string` | covered |
| `create_buffer`, `create_buffer_with_capacity` | `create_buffer(len)`, `create_buffer_from(bytes)` | covered |
| `create_module_thread` | `create_thread_with_env(f, env)` | covered. Thread-data inheritance is vacuous (seam 3.3), so nothing is copied |
| `create_thread` | `create_thread` | covered |
| `create_c_function` | `create_host_function` / `create_host_function_k`, named (seam 3.7: argument errors carry the real name) | covered |
| registry values (keyed and named) | `create_registry_value`/`registry_value`/`remove_registry_value`, `set_named_registry_value`/`named_registry_value` | covered |
| `set_app_data` / `app_data_ref` / `remove_app_data` | `set_app_data` / `app_data` (`Rc<T>`) / `remove_app_data` | covered. `app_data_mut` (unused) is not provided: `Rc<RefCell<_>>` app data does the same |
| `gc_collect` | `collect` | covered |
| `load_bytecode` | `load(name, bytecode)` takes the name verbatim. A caller that wants luaur-rt's `"=" + name` passes it | covered |
| `load` (source) | not provided: the crate has no compiler. Test-only in the product | n/a |
| `current_thread` | `current_thread` returns a `Thread` handle, not a raw state. Every product use was for a raw hatch | covered |
| `set_interrupt`, `scoped_interrupt`, `remove_interrupt` | `set_interrupt`, `replace_interrupt`, `remove_interrupt` | covered |
| `set_memory_limit` | `set_memory_limit(Option<usize>)` | covered |
| `state_identity` | `state_identity` | covered |
| `exec_raw` | none: no raw hatch exists | by design |
| `weak` / `try_upgrade` | not VM concepts (the product's `Rc` of its VM owner) | n/a |
| `unpack` / `pack` | `FromLuaMulti` / `IntoLuaMulti` | covered |
| `coerce_number` | `coerce_number` | covered |
| `inspect_stack` | `inspect_stack(level) -> StackInfo { source, short_src, current_line, is_native }` | covered |
| `sandbox` | `Vm::sandbox` (lane 1) | covered |
| `scope` | **not provided**. Callbacks are `'static`; `resource_limits` can capture `Rc` state instead (section 4) | lane 7 |
| `set_type_metatable::<Vector>` | `set_vector_metatable` | covered |
| `Table::get` / `set` / `get_field` | same; `get` honors `__index` (luaur-rt's rawget-when-no-metatable patch is the same observable result) | covered |
| `raw_get` / `raw_set` / `raw_len` / `raw_push` | same | covered |
| `len`, `clear`, `contains_key` | same | covered |
| `push`, `pop` (`Table`) | **not provided** as methods. `raw_push` covers the one product site (`lua_artboards`); `len` + `set` is the metamethod form | lane 7 |
| `pairs`, `sequence_values` | same | covered |
| `set_metatable`, `set_readonly`, `is_readonly`, `to_pointer` | same | covered |
| `Function::call`, `environment`, `set_environment` | same | covered |
| `Function::bind`, `wrap`, `call_async` | not provided (test-only or unused; the plan drops `call_async`) | dropped |
| `Thread::resume_module` | `Thread::resume`: a failed thread keeps its frames and reports `ThreadState::Error`. Not compared against the patch's exact frame-retention behavior | lane 7 to confirm |
| `Thread::state` | none (raw pointer) | by design |
| `Thread::resume`, `reset`, `status` | same | covered |
| `AnyUserData::borrow` / `borrow_mut` / `is` | same | covered |
| `UserDataMethods` / `UserDataFields` | `UserDataType<T>`. `add_field_index_method_get/_set` (1 site each) are not separate: use `add_meta_function("__index")` | covered, 1 gap |
| `Buffer::read_bytes` / `write_bytes` / `to_vec` | same | covered |
| `LuaString` accessors | same | covered |
| `Value::type_name`, `as_table` | `Value::type_name`, `as_table` | covered |
| `Vector::new` | `[f32; 3]` converts to and from a vector | covered |
| `MultiValue::from_vec` | `MultiValue(vec)` | covered |
| `RegistryKey`, `LightUserData` | same names | covered |
| `Error::runtime` | `HostError::raw`/`new` (see section 4 for the `runtime error:` prefix) | covered, text differs |
| `Error::lua_l_runtime` | `HostError::new` (positioned like `luaL_error`) | covered |
| `Error` classification | `Error::kind()`, `is_memory()`, `message()` | covered |
| `VmState::Continue` | `Ok(())` from the interrupt | covered |

### 3.2 Section 2, raw escape hatches

| Rows | New API | Status |
|---|---|---|
| R1, R34 | `load` (verbatim name) | covered |
| R2 | `create_function` + `inspect_stack` + `named_registry_value` + `HostError::new` | covered (rewrite in lane 7) |
| R3 | `create_thread` (thread data is vacuous) | covered |
| R4, T1 | useratom: no product reader | dropped |
| R5 | cannot occur (no shared parent C stack) | dropped |
| R6–R9 | VM-scoped handles and `RegistryKey`. A `RegistryKey` drop is allocation-free and thread-free (an `Rc` marker) | covered |
| R10–R20 | `AnyUserData::metatable` returns the shared per-type table, writable until `set_readonly` | covered |
| R21–R23, R28 | `namecall_method` (section 4) | covered |
| R24–R29 | `create_userdata_tagged`, `set_tag_metatable`, `register_direct_fields`, `get_copy`/`set_copy`, `HostCtx::check_*`, named functions | covered |
| R30 | `HostCtx::check_*` + named host functions | covered |
| R31, R32 | `coerce_string`, `objlen` | covered |
| R33 | per-VM interrupt, memory limit and exhaustion flag: covered. A `Send` VM: **not provided** | partial (section 4) |
| R35 | `gc_step_kb`, `Buffer::with_bytes_mut`, `memory_exhausted` | covered |
| R36 | `create_function` with captures, `Table::pairs`, `to_pointer` | covered |
| T2–T6 | none needed (stack-balance assertions) | dropped |

### 3.3 Sections 3 and 6

| Item | Status |
|---|---|
| 3.1 interrupt placement | The new VM fires at CALL, CALLFB, RETURN, FORNLOOP, FORGLOOP, JUMPBACK and JUMPX, and inside pattern matching, excluding GC steps. The interrupt counts agree with `luaur-rt` in the differential case (numeric for 11, generic for 6, while 6, calls 10, `string.find` pattern 904, `gsub` 14). An interrupt error stops the script and is catchable |
| 3.2 atoms and namecall | String-level only. `namecall_method` returns the method name |
| 3.3 thread data | vacuous; `call_async` dropped |
| 3.4 direct fields | `register_direct_fields` at GETTABLEKS, as C++. GETIMPORT paths (`return m.m11` on a global) go through `__index` on both VMs (`direct_fields_skip_index_for_their_keys`) |
| 3.5 memory | `set_memory_limit`, `memory_exhausted` |
| 3.6 GC | `collect`, `gc_step_kb`, destructors as `Drop`, weak tables (lane 3) |
| 3.7 error text | `luaL_where` positioning (D8), verbatim chunk names, named host functions |
| 3.9 handles across threads | VM-scoped (`handles_minted_on_a_coroutine_work_on_the_main_thread`) |
| 6.1–6.8, 6.10 | covered as above |
| 6.9 `Send` VM | not provided (section 4) |
| plan section 4: log sink | the product's `print` is a `create_function`; nothing VM-side |
| plan section 4: useratom / userthread | no product caller in the seam doc; not built |

## 4. Notes for lane 7

- **`runtime error:` prefix.** luaur-rt's `Error::runtime` reaches Lua as
  `runtime error: m`. `HostError::raw(m)` raises the bare `m`, and
  `HostError::new(m)` positions it like `luaL_error`. Where a product
  test pins the prefix, lane 7 writes `HostError::raw(format!("runtime
  error: {m}"))` or updates the expectation. luaur-rt's interrupt raises
  the bare text, as `HostError::raw` does.
- **Two `luaur-rt` texts this VM does not reproduce.** The differential
  case leaves them out by design:
  - a non-string error object coming back through a host call renders
    as `<non-string error>` in luaur-rt. Here it is C++
    `lua_tostring`'s failure text, `(error object is a table value)`;
  - a method called on the wrong receiver is `userdata type mismatch`
    in luaur-rt. Here it is `luaL_typeerror`'s
    `invalid argument #1 to 'm' (T expected, got X)`.

  A product test that pins either needs updating.
- **Unknown-field store** text agrees with luaur-rt:
  `attempt to set unknown field '<tostring(k)>' on userdata`, unpositioned.
  The comment that said luaur-rt prefixes it was wrong (fixed; the case
  now compares the full text for string, number, boolean and
  `__tostring` keys).
- **`namecall_method`.** It returns the name of the last NAMECALL
  dispatched through a `__namecall` metamethod while a host function is
  running. That is C++ `lua_namecallatom`. The seam doc asked for "`Some`
  only inside a `__namecall` invocation". The product calls it only
  there, so the difference is not observable. Gating on the callback's
  name (the first version) broke upstream's unnamed `__namecall`s.
- **Direct fields apply at GETTABLEKS only**, as in C++. A script that
  reads `Mat4.m11` through a global import path goes through `__index`.
- **No `Send` VM (seam item 9).** The VM holds `Rc` handles, `Rc`
  interrupt and app-data slots, and `Rc<RefCell<T>>` userdata payloads,
  so it is `!Send`. A `Send` variant would need `Arc` and `Mutex`, or a
  `Send` bound on every payload and closure. The choice was the safe,
  simple one: one `!Send` VM. Two alternatives for the audio lanes
  (R33–R36), both open:
  - build the VM on the audio thread and keep it there, moving only
    `Send` messages in and out. Nothing in the API needs to cross
    threads;
  - keep the audio engine on the vendored VM until a later lane adds an
    isolated `Send` VM.
- **`Lua::scope`** (one site, `resource_limits`) has no equivalent.
  Capture `Rc` state in a `'static` closure.
- **`Table::push`/`pop`** are not methods. Use `raw_push`, or `len` +
  `set`.
- **Chunk names:** `load` is verbatim. luaur-rt's `load_bytecode`
  prefixed `=`; the product's R1 hatch already avoided that.
- **Thread status.** `Thread::resume` returns the error of a failed
  thread with luaur-rt's kind, and `status` reports `Error`. Whether the
  product's `resume_module` frame retention is matched exactly was not
  tested here.

## 5. Tests

- **Crate unit tests** (`embed_tests.rs`, 23 tests; 30 in the crate). The
  four review tests are in section 7. The rest cover:
  - a handle keeps its object alive across full collections and releases
    it on drop;
  - destructors run exactly once, including at VM drop;
  - borrow conflicts are errors;
  - userdata memory charge;
  - tagged metatables;
  - typed-argument error texts;
  - host error statuses;
  - re-entry past `max_reentry` is `C stack overflow`;
  - a yield from a host call made by the host errors;
  - a thread yields from a host continuation and continues;
  - a failing thread;
  - `call_yieldable` and `pcall_yieldable` with an error;
  - the stack API;
  - table metamethods through handles;
  - VM-scoped handles;
  - registry and app data;
  - the memory limit with zero drift;
  - coercions.
- **`luau-vm-diff` scripted tests** (`host_api_tests.rs`, 13 tests; 53 in
  the tool):
  - the interrupt stopping a script, and the scoped swap;
  - an interrupt error caught by `pcall` with `luaL_error` position;
  - a parked main thread refusing API calls;
  - C++'s `multipleYields` through `coroutine.wrap`;
  - `__namecall` method names;
  - direct fields without `__index`;
  - `inspect_stack`;
  - `UserDataType` dispatch from scripts, including a borrow conflict on
    re-entry;
  - handles made on coroutines;
  - module environments;
  - the memory limit with zero drift;
  - table metamethods;
  - a host vector metatable with component access and an unnamed
    `__namecall`.
- **Differential** (`tests/luau-vm-diff/host_surface.luau` with
  `tools/luau-vm-diff/src/host_surface.rs`, run under all four compiler
  profiles):
  - typed host functions and host calls back into Lua;
  - errors through host calls;
  - re-entry six deep;
  - yields across a host call (both error);
  - userdata fields, methods and metamethods;
  - unknown-field stores;
  - destructor counts after collection;
  - interrupt counts per construct;
  - an interrupt limit inside a loop and inside pattern matching;
  - the memory limit.
- **Conformance helpers** (`tools/luau-conformance/src/nuxie_helpers.rs`)
  mirror `Conformance.test.cpp`:
  - `makelud`;
  - the cyield helpers (`singleYield`, `multipleYields`,
    `multipleYieldsWithNestedCall`, the five passthroughs,
    `pcallThenCall`, `pcallThenPcall`);
  - `int64`;
  - `setupVectorHelpers`;
  - `setupUserdataHelpers` (`vec2`, `vertex`).

  A C++ `LUAU_ASSERT` in a helper becomes a raised error. The scripts
  are unmodified.

## 6. Gates

On the final tip, rebased on `origin/main` `e8c3cb91d5`. Load is the
1-minute average when the gate started.

Run on `a48deaa79d`, 0 behind `origin/main`. The evidence commit after
it changes only this file. Free disk: 263 GB.

| Gate | Result | Load (1 min) |
|---|---|---|
| `python3 tools/bazel/generate.py --check` | pass | 10.6 |
| clippy `-D warnings --all-targets`: `nuxie-luau-vm`, `luau-vm-diff`, `luau-conformance`, `luau-bench` | clean | 10.6–11.4 |
| unit tests `nuxie-luau-vm` | 30 pass | 10.6 |
| unit tests `luau-vm-diff` | 53 pass | 11.6 |
| unit tests `luau-conformance` / `luau-bench` | 7 / 4 pass | 11.4 / 11.1 |
| `make luau-vm-diff` | 174 runs, 0 disagreements, every admitted opcode reached; accounting drift checked after every nuxie run | 11.1 |
| `make luau-conformance-nuxie` | PASS: 36 OK, 36 required, 0 required failing, 0 expected-fail passing | 8.4 |
| `make luau-conformance` (luaur) | PASS: 30 required, unchanged | 8.4 |
| `make luau-vm-ratchets` | PASS | 9.4 |
| `make ratchet-unsafe` | PASS; `crates/nuxie-luau-vm` 0 | 9.1 |
| `make ratchet-panic-lints` | PASS; `crates/nuxie-luau-vm` 0 sites | 9.1 |
| wasm32: `cargo build -p nuxie-luau-vm --target wasm32-unknown-unknown` with the stable toolchain's `rustc` (Bazel has no wasm target for the crate) | builds | 9.2 |

The first full run, before the review fixes, had two non-passing gates:

- clippy on `luau-conformance` (type complexity and let-and-return in the
  new helpers), fixed in `a0ef30c8cf`;
- the wasm32 build, because the default toolchain lacks the target. The
  rerun used `rustup which --toolchain stable rustc`, as
  `make crate-seams-browser-check` does.

## 7. Codex review

`codex review --base origin/main`, three rounds. Every finding was real and
is fixed with a test that fails on the code before the fix (checked by
reverting the fix and running the test).

- **Round 1, three findings:**
  - [P1] `Table::set`/`raw_set` (and `get`, `raw_get`, the registry
    setters) lowered a converted key to a raw id and dropped its handle
    before converting the value. A collection during that conversion
    swept the key string, and its slot was reused. Fixed: `lower_into`
    returns the rooting `Value` with the raw value, kept until the store
    (`3b09a5a525`). The same gap in `register_userdata_type`'s dispatcher
    upvalues was fixed in the same commit. Test:
    `a_key_stays_rooted_while_its_value_converts`.
  - [P2] `call_yieldable` without a continuation counted results above
    the original arguments, so a relay that called its own argument
    returned nothing. Fixed: count from the callee slot (`572538c32e`).
    Test: `call_yieldable_without_a_continuation_returns_the_call_results`.
  - [P2] Integer conversions went through `f64`, losing precision, and
    `MAX as f64` rounds up, so `i64` accepted 2^63 and `u64` 2^64.
    Fixed: `try_from` for integer values; exact power-of-two bounds for
    numbers (`cc55ab9115`). Test:
    `integer_conversions_are_exact_and_range_checked`.
- **Round 2, one finding:**
  - [P2] `Table::pairs` wrapped one pair at a time, so a conversion that
    allocated could collect a weak-value table's later entries and
    return handles to freed slots. Fixed: root the whole snapshot first
    (`3eadfcaa3e`). Test: `pairs_roots_its_snapshot_before_converting`.
    The first version of this test called `collect()` before the
    snapshot, which emptied the weak table, and passed on the broken
    code. The test now snapshots fresh, unreferenced entries.
- **Round 3:** no findings.

Before review, writing the conformance helpers found two VM gaps: the
vector component fast path and `namecall_method`'s name gate. Checking
the host-surface comment found a third, the key rendering. Section 4
covers all three.

## 8. Conformance scope

- Before: 31 required. After: 36. New: `cyield`, `native_userdata`,
  `udata_direct`, `userdata`, `vector`. The required set stays a superset
  of `scope.txt`'s.
- `tables` (permanent non-goal) now fails only at :815. That is the
  constant-key iteration-order block at :797-821, one of D4's three
  accepted hash-order sites. `makelud` (:684) passes.
- `udata_direct` runs upstream's `ValidateMetatable` subcase. The
  `DirectAccess` subcase registers `lua_registeruserdatadirectaccess` and
  a `useratom` callback; both are absent from the product (seam 3.2,
  3.4), so the API does not have them.
- Still not required, each with its reason in `scope-nuxie.txt`:
  - `coverage` (no coverage counters, no product caller);
  - `debugger` (breakpoint callbacks, no product caller);
  - `types` (the analysis frontend's RTTI);
  - `iter` (`newproxy`);
  - `native_types` and `native` (codegen);
  - the Rive `luaL_where` scripts (D8);
  - the profile non-goals.

## 9. What was not verified

- `nuxie-scripting` was not built against the new crate (lane 7). The
  seam coverage table is by API, not by a compiling port.
- The `Thread::resume_module` frame-retention patch was not compared.
- The `Send` VM (seam item 9) is not provided.
- The conformance helpers mirror upstream's C++ by reading it; the C++
  harness was not run beside them.
- Speed was not measured (no perf work in this lane).

## 10. What could still abort the process

Nothing new in the crate. It has no `unsafe` and no `allow` of the forced
panic lints. Host closures are user code: a host closure that panics
unwinds through the VM like any Rust panic. The VM does not catch panics;
`catch_unwind` would need `UnwindSafe` bounds across the API. Stack
depth from Rust recursion stays bounded by `max_reentry` (lane 4).
Running out of thread stack below lane 4's 1 MiB measurement remains the
host's configuration.
