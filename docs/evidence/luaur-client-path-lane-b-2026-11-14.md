# Luau on clients, lane B: `luaur-common` leaves the device graph (2026-11-14)

Design: `docs/luaur-client-path-safe.md`, section 4 (lane B) and the ratchet
step of section 6. Facts it builds on:
`docs/evidence/luaur-client-path-survey-2026-11-14.md` (cited as S).
Base: `7fd7c6f5f8` (`levi/luaur-client-safe`). Host: macOS arm64; load
averages are recorded beside each run (the host was busy with other lanes
throughout; nothing waited for a quiet host).

## 1. What changed

| Commit | Change |
|---|---|
| `5d1361fbbb` ratchets | `crates/nuxie-scripting` added to `tests/ratchets/unsafe-baseline.txt` (52) and `panic-lints-baseline.txt` (481 on the default label). `tools/ratchets/ratchets.py` takes `CRATE_DIR:TARGET@PLATFORM` as a panic-lint row: a named Bazel variant built for a platform listed in `PLATFORMS` (today `wasm`: the kernel's `--platforms=@rules_rust//rust/platform:wasm` and `-Cpanic=abort`, as `tools/luau-audio-web/pool.py build` passes them). One Bazel build per platform; diagnostics are keyed by target, so two variants of one package no longer collide. `crates/nuxie-scripting:nuxie-scripting__publisher@wasm` is its own row (446). `make luau-vm-ratchets` runs every baseline row, so it runs this one. New tool test: a variant row is linted on its platform in its own build and fails on its own count. |
| `0ba2e38671` VM | `nuxie_luau_vm::bytecode_format::{Opcode, op, tag}`: the `op` numbers (now `pub`), `op::COUNT`, `Opcode::new(u8) -> Option<Opcode>` (the checked conversion), `Opcode::{number, length, name}` and a `Debug` that prints the C++ enumerator name; `tag::{TYPE_VERSION_MIN, TYPE_VERSION_MAX, CONSTANT_*, TYPE_FUNCTION, FEEDBACK_CALL_TARGET}`. The loader reads its tags from these constants instead of private copies and literals. No dependency added; 0 `unsafe`, 0 forced panic-lint sites. |
| `9d0ca34622` validator | `crates/nuxie-scripting/src/vm/bytecode.rs` takes every number from `bytecode_format`; the op byte goes through `Opcode::new` (no `transmute`). The `FFlag::LuauCallFeedback` check is gone: v11 is admitted, as it was in every device configuration. `Lua::new()` no longer calls `set_all_flags(true)`; `validate_executable_luau_bytecode` no longer enters `ScopedAllFlags`. `compile()` (`compiler` feature), `build.rs` and the tests keep setting flags where they compile. |
| `89a8b4eb31` graph | `luaur-common` is enabled by `compiler` (not `luau-host`), stays a build-dependency, and is a dev-dependency for the tests that set flags before compiling. A `compiler`-feature test, `vm::bytecode::compiler_table_tests`, compares every byte 0..=255: opcode existence, number, name and length, then every constant tag, the type-version range, `LBC_TYPE_FUNCTION` and `LFT_CALLTARGET`, against `luaur-common`. `tools/check-nux-capi-apple.sh` rejects `luaur-common` in the Apple device `cargo tree`. Bazel files regenerated; both crate universes repinned (`CARGO_BAZEL_REPIN=1`, `CARGO_BAZEL_REPIN_ONLY` each in turn): the lock files did not change, because path crates are not in them. |

Behaviour a script or host observes: unchanged on every device path. The
validator's error texts are the same strings (the opcode name in
`instruction N (LOP_X) extends past code stream` now comes from the VM's
name table, which the drift test pins to `luaur-common`'s `Debug` output).
One host-only difference, by design (section 4 of the design): a caller that
validates v11 bytecode inside its own `ScopedAllFlags::enter(false)` on the
same thread was rejected before (`bytecode v11 requires LuauCallFeedback
flag`) and is accepted now, as the VM accepts it. No device configuration
had the flag off; the only code that did this was the test
`executable_validator_scopes_runtime_flags_without_perturbing_the_caller`,
now `executable_validator_reads_no_flag_and_leaves_the_callers_profile`.

## 2. The device graph after

Command (S1.3): `bazelisk --nosystem_rc --nohome_rc query "kind(rule, deps(<LABEL>)) intersect attr(name, 'luaur|luau-vm|nuxie-scripting', //vendor/... + //crates/...)"`.

| Shipped label | Luau crates in the target configuration | Exec configuration (build script) |
|---|---|---|
| `//crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib` | `nuxie-scripting__apple`, `nuxie-luau-vm__apple` | `luaur-{common,compiler,bytecode,ast}__host` |
| `//crates/nux-capi:nux-capi__android__cdylib` | `nuxie-scripting__android`, `nuxie-luau-vm__android` | same |
| `//tools/luau-audio-web:kernel` | `nuxie-scripting__publisher`, `nuxie-luau-vm__publisher` | same |

No `luaur-common__apple`, `__android` or `__publisher`. `cargo tree --edges
normal` for `nux-apple-product-extension --features apple-runtime` on
`aarch64-apple-ios` and for `nux-capi` with the Android features on
`aarch64-linux-android` lists no `luaur-*` crate.

The generator now emits `luaur-common__{apple,android,webgpu,webgl2}` with
no features (nothing in a device graph enables them). `__webgl2` is still
linked by the browser qualification probe (`tools/luau-audio-web:probe`,
not shipped), which now builds it without `std`; in `luaur-common` `std`
only selects whether the assertion handler prints (`functions/assert_call_handler.rs:24`).

## 3. Product validator against the VM loader (B.5, open question 2)

Report only; nothing was merged. Every device path runs the validator first
(`ScriptVm::load_bytecode`, `register_audio_processor_module`; `vm.rs:2642,2741`)
and then the VM loader, so a chunk must pass both. Bold marks a check only
one side makes.

| Check | Product validator (`nuxie-scripting/src/vm/bytecode.rs`) | VM loader + verifier (`nuxie-luau-vm/src/bytecode/{load,verify}.rs`) |
|---|---|---|
| Chunk size cap | no | yes (`Limits::max_bytecode_bytes`, `TooLarge`) |
| Version 0 (compiler diagnostic) | rejected | rejected (`CompileError`, with the message) |
| Version range 3..=11 | yes (same constants) | yes |
| Type-info version 1..=3 when version >= 4 | yes | yes |
| Every count fits in the bytes left | yes (`count × minimum width`) | yes (`count` ≤ bytes left) |
| Varint overflow (u32 at 5 bytes, u64 at 10) | yes | yes |
| String ids in range; id 0 rejected where a string is required | yes | yes |
| Userdata type remap (type-info v3) names in range | yes | yes |
| At least one proto | yes (`proto table is empty`) | no explicit check; the main-proto lookup then fails |
| v1 type-info: size = 2 + numparams, header `LBC_TYPE_FUNCTION`, numparams | **yes, validator only** | no (skips the blob) |
| Constant tags known | yes | yes |
| Class-shape constants | parsed and checked (class/member names are earlier string constants) | **rejected outright** (`Unsupported`) |
| Double-precision vector constant (tag 11) | rejected (`unknown constant tag`) | rejected (`Unsupported`, v13) |
| Table / table-with-constants keys and values index constants | keys: `< sizek`; values: `< sizek` | keys and values: an **earlier** constant (`k.get` on the constants loaded so far) |
| Table template keys are valid table keys (not nil/NaN) | no | **yes** (`bad table template key`) |
| Import constant ids name earlier constants | no (skips 4 bytes) | **yes** |
| Closure constants and child protos name an earlier proto | yes | yes |
| Opcode known (< 90) | yes | yes, and stricter: `BREAK`, `NATIVECALL`, class opcodes, `CMPPROTO` rejected |
| Instruction plus AUX inside the code | yes | yes |
| `GETTABLEKS`/`SETTABLEKS`/`NAMECALL` AUX key is a string constant | yes | yes |
| Register, upvalue, constant-kind, child-proto operands, jump targets, `CAPTURE` placement, `NAMECALL` followed by a call, `numparams ≤ maxstacksize` | no | **yes** (verifier) |
| Line info: `linegaplog2` bound | `< usize::BITS` (64) | `< 32` |
| Debug info: upvalue name count = nups | yes | yes |
| v11 feedback slots: kind `LFT_CALLTARGET` | yes | yes |
| Main proto id in range | yes | yes |
| Main proto has no upvalues | no | **yes** |
| No trailing bytes after the chunk | no | **yes** |
| Memory: allocations charged to the VM budget | n/a (allocates `code`/`constants` vectors sized by counts already bounded by the input length) | yes |

Answer: **the validator is not a strict subset of the loader.** The loader
and verifier check far more (every operand, jump target, `CAPTURE`
placement, trailing bytes, main-proto upvalues, earlier-constant references
for imports and table templates, a tighter `linegaplog2` bound) and refuse
whole features the validator parses (class-shape constants, the class
opcodes, `BREAK`, `NATIVECALL`). The validator adds only three things the
loader does not check: (1) a non-empty proto table, which the loader still
rejects, as "main proto out of range"; (2) the v1 type-info header
(size `2 + numparams`, first byte `LBC_TYPE_FUNCTION`, second byte
`numparams`), which the loader skips because this VM does not read type
info; (3) its own wording. So on the device the only bytecode the
validator rejects and the loader would admit is v1 type info with a
malformed header, which the VM never reads. The other effect of the
validator is the error text: a host sees
`ScriptAsset '<name>': malformed Luau bytecode: <validator message>`
instead of the VM's `LoadError` text, so dropping it would change
observable error strings. There are no version or signature rules in the
validator: versions are the VM's own constants, and signatures are checked
elsewhere (`nuxie-script-signature`).


## 4. Can two `ScriptVm::new()` run on different threads on a device? (open question 1)

As far as the code shows: **yes, the C ABI allows it, so the race was live
by contract.** Whether any shipped app does it was not established.

- The only non-test `ScriptVm` constructor on a device path is
  `nuxie::import_scripted` (`crates/nuxie/src/script_import.rs:78`,
  `ScriptVm::new_with_execution_limits`, which reaches `Lua::new()`). Lines
  793 and 815 are in `#[cfg(test)]` code.
- Its one caller is `nux-capi`'s `import_file_with_prepared_host_commands`
  (`crates/nux-capi/src/lib.rs:2292`, call at 2366), reached from the
  `extern "C"` import entry points: `nux_file_import_trusted_with_host_commands`
  (`lib.rs:2420`), the configured import in `asset_hooks.rs:1022`, and the
  Android import (`android_product_import.rs:353`). Each runs on the
  calling thread inside `ffi_guard_*`; none takes a process-wide lock and
  none checks the thread (there is no handle yet to be affine to).
- The ABI contract (`crates/nux-capi/include/nux_capi.h`, rule 4) makes
  each handle affine to the thread that created it and says nothing that
  confines imports to one thread. Two host threads importing two scripted
  files at once therefore each ran `set_all_flags(true)`: unsynchronized
  writes to the same `static` flags (S, surprise 1). The values written were
  identical, so the effect would have been a data race without a visible
  difference, but it was undefined behaviour all the same.
- `ScriptVm` itself is `!Send` (it holds `Rc`), so one VM never moves
  between threads; audio processor VMs never called `Lua::new()` (S2.1).

After lane B the question is moot for the device: `Lua::new()` writes no
flag and nothing on the device path reads one (`luaur-common` is not
linked). Flag writes remain only where compiling happens (build script,
`compile()` under `compiler`, tests, tools), all on build hosts.


## 5. Checks

All on the branch tip before this document's commit (`89a8b4eb31`), with
`RIVE_RUNTIME_DIR` at the `cd5351c3` pin. Builds went through
`tools/bazel/runtime.py` / `make`, which routed some to the pool.

| Check | Result | Load average (start → end) |
|---|---|---|
| `bazelisk query` (S1.3) on the three shipped labels | no `luaur-common__{apple,android,publisher}`; `luaur-*__host` build-script deps remain (section 2) | — |
| `python3 tools/bazel/generate.py --check` | clean (exit 0) | — |
| `python3 tools/ratchets/test_ratchets.py` | 5 passed (4 existing, 1 new) | — |
| `make luau-vm-ratchets` | exit 0: unsafe PASS (`nuxie-scripting` 52, VM 0); panic-lints PASS, every row `+0` (VM 0, `nuxie-scripting` 481, `…__publisher@wasm` 446); conformance 55 scripts, 37 required all OK, 0 panics, PASS; 7 m 03 s | 18.6 → 12.2 |
| `make luau-vm-diff` | 409 runs, 0 disagreements, bytecode versions 7 and 11, every admitted opcode reached | 11.3 → 13.9 |
| `runtime.py clippy -p nuxie-luau-vm --all-targets` | exit 0, no warning | 9.8 → 9.9 |
| `runtime.py test -p nuxie-luau-vm` | 76 passed | 9.9 |
| `runtime.py test -p nuxie-scripting` | 39/39 targets, 720 tests passed, 5 ignored; includes `compiler_table_tests` (2) and `executable_validator_reads_no_flag_and_leaves_the_callers_profile` | 9.2 → 18.6 |
| `runtime.py check -p nuxie-scripting --no-default-features` | exit 0; warnings only in `nuxie-runtime`, `nuxie-ore-metal`, `jpeg-decoder`, `nuxie-binary`, `nuxie-audio` (none in `nuxie-scripting`) | 13.4 → 11.4 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41/42 targets pass; the one failure is `upstream_data_binding_images_direct::stateful_component_image_bind`, known red on main | 15.1 → 17.1 |
| `runtime.py test --locked -p nux-capi --no-default-features` | first run 24/25: `list_acquire_guards_provenance_and_no_mutation` got `WrongThread` for a freed handle where it expects `HandleMismatch` (`list_acquire_tests.rs:577`); rerun 25/25. The handle registry is keyed by address and the unit tests run in parallel, so a freed address can be reused by another test's handle owned by another thread; no Luau code is involved. Not investigated further. | 14.0 → 14.5 |
| `make scripted-golden-compare` | 364 entries: 348 exact, 16 diverge, exit 0 | 16.3 → 11.3 |
| `tools/luau-audio-web/pool.py build --release` (base and tip) | both built (the first base attempt hit a pool `Remote Execution Failure`, exit 34, and was rerun) | 11.4 → 9.4 |
| `tools/luau-audio-web/pool.py test` (probe native test, transport tests) | **not run**: it exits before building, in the homelab input resolver (`~/.local/share/nuxie-homelab/runtime_inputs.py`, `pinned_checkout`), which ignores the exported `RIVE_RUNTIME_DIR` for this command, falls back to `/Users/levi/dev/oss/rive-runtime` and picks its worktree entry for the pin at `/private/tmp/rive-upstream-oct30-accounting`, which no longer exists (`git rev-parse` exit 128). Host state outside this lane; left as found | 10.5 |
| `codex review --base levi/luaur-client-safe` | round 1: no findings | — |


## 6. Sizes

**Browser audio kernel** (`nuxie_luau_audio_kernel.wasm`, pool, `--release`):

| | base `7fd7c6f5f8` | tip | difference |
|---|---:|---:|---:|
| bytes | 976,869 | 973,275 | −3,594 |
| exports | 23 | 8 | −15 |
| imports | `nuxie_audio.{host_call, now_ms, shared_raw}` | same | none |

The 15 exports that went are `luaur-common`'s `wasm_libc.rs` and
`strtod_shim` symbols: `malloc`, `free`, `realloc`, `strchr`, `strtod`,
`strtoul`, `strtoull`, `time`, `clock`, `sysconf`, `mmap`, `munmap`,
`mprotect`, `gmtime_r`, `localtime_r`. The tip exports `memory`,
`__data_end`, `__heap_base` and the five `nuxie_audio_*` entry points.
Nothing needed the shims: the module imports nothing but `nuxie_audio.*`
(so no wasm code called out to a libc), and the JS host
(`tools/luau-audio-web/worklet.mjs`) calls only `nuxie_audio_alloc`,
`nuxie_audio_request`, `nuxie_audio_response_len`, `nuxie_audio_render`,
`nuxie_audio_free` and reads `memory`. Exports and imports read from the
wasm export and import sections with a scratch parser.

**`aarch64-apple-ios` slice**, built at the base (a detached worktree) and at
the tip the way `tools/build-nux-capi-xcframeworks.sh` does
(`tools/bazel_runtime_build.py build` of
`//crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib`,
pinned `llvm-objcopy`, `strip -S -x`), with identical fixed `NUX_RUNTIME_*`
provenance values on both sides:

| | base `7fd7c6f5f8` | tip | difference |
|---|---:|---:|---:|
| as built (`.a`) | 41,630,832 B | 41,629,576 B | −1,256 |
| shipped | 30,695,872 B | 30,695,312 B | −560 |
| members | 369 | 369 | 0 |
| `nm` symbols matching `luaur_common` | 4 | 0 | −4 |
| `nm` symbols matching `nuxie_luau_vm` | 712 | 713 | +1 |

Against the deletion evidence's 30,696,000 B (built with real provenance
strings, so not byte-comparable): 688 B under. The tip slice took 8 m 38 s
(load 10.2 → 12.6), the base 6 m 48 s (9.6 → 7.2). Android was not measured.


