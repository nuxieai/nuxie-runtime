# Safe Luau VM, Phase 2 lane 7: `nuxie-scripting` on the safe VM (2026-11-11)

Lane 7 of Phase 2 in `docs/luau-safe-vm-plan.md`: build and run
`crates/nuxie-scripting` on `crates/nuxie-luau-vm`, with the vendored VM
still the default. Branch `claude/luau-vm-lane7-scripting-port-5cf520`
on `origin/main` `94b0d21fcb`; not pushed, no PR.

**Outcome.**

- **A build-time VM choice.** `nuxie-scripting` has a `nuxie-vm` feature
  beside the default `luau`. The default build, its tests and its binaries
  are unchanged. Under `nuxie-vm` the crate compiles against the safe VM,
  without `luaur-rt`, through one binding source: `crate::luau` is
  `luaur-rt` under the default and a `luaur-rt`-shaped facade over the
  safe VM under `nuxie-vm`.
- **Every raw hatch outside the audio engine is off the vendored VM**
  under `nuxie-vm`: R1–R32 lifted, reimplemented or dropped as the seam
  doc proposed (final states in `docs/luau-safe-vm-seam.md` section 8).
  Raw references left under `nuxie-vm`: 6, all in the audio processor
  engine, which stays on `luaur-vm` under both settings (R33–R36).
- **The whole `nuxie-scripting` suite passes on the safe VM**: 40 of 40
  targets, 748 tests passed, 0 failed, 5 ignored (the same 5 as the
  default). Two failures the first run found were real gaps,
  fixed with a test each: the safe VM's generic `for` ignored `__iter` on
  userdata (a VM bug), and the facade flattened `luaur-rt`'s structured
  callback errors.
- **Scripted goldens on the safe VM:** 347 exact, 16 diverges, 2 failures
  against the default's 348 / 16 / 1. The new failure,
  `data_bind_blob_test`, is authored content that depends on `pairs`
  order, which the safe VM's hash part does not keep (D4). Not fixed here;
  it is the first Phase 3 item.
- **No new `unsafe`**, both ratchets unchanged; `nuxie-luau-vm` stays at
  zero `unsafe` and zero forced panic lints.

## 1. Commits

