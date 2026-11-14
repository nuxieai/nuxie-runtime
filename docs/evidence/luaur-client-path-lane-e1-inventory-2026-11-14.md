# Luau client path, lane E1: panic classes the lints do not see (2026-11-14)

Design: `docs/luaur-client-path-safe.md` section 7. Survey:
`docs/evidence/luaur-client-path-survey-2026-11-14.md` (S). Lane A:
`docs/evidence/luaur-client-path-lane-a-2026-11-14.md`. Tree: branch
`claude/luaur-lane-e1-inventory-1b363b` at `9eb238d58f` (lane A's tip).
Read-only lane: no source file changed. Paths are relative to
`crates/nuxie-scripting/src` unless they name another crate. Line numbers
are at `9eb238d58f`; every one carries its function name, because the
audio, GPU and renderer files are being rewritten in parallel.

Scope: shipped code (`cfg(test)`, `*_tests.rs`, `upstream_scripting_*`,
`tests.rs`, and items gated on `compiler`, `scriptnet`,
`upstream-test-seams` or `tools` excluded). Classifier: `classify.py` (in
the session scratchpad, not committed): it drops files reached only
through a gated `mod` line (16 files: the test files, `lua_rive_file.rs`,
`lua_scriptnet.rs`) and brace-matched items whose `#[cfg]` names `test` or
one of those features after `not(...)` arms are removed. On what it keeps,
`\.borrow\(\)` matches 175 times, `\.borrow_mut\(\)` 222 and `try_borrow`
39, against S3.3's 173 / 222 / 38.

How the work was done: item 1 by reading the facade and the safe VM's
embedding layer. Items 2, 3, 5 and 6 by reading every candidate site
(listed by the classifier) in five file groups, by five read-only agents,
each given item 1's definition, with the groups' results checked against
the code where they were the basis of a headline finding. Item 4 and the
throwaway tests by me. Throwaway tests lived in an uncommitted
`vm/e1_probe_tests.rs`, built and run with
`python3 tools/bazel/runtime.py test -p nuxie-scripting --lib <filter> -- --nocapture`
(debug build, aarch64 macOS, test thread stack 2 MiB); they are described
with each result and were deleted afterwards.

## Largest findings

Ordered by severity: process abort reproduced, then host memory without a
bound reproduced, then derived ones.

1. **Script-driven stack overflow: the `Error::CallbackError` chain.**
   Reproduced. A script alone can grow the chain one layer per
   `pcall(print, x)` (the starter error comes from `debug.info` on the
   `xpcall` guard's scoped handler). Formatting a 2,500-layer chain aborts
   the process. So does dropping a 20,000-layer chain during collection,
   and at about 40,000 to 60,000 layers so does growing it. (4.1)
2. **Script-driven stack overflow: audio snapshot hold chains.**
   Reproduced. `s:publish(prev)` makes each `Audio.Snapshot` hold the
   previous one. Dropping a 5,000-long chain at a collection aborts. Only
   the 65,536-object budget limits the length. Lane A's depth limit does
   not cover it (it bounds one value, not a chain across objects).
   (4.4)
3. **Host memory doubling: `Path:add(self)`.** Reproduced. 26 calls
   made 2^27 path commands in 0.56 s while the VM was charged 82 KB; at
   40 calls the test process passed 14 GB resident and was killed. (5)
4. **Registration `sort` reports cycles that do not exist**
   (`vm/native_registration.rs:77`), and its recursion depth equals the
   number of pending scripts. In random acyclic dependency graphs it fails
   with "cyclic script module dependency" in 9,560 of 76,871 cases. (4.2)
5. **No `RefCell` guard in this crate is alive across a class-L
   operation** (one that runs Luau) in a way that can conflict today.
   Every conflicting borrow found needs either a runtime-held borrow (the
   view-model property listener, the most plausible: 2(a) row 1) or a
   payload `Drop` that borrows the same cell during a drain (class H, 12
   latent sites: 2(a)). The real `RefCell` exposure is in `nuxie_runtime`'s
   `CoreHandle` (`core.rs:1057,1088`), which bindings call into.
6. **Derived, not run:**
   - `HostStream::restart` spins forever on a record larger than its
     buffer, and a script can put one there. (3)
   - `print` appends unbounded copies of one VM string to a host line
     buffer: 8,000 references to an 8 MiB string make 64 GB. (5)
   - The shipped ORE GPU path has no GPU or host byte budget. (5)
   - `decodeImage` has no decoded-size cap. (5)
   - The processor VM's `pcmReader` is still uncharged and is never
     freed. (5)
   - Ten `expect`/`unwrap` calls follow a `set_app_data` that can silently
     do nothing. (3)

## 1. Which operations can run script code

Read from `crates/nuxie-scripting/src/luau/facade/` and
`crates/nuxie-luau-vm/src/embed/` at `9eb238d58f`. Three classes. Class L
and class H together are "a call into the VM" for item 2.

### Class L: can run Luau, and so any host callback in the crate

A Luau call can reach every binding the script can name, in any file, so a
guard held across a class-L operation conflicts with any binding that
borrows the same cell.

| Facade operation | Why | Reference |
|---|---|---|
| `Function::call` | `vm::Function::call` → `Vm::call_raw` → `call_value` | `facade/handles.rs:442` `Function::call`; `embed/mod.rs:262` `call_raw` |
| `Thread::resume`, `Thread::resume_module` | `coroutine.resume` through `call_raw` | `handles.rs:713,801`; `embed/mod.rs:1831` `Thread::resume` |
| `Table::get`, `get_field`, `contains_key` | `__index` (`get_table_step` + `meta_now`) | `handles.rs:147,157,196`; `embed/mod.rs:1440` `Table::get` |
| `Table::set` | `__newindex` (`set_table_step` + `meta_now`) | `handles.rs:136`; `embed/mod.rs:1456` |
| `Table::len` | `__len` (`length_step`) | `handles.rs:209`; `embed/mod.rs:1498` |
| `Table::push` | `len()` then `set()` | `handles.rs:191` |
| `Table::call_function_unit`, `call_function_truthy` | `get` then `call` | `handles.rs:326,339` |
| `Value::to_string` on a table, function, userdata, thread or buffer | `Lua::value_to_string` → `Vm::tostring` → `__tostring` | `facade/value.rs:274` `Value::to_string`; `state.rs:623` `value_to_string`; `embed/mod.rs:881` `tostring` → `hosted.rs:607` `tostring_value` |
| `HostCtx::call_yieldable`, `pcall_yieldable` | call from a raw host function | `embed/mod.rs:1234,1261` (used only by `vm.rs` and `vm/lua_mat4_safe.rs`) |
| Interrupt callback | runs inside the interpreter, during a class-L operation only; the VM is out of the slot, so a facade call from it is an error | `state.rs:728-778` `set_interrupt`, `interrupt_fn` |
| `Chunk::call`, `exec`, `eval` | `compiler` feature only, not shipped | `state.rs:1011-1050` |

Not Luau (no metamethod, no call): `Table::raw_get`, `raw_set`, `raw_len`,
`raw_push`, `pairs`/`pairs_vec`/`for_each` (a raw snapshot,
`embed/mod.rs:1520`), `sequence_values`/`for_each_value` (raw), `clear`
(raw), `metatable`/`set_metatable`, `Lua::coerce_string` (`lua_tolstring`:
strings and numbers only, `nuxie-luau-vm/src/meta.rs:221`
`coerce_to_str`), `coerce_number`, `objlen`, `getmetatable`, all `create_*`,
registry and app-data operations, and the `FromLua`/`IntoLua` conversions
(`value.rs:418` `from_vm`, `:443` `to_vm`: no metamethods; tables stay
handles). Luau `==`, `<`, `..`, arithmetic and `for ... in` metamethods run
only inside the interpreter, i.e. inside class L; the facade has no
operation that compares, concatenates or iterates with metamethods.

### Class H: runs host code (Rust `Drop`) without Luau

The safe VM does not run `__gc` (no finalizers). The collector lists freed
userdata keys and freed host closures, and the embedding layer drops their
payloads at its next drain point (`embed/compat.rs:272` `Vm::drain_freed`:
`release_payload` drops the `Rc<RefCell<T>>`, so `T::drop` runs if it was the
last `Rc`; `release` drops a host closure's `Box`, so its captures' `Drop`
runs). Drain points:

- `Vm::protect`, at entry and at exit (`embed/mod.rs:218-238` `protect`):
  every facade operation that goes through `protect`.
- `Vm::settle` → `host_safepoint` (`embed/mod.rs:242`, `compat.rs:291`):
  drain, one collector step, drain. Collected userdata die here.
- `new_userdata` (`compat.rs:339`) and `new_host_native` (`compat.rs:391`).
- Every host function call, at entry and exit (`compat.rs:220` `call`,
  `:228` `cont`), so inside any class-L operation.
- `gc_step_kb` (`embed/mod.rs:784`), `collect` (`embed/run.rs:145`).

So, in addition to every class-L operation, these facade operations can run
the `Drop` of any collected userdata type and of any released host closure:
`create_table`/`create_table_result`/`create_table_from`/`create_sequence_from`,
`create_string`, `create_buffer`/`create_buffer_with_capacity`,
`create_function`/`create_callback_function`, `create_userdata`,
`create_thread`, `Table::raw_set`/`raw_push`/`clear`, the registry setters,
`load_bytecode`, `gc_collect`, `gc_step_kbytes`, `to_vm` of a `Value::Error`
(it creates a string, `value.rs:461`), and `Error::raise` (it creates a
userdata, `error.rs:138`). Dropping a facade handle only queues its root in
a `Cell` (`embed/handle.rs:32-80` `ReleaseQueue`, `Marker::drop`): no host
code. Dropping the last owning `Lua` drops the VM and every payload
(`state.rs:169` `Owner::drop` → `Shared::close`).

Consequences for item 2 and 6:

- A guard held across any class-H operation conflicts with the `Drop` of a
  userdata payload that borrows the same cell (item 6 lists them).
- A panic in a payload `Drop` during a drain at host-call entry or exit
  (`compat.rs:220-239`) is outside the trampoline's `catch_unwind`
  (`state.rs:904` `trampoline` catches only the callback body).

### Class 0: the facade's own cells cannot panic

Every borrow in `facade/state.rs`, `handles.rs`, `userdata.rs`, `error.rs`
is `try_borrow`/`try_borrow_mut` (33 sites; item 2's try-borrow table). A
nested operation reaches the VM through `lend` (`state.rs:120`), which parks
the VM in the slot for the callback; a facade operation while the VM is out
of the slot (inside an interrupt, or a payload `Drop` during a drain) fails
with "the Lua VM is in use by another operation" (`state.rs:44`
`Shared::with_vm`). Userdata methods borrow the receiver with
`try_borrow`/`try_borrow_mut` for the whole method body (`userdata.rs:451,
470, 487, 505`), so a re-entrant conflicting method on the same userdata is
`Error::UserDataBorrowError` ("userdata already mutably borrowed"), a Luau
error, not a panic. What does panic is binding code that holds an
`Rc<RefCell<T>>` itself (from `AnyUserData::shared_cell`, `userdata.rs:241`,
the VM's `AnyUserData::cell`, `embed/mod.rs:1745`, or its own struct) and
calls `.borrow()`/`.borrow_mut()` on it.

`Lua::app_data_ref` / `app_data_mut` (`state.rs:848,873`) hold a guard on
the whole app-data map, not one entry, and return `None` on conflict: while
any binding holds an `AppDataRef` across a class-L call, every
`app_data_mut` and `set_app_data` (`state.rs:834`, which ignores the error)
in a re-entered binding silently does nothing.

## 2. `RefCell` guards alive across a call into the VM

Rule for "alive": a guard counts as alive across an item-1 operation if the
operation runs while the guard is in scope: directly, through a helper, or
as a temporary of the same statement (crate edition 2024: an `if let`
scrutinee temporary lives through the then-block; a plain `if` condition's
is dropped before the block; `cell.borrow_mut().m(arg)` keeps the guard
while `arg` is evaluated). Facade userdata borrows
(`AnyUserData::borrow::<T>()`, the method receiver) are `try_borrow` and
not counted as panicking (item 1, class 0).

### 2(a) Guards held across a call into the VM

None is held across a class-L operation in a way that conflicts with
another binding today. What exists: one runtime-held borrow that a Luau
listener can hit, and guards held across class-H operations (a
`create_userdata`/`create_table`/... that can drop a collected payload)
whose conflicting `Drop` was not found.

| # | Guard: file:line, function | Cell, owner | VM call while held | Conflicting second borrow and script | Status |
|---|---|---|---|---|---|
| 1 | **Not ours: the runtime's.** `ViewModelInstanceValue::on_value_changed(&mut self)` (`nuxie-runtime` `viewmodel_instance_value.rs:549-588`) runs our listener sinks while the value's `CoreHandle` slot is borrowed mutably (`core.rs:1088` `with_mut`). Sinks: `vm/view_model.rs:1146` `property_watch`, `:1665` `ScriptedPropertyBlob::new`, `:2973` `ScriptedPropertyTrigger::new`; they call Luau through `call_property_listeners` (`:1261`, `Function::call`). | `CoreHandle` slot `RefCell` of the property value (runtime arena) | class L: the script's listener | Listener reads the value: `vm.c:addListener(function() print(vm.c.value) end)` → getter (`ScriptedPropertyColor`, `view_model.rs:1383`) → `CoreHandle::with` → panicking `borrow()` (`core.rs:1057`). Binding writes are deferred (`HOST_MUTATION_NOTIFICATIONS`, `native_view_model.rs:216-229` `mutate`), so only runtime-internal writes outside a host transaction trigger it (data bind target→source, state-machine actions: not enumerated). Caught by the trampoline as `rust panic: ...`, and the listener error is then discarded (`view_model.rs:1266` `let _ =`). | **Medium; not reproduced** (needs a file with a data bind that writes a non-number property). Fix needs: the runtime releases or defers before `before_notify` for every writer, or our sink always defers. |
| 2 | `gpu_canvas_ore/resources.rs:297→325` `install` (`GPUTexture.new`) | `OreContextHandle = Rc<RefCell<dyn ContextApi>>` (`nuxie-render-api` `routing.rs:10`), one per VM, possibly shared by VMs on a thread | `lua.create_userdata` (class H) | A payload `Drop` that borrows the ORE context. Only an inline render pass's does (`nuxie-ore-metal` `ore_deferred_render_pass.rs:38`); scripts get recorded passes only (`gpu_canvas_ore/canvas.rs:211` `check_begin_pass`). | Latent. Fix needs: end the `ctx` borrow before `create_userdata` (all six ORE rows). |
| 3 | `gpu_canvas_ore/resources.rs:255→259` `install` (`GPUBuffer.new`) | same | `create_userdata` | same | Latent |
| 4 | `gpu_canvas_ore/resources.rs:367→371` `install` (`GPUSampler.new`) | same | `create_userdata` | same | Latent |
| 5 | `gpu_canvas_ore/resources.rs:151→155` `Texture::add_methods` (`view`) | same | `create_userdata` | same | Latent |
| 6 | `gpu_canvas_ore/pipeline.rs:419→430` `install` (`GPUPipeline.new`) | same | `create_userdata` | same | Latent |
| 7 | `gpu_canvas_ore/shader.rs:241→260` `install` (`GPUBindGroupLayout.new`) | same | `create_userdata` | same | Latent |
| 8 | `vm/lua_renderer_library.rs:168` `gpu_features` | same ORE context (`Ref`) | `Table::set` ×20 on a fresh table (no metatable: class H only) | none found | Latent. Fix needs: copy `features()` out first. |
| 9 | `vm/lua_text.rs:301` `add_methods` (`selectionRects`) | `ScriptedText.layout: Rc<RefCell<RawText>>`, shared by the Text's lines and glyphs | `create_table`, `row.set`, `raw_set` on fresh tables (class H) | no `Drop` borrows a `RawText` | Latent. Fix needs: collect, release, then build. |
| 10 | `vm/view_model.rs:2648` `ScriptedContext` `videoGroup`; `:2686` `image`; `:2721` `blob` | `ScriptedContext.source: Rc<RefCell<Option<ScriptedContextSource>>>`, shared with `LuaScriptInstance.context_source` (`vm.rs:1211`) | `lua.create_userdata` (class H) | `vm.rs:3412` `set_context_source` (`borrow_mut`), called by the runtime; no payload `Drop` path to it found | Latent. Fix needs: clone what is needed and release before creating. |
| 11 | `vm/view_model.rs:288` `ScriptedPropertyState::dispose` | `owning_model: RefCell<Option<ScriptViewModel>>` | `self.owning_model.borrow_mut().take();` drops the taken model while the `RefMut` is alive; that can release the last File lease and cascade into arena and script-instance drops (host code) | re-entry into the same state's `owning_model` (only the flush closure, `:716`, which needs Luau) | Latent. Fix needs: `let m = ...take(); drop(m);`. |
| 12 | `vm/lua_audio_samples.rs:454` `StreamController::add_fields` (`descriptor`); `:461` `add_methods` (`reader`) | `Rc<RefCell<HostStream>>` (owned by `Registry.streams`) | `descriptor_value` → `from_shared` / `stream_reader_value` → `create_userdata`, `create_table` (class H) | `StreamController::drop` → `release` → `stream.borrow_mut()` (`:426`), in a drain, **outside `catch_unwind`**. Unreachable today: one controller per stream, and it is the rooted receiver. | Latent. Fix needs: copy the descriptor, release, then convert. |
| 13 | `vm/lua_audio_processors.rs:639-650` `install` (`Audio.start`) | `AppDataRef<VmOwners>` (whole app-data map) and `RefMut<Vec<WeakProcessorOwner>>` | `create_userdata` (class H) | none; side effect: any `set_app_data` in a drained `Drop` silently fails | Latent. Fix needs: scope the borrows. |
| 14 | wasm only: `vm/luau_audio_web.rs:168-187` `kernel_request`, `:203-206` `render` | thread-local `KERNEL: RefCell<Option<BrowserKernel>>` | processor Luau (`BrowserKernel::start` → `Processor::new`; `lane.render` → `process_block`) | JS re-entering the `kernel_request`/`render` exports from an import the processor calls → `BorrowMutError` → wasm trap | Low (JS contract). Fix needs: move the kernel out of the slot for the call. |

Guards held across class-L operations that cannot panic (all `try_`):
`vm/lua_renderer.rs:291` `ScriptedRenderer::with_renderer_mut`
(`try_borrow_mut` of the canvas frame, held across `Artboard:draw`'s
visitor, which runs Luau, `lua_artboards.rs:296`): a conflict is the Luau
error "Renderer is already mutably borrowed by this callback". Every facade
method receiver borrow (e.g. `GpuCanvas` `beginRenderPass`,
`gpu_canvas.rs:793-929`, across descriptor `get`s).

Tried as throwaway tests: none of these. Rows 2 to 14 have no reachable
trigger in this tree, and row 1 needs an authored file with a data bind;
the time went to the reproducible crashes (items 4 and 5). So no panic
from a conflicting `RefCell` borrow was reproduced.

### 2(b) Cells whose every borrow is provably short

Rules: (R1) the guard is a statement temporary in a statement with no
item-1 operation; (R2) the guard is scoped to a block or closure that
touches only Rust data, `MainThreadRef::new/get`, `ScriptedListener::new`,
raw ring/buffer access or host renderer calls that run no Luau; (R3) the
data is cloned out before any VM call (`.borrow().clone()`, listener vecs
cloned before calling, `dispatch_property_change` clones); (R4) the guard
covers only drops of values whose `Drop` runs no host code that reaches
the cell. `Cell` (`get`/`set`/`replace`/`take` on `Cell`) cannot panic and
is listed separately.

| File | Panicking borrow sites (`borrow` + `borrow_mut`) | In 2(a) or 2(c) | Short, rule |
|---|---:|---|---|
| `vm/view_model.rs` | 116 | rows 10, 11; 2(c) | rest: R1, R2, R3, R4 (22 of 24 `RefCell` kinds short at every site) |
| `gpu_canvas.rs` | 40 | 2(c) `:2516` | rest R1, R2 (10 cells) |
| `vm/lua_audio_samples.rs` | 38 | row 12 (2 sites) | rest R1, R2 (`Registry.pending/waits/streams`, `HostStream`) |
| `vm/lua_text.rs` | 37 | row 9 | 36 R1/R2 (`with` closures are pure layout reads; `push_*` after release) |
| `vm/lua_audio_processors.rs` | 18 | row 13 | rest R1/R2/R4 |
| `vm.rs` | 15 | — | R1, R3 (`native_shader_authorities`, `module_hooks`, `gpu_canvas_shaders`, `context_view_model`, `context_source`) |
| `vm/lua_renderer_library.rs` | 10 | row 8 | rest R1/R4 (`open_canvas_frames`, `routing`; dropped entries are handles) |
| `vm/lua_paint.rs` | 10 | — | R1 (one `RenderPaint` setter per borrow) |
| `vm/resource_limits.rs` | 10 | — | R1 (`message`, `callback_failure`, `host_callback_failure`; `Display`, not `__tostring`) |
| `vm/lua_artboards.rs` | 9 | — | R1, R2 (draw visitor `current`/`failure`; `data`) |
| `vm/lua_canvas.rs` | 8 | 2(c) `:127,:163,:407` | rest R1 |
| `gpu_canvas_ore/*` (7 files) | 34 | rows 2-7 | rest R1/R2 |
| `host_commands.rs` | 6 | — | R1 (`HostCommandState.commands`) |
| `vm/lua_transition.rs` | 6 | — | R1, R3 (clone before `child.draw`) |
| `vm/lua_audio.rs` | 6 | — | R2 (asset maps) |
| `vm/lua_renderer.rs` | 6 | — | R1 (`target`, `ACTIVE_RENDERERS`: the registry borrow ends before the renderer callback) |
| `vm/command_server.rs` | 5 | — | R3 (clone then call) |
| `vm/lua_image_decode.rs` | 5 | — | R1 (`pending`; promises resolved after release) |
| `vm/lua_blob.rs`, `lua_image.rs`, `lua_path.rs`, `lua_mesh.rs` | 4, 3, 2, 1 | — | R1/R2 |
| `vm/luau_audio_web.rs` (wasm) | 4 | row 14 | rest R1 |
| `vm/luau_audio_processor_engine.rs` | 3 | — | R1 (`WEB_OWNERS`, wasm) |
| `vm/luau_audio_processor_engine/safe.rs` | 0 | — | no cells: host state is `&mut` through `split` |
| facade (`luau/facade/*`) | 0 | — | all `try_` |

Counts are `\.borrow\(\)` plus `\.borrow_mut\(\)` per file on shipped,
comment-stripped lines (397 in all; a few are facade typed borrows or
`Ref` methods, not `RefCell::borrow`, and were read as such).

### 2(c) Not classified

- `gpu_canvas.rs:2516` `enter_script_call_gpu_scope` and
  `gpu_canvas_ore/mod.rs:240` `close_orphan_render_pass`: shared borrows of
  the ORE context at every script-call entry and exit (`vm.rs:1093`
  `protected_call`). They panic if a host caller runs a script while it
  holds the context mutably; that needs the renderer and embedder call
  sites.
- `vm/lua_canvas.rs:127` `allocate_script_render_canvas`, `:163`
  `DeferredCanvasFrame::finish`, `:407` `beginFrame`: `borrow_mut` of the
  embedder's `DeferredCanvasHostHandle`. They panic if the embedder holds
  it while driving script.
- `vm/lua_video.rs:34` `ScriptVideo::with` and `vm/lua_image.rs:99`
  `ScriptedImage::current` go through `CoreHandle::with_downcast_mut` /
  `with_downcast` (panicking, `core.rs:1057,1088`). They panic if script
  runs while the runtime holds that Video borrowed; this is the same
  runtime question as 2(a) row 1.
- `vm/view_model.rs:2437` `failed_generator`: `source.borrow()` held
  across `retain_failed_script_cleanup` → `owner.with_mut` (runtime,
  panicking). It panics if the runtime holds the ScriptedObject borrowed
  when the generator fails (callers `vm.rs:1580,1948,2072,3211`).
- `LuaScriptInstance::drop` (`vm.rs:3340`) → `dispose_script_lifetime`
  (`:1439`) → `property_listener_owner.dispose()` can run inside a drain:
  a collected `artboard:instance()` payload (`lua_artboards.rs:188`) owns
  runtime artboards whose scripted objects own same-VM script instances.
  The `dispose` path's borrows are short (2(b)), but the File-lease drop
  cascades under it were not followed.

### `try_borrow` / `try_borrow_mut` sites and what they do on failure

| Site | On failure |
|---|---|
| `luau/facade/state.rs` (21): `Shared::with_vm` :45, `close` :71-80, `spare` :89, `Taken::drop` :111, `lend` :123,:131,:138,:143, `create_function_mut` :433, `try_set_app_data` :842, `app_data_ref` :849, `try_app_data_ref` :863, `app_data_mut` :874, `remove_app_data` :883, `Scope::drop` :953, `Scope::create_function` :971,:977 | errors ("the Lua VM is in use by another operation", `RecursiveMutCallback`, "cannot mutably borrow app data container"), `None` (app data), or silently skip (`close`, `Taken::drop`, `Scope::drop`, `set_app_data`) |
| `luau/facade/userdata.rs` (8): `borrow` :255, `borrow_mut` :263, method dispatch :451,:470,:487,:505, :643, metatable cache :699 | `UserDataBorrowError` / `UserDataBorrowMutError` (Luau error); cache: skip |
| `luau/facade/handles.rs` (3), `error.rs:168` `recover_error_object` | error; `recover_error_object` falls back to `<non-string error>` |
| `vm/lua_mat4_safe.rs:120` `mat4_at`, `:130` `write_mat4`, `:392` `mat4_newindex` | `HostError` "Mat4 is in use" |
| `vm/lua_renderer.rs:142` `ScopedRendererAccess::drop`, `:168` `ActiveRendererBorrow::drop` | sets `RENDERER_STACK_POISONED` (never cleared: every renderer on that thread fails from then on) and `BROKEN_RENDERER_SCOPE` |
| `vm/lua_renderer.rs:291` `with_renderer_mut` | Luau error |
| `nuxie-luau-vm` `embed/mod.rs:1764,1773` `AnyUserData::borrow/borrow_mut`, `:567` direct-field getter | error / field absent |

Callers that turn an app-data `None` into a panic are in item 3.

### Cells shared between threads or VMs

No non-`Sync` cell reaches two threads: `SafeProcessor: Send` is checked by
the compiler (`vm/luau_audio_processor_engine/safe.rs:210-213`) and the
init job is `Box<dyn FnOnce + Send>` (`luau_audio_processor_engine.rs:625`).
Shared across threads, with their primitives: `Shared`
(`luau_audio_processor_engine.rs:237-245`, atomics plus
`OnceLock<String>`); `LogRing` (`:170-235`, atomics plus `Mutex<()>`,
single writer by protocol only); `SharedBudget.state`
(`luau_audio_shared.rs:51`, spin lock that recovers poison); `RingState`
(`:188`, `Mutex`; the audio thread spins on `lock` for `send`/`read`
(priority inversion) and uses `try_lock` in `read_record`);
`SnapshotState` (`:386-391`, seqlock); `SharedRegion` (`:862-869`,
atomics); `CompletionQueue` (`lua_audio_samples.rs:66`, `Arc<Mutex>`,
poison recovered); `Owner.node`/`offline_vm`
(`luau_audio_processor_engine.rs:318-319`). Process-wide:
`OFFLINE_RENDER` (`:47`, one VM's `set_render_offline` changes every VM)
and the single `luau-audio-init` thread with its unbounded queue (`QUEUE`,
`:628-646`): one VM's initialisations delay every VM's.
Shared between VMs on one thread: the ORE context (`OreContextHandle`,
when the host factory hands one context to several VMs); the renderer
thread-locals (`lua_renderer.rs:90-99`: `ACTIVE_RENDERERS`,
`NEXT_RENDERER_TOKEN`, `BROKEN_RENDERER_SCOPE`,
`RENDERER_STACK_POISONED`); runtime view-model value delegates (a setter in
VM A runs VM B's listener synchronously, `view_model.rs:1264`).

## 3. Other re-entrancy panics

Search: `grep -rnE` over `crates/nuxie-scripting/src` for each pattern
below, minus `*_tests.rs`, `upstream_scripting_*` and `tests.rs`, then each
hit read and checked against the classifier (`cfg(test)`/feature-gated
hits dropped).

| Pattern | Shipped hits | Verdict |
|---|---|---|
| `Rc::get_mut(..).unwrap()` / `Rc::try_unwrap(..).unwrap()` | 0. `get_mut` hits are `Vec`/`HashMap`/`Cell::get_mut` (`lua_mat4_safe.rs:394`, `resource_limits.rs:238`, `lua_audio_samples.rs:618,627`, `view_model.rs:294-295` in `Drop` on `&mut self`, `safe.rs:836`, facade `state.rs:876,952`). `lua_canvas.rs:100` `end_frame` `Rc::try_unwrap(frame)` maps failure to a Luau error. | none can panic |
| `take().expect` in-use markers | 1: `vm/lua_canvas.rs:99` `ScriptedCanvas::end_frame`, `self.frame.take().expect("active Canvas frame")`; both callers check first (`:145` `close_orphan_canvas_frames`, `:421` `endFrame`) | safe |
| `OnceCell` / `OnceLock` / `LazyLock` | `vm.rs:985` `ScriptVm.image_decode: OnceCell` (`image_decode_registry`, `vm.rs:3054`: `get`/`set`, no `get_or_init`, so no re-entrant init); `luau_audio_processor_engine.rs:240` `Shared.error: OnceLock<String>` (`set` result ignored); `:628` `QUEUE` (`get_or_init` spawns the init thread, runs no VM; a failed spawn is cached as `Err` and every later `Audio.start` fails) | none can panic |
| `thread_local!` `with` nested across a VM call | `vm/lua_renderer.rs:90` (`ACTIVE_RENDERERS` and others: `try_with` + `try_borrow_mut`; registry borrows end before the renderer callback); `luau_audio_processor_engine.rs:370` `WEB_OWNERS` (wasm; never borrowed across a VM call); `luau_audio_web.rs:144` `KERNEL` (wasm; **held across processor Luau**, 2(a) row 14). (`lua_scriptnet.rs:18` is `scriptnet`-gated.) | one wasm site, 2(a) row 14 |
| `Mutex::lock().unwrap()` / `.expect` | 0 shipped. All hits are `AUDIO_TEST_LOCK` (`#[cfg(all(test, feature = "compiler"))]`, `vm/lua_audio.rs:311`). Shipped locks recover poison (`lua_audio_samples.rs:31` `lock_unpoisoned`, `luau_audio_processor_engine.rs:156`, `luau_audio_shared.rs:19-35`). Reached outside the crate: `nuxie-runtime` `work_pool.rs:94,106,110` `join_task` `lock().unwrap()` (from `lua_audio_samples.rs:611`, offline render only); `nuxie-renderer` `deferred_canvas.rs:38` `CanvasRetirer::retire` `lock().unwrap()` from `RenderCanvas::drop` (item 6). | none in crate |

### `set_app_data` then `expect`/`unwrap` (a re-entrancy panic of its own)

`Lua::set_app_data` (facade `state.rs:836`) does nothing when the app-data
map is borrowed (`try_borrow_mut` fails); `app_data_ref` then returns
`None`. Ten sites follow a `set_app_data` with an `expect` or `unwrap` of
the read-back:

- `vm/lua_audio_samples.rs:208-211` `registry` (lazy install on the first
  `toBlob`/`stream`, inside a script call);
- `vm/lua_audio_processors.rs:211-215` `register_module` (called by the
  host from `vm.rs:2746`, `:2886`, so a panic there reaches the embedder,
  not the trampoline);
- `vm/lua_audio_processors.rs:510`, `:526`, `:542`, `:558`, `:601`, `:641`
  (`install`), `:1524-1529` and `:1555` `region_exclusive` (lazy
  `RegionHeld`, inside `tryExclusive`).

Trigger: an `AppDataRef` alive up the stack during the install. The only
long `AppDataRef` found is 2(a) row 13 (`Audio.start`), which creates
userdata, so a drained `Drop` that installs app data would hit it. None was
found. Status: latent. Fix needs: `try_set_app_data` and propagate the
error, or install eagerly.

### Panics that a script reaches in a host callback (caught, become `rust panic: ...`)

Not re-entrancy, listed because they are the crash class when they move
outside the trampoline (a `Drop`, a host-called entry):
`vm/lua_canvas.rs:172` `DeferredCanvasFrame::target`
`expect("deferred host returned a null renderer")` (every `renderer:*`
call after `beginFrame` got no renderer, `:407`); `vm/lua_text.rs:340`
`glyphs`, `:466` `next`, `:491` `ScriptedGlyph::with`, `:498` font
getter, `:528` path (`unwrap`/`expect` on glyph runs; the revision check
they rely on is a `u32` that wraps); `vm/lua_video.rs:143`, `:179`,
`:182` (`property_number(..).unwrap()`); `gpu_canvas_ore/target.rs:33`,
`:38`, `:44`, `:50` and `gpu_canvas_ore/canvas.rs:143`
(`textureViewBase().unwrap()`, `None` off the recording thread or for a
non-view resource); `gpu_canvas_ore/resources.rs:64-93`
(`expect("texture view")`). Host-called, not caught:
`vm/command_server.rs:48` `PersistentRenderContext::adopt` (from
`vm.rs:3137`); `vm.rs:1319` `LuaScriptInstance::table` and `vm.rs:2467`
`ScriptVm::set_view_models` (pub, test-only callers in tree; the latter
fails when a script has filled the VM to its ceiling). Audio thread,
outside `process_block`'s `catch_unwind`:
`luau_audio_processor_engine/safe.rs:394`, `:399` `SafeProcessor::render`
`out[first..]` (engine contract). wasm: `browser_kernel.rs:53`
`BrowserKernel::start` has no `catch_unwind`.

### Non-panic re-entrancy hazard: nested entries reset the budgets

`vm.rs:1334-1351` `LuaScriptInstance::reset_execution_budget` and
`vm.rs:2285-2300` `ScriptVm::reset_execution_budget`: when
`script_cycle_active` is false, every host entry runs `begin_cycle()`
(clears `tripped`), `script_safepoints.set(0)` and `budget.reset()`. A
nested entry from the same VM (a script calling `art:advance()` on an
artboard whose scripted drawables share this VM) resets them, so
`while true do art:advance(0) end` never reaches either cap: a hang, and
a tripped terminal limit is forgotten. From the g4a reading; not run (no
fixture with a same-VM nested artboard was at hand). Fix needs: a nesting
depth so only the outermost entry resets.

`host_commands.rs:232` `begin_cycle` / `:272` `drain_effects`: a nested
runtime transaction mid-cycle clears or drains the outer cycle's queued
commands (data loss, not a panic).

`HostStream::restart` (`vm/lua_audio_samples.rs:376-384`) is an
uninterruptible infinite loop on the main thread:
`while self.descriptor.ring.read_record(&mut self.record)? != 0 {}`, and
`SharedRing::read_record` (`luau_audio_shared.rs:240-265`) returns
`size + 1` without consuming when the head record is larger than `out`.
The stream ring is reachable from script (`controller.descriptor.ring`).
Sketch (not run):

```lua
local c = src:stream(48000, 2, 256, 4096):await()
local r = c.descriptor.ring
for i = 1, 64 do pcall(function() r:receive() end) end -- drain PCM records
r:send(string.rep("x", 4000))                         -- a record larger than the chunk buffer
c:seekFrame(0)                                        -- restart: spins forever
```

Fix needs: `restart` consumes or rejects an oversized record.

## 4. Recursion over script-built or file-built data

### 4.1 `Error::CallbackError` chain (`luau/facade/error.rs`): reproduced, aborts the process

Growth: each round adds one layer. A host callback that calls script code
with `?` gets the script's error back from `Shared::with_vm`
(`state.rs:44`), which wraps a raised `WrappedError` userdata in a new
`CallbackError { cause: Arc<previous> }` (`error.rs:154`
`recover_error_object`). The callback returns it; `trampoline`
(`state.rs:904`) calls `Error::raise` (`error.rs:138`), which re-raises it as
a new `WrappedError` userdata because `is_structured` (`error.rs:124`, itself
recursive over the chain) is true for any chain ending in a structured
error. The script catches that userdata with `pcall` and feeds it back.

The callback used: `print` (`vm/lua_rive_base.rs:8` `install_host_print`
calls `tostring.call(value)?` at `:20`). Any shipped callback that calls
script code and propagates the error works the same way (`lua_font.rs:292,
311, 331` dispatchers, `view_model.rs:1275` property listeners,
`lua_artboards.rs:284` visitor, ...).

The starter (a structured error) is reachable from a script alone:
`resource_limits.rs:229` `install_protected_call_guards` wraps every
`xpcall` handler in a `Lua::scope` function; `debug.info(level, "f")` inside
the handler returns that scoped function; called after `xpcall` returns it
fails with `Error::CallbackDestructed` (`state.rs:979`), which is
structured.

Script (ran through `ScriptVm::new` + `install_rive_globals`, the shipped
global set including the sandbox and the protected-call guards):

```lua
-- starter
local captured = {}
xpcall(function() error("x") end, function(e)
  for level = 1, 8 do
    local ok, f = pcall(debug.info, level, "f")
    if ok and f then table.insert(captured, f) end
  end
  return e
end)
for _, f in ipairs(captured) do
  local ok, err = pcall(f, "probe")
  if not ok and typeof(err) == "userdata" then E1OBJ = err break end
end
E1VICTIM = setmetatable({}, { __tostring = function() error(E1OBJ) end })

-- growth: 500 rounds per call (one script cycle), repeated across cycles
for i = 1, 500 do
  local ok, e = pcall(print, E1VICTIM)
  E1OBJ = e        -- one more CallbackError layer each round
end

-- trigger: any script method that fails with it
error(E1OBJ)
```

Results (throwaway test `vm/e1_probe_tests.rs`, not committed; debug
build, test thread stack 2 MiB, aarch64 macOS):

| Layers | `error.to_string()` (`Display`) | dropping the error, then collecting the userdata that hold the chain |
|---:|---|---|
| 1,000 | ok | ok |
| 2,500 | **stack overflow, process aborts** | not run |
| 5,000 | **abort** | ok |
| 20,000 | **abort** | **abort**, during `gc_collect` (the collector's payload drain drops the `WrappedError`s, then the `Arc` chain recursively) |
| ~30,000 (first run, `unwrap` panic while unwinding) | — | **abort**: the chain's `Drop` while unwinding |
| 60,000 | — | **abort during growth**, before the trigger (one script round past about 40,000 layers; most likely `is_structured`, which recurses in an unoptimized build; not pinned down) |

A stack overflow is not a panic: no `catch_unwind` contains it, under
either panic strategy.

Cost to the script: 5,000 layers took 0.15 s and 30,000 took 3.5 s of
script time (quadratic: `is_structured` walks the chain on every raise) and about 30,000 × (VM userdata + `Arc`)
of memory. One script cycle allows 100,000 safepoints
(`script cycle exceeds 100000 script safepoints`), which stopped one cycle
at about 30,000 rounds; the chain lives in a global, so it grows across
cycles with no bound.

Product path that overflows: a scripted method that fails with the
object → `ScriptVm::script_error` (`vm.rs:1330`) →
`ResourceLimitTracker::observe_callback_failure` (`resource_limits.rs:142`)
and `tracked_script_error` (`vm.rs:4247`), both `error.to_string()` →
`impl Display for Error` (`error.rs:220`, `CallbackError { cause, .. } =>
write!(f, "{cause}")`, `:244`): one recursion per layer, not a tail call.
`log_error` (`logging_scripting_context.rs:13`), `observe_vm_error`
(`resource_limits.rs:132`) and `is_structured` recurse in tail position
(may become loops in release; not measured). The derived `Drop` of the
chain (`Arc<Error>` → `Error` → ...) recurses once per layer.

Fix needs: a cap on the layers (or an iterative chain) in
`recover_error_object`, and iterative `Display`, `is_structured`,
`Drop`.

### 4.2 `sort` in `vm/native_registration.rs` (`register`, nested `fn sort` at :77)

The early return for an already-visited index (`:87-89`) is **not** the
correctness bug: each non-early frame performs exactly one pop, so every
pending index is still reached and the order stays topological. The bug is
elsewhere in the same function: every frame ends with
`remaining.pop()` and a **nested** `sort` call (`:116-127`). When that pop
happens inside a dependency recursion (`:96-110`), it can pop an ancestor
that is still in `visiting`; `visiting.insert` (`:90`) then fails and the
function returns `Err("cyclic script module dependency")` for a graph with
no cycle.

Minimal input, file order [Y, X, P, C]:

| Script | Body |
|---|---|
| Y | `require("C")` |
| X | `require("Y")` |
| P | `require("X")` |
| C | `return {}` |

Pass 1: Y, X, P fail (deps {C}, {Y}, {X}); C registers and is erased from
Y's deps. `remaining` = [Y, X, P]. pop P → `sort(P)` → dep `sort(X)` → dep
`sort(Y)`; Y pushes itself, then pops X, which is still in `visiting` →
cycle error. `run()` aborts; Y, X and P all get "cyclic script module
dependency" and nothing registers. Correct result: [Y, X, P] in order, all
registered on the retry. A smaller form: [Y (plain runtime error),
X requires Y, P requires X] gives the same false error instead of each
script's own.

Count: `sortsim3.py` (scratchpad) reimplements `sort` line for line and
compares it with a reference topological sort on random acyclic graphs: 9,560
of 76,871 graphs give the false cycle error. **Correction:** my first
simulation (`sortsim.py`) skipped runs that returned the cycle error and so
reported "no bug"; the g4a reading found the false cycle, and `sortsim3.py`
confirms it.

Depth: nested pops make the recursion depth at least the number of pending
scripts plus the dependency-chain length, with no cap (the bound is the
number of ScriptAssets in the file). The frame size was not measured, so
the script count that overflows a 512 KiB thread is unknown. `sorted.contains` (`:111`)
makes it O(n²), and `attempt` (`:216-230`) re-snapshots every script's
dependency table each pass.

Fix needs: an iterative topological sort with an explicit stack, seeded from
`remaining` in an outer loop.

### 4.3 The VM's own traversals (`crates/nuxie-luau-vm`)

Method: a self-recursion search over the crate's non-test source (every
`fn` whose body calls its own name; 0 data-driven hits, every hit was a
same-named method on another type), then reading the traversals named in
the task.

| Walk | Implementation | Verdict |
|---|---|---|
| GC marking | gray lists (`heap.rs:437` `gray: Vec<GcRef>`, `gray_again`), propagate loop (`heap.rs:1020`) | iterative |
| GC sweep | per-arena loop (`heap.rs:187` `Arena::sweep`) | iterative |
| Freeing heap objects | arenas of slots, no owning pointers between objects | no recursive `Drop` |
| `__index` / `__newindex` chains | `index_chain` loop bounded by `Limits::max_meta_chain` = 100 (`limits.rs:32`, `meta.rs:232`) | bounded |
| Table equality | identity, or one `__eq` call (no deep comparison) | not a walk |
| `tostring` | one `__tostring` call (`hosted.rs:607` `tostring_value`) | not a walk |
| Lua-to-Lua calls, metamethods, `pcall`, `resume` | on the VM's own frame stack, not the Rust stack (`call.rs:36`); `max_calls` 20,000, `max_c_calls` 200 | bounded, heap |
| Rust-stack re-entry (host callback calling back, `xpcall` handler, facade `Table::get` with `__index`) | `Limits::max_reentry` = 16 (`limits.rs:28`, checked `hosted.rs:541`); measured 46-49 KiB per level unoptimized | bounded; see could-not-determine |
| `table.sort` | explicit `pending` ranges and heapsort fallback (`libs/sort.rs:8-20`) | iterative |
| Value conversion (`embed/convert.rs`, facade `conversion.rs`) | type-driven (`Option`, `Vec`, tuples); tables stay handles | not data-driven |
| Bytecode loading | flat proto list, counts checked against remaining input (`bytecode/`) | no recursion found |

`nuxie-scripting` itself: lane A's per-file call-graph search stands; the
two walks it left open are 4.1 and 4.2. The walks inside `nuxie_runtime`
reached from bindings are in 4.5.

### 4.4 Audio shared-object hold chains: reproduced, aborts the process

`SharedSnapshot::publish` stores the handles it carries in
`SnapshotWriter.holds` (`luau_audio_shared.rs:383`, `:481`), and a ring send
stores them in `RingState.holds` (`:168`, `:293`). Each object therefore owns
the previous one, and dropping the head drops the chain recursively
(`SharedValue` → `Arc<SharedSnapshot>::drop_slow` → `SnapshotState` →
`Mutex<SnapshotWriter>` → `[Vec<SharedValue>; 2]` → `SharedValue` ...). The
only bound is the 65,536 objects per `SharedBudget`
(`luau_audio_shared.rs:14`; a snapshot costs 1 object, a ring 3).
`codec`'s comment (`luau_audio_shared_codec.rs:3-6`) that dropping is
bounded by `MAX_SHARED_DEPTH` is wrong across objects: lane A's depth limit
bounds one value, not a chain of objects.

Script (throwaway test, main VM, shipped globals):

```lua
E1PREV = Audio.Snapshot.create(16)
-- 1,000 per script cycle, repeated
for i = 1, 1000 do
  local s = Audio.Snapshot.create(16)
  s:publish(E1PREV)
  E1PREV = s
end
-- then
E1PREV = nil   -- and the host runs gc_collect
```

| Chain length | Result |
|---:|---|
| 5,000 | built; **stack overflow during `gc_collect`, process aborts** |
| 60,000 | built; **abort** during `gc_collect` |

The drop runs in the VM's payload drain (class H), outside any
`catch_unwind`; even if it did not, a stack overflow is not catchable. When
the head is a processor's input, the drop lands on the `luau-audio-init`
thread or wherever the processor VM is dropped (`Owner::drop`,
`luau_audio_processor_engine.rs:355`). Rings (`ring:send(nextRing)`)
behave the same (not run). `SharedBudget::close`
(`luau_audio_shared.rs:102-138`) drops `holds` per entry and recurses into
entries not yet closed. Fix needs: drain `holds` into a worklist on drop
(iterative), or cap the hold-chain length.

### 4.5 Walks inside `nuxie_runtime` reached from bindings

| Walk | Reached from | Bound | Verdict |
|---|---|---|---|
| `copy_view_model_instance_with_map` (`nuxie-runtime` `file.rs:1886-1904`) | `vm:instance()`/`new()` (`view_model.rs:892`), `Data.X.new` (`:2955`) | `depth < 256` (`file.rs:1660`) and a memo map against cycles | bounded |
| `DataContext::root_view_model_instance` / `view_model_value` (`data_context.rs:640-651`) | `rootViewModel` (`view_model.rs:2540`), `dataContext():viewModel` (`:2879`) | file nesting depth (nested artboards, list items); no cap; a parent cycle would not end | not proven bounded (file-driven) |
| Parent relink and notify walks after `set_view_model` (`view_model.rs:1342` → `native_view_model.rs:599` `replace_view_model_property_occurrence`) | script assigning a view-model property, which can build a cycle | not read | unclassified |
| `RawText::update` shaping and layout | `Text` bindings (`lua_text.rs`) | text length | bounded (g4b; not re-read) |
| Promise library (`lua_promise.rs`, Luau) | `andThen` on a settled promise inside a handler: `notifyCallbacks` → the Rust-guarded `pcall` (`resource_limits.rs:213`) → handler | Luau-to-Luau parts on the VM stack; each host `pcall` level is one Rust re-entry, capped by `max_reentry` 16 if the guard re-enters through `hosted.rs:541` (not verified) | probably bounded |
| `host_commands.rs:436` `value_at_depth`, `:475` `table_value` | `command(name, payload)` | `max_value_depth` ≤ 64 (`:18`), `active_tables` rejects cycles | bounded |
| Audio `to_shared` / `from_shared` / codec / processor `read`/`push` | `send`, `publish`, `Audio.start` | `MAX_SHARED_DEPTH` 64 (`lua_audio_processors.rs:718`), 2^22 nodes | bounded (lane A) |
| Shader asset decode (`gpu_canvas_ore/shader_asset.rs`) | file shader assets | iterative cursor; counts checked against remaining bytes; RSTB ≤ 4 MiB | bounded |
| `nuxie-luau-vm` re-entry | any host callback that calls back | `max_reentry` 16 | bounded (4.3) |

## 5. Host memory a script or file can grow

Traced by data flow from each binding's argument to the host container it
lands in. "VM charge" is what the VM allocator sees (a userdata header is
tens of bytes); everything else in a payload's `Vec`s is host memory the VM
ceiling (16 MiB) does not cover. Reference design for a budget:
`MeshInstanceBudget` (`vm/lua_mesh.rs:161-190`, 16 MiB per VM, charged
before `try_reserve`, high-water). Ordered by the worst ratio of host bytes
to VM bytes.

| # | Site: file:line, function | Grown by | Current bound | Nearby budget type | Cost per unit | Fix needs |
|---|---|---|---|---|---|---|
| 1 | `vm/lua_path.rs:292` `"add"` → `raw_path.add_self_path` (`nuxie-runtime` `raw_path.rs:465`, clone then extend) when the argument is the receiver | `for i = 1, 40 do p:add(p) end` | **none; doubles per call** plus a 1× transient clone | none | 2^n × the path; **measured**: 26 calls → 2^27 commands in 0.56 s with the VM at 82 KB; 40 calls passed 14 GB RSS (killed) | a per-VM path-point budget checked before extend |
| 2 | `vm/lua_rive_base.rs:10-27` `install_host_print` → runtime `logging_scripting_context.rs:34` `append`, `:45` `end_line` clone | `print(table.unpack(t))`, each entry the same 8 MiB VM string | **none** (≈ 8,000 args via `unpack`) | none | sum of arg lengths, ×2 at `end_line`; 8,000 × 8 MiB ≈ 64 GB for 8 MiB charged | truncate the line or charge it |
| 3 | `vm/lua_image_decode.rs:33` `decode_image_rgba_unbounded` from `:120` `execute` (`context:decodeImage(buf)`) | each call | **no dimension or decoded-size cap** (`nuxie-image-codec` `lib.rs:290` passes `None`); the result is rejected afterwards if over the VM ceiling (`:351`) | the bounded `decode_image_rgba` (64 MiB) exists | 4 B/pixel on a pool thread; a ~1 MB PNG can decode to ~1 GB | use the bounded decoder, or preflight dimensions before submit |
| 4 | `vm/lua_image_decode.rs:240` `start` `encoded.to_vec()` plus the `pending` map | `for i = 1, 1e4 do context:decodeImage(b) end` | **none** on requests in flight; cancel is an atomic flag | none | `len(buf)` per request, plus the decoded output | per-VM in-flight count/bytes cap |
| 5 | ORE GPU resources: `gpu_canvas_ore/resources.rs:208-263` `GPUBuffer.new` (size ≤ `u32::MAX`, `:211`), `:264-326` `GPUTexture.new`, `:114` `Texture:view`, `:330` sampler; `pipeline.rs:164`, `:226`; `shader.rs:200` | each call | **none** (shipped path) | `GpuCanvasResourceBudget` (`gpu_canvas.rs:57-76`) covers the direct path only, and never refunds (64 MiB over a program's life) | up to 4 GiB per buffer, w×h×bpp×layers per texture, backend memory; GC pace follows the VM's tiny charge | per-VM GPU byte budget charged at create, refunded on payload drop; dimension caps |
| 6 | `gpu_canvas_ore/canvas.rs:19` `Canvas::create`, `:162` `resize` → `vm/lua_canvas.rs:119` `allocate_script_render_canvas` | each call | **no dimension cap** (the direct path checks, `gpu_canvas.rs:762`, `:1443`) | none | w×h×4 GPU, plus retired backings until the snapshot drops | cap dimensions |
| 7 | `gpu_canvas_ore/resources.rs:23` `Buffer:write`, `:178` `Texture:upload` (after `data.to_vec()`, `:17`, `:163`) and pass commands | per frame | interrupt budget only; whether the recorder retains copies until replay was not verified | none | bytes written | per-frame recorded-bytes cap |
| 8 | `gpu_canvas.rs:1520` → `:1467` → `:1193` `context:shader(name)` | each call compiles a new occurrence | none | none | backend module + CPU compile | count/bytes budget |
| 9 | Direct path (`gpu_canvas.rs`): non-first canvases' `completed.plan` (`:1887`, `:2175`, `:2161`, `:2166`; `canvases` push `:1453`, never pruned); `GpuTexture.uploads` push `:544`; `draws` push `:1798`, `:1858`; `SnapshotImages.missed` `:1515` | per frame / per call | none | `GpuCanvasResourceBudget` not charged for these | ~200 B/pass, up to 16 MiB per vertex buffer, full upload copies, ~100 B/draw | reset per frame; charge uploads; cap draws and names |
| 10 | `vm/lua_mesh.rs:65-74` `VertexBuffer` `"add"` `values.push(Vec2D)` | `vb:add(v, ...)` in a loop | **none** (infallible push: abort on OOM) | `MeshInstanceBudget` next to it, not applied | 8 B per vertex (+ `Vec` slack); `update` (`:31`) makes a second full copy (`make_render_buffer(len*8)`) | same budget, `try_reserve` |
| 11 | `vm/lua_mesh.rs:145-155` `TriangleBuffer` `"add"` `values.push(u16)` ×3 | `tb:add(a, b, c)` | **none** | same | 6 B per call (+ slack); `update` (`:108`) mirror `len*2` | same budget |
| 12 | `vm/lua_path.rs:255-288` `moveTo`/`lineTo`/`quadTo`/`cubicTo`/`close`; `:418` `extract_measure_segment`; `lua_text.rs:528` glyph `path` | loops of calls | none | none | Move/Line 9 B, Quad 17 B, Cubic 25 B, Close 1 B; `render_path` (`:67`) builds two more copies per dirty draw | path-point budget |
| 13 | `vm/lua_text.rs:222` `Text` `"append"` → `StyledText` (`text.rs:306` `value.push(u32)`) | `t:append(s, style)` with one interned 16 MiB string | none | none | 4 B/char + one `TextRun` per append + ~20 B shaped data per glyph (≈ 24× the VM bytes); each distinct paint adds a style; `style_id` truncates to `u16` (`raw_text.rs:181`, wrong styles past 65,535, no panic) | per-Text/per-VM char and style budget |
| 14 | `luau_audio_processor_engine/safe.rs:810-818` `pcm_reader` → `host.pcm_readers.push` (`safe.rs:72`) | `Audio.streamReader` in processor init or `process` | **none**: never removed, survives GC; `try_reserve_exact` (`luau_audio_samples.rs:134-138`) makes failure an error, not an abort | none reachable from `AudioHost` (the main VM's `ScriptedPcmReader`, `lua_audio_processors.rs:341-357`, is charged to `SharedBudget`) | `record_capacity` (≤ ring capacity ≤ 256 MiB) per call, on the audio/init thread; **lane A's gap is still open at this tip** | budget on `AudioHost`; reuse or evict readers |
| 15 | `vm/lua_audio_processors.rs:630-638` `Audio.start` → `owner.processors` | each call | **none** (`admit`, `:57-66`, only prunes released) | none | one processor VM (≤ 16 MiB, `:624`) + renderer node + output buffer, and audio-thread work per processor | cap processors per VM |
| 16 | `luau_audio_processor_engine.rs:627-646` init queue (unbounded `mpsc`) | starts while `luau-audio-init` is busy | **none** | per-value `SharedValueBudget` only | up to 256 MiB input copy per queued job | bound the queue; charge the input |
| 17 | `RingState.holds` (`luau_audio_shared.rs:168`, `:293`), `SnapshotWriter.holds` (`:383`, `:481`) | handles in sent/published messages | ring bytes only | `SharedBudget` charges 9 B per handle | 32 B `SharedValue` per handle (≈ 3.6×); a snapshot keeps 2 copies | charge holds at their size |
| 18 | `vm/lua_audio_samples.rs:235-245`, `:684-689` `to_blob` | each `toBlob` | the reservation is the decoder's length estimate; a decoder that reports 0 reserves ~0 | `DecodeReservation` | 3 transient copies up to 256 MiB each | reserve on actual growth |
| 19 | `vm/lua_artboards.rs:188` `artboard:instance()` | each call builds a full runtime artboard in the payload | none | `ScriptResourceGuard`, the mesh budget pattern | the artboard graph per call | per-VM instance budget |
| 20 | `vm/lua_font.rs:199-202` `Font.decode` (+ `withOptions` `:182`, `glyphPath` `:185`); `vm/renderer.rs:21`, `:42` `Gradient.linear/radial` | each call | none | none | font size per decode; a shader object per gradient | charge decoded fonts; count gradients |
| 21 | `vm/view_model.rs:1197`, `:1774`, `:3074` `watch.listeners` push (`addListener`, no dedupe) | `prop:addListener(f)` loop | none | none | ~72 B + a VM root; each notification clones the whole `Vec` (`:1262`) | cap per property |
| 22 | `vm/view_model.rs:149`, `:155`, `:161` `ScriptedPropertyListenerOwner.watches` (from `property_watch` `:1175`, `:1702`, `:3036`) | each property wrapper created while an owner is current | until owner `dispose` (`:165`); dead `Weak`s never pruned, and they keep the watch allocation | none | ~186 B | prune dead entries |
| 23 | runtime `ViewModelInstanceValue.delegates` (`viewmodel_instance_value.rs:512`, via `native_view_model.rs:180`) | wrapper re-creation | dead `Weak`s pruned only on the next value change | none | ~116 B | prune on push |
| 24 | `vm/lua_data_value.rs:123-135` `ScriptedDataValue.value` strings; Blob setter `view_model.rs:1752-1757`; list `push`/`insert` `:2175-2186`; `Data.X.new` `:2955` | sets and pushes | per value: the VM string size; count: none | none | string length; a full view-model copy per `new` | charge host copies |
| 25 | `vm.rs:3093-3109` `install_native_file_assets` | each host call (not script) re-pushes every ShaderAsset with `encoded_payload().to_vec()` | file size × calls | none | payload size | dedupe by name |
| 26 | `host_commands.rs:486` `table.pairs()` snapshots | `command(name, payload)` | transient ≈ 3× the table, up to 64 levels, before the node budget runs | `HostCommandLimits` (applied after) | 3 × (key + value) per entry | budget-aware iteration |

CPU, not memory, from the same reading: `buffer_ext.rs:63` `stridedcopy` and
`:129` `convert` accept zero strides with `count` up to `i32::MAX` and run
with no interrupt check; with `source == destination` each iteration does
`source.to_vec()` (`:115`, `:225`), O(count × len). Sketch:
`buffer.convert(b,0,"u8",b,0,"u8",2^31-1,1,0,1)`.

Bounded and budgeted, for contrast: `MeshInstances` (`lua_mesh.rs:212`),
`Audio.Ring/Snapshot/Region/Blob` (`SharedBudget` 512 MiB and 65,536
objects per VM, refunded on drop), host commands per cycle (≤ 4,096
commands, ≤ 32 MiB), main-VM `ScriptedPcmReader`, `registry.streams`
(≤ 16,384 by object count), LogRing (8 KiB).

## 6. Every `Drop` impl in shipped code

Count: `grep -rn "impl.*Drop for"` over the crate, filtered by the
classifier: 34 impls (30 in `vm*`, 4 in the facade). Every `Drop` of a
userdata payload or anything it owns runs in the VM's drain (item 1,
class H), which is **outside** the trampoline's `catch_unwind`: a panic
there unwinds through the host call that triggered the drain (or aborts
under `panic=abort`).

| Site | What it does | Can panic? |
|---|---|---|
| `luau/facade/state.rs:103` `Taken::drop` | `try_borrow_mut`, puts the VM back | no |
| `luau/facade/state.rs:169` `Owner::drop` | closes the shared state (`try_` paths) | no |
| `luau/facade/state.rs:225` `InterruptGuard::drop` | `let _ = with_vm(..)` restore; can silently fail inside a drain | no |
| `luau/facade/state.rs:950` `Scope::drop` | `try_borrow_mut`, destructs scoped callbacks | no |
| `vm.rs:845` `ModuleExecutionGuard::drop` | drops `InterruptGuard`, sets `Cell`s | no |
| `vm.rs:1688` `ScriptVm::drop` | `lua_audio_samples::shutdown`, `lua_audio_processors::shutdown` | no found path (poison-recovering locks) |
| `vm.rs:3340` `LuaScriptInstance::drop` → `dispose_script_lifetime` (`:1439`) | owner `dispose`, `table.take()` (queues a root), `clear_scripted_object` | no found path; **reachable inside a drain** (a collected `artboard:instance()` payload owns same-VM script instances), and the File-lease cascade under it was not followed (2(c)) |
| `vm/native_registration.rs:16` `RegistrationScope::drop` | `let _ = set_named_registry_value` (itself drains) | no |
| `vm/view_model.rs:292` `ScriptedPropertyState::drop` | `tracker.borrow_mut()` on owner/orphan vecs (panicking) | no today: every other borrow is a statement (2(b)) |
| `vm/view_model.rs:306` `ScriptedPropertyOwnerScope::drop` | `current_owner.borrow_mut()` (panicking), drops the old owner while held | no today; non-LIFO drops would restore the wrong owner (logic) |
| `vm/lua_transition.rs:16` `ChildScope::drop` | `borrow_mut().take()` on a short-borrowed cell | no |
| `vm/lua_canvas.rs:314` `ScriptedCanvas::drop` | `end_renderer` → `ScriptedRenderer::end` (`target.borrow_mut()`, `lua_renderer.rs:276`, panicking but unconflicted) → frame take | no in practice: the `:172` `expect` needs a prior successful `save` |
| `vm/lua_mesh.rs:287` `ScriptedImageMeshInstances::drop` | saturating `Cell` arithmetic | no |
| `vm/lua_image_decode.rs:84` `CompletionQueue::drop` | atomic `fetch_sub` | no |
| `vm/lua_image_decode.rs:212` `ImageDecodeRegistryOwner::drop` | `cancel_all`: `pending.borrow_mut()` (panicking, never held across anything) + atomic cancel | no in practice |
| `vm/lua_renderer.rs:137` `ScopedRendererAccess::drop` | `try_with` + `try_borrow_mut`; sets `RENDERER_STACK_POISONED` on failure (never cleared) | no |
| `vm/lua_renderer.rs:165` `ActiveRendererBorrow::drop` | same | no |
| `vm/lua_renderer.rs:309` `RestoreTarget::drop` | `target.borrow_mut()` (panicking; target only borrowed as statement temporaries) | no in practice |
| `vm/lua_audio_samples.rs:54` `CompletionQueue::drop` | atomic counter | no |
| `vm/lua_audio_samples.rs:136` `PendingDecode::drop` | atomic store, `WorkCancellationHandle::cancel`, drops a `RegistryKey` and a `DecodeReservation` | no |
| `vm/lua_audio_samples.rs:155` `StreamWait::drop` | counter | no |
| `vm/lua_audio_samples.rs:188` `RegistryOwner::drop` | `shutdown`: `borrow_mut` of `pending`/`waits`/`streams` and each `HostStream` | only if one is borrowed at app-data teardown; none found |
| `vm/lua_audio_samples.rs:405` `HostStream::drop` | counter | no |
| `vm/lua_audio_samples.rs:432` `StreamController::drop` | `release` → `HostStream` `borrow_mut()` (`:426`) | **yes if a `HostStream` `Ref` is live at the drain** (2(a) row 12); unreachable today |
| `vm/lua_audio_processors.rs:35` `ProcessorOwnerInner::drop` | `dispose`: `processors.borrow_mut()`, `stop` (may block in `detach_renderer`, drops processor VMs) | only if `processors` is borrowed; none crosses a VM call. Can overflow the stack through a hold chain in a processor's values (4.4) |
| `vm/luau_audio_samples.rs:227` `PcmStreamReader::drop` | atomic add, ignores `Err` | no |
| `vm/luau_audio_shared.rs:147` `Charge::drop` | spin-locks `budget.state`, subtracts | no (would spin only if the same thread held it: never) |
| `vm/luau_audio_shared.rs:991` `RegionGuard::drop` | atomic store | no |
| `vm/luau_audio_shared_web.rs:56` `DecodeReservation`, `:87` `Handle`, `:390` `RegionGuard` (wasm) | JS `request`, result ignored | no (OOM trap only) |
| `vm/luau_audio_processor_engine.rs:308` `Shared::drop` | atomic counter | no |
| `vm/luau_audio_processor_engine.rs:355` `Owner::drop` | `stop` (`Mutex::into_inner`, `detach_renderer` blocks for the audio thread, drops the `HostedVm`), wasm `WEB_OWNERS.borrow_mut` + `request` | borrow: no; **stack overflow via a hold chain (4.4)**; may run on the init thread (`:466`) |
| `vm/browser_kernel.rs:92` `BrowserKernel::drop` (wasm) | `retire` atomics, `HostedVm` drop | no |

Not `impl Drop` in this crate but run by payload drains (from g2):
`Pass` (`gpu_canvas_ore/pass.rs:15`, `Box<dyn RenderPassApi>`): an inline
pass's drop panics (`nuxie-ore-metal` `ore_deferred_render_pass.rs:37`
`expect`, `:38` `borrow_mut`), and scripts get only recorded passes, so
unreachable; `Canvas` (`gpu_canvas_ore/canvas.rs:11`, `RenderCanvasHandle`):
`RenderCanvas::drop` → `CanvasRetirer::retire` →
`self.state.lock().unwrap()` (`nuxie-renderer`
`deferred/cmd/deferred_canvas.rs:38`) panics on a poisoned mutex.
`PreparedGpuCanvasShader` (`gpu_canvas.rs:947`) can hold the last
`OreContextHandle`, so the context's own `Drop` (`ore_deferred_context.rs:365`)
can run in a drain. The `Error::CallbackError` chain's derived `Drop`
(4.1) and the hold chains (4.4) are the two `Drop` paths that were made to
abort.

## Could not be determined

- Rust stack used per `max_reentry` level through real bindings, and whether
  16 levels fit a 512 KiB iOS secondary thread (46-49 KiB per level was
  measured unoptimized in the VM alone).
- Release-build thresholds for every overflow above: all measurements are
  a debug build on a 2 MiB test thread. Whether `Display` or `Drop` of the
  `CallbackError` chain overflows first on a device, and the frame size of
  the audio hold-chain drop.
- The exact frame that overflows while the `CallbackError` chain grows past
  ~40,000 layers (most likely `is_structured`).
- Which `nuxie_runtime` writers call `on_value_changed` outside a host
  transaction (data-bind target→source, state-machine actions), which
  decides whether 2(a) row 1 is reachable.
- Whether the embedder can hold a `CoreHandle`, the ORE context or the
  `DeferredCanvasHostHandle` borrowed while it drives script (2(c)).
- The File-lease drop cascade under `LuaScriptInstance::drop` and
  `ScriptedPropertyState::dispose` inside a drain.
- Whether the ORE recorder keeps copies of `Buffer:write` /
  `Texture:upload` data until replay (5, row 7).
- Whether the promise library's nested `pcall` re-entry is counted by
  `max_reentry` (4.5).
- The `sort` recursion's frame size, hence how many pending scripts
  overflow it.
- Whether a script can store a handle into an artboard's `data` cell and
  leak the cycle (`lua_artboards.rs:44`).
- The `HostStream::restart` hang, the `print` buffer growth and the
  nested-entry budget reset were derived from the code, not run.
