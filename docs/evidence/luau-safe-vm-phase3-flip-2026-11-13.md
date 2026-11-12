# Safe Luau VM, Phase 3: the default flip (2026-11-13)

`nuxie-scripting`'s default VM is now `crates/nuxie-luau-vm`
(`docs/luau-safe-vm-plan.md`, Phase 3). `nuxie`'s `scripting`, and so
`nux-capi`'s `scripting = ["nuxie/scripting"]` and the Apple and Android
artifacts, build on the safe VM with no new `nux-capi` feature. The vendored
VM (`luaur-rt`/`luaur-vm`) stays selectable through one opt-out,
`vendored-vm`, until a separate deletion PR that awaits Levi's approval. No
vendored VM crate was edited (their generated `BUILD.bazel` files change only
because the feature graphs that reach them changed).

Branch `claude/luau-vm-flip-c595da` from `origin/main` at `766fcbe7e1`. Not
pushed, no PR.

## 1. The feature graph

### Before (`origin/main`, `766fcbe7e1`)

| Crate | Feature | Meant |
|---|---|---|
| `nuxie-scripting` | `default` | `luau`, `compiler`, `async`, `js-host-seed` |
| | `luau` | `luau-host` + `luaur-rt`: the vendored VM, the default |
| | `nuxie-vm` | `luau-host` + `nuxie-luau-vm` + `luaur-ast`/`luaur-bytecode`/`luaur-compiler`; won over `luau` when both were on |
| | `luau-host` | shared deps, including `luaur-vm` |
| | `compiler` | `luau-host`, `luaur-rt?/compiler` |
| | `async` | `luau`, `luaur-rt/async` |
| | `upstream-test-seams` | `luau-host` |
| `nuxie` | `scripting` | `nuxie-scripting?/luau` (vendored) |
| | `scripting-nuxie-vm` | `scripting` + `nuxie-scripting?/nuxie-vm` |
| `nux-capi` | `scripting` | `nuxie/scripting` |
| `nux-apple-product-extension` | `apple-runtime` | `nux-capi/scripting` (+ Metal, authored MSL) |
| `rust-golden-runner` | `scripting` / `scripting-nuxie-vm` | runner variants `scripted` (vendored) / `scripted-nuxie-vm` |
| `tools/bazel/generate.py` | editor/publisher graphs | `nuxie-scripting: luau` (vendored) |
| | browser graphs (webgpu, webgl2) | `nuxie-scripting: luau, async, …` (vendored) |
| | apple, android graphs | through `nux-capi/scripting` (vendored) |

Code gated on `nuxie-vm` in 219 places (44 `feature = "nuxie-vm"`, 175
`not(feature = "nuxie-vm")`). Because Cargo features only add, an opt-out
has to be the feature that wins when present. On `main` the winner was
`nuxie-vm`, the opt-in. So the precedence had to invert.

### After

| Crate | Feature | Means |
|---|---|---|
| `nuxie-scripting` | `default` | `luau`, `compiler`, `js-host-seed` |
| | `luau` | `luau-host` + `nuxie-luau-vm`: **the safe VM** |
| | `vendored-vm` | `luau` + `luaur-rt` + `luaur-vm`: **the opt-out; wins when on** |
| | `luau-host` | shared deps (no VM crate) |
| | `compiler` | `luau`, `luaur-rt?/compiler`, `luaur-ast`, `luaur-bytecode`, `luaur-compiler` |
| | `async` | `vendored-vm`, `luaur-rt/async` (vendored-only API, no product caller) |
| | `upstream-test-seams` | `luau` |
| `nuxie` | `scripting` | `nuxie-scripting?/luau` (**safe VM**) |
| | `scripting-vendored-vm` | `scripting` + `nuxie-scripting?/vendored-vm` |
| `nux-capi`, `nux-apple-product-extension` | unchanged | now the safe VM |
| `rust-golden-runner` | `scripting` / `scripting-vendored-vm` | variants `scripted` (**safe VM**, the Makefile default) / `scripted-vendored-vm` |
| `tools/bazel/generate.py` | every graph | the safe VM; `async` dropped from the browser graphs |