## 7. What was not verified

- The Android AAR's size, and the `aarch64-linux-android` build itself: only
  its Bazel graph (section 2) and `cargo tree` were checked.
- `tools/check-nux-capi-apple.sh` was extended but not run end to end (it
  builds and links every Apple slice with Xcode); the new rule's pattern was
  checked against `cargo tree` output for `aarch64-apple-ios`, which lists no
  `luaur-common`.
- The kernel's probe and transport tests (`pool.py test`) and the kernel in
  a browser: neither ran (section 5 says why the pool test did not start).
  What was checked is that the kernel builds and that its import and export
  sections no longer carry the shims (section 6).
- Whether any shipped app imports scripted files on two threads at once
  (section 4 answers what the ABI allows, not what apps do).
- The `nux-capi` handle-registry flake (section 5) was seen once and not
  chased.


## 8. Where the design was wrong or incomplete

1. Section 4.5 asks whether the validator is "a strict subset plus the
   version and signature rules". It is neither: it is mostly a subset, but
   it checks v1 type-info headers that the loader skips, it parses
   class-shape constants the loader rejects outright, its `linegaplog2`
   bound is looser, and it has no version rule of its own (it uses the VM's)
   and no signature rule at all. Its one observable role on the device is
   its error text (section 3).
2. The design says the VM "already has" the tables. It had the opcode
   numbers and the AUX rule as `pub(crate)` items, the constant tags as
   private constants in `load.rs`, and the type-version range and feedback
   kind only as literals (`1..=3`, `0`). It had no opcode names and no
   `LBC_TYPE_FUNCTION`. Exporting them needed new public items (an opcode
   name table and a checked `Opcode` type) but no dependency, and the
   validator's error text needed the names to stay the same.
3. "A dependency of the `compiler` feature and a build-dependency only"
   was not enough: the crate's integration tests (`tests/*.rs`) and the
   test modules in lane A's files call `luaur_common::set_all_flags` and
   are built without `compiler` in some configurations, so `luaur-common`
   is also a dev-dependency. Dev-dependencies never reach a device graph.
4. Moving `luaur-common` out of `luau-host` changes the generated variants
   `luaur-common__{apple,android,webgpu,webgl2}` to no features; the
   browser qualification probe still links `__webgl2` directly. Harmless
   (section 2), but outside the three shipped labels the design checked.
5. Open question 1's hazard was reachable through the public C ABI, not
   only in theory (section 4).

