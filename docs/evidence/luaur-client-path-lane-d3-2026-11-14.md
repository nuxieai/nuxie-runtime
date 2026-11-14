# Luaur client path, lane D3: the nuxie-scripting core (2026-11-14)

Lane D3 of `docs/luaur-client-path-safe.md` section 6. Branch
`claude/luaur-lane-d3-core-b5f848`, based on `levi/luaur-client-stage1c`
(main at d15e2cfbeb). Nothing is pushed.

## Files taken

These are all under `crates/nuxie-scripting/src/`:

- `vm.rs`, `vm/view_model.rs`, `vm/bytecode.rs`, `host_commands.rs`
- `vm/listener_invocation.rs`, `vm/resource_limits.rs`
- `vm/command_server.rs`, `envelope.rs`
- Read but unchanged: `vm/native_registration.rs`, `vm/buffer_ext.rs`, `lib.rs`, `luau/`. These already had 0 sites.
- Test callers that follow the signature changes: `tests/vm_boot.rs`, `tests/generic_host_commands.rs`, `tests/recursion_and_hang_tests.rs`, `src/vm/property_lifetime_a637_tests.rs`

No D2, E2b, audio, GPU or ratchet file was edited.

## Shape changes (one commit each)

| Commit | Change |
|---|---|
| 58883f2255 | `bytecode.rs`: the validator reads through one `Cursor { data, rest }`. It uses `split_first_chunk`, `split_at_checked` and `get`. The shift tables use `checked_shl`, and the line-gap intervals use `checked_sub`/`checked_shr`/`checked_add`. Every error text and every accept/reject decision is unchanged. A dead `abs_offset` computation was removed. |
| baa4a891f7 | Hostile-input tests. Every strict prefix of a real chunk is rejected. Every single-byte corruption (0x00/0x7f/0x80/0xff at each offset) returns without panicking. |
| 9d37a7592f | `LuaScriptInstance::table` returns `Result<&Table, ScriptError>` ("scripted object has no live Lua table"). `ScriptVm::set_view_models` returns `luau::Result<()>`. |
| 2fe024b793 | `vm.rs`: the safepoint and budget counters use checked arithmetic and keep the same error text. Capacities and timeouts saturate. The atom table is `#[cfg(test)]`, because only its test reads it. |
| 6bec0c3a5b | Row 25: `install_native_file_assets` registers each shader once (same name and short name). |
| 98e1838818 | `view_model.rs`: two traits replace the `unreachable!` matches over wrapper kinds. `PropertyWatchKind` covers the watch types and `WatchedProperty` covers the wrappers. Symbol-list indices return an ordinary error. Argument checks use slice patterns. Index conversions are checked. |
| b43a15d14c | `view_model.rs` Part 2: rows 10, 11 and 21–22. See below. |
| 5f76b3906c | `listener_invocation.rs`: button bits use checked shifts. `pointer_event_kind` returns an `Option`. `envelope.rs`: the signature is read with `split_first_chunk`. `command_server.rs`: `adopt` assigns without indexing. |
| b8875a04db | Row 26: host-command payload tables are walked with `next`, one entry at a time. Depth counters saturate. |
| f171ede2de | xpcall: guards outlive their call without becoming destructed-callback objects. The first version used one shared trampoline. |
| 0b3d628b3b | codex round 1: host_commands keeps the builtin `next` in the registry. Captured in the closure, it kept the VM alive. Adds a lifecycle test. |
| f3baf2e229 | codex round 1: after a prune, the watch list reserves room for as many entries as are live, so each scan is paid for by the pushes since the last one. Without this, one dead entry per prune made every push rescan the list. |
| 49b6f5928d | codex round 2: each xpcall gets its own guarded handler, found by id, in place of the shared stack-top trampoline. |

## Forced panic-lint counts

Measured with `tools/ratchets/panic-lints.sh --baseline <scratch> crates/nuxie-scripting 'crates/nuxie-scripting:nuxie-scripting__publisher@wasm'`, counted per file from the clippy diagnostics.

| File | Native before | Native after | Wasm before | Wasm after |
|---|---:|---:|---:|---:|
| vm.rs | 37 | 0 | 37 | 0 |
| vm/bytecode.rs | 21 | 0 | 21 | 0 |
| vm/view_model.rs | 11 | 0 | 11 | 0 |
| host_commands.rs | 4 | 0 | 4 | 0 |
| vm/listener_invocation.rs | 4 | 0 | 4 | 0 |
| envelope.rs | 2 | 0 | 2 | 0 |
| vm/command_server.rs | 1 | 0 | 1 | 0 |
| native_registration.rs, buffer_ext.rs, luau/, lib.rs | 0 | 0 | 0 | 0 |
| **D3 total** | **80** | **0** | **80** | **0** |
| **Crate total** | **212** | **132** | **211** | **131** |

Remaining sites, all in files owned by other lanes:

- D2: lua_mat4 57, lua_mat4_safe 5, lua_vec2d 5, lua_mat2d 4, lua_canvas 3, lua_video 3, lua_blob 2, lua_artboards 1
- E2b: lua_text 41, lua_path 4, lua_mesh 3, lua_font 2, renderer 1, lua_image_decode 1 (native only; 0 in wasm)

