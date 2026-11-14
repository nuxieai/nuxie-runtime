# Luau client path, lane H4: the abort gate and the hostile corpus (2026-11-14)

Design: `docs/luaur-client-path-safe.md` section 8 (lane F there). Branch
`claude/luaur-lane-h4-abort-gate-5744ba`, based on `levi/luaur-client-stage1f`
(`b495c08c29`). Tests, tooling and build configuration only: no product
source changed.

## How to run it

```
export RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3
NUXIE_EXECUTION=local make luau-abort-gate
```

`tools/luau-abort-gate/gate.py` makes three runs, each to its end whatever
the others did, and exits 1 if any failed:

1. `NUX_RUNTIME_PANIC=abort runtime.py test -p nuxie-luau-vm -p nuxie-scripting`
   with `--exact --skip hostile_corpus --skip <each excluded test>`;
2. the corpus under abort: `NUX_RUNTIME_PANIC=abort runtime.py test -p nuxie-scripting --test hostile_corpus -- --nocapture`;
3. the corpus under unwind: the same without `NUX_RUNTIME_PANIC`.

Subsets: `NUXIE_HOSTILE_FILTER=<part of a case path>`. Debugging one case:
`NUXIE_HOSTILE_VERBOSE=1` prints each child's whole output, and the built
test binary runs a single case directly with
`NUXIE_HOSTILE_CASE=<dir/name.luau> <binary> hostile_corpus --exact --nocapture`
from the worktree root.

## The abort configuration as built

- `.bazelrc` `build:panic-abort`: `extra_rustc_flag=-Cpanic=abort`,
  `extra_rustc_flag=-Zpanic-abort-tests`, `--action_env=RUSTC_BOOTSTRAP=1`.
  Target configuration only; proc macros and build scripts (exec
  configuration) keep unwind. The later `-Cpanic=abort` overrides the
  `-Cpanic=unwind` the base config passes.