Code gates: `feature = "nuxie-vm"` became `not(feature = "vendored-vm")`
and `not(feature = "nuxie-vm")` became `feature = "vendored-vm"`. The
`nuxie-vm` and `scripting-nuxie-vm` features are gone. Every crate that
already said `luau` or `scripting` (`nuxie-runtime`'s dev-dependency,
`silver-corpus`, `nux-capi`, the Apple and Android graphs, the editor,
publisher and browser graphs) now gets the safe VM without a change.

## 2. Changes

Commits (Levi sole author, no trailer):

| Commit | What |
|---|---|
| `8c04b406fe` | the flip: features, gates, runner variants, generator, docs; `Chunk::eval` test-only; two audio probes moved off `luaur-rt` |
| `6baa254005` | bytecode v12/v13 rejected by the product validator on the safe VM, with a test |
| the docs commit after these | this document, the plan entry, the plan's abort-probe ratchet row |

Detail:

1. **Flip.** `crates/nuxie-scripting/Cargo.toml`, `crates/nuxie/Cargo.toml`,
   `tools/rust-golden-runner/Cargo.toml`, the mechanical gate swap in 24
   source files, `src/luau/mod.rs`'s header and `compile_error!`, and the
   comments that named `nuxie-vm`. `tools/golden-runner/rust_runner_provenance.py`:
   `scripted` is the safe VM, `scripted-nuxie-vm` became
   `scripted-vendored-vm` (`rust-golden-runner --features scripting-vendored-vm`).
   The Makefile's `SCRIPTED_RUST_RUNNER_VARIANT` default stays `scripted`,
   so `make scripted-golden-compare` and CI's `scripted-golden` lane
   (`.github/workflows/_trusted-macos.yml`, which runs that target) now
   compare the safe VM. Regenerated Bazel files (`generate.py --check` clean).
2. **Two dependency moves the device graph needed.** `luau-host` no longer
   lists `dep:luaur-vm`; it moved to `vendored-vm` (the seam doc's lane 8
   note proposed this). The compiler crates moved from the safe VM's
   feature to `compiler`: `tools/check-nux-capi-apple.sh` fails the Apple
   distribution if `luaur-ast`, `luaur-bytecode` or `luaur-compiler` are
   normal target dependencies of `apple-runtime`, and on `main` the safe
   VM's feature pulled all three in unconditionally. The crate's tests get
   the compiler from its dev-dependencies, as before.
3. **`async`.** luaur-rt's async API has no facade equivalent and no product
   caller (`tests/default_features.rs` is its only user, already
   vendored-only). It now implies `vendored-vm`, so it left the default set
   and the browser graphs; otherwise it would have pulled the vendored VM
   back in everywhere.
4. **Tools that named `luaur-rt` directly.** `tools/audio-device-validation`
   (`luau_lib.rs`, hand-written `BUILD.bazel`) and `tools/luau-audio-web`
   (`probe.rs`, `BUILD.bazel`) used `luaur_rt::{Function, Table}` with
   `ScriptVm`, which would no longer type-check on the default. They name
   the types through `nuxie_scripting::luau` and drop their `luaur-rt` deps.
   `tools/silver-corpus` already did (prep evidence, section 1). The VM
   harnesses (`luau-vm-diff`, `luau-conformance`, `luau-bench`,
   `luaur-rt-test-harness`) use `luaur-rt` on purpose, as the reference
   VM, and are unchanged.
5. **`Chunk::eval`.** Now `#[cfg(test)] pub(crate)`: only the crate's own
   unit tests call it. Its outside callers move to `into_function` and
   `call` (or `exec` for `eval::<()>`): `tests/upstream_async_work_host.rs`
   (the one lane 7 named), and also `tests/luaur_table_call_contract.rs`,
   `tests/generic_host_commands.rs`, `examples/safe_vm_abort_probe.rs`,
   `tools/silver-corpus/tests/upstream_scripting_renderer.rs` and the
   audio-device probe. Every moved chunk is a `return …` chunk or a
   statement, so the semantics are unchanged.