| Commit | Concern |
|---|---|
| `f71fd04ef3` | VM selection features (`luau-host`, `luau`, `nuxie-vm`) and the `crate::luau` adapter; bindings name handle types through it |
| `c97bc7c03b` | VM: `Vm::userdata_dispatchers` and `AnyUserData::payload` (host-built userdata dispatch, erased payload access) |
| `a1dcbe2a4b` | VM: `Thread::stack_value` (a parked or dead thread's stack, as C++ `lua_pushvalue` + `lua_xmove`) |
| `f87d8a9563` | the facade: `luaur-rt`'s handle API over the safe VM |
| `28191fbbe9` | Mat4 on the safe VM (R24–R29), `vm/lua_mat4_safe.rs` |
| `0747f727cc` | the remaining seam sites (R1–R23, R30–R32) and the raw test observations |
| `0c87a92f04` | VM fix: generic `for` reads `__iter` and `__call` from userdata |
| `3f6ff35506` | VM: `Vm::take_error_object` (a non-string error object that reached the host) |
| `3e73ee9731` | facade: structured callback errors, as `luaur-rt` |
| `bd71fdf302` | docs: the facade's notes in `luau/mod.rs` |
| `7467ff1f53` | `rust-golden-runner`: `scripting-nuxie-vm` pass-through feature (the scripted goldens on the safe VM) |
| `be6b77a0ec` | `nuxie`: `scripting-nuxie-vm` pass-through feature |
| `f3fb37dfab` | test: `tests/default_features.rs` (luaur-rt's async API) is vendored-only (review round 1) |
| `3befd2795c` | facade: handle equality is per-VM identity, strings by content; unused `equals`/`type_id` removed (round 1) |
| `ef4b6e1d1f` | facade: error objects classified by the object the VM kept, not by text (round 1) |
| `d90e34c606` | facade: `set_memory_limit(0)` removes the cap (round 1) |
| `ad8fdd4d8a` | facade: a scope revokes its callbacks when it unwinds (round 2) |
| `46dc31025a` | facade: a callback's `Lua` does not own the VM (round 2) |
| `d23c373bcb` | the new setting's clippy findings in this lane's code (3 facade `arithmetic_side_effects` sites among them) |
| `50e378725e` | VM: `Thread::stack_value` reads a parked thread's API stack only (round 3) |
| `2d9ee928e8` | VM: a kept error object belongs to the latest failure (round 3) |
| `eaabc1fb72` | facade: a VM operation that unwinds puts the VM back (round 3) |
| `501b2f8d5d` | facade: a module error is classified by its value (round 3) |
| `5365004ec0` | facade: the unused `Function::bind` removed (round 4) |
| `4a974b5aeb` | facade: an ordinary failed resume consumes the thread's error (round 4) |
| (docs commit) | this report, seam doc section 8, plan notes |

The tip builds and passes its tests under both settings. Commits were
built as they were made, except three split out of one tested change in
review rounds 3 and 4 (`50e378725e`, `eaabc1fb72`, `5365004ec0`), which
were not rebuilt on their own. Authored by Levi; no trailer.

## 2. Selecting the VM

`crates/nuxie-scripting/Cargo.toml`:

| Feature | Means |
|---|---|
| `luau-host` | the scripting host both settings share (`luaur-common`, `luaur-vm` for the audio engine, image/render deps). Code gates on this. It selects no VM. |
| `luau` (default) | `luau-host` + `luaur-rt`. Unchanged meaning; every other crate's feature graph selects it. |
| `nuxie-vm` | `luau-host` + `nuxie-luau-vm` + the vendored compiler crates (`luaur-ast`, `luaur-bytecode`, `luaur-compiler`; a compiler, not a VM). If both are on, `nuxie-vm` is the VM the crate runs. |
| `compiler` | source compilation for tests and tooling; `luaur-rt`'s `Lua::load` under `luau`, the facade's `compile` + `load_bytecode` under `nuxie-vm` |

Bazel: the checked-in `BUILD.bazel` keeps the default graph (its targets
gain only the `luau-host` crate feature). `tools/bazel/runtime.py`
materializes any other feature graph under `build/bazel-config/`, so the
new setting needs no checked-in target:

```bash
python3 tools/bazel/runtime.py test -p nuxie-scripting
```

```bash
python3 tools/bazel/runtime.py test -p nuxie-scripting --no-default-features --features nuxie-vm,compiler,js-host-seed,upstream-test-seams,tools
```

(`RIVE_RUNTIME_DIR` must point at a checkout of the pinned
`cd5351c3`, e.g. `~/dev/worktrees/rive-runtime-pin-cd5351c3`.) A
downstream crate selects the safe VM by adding `nuxie-scripting/nuxie-vm`
to its graph (section 7).

## 3. The facade

`crates/nuxie-scripting/src/luau/facade/` (no `unsafe`; the crate's panic
lints denied in the module):

- **The VM sits in a slot on `Lua`.** `luaur-rt`'s API is implicit-VM
  (`table.get(k)`); the safe VM's is explicit (`table.get(&mut vm, k)`). A
  facade operation takes the VM out of the slot and puts it back. A host
  callback receives the running VM as `&mut HostCtx`; the trampoline lends
  it to the slot for the callback's duration by swapping it with a pooled
  spare, so every handle operation inside the callback reaches it. Re-entry
  while the VM is out is an error ("the Lua VM is in use by another
  operation"), not a panic.
- **Handles** are `{ lua, raw: Result<vm handle, Error> }`, so `luaur-rt`'s
  infallible signatures (`create_table`, `globals`) stay infallible and
  report a failed creation at first use.
- **Userdata** keep their Rust value in the VM's `Rc<RefCell<T>>`; `borrow`
  returns a `Ref` from the erased payload. Each `UserData` type gets one
  metatable built from its registration (methods table, meta methods,
  getters/setters behind `Vm::userdata_dispatchers`), cached per `TypeId`.
- **Ownership** follows `luaur-rt`: a `Lua` from `Lua::new` and its
  clones own the VM, which is dropped with the last of them. The `Lua` a
  callback or interrupt receives is borrowed (`luaur-rt`'s trampoline
  builds a non-owning `LuaInner`), so a userdata made in a callback whose
  payload keeps a handle, as the canvases do, does not keep its VM alive.
  A handle that outlives its VM reports "the Lua VM was dropped".
- **Errors** keep `luaur-rt`'s variants and `Display` text. A callback's
  `LuaLRuntimeError` is raised positioned (`luaL_error`), out of memory
  keeps its status, the scope-destruction errors travel as a userdata error
  object and come back as `CallbackError` (one layer per protected
  boundary), and anything else is raised as its `Display` text, as
  `luaur-rt`'s trampoline does.

## 4. Seam rows

`docs/luau-safe-vm-seam.md` section 8 has the final state per row. In
short: R1, R6–R23, R30–R32 lifted; R24–R29 reimplemented
(`lua_mat4_safe.rs`); R2–R5 dropped; R33–R36 not retired (audio, by
decision). Of the raw test observations, T1 is a whole test (the atom
resolver, gone with R4) and is default-only; T2–T6 are statements inside
tests that run under both settings, with only the raw stack and
thread-data observations compiled in the default setting.

Grep-level check, under `nuxie-vm` (a lexical scan of `src/` and `tests/`
for `luaur_vm`, `luaur_rt` and `exec_raw` outside items gated
`cfg(not(feature = "nuxie-vm"))`):

| | count | where |
|---|---:|---|
| `exec_raw` | 0 | |
| `luaur_rt` | 0 | |
| `luaur_vm` | 6 | `vm/luau_audio_processor_engine.rs` (imports at `:20-40`, `:1563`) |

## 5. VM API added by this lane

Each in its own commit with a test in `nuxie-luau-vm` or `luau-vm-diff`:

| Addition | Why | Test |
|---|---|---|
| `Vm::userdata_dispatchers(getters, setters, methods)` | the facade builds `luaur-rt`'s per-type `__index`/`__newindex` over host-filled tables | `host_built_userdata_dispatchers_route_fields_and_methods` |
| `AnyUserData::payload(&vm)` | `AnyUserData::borrow` without knowing `T` at the handle | `userdata_payloads_are_reachable_erased` |
| `Thread::stack_value(&mut vm, i)` | module hooks read a dead module thread's error (C++ `lua_pushvalue` + `lua_xmove`) | `thread_stack_values_read_as_cpp_leaves_them` |
| `Vm::take_error_object()` | a non-string error object reached the host only as text; `luaur-rt`'s structured callback errors need the object | `a_non_string_error_object_is_kept_until_taken` |
| FORGPREP reads `__iter`/`__call` from userdata (fix) | C++ takes the metatable of a table or full userdata; the VM looked only at tables, so `for _, g in line do` on a `TextLine` failed | `generic_for_over_userdata_uses_iter_then_call` |

## 6. Bytecode versions

| Producer | Version | Loaded under `nuxie-vm` by |
|---|---|---|
| Nuxie Editor (nuxie-dev, flags off) | 7 | `corpus_scripts` (`editor_bytecode_matrix_rows_extract_pin_version_and_load`) and the scripted goldens |
| Historical editor assets | 6 | the same matrix test |
| `nuxie-scripting/build.rs` helpers (`promise-library`, `data-value-metatable`, `property-metatable`), `set_all_flags(true)` | 11 | every `ScriptVm` |
| Tests via `crate::luau::compile` (`set_all_flags(true)`) | 11 | the whole suite |

`luaur-rt`'s own four build-time helpers (`bind`, `table-len`,
`userdata-dispatch`, `async-poller`) are not part of the new setting: the
facade implements `#` and userdata dispatch on the VM API, and has no
`Function::bind` (no caller; removed in review round 4) and no async
poller. The safe VM admits versions 3–11
(`nuxie-luau-vm/src/bytecode/mod.rs`); the product validator admits 3–13.
**Admitted by the product but not emitted by anything:** v12 (cost
model) and v13 (`VECTORD`), both behind flags that are off, and v100
(classes), which the product rejects. The safe VM rejects v12 and v13; that
is correct while nothing emits them, and must change if the editor moves.

## 7. Builds per setting

| Crate | Default | `nuxie-vm` |
|---|---|---|
| `nuxie-scripting` (`--all-targets`) | builds (the test run in section 8) | builds (the test run in section 8) |
| `nuxie` (`--features scripting[,nuxie-scripting/nuxie-vm] --all-targets`) | builds | library builds; `--all-targets` fails in the dev-dependency `tools/silver-corpus` (11 errors, Phase 3 item 3) |
| `nux-capi` (same) | builds (library and `--all-targets`) | builds (library and `--all-targets`), with a scratch feature; see below |

`runtime.py` takes no `dep/feature` syntax, so a downstream crate reaches
the new setting through a pass-through feature of its own:
`nuxie` has `scripting-nuxie-vm = ["scripting", "nuxie-scripting?/nuxie-vm"]`
and `rust-golden-runner` has
`scripting-nuxie-vm = ["scripting", "nuxie-scripting/nuxie-vm"]`. Both VMs
are then compiled, and `nuxie-vm` is the one `nuxie-scripting` runs.

`nux-capi` has no such feature. Adding one trips
`tools/pure-runtime-boundary/check.py` ("forwards forbidden portable ABI
facade feature"), so it was added only in a scratch edit to prove the
build and then reverted. A committed one needs a boundary-policy decision
(Phase 3 item 9).

`tools/silver-corpus` mixes `luaur_rt` types with `nuxie_scripting::luau`
in `scripted_misc_replay.rs` and `scripted_source.rs`; under `nuxie-vm`
those name different types, so `nuxie`'s tests do not build. The `nuxie`
library does.

## 8. Tests

`nuxie-scripting`, per target, passed/failed/ignored (load averages are
the 1-minute value before and after each run; the host ran other lanes
throughout). The `nuxie-vm` column ran at the tip, `4a974b5aeb`; the
default column at `501b2f8d5d` and the middle one at `d23c373bcb`; the
commits after each touch only code compiled under `nuxie-vm` and the VM
crate:

| Target | default | default + `upstream-test-seams,tools` | `nuxie-vm` (+ same features) |
|---|---|---|---|
| `async_shader_instantiation` | 6/0/0 | 6/0/0 | 6/0/0 |
| `buffer_extensions` | 37/0/0 | 37/0/0 | 37/0/0 |
| `corpus_scripts` | 10/0/0 | 10/0/0 | 10/0/0 |
| `default_features` | 1/0/0 | 1/0/0 | 0/0/0 |
| `deterministic_seed` | 1/0/0 | 1/0/0 | 1/0/0 |
| `explicit_error_recovery` | 2/0/0 | 2/0/0 | 2/0/0 |
| `generic_host_commands` | 7/0/0 | 7/0/0 | 7/0/0 |
| `gpu_canvas_inputs` | 8/0/0 | 8/0/0 | 8/0/0 |
| `host_logging` | 7/0/0 | 7/0/0 | 7/0/0 |
| `library_scope` | 5/0/0 | 5/0/0 | 5/0/0 |
| `listener_invocations` | 11/0/0 | 11/0/0 | 11/0/0 |
| `logging_factory` | 2/0/0 | 2/0/0 | 2/0/0 |
| `luaur_table_call_contract` | 2/0/0 | 2/0/0 | 2/0/0 |
| `mat4_bindings` | 14/0/0 | 14/0/0 | 14/0/0 |
| `path_bindings` | 5/0/0 | 5/0/0 | 5/0/0 |
| `path_render_path_lifetime` | 1/0/0 | 1/0/0 | 1/0/0 |
| `promise_scenarios` | 4/0/1 | 4/0/1 | 4/0/1 |
| `renderer_bindings` | 4/0/0 | 4/0/0 | 4/0/0 |
| `scripted_interpolator` | 1/0/0 | 1/0/0 | 1/0/0 |
| `scripted_layout` | 10/0/0 | 10/0/0 | 10/0/0 |
| `scripted_transition` | 3/0/0 | 3/0/0 | 3/0/0 |
| `shader_asset_resolution` | 18/0/0 | 18/0/0 | 18/0/0 |
| `upstream_async_work_host` | 0/0/0 | 6/0/0 | 6/0/0 |
| `upstream_module_hooks` | 8/0/0 | 8/0/0 | 8/0/0 |
| `upstream_scripting_converter` | 7/0/0 | 7/0/0 | 7/0/0 |
| `upstream_scripting_direct_field_flag` | 1/0/1 | 1/0/1 | 1/0/1 |
| `upstream_scripting_drawable_scroll` | 0/0/0 | 7/0/0 | 7/0/0 |
| `upstream_scripting_mat4` | 17/0/1 | 17/0/1 | 17/0/1 |
| `upstream_scripting_paint` | 4/0/0 | 4/0/0 | 4/0/0 |
| `upstream_scripting_renderer` | 0/0/0 | 1/0/0 | 1/0/0 |
| `upstream_scripting_require_check` | 10/0/0 | 10/0/0 | 10/0/0 |
| `upstream_scripting_scope` | 4/0/0 | 4/0/0 | 4/0/0 |
| `upstream_scripting_text` | 0/0/0 | 13/0/0 | 13/0/0 |
| `upstream_scripting_update_phase_guard` | 3/0/0 | 3/0/0 | 3/0/0 |
| `upstream_scripting_vector` | 14/0/1 | 14/0/1 | 14/0/1 |
| `upstream_scripting_wake_advance` | 0/0/0 | 6/0/0 | 6/0/0 |
| `upstream_signed_content_header` | 16/0/0 | 16/0/0 | 16/0/0 |
| `vm_boot` | 20/0/0 | 20/0/0 | 20/0/0 |
| `wrapper_bindings` | 3/0/0 | 3/0/0 | 3/0/0 |
| `unit_test` | 430/0/1 | 439/0/1 | 450/0/1 |
| **total (40 targets)** | **696/0/5** | **738/0/5** | **748/0/5** |
| 1-min load before / after | 18.0 / 13.3 | 10.3 / 9.5 | 51.4 / 53.3 |

Differences in counts are cfg'd tests, not skipped ones:

- `default_features` (1 test) is `cfg(feature = "async")`: it asserts
  `luaur-rt`'s `create_async_function`/`call_async` exist. The seam found
  no product caller; the facade has no async API.
- `unit_test` 450 under `nuxie-vm` against 439: +12 facade tests
  (`luau/facade/tests.rs`, compiled only under `nuxie-vm`) and -1,
  `vm::context_init_tests::script_vm_installs_the_compile_time_atom_resolver`,
  which is default-only because R4 (the compile-time atom hook) is dropped
  on the safe VM (seam row T1). T2–T6 are raw observations inside tests
  that run under both settings, so they do not change the count.
- With the plain default command, five `upstream-test-seams` targets
  compile to zero tests; the default row above is run with
  `--features upstream-test-seams,tools` to compare like for like.

Error text (D8): no test needed a setting-dependent expectation. Every
product test that pins a host error text passes unchanged on the safe VM.

## 9. Scripted goldens

`make scripted-golden-compare` with `RIVE_RUNTIME_DIR` at the pinned
`cd5351c3`, release C++ runner. The safe-VM run uses the Rust runner built
with `rust-golden-runner`'s new `scripting-nuxie-vm` feature, passed as
`SCRIPTED_RUST_GOLDEN_RUNNER`:

```bash
python3 tools/bazel/runtime.py build -p rust-golden-runner --features scripting-nuxie-vm
```

| Setting | entries | exact | diverges | failures | 1-min load before/after |
|---|---:|---:|---:|---:|---|
| default | 364 | 348 | 16 | 1 (`semantic_text_inference_lt1`, known red) | 15.3 / 23.3 |
| `nuxie-vm` | 364 | 347 | 16 | 2 | 13.3 / 10.7 |
| `nuxie-vm`, rerun after review round 2 (runner rebuilt at `46dc31025a`) | 364 | 347 | 16 | 2 (the same two) | 8.8 / 7.7 |
| `nuxie-vm`, final, at the tip `4a974b5aeb` (runner rebuilt) | 364 | 347 | 16 | 2 (the same two) | 87.6 / 82.4 |

(The tool's summary line prints `exact=348` for every run: it counts the
entries labelled exact, including one that then fails exact
verification. The exact column here is entries that verified.)

The 16 diverges are the same rows with the same signatures. The one new
failure is **`data_bind_blob_test`**: a list label's glyph outlines read `Fill 1`
where C++ and the vendored VM draw `1` (read from the outlines: the
`name` attribute instead of `version`). The fixture's editor script parses an XML
blob with an embedded `XmlDom` Luau module and walks each node's attribute
table with `pairs`; the label shows the first attribute it meets. The safe
VM iterates string-keyed hash tables in a different order from C++ Luau
once a table has more than a few keys. That is the D4 decision ("the hash
part is free ... at the cost of 3 hash-order sites"), and this is the first
authored content seen to depend on it. Reproduction, as a
`tests/luau-vm-diff` case (4 of 4 profiles disagree):

```lua
local keys = { "defaultStateMachineId", "selectedAnimations", "clip", "width",
  "height", "styleId", "name", "id" }
local t = {}
for i, k in ipairs(keys) do t[k] = tostring(i) end
local out = {}
for k in pairs(t) do out[#out + 1] = k end
print(table.concat(out, ","))
-- vendored: defaultStateMachineId,id,styleId,name,selectedAnimations,height,clip,width
-- safe VM:  selectedAnimations,clip,id,width,height,styleId,defaultStateMachineId,name
```

Not fixed here: matching C++'s order means porting Luau's string hash and
node placement into the hash part, which reopens D4 (Phase 3 list, item 1).

## 10. The audio boundary

`vm/luau_audio_processor_engine.rs` is a second, raw embedding of the
vendored VM: a `lua_State` per audio processor with its own allocator,
interrupt, sandbox and 29 C functions, run on audio threads. Levi decided
(2026-11-11) that it stays on the vendored VM in this lane. It shares no
handle or value with the scripting VM (requests cross as `SharedValue`
copies and bytecode bytes), so both settings compile it unchanged; under
`nuxie-vm` it is the only `luaur-vm` user in the crate. What a later lane
needs to move it, from seam section 6 items 9–10 and what lane 6 left out:

1. a `Send` VM with no thread-local state (the facade's `Lua` is `Rc`
   based; the audio engine moves a VM across threads);
2. a per-VM allocation budget with a sticky exhaustion flag (lane 6 has
   it) reachable from a host function;
3. an explicit GC step after every block (`lua_gc(LUA_GCSTEP)`);
4. in-place `Buffer` byte access for the output block;
5. host functions carrying an upvalue (the channel index) or closures,
   which the embedding API has;
6. `luaL_sandbox` (the facade supports `sandbox(true)` only).

## 11. Gates

Run at `501b2f8d5d`, after review round 3; round 4 changed only facade
code, which none of the `luau-*` gates compiles. Clippy for
`nuxie-scripting` under `nuxie-vm` was rerun at the tip. The 1-minute load
before each run is in brackets; the host ran other lanes:

| Gate | Result |
|---|---|
| `make luau-conformance` [13.3] | PASS: 30 of 30 required, 0 panics |
| `make luau-conformance-nuxie` [54.7] | PASS: 36 of 36 required, 0 panics |
| `make luau-vm-diff` [10.7] | 174 runs, 0 disagreements, bytecode versions 7 and 11, every admitted opcode reached |
| `make luau-vm-ratchets` (`ratchet-unsafe`, `ratchet-panic-lints`) [51.0] | PASS: 3918 `unsafe` sites, +0; `nuxie-luau-vm` 0 `unsafe`, 0 forced panic-lint sites |
| `python3 tools/bazel/generate.py --check` | OK |
| clippy `nuxie-luau-vm`, `luau-vm-diff` (`--all-targets`) | clean |
| clippy `nuxie-scripting`, default | 74 errors, the same 74 sites as `origin/main` (the crate is outside the repo's lint gate, `Makefile` `lint-gate`) |
| clippy `nuxie-scripting`, `nuxie-vm` | 71 errors, a subset of `main`'s sites; the 7 the first run found in this lane's code are fixed (`d23c373bcb`), among them 3 `arithmetic_side_effects` sites in the facade, which denies the panic lints |
| `nuxie-scripting` tests, both settings | section 8 |
| scripted goldens, both settings | section 9 |

## 12. Review

`codex review --base origin/main`, rounds until clean or four.

**Round 1** (7 findings):

| Finding | Verdict | Fix |
|---|---|---|
| P1 `Chunk::eval` made crate-private breaks integration callers | real against the tree it saw; the change was already reverted (see below) | none needed |
| P2 default features + `nuxie-vm` still compiles the async API test | real | `tests/default_features.rs` is vendored-only (`f3fb37dfab`); checked with `--features nuxie-vm` |
| P2 `set_memory_limit(0)` installed a zero-byte budget | real | zero means no cap (`d90e34c606`), test `a_zero_memory_limit_means_no_limit` |
| P2 handle equality compared VM-local addresses only; strings by address | real | per-VM identity, strings by content (`3befd2795c`), test `handle_equality_is_per_vm_identity_and_string_content` |
| P2 `Table::equals` ignored `__eq` | real, no caller | removed with `Value::equals` and `AnyUserData::equals` (`3befd2795c`) |
| P2 `AnyUserData::type_id` returned `RefCell<T>`'s id | real, no caller | removed (`3befd2795c`) |
| P2 a string reading `(error object is a table value)` became `<non-string error>` | real | classified by the object the VM kept (`ef4b6e1d1f`), test `error_objects_are_told_from_strings_that_look_like_them` |

The `eval` episode: `tools/pure-runtime-boundary/check.py` flags any new
`pub fn eval` in a runtime crate (the editor-gpu-tooling debt marker), so
the facade's `Chunk::eval` was briefly made crate-private; an integration
test (`tests/upstream_async_work_host.rs`) evaluates source through it, so
it stays public, as `luaur-rt`'s is (`luaur-rt` is a vendored, unscanned
path). The checker is red on `main` (129 findings, none in this lane's
files); this branch adds one: `crates/nuxie-scripting/src/luau/facade/state.rs`
`pub fn eval`. It is left for Levi: allowlist the facade file for that
marker, or keep `Chunk::eval` out of the facade and move that test to
`into_function`/`call` under `nuxie-vm`.

**Round 2** (2 findings):

| Finding | Verdict | Fix |
|---|---|---|
| P1 a `Lua` held by a guest-retained handle owns the VM: `Shared → VM heap → payload → handle → Shared` leaks the VM when a callback-made userdata keeps a handle | real; `luaur-rt` hands callbacks a borrowed `LuaInner` | the `Lua` a callback or interrupt receives is non-owning (`46dc31025a`), tests `handles_made_in_callbacks_do_not_keep_the_vm_alive`, `a_handle_outliving_its_vm_reports_it` |
| P2 a scope body that panics after publishing a callback leaves it callable | real | revocation in `Scope`'s `Drop` (`ad8fdd4d8a`), test `scoped_callbacks_are_revoked_when_the_scope_unwinds` |

**Round 3** (4 findings):

| Finding | Verdict | Fix |
|---|---|---|
| P2 a panic inside `Lua::with_vm` dropped the VM it had taken: the slot stayed empty, or inside a callback the outer operation kept the spare | real | an unwind guard puts it back (`eaabc1fb72`), test `a_panicking_vm_operation_puts_the_vm_back` |
| P2 `Thread::stack_value` exposed the execution stack: after a yield or return it showed internal slots where C++ and `luaur-rt` show nothing | real (VM API) | it reads the API stack only: a fresh thread's function, a failed thread's error, else nothing (`50e378725e`), test `a_thread_that_yielded_or_returned_holds_nothing` and the extended `thread_stack_values_read_as_cpp_leaves_them`. The failed thread keeps its error on top, as C++ `lua_resume` leaves it and as the module error hook reads it |
| P2 `resume_module` told an error object by the text `<non-string error>`, so a module raising that string lost its message | real | classified by the value the thread died with (`501b2f8d5d`), test `a_module_error_is_classified_by_its_value` |
| P2 an error object a host handled inside `with_vm` stayed kept and classified a later lookalike string | real (VM API) | a string or number error reaching the host clears the kept object (`2d9ee928e8`), tests `a_kept_error_object_belongs_to_the_latest_failure` (VM) and `a_handled_error_object_does_not_classify_a_later_string` (facade) |

**Round 4** (3 findings; the last round the brief allows):

| Finding | Verdict | Fix |
|---|---|---|
| P2 `Function::bind` held an owning handle inside the VM (a cycle that kept the VM alive) | real; nothing calls `bind` | removed (`5365004ec0`), as round 1's unused `equals`/`type_id` were |
| P2 a bound function could not yield (it forwarded through a host call) | real, same function | the same removal |
| P2 after an ordinary failed `Thread::resume`, `stack_value(-1)` still showed the error; `luaur-rt`'s `finish_resume` moves it off | real | a failed thread reads nil except inside a module error hook, which `resume_module` hands the error it died with (`4a974b5aeb`), test `an_ordinary_failed_resume_consumes_the_thread_error`; the module-hook tests still read the error |

No fifth round was run. Every finding from the four rounds was real and
is fixed or removed with a test; none was rejected.

## 13. Not verified

- **`nuxie`'s own tests under the safe VM.** They do not build
  (`tools/silver-corpus`, Phase 3 item 3); only the `nuxie` library was
  built under `scripting-nuxie-vm`.
- **The `nux-capi` build under the safe VM** was proven with a scratch
  feature that is not committed (section 7).
- **Platform hosts.** Nothing outside this Mac's Bazel graph (iOS, Android,
  wasm) was built with `nuxie-vm`.
- **Performance.** Out of scope for the lane; nothing was measured.
- **Each fix's test failing without its fix** was checked by reasoning,
  not by reverting the fix and rerunning, for the round 2 tests.

## 14. Phase 3 list

1. **`pairs` order (D4).** `data_bind_blob_test` fails under `nuxie-vm`
   because authored content reads the first attribute `pairs` yields
   (section 9, with the eight-key reproduction). Matching C++ means
   porting Luau's string hash and node placement into the hash part, or a
   product decision that authored content may not depend on `pairs`
   order.
2. **The audio processor engine** (R33–R36) is still a raw `luaur-vm`
   embedding under both settings; section 10 lists what a later lane
   needs (a `Send` VM, a per-VM budget reachable from host code, explicit
   GC steps, in-place buffer access, upvalue-carrying host functions,
   `luaL_sandbox`).
3. **`tools/silver-corpus` on the facade.** `nuxie`'s tests do not build
   under `scripting-nuxie-vm` because the crate names `luaur_rt` types
   next to `nuxie_scripting::luau` ones. Reproduction:
   `python3 tools/bazel/runtime.py check -p nuxie --features scripting-nuxie-vm,test-support --all-targets`
   (11 errors in `scripted_misc_replay.rs` and `scripted_source.rs`). Fix:
   name the types through `nuxie_scripting::luau`.
4. **No async API in the facade.** `create_async_function`/`call_async`
   have no product caller (seam section 3); `tests/default_features.rs`
   is vendored-only.
5. **Interrupt `VmState::Yield`** is accepted and treated as `Continue`;
   the product never yields from an interrupt.
6. **`Lua::sandbox(false)`** is an error on the safe VM; the product only
   enters the sandbox.
7. **`tools/pure-runtime-boundary/check.py` and `Chunk::eval`**: the
   facade's `pub fn eval` adds one finding to a checker that is red on
   `main` (section 12). Levi to choose: allowlist the facade, or move the
   one test off `eval`.
8. **A `nux-capi` feature for the safe VM** trips the same checker's ABI
   facade rule; it needs a boundary-policy decision before it is
   committed.
9. **`luaur-rt`'s build-time helpers** (`bind`, `table-len`,
   `userdata-dispatch`, `async-poller`) are not used under `nuxie-vm`; the
   facade reimplements `#` and userdata dispatch on the VM API, and has no
   `bind` (no caller). If the facade is kept, those chunks retire with
   `luaur-rt`.
10. **The module error hook test's frame inspection**
    (`tests/upstream_module_hooks.rs`) reads the dead thread's frame with
    raw `lua_getinfo` and is default-only: the safe VM keeps a dead
    thread's frames but has no host API for them.
11. **Bytecode v12 and v13**: the safe VM rejects them while the product
    validator admits them (section 6). Nothing emits them today; this
    must change before the editor turns on either flag.