- Why the two extra flags: stable rustc refuses `--test` with
  `panic=abort` ("building tests with panic=abort is not supported without
  `-Zpanic_abort_tests`"). With it, libtest runs every test in its own child
  process and reports an abort as that test's failure; the run goes on. The
  pinned toolchain (1.94.1) is stable, hence `RUSTC_BOOTSTRAP=1` for these
  actions only.
- `tools/bazel/runtime.py`: `NUX_RUNTIME_PANIC=abort` adds
  `--config=panic-abort` (`unwind` or unset: nothing; anything else is an
  error). The variable is kept out of the forwarded `--action_env` set, so it
  does not change unwind cache keys. Test:
  `test_panic_abort_selects_the_abort_config_without_widening_action_env`
  in `tools/bazel/test_runtime.py` (31 tests pass).
- Checked that it is real: the abort unit-test binary of `nuxie-luau-vm`
  links `library/panic_abort` and has no `panic_unwind`, and carries
  libtest's `__RUST_TEST_INVOKE` child-process path.

### What it costs

Measured on this Mac with three other lanes building (load average 8 to 22
throughout; every time below is inflated by that load):

| Step | Wall time |
| --- | ---: |
| First abort build and run of the `nuxie-scripting` suites (842 actions, 295 from the disk cache) | 241 s |
| `make luau-abort-gate`, whole (abort suites 287 s, abort corpus 79 s, unwind corpus 197 s, each including the switch of configuration) | 9 min 23 s |
| The corpus alone, once built (184 cases, 4 at a time) | 31 to 110 s |

Disk: the brief expected a second set of outputs. Bazel does not make one:
the abort flags are command-line build settings, so both strategies write the
same `bazel-out/darwin_arm64-fastbuild` tree, and switching strategy re-runs or
re-fetches every target action. This worktree's output base was 9.1 GB at the
end (both strategies have been built in it; only one is on disk at a time).
The second set lives in the shared disk cache
(`~/.cache/nuxie/bazel/actions`, 2.1 GB when measured, under its 16 GB GC
cap). The cost of the duplication is therefore time on every switch, not a
standing second tree. Free disk on the shared volume went from 162 GB to 63 GB
during the session, mostly other lanes' builds; I did not see a step of my own
larger than the output base above.

## Tests left out of the abort run

`tools/luau-abort-gate/excluded-tests.txt` (package, full name, reason; the
gate fails on a line whose test no longer exists):

| Test | Why it cannot run under abort |
| --- | --- |
| `nuxie-scripting` `luau::facade::tests::scoped_callbacks_are_revoked_when_the_scope_unwinds` | `catch_unwind` around a panicking `Lua::scope` body; it asserts what the unwind does |
| `nuxie-scripting` `luau::facade::tests::a_panicking_vm_operation_puts_the_vm_back` | `catch_unwind` around a panicking `with_vm`, plus a host callback panic the trampoline's `catch_unwind` (`facade/state.rs`) turns into an error |

There is no `#[should_panic]` test in either crate. (Under
`-Zpanic-abort-tests` they would work anyway: libtest checks the child's
panic. A scratch test showed `#[should_panic(expected = ...)]` passing and a
`catch_unwind` test failing.) The three `catch_unwind` sites in shipped code
(`facade/state.rs`, `luau_audio_processor_engine.rs`,
`luau_audio_processor_engine/safe.rs`) are not tests and stay.

## Which currently passing tests abort

None, outside the two excluded above. In the gate run, under abort: 41 test
targets, 903 tests pass, 0 fail, 5 ignored (as in the unwind suites).

The first abort run (before the exclusion list) failed three tests: the two
listed above, and
`vm::luau_audio_processor_engine::tests::processor_stream_readers_are_released_when_collected`
("audio processor execution budget exceeded", left 3, right 1). That one is
not abort-related: its blocks get a 42 ms wall-clock deadline, and the machine
was at load 16 to 18. It passed alone under abort and in the full gate run.
It is a load flake that can redden the gate on a busy host; it is not
excluded.

## The corpus

`crates/nuxie-scripting/tests/hostile/`, 184 cases. Each starts with one
header line:

```
--! attacks: <what>; expect: ok|error|error(<text>)|either; within: <N>s[; harness: module|instance|bytecode][; viewmodels: <file.riv>][; bytecode: <directive>]
```

- `module` (default): `ScriptVm::run_bytecode`, the product's module path
  (a module must return a table or a function).
- `instance`: the chunk returns a generator; the runner drives its table as a
  scripted object: `init`, then twice `draw` (a fresh renderer each time),
  `advance(1/60)`, `update`. This is how a script gets a `Renderer`.
- `bytecode`: the chunk is compiled, then a directive makes variants
  (`truncate-all`, `every-byte <v>`, `every-byte-flipped`, `every-version`,
  `every-varint-max`, `append <hex>`, `hex <literal>`, `payload-hex`,
  `payload-prefix`); each variant loads and runs in its own fresh VM, after a
  control (the unchanged chunk) that must run.
- `viewmodels: <file.riv>` loads only the view-model definitions of a
  rive-runtime unit-test asset, so `Data.<Model>.new()` exists. The file's
  own scripts do not run.

Only `either` accepts both outcomes; 1 case uses it
(`loops/bounded_string_workloads`). Most cases run each attack under `pcall`
and assert the specific error or result, then return normally, so a typo or
an API that silently does nothing fails the case.

| Class (directory) | Cases | Covers |
| --- | ---: | --- |
| `recursion/` | 20 | Luau recursion (plain, tail, mutual), metamethod chains (`__index`, `__newindex`, `__call`, `__eq`/`__lt`, `__concat`, `__len`, `__tostring`), host callback chains (`pcall` nesting, sort comparator into sort, gsub into gsub, print into `__tostring` into print), `xpcall` handlers (erroring, recursing, nested, kept past their call), the E1 4.1 error chain, coroutine nesting, deep tables, `require` of odd names, pattern capture nesting and catastrophic backtracking |
| `loops/` | 14 | `while true`, `repeat until false`, `for` to `math.huge`, inside `pcall`, retry loops, `xpcall` handlers, `__index`, print's `__tostring`, sort comparators, gsub callbacks, coroutines, resume/yield ping-pong, wrap generators, long bounded string work |
| `numeric/` | 15 | VM library extremes: `string.rep`, `table.create`, `buffer.create`, buffer offsets and widths at -1, 2^31, 2^32-1, NaN, inf; copy/fill overflow, bit ops, `string.format` widths, forged `string.pack` counts, `sub`/`byte`/`char`, `utf8`, `bit32` shifts, math NaN/inf, `table.insert`/`move`/`concat`/`unpack`/`select`, integer edges |
| `bytecode/` | 15 | malformed bytecode: every prefix, every byte at 0x00/0x80/0xff/flipped, all 256 versions, every varint at `u32::MAX` and unterminated, trailing garbage, literal chunks with forged counts and lengths, the signed-content envelope empty, truncated and unknown (about 5,400 variants in all) |
| `alloc/` | 17 | growth past the 64 MiB host budget through `Path` (`add(self)` doubling, `add(other)`, measures, contours, `extract`, reset after draw), `VertexBuffer`, `MeshInstances`, `Text` (append, layout over budget, many laid-out texts), `Font.decode`, font view-model property, `Font:withOptions`, `Text` keeping a gradient, `DataValue.string`, `print` of many references to one large string |
| `bindings/` | 31 | NaN, inf, negative, huge and wrong-typed arguments to `Path`, path and contour measures, meshes, every `Renderer` call (NaN draws, 20,000 saves, restore without save, stale use, image calls, huge clips), `Text` sizes and indices, text styles, malformed and patched fonts, gradients, paints, `Vector`, `Mat2D`, `Mat4`, `buffer` `stridedcopy`/`convert`, `DataValue`, `print` |
| `audio/` | 11 | `Ring`/`Region`/`Snapshot` sizes, the shared budget against the host budget, ring records, deep and cyclic shared values, snapshot hold chain to the 65,536-object cap, ring hold chains, `Region` exclusive access, `Blob`, `streamReader`, `Audio.start`, `play*` |
| `gpu/` | 6 | `GPUBuffer`, `GPUTexture`, `GPUSampler` sizes and formats, constructors in a loop, `GPUBindGroup` with re-entrant descriptors, `GPUTarget`/pipeline descriptors |
| `viewmodel/` | 22 | list growth, hostile indices, self-containing lists, deep chains; huge string, blob and number values, enums, colors, fonts; listeners that write their own property, add or remove listeners while notified, A→B→A cycles, triggers; stale property handles; nested self-assignment |
| `reentry/` | 20 | `__tostring`/`__index`/`__eq`/`__lt`/`__len`/`__concat` re-entering the host binding that called them (`Path`, `Paint.with`, `Gradient` stops, `Text` styles, buffers), sort comparators mutating their array, gsub into gsub, Promise self-resolution and cycles, `await` and `yield` across host calls, `coroutine.close` of running and normal coroutines, uncaught error objects, a view-model listener rewriting its own property |
| `stale/` | 13 | a renderer used after its draw (stashed, from a coroutine, its methods kept and rebound, loan-forwarding host frames kept), objects mutated mid-draw, VM drop with live handles, text lines and glyphs after edits, iterators after their source changed, `xpcall` handlers and host frames kept past their call, userdata in weak tables, path handles after mutation, Promise callbacks after settling, dead coroutines |

### The runner

`crates/nuxie-scripting/tests/hostile_corpus.rs`, one test, `hostile_corpus`:

- Each case runs in a child process (the test binary re-run with
  `NUXIE_HOSTILE_CASE`), 4 at a time, in both strategies. Under abort,
  libtest adds one more process level; the runner watches and kills the whole
  process tree.
- In the child: `ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default())`
  (64 MiB VM heap, 64 MiB host budget, 50,000 interrupt safepoints per host
  callback), `install_render_factory(NullFactory)`, `install_rive_globals()`,
  on a thread with a 2 MiB stack. After the case: a full collection and the
  VM's drop, both on that thread, so faults there are the case's.
- One limit is not the default: the module's wall-clock deadline is 20 s,
  not 1 s. At 1 s, a loaded host turned a dozen budget cases into
  "execution exceeded 1.0 second timeout" in one run and not in the next. The
  deterministic bounds stay default; the case's `within` bounds wall time.
- The parent fails a case on: a non-zero exit, a signal, or a missing
  "dropped" line (`crash`); a host panic, whether it reached the outcome as
  `rust panic` or a script's `pcall` swallowed it (`panic`: the panic hook's
  "panicked at" is in the child's output); wall time past `within`
  (`timeout`); resident memory of the child's process tree past 512 MiB,
  sampled every 50 ms with `ps` (`memory`); or a clean exit with the wrong
  outcome (`outcome`).
- `expected-failures.txt`: `case | strategy (both, abort, unwind) | kind | owner | reason`.
  The gate fails on an unexpected failure, on a listed failure that passes,
  on a listed failure of another kind, and on a line naming no case.

## Failures found

Kinds are the same under both strategies except the NaN distance, which is
a caught `panic` under unwind and a `crash` (abort) under abort. Backtrace
frames come from lldb on the debug build (the Bazel test binaries carry no
debug info, so the runner's own backtraces are empty; add
`RUSTFLAGS=-Cdebuginfo=line-tables-only` for frames).

| Case | Kind | Fault | Owner |
| --- | --- | --- | --- |
| `bindings/path_measure_nan_distance` | panic (unwind) / crash (abort) | `PathMeasure:positionAndTangent(NaN)` (and `warp`): `ContourMeasure::get_pos_tan`'s clamp lets NaN through to `find_segment`'s `assert!(distance >= 0.0 && distance <= self.length)`, `nuxie-runtime` `mechanical_port/source/math/contour_measure.rs:403`. Frames: `find_segment` ← `get_pos_tan` ← `PathMeasure::at_distance` ← `ScriptedPathMeasure` method in `nuxie-scripting` `vm/lua_path.rs` | H1 |
| `bindings/contour_measure_nan_distance` | same | the same assert, from `ScriptedContourMeasure` | H1 |
| `viewmodel/nested_self_assignment` | crash (stack overflow, both) | `main.chi = main`: `nuxie-runtime` `ViewModelInstance::rebind_dependents_occurrence` (`mechanical_port/source/viewmodel/viewmodel_instance.rs:447`) recurses without bound on the cycle; about 6,460 frames, entered from `replace_view_model_property_occurrence` ← `NativeScriptViewModel::set_view_model` ← the scripted view-model `value` setter (`vm/view_model.rs`) | H2 |
| `viewmodel/nested_subslot_cycle` | crash (stack overflow, both) | `child.subch = child`; the same recursion. Also: an instance of the wrong view-model type was accepted into the slot | H2 |
| `viewmodel/string_property_huge_many_instances` | outcome | the view-model string setter copies through `checked_string`, not `charged_string` (`vm/view_model.rs`, `vm/lua_data_value.rs`): six instances hold 96 MiB of copies of one 16 MiB string. Unbounded (the case's first form), it passed the 512 MiB ceiling | H2 |
| `alloc/font_with_options_hoard` | outcome | `Font:withOptions` passes the face's shared charge to each variant; the cloned feature and axis maps are uncharged (`vm/lua_font.rs`). 451 to 1,775 variants of 2,704 features each, never refused (about 100 KB each; 70 to 200 MB resident). The open G3/G4 item, confirmed | H2 |
| `audio/shared_budget_vs_host_budget` | outcome | shared audio objects charge their own 512 MiB budget (`luau_audio_shared.rs` `DEFAULT_SHARED_BUDGET`), not the VM's 64 MiB host budget, and zero-fill on create: 32 MiB regions are made past 96 MiB held (384 MiB, 413 MB resident, before the case was made to stop early) | H2 |
| `audio/stream_reader_hostile` | outcome | `Audio.streamReader` reserves against that 512 MiB shared budget only (`vm/lua_audio_processors.rs` `stream_reader_value`): 8 readers of 32 MiB, 270 MB resident | H2 |

Owners: H1 is the lane editing `nuxie-runtime`, H2 the lane editing the
`nuxie-scripting` bindings, H3 the renderer. The NaN assert is in
`nuxie-runtime` code (H1), though `lua_path.rs` could also refuse a
non-finite distance (H2). The view-model recursion is in `nuxie-runtime` too,
but it is a cycle in the native view-model graph that a binding lets a script
build, which I gave to H2 (native ownership); move it to H1 if that lane owns
`viewmodel_instance.rs`. No failure is in the renderer (H3).

### Not failures, but worth knowing

- Budget errors (safepoints, memory, the deadline) cannot be caught by
  `pcall`, `xpcall` or `coroutine.resume`; they end the chunk. So
  `while true do pcall(function() while true do end end) end` ends.
- Host-function recursion stops at about 195 levels ("C stack overflow"),
  far below the 2 MiB stack; `print` → `__tostring` → `print` at about 20.
- `table.move(a, 1, 2^53, 1)`: `check_integer` clamps 2^53 to `i32::MAX`, and
  `resize_array` asks for a 2^31-slot array (`nuxie-luau-vm`
  `libs/table.rs` `tmove`). The memory limit refuses it, as an uncatchable
  memory error rather than an argument error.
- The E1 4.1 error chain no longer nests: each relay is a flat string, but
  it grows by "runtime error: " per round, so the relay is quadratic in time.
  Nothing but the module deadline stopped it (20 s at the raised deadline,
  about 1 s at the product's), not the safepoint budget.
- Gradient stops are read with no interrupt check: 12 gradients of 300,000
  stops ran 9 s inside one `draw` call. Bounded, but hang-shaped in an
  instance callback, which has no wall-clock deadline.
- `string.format("%s", t)` ignores `__tostring` ("string expected, got
  table"); stock Luau calls it. A parity difference, not a crash.
- An error object raised through `print`'s `__tostring` arrives as
  "runtime error: <non-string error>".

## What a script-only corpus could not reach

- **The Core-borrow paths** (G4 "Other paths that run script code under a
  Core borrow", E1 2(a) row 1): they need the runtime to call the script
  while it holds a borrow (state-machine listener actions, scripted path
  effects, scripted layout, transition conditions, interpolators, data-bind
  source updates). Scripts' own view-model writes are deferred, so
  `reentry/viewmodel_listener_rewrites_own_property` passes. These need a
  `.riv` scene around the script.
- **GPU resources**: without an ORE context every constructor refuses with
  "GPU context not initialized" (except `GPUBindGroup`, which reads its
  descriptor first), so `Buffer:write`, texture uploads, the deferred
  recording gap (G2 section 4) and canvases (`context:canvas`, which needs
  the init context) are not reached.
- **`decodeImage`** is a method on the script's context userdata, which the
  instance harness does not pass to `init`; no image object is reachable
  either, so image draws got argument checks only.
- **`Audio.start`, processors and `HostStream::restart`** (E1 section 3):
  processor modules and audio assets are registered by the host only; every
  start is refused. A forged descriptor through `Audio.streamReader` with an
  oversized record is rejected with an ordinary error.
- **`addToPath`** (G3: uncharged) needs an artboard instance.
- **Real `require` cycles** need registered scripts; the harness registers
  none, so only "could not find" is covered.
- **The `u32` text revision wrap**: 2^32 edits do not fit in a case; 2,000
  edits back to identical content are covered.
- **`Text:append` keeping the gradient shader** (G3 item 4): `NullFactory`
  shaders hold nothing, so the case shows only that the path is safe.
- **Finalizers**: the safe VM runs no `__gc`, so there is no loop inside one.
- **Growth that needs many frames**: the instance harness runs seven
  callbacks, each with 50,000 safepoints. Bare view-model list slots (21,000
  pushes, about 4 MiB), `TriangleBuffer` indices (12 bytes a call) and
  gradient stops (8 bytes each) cannot reach 64 MiB in that, so no case
  drives them past the budget; in a client they could grow over many frames.
  Their charges are shown only by the lanes' own unit tests.

## Not verified

- The final tip: the round-2 edits (923b448783, b46476dd31) and the
  round-3 fix (an empty filtered case set fails) were committed without a
  gate run, because disk was under the stop line. The last corpus run was
  unwind on 3f04e36eee; codex round 3 ran the corpus on the round-2 tip
  only with a filter. Run `make luau-abort-gate` once before landing.
- Apple and Android devices, and release builds: the corpus ran on macOS
  arm64 debug (fastbuild) only. Stack depths differ in release, and iOS
  secondary threads have 512 KiB stacks, not the 2 MiB used here.
- Peak memory is sampled every 50 ms; a spike shorter than that is not seen,
  and a case that ends within 50 ms shows 0 KiB.
- That each case's attack is the sharpest one for its class: the cases were
  written by four agents from the lanes' evidence and the binding sources,
  and checked by running them (every case's outcome text was read for the
  attack it names), not proven exhaustive.
- The timing-dependent cases (`alloc/font_with_options_hoard` counts
  variants for a fixed number of callbacks) have counts that vary with load;
  their pass or fail does not.

## Where the brief was wrong

- "An abort configuration means a second set of build outputs": not in this
  Bazel setup. Both strategies share one output tree and swap through the
  disk cache (see the cost section).
- "`#[should_panic]` tests cannot work" under abort: with
  `-Zpanic-abort-tests`, which stable rustc requires for any abort test
  build, they do. There were none to list in any case.
- "Run each case in a child process in the abort configuration": the runner
  does so in both, because hangs and memory blow-ups need a process to kill
  in either strategy, and because under unwind a host panic inside a
  binding is caught by the trampoline and would otherwise look like an
  ordinary error.
- "Within a stated time" with the default limits: the default 1 s module
  deadline is wall-clock, so on this loaded host it made outcomes depend on
  load; the runner raises that one deadline (above).
- The owners H1/H2/H3 do not map cleanly onto two findings (NaN assert,
  view-model recursion), which sit in `nuxie-runtime` but are reached
  through binding choices.

## Review

`codex review --base levi/luaur-client-stage1f`, three rounds (of the four
allowed), bound to corpus coverage and runner correctness:

- Round 1, 3 P2, all fixed: a host panic swallowed by `pcall` passed (the
  runner now also fails on `panicked at` in the child's output); `ipairs`
  over argument lists starting with `nil` skipped them (seven files moved to
  `table.pack`/`.n`); fallback error texts could match the expected
  `exceeded` substring (removed).
- Round 2, 2 P2 + 2 P3, all fixed: cases bounded by a time window became
  fixed-count; `bindings/list_push_past_budget` relabelled, since the budget
  is not reachable in its seven callbacks; the format case's literal `\n`
  became a newline; `bytecode/envelope_signed_truncated` now really
  truncates the signature.
- Round 3, 1 P2, fixed: a `NUXIE_HOSTILE_FILTER` that selected no case let
  the corpus test pass on zero cases; it now fails.

No round 4: free disk fell to 59 GB after round 3, under the brief's 60 GB
stop line, so I ran no more builds. The round-2 case edits and the
round-3 `assert!` have not been run by me (see Not verified); the last
corpus run was the unwind run after round 1 (3f04e36eee).