6. **Bytecode v12/v13.** `nuxie-luau-vm` exports its load range as
   `BYTECODE_VERSION_MIN`/`BYTECODE_VERSION_MAX` (3 and 11, the constants
   the loader already used). `validate_luau_bytecode` admits that range
   unless `vendored-vm` is on (then the vendored range, 12 and 13
   included, as before). A v12 or v13 chunk fails at load with
   `bytecode version 12 is outside supported range 3..=11 (v12, the cost
   model, is not run by the safe Luau VM)` (v13: double-precision vector
   constants). Test `safe_vm_rejects_bytecode_versions_twelve_and_thirteen`
   patches the version byte of the build-pinned promise library chunk.
7. **Docs.** `docs/PORTING.md` (library table row and the luaur note),
   `docs/luau-safe-vm-seam.md` (a Phase 3 note: "under `nuxie-vm`" now means
   the default), `docs/luau-safe-vm-plan.md` (dated entry). Dated evidence
   files keep the names they were written with. `docs/SIZE.md` untouched.

## 3. Battery

Host: the shared 18-core Mac, other sessions' lanes running. The 1-minute
load average is recorded before/after each step; it ran from 5.9 to 18.2.
Fixtures: `make fixtures` (`RIVE_RUNTIME_DIR` at the cd5351c3 pin) and the
untracked `crates/nuxie/tests/fixtures/` copied in. Builds went through
`tools/bazel/runtime.py`, `make` and the guarded runner targets; some ran on
the build pool, as this machine's existing build mode routes them (no build
setting was changed). Bazel test runs report every target to the end.

"Default" is the safe VM (`scripting`, `luau`); "opt-out" is `vendored-vm`
(`scripting-vendored-vm`, runner variant `scripted-vendored-vm`).

### Generation, VM crate, harnesses

| Command | Result | Load |
|---|---|---|
| `tools/bazel/generate.py --check` | clean | — |
| `make luau-vm-ratchets` | pass. `unsafe` sites unchanged (`nuxie-luau-vm` 0; 3,918 over 7 crates). Panic lints: `nuxie-luau-vm` 0; `luaur-rt` down 13 (see below); none up | 13.2 / 18.0 |
| ↳ `make luau-conformance` (inside the ratchets) | vendored VM: 30 of 30 required pass, 0 panics, PASS | (same) |
| `make luau-conformance-nuxie` | safe VM: 37 of 37 required pass, 0 panics, PASS | 7.3 / 5.9 |
| `make luau-vm-diff` | 409 runs, 0 disagreements (bytecode v7, v11) | 18.0 / 7.3 |
| `runtime.py test -p nuxie-luau-vm` | 76 passed (as on `main`) | 10.9 / 10.9 |
| `runtime.py clippy -p nuxie-luau-vm --lib` | pass | 9.2 / 9.2 |
| `runtime.py clippy -p nuxie-luau-vm --all-targets` | fails: `src/libs/integer.rs:309` (`arithmetic_side_effects` in its `#[cfg(test)]` module). That file is unchanged from `main`, as is the lint table, so the failure is on `main` too. Not this change; not fixed | 10.9 / 11.2 |

The `luaur-rt` panic-lint drop (`expect_used` 87 to 81,
`arithmetic_side_effects` 42 to 37, `unwrap_in_result` 26 to 24) comes from
the feature graph, not the code: the default graph no longer turns on
`luaur-rt/async`, so that code is not compiled for the ratchet's label.
The baseline was not updated (`--update` not run); the vendored crate is
due for deletion.

### `nuxie-scripting`

