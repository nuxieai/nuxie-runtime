# Safe Luau VM, Phase 3: cutover preparation (2026-11-13)

The mechanical preparation for flipping `nuxie-scripting` from the vendored
`luaur-vm`/`luaur-rt` to `crates/nuxie-luau-vm` (`docs/luau-safe-vm-plan.md`,
Phase 3). Branch `claude/luau-vm-cutover-prep-f866c3` from `47d8119ab4`; not
pushed, no PR. The default VM is not flipped and no vendored VM crate is
touched.

Commits:

| Commit | What |
|---|---|
| `56007e9c37` | `tools/silver-corpus` names Luau types through `nuxie_scripting::luau`; new seam `ScriptVm::upstream_test_stack_top`; silver-corpus drops its `luaur-rt`/`luaur-vm` deps |
| `99775bb331` | guarded `scripted-nuxie-vm` golden-runner variant, selected by `SCRIPTED_RUST_RUNNER_VARIANT` |
| `25fc9446ea` | `nuxie-scripting/examples/safe_vm_abort_probe.rs`, the abort-profile probe on the safe VM |
| `7144b028f3` | `nuxie/examples/scripted_startup.rs`, startup under a whole-process counter |

Host: the shared 18-core Mac, with other sessions' lanes running throughout.
The 1-minute load average is recorded next to each result; it ran from 10 to
127. Results are not taken on a quiet host.

**What blocks flipping the flag** (detail in the sections below):

1. `pairs` order (D4). One scripted golden fails (`data_bind_blob_test`), and
   three `nuxie` tests fail on the safe VM, all from hash-part iteration order.
   Two of the three are new: `upstream_scripted_path`'s
   `path_drawing_examples` and `paths_with_opacity_applied`, through
   `lua_paint.rs` `apply_definition`, which walks a Paint definition with
   `pairs`. Another session is on it.
2. **Device size is +396,816 B (+1.31%)** on the shipped `aarch64-apple-ios`
   slice (30.29 MB to 30.69 MB, section 4). Is that acceptable? Levi to
   decide; no budget binds this artifact today. Fat LTO removes the vendored
   VM and the compiler crates from the safe-VM slice, even though the
   shipping graph still compiles them. `nuxie-vm` pulls in `luaur-ast`,
   `luaur-bytecode` and `luaur-compiler` unconditionally, and
   `scripting-nuxie-vm` keeps `luau` on. That costs build time, not bytes.
3. No committed way to build the shipping artifact on the safe VM. `nux-capi`
   has no safe-VM feature. Adding one trips
   `tools/pure-runtime-boundary/check.py` (lane 7, Phase 3 item 8), and
   `runtime.py` takes no `dep/feature` roots. The size number here comes from
   a scratch edit that was reverted.
4. VM creation costs more, but small in absolute terms: about 1.9x the retired
   instructions and 4.7x the wall time of the vendored VM's
   `ScriptVm::new` plus globals plus sandbox (section 4). It is not a
   performance gate (performance is parked), but the plan asks for it to be
   measured before the flip.
5. Lane 7's open Phase 3 items still stand: the async API, interrupt `Yield`,
   `sandbox(false)`, the boundary checker and `Chunk::eval`, module-hook frame
   inspection, and bytecode v12/v13.

## 1. `tools/silver-corpus` on the facade

Reproduction (before the fix, at `47d8119ab4`), with load 23.1:

```bash
python3 tools/bazel/runtime.py check -p nuxie --features scripting-nuxie-vm,test-support --all-targets
```

The check failed with 11 errors in `tools/silver-corpus/src/scripted_misc_replay.rs` and
`scripted_source.rs`, each mixing `luaur_rt::{Function, Table, Value, Error,
Result, FromLuaMulti}` with `ScriptVm`'s `nuxie_scripting::luau` types, and
`Thread::state()` (raw `lua_State`), which the facade does not have.

Fix (`56007e9c37`):

- Both files import their types from `nuxie_scripting::luau`.
- The animated-oval producer's raw `lua_gc(LUA_GCCOLLECT)` becomes
  `Lua::gc_collect()` (identical call on `luaur-rt`; `Vm::collect` on the
  safe VM).
