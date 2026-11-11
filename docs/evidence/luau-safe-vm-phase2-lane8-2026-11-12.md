# Safe Luau VM, Phase 2 lane 8: the plain-data embedding core and the audio engine (2026-11-12)

Lane 8 of Phase 2 in `docs/luau-safe-vm-plan.md`: give `crates/nuxie-luau-vm`
an embedding core that holds plain data and moves between threads, keep
lane 6's API as a layer over it, and move the audio processor engine (seam
rows R33–R36) onto the core. The design is
`docs/luau-safe-vm-embedding-core.md` (stage A, amended after the
coordinator's review, approved by Levi). Branch
`claude/luau-vm-lane8-data-core-9757dd`, rebased on `origin/main`
`02a017b72a` (which carries PR #1374, the bridge-cost fixes); not pushed,
no PR.

**Outcome.**

- **`Core` is plain data and `Send`**, asserted at compile time
  (`crates/nuxie-luau-vm/src/vm.rs`). No `Rc`, no closure, no `dyn Any`, no
  host payload in it. **`HostedVm<H: Host>`** is the core, the host's state
  and a table of host function pointers; it is `Send` when `H` is.
- **Builtins are plain or resumable by type**, and plain natives (builtins
  and plain host functions) run frameless from `CALL`. A builtin that
  requests a call or a yield does not compile unless it is registered as
  resumable.
- **Lane 6's API is a layer**: `Vm = HostedVm<Compat>` in `src/embed/`,
  behind a boundary test that fails if a core source names the layer. The
  lane 7 facade and every `nuxie-scripting` binding compile against it
  unchanged.
- **The audio processor runs on the safe VM under `nuxie-vm`**:
  `HostedVm<AudioHost>`, a const assertion that the processor is `Send`,
  no `unsafe`, and no `luaur-vm` item under `nuxie-vm`. R33–R36 are
  retired under `nuxie-vm`; the default setting keeps the vendored
  processor, moved unchanged to `raw.rs`. The 15 engine tests pass under
  both settings, including
  `authored_pcm_and_shared_controls_survive_callback_thread_migration`.
- **`nuxie-scripting`: 40 of 40 targets under both settings** (section 10).
- **The audio-block prediction failed**: 4.6x luaur after the frameless
  path, against a predicted 2 to 2.6x (from 6.2x). The cause is a design
  error, not a missing tweak: a frameless call is still a call, and luaur
  runs these builtins without one (`FASTCALL`). Section 7 is the revisit.
- **The suite is 5.3% fewer instructions** (geomean 0.947, 89 benches;
  4.54x → 4.33x luaur), the builtin-call group 18% fewer, but **Lua calls
  (+8%), coroutines (+19%) and pcall (+7% to +15%) got dearer** where the
  design predicted small savings, and host calls through the lane 6 API
  went from 936 to 1,529 instructions. A commit scan pins it on the
  reserved frame of a frameless native, the driver split and protos by
  index (section 6.3); the revisit is section 6.4.

## 1. Commits

On `origin/main` `02a017b72a`, oldest first. Authored by Levi; no trailer.

| Commit | Concern |
|---|---|
| `cb5fa22f65` | design doc, stage A |
| `aa7d1111dd` | `tools/luau-bench`: the two audio-block benches, both VMs (decision 2) |
| `9371f7d7ae` | design amended after review (decisions 1–6) |
| `8ab45193d2` | string bytes `Arc`, native names boxed, builtin upvalues in the closure |
| `9cfadfb87e` | the frame stack as a type, with a reserved slot for a frameless native |
| `ff95eed204` | builtins plain or resumable by type; `%*` and the `__newindex` dispatcher as continuations; vm-diff case `format_tostring.luau` |
| `f932d31c10` | `Core` / `HostedVm<H>` split, host calls at the driver, `Compat` |
| `6e97c2583b` | prototypes by index |
| `0d572d3a80` | userdata payloads belong to the host |
| `5c6a0bb07b` | the interrupt is the host's; `Interruptible` builtins |
| `fd3a516d89` | core roots, lane 6 handles over them, `Core: Send` |
| `66abf721ed` | the lane 6 API is a layer (`src/embed/`), boundary test |
| `d5832db6e3` | frameless `CALL` for plain natives; plain host functions |
| `82be874739` | a stack API for hosts of their own |
| `643d85a343` | `nuxie-scripting`: the audio processor on the safe VM under `nuxie-vm` |
| `9d78a02b50` | a steady collection cycle allocates nothing (review round 1) |
| `b62da9522d` | a host function id names its VM (review round 1) |
| `af120563b6` | hosts drain their freed natives (review round 1) |
| `8a9a03bec3` | clippy: this lane's sites in the VM tests and the audio engine |
| `30d9005cd2` | a new handle reuses dropped handles' roots (review round 2) |
| `6986353899` | a callback's collection runs destructors before it returns (review round 2) |
| `63f801024a` | every safepoint of the lane 6 layer drains around it (review round 3) |
| `ac0c7b9ee2` | `enter` ends at a safepoint (review round 3) |
| `86a1527fdf` | `host_function` ends at a safepoint (review round 4) |
| (docs commit) | this report, design doc as built, plan, seam |

The branch was rebased twice, onto `6bff9f3ede` and then, after review,
onto `02a017b72a`; both were clean (main's commits touch none of this
branch's files). Every code commit was built and its crate's tests run as
it was made; `nuxie-scripting` under `nuxie-vm` was run at `f932d31c10`,
`fd3a516d89` and `643d85a343`, both settings at `643d85a343` and at the
tip.

## 2. The core's shape

| Piece | What it is | Where |
|---|---|---|
| `Core` | heap, stacks, `Frames`, interpreter, libraries, collector, budget, roots, prototype table. Plain data, `Send`. | `vm.rs`, `frames.rs`, `roots.rs` |
| `HostedVm<H>` | `{ core, host: H, fns: Vec<HostEntry<H>> }`, `Deref<Target = Core>`. The driver: runs full host calls, `xpcall` handlers, `meta_now`, `tostring_value`; no core code calls Lua from Rust. | `hosted.rs` |
| `Host` | `interrupt(&mut self, &mut Core) -> Result<(), HostError>`, `direct_field(..)`. Reached from the frame loop through the crate-private `Dispatch`, split from the core by field. | `host.rs`, `hosted.rs` |
| `NativeFn` | `Copy`, five variants: `Plain(fn(&mut Core, Args))`, `Interruptible(fn(&mut Core, &mut dyn Dispatch, Args))`, `Resumable(fn(&mut Resume, Args))`, `HostPlain(id, data)`, `HostFull(id, data)` | `native.rs` |
| Host functions | `HostEntry::Plain(fn(&mut PlainCtx<H>))`: no way to call Lua, frameless from `CALL`. `HostEntry::Full(fn(&mut HostedCtx<H>))` with an optional continuation: a frame, run at the driver. A function value is `(entry id, u32 data)`. | `hosted.rs` |
| `Root` | `{ vm, index, gen }`, released explicitly; a stale or foreign root is refused; a slot whose generation would wrap is retired. | `roots.rs` |
| Userdata | `(key: u64, metatable, tag, declared size)`; the sweep lists freed keys and freed owning natives once; the host drops payloads at its drain points. | `userdata.rs`, `gc.rs` |
| `Stack` | the C API's stack functions over one frame (1-based, negative from the top): reads give plain data or byte slices, writes push, raw-set, set a metatable, iterate; readonly tables refused. | `stack.rs` |
| `HostedVm` setup | `with_host`, `add_host_fn`, `host_function`, `enter` (a top-level frame for the host), `sandbox_globals` (`luaL_sandbox`), memory limit and sticky exhaustion flag, `collect_step`, `set_interrupt_enabled` | `hosted.rs`, `vm.rs` |
| The lane 6 layer | `Vm = HostedVm<Compat>`, `HostCtx<'a> = HostedCtx<'a, Compat>`; `Compat` holds closures (`Rc<HostFn>`) in a slot store reached through three fixed full entries, `Rc<RefCell<T>>` payloads under generation-checked keys, app data, the interrupt closure, direct-field getters, the handle release queue and the kept error object. | `embed/` |

`boundary_tests.rs` reads every core source (comments excluded) and fails
if one names `embed`, `Vm` or `Compat`, and fails if a module the crate
root or a core directory declares is missing from its list.

## 3. The audio engine on the core (R33–R36)

`crates/nuxie-scripting/src/vm/luau_audio_processor_engine.rs` keeps the
lane both settings share (format, offline clock, `LogRing`, `Shared`,
`Owner`, `ProcessorHandle`, initialization, the tests) and names the
processor through one alias. `raw.rs` is the vendored processor, moved
unchanged except for visibility and a `shared()` accessor, under
`cfg(not(feature = "nuxie-vm"))`. `safe.rs` is the new one. The browser
kernel (`browser_kernel.rs`, wasm32) uses the alias; its `luaur-common`
flag call is default-only.

| Row | Raw (vendored) | Safe (`safe.rs`) |
|---|---|---|
| R33 lifecycle | `lua_newstate` with a counting `lua_Alloc` and an `oom` flag; `lua_close`; interrupt via `lua_callbacks()`, raising preallocated registry strings; `luaL_openlibs`; globals via `lua_pushcclosurek`; `luaL_sandbox`; `unsafe impl Send` | `HostedVm::with_host(Limits { max_memory_bytes, .. }, AudioHost)`; the core's budget and its sticky exhaustion flag; `Drop` of plain data; `Host::interrupt` raising rooted strings (`HostError::from_root`, no allocation when the budget is spent); libraries opened by `with_host`; 20 host function entries (`print` and `require` full, the rest plain); `sandbox_globals`; `Send` by construction, asserted |
| R34 `load_module` | `luau_load`, `lua_pcall`, `lua_ref` cache, cycle check | `HostedCtx::push_loaded` (verbatim chunk name), `pcall_top`, `BTreeMap<String, Root>` cache, the same cycle check |
| R35 `process_block` | registry refs; `lua_tobuffer` raw pointer zeroed with `write_bytes`, read as `f32`; `lua_pcall`; `lua_gc(LUA_GCSTEP, 0)` | `enter`: zero through `Stack::buffer_mut` on the rooted buffer, `pcall_top`, read through `Stack::buffer`, `collect_step(0)`; no pointer |
| R36 C functions | closures with the channel index as upvalue; `lua_next` decoding; `lua_topointer` identity; `lua_newbuffer` | plain host functions whose `u32` data is the channel index; `Stack::next`; `Stack::to_pointer` of the rooted table; `Stack::push_buffer` |

The safe processor follows the raw one step for step: the same globals,
sandbox (C++ `luaL_sandbox` only, no per-thread globals), deadlines,
sticky failures and error texts; the engine tests did not change except
to reach the lane's state through `shared()`.

Under `nuxie-vm`, the engine's non-test code names no `luaur_vm`,
`luaur_rt` or `exec_raw` item. Its tests still compile bytecode with the
vendored compiler (`luaur_compiler`), as every `nuxie-scripting` test does
under both settings.

## 4. Tests (acceptance)

| Acceptance item | Test |
|---|---|
| a root keeps its value until released; stale, foreign and retired roots | `a_root_keeps_its_value_until_released`; `roots.rs`: `a_released_root_is_stale_after_its_slot_is_reused`, `another_vms_root_is_refused`, `a_slot_is_retired_before_its_generation_wraps` |
| the freed list is drained exactly once | `a_freed_userdata_key_and_owned_native_are_listed_exactly_once`, `a_collected_host_function_releases_its_closure_once`, `a_released_payload_key_finds_nothing_once_its_slot_is_reused` |
| budget for declared payload sizes | `a_declared_payload_size_is_charged_and_credited` |
| re-entry from host code | `reentry_from_a_host_function_is_bounded_and_unwinds` |
| plain host functions | `a_plain_host_function_runs_with_its_data_and_its_host`, `a_host_function_needs_a_function_entry_of_its_host`, `a_plain_host_function_makes_host_functions` |
| the stack API and `enter` | `a_frame_builds_tables_and_reads_them_back`, `a_buffer_is_written_and_read_in_place`, `a_call_from_a_frame_runs_and_its_error_unwinds`, `sandboxed_globals_refuse_a_raw_store` |
| prototypes by index | `a_collected_prototype_is_dropped_at_once_when_no_loop_holds_the_table`, `a_held_table_keeps_a_dead_prototype_until_the_next_load_reaps_it` |
| the layer boundary | `every_core_module_is_checked`, `the_core_never_names_the_embedding_layer`, `the_check_sees_a_mention` |
| `Core` and the audio VM with its state are `Send` | compile-time: `vm.rs` (`Core`), `safe.rs` (`SafeProcessor`, which holds `HostedVm<AudioHost>`) |
| engine behavior | the engine's 15 tests, both settings, incl. `authored_pcm_and_shared_controls_survive_callback_thread_migration` |
| review findings (§11), one test each | `a_steady_collection_cycle_keeps_the_collectors_storage`, `a_host_function_id_of_another_vm_is_refused`, `a_custom_host_drains_its_freed_natives`, `dropped_handles_free_their_roots_before_the_next_handle`, `a_callbacks_collect_runs_destructors_before_it_returns`, `a_callbacks_finishing_gc_step_runs_destructors_before_it_returns`, `set_global_collects_what_a_dropped_handle_let_go`, `an_enter_that_only_pushes_is_collected_when_it_returns`, `host_functions_made_and_released_are_collected` |
| no `Rc`-recursion with a host callee; frameless natives seen as frames | vm-diff `format_tostring.luau` (`%*` with Lua `__tostring`: nested, errors, traceback and `debug.info` inside, yield across; a host `xpcall` handler); every builtin call in the differential corpus runs frameless |
| the lane 6 API and facade unchanged | the VM's `embed/tests.rs`; the conformance runner's host helpers; `nuxie-scripting`'s 40 targets under `nuxie-vm` |

## 5. Gates

On the rebased tip (code `86a1527fdf`, the docs commit on top; only this
report's last edits uncommitted), sequential, one Bazel server, 1-minute
load beside each (a shared 18-core laptop with other sessions building;
nothing here is timed against a ceiling):

| Gate | Result | Load |
|---|---|---:|
| `make luau-vm-diff` | 178 runs, **0 disagreements**, bytecode versions 7 and 11; the corpus reaches every admitted opcode | 21.4 |
| `make luau-conformance-nuxie` | PASS: 36 required OK, 0 required failing, 4 expected-fail, 15 non-goal | 21.6 |
| `make luau-vm-ratchets` | PASS | 36.0 |
| `make luau-conformance` | PASS: 30 required OK, 0 required failing | 35.9 |
| `python3 tools/bazel/generate.py --check` | OK | 43.0 |
| clippy `nuxie-luau-vm`, lib and `--tests` | clean | 42.8, 45.2 |
| clippy `luau-bench` | clean | 47.3 |
| clippy `nuxie-scripting`, both settings | red, as on `main`: 74 errors (default) and 62 (`nuxie-vm`), none in this lane's code. The ones in the engine's files are `main`'s engine code, unchanged in `raw.rs` (`unsafe` calls without blocks, `div_ceil`, nine arguments), and the `SharedBlob`/`SharedRegion` import that only `main`'s tests use. The crate is outside the lint gate | 52.5, 47.2 |
| tests `nuxie-luau-vm` | 66 passed | 37.4 |
| tests `luau-bench` | 4 passed | 37.4 |
| tests `nuxie-scripting --features nuxie-vm` | **40 / 40** | 35.2 |
| tests `nuxie-scripting` (default) | **40 / 40** | 21.2 |

No `unsafe` in `nuxie-luau-vm` (`#![forbid(unsafe_code)]`) or in the
engine's `safe.rs`; the panic lints stay denied, tests included.

## 6. Predictions and measurements

All numbers are millions of retired instructions per timed run
(`LUAU_BENCH_CLOCK=instructions`, Bazel fat-LTO release, `cgu1`), which do
not move with host load; load is recorded anyway. **Before** is
`make luau-vm-perf-report` at `ca0358e0a2` (the pre-rebase commit with the
audio-block benches and the amended design: the base's VM crate,
unchanged), 3 passes, 2026-11-11 18:04 UTC. **After** is the same target
at `917edcb359` (pre-rebase SHA of `6986353899`), 3 passes, 1-minute load
30 to 38. The commits after it (`63f801024a`, `ac0c7b9ee2`,
`86a1527fdf`) touch only host-boundary safepoints, which no bench's
measured region (Lua code between two `os.clock` reads) reaches. luaur
counts are the perf map's table 2.1
(`docs/evidence/luau-safe-vm-perf-map-2026-11-11.md`; the vendored luaur
is unchanged by this lane) and, for the audio-block benches, §7.

### 6.1 The bench suite

| Prediction (design §6) | Measured | Verdict |
|---|---|---|
| `AudioBlock_*`: 1,100 to 1,400 instructions a frame, 2 to 2.6x luaur | 79.57 M and 80.54 M (−25.5%, −25.1%): 2,428 a frame, 4.58x and 4.42x luaur | **fails** (§7) |
| The 21 builtin-call benches fall the most, toward 1.5 to 2x luaur | they fall the most: group 5.28x → 4.33x luaur (−18%); the scalar-builtin loops −27% to −33% (`AbsSum_*`, `SqrtSum_*`, `Assert`, `vector_lib`); `TableFind_table_find` +10% (`table.find` is resumable, it can call `__eq`) | direction right, size **fails** (§7) |
| `string.format`, `tostring`, `print` do not move | `%s format` +11.5%, `ToNumberString/tostring` +5.8%; `%d format` −7.4%, `%* format` −6.9%, interp `tostring` −6.6% | **fails**: they move 6% to 12% both ways |
| Call-heavy benches 2% to 5% fewer instructions | `Factorial` +9.9%, `OOP_method_call` +7.5%, `MethodCalls` +5.7%, `Pcall_call_return` +11.4%; `UpvalueCapture` −13.7% | **fails** (§6.3) |
| Everything else within the run spread | `sieve` +22.1%, `Pcall_pcall_yield` +15.0%, `Pcall_pcall_return` and `xpcall_return` +11.6%, `Failure_*` +6.5% to +7.9%; `chess` −10.3%, `TableCreate_*` −8% to −10%, `string_lib` rep/reverse −10% to −12% | **fails** (§6.3) |

The suite: geomean after/before **0.947** over 89 benches (48 fewer
instructions, 30 more, 11 within the noise); against luaur, 4.54x → 4.33x
over the perf map's 87.

| Group (perf map §3.1) | Benches | Before / luaur | After / luaur | After / before |
|---|---:|---:|---:|---:|
| builtin calls (FASTCALL-able) | 21 | 5.28x | 4.33x | 0.820 |
| string building, interning, number formatting | 18 | 5.51x | 5.26x | 0.955 |
| table library bulk ops | 10 | 10.13x | 9.95x | 0.982 |
| array access and iteration | 14 | 3.03x | 3.04x | 1.003 |
| Lua calls and returns | 8 | 3.59x | 3.87x | 1.077 |
| table get/set by constant key | 7 | 3.63x | 3.51x | 0.966 |
| coroutines | 2 | 3.68x | 4.35x | 1.185 |
| error path (pcall/xpcall of a failing call) | 4 | 1.90x | 2.04x | 1.073 |
| arithmetic and loops | 2 | 2.32x | 2.37x | 1.020 |
| closures and upvalues | 1 | 3.37x | 2.91x | 0.863 |

### 6.2 Host calls

Not in the suite, so measured on their own: a loop of 100,000 calls of a
one-argument function returning one number (`s += f(1)`), minus the same
loop without the call (`s += 1`), per call. The function reads a word, as
the audio-block bench's `__bench_atomic_load` does. Plain and core-full
forms exist only after the lane; they were registered by a
measurement-only patch to `tools/luau-bench/src/vm.rs` in a scratch
worktree (not committed). Before is `ca0358e0a2`, built the same way.

| Kind | Before | After | luaur |
|---|---:|---:|---:|
| lane 6 API (`Vm::register`; after: through `Compat`) | 936 | 1,529 | |
| core full (`HostEntry::Full`, `HostedCtx` stack) | | 1,176 | |
| core plain (`HostEntry::Plain`, frameless) | | 539 | |
| luaur-rt `create_function` | | | 1,527 |
| the loop without a call (per iteration) | 163 | 167 | 68 |

Against the predictions:

- **Plain host calls, "one or two hundred": fails** at 539. The same
  error as §7: the frameless call still costs about 430, and the host
  dispatch (entry lookup with its VM check, `PlainCtx`, checked stack
  reads and writes) adds about 100.
- **Full calls, "about flat": fails.** A core full call costs 1,176, more
  than the whole lane 6 call before (936). Full minus plain is 637: the
  frame protocol (about 480, §7) that full calls keep, and about 160 for
  leaving the interpreter loop for the driver and re-entering it. The
  design called the driver exit "slightly dearer".
- **The facade's host calls, "about flat": fails** at +63% (936 → 1,529).
  `Compat` adds 353 to a core full call: two `drain_freed` passes, an
  `Rc` clone of the closure, a `dyn` call, and a result that crosses three
  value representations (`HostValue` → facade value → core value; §8).
  Every product binding is a lane 6 function, so this is the product's
  host-call cost until the layers collapse. Product frames themselves were
  not measured.
- **Audio, vendored to safe engine, "2 to 2.6x": fails** with the
  audio-block result (4.4x to 4.6x luaur).

### 6.3 What made Lua calls, coroutines and pcall dearer

A scan of the lane's code commits on the regressed benches (one pass per
commit; at these sizes the spread is under 1.2%), millions of
instructions:

| Bench | before | `8ab45193d2` strings | `9cfadfb87e` frame stack | `ff95eed204` builtin kinds | `f932d31c10` driver | `6e97c2583b` protos by index | `0d572d3a80` payloads | `5c6a0bb07b` interrupt | `fd3a516d89` roots | `66abf721ed` layer | `d5832db6e3` frameless | tip |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| `sieve` | 5,263 | 5,184 | **5,974** | 5,983 | **6,299** | 6,349 | 6,305 | 6,364 | 6,354 | 6,354 | 6,428 | 6,425 |
| `Pcall_pcall_yield` | 1,090 | 1,081 | **1,210** | 1,213 | 1,266 | 1,283 | 1,267 | 1,274 | 1,272 | 1,289 | 1,250 | 1,253 |
| `Factorial` | 2,610 | 2,611 | **2,706** | 2,708 | **2,785** | **2,852** | 2,804 | 2,822 | 2,819 | 2,818 | 2,874 | 2,869 |
| `MethodCalls` | 2,423 | 2,427 | 2,464 | 2,461 | 2,509 | 2,540 | 2,509 | 2,521 | 2,515 | 2,516 | 2,566 | 2,561 |
| `Pcall_call_return` | 88.1 | 88.1 | 92.2 | 92.0 | 95.3 | 97.4 | 95.4 | 96.1 | 95.9 | 96.1 | 98.4 | 98.1 |
| `Pcall_pcall_return` | 224.0 | 221.7 | 232.3 | 240.1 | 244.7 | 255.5 | 245.5 | 247.5 | 247.1 | 247.4 | 250.1 | 250.1 |
| `TableFind_table_find` | 211.0 | 209.1 | 214.8 | **225.6** | 228.7 | 230.7 | 229.4 | 231.1 | 231.0 | 230.9 | 233.0 | 232.8 |
| `AbsSum_math_abs` (control) | 1,008 | 988 | 1,045 | 1,048 | 1,070 | 1,100 | 1,083 | 1,101 | 1,099 | 1,099 | **738** | 733 |

Three design choices carry it, in order:

1. **The reserved frame of a frameless native** (`9cfadfb87e`): `sieve`
   +15%, `pcall_yield` +12%, `Factorial` +3.6%, before any native ran
   frameless. `Frames` keeps a whole `Frame` (with its `Wait`
   continuation state) inline in every thread's state, and every view of
   the frame stack (`last`, `last_mut`, `get`, `len`, `push`, `pop`)
   branches on whether that frame is active. Lua calls pay the branches on
   every frame access; a coroutine switch swaps `Frames` twice
   (`swap_exec` on both threads), copying the inline frame each time. The
   design put the cost of "an error names the builtin" on every frame
   access in the VM, to save it on a path that errors rarely.
2. **Host calls at the driver** (`f932d31c10`): `sieve` +5.3%,
   `Factorial` +2.8%, on benches that make no host call. The interpreter
   loop now takes the host as `&mut dyn Dispatch` and returns a wider exit
   type to a driver loop; Lua calls, returns and thread switches pay for
   the possibility. Not attributed below the commit.
3. **Prototypes by index** (`6e97c2583b`): `Factorial` +2.4%,
   `pcall_return` +4.4%, against a predicted 2% to 5% saving. The
   prediction counted the `Rc` increment and decrement it removes from
   each frame switch and not the indexed lookup (table pointer, bounds
   check, index) it adds there. The payloads commit after it gives about
   half back (`Factorial` −1.7%), probably from code layout; not
   attributed.

The frameless `CALL` (`d5832db6e3`) then costs Lua calls about 2% (the
`CALL` handler tests for a plain native first) and takes a third off the
builtin loops. `table.find` lost 5% at `ff95eed204` because it is
resumable by type (it can call `__eq`), so it keeps the full native
protocol.

### 6.4 The design, revisited

Per the brief, none of this was tuned; the revisit is for the next lane
(the collapse of the two layers, §8), together with §7's:

- **No reserved frame.** The frameless native is not a frame at all: no
  slot in `Frames`, no branch in its views, nothing extra to swap on a
  thread switch. Error positions, `debug.info` and argument-error names
  get the builtin from the `CALL` instruction at the saved pc, built on
  the error path. This is §7's "no frame record on the hot path", and the
  scan shows it is also the largest cost on paths that never call a
  builtin.
- **The host statically known to the interpreter.** `run_frames` over a
  generic host instead of `&mut dyn Dispatch`, so a VM with a plain-only
  host has no exit path to pay for. Whether that is worth a generic core
  (code size, compile time) is the next lane's call; the measured price
  of the dynamic shape is 3% to 5% on calls and thread switches.
- **Protos by index stay** (they are what makes `Core` `Send`); the next
  call-path design is measured with them in, and the +2.4% is booked as
  their cost, not as a saving.
- **Product host functions plain where they can be.** Most bindings
  neither call Lua nor yield; as plain entries they would cost about 540
  instead of 1,529 today, and less once plain calls get §7's
  fast-body treatment. The collapse removes `Compat`'s 353 and the
  third value representation.

## 7. The audio-block prediction failed: the design revisit

Measured on the instruction clock (`LUAU_BENCH_CLOCK=instructions`,
fat-LTO release; instruction counts do not move with host load, which was
about 7 for this run):

| Bench | before (lane 6 shape) | after the frameless path | luaur | after / luaur |
|---|---:|---:|---:|---:|
| `AudioBlock_sine` | 107.09 M | 79.90 M | 17.38 M | 4.6x |
| `AudioBlock_shared_controls` | 107.76 M | 80.86 M | 18.23 M | 4.4x |

Per frame, 2,438 instructions against a predicted 1,100 to 1,400. Per
loop item, from one-line variants of the block loop (32,768 iterations
each, both VMs, same harness):

| Variant | safe VM | luaur | note |
|---|---:|---:|---|
| empty numeric `for` | 236 | 107 | the loop and its interrupt poll |
| `GETIMPORT math.sin`, no call | 87 | 24 | |
| `math.abs(x)` | 574 | 68 | luaur: `FASTCALL`, no call |
| `math.sin(x)` | 627 | 131 | luaur: `FASTCALL` |
| `buffer.writef32(b, o, x)` | 747 | 119 | luaur: `FASTCALL` |
| local Lua function call | 866 | 169 | a full Lua frame on both |
| `type(x)` | 621 | about 35 | luaur: `FASTCALL` |
| `select("#")` | 430 | 185 | a real C call on luaur too |

**Why the prediction was wrong.** Two errors, both in the design, not in
the build:

1. The design priced the frame protocol at about 900 instructions a call
   and assumed the frameless path would leave 150 to 250. The protocol was
   about 480 (the same builtin, framed against frameless), and the
   frameless path itself costs about 430: the argument window
   (`ensure_stack`), the reserved native frame written on every call so
   that an error names the builtin, bounds-checked reads and writes of
   every argument and result, `check_number`, result pushes that recheck
   the stack, `move_results` and `fill_nil`, the request backstop, and a
   safepoint after every call. Each is 10 to 60 instructions.
2. The larger miss: luaur does not call `math.sin`, `buffer.writef32` or
   `type` in this loop at all. The compiler emits `FASTCALL` before these
   calls, and luaur runs the builtin's body inline over registers. The
   design assumed a lighter call would close the gap `FASTCALL` closes. It
   cannot: a call with an argument window, however light, is a call.

**What the design should be instead** (for the next lane; not done here,
and not tuning of this path):

- **Fast bodies over registers for every `Plain` builtin the compiler
  emits `FASTCALL` for** (math, bit32, buffer, vector, `type`, `typeof`,
  ...). The bytecode already carries the fast path and its fallback; the
  safe VM has fast bodies for four builtins. A fast body takes register
  values and writes the result register: no window, no frame record, no
  safepoint unless it allocated. This is the representation the bytecode
  asks for, and it removes the call where AudioBlock spends its time.
- **No frame record on the hot path.** The reserved frame exists so an
  error names the builtin; the error path can build it from the `CALL`
  instruction's pc (the callee is in its register).
- **Results written in place** for fixed result counts, with no push loop
  and no `move_results`.
- **A safepoint only when the call allocated** (the heap's allocation
  counter moved).

The empty-loop cost (236 against 107) is the interpreter's per-instruction
cost, outside this lane (the perf map's section 4).

## 8. Two stacked compatibility layers: what this lane learned

The product now runs `nuxie-scripting` bindings → the lane 7 facade
(`luaur-rt`-shaped handles) → `Compat` (lane 6's API) → the core. Both
layers passed their suites unchanged at every commit, so the stacking is
functionally sound. What it costs, as the lane found it:

- **Every product host call is a full host call.** `Compat`'s closures may
  call Lua, so its three entries are `Full`: a frame, an exit to the
  driver, a slot lookup, then the facade's per-call `Vec`s, handles and
  `lend` swaps. None of the frameless work reaches the product. Only a
  binding written against `HostEntry::Plain` gets it, as the audio
  engine's do.
- **Each layer reproduces the timing of the one above.** The facade
  expects a collected closure's captured state and a userdata's `Drop` to
  be gone after a collection and before the next host call, as `luaur-rt`
  does. `Compat` meets that at 24 call sites: 18 `drain_freed` (start
  and end of each trampoline call, before it allocates, after every
  protected operation, around a host-requested collection) and 6
  `host_safepoint`, the layer's only safepoint, which drains on both sides.
  Handles dropped by the facade are queued and released at the same
  points and before each new handle. The list grew twice during the lane
  (commits `f932d31c10`, `fd3a516d89`) as tests found a timing the layer
  above relied on, and review found four more sites it had missed (rounds
  2 and 3, §11).
- **A value crosses three representations.** A facade handle wraps a lane
  6 handle, which owns a core `Root`, which indexes the root table. An
  error value is a facade error, a lane 6 `HostError` holding a handle
  until it is raised, and a core `HostError` holding a value or a `Root`;
  the kept error object `take_error_object` reads lives in `Compat`.
  Release is O(1) now (no `Weak` scan), but each level allocates its own
  bookkeeping.
- **The layer's names constrain the core.** `Vm` is an alias, so inherent
  methods on the generic `HostedVm<H>`/`HostedCtx<H>` collide with the
  layer's inherent methods of the same name: a generic `new_host_native`
  met `Compat`'s (E0592, and E0034 at a call that reached both), so the
  core's became `new_host_function`, and every later core method had to
  avoid the layer's names. Methods moved from the layer to `Core` are
  reached through `DerefMut`, which loses two-phase borrows:
  `vm.set_memory_limit(Some(vm.used_memory() + n))` stops compiling, so
  the layer keeps forwarding methods.
- **The audio engine is the control.** Written against the core with the
  stack API and roots, it needed neither layer: no handles, no `Rc`, no
  drain discipline, frameless host calls. It is 1,035 lines against the
  raw processor's 982; the extra length is explicit error paths where the
  raw one had `unreachable!`, a small trait shared by the two context
  kinds, and the entry table, not anything a layer would have saved.

Collapsing the layers is the next lane (design section 8): bindings on
plain host functions and roots where they are hot, and one handle
representation where they are not.

## 9. Deviations from the design

| Design | Built | Why |
|---|---|---|
| §7 order: protos by index first, the split fourth | frame type and plain/resumable split first (behavior-neutral), then the split, then protos | the split decides where loads run (the driver), which protos by index relies on (no frame loop holds the table during a load) |
| plain builtins take `Native<'_>` (core plus interrupt hook) | plain builtins take `&mut Core`; the matcher's builtins are a fifth kind, `Interruptible`, taking `&mut dyn Dispatch` | only the pattern matcher needed the host; a separate kind keeps every other plain builtin free of it |
| `Plain<'_, H>`, `HostCtx<'_, H>` | `PlainCtx<'_, H>`, `HostedCtx<'_, H>` | the layer exports `HostCtx<'a>` (its old name) as `HostedCtx<'a, Compat>` |
| `Dispatch` with the split | introduced with payloads (`0d572d3a80`) | the split's driver handled full host calls by exiting the loop; the loop needed the host only once direct fields became a host hook |
| boundary test and `embed/` move with the split | in their own commit (`66abf721ed`) | the split commit was large enough; the move is mechanical |
| proto table borrowed while the loop mutates the rest of `Core` | `Arc<ProtoTable>` cloned once per loop entry, `Arc::make_mut` at load | the borrow could not be split from `&mut Core` through every library call; loads run at the driver, so `make_mut` never copies in practice (it would be correct if it did) |
| `error_object` as a core `Option<Root>` | kept by `Compat` as a handle value | only the lane 6 API reads it |
| `HostError` holds a lane 6 handle | holds a core value (no object) or a `Root`; `HostError::from_root` | the core cannot name handles; the layer keeps an object's handle with the error until it is raised |
| drain points: trampoline start and end | also before a collection, at the start of a protected operation, before the settle safepoint | handle drops are queued since roots; a dropped handle's object must stay collectable where it was |
| a frameless native has a virtual record materialized from pc on error | the reserved frame in `Frames` is written on every frameless call (the fields a native frame sets) and pushed for real on error | simpler to keep every frame view correct; it is part of the 430-instruction residual (section 7) |
| no stack API | `Stack`, `PlainCtx::stack`, `HostedCtx::{call_top, pcall_top, push_tostring, push_field, push_loaded}`, `HostedVm::enter` | a host other than `Compat` had no way to reach values (the core's `Value` is crate-private); the audio engine needed one |
| layer memory methods move to `Core` | moved, with forwarding methods kept on the layer | two-phase borrows through `DerefMut` (section 8) |
| `sandbox` | `Core::sandbox_globals` (C++ `luaL_sandbox`); the layer's `sandbox` adds `luaL_sandboxthread` | the audio engine uses `luaL_sandbox` only, as its raw processor does |
| the safe processor shorter than the raw one | 1,035 lines against 982 | section 8, last item |

## 10. Builds and tests per setting

| Setting | Build | Tests |
|---|---|---|
| `nuxie-luau-vm`, native (Bazel) | ok | 66 unit tests |
| `nuxie-scripting` default (vendored VM), native (Bazel) | ok | 40 / 40 targets, the engine's 15 tests among them on `raw.rs` |
| `nuxie-scripting --features nuxie-vm`, native (Bazel) | ok | 40 / 40 targets, the engine's 15 on `safe.rs` |
| `nuxie-luau-vm`, `wasm32-unknown-unknown` (cargo, stable `rustc`) | `cargo build`: ok | not run |
| `nuxie-scripting` default, `wasm32-unknown-unknown` | `cargo check`: ok | not run |
| `nuxie-scripting --no-default-features --features nuxie-vm`, `wasm32-unknown-unknown` | `cargo check`: ok | not run |

Under `nuxie-vm` the audio engine names no `luaur-vm` item (raw
references 6 to 0, seam rows R33–R36); the default build is unchanged.

## 11. Review

`codex review --base origin/main`, rounds until clean or four.

**Round 1** (3 findings, on the rebased tip with the docs uncommitted):

| Finding | Verdict | Fix |
|---|---|---|
| P2 the audio block's `collect_step(0)` allocates: `gc_roots` builds a fresh `Vec` at the steps that need roots | real; and not only the roots: the gray lists were exchanged by `mem::take` (one list's storage dropped per cycle) and the atomic step consumed the weak list | `9d78a02b50`: the root set is kept in `Core` between steps, the gray lists swap, the weak list keeps its storage. Measured with a counting-allocator probe outside the crate (it forbids the `unsafe` one needs), 1000 warmed steps: `enter` + `collect_step(0)`, 800 allocator calls before, 0 after; an audio-like block (a Lua call writing 1024 stereo frames with `math.sin` and `buffer.writef32`) + `collect_step(0)`, 728 before, 0 after. Test `a_steady_collection_cycle_keeps_the_collectors_storage` |
| P2 a `HostFnId` of another VM with an entry at the same index is accepted | real: the id was an index only, while the doc said a foreign entry is refused | `b62da9522d`: the id carries its VM's id; `host_function`, `push_host_function` and a full entry's continuation check it. Test `a_host_function_id_of_another_vm_is_refused` |
| P2 a host of its own cannot take the freed natives of its `owns_data` entries (only `Compat` could), so their state leaks and the list grows | real | `af120563b6`: `HostedVm::drain_freed_natives`, with the host's state, keeping the list's storage. Test `a_custom_host_drains_its_freed_natives` |

The clippy sites the gate run found in this lane's code (one test
indexing site in the VM crate; in the engine, `raw.rs`'s `shared()` dead
in a native default build, and two in `safe.rs`) are `8a9a03bec3`.

**Round 2** (2 findings, both in the lane-6 layer, `Compat`):

| Finding | Verdict | Fix |
|---|---|---|
| P2 a dropped handle's root stays until a drain point: a handle-making call outside one (`globals`, `registry_value`) grows the root table with every handle made and dropped (100,000 `drop(vm.globals()?)` kept about 6 MiB) | real; luaur-rt has the same shape (its handle drops queue until the next `lua_` entry that drains), so a layer reproducing luaur-rt timing inherits it | `30d9005cd2`: `new_handle` releases the queued roots before inserting. Test `dropped_handles_free_their_roots_before_the_next_handle` (fails without the fix) |
| P2 `HostCtx::collect` and `gc_step` drained before collecting only, so a destructor of what the callback's collection freed waited for the next drain point | real: `Vm::collect` drains after too, the callback forms did not | `6986353899`: both drain after as well. Tests `a_callbacks_collect_runs_destructors_before_it_returns`, `a_callbacks_finishing_gc_step_runs_destructors_before_it_returns` (each fails without its half of the fix) |

Both are the drain-point timing §8 names: the layer has sites where it
must remember to drain, and the review found two it had missed.
The core has none (a host drains its own freed natives and payloads when
it chooses), so collapsing the layers removes this class rather than
these two cases.

**Round 3** (2 findings, one in each layer):

| Finding | Verdict | Fix |
|---|---|---|
| P2 `set_global` runs a bare core safepoint: a userdata whose last handle was dropped stays rooted through every cycle it starts (256 KiB VM, dropped 16 KiB userdata, 1000 string updates: no destructor; the base ran it once) | real; the other layer sites drained after the safepoint only, `settle` before and after | `63f801024a`: one `host_safepoint` (release dropped handles, safepoint, release freed payloads) is every safepoint of the layer, `settle` included. Test `set_global_collects_what_a_dropped_handle_let_go` (fails without the fix) |
| P2 `HostedVm::enter` reaches no safepoint when the host only pushes: 1000 enters leaving a 128-slot table each exhaust a 256 KiB budget | real: the interpreter's safepoints run only in calls | `ac0c7b9ee2`: a safepoint when `enter` returns, after the caller's top is restored (nothing the frame pushed is live then unless rooted). Test `an_enter_that_only_pushes_is_collected_when_it_returns` (fails without the fix) |

The second fix collects at the end of an `enter`, not at each allocating
stack operation as C++'s `lua_createtable` and friends do
(`luaC_checkGC`): a single `enter` that makes garbage in a loop without a
call collects only when it returns. A per-operation safepoint would also
run inside frameless plain natives, whose argument window would have to
be proven rooted at that point first; that is the next lane's call, with
the collapse. Neither fix touches a bench's measured region (the benches
count instructions inside Lua, between two `os.clock` reads), so the
perf report run on `6986353899` (§6) stands for the tip.

**Round 4** (1 finding; the cap, so no fifth round):

| Finding | Verdict | Fix |
|---|---|---|
| P2 `HostedVm::host_function` allocates and returns a root without reaching a safepoint: making and releasing functions in a loop exhausts 256 KiB after about 4,900 | real, and the same class as round 3's `enter` | `86a1527fdf`: a safepoint once the new function is rooted. Test `host_functions_made_and_released_are_collected` (fails without the fix) |

The other `HostedVm` entry points allocate nothing on the heap
(`add_host_fn`, `drain_freed_natives`) or run inside `enter` or a host
call, which both end at a safepoint now.

**What the four rounds found, as a class.** Eight findings, all real.
The five of rounds 2 to 4 are one shape: a boundary where the host takes control back
(a handle made or dropped, a collection a callback asked for, a host
operation that allocates, the end of an `enter`) without the collector or
the drains running there. The interpreter has its safepoints at
allocating instructions and after calls; the host side has no such rule,
so each entry point needed its own. The next lane's API should make this
structural (one host-boundary type whose drop runs the safepoint and the
drains) rather than a list each new entry point must join.

## 12. Not verified

- **Engine-level speed.** The audio-block bench measures the VM in the
  bench harness, not the engine's `process_block` on either VM; the
  engine's real-time margin on the safe VM was not measured.
- **Product scripted frame cost** on the facade: not measured as frames;
  only its per-host-call component was (§6.2).
- **luaur suite counts** come from the perf map (2026-11-11), not a run
  in this lane; the vendored luaur did not change.
- **The commit scan** ran one pass per commit on eight benches; its
  mechanisms for the driver split and the payloads commit are inferred,
  not profiled.
- **Platform hosts.** wasm32 was built with cargo and the stable `rustc`
  (Bazel has no wasm target for these crates); nothing was run in a
  browser worklet.