| Setting | Targets | Passed / failed / ignored | Load |
|---|---|---|---|
| default | 40 / 40 pass | 718 / 0 / 5 | 10.6 / 10.4 |
| opt-out (`--features vendored-vm`) | 40 / 40 pass | 705 / 0 / 5 | 10.4 / 10.9 |
| `--features async --test default_features` | 1 / 1 pass | 1 / 0 / 0 | 7.6 / 9.8 |

The 13-test difference by name: the default runs 13 facade unit tests
(`luau::facade::tests`, safe VM only) and the new
`safe_vm_rejects_bytecode_versions_twelve_and_thirteen`. The opt-out runs
one test the default does not: `script_vm_installs_the_compile_time_atom_resolver`.
`default_build_preserves_luaur_async_api` runs on neither (it needs
`async`), so it was run on its own, the third row.

### `nuxie` (`--features scripting,test-support` / `scripting-vendored-vm,test-support`)

| Setting | Targets pass / fail | Passed / failed / ignored | Load |
|---|---|---|---|
| default | 41 / 1 | 350 / 1 / 5 | 6.3 / 8.0 |
| opt-out | 41 / 1 | 350 / 1 / 5 | 8.0 / 7.3 |

The one failure on both:
`upstream_data_binding_images_direct::stateful_component_image_bind`, the
known red (its fixture has no scripts). Since the `pairs`-order PR, the
three safe-VM-only failures of the prep evidence are gone: the two settings
match test for test.

### `silver-corpus`

| Setting | Targets pass / fail | Passed / failed / ignored | Load |
|---|---|---|---|
| default | 21 / 1 | 255 / 2 / 4 | 7.3 / 13.2 |

