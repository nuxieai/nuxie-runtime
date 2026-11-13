# Phase 3: deleting the vendored Luau VM (2026-11-13)

Plan: `docs/luau-safe-vm-plan.md`, Phase 3. The flip
(`docs/evidence/luau-safe-vm-phase3-flip-2026-11-13.md`, PR #1417) made the
safe VM (`crates/nuxie-luau-vm`) the default everywhere and left the vendored
VM (`vendor/luaur-rt-0.1.8`, `vendor/luaur-vm-0.1.8`) as the `vendored-vm`
opt-out and as the reference inside four tools. Levi approved deleting it in
five steps, one commit each. Base: `origin/main` `e25de6390c`.

## 0. Inventory

Every file outside the two deleted crates that names `luaur-rt`, `luaur_rt`,
`luaur-vm`, `luaur_vm`, `vendored-vm` or `scripting-vendored-vm` at the base
(`grep -rlE`; count = matching lines), and what happens to it. "Step" is the
commit that handles it.

### Deleted

| Item | Lines | Step |
|---|---:|---|
| `vendor/luaur-rt-0.1.8/**` (crate, tests, `BUILD.bazel`, patch notes) | all | 4 |
| `vendor/luaur-vm-0.1.8/**` (crate, `luau_run` bin, `BUILD.bazel`) | all | 4 |
| `bazel/cargo/luaur-rt/`, `bazel/cargo/luaur-vm/` and the `bazel/native-tools-cargo/` copies (generated) | 5+2 each | 4 |
| `tools/luaur-rt-test-harness/**` (`BUILD.bazel` 118, `Cargo.toml` 7, `README.md` 3, examples 17, src 1, tests 4) and its generated `bazel/*/luaur-rt-test-harness/` | all | 3 |
| `tools/luau-conformance/src/harness.rs` as it was (the vendored-VM harness, 11); `harness_nuxie.rs` takes its name | all | 3 |
| `tests/luau-conformance/scope.txt` (the vendored VM's scope, 2; only `make luau-conformance` reads it) | all | 3 |
| `crates/nuxie-scripting/src/vm/luau_audio_processor_engine/raw.rs` (raw C API audio engine, 9) | all | 2 |

### Rewritten

| Item | Lines | Step | Change |
|---|---:|---|---|
| `tools/luau-vm-diff/src/{lib,main}.rs`, `Makefile` `luau-vm-diff` | 15 | 1, 3 | snapshot mode (1); reference side removed (3) |
| `tools/luau-vm-diff/src/host_surface.rs` (12), `host_api_tests.rs` (1), `Cargo.toml` (2), `BUILD.bazel` (18) | 33 | 3 | `install_reference` and the `luaur-rt`/`luaur-vm` deps removed |
| `tests/luau-vm-diff/README.md`, `debuglib.luau` (1), `host_surface.luau` (2) | 3 | 1, 3 | snapshot workflow; comments name C++ Luau's behavior |
| `tools/luau-vm-diff/count_pairs_order.py` | 0 | 1 | reads the snapshot side of `--verbose` |
| `crates/nuxie-scripting/Cargo.toml` (10) | 10 | 2 | `vendored-vm`, `async`, the `luaur-rt`/`luaur-vm` deps and `luaur-rt?/compiler` removed |
| `crates/nuxie-scripting/src/**` (`vm.rs` 59, `lua_mat4.rs` 98, `luau/facade/*` 93, `luau/mod.rs` 12, `view_model.rs` 20, `lua_main_ref.rs` 20, `property_lifetime_a637_tests.rs` 18, `lua_promise.rs` 14, `lua_rive_file.rs` 12, `lua_path.rs` 11, `artboard_context_8f06_tests.rs` 10, `lua_font.rs` 9, `gpu_canvas_ore/*` 9, `lua_input.rs` 6, `lua_data_value.rs` 6, `lua_transition.rs` 6, `luau_audio_processor_engine.rs` 6, `bytecode.rs` 3, `lua_mat4_safe.rs` 2, 7 files with 1) | ~430 | 2 | every `cfg(feature = "vendored-vm")` item deleted, the safe-VM side kept unconditional |
| `crates/nuxie-scripting/tests/{default_features,upstream_module_hooks,upstream_scripting_converter,upstream_scripting_vector}.rs` | 14 | 2 | vendored-only tests and assertions deleted (flip evidence section 5) |
| `crates/nuxie-scripting/examples/safe_vm_abort_probe.rs` (1) | 1 | 3 | comment no longer points at the deleted harness |
| `crates/nuxie/Cargo.toml` (2), `crates/nuxie/examples/scripted_startup.rs` (1) | 3 | 2 | `scripting-vendored-vm` removed |
| `tools/rust-golden-runner/Cargo.toml` (2), `tools/golden-runner/rust_runner_provenance.py` (4), `Makefile` (1) | 7 | 2 | `scripting-vendored-vm` and the `scripted-vendored-vm` variant removed |
| `tools/luau-conformance/{BUILD.bazel 9, Cargo.toml 1, src/main.rs 2, src/harness_nuxie.rs 3}`, `Makefile` `luau-conformance*`, `tests/luau-conformance/ATTRIBUTION.md`, `tools/luau-conformance/fetch-upstream.sh` | 15 | 3 | the safe VM is the only VM: `--vm` is gone, `harness_nuxie.rs` becomes `harness.rs` (with the helpers the two harnesses shared and their tests), `luau-conformance-nuxie` becomes `luau-conformance` |
| `tools/luau-bench/{BUILD.bazel 18, Cargo.toml 1, src/vm.rs 6}`, `Makefile` `luau-bench*` | 25 | 3 | vendored VM side removed; `baseline-luaur.json` kept as a record |
| root `Cargo.toml` (7) | 7 | 3, 4 | harness member (3); exclude and patch entries, fork comment (4) |
| `Cargo.lock` (12), `bazel/{cargo,native-tools-cargo}/{Cargo.lock 18, Cargo.Bazel.lock 15, Cargo.toml 3}` and per-crate generated manifests (`luau-bench`, `luau-conformance`, `luau-vm-diff`, `nuxie-scripting`, `nuxie`, `rust-golden-runner`), root `BUILD.bazel` (1) | 105 | 2, 3, 4 | regenerated (`tools/bazel/generate.py`, repin) |
| `tools/bazel/emit.py` (1) | 1 | 4 | `luaur-rt-derive` (a dependency of `luaur-rt` only) leaves `PROC_MACROS` |
| `tests/ratchets/unsafe-baseline.txt` (2), `tests/ratchets/panic-lints-baseline.txt` (14), `tools/ratchets/panic-lints.sh` (1, comment) | 17 | 4 | the two crates' lines removed |
| `tools/pure-runtime-boundary/check.py` (2) | 2 | 4 | the two paths leave the audited third-party list |
| `THIRD_PARTY_NOTICES.md` (1) | 1 | 4 | lists the luaur crates that remain |
| `crates/nuxie-luau-vm/src/{fastcall.rs 1, value.rs 1}` | 2 | 4 | source citations point at C++ Luau instead of the deleted Rust transliteration |
| `crates/nuxie-luau-vm/src/{embed/mod.rs 14, embed/compat.rs 2, embed/convert.rs 1, error.rs 1}` | 18 | 4 | doc comments say "the vendored `luaur-rt`'s shape" in the past tense |
| Living docs: `docs/PORTING.md` (2), `docs/luau-fork.md` (12), `docs/luau-safe-vm-seam.md` (58), `docs/luau-safe-vm-embedding-core.md` (3), `docs/luau-safe-vm-call-path.md` (1), `docs/luau-safe-vm-pairs-order.md` (1), `docs/perf-gap-map.md` (1), `docs/luau-authored-audio-processors-plan.md` (1) | 79 | 5 | describe the old VM as gone (see section 6) |
| `docs/luau-safe-vm-plan.md` (15) | 15 | 5 | dated entry appended; history kept |

### Kept as written (dated records)

`docs/evidence/*-2026-*.md` (14 files, 151 lines) and `docs/sync/triage-2026-07-1{8,9}-d788e8ec.md`
(2): dated evidence and triage records of what was true then.

### Stopped on (not on the approved list)

| Item | Lines | Why |
|---|---:|---|
| `tools/size-report.sh:372-376` | 3 | A live check: the scripting-on size artifact must retain `luaur_vm` symbols (and the scripting-off one must not). Since the flip the scripting-on graph links `nuxie_luau_vm`, so the check already fails on `main`; the fix is to name `nuxie_luau_vm`. Not on the approved list, so left as is. |
| `docs/SIZE.md:174` | 1 | Describes that same check. Left with it. |

## 1. Step 1: the differential's reference, frozen

`tools/luau-vm-diff` now compares the safe VM with committed snapshots,
`tests/luau-vm-diff/snapshots/<case>.<profile>.txt`: one per case and
compiler profile, holding exactly what the live differential compared
(returned values, every `print` line, the escaping error) plus the bytecode
version, one quoted and escaped item per line. A snapshot without a case
fails the run. `make luau-vm-diff` runs the safe VM only.

Generated from the vendored VM with `--record`, with the vendored VM still
in the tree:

| Check | Runs | Differences |
|---|---:|---:|
| `--record` twice (committed dir, then a scratch dir), `diff -r` | 409 + 409 | 0 files differ |
| `--against-reference`: vendored VM live vs snapshot | 409 | 0 |
| `--against-reference`: safe VM live vs vendored VM live | 409 | 0 |
| default mode: safe VM vs snapshot | 409 | 0 |
| negative check: one edited snapshot line + one stray snapshot file | 409 | 2 (both reported, exit 2) |

- 409 snapshot files (103 cases: 100 under 4 profiles, 3 flags-on cases under 3), 19 MB on
  disk; the largest are the generated `fastcall_math` (803 KB each) and
  `fastcall_buffer` cases.
- Bytecode versions 7 (base profile) and 11. The corpus still reaches every
  admitted opcode.
- Cases that could depend on addresses, timing or collector pacing
  (`gc.luau`, `coroutine_gc.luau`, `fastcall_base` `tostring` of tables,
  the pairs-order corpus) were already written to mask them; the two
  recordings are byte-identical.
- `count_pairs_order.py` now reads the snapshot side of a `--verbose` run.
- Host load during the recordings: load average 15-18 (other sessions
  running); the build went through the pool.

## 2. Step 2: the opt-out removed

Commit `4ad0627684` (45 files, +74 -2566). `vendored-vm` (nuxie-scripting),
`scripting-vendored-vm` (nuxie, rust-golden-runner), the
`scripted-vendored-vm` runner variant, the `async` feature and its one test,
and every `cfg(feature = "vendored-vm")` item (173, removed mechanically and
then reviewed) are gone, with `luau_audio_processor_engine/raw.rs` and
`tests/default_features.rs`. Where a `cfg(not(feature = "vendored-vm"))`
item had a vendored twin, the safe-VM item is now unconditional and
unchanged. Runtime string literals that name the old VM
(`"luaur-rt-c-function"`, `"luaur-rt-callback"`, `"luaur-rt-exec-raw"`) are
behavior (chunk names and error sources scripts can observe) and stay.

## 3. Step 3: the tools' vendored sides removed

- `tools/luau-vm-diff`: the reference VM, `--against-reference`, the
  vendored host surface and the `luaur-rt`/`luaur-vm` dependencies are gone.
  `--record` now writes only the snapshots a case is missing, from the safe
  VM, and never overwrites one; the README says how a new case's snapshots
  are reviewed by hand until the pinned C++ Luau VM becomes the live
  reference. The NaN-key test keeps its assertion as the literal the
  vendored VM returned (checked against it before step 3). Comments in two
  corpus cases changed without changing their line counts, so no error
  position in a snapshot moved.
- `tools/luau-conformance`: one harness, one scope file
  (`scope-nuxie.txt`), one target (`make luau-conformance`, which
  `luau-vm-ratchets` runs).
- `tools/luau-bench`: `nuxie` is the only VM; `luau-bench-check` (the
  vendored VM's ceiling) is gone. `tests/luau-bench/baseline-luaur.json`
  stays as a record; nothing re-measures it.
- `tools/luaur-rt-test-harness` is deleted. Nothing outside it depended on
  it; the abort-profile probe (`crates/nuxie-scripting/examples/safe_vm_abort_probe.rs`)
  was already its port.
- Bazel: `tools/bazel/generate.py`, then both crate universes repinned
  (`--reset-lock`, `CARGO_BAZEL_REPIN=1`, `CARGO_BAZEL_REPIN_ONLY`).

| Check | Result |
|---|---|
| `runtime.py check --all-targets` on luau-vm-diff, luau-conformance, luau-bench, nuxie-scripting | exit 0, no warning in the changed tools |
| `runtime.py test` on the three tools | 57 + 7 + 4 passed |
| `make luau-vm-diff` | 409 runs, 0 disagreements, every admitted opcode reached |
| `make luau-conformance` | 55 scripts, 37 required all OK, 0 panics, PASS |
| `--record` on a scratch corpus | writes 4 missing snapshots; rerun writes none; an edited snapshot still fails (exit 1) and is not overwritten |

Load average 15-23 during these runs.

## 4. Step 4: the two crates deleted

Commit `54115c85ee` (1,786 files, +312 -80,025). `vendor/luaur-rt-0.1.8`
and `vendor/luaur-vm-0.1.8` (1,733 tracked files, 79,318 lines) are gone,
with their root `Cargo.toml` exclude and patch entries, the generated
`bazel/{cargo,native-tools-cargo}/luaur-{rt,vm}` packages, their lockfile
entries (root and both Bazel universes, repinned), their ratchet baseline
lines, `luaur-rt-derive` in `tools/bazel/emit.py`, and their paths in
`tools/pure-runtime-boundary/check.py` and `THIRD_PARTY_NOTICES.md`. Nothing
the kept crates or their tests need came from `luaur-rt`: after step 3,
`cargo metadata` reported both patches as unused, so no `compiler`-feature
move onto the compiler crates was needed.

Doc comments in `nuxie-scripting`'s facade and in `nuxie-luau-vm`'s
embedding API cited `luaur-rt` as the shape they keep. They now call it
"the old binding crate", defined once in `nuxie_scripting::luau` and once in
`nuxie_luau_vm::embed` with a pointer to this file. Two source citations
into the deleted VM (`fastcall.rs`, `value.rs`) now point at C++ Luau.

### What still names the two crates

`git grep -E 'luaur-rt|luaur_rt|luaur-vm|luaur_vm|vendored-vm'` outside
`docs/evidence/`:

| Where | Lines | Why it stays |
|---|---:|---|
| `crates/nuxie-scripting/src/vm.rs`, `vm/lua_mat4_safe.rs`, `vm/lua_promise.rs`, `luau/facade/state.rs` | 6 | String literals: the chunk names `"luaur-rt-c-function"` and `"luaur-rt-callback"` and the error text `"luaur-rt: missing callback upvalue"`. Scripts observe them (`debug.info`, tracebacks, error messages), so renaming them is a behavior change, not a deletion. |
| `tools/size-report.sh:372-376`, `docs/SIZE.md:174` | 4 | The stopped-on item (section 0). |
| `docs/luau-safe-vm-plan.md` | 15 | The plan's history. |
| `docs/luau-fork.md` | 10 | Dated history: the carried-patch list and the 2026-08-04 audit. The fork table and status now list only the kept crates. |
| `docs/luau-safe-vm-{seam,embedding-core,call-path,pairs-order}.md`, `docs/perf-gap-map.md` | 74 | Dated design records. Each now opens with a note that the crates were deleted and that its paths name files at `e25de6390c`. |
| `docs/sync/triage-2026-07-1{8,9}-d788e8ec.md` | 2 | Dated triage records. |

### `unsafe` sites (`tools/ratchets/unsafe-count.sh`)

| Crate | Before (`e25de6390c`) | After |
|---|---:|---:|
| `crates/nuxie-luau-vm` | 0 | 0 |
| `vendor/luaur-ast-0.1.8` | 1,017 | 1,017 |
| `vendor/luaur-bytecode-0.1.8` | 232 | 232 |
| `vendor/luaur-common-0.1.8` | 174 | 174 |
| `vendor/luaur-compiler-0.1.8` | 744 | 744 |
| `vendor/luaur-rt-0.1.8` | 320 | deleted |
| `vendor/luaur-vm-0.1.8` | 1,431 | deleted |
| **ratcheted total** | **3,918 over 7 crates** | **2,167 over 5 crates** |
| `crates/nuxie-scripting` (not ratcheted; same pattern) | 338 | 52 |

The 52 in `nuxie-scripting` are 51 sites and one comment that matches the
pattern (`luau_audio_processor_engine/safe.rs:205`). None is VM access:

| Kind | Sites | Where |
|---|---:|---|
| The luaur compiler's C entry point (`luau_compile` output read with `from_raw_parts`, `extern "C" free`) | 3 each | `build.rs`, `gpu_canvas.rs`, `vm/lua_path.rs`, `vm/lua_audio_processors.rs`, `vm/luau_audio_processor_engine.rs`; tests `support/mod.rs`, `promise_scenarios.rs`, `scripted_interpolator.rs` (24) |
| Same, one site | 1 each | tests `async_shader_instantiation.rs`, `shader_asset_resolution.rs` (2) |
| Browser audio host imports (wasm `extern "C"`, `unsafe fn` exports) | 12 | `vm/luau_audio_web.rs` |
| GPU canvas texture wrapping (`wrapCanvasTextureInfo`, `wrapImageSampleView`, `unsafe fn` trait items in test doubles) | 10 | `gpu_canvas_ore/{canvas,mod}.rs`, `shader_asset.rs`, `vm/script_call_gpu_scope_tests.rs`, `vm/upstream_scripting_gpu_features.rs` |
| Renderer lifetime erasure (`transmute` of `NonNull<dyn Renderer>`, `as_mut`) | 2 | `vm/lua_renderer.rs` |
| `unsafe fn upstream_test_scripted_artboard_render` (test seam) | 1 | `vm/lua_artboards.rs` |
| comment | 1 | `vm/luau_audio_processor_engine/safe.rs` |

Step 2 removed 286 of the 338 (the raw C API audio engine, the vendored
VM's `unsafe` blocks and the facade-free paths).

## 5. Battery on the tip

Run on `54115c85ee` (step 4) with the step-5 docs commit on top for the
artifact builds; the docs commit changes no code. Fixtures: `make fixtures`
with `RIVE_RUNTIME_DIR` at the `cd5351c3` pin, plus the untracked
`crates/nuxie/tests/fixtures/` copy (moved out of the tree for the
artifact builds, which refuse a dirty tree). Builds and tests through
`tools/bazel/runtime.py` and `make`. Load is the 5-minute average before /
after each step; other sessions were running throughout.

Three things went wrong on the way; what they touched was re-run, not trusted:

- The first test and golden pass ran without `RIVE_RUNTIME_DIR`, so every
  test that loads an upstream asset failed and both golden runners refused
  the wrong checkout. Re-run with it set (rows below).
- The machine was interrupted while the re-run was going. Only steps with a
  recorded exit status count; the ordinary golden compare and the workspace
  check had none and were run again.
- Free disk fell under 60 GB twice while the workspace check filled this
  worktree's Bazel output base (53 GB at one point; other sessions were
  writing too). A watchdog stopped it both times. It was then run in three
  package groups with `bazel clean` between them.

### Ratchets, differential, conformance, VM crate

| Command | Result | Load |
|---|---|---|
| `tools/bazel/generate.py --check` | clean | 19.3 |
| `make luau-vm-ratchets` | pass. `unsafe`: 2,167 over 5 crates, none up. Panic lints: `nuxie-luau-vm` 0 forced sites, none up | 19.3 / 15.1 |
| ↳ `make luau-conformance` (inside the ratchets) | 55 scripts, 37 of 37 required OK, 0 panics, 0 timeouts, PASS | (same) |
| `make luau-vm-diff` | 409 runs, 0 disagreements, bytecode v7 and v11, every admitted opcode reached | 15.1 / 13.3 |
| `runtime.py test -p nuxie-luau-vm` | 76 passed | 13.3 / 13.1 |
| `runtime.py clippy -p nuxie-luau-vm --all-targets` | pass | 13.1 / 14.0 |

### Tests

| Command | Targets | Passed / failed / ignored | Failures | Load |
|---|---|---|---|---|
| `runtime.py test -p nuxie-scripting` | 39 / 39 pass | 718 / 0 / 5 | — | 8.6 / 14.4 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41 / 42 pass | 350 / 1 / 5 | `stateful_component_image_bind` (known red on `main`) | 14.4 / 15.0 |
| `runtime.py test -p silver-corpus` | 21 / 22 pass | 255 / 2 / 4 | `wave_c2_layout_020_aspect_ratio`, `aspect_ratio_fill_and_stretch_follow_both_animated_parent_axes` (known `wave_c2` reds) | 15.0 / 11.2 |
| `runtime.py test --locked -p nux-capi --no-default-features` | 25 / 25 pass | 313 / 0 / 0 | — | 11.2 / 14.4 |

Every count equals the flip's default-VM row. `nuxie-scripting` runs the
same 718: the tests deleted in step 2 ran only under the opt-out.

### Goldens

| Run | Entries | Exact | Diverges | Exit | Load |
|---|---:|---:|---:|---|---|
| `make scripted-golden-compare` | 364 | 348 | 16 | 0 | 14.4 / 10.7 |
| `make golden-compare` | 364 | 356 | 8 | 0 | 22.2 / 12.3 |

### Abort-profile probe (`safe_vm_abort_probe`)

| Build | rc | Sections | Binary | Load |
|---|---|---|---|---|
| `runtime.py run -p nuxie-scripting --example safe_vm_abort_probe` (`panic=unwind`) | 0 | 8 of 8 | — | 16.4 / 14.7 |
| same with `RUSTFLAGS=-Cpanic=abort` | 0 | 8 of 8 | 4,971,464 B; `__gcc_except_tab` 4,956 B; names `library/panic_abort/src`, no `panic_unwind`; 1,889 `nuxie_luau_vm` symbols, 0 `luaur_vm` or `luaur_rt` | 13.7 / 7.3 |

The abort build's numbers equal the flip's.

### Whole workspace and other graphs

| Command | Result | Load |
|---|---|---|
| `runtime.py check --all-targets`, every workspace package, in three groups (runtime and renderer crates; C API, scripting, VM and data crates; tools) | 329 + 113 + 79 = 521 targets, 0 errors (the flip's 531 included the deleted harness's targets) | 9.5 / 18.8, 7.6 / 18.0, 11.5 / 17.2 |
| `runtime.py check -p nuxie-scripting --no-default-features` | clean | 18.8 / 24.7 |
| `tools/luau-audio-web/pool.py build` (wasm, `-Cpanic=abort`, `//tools/luau-audio-web:kernel :probe`) | builds: `nuxie_luau_audio_kernel.wasm` 4,448,220 B, `nuxie_luau_audio_probe.wasm` 84,685,938 B | 24.7 / 14.5 |
| `tools/pure-runtime-boundary/check.py` | 129 findings, as on `origin/main` (`e25de6390c`); none names a deleted path | 14.5 / 12.1 |

### Apple

| Command | Result | Seconds | Load |
|---|---|---:|---|
| `make nux-capi-xcframeworks` | pass: "shipped surface contains only runtime/platform semantics", "dual-XCFramework verification passed" | 2,146 | 7.3 / 7.7 |
| `make crate-seams-apple-check` | pass | 337 | 7.7 / 9.9 |

`target/nux-capi-apple/SIZE_REPORT.json` against
`crates/nux-capi/size-budgets-v3.json` and the flip (its section 8):

| Artifact | Measure | Bytes | Budget | Used | vs flip |
|---|---|---:|---:|---:|---:|
| full-apple | compressed | 55,620,762 | 78,643,200 | 70.7% | -139 |
| full-apple | expanded | 158,145,999 | 241,172,480 | 65.6% | -172 |
| full-apple | linked C, macOS arm64 | 22,505,136 | 30,408,704 | 74.0% | 0 |
| full-apple | linked Swift, macOS arm64 | 22,636,040 | 30,408,704 | 74.4% | 0 |
| full-apple | slice `aarch64-apple-darwin` | 30,217,992 | 48,234,496 | 62.6% | +48 |
| full-apple | slice `aarch64-apple-ios` | 30,696,000 | 49,283,072 | 62.3% | +80 |
| full-apple | slice `aarch64-apple-ios-sim` | 30,683,832 | 49,283,072 | 62.3% | +144 |
| full-apple | slice `x86_64-apple-darwin` | 32,045,360 | 49,283,072 | 65.0% | +184 |
| full-apple | slice `x86_64-apple-ios` | 32,046,152 | 48,365,568 | 66.3% | +184 |
| ios-only | compressed | 33,632,262 | 47,185,920 | 71.3% | +76 |
| ios-only | expanded | 95,707,942 | 147,849,216 | 64.7% | -404 |
| ios-only | linked C, iOS arm64 | 22,724,184 | 30,408,704 | 74.7% | 0 |
| ios-only | linked Swift, iOS arm64 | 22,837,264 | 30,408,704 | 75.1% | 0 |

No budget is exceeded. The slices are 48 to 184 B over the flip's; the
flip measured before `e25de6390c`, which added shipped runtime code
(`state_machine_instance.rs`). Section 6 separates the two.

### Android

| Command | Result | Seconds | Load |
|---|---|---:|---|
| `make nux-capi-android` | pass: "NuxieRuntimeAndroid.zip satisfies the immutable ABI-v4 Android contract" | 891 | 10.0 / 8.7 |
| `make nux-capi-android-contract-test` | pass: 28 tests OK, then the build and publish plans | 1 | 9.9 |

| Entry | Bytes | Budget | Headroom | vs flip |
|---|---:|---:|---:|---:|
| archive (`NuxieRuntimeAndroid.zip`) | 25,605,445 | 26,214,400 | 608,955 (2.3%) | +929 |
| expanded | 73,657,175 | 75,497,472 | 1,840,297 | — |
| `arm64-v8a/libnux_capi.so` | 26,406,824 | 29,360,128 | 2,953,304 | 0 |
| `x86_64/libnux_capi.so` | 28,799,368 | 29,360,128 | 560,760 | 0 |
| `arm64-v8a/libc++_shared.so` | 9,290,184 | — | — | 0 |
| `x86_64/libc++_shared.so` | 9,015,544 | — | — | 0 |

Every member is the flip's size. The archive is compressed, so it moves
with member bytes that do not change sizes (the embedded source revision
among them).

## 6. The shipped slice: this branch against its base

To separate this branch from the base's own movement since the flip, the
`aarch64-apple-ios` slice was built at the base (`e25de6390c`, a detached
worktree) and at this tip the way `tools/build-nux-capi-xcframeworks.sh`
builds it (`tools/bazel_runtime_build.py build` for
`//crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib`,
then the pinned `llvm-objcopy` and `strip -S -x`), with identical
`NUX_RUNTIME_*` provenance values on both sides.

| `aarch64-apple-ios` | base `e25de6390c` | this branch | difference |
|---|---:|---:|---:|
| as built (`.a`) | 41,630,960 B | 41,630,960 B | 0 |
| shipped | 30,696,000 B | 30,696,000 B | 0 |
| members | 369 | 369 | 0 |
| members that differ in size | | | 0 |
| members that differ in bytes | | | 1 (`nux_apple_product_extension` codegen unit: 182 bytes, same section sizes, same strings) |

The shipped slice did not grow: it is the base's size exactly, and
30,696,000 B is also what `make nux-capi-xcframeworks` shipped (section 5).
The +80 B against the flip's slice is the base's (`e25de6390c` added
shipped runtime code after the flip measured). Symbols in the as-built
archive: 712 `nuxie_luau_vm`, 0 `luaur_vm` or `luaur_rt`, as at the flip.
Builds took 397 s and 384 s, load 17.3 / 9.1 and 10.4 / 8.1.

## 7. What was not verified

- Nothing ran on a device. `make crate-seams-apple-check` ran its hosts in
  the simulator and natively on macOS, as at the flip; the Android contract
  test checks the package, not a device run.
- The browser runtime: the wasm kernel and probe build; nothing ran them
  (`//tools/luau-audio-web:transport_test` needs `node`, which the local
  sandbox lacks; the pool runs it on Linux).
- Hosted CI: not run (local gates only). The branch is not pushed.
- The new `--record` workflow was exercised on a scratch corpus only; no
  real case has been added through it yet.
- `tools/size-report.sh`'s scripting-on symbol check (section 0, stopped
  on) was not run; it fails on `main` since the flip and is untouched here.

## 8. Review

`codex review --base origin/main`, round 1 (on the four code commits): no
findings. It ran the three tools' tests (68 passed) and the differential
(409 runs, 0 disagreements).
Round 2 (with this document and the plan entry): no findings; it checked
the generated Bazel files, that all 409 snapshots are present, and ran the
build-tool tests (49 passed).