The ratchet reports PASS ("counts went down"). `tests/ratchets/*` was not updated.

## Part 2 outcomes

### Rows 10–11 (view-model ScriptedContext and dispose)

- **Row 10.** `videoGroup` borrows the source once per lookup. `image` and `blob` read `source_file` first and release that borrow before `create_userdata`.
- **Row 11.** `dispose` takes the model out of the cell, releases the borrow, then drops it. The two `Drop` impls in `view_model.rs` (`ScriptedPropertyState`, `ScriptedPropertyOwnerScope`) use `try_borrow_mut`. They drop what they replace only after the borrow ends.

### `set_app_data` followed by expect

`try_set_app_data` already exists in `luau/facade/state.rs`. No site in the D3 files follows `set_app_data` with `expect` or `unwrap`: the sites in vm.rs and view_model.rs ignore the old value. Nothing needed changing.

### The xpcall starter

I did not use the Luau wrapper E2a proposed. A Luau wrapper adds a visible Lua frame to `debug.traceback` and `debug.info` inside the handler, which a script can observe.

Instead, each guarded xpcall pushes the script's handler, with a fresh id, onto a `HandlerStack`. It then creates a non-scoped host function for that call, which holds only the stack and the id. That function finds its own handler while the xpcall runs. An `ActiveHandler` guard removes the entry when the xpcall returns; its `Drop` uses `try_borrow_mut` and does not panic.

A guard kept past its xpcall (through `debug.info`) raises the ordinary error "xpcall error handler called outside its xpcall", not a destructed-callback error object. Inside its own xpcall it still reaches its own handler, exactly as the scoped per-call handler did. The function holds no Lua handle, so it cannot keep the VM alive. Lane C.1's loan forwarding is kept, because every guard is `create_function_lent` and forwards with `call_with`.

Tests:
- `xpcall_handler_kept_past_its_xpcall_raises_an_ordinary_error`
- `guarded_xpcall_handlers_nest_and_stay_host_frames`: `debug.info(2,"s")` inside a handler is still `[C]`.
- `a_kept_xpcall_guard_reaches_only_its_own_handler`: a kept guard fails during a later, unrelated xpcall, and an outer guard re-entered inside a nested xpcall reaches the outer handler. This test fails against the first version, which used one shared trampoline.

ERROR_CHAIN_STARTER now raises its error through a test host function (`structuredError`). The old starter depended on the scoped handler being destructed.

### Rows 21–23 (listener growth)

- **Rows 21–22.** At most 256 listeners per property (`MAX_LISTENERS_PER_PROPERTY`). Past that, `addListener` raises "addListener: a property holds at most 256 listeners". An owner's tracked-watch list drops dead `Weak` entries whenever it reaches capacity, then reserves room for as many entries as are live, so each push is amortised O(1).
  - Tests: `a_property_holds_a_bounded_number_of_listeners`, `an_owner_drops_entries_of_watches_that_are_gone`.
- **Row 23** is in nuxie-runtime and is reported, not fixed. `viewmodel_instance_value.rs` `add_delegate` pushes without pruning, and `native_view_model.rs` `property_dirt_sink` registers a delegate per watch.

### Rows 24–26 (host memory)

- **Row 24 needs E2b's budget.** These sites copy script-sized data into host memory and cannot be bounded without a byte budget:
  - data-value string setters
  - the Blob setter (copies the bytes)
  - list `push`/`insert` (instance growth)
  - `Data.X.new`

  Each needs a "charge N bytes or return an error" call at the copy. Nothing else in D3 is needed.
- **Row 25 is fixed.** Repeating `install_native_file_assets` registers each shader once. Test: `native_file_asset_tests::repeating_the_install_registers_each_shader_once`.
  - Side effect: the "available: …" list in the unknown-shader error no longer repeats names added by repeated installs.
- **Row 26 is fixed.** `table_value` walks a payload table with `next`, one entry at a time, and stops at the node budget. Before, it collected the whole table first.
  - Test: `a_huge_payload_table_is_walked_only_as_far_as_the_node_budget` (a table with a 1,000,000-slot hint; at most `max_nodes + 1` steps; the error contains "host value exceeds 4096 nodes").
  - Caveat: the host module captures the global `next` at install, so it assumes install runs before any script code. That is true for every installer today.

### Row 1: the listener `RefCell` panic, reproduced

This was reproduced with a temporary test, which was not committed. The test made a String property with a Lua listener that reads `property.value`, then wrote the property natively with `handle.with_downcast_mut::<ViewModelInstanceString,_>(|s| s.set_value(..))`. The result was a `RefCell already mutably borrowed` panic at `nuxie-runtime/src/mechanical_port/source/core.rs:1057` (`CoreHandle::with`, `slot.object.borrow()`).

The exact path, from the captured backtrace:

```
ViewModelInstanceString::set_value          (slot mutably borrowed by with_mut)
 -> property_value_changed -> ViewModelInstanceValue::on_value_changed
 -> NativePropertyDelegate::value_changed -> RuntimeCellDependent::add_dirt
 -> view_model_cell::invoke_before_notify
 -> ScriptViewModel::property_listener_sink closure
 -> nuxie_scripting view_model::property_watch -> call_property_listeners
 -> Lua listener -> ScriptedPropertyString `value` getter
 -> ScriptViewModel::string -> NativeScriptViewModel::property
 -> ViewModelInstance::property_value_named -> CoreHandle::with   PANIC
```

Today the facade trampoline's `catch_unwind` turns the panic into a listener error. Listener errors are ignored, so the script sees the read fail silently (`seen` stays nil). Under `panic=abort`, the client goal, this aborts.

The production trigger is a data-bind to-source write: `context_value/native_binding.rs` `apply_source_value` runs `mutate(|| source.with_mut(.. apply_value ..))` for String, Color, Boolean, Enum, SymbolListIndex, Artboard and the asset types. `mutate` arms `HOST_MUTATION_NOTIFICATIONS`. But `on_value_changed` defers only when `HOST_TRANSACTION_PUBLICATION` is set (`host_viewmodel/cell.rs` `defer_transaction_notification`), so the listener runs inside the borrow. Number is safe, because `ViewModelInstanceNumber::set_value_handle` releases the slot before dependents run. `ListenerViewModelChange::perform` reaches the same path through `DataBind::update_source_binding_handle`.

nuxie-runtime would need one of two changes:
- give every value type a `*_handle` setter that releases the slot borrow before `on_value_changed` notifies, as Number does; or
- have `on_value_changed` defer delegate notification whenever `HOST_MUTATION_NOTIFICATIONS` is armed, not only under a transaction publication.

Either is a nuxie-runtime change, outside D3.

### Drop, recursion, division

- **Drop.** I reviewed every `Drop` impl in the D3 files:
  - ModuleExecutionGuard
  - ScriptVm
  - LuaScriptInstance
  - RegistrationScope
  - the two view-model drops (fixed)
  - the four facade drops
  - the new ActiveHandler

  None can panic.
- **Recursion.** The only recursion is the host_commands value walk (`value_at_depth`/`table_value`) and `host_value_bytes`. Both are bounded by a maximum depth of at most 64, and the depth counters saturate.
- **Division.** The only division in shipped D3 code is `checked_div` in `buffer_ext.rs`.

### `PersistentRenderContext::adopt`

`adopt` no longer has a forced site, but it is still infallible. An error return would need changes outside D3:
- its caller, `route_to_import_factory`, is in D2's `lua_renderer_library.rs`;
- nuxie-runtime's `ScriptingVm::route_to_import_factory` trait method (`scripting.rs`, called from `file.rs`) has no error channel.

## Check results

Final run on tip 49b6f5928d (code), after `bazel clean`, 2026-11-14. Load averages are 1/5/15-minute values at the start of each run.

| Check | Result | Load at start |
|---|---|---|
| `tools/ratchets/panic-lints.sh` (native and wasm rows) | PASS, 132 / 131 sites, D3 files 0 / 0 | 6.43 9.53 14.98 |
| `runtime.py test -p nuxie-scripting` | 39/39 targets pass | 10.47 13.29 18.36 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42 pass; the one failure is `stateful_component_image_bind`, known red on main | 10.67 10.61 14.96 |
| `runtime.py test --locked -p nux-capi --no-default-features` | 25/25 pass | 9.56 10.85 14.37 |
| `make scripted-golden-compare` | 348 exact / 16 diverges, exit 0 | 13.25 12.54 14.58 |
| `make luau-conformance` | PASS: 37 required OK, 0 required not passing | 10.50 11.93 14.17 |
| `make luau-vm-diff` | 409 runs, 0 disagreements, bytecode versions 7 and 11 | 10.30 11.86 14.14 |

An earlier full pass, at tip f171ede2de before the codex fixes and under load averages of 22–60, gave the same results.

Free disk stayed at or above 137 GB.

## Not verified

- Row 1 was reproduced with a direct native write, not with an authored data-bind fixture driving `apply_source_value`. The data-bind path is established by reading the code: the same `with_mut` + `set_value` call, and no deferral outside a transaction publication.
- No device or wasm runtime run. The wasm row is clippy only.

## codex review

Three rounds of `codex review --base levi/luaur-client-stage1c`:

1. Two real findings, both fixed:
   - P1: the host module's captured `next` kept the VM alive.
   - P2: a low-yield prune made watch tracking quadratic.
2. One real finding, fixed: the shared xpcall trampoline sent a kept guard to whichever handler was on top of the stack.
3. Clean: "No actionable regressions were found."

## Where the design was wrong

- **E2a's xpcall fix.** The Luau wrapper it proposed would change what handlers see (an extra Lua frame). The native guard stack keeps the frames unchanged.
- **The `set_app_data` item.** No such site existed in the D3 files.
- **`adopt` cannot return an error inside D3.** Its error channel crosses D2's file and the nuxie-runtime trait.
- **Row 1 is not a nuxie-scripting bug.** The borrow is held by nuxie-runtime across its own delegate call. nuxie-scripting cannot release it, and catching the panic is the only defence it has today.