- Its stack-balance assertion (`lua_gettop` before and after each frame) reads
  the top through a new `upstream-test-seams` method,
  `ScriptVm::upstream_test_stack_top() -> Option<i32>`. It returns `lua_gettop`
  of the main thread on the vendored VM and `None` on the safe VM, which has no
  host API for the stack outside a call. This matches the in-crate
  stack-balance checks (`view_model.rs`, `lua_path.rs`, `lua_data_value.rs`,
  `upstream_scripting_converter.rs`), which are already vendored-only.
- `silver-corpus` no longer depends on `luaur-rt` or `luaur-vm`
  (`Cargo.toml`, `Cargo.lock`, regenerated Bazel files; `generate.py --check`
  clean). It still uses `luaur-compiler` to compile test sources, which is a
  compiler, not a VM.

After the fix, both settings check clean (worktree fixtures rsynced and
`make fixtures` run first, as fresh worktrees need):

| Command | rc | 1-min load before/after |
|---|---|---|
| `runtime.py check -p nuxie --features scripting-nuxie-vm,test-support --all-targets` | 0 | 25.7 / 21.1 |
| `runtime.py check -p nuxie --features scripting,test-support --all-targets` | 0 | 21.1 / 21.4 |

### `nuxie`'s tests on both VMs

```bash
RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3 \
  python3 tools/bazel/runtime.py test -p nuxie --features scripting-nuxie-vm,test-support
RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3 \
  python3 tools/bazel/runtime.py test -p nuxie --features scripting,test-support
```

Bazel ran all 42 targets of each setting to the end; one failing target
does not stop the others.

| Setting | targets pass / fail | tests passed / failed / ignored | 1-min load before/after |
|---|---|---|---|
| `scripting` (vendored) | 41 / 1 | 350 / 1 / 5 | 12.9 / 10.8 |
| `scripting-nuxie-vm` | 39 / 3 | 347 / 4 / 5 | 18.5 / 12.9 |

Every failure:

| Test | vendored | safe VM | Cause |
|---|---|---|---|
| `upstream_data_binding_images_direct::stateful_component_image_bind` | fails | fails | not VM: the fixture has no scripts. The test expects a reviewed layout difference against the C++ silver ("the reviewed layout difference should remain present") and the stream now matches C++. Red on this tip under both settings. |
| `upstream_wave_a_final::data_bind_blobs_internal_external_complete_action_flow` | passes | fails | `pairs` order: `data_bind_blob_test`, the known lane 7 failure (frame 0, op 1152 `addRawPath`, point x 89.38 vs 87.33, the label glyphs of `Fill 1` instead of `1`) |
| `upstream_scripted_path::path_drawing_examples` | passes | fails | `pairs` order: `script_paths` silver, "frame 0, op 6 (style): expected style, got color" |
| `upstream_scripted_path::paths_with_opacity_applied` | passes | fails | `pairs` order: `script_path_opacity` silver, "frame 0, op 7 (thickness): expected thickness, got cap" |

