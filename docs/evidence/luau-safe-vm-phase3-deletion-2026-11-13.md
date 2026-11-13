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
| `tools/luau-conformance/src/harness.rs` (the vendored-VM harness, 11) | all | 3 |
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
| `tools/luau-conformance/{BUILD.bazel 9, Cargo.toml 1, src/main.rs 2, src/harness_nuxie.rs 3}`, `Makefile` `luau-conformance*`, `tests/luau-conformance/ATTRIBUTION.md`, `tools/luau-conformance/fetch-upstream.sh` | 15 | 3 | the safe VM is the only VM; `luau-conformance-nuxie` becomes `luau-conformance` |
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