The two failures are the known `wave_c2` reds:
`wave_c2_layout_020_aspect_ratio` (`addRawPath` point (266, 0) against
C++'s (142, 0)) and `aspect_ratio_fill_and_stretch_follow_both_animated_parent_axes`
(child 5 height 416 against 202). `tests/wave_c2.rs` names no script or VM
API; they are layout. `silver-corpus` has no opt-out setting of its own
(`runtime.py` takes no `dep/feature` roots), so it was not run on the opt-out.

### Goldens

| Run | Entries | Exact | Diverges | Failures | Load |
|---|---:|---:|---:|---|---|
| `make scripted-golden-compare` (default, safe VM) | 364 | 348 | 16 | `semantic_text_inference_lt1` | 5.9 / 8.0 |
| `… SCRIPTED_RUST_RUNNER_VARIANT=scripted-vendored-vm` | 364 | 348 | 16 | `semantic_text_inference_lt1` | 8.0 / 13.4 |
| `make golden-compare` (ordinary, no VM) | 364 | 356 | 8 | `semantic_text_inference_lt1` | 13.4 / 13.5 |

The two scripted runs' per-case reports
(`target/runtime-differentials/golden-scripted.json`) agree on every one
of the 364 rows (outcome and verification). The 16 diverges are the same
16 as the prep evidence. The runners are the provenance guard's rebuilds:
`rust-golden-runner-scripted` has 2,691 `nuxie_luau_vm` symbols and no
`luaur_vm` or `luaur_rt`; `rust-golden-runner-scripted-vendored-vm` has 936
`luaur_vm` and no `nuxie_luau_vm`. `semantic_text_inference_lt1` fails
on all three with the known message ("semantic comparison is now exact;
remove the reviewed signature explicitly").

### Abort-profile probe (`safe_vm_abort_probe`, default features)

| Build | rc | Sections | Binary | Load |
|---|---|---|---|---|
| `runtime.py run -p nuxie-scripting --example safe_vm_abort_probe` (`panic=unwind`) | 0 | 8 of 8 | `__gcc_except_tab` 84,640 B | 11.2 / 7.2 |
| same with `RUSTFLAGS=-Cpanic=abort` | 0 | 8 of 8 | `__gcc_except_tab` 4,956 B, names `library/panic_abort/src`; 1,889 `nuxie_luau_vm` symbols, 0 `luaur_vm` | 7.2 / 6.3 |

The probe now builds from the default features: no `--no-default-features`
is needed to keep the vendored VM out.

### Whole workspace and other graphs

| Command | Result | Load |
|---|---|---|
| `runtime.py check --all-targets` (every workspace package) | 531 targets, clean | 13.5 / 18.2 |
| `runtime.py check -p nuxie-scripting --no-default-features` | clean | 16.4 / 13.4 |
| `runtime.py check -p nuxie-scripting --features vendored-vm --all-targets` | clean | 17.4 / 16.4 |
| `bazel test //tools/audio-device-validation:luau_probe_test //tools/luau-audio-web:probe_native_test` | pass | 9.2 / 10.4 |
| `bazel build` wasm (`--platforms=@rules_rust//rust/platform:wasm`, `-Cpanic=abort`) `//tools/luau-audio-web:kernel :probe` | builds (browser graph, now the safe VM) | 9.5 / 11.5 |
| `//tools/luau-audio-web:transport_test` | fails locally, `node: command not found` in the sandbox; a JavaScript test the pool runs on Linux, untouched here | — |
| `make nux-capi-distribution-contract-test` | pass (unit, tooling and distribution checks) | — |
| `make rust-runner-provenance-test` | 16 tests OK | — |
| `cargo tree -p nux-apple-product-extension --features apple-runtime --target aarch64-apple-ios --edges normal` (the graph `tools/check-nux-capi-apple.sh` inspects) | Luau crates: `nuxie-luau-vm`, `luaur-common`; no `luaur-rt`, `luaur-vm`, `luaur-ast`, `luaur-bytecode`, `luaur-compiler` | — |
| same for `nux-capi` `android-vulkan,scripting,android-authored-wgsl` on `aarch64-linux-android` | same two | — |
| `tools/pure-runtime-boundary/check.py` | 129 findings; `origin/main` has 130. The one gone is `luau/facade/state.rs` `pub fn eval`; none added | — |

Disk: free space fell to 58 GB once, under the 60 GB stop line, after the
workspace check downloaded pool products for all 531 targets into this
worktree's `target/debug` (37 GB). The battery script stopped there.
Removing that regenerable output (the three golden runners kept) brought it
back to 95 GB, and the remaining steps ran. Lowest point otherwise: 92 GB.

Review: `codex review --base origin/main`, two rounds (after the code
commits, and after this document): no findings either time.

## 4. The shipped Apple device slice

Built as `tools/build-nux-capi-xcframeworks.sh` builds it, from committed
features only (`apple-runtime` names `nux-capi/scripting`, which is now the
safe VM):

```bash
python3 tools/bazel_runtime_build.py build --repo-root "$PWD" \
  --label //crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib \
  --target aarch64-apple-ios --output <dir>/libnux_apple_product_extension.a
llvm-objcopy --remove-section=__LLVM,__bitcode --remove-section=__LLVM,__cmdline <a>   # rustup 1.94.1
strip -S -x <a>
```

462 s, load 11.6 before and 9.4 after.

| `aarch64-apple-ios` slice | prep: vendored (`7144b028f3`) | prep: safe VM (scratch feature) | **this branch (committed)** | vs vendored |
|---|---:|---:|---:|---:|
| as built (`.a`) | 41,191,448 B | 41,622,648 B | 41,630,800 B | +439,352 B |
| shipped (objcopy + `strip -S -x`) | 30,293,008 B | 30,689,824 B | **30,695,960 B** | **+402,952 B (+1.33%)** |
| `__TEXT` summed over members | 20,133,082 B | 20,525,818 B | 20,531,342 B | +398,260 B |
| data summed over members | 1,896,425 B | 1,886,817 B | 1,886,817 B | -9,608 B |
| member objects | 368 | 368 | 369 | +1 |

Against the prep's safe-VM slice it is 6,136 B and one member larger. The
base moved in between: the prep measured before the `pairs`-order work in
`nuxie-luau-vm` landed. The difference was not traced further.

Symbols in the as-built archive (`nm`, names containing each crate):
`nuxie_luau_vm` 712, `nuxie_scripting` 1,993, `luaur_common` 4, and
`luaur_vm`, `luaur_rt`, `luaur_compiler`, `luaur_ast`, `luaur_bytecode`
all 0.

Budgets: `crates/nux-capi/size-budgets-v3.json` caps the `aarch64-apple-ios`
slice at 49,283,072 B and the linked representative iOS consumers at
30,408,704 B. The two iOS C consumers that `tools/verify-nux-capi-xcframeworks.sh`
links (`product_extension_consumer.c`, `capi_metal_smoke.c`), linked here
against this slice with the same flags, are 22,705,720 B and 22,724,184 B.
No committed size budget or symbol contract names the VM or needs
updating. No recorded number was changed.

## 5. Vendored-only tests (gated to the opt-out)

Whole tests:

- `nuxie-scripting` `vm::context_init_tests::script_vm_installs_the_compile_time_atom_resolver`
  (the safe VM has no atom hook, seam row R4).
- `nuxie-scripting` `tests/default_features.rs` `default_build_preserves_luaur_async_api`
  (needs `async`, which now implies `vendored-vm`).
- `tests/upstream_scripting_vector.rs` `two_component_vectors_always_have_z_zero`
  has a vendored body and a safe-VM body of the same name.

Assertions inside shared tests (the test runs on both; the assertion only on
the opt-out). All read the raw Lua stack top or the vendored thread data,
which the facade has no API for (prep evidence, section 1):
`src/vm/property_lifetime_a637_tests.rs`, `src/vm/view_model.rs`,
`src/vm/lua_path.rs`, `src/vm/lua_data_value.rs`,
`tests/upstream_scripting_converter.rs`, `tests/upstream_module_hooks.rs`,
and `ScriptVm::upstream_test_stack_top` (returns `None` on the safe VM; used
by `tools/silver-corpus`).

## 6. Not verified

- **The full XCFramework.** Only the `aarch64-apple-ios` slice was built.
  The simulator and macOS slices, the packaging, `tools/check-nux-capi-apple.sh`
  and `tools/verify-nux-capi-xcframeworks.sh` end to end were not run. The
  Swift consumers were not linked.
- **Android.** Its target graph was inspected (`cargo tree`), not built.
- **The browser runtime.** The wasm kernel and probe build; nothing ran
  them in a browser, and `transport_test` (Node) did not run here.
- **`silver-corpus` on the opt-out**: no setting for it (section 3).
- **`nux-capi`'s own tests** were checked (`runtime.py check --all-targets`),
  not run.
- **Repeatability.** Each golden and test setting ran once.

## 7. What still depends on the vendored VM

- The `vendored-vm` opt-out itself: `nuxie-scripting` (`luaur-rt`,
  `luaur-vm`), `nuxie`'s `scripting-vendored-vm`, the golden runner's
  `scripting-vendored-vm` and its `scripted-vendored-vm` variant, and the
  code behind `cfg(feature = "vendored-vm")` (section 5, the raw audio
  processor `vm/luau_audio_processor_engine/raw.rs`, the raw C API sites).
- `nuxie-scripting`'s `async` feature (luaur-rt's async API).
- The reference harnesses, on purpose: `tools/luau-vm-diff` (the
  differential's other side), `tools/luau-conformance` (`make
  luau-conformance` scores the vendored VM), `tools/luau-bench` (the
  baseline), `tools/luaur-rt-test-harness` (the original abort probe), and
  the ratchets that count the vendored crates.
- Not the VM, kept: `luaur-common` (opcode tables in the product
  validator, and the build script), and `luaur-compiler`/`luaur-ast`/`luaur-bytecode`
  (the tooling compiler: build script, `compiler` feature, tests).

Deleting the vendored VM is a separate PR that awaits Levi's approval.