The two `upstream_scripted_path` failures are new in that they could not be
seen before (lane 7 could not build `nuxie`'s tests). The cause is host-side:
`crates/nuxie-scripting/src/vm/lua_paint.rs` `apply_definition` walks a Paint
definition table with `definition.pairs()` (the VM's `next`) and calls one
render-paint setter per key, so the recorded op order is the hash order of
keys such as `style`, `color`, `thickness`, `cap` and `join`. C++ walks the
same table with `lua_next`, so matching C++'s node order fixes it. A host-side
fixed key order would not, because the silver records C++'s hash order.

This cannot be a `tests/luau-vm-diff` case, because the corpus forbids
hash-order dependence (its README). It reproduces as a plain script whose
output differs between the VMs:

```lua
local t = { style = "stroke", color = 0xff00ff00, thickness = 4, cap = "round", join = "round" }
local keys = {} for k in pairs(t) do keys[#keys + 1] = k end
print(table.concat(keys, ","))
```

Handed to the `pairs`-order session in the shared notes
(`phase3-cutover-prep-pairs-sites.md`).

## 2. Goldens on both VMs

The safe-VM scripted runner is built through the content-provenance guard
(`99775bb331`): `rust_runner_provenance.py` has a `scripted-nuxie-vm` variant
(`rust-golden-runner --features scripting-nuxie-vm`, artifact
`target/<profile>/rust-golden-runner-scripted-nuxie-vm`). The Makefile picks
the scripted Rust runner with `SCRIPTED_RUST_RUNNER_VARIANT` (default
`scripted`). This replaces lane 7's hand-built runner passed as
`SCRIPTED_RUST_GOLDEN_RUNNER`. `make rust-runner-provenance-test`: 16 tests, OK.

```bash
export RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3
make scripted-golden-compare SCRIPTED_RUST_RUNNER_VARIANT=scripted
make scripted-golden-compare SCRIPTED_RUST_RUNNER_VARIANT=scripted-nuxie-vm
make golden-compare
```

The runners are the guard's rebuilds at `99775bb331`'s source state (the
debug profile, as `make` defaults). The safe-VM runner has 2,677
`nuxie_luau_vm` symbols and the default one has none (`nm`), so each run
used the VM it claims.

| Run | entries | exact (summary) | diverges | failures | 1-min load before/after |
|---|---:|---:|---:|---|---|
| scripted, vendored VM | 364 | 348 | 16 | 1: `semantic_text_inference_lt1` | 31.8 / 23.9 |
| scripted, safe VM | 364 | 348 | 16 | 2: `semantic_text_inference_lt1`, `data_bind_blob_test` | 23.9 / 30.1 |
| ordinary (no VM) | 364 | 356 | 8 | 1: `semantic_text_inference_lt1` | 30.1 / 49.4 |

The summary's `exact` counts entries labelled exact, including ones that then
fail verification (lane 7 section 9 notes the same).

Rows that differ between the two VMs: one, `data_bind_blob_test`. The safe
VM's stream differs from C++ at line 547, a `drawPath` whose rectangle is
89.38 wide instead of 87.33 (the label `Fill 1` instead of `1`: `pairs`
order in the fixture's `XmlDom` attribute walk, lane 7 section 9). The 16
diverges are the same 16 entries on both VMs, each with its reviewed signature
verified:

`bankcard collapse_data_binds data_bind_test_cmdq data_viz_demo
db_health_tracker editor_scripted_vector_v7 hit_test_test layout_text_match
list_focus_order list_index_script_access number_to_list_nested_children
reset_phase script_create_text_runs script_create_viewmodel_instance
styled_flex superbowl`

`semantic_text_inference_lt1` fails on all three runs with the same message
("semantic comparison is now exact; remove the reviewed signature
explicitly"). It is the known red on `main`, not a VM difference.

Nothing is new since lane 7: lanes 8 to 10's VM-core rewrites changed no
golden row. No VM disagreement needed reducing.

The non-scripted product compare (`make golden-compare`) uses the `ordinary`
runner, which is built without `scripting` and links no Luau VM. It is the
same binary under both settings, so one run covers both.

## 3. The abort-profile probe

**Before this branch, nothing equivalent ran against `nuxie-luau-vm`.**
`tools/luaur-rt-test-harness` depends only on `luaur-rt`/`luaur-vm`. No
example, test or make target built the safe VM with `panic=abort`. The
crate's own tests and the `luau-vm-diff` and `luau-conformance` harnesses run
under the test harness, which needs unwinding.

The port was mechanical, so it is built (`25fc9446ea`):
`crates/nuxie-scripting/examples/safe_vm_abort_probe.rs`. It is
`protected_error_recovery.rs` rewritten onto `nuxie_scripting::luau`
(`required-features = ["nuxie-vm", "compiler"]`). It is built with
`--no-default-features`, so the binary has no `luaur-rt` or `luaur-vm`
(`nm`: 0 `luaur_vm` symbols, 1,878 `nuxie_luau_vm`).

Kept from the original, three times over in one VM: an interrupt error and
recovery; scoped callback revocation; environment set and inspect; `__index`,
`__newindex`, field-call and `__eq` errors seen from the host; coroutine
yield, error and fresh resume; the whole guest block (pcall/xpcall with
failing handlers, arithmetic and index errors, math, vector, buffer, bit32,
string, pack, utf8 and table library argument errors, `__concat`, frozen
tables, `table.sort` with an erroring comparator, coroutine and wrap
errors); unprotected and malformed chunks; a successful call after each.

Added: `table.create(1e9)` must fail rather than allocate; an inconsistent
`table.sort` comparator must not abort; unbounded recursion must fail; a 4 MiB
memory cap must stop an allocation loop, and the VM must work after the cap
is lifted.

Not ported, because the facade has no such surface: the raw C API helpers
(`exec_raw`, light userdata, `luaL_getmetatable`, raw `lua_error`), scoped
userdata, `Thread::resume_error`, `Table::equals` (the `__eq` failure is
reached through a chunk instead) and the async functions.

```bash
python3 tools/bazel/runtime.py run -p nuxie-scripting --no-default-features \
  --features nuxie-vm,compiler,js-host-seed --example safe_vm_abort_probe
RUSTFLAGS=-Cpanic=abort python3 tools/bazel/runtime.py run -p nuxie-scripting \
  --no-default-features --features nuxie-vm,compiler,js-host-seed --example safe_vm_abort_probe
```

| Build | rc | sections passed | 1-min load before/after |
|---|---|---|---|
| default (`panic=unwind` from `.bazelrc`) | 0 | 8 of 8 | 40.9 / 58.8 |
| `RUSTFLAGS=-Cpanic=abort` | 0 | 8 of 8 | 58.8 / 127.4 |

The aborting binary links the `panic_abort` runtime: its strings name
`library/panic_abort/src/lib.rs`, and its `__gcc_except_tab` is 4,956 bytes,
all from std. The `-Cpanic=abort` flag comes after `.bazelrc`'s
`-Cpanic=unwind` on rustc's command line, and the last one wins.

## 4. Size and startup

### Size

**The scripting-ON size campaign's tool is broken.** `tools/size-report.sh`
(`docs/SIZE.md`, UNIV-1641's reference) builds `nux-capi` with `--features
nuxie/renderer`. `nuxie` lost its `renderer` feature in `522b551fc7`
(2026-08-24, the exact renderer cutover that removed the legacy wgpu), and
`runtime.py` rejects `dep/feature` roots (`unknown feature
nux-capi/nuxie/renderer`). Its link-closure checks also expect `wgpu_core`. It
no longer has a Makefile target.

The live size authority is the Apple distribution,
`tools/build-nux-capi-xcframeworks.sh` (its `SIZE_REPORT.json` and
`tools/apple_runtime_contract.py`). Its device artifact is the
`aarch64-apple-ios` static library of
`//crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib`
(feature set `apple-runtime`: `nux-capi/apple-authored-msl`, `apple-metal`,
`scripting`, so scripting ON). It is built in `release-apple` with fat LTO,
`codegen-units=1` and `-Cdebuginfo=0`, then `llvm-objcopy` drops
`__LLVM,__bitcode` and `__LLVM,__cmdline`, then `strip -S -x`. That slice is
what was measured, built the same way:

```bash
python3 tools/bazel_runtime_build.py build --repo-root "$PWD" \
  --label //crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib \
  --target aarch64-apple-ios --output <dir>/libnux_apple_product_extension.a
# then, as the XCFramework script does:
llvm-objcopy --remove-section=__LLVM,__bitcode --remove-section=__LLVM,__cmdline <a>
strip -S -x <a>
```

(`llvm-objcopy` from the rustup 1.94.1 toolchain, the pinned Rust version.)

The safe-VM slice needs `apple-runtime` to select the safe VM, and no
committed feature does that (blocker 3). It was built from a scratch,
uncommitted edit, reverted afterwards: `nux-capi` gained
`scripting-nuxie-vm = ["scripting", "nuxie/scripting-nuxie-vm"]`, and
`apple-runtime` named it instead of `nux-capi/scripting`, regenerated with
`tools/bazel/generate.py`. Same label, target, toolchain and flags.

| `aarch64-apple-ios` slice | vendored VM | safe VM | delta |
|---|---:|---:|---:|
| as built (`.a`) | 41,191,448 B | 41,622,648 B | +431,200 B (+1.05%) |
| shipped (objcopy + `strip -S -x`) | **30,293,008 B** | **30,689,824 B** | **+396,816 B (+1.31%)** |
| `__TEXT` sizes summed over members (`size`) | 20,133,082 B | 20,525,818 B | +392,736 B |
| data summed over members | 1,896,425 B | 1,886,817 B | -9,608 B |
| member objects | 368 | 368 | |

Symbols in the as-built archive (`nm`, count of names containing each crate):

| Crate | vendored | safe VM |
|---|---:|---:|
| `luaur_vm` | 464 | 0 |
| `luaur_rt` | 1,124 | 0 |
| `luaur_compiler` / `luaur_ast` / `luaur_bytecode` | 0 | 0 |
| `nuxie_luau_vm` | 0 | 710 |
| `nuxie_scripting` | 837 | 1,993 |

Fat LTO drops what the shipping graph compiles but does not reach. Under the
safe-VM setting, the vendored VM (still on through `luau`) and the compiler
crates (pulled in by `nuxie-vm`'s unconditional `dep:luaur-*`) leave no code
in the slice. The growth, +0.39 MB of code, is the safe VM and the facade
against the vendored VM and `luaur-rt`. The graph cost is build time, not
bytes.

Builds: vendored at `7144b028f3`, load 42.2 before and 29.4 after, 707 s;
safe VM from the scratch edit, load 30.0 before and 34.8 after, 274 s. Free
disk was 88 GB at the lowest.

### Startup

No tooling measured VM startup. `tools/luau-bench` measures scripts after the
VM exists. Added (`7144b028f3`): `crates/nuxie/examples/scripted_startup.rs`
runs one stage N times after reading the fixture once:

- `vm`: `ScriptVm::new` plus `install_rive_globals`, which ends in
  `luaL_sandbox` plus `luaL_sandboxthread`.
- `import`: the whole unsigned scripted import (VM, globals, sandbox, every
  script asset's bytecode loaded and initialised), dropped each iteration.

Each binary is built in release (`--config=release`: fat LTO, `codegen-units=1`),
`--features scripting,test-support` and
`--features scripting-nuxie-vm,test-support`. The measure runs
`/usr/bin/time -l` at two iteration counts (vm 20 and 220, import 10 and 110).
The per-start figure is the difference in the process's retired instructions
over the difference in iterations, so process start and the fixture read
cancel. Three repeats each; minimum reported. Wall time is the binary's own
`Instant` over the larger run, per iteration. Fixtures come from the pinned
upstream: `script_artboard_test.riv` (3.9 KB, one script) and
`script_dependency_test_using_library.riv` (1.7 MB, a script library with
dependent modules). Both are exact on both VMs in the scripted goldens.
Load 52.2 before, 47.9 after.

| Stage | Fixture | vendored instr | safe VM instr | ratio | vendored wall min/med (µs) | safe VM wall min/med (µs) |
|---|---|---:|---:|---:|---:|---:|
| `vm` | none | 1.319 M | 2.520 M | 1.91x | 90.9 / 100.0 | 427.3 / 501.1 |
| `import` | `script_artboard_test` | 3.811 M | 5.137 M | 1.35x | 527.1 / 919.5 | 867.9 / 929.7 |
| `import` | `script_dependency_test_using_library` | 109.851 M | 111.395 M | 1.01x | 10,777.5 / 11,203.8 | 9,701.8 / 10,220.5 |

Repeat spread was within 3% of each minimum for instructions. Wall times are
on a loaded host (load around 50), so their medians are rough.

The safe VM's fixed cost is VM creation: about +1.2 M instructions and
+340 µs per VM. Its wall time grows more than its instructions, which points
at memory first-touch rather than work. This was not profiled here
(performance is parked). On the library fixture, the import's other work
dominates and the two VMs are within 1.5%.

## 5. Not verified

- **Platform builds other than the iOS device slice.** The simulator, macOS
  and Android slices, and the XCFramework packaging, were not built with
  `nuxie-vm`. The safe-VM size uses a scratch feature, not a committed build
  path.
- **A linked size.** The measured artifact is the shipped static library, not
  an app that links it. Dead-stripping in an app link may remove more under
  either VM.
- **The original probe on the vendored VM under `panic=abort`.** Only the
  safe-VM port was run; `tools/luaur-rt-test-harness` was not rerun.
- **The probe's unported cases** (raw C API, scoped userdata, `resume_error`,
  async) have no safe-VM equivalent today.
- **`stateful_component_image_bind`** fails on both VMs. It was not traced
  further; the fixture has no scripts.
- **Startup wall times** come from a loaded host; the instruction counts are
  the reliable figure. Startup was not profiled.
- **`nux-capi` and the other `nuxie` dependents' test suites** were not run
  under `scripting-nuxie-vm`; only `nuxie`'s.
- **Repeatability of the goldens.** Each golden setting ran once.
