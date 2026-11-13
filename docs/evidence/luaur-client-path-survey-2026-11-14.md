# luaur client-path survey, 2026-11-14

Facts for the design that makes the client-shipped luaur code memory-safe and
panic-free. No source was changed. Tree: `main` at `11fc709367` (PR 1419, the
vendored VM deleted). Host: macOS arm64, load average recorded beside each
measurement.

Scratch helpers referenced below (not committed, session scratchpad):
`lint.sh` (the panic-lint tool's exact Bazel flags, on explicit variant
labels), `parse.py` (counts diagnostics the way `tools/ratchets/ratchets.py`
does: one per diagnostic whose code is one of the seven lints, keyed by primary
span), `classify.py` (heuristic shipping/non-shipping line classifier for
`crates/nuxie-scripting/src`, explained in section 3.3). Their logic is
described where used so the numbers can be reproduced.

## Main surprises

1. **`Lua::new()` writes about 120 process-global flags through `UnsafeCell`
   on every VM creation** (`crates/nuxie-scripting/src/luau/facade/state.rs:302`
   calls `luaur_common::set_all_flags(true)`), with no synchronization; the
   statics are `unsafe impl Sync`. It is sound only while every `Lua::new()`
   and every flag read happen on one thread or are otherwise ordered. Audio
   processor VMs do **not** go through `Lua::new()` (they build
   `nuxie_luau_vm` VMs directly, `luau_audio_processor_engine/safe.rs:15,73`),
   so the known second thread does not race; whether two `ScriptVm::new()`
   calls (`crates/nuxie/src/script_import.rs:78,793,815`) can run concurrently
   on a device was not established. Only one flag is read on the device
   path at all: `FFlag::LuauCallFeedback`, by the bytecode validator
   (`crates/nuxie-scripting/src/vm/bytecode.rs:273`). The safe VM reads none
   (`crates/nuxie-luau-vm` has an empty `[dependencies]` table).
2. **The device path uses a tiny slice of `luaur-common`** (four enums,
   `getOpLength`, and the flag machinery) and **that slice has 0 forced
   panic-lint sites**; the 288 panic-lint and 174 `unsafe` sites of the crate
   live almost entirely in code only the compiler crates use. Dropping the
   dependency for a small safe module looks cheap (section 2).
3. **Panic containment depends on `panic = "unwind"` in two places that are not
   the C boundary**: the Luau facade trampoline catches panics in every host
   callback and turns them into Luau errors
   (`crates/nuxie-scripting/src/luau/facade/state.rs:917`), and the audio
   render loop catches per block (`.../luau_audio_processor_engine/safe.rs:380`).
   Under `panic = "abort"` both become process aborts.
4. **Two hostile-input crash paths that no `catch_unwind` can contain**:
   recursion with no depth bound over script-built tables when posting audio
   messages (`crates/nuxie-scripting/src/vm/lua_audio_processors.rs:656`
   `to_shared`, cycle check only; a deeply nested table overflows the native
   stack), and host-side allocations sized by script outside the 16 MiB VM
   ceiling (`ImageMeshInstances:resize(count: u32)` at
   `crates/nuxie-scripting/src/vm/lua_mesh.rs:197`, allocation failure aborts).
5. The panic-lint tool's view of `nuxie-scripting` barely depends on features:
   481 sites in the Apple set, 480 Android, 481 default, 446 wasm kernel.
   `compiler`-only non-test code contributes **1** site; the rest is device
   code.
6. The compiler crates are **in the shipped Bazel graphs**, but only in the
   exec configuration as build-script dependencies (`luaur-*__host`); they are
   not linked into any artifact.

---

## 1. The device graph

### 1.1 Shipped configurations

| Artifact | Bazel label | Source of the label |
|---|---|---|
| Apple xcframework | `//crates/nux-apple-product-extension:nux-apple-product-extension__apple__staticlib`, `crate_features = ["apple-runtime"]` (`crates/nux-apple-product-extension/BUILD.bazel:451-458`) | `tools/build-nux-capi-xcframeworks.sh --plan` |
| Android AAR | `//crates/nux-capi:nux-capi__android__cdylib`, `crate_features = ["android-authored-wgsl", "android-vulkan", "scripting"]` (`crates/nux-capi/BUILD.bazel:990-997`) | `tools/build-nux-capi-android.sh --plan` (what `make nux-capi-android` runs) |
| Browser audio kernel | `//tools/luau-audio-web:kernel` (rust_shared_library, deps `//crates/nuxie-scripting:nuxie-scripting__publisher`) | `tools/luau-audio-web/BUILD.bazel`; built by `tools/luau-audio-web/pool.py build` |

`apple-runtime` = `nux-capi/apple-authored-msl`, `nux-capi/apple-metal`,
`nux-capi/scripting` (`crates/nux-apple-product-extension/Cargo.toml`).

### 1.2 `nuxie-scripting` features per shipped variant

From `crates/nuxie-scripting/BUILD.bazel` (`grep -n -A12 'name = "nuxie-scripting__<v>",'`):

| Variant (artifact) | Features on | Off (notably) |
|---|---|---|
| `nuxie-scripting__apple` (Apple) | `apple-authored-msl`, `luau`, `luau-host` | `compiler`, `js-host-seed`, `upstream-test-seams`, `scriptnet`, `tools`, `browser-audio-host` |
| `nuxie-scripting__android` (Android) | `android-authored-wgsl`, `luau`, `luau-host` | same |
| `nuxie-scripting__publisher` (wasm kernel) | `browser-audio-host`, `luau`, `luau-host` | same |
| default (`cargo test`, tooling; not shipped) | `luau`, `compiler`, `js-host-seed` (+ implied `luau-host`) | |

The build script variants (`nuxie-scripting__{apple,android,publisher}__build_script`)
carry the same features and depend on `//vendor/luaur-common-0.1.8:luaur-common__host`
and `//vendor/luaur-compiler-0.1.8:luaur-compiler__host`.

### 1.3 Luau-side crates in each graph

Command (query only, no build):

```
bazelisk --nosystem_rc --nohome_rc query \
  "kind(rule, deps(<LABEL>)) intersect attr(name, 'luaur|luau-vm|nuxie-scripting', //vendor/... + //crates/...)"
bazelisk --nosystem_rc --nohome_rc query \
  "somepath(<LABEL>, //vendor/luaur-compiler-0.1.8:luaur-compiler__host)"
```

Result, identical shape for all three artifacts (suffix `apple`/`android`/`publisher`):

- linked (target configuration): `nuxie-scripting__<v>`, `nuxie-luau-vm__<v>`,
  `luaur-common__<v>`;
- exec configuration only (build script): `luaur-common__host`,
  `luaur-compiler__host`, `luaur-bytecode__host`, `luaur-ast__host`.

Path to the compiler (Apple): `nux-apple-product-extension__apple__staticlib →
nux-capi__apple → nuxie__apple → nuxie-scripting__apple →
nuxie-scripting__apple__build_script → luaur-compiler__host`. Android is the same
through `nux-capi__android__cdylib → nuxie__android`; the kernel goes straight
`kernel → nuxie-scripting__publisher → …__build_script`.

**Claim confirmed for what is linked**: only `luaur-common`, `nuxie-scripting`
and `nuxie-luau-vm` are linked into the shipped artifacts from the Luau side.
**Refined**: the compiler crates are in the build graph of every shipped
artifact, as host tools.

Cargo agrees: `nux-capi`, `nuxie-runtime` and `nuxie` list `luaur-common` and
`luaur-compiler` only under `[dev-dependencies]`; `nuxie-scripting` has
`luaur-common` as an optional normal dependency (enabled by `luau-host`) and
`luaur-common` + `luaur-compiler` as `[build-dependencies]`
(`crates/nuxie-scripting/Cargo.toml:57-85`). `nuxie-luau-vm` has no
dependencies at all.

### 1.4 `nuxie-scripting` modules in the device graph

`crates/nuxie-scripting/src/lib.rs`: `envelope`, `logging_scripting_context`
always; `gpu_canvas` (+ `gpu_canvas_ore/{canvas,enums,pass,pipeline,resources,shader,target}`),
`host_commands`, `shader_asset`, `luau` (facade: `conversion`, `error`,
`handles`, `multi`, `state`, `userdata`, `value`), `vm` under `luau-host`.

`crates/nuxie-scripting/src/vm.rs:14-85`, compiled in every shipped variant:
`buffer_ext, bytecode, command_server, embedded_chunk, host_dispatch,
listener_invocation, logging_scripting_context, lua_audio_samples, lua_blob,
lua_canvas, lua_color, lua_data_value, lua_font, lua_image, lua_image_decode,
lua_input, lua_main_ref, lua_mat4 (+ lua_mat4_safe), lua_math, lua_mesh,
lua_promise, lua_rive_base, lua_scripted_context, lua_text, lua_vec2d,
luau_audio_samples, native_registration, renderer, lua_artboards, lua_audio,
lua_audio_processors, lua_mat2d, lua_paint, lua_path, lua_renderer,
lua_renderer_library, lua_transition, lua_video, luau_audio_processor_engine
(+ safe), resource_limits, view_model`.

Target-gated: `luau_audio_shared` (native: `luau_audio_shared.rs`; wasm32:
`luau_audio_shared_web.rs`; both `include!` `luau_audio_shared_codec.rs`),
`luau_audio_web` and `luau_audio_processor_engine/browser_kernel` (wasm32 only,
so only the kernel).

Not in any shipped variant: `lua_rive_file`, `lua_scriptnet` (`scriptnet`),
the `upstream_scripting_*` and `*_tests` modules (`test` and/or `compiler`).

### 1.5 Build time versus device

`crates/nuxie-scripting/build.rs` (85 lines) runs **on the build host**, once
per variant. It reads five Luau sources embedded as raw-string constants
(`PROMISE_LIBRARY` in `src/vm/lua_promise.rs`,
`DATA_VALUE_METATABLE_PATCHER_SOURCE` in `src/vm/lua_data_value.rs`,
`PROPERTY_METATABLE_PATCHER_SOURCE` in `src/vm/view_model.rs`,
`HOST_DISPATCH_SOURCE` in `src/vm/host_dispatch.rs`,
`VECTOR_CONSTRUCTORS_SOURCE` in `src/vm/lua_vec2d.rs`), calls
`luaur_common::set_all_flags(true)` and the compiler's C entry point
`luaur_compiler::functions::luau_compile::luau_compile` (raw pointer in, malloc
buffer out, read with `slice::from_raw_parts`, released with libc `free`), and
writes five `*.luau-bytecode` files to `OUT_DIR`. The device gets those bytes
as `include_bytes!` data and loads them through the safe VM like any other
chunk. No compiler code, and no `luaur-common` code from the build script, runs
on the device. There is no proc-macro dependency on any luaur crate.

## 2. `luaur-common` on the client path

### 2.1 Every non-test, device-graph use

Command: `grep -rn "luaur_common" crates/{nuxie-scripting,nux-capi,nuxie-runtime,nuxie,nuxie-luau-vm,nux-apple-product-extension,nuxie-project-data-scripting} tools/luau-audio-web | grep "\.rs:"`
then each hit checked for its `cfg` gate (test modules, `cfg(feature = "compiler")`, `tests/`).

`nux-capi`, `nuxie-runtime`, `nuxie`, `nuxie-luau-vm`, the product extension
and the kernel: **no non-test use**. (`nux-capi/src/android_product_import.rs:92`
is inside `#[cfg(all(test, feature = "android-authored-wgsl"))] mod tests`.)

`nuxie-scripting`, device graph:

| Site | Item | What it is | Size | `unsafe` / global state in `luaur-common` |
|---|---|---|---|---|
| `src/vm/bytecode.rs:5-15` | `enums::luau_bytecode_tag::{LBC_CONSTANT_*, LBC_TYPE_VERSION_MIN, LBC_TYPE_VERSION_MAX}` | newtype `u8` constants for constant tags and type-version range | `src/enums/luau_bytecode_tag.rs`, 68 lines | none |
| `src/vm/bytecode.rs:12` | `enums::luau_bytecode_type::LBC_TYPE_FUNCTION` | type-info tag | 62 lines | none |
| `src/vm/bytecode.rs:13` | `enums::luau_feedback_type::LuauFeedbackType` | v11 feedback slot kind (only `LFT_CALLTARGET` used) | 18 lines | none |
| `src/vm/bytecode.rs:14`, used `:394-409` | `enums::luau_opcode::LuauOpcode` (+ `LOP__COUNT`) | the opcode enum, `repr(u8)` | 199 lines | **1 `unsafe`**: `impl From<u8>` is `mem::transmute` (`src/enums/luau_opcode.rs:192-198`); the validator guards `opcode_byte < LOP__COUNT` first (`bytecode.rs:394`), so sound as used, but the impl itself is unsound for out-of-range input |
| `src/vm/bytecode.rs:16`, used `:400` | `functions::get_op_length::getOpLength` | opcode → 1 or 2 words | 43 lines, a `match` | none |
| `src/vm/bytecode.rs:4`, used `:273` | `FFlag::LuauCallFeedback.get()` | the only flag read on the device path: v11 bytecode is rejected unless it is on | `FValue::get`, `src/methods/f_value_t.rs` (22 lines) | `unsafe { *self.value.get() }` read of an `UnsafeCell`; if any override was ever pushed (`OVERRIDES_ACTIVE`, a global `AtomicBool`), also a thread-local `RefCell<HashMap>` lookup |
| `src/vm.rs:3050` | `ScopedAllFlags::enter(true)` in `validate_executable_luau_bytecode` (public, re-exported at `crates/nuxie-scripting/src/lib.rs:37`; compiled into every device variant, but no non-test caller in `nux-capi`, `nuxie`, `nuxie-runtime`, `nuxie-project-data-scripting` or `nuxie-scripting`, by `grep -rn validate_executable_luau_bytecode`) | pushes a thread-local override for every catalogued flag, pops on drop | `src/lib.rs:41-189` (catalogue + scope) | if called: flips the global `OVERRIDES_ACTIVE` to `true` forever, so every later `FValue::get` in the process takes the thread-local path; allocates a `Vec` and touches ~120 `HashMap` entries per call; `RefCell::borrow_mut` on thread-locals |
| `src/luau/facade/state.rs:302` | `set_all_flags(true)` in `Lua::new()` (reached from `ScriptVm::new`, `vm.rs:2106-2107`, called by `crates/nuxie/src/script_import.rs:78,793,815`) | writes every catalogued flag | `src/lib.rs:41-163` | **unsynchronized write through `UnsafeCell` (`FValue::set`, `src/records/f_value.rs:174-176`) to `static`s declared `unsafe impl Sync`**, on every `Lua::new()`; the comment at `f_value.rs:20-21` states the contract "flags configured before worker threads start". Audio processor VMs do not call `Lua::new()` (they use `nuxie_luau_vm` directly), so the `luau-audio-init` thread does not write flags; whether `ScriptVm::new()` runs on more than one thread on a device was not established |

Non-device uses, for completeness: `src/vm.rs:5784` (`#[cfg(test)]`),
`src/gpu_canvas.rs:3590`, `src/vm/lua_path.rs:632`,
`src/vm/lua_audio_processors.rs:853`, `src/vm/luau_audio_processor_engine.rs:661`
(test modules), `src/luau/facade/state.rs:1065` (`#[cfg(feature = "compiler")] fn compile`),
`build.rs:63`, and the `tests/` directories.

Forced panic-lint sites inside the device-used files of `luaur-common`
(`luau_opcode`, `luau_bytecode_tag`, `luau_bytecode_type`,
`luau_feedback_type`, `get_op_length`, `records/f_value.rs`,
`methods/f_value_*`, `lib.rs`): **1**, in
`methods/f_value_version_setter_f_value_version_setter.rs`, which the device
path does not call. Command: `parse.py` over
`luaur-common__apple.clippy.diagnostics` (section 3.1 run), filtered by those
paths.

### 2.2 What `set_all_flags(true)` changes for code that only loads bytecode

`set_all_flags(true)` sets the ~120 flags in `apply_all_flags`
(`vendor/luaur-common-0.1.8/src/lib.rs:44-156`): most to `true`, about 30
forced to `false` regardless of the argument. For the device path:

- **The safe VM reads no flag.** `crates/nuxie-luau-vm/Cargo.toml` has an empty
  `[dependencies]`; `grep -rn "FFlag\|DFFlag\|FInt::\|luaur" crates/nuxie-luau-vm/src`
  finds only two comments. Its accepted versions come from its own constants
  (`nuxie_luau_vm::BYTECODE_VERSION_MIN/MAX`, used at `bytecode.rs:58-62`).
- **The bytecode validator reads one flag**, `LuauCallFeedback`, set to
  `value` (true) by the catalogue (`lib.rs:54`). Its only effect: v11 bytecode
  is accepted (`bytecode.rs:272-277`). Under `ScopedAllFlags::enter(true)` the
  answer is the same. So on the device, `set_all_flags(true)` is equivalent to
  the constant "v11 feedback is allowed".
- `grep -rn "FFlag\|DFFlag\|FInt::\|DFInt" crates/nuxie-scripting/src` finds no
  other non-test flag read.
- `crates/nuxie-scripting/src/luau/facade/state.rs:296-298` comments that
  `Lua::new` sets the flags because "the bytecode validator and the audio
  engine's raw VM read" them. The raw VM is gone; the audio engine on the safe
  VM (`luau_audio_processor_engine/safe.rs`) reads no flag (no `FFlag` in that
  file).

### 2.3 What the compiler crates need from `luaur-common` that the device path does not

Command: `grep -rhoE "luaur_common::[a-z_]+(::[a-z_0-9]+)?" vendor/<crate>/src | sort | uniq -c`
and `grep -rhoE '\b(D?FFlag|D?FInt)::[A-Za-z0-9_]+'`.

| Crate | Files importing `luaur_common` | Main imports beyond the device set | Flag reads |
|---|---|---|---|
| `luaur-ast` | 108 | `luau_assert` macros, `dense_hash_map/set/table`, `variant`, `overloaded`, `escape`, `format*`, `luau_timetrace_scope` | 15 distinct, 52 sites |
| `luaur-bytecode` | 186 | `luau_assert`, instruction macros (`luau_insn_*`), `dense_hash_*`, `small_vector`, `format_append`, `get_jump_target`, `is_fast_call`, `is_jump_d`, `luau_capture_type` | 7 distinct, 51 sites |
| `luaur-compiler` | 154 | `luau_assert`, `dense_hash_*`, `insertion_ordered_map`, `dense_hash_pointer`, `luau_builtin_function`, `luau_proto_flag`, `vformat`, `set_all_flags` | 16 distinct, 77 sites |

`luaur-common` size by area (`find src/<dir> -name '*.rs' | xargs cat | wc -l`;
`unsafe` by the ratchet regex): `enums` 9 files/788 lines/1 `unsafe`;
`functions` 51/1305/30; `methods` 196/3641/72; `records` 35/2392/45;
`type_aliases` 43/444/5; `macros` 41/549/5; `wasm_libc.rs` 405 lines/15;
`strtod_shim.rs` 1. Total 383 files, 11,174 lines, 174 `unsafe` sites. The
hash containers, `vec_deque`, `variant` and allocator code that carry most of
the `unsafe` are used only by the compiler crates. Of the 174, the device path
reaches 1 (opcode transmute) + 13 (`records/f_value.rs` and `methods/f_value*`).

The wasm kernel also links `wasm_libc.rs`: `#[no_mangle]` `malloc`, `free`,
`realloc`, `strchr`, `time`, `clock`, `sysconf`, `strtoul`, `strtoull`, `mmap`,
`munmap`, `mprotect`, `gmtime_r`, `localtime_r` (15 `unsafe` sites) and
`strtod_shim` (1), because `luaur-common__publisher` is a dependency, whether or
not anything calls them. `luaur-common__publisher` lints at 341 forced sites
versus 288 natively for this reason (section 6).

### 2.4 The design question, in facts

Everything the device path takes from `luaur-common` is: four small enums
(about 350 lines, all data), one 43-line `match`, and one flag value that is a
constant `true` on every device configuration. That slice has 1 `unsafe`
(the transmute) and 0 relevant panic-lint sites. The flag machinery it drags in
is the source of the global-mutable-state hazard in section 2.1. The rest of
the crate (about 10,500 lines, 160 `unsafe`, 287 panic-lint sites) serves the
compiler crates, which do not ship. Also note the bytecode format constants
already live a second time in `nuxie-luau-vm` (`crates/nuxie-luau-vm/src/bytecode/mod.rs:21`
documents its opcode numbers as "luaur-common `LuauOpcode`, in declaration
order"), so a safe module, or the VM's own tables, can replace the dependency
without new knowledge. What this survey cannot settle: whether the device needs
to keep a runtime switch for v11 feedback, and how the build script's compiler
(host side, keeps `luaur-common`) stays in step with the device's tables.

## 3. `nuxie-scripting` panic sites

### 3.1 Forced panic-lint counts

The tool `tools/ratchets/panic-lints.sh` lints `//<crate>:<package>` only (the
default feature set). To get the device sets without touching the committed
baseline, the same Bazel command the tool builds (`ratchets.py:155-175`) was
run on the variant labels:

```
NUXIE_EXECUTION=local bazelisk --nosystem_rc --nohome_rc build --keep_going \
  --aspects=@rules_rust//rust:defs.bzl%rust_clippy_aspect \
  --output_groups=clippy_output \
  --@rules_rust//rust/settings:clippy_output_diagnostics=true --show_result=1000 \
  --@rules_rust//rust/settings:clippy_flag=--force-warn=clippy::{unwrap_used,expect_used,indexing_slicing,panic,unreachable,arithmetic_side_effects,unwrap_in_result} \
  //crates/nuxie-scripting:nuxie-scripting{,__apple,__android} <Apple-graph crates, section 6>
# wasm kernel variant, with the kernel's own build flags:
... same, plus --platforms=@rules_rust//rust/platform:wasm \
  --@rules_rust//rust/settings:extra_rustc_flag=-Cpanic=abort \
  //crates/nuxie-scripting:nuxie-scripting__publisher //vendor/luaur-common-0.1.8:luaur-common__publisher //crates/nuxie-luau-vm:nuxie-luau-vm__publisher
```

Load average: 8.29 at start, 12.31 at end of the native run (76 s wall, 507
disk-cache hits); wasm run under ~9.8. The build exited 1 only because of a
stray `x` label and `nuxie-script-signature__publisher` failing to compile for
the host platform (it needs the wasm platform; irrelevant to these counts). The
default-variant total, **481**, is the same order as a plain
`tools/ratchets/panic-lints.sh --baseline <scratch> crates/nuxie-scripting`
would give (that exact command was not run, to avoid a second identical build).

| Variant | Total | unwrap_used | expect_used | indexing_slicing | panic | unreachable | arithmetic_side_effects | unwrap_in_result |
|---|---|---|---|---|---|---|---|---|
| `__apple` (Apple) | 481 | 19 | 50 | 181 | 0 | 6 | 220 | 5 |
| `__android` (Android, built for the macOS host) | 480 | 19 | 50 | 181 | 0 | 6 | 219 | 5 |
| `__publisher` (wasm kernel, wasm platform) | 446 | 19 | 50 | 186 | 0 | 6 | 181 | 4 |
| default (`luau`, `compiler`, `js-host-seed`) | 481 | 19 | 51 | 181 | 0 | 6 | 219 | 5 |

Differences by (lint, file, line), computed with `collections.Counter`
subtraction over `parse.py` output:

- default − apple: `expect_used src/vm/lua_image_decode.rs:294`. That is the
  only site gated to non-shipping features (`compiler`/`js-host-seed`); the
  lint runs on the lib target, so `cfg(test)` code never appears.
- apple − default and apple − android: `arithmetic_side_effects src/shader_asset.rs:146` (`apple-authored-msl`).
- publisher − apple: 30 sites in wasm-only files (`luau_audio_shared_web.rs`,
  `luau_audio_web.rs`, `browser_kernel.rs`) and wasm-only branches of
  `lua_audio.rs` and `luau_audio_processor_engine.rs`.
- apple − publisher: 66 sites in native-only `luau_audio_shared.rs` plus the
  `shader_asset.rs:146` site.

So effectively all lint-visible sites are device-graph code.

Android caveat: the Android variant was linted for the macOS host, so any
`cfg(target_os = "android")` code in `nuxie-scripting` is not counted; `grep -rn
'target_os = "android"' crates/nuxie-scripting/src` finds none.

### 3.2 Per file, Apple variant (top 30 of 39 files)

| File (`crates/nuxie-scripting/`) | Total | unwrap | expect | index | panic | unreach | arith | unwrap_in_result |
|---|---|---|---|---|---|---|---|---|
| src/vm/luau_audio_shared.rs | 64 | 0 | 3 | 20 | 0 | 0 | 40 | 1 |
| src/vm/lua_mat4.rs | 57 | 0 | 0 | 30 | 0 | 0 | 27 | 0 |
| src/vm/lua_text.rs | 41 | 3 | 2 | 22 | 0 | 0 | 14 | 0 |
| src/vm.rs | 37 | 0 | 2 | 13 | 0 | 0 | 22 | 0 |
| src/vm/buffer_ext.rs | 36 | 0 | 6 | 9 | 0 | 0 | 21 | 0 |
| src/vm/luau_audio_samples.rs | 27 | 3 | 0 | 7 | 0 | 0 | 14 | 3 |
| src/vm/lua_audio_processors.rs | 25 | 0 | 11 | 4 | 0 | 0 | 10 | 0 |
| src/vm/bytecode.rs | 21 | 0 | 0 | 14 | 0 | 0 | 7 | 0 |
| src/shader_asset.rs | 20 | 0 | 2 | 16 | 0 | 0 | 2 | 0 |
| src/vm/lua_audio_samples.rs | 14 | 4 | 1 | 1 | 0 | 0 | 8 | 0 |
| src/vm/luau_audio_processor_engine/safe.rs | 14 | 0 | 0 | 3 | 0 | 0 | 11 | 0 |
| src/vm/luau_audio_processor_engine.rs | 12 | 0 | 0 | 5 | 0 | 0 | 7 | 0 |
| src/gpu_canvas.rs | 11 | 0 | 1 | 4 | 0 | 1 | 5 | 0 |
| src/vm/view_model.rs | 11 | 0 | 1 | 3 | 0 | 4 | 3 | 0 |
| src/gpu_canvas_ore/resources.rs | 10 | 0 | 6 | 1 | 0 | 0 | 3 | 0 |
| src/vm/native_registration.rs | 9 | 0 | 0 | 9 | 0 | 0 | 0 | 0 |
| src/gpu_canvas_ore/pipeline.rs | 6 | 0 | 2 | 1 | 0 | 0 | 3 | 0 |
| src/vm/lua_mat4_safe.rs | 5 | 0 | 0 | 2 | 0 | 0 | 3 | 0 |
| src/vm/lua_vec2d.rs | 5 | 0 | 0 | 3 | 0 | 0 | 2 | 0 |
| src/vm/lua_renderer.rs | 5 | 0 | 2 | 0 | 0 | 0 | 3 | 0 |
| src/gpu_canvas_ore/canvas.rs | 4 | 2 | 1 | 1 | 0 | 0 | 0 | 0 |
| src/gpu_canvas_ore/target.rs | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 0 |
| src/host_commands.rs | 4 | 0 | 0 | 0 | 0 | 0 | 4 | 0 |
| src/vm/listener_invocation.rs | 4 | 0 | 0 | 0 | 0 | 1 | 3 | 0 |
| src/vm/lua_mat2d.rs | 4 | 0 | 0 | 4 | 0 | 0 | 0 | 0 |
| src/vm/lua_path.rs | 4 | 0 | 2 | 1 | 0 | 0 | 1 | 0 |
| src/gpu_canvas_ore/pass.rs | 3 | 0 | 2 | 0 | 0 | 0 | 1 | 0 |
| src/gpu_canvas_ore/shader.rs | 3 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |
| src/vm/lua_canvas.rs | 3 | 0 | 3 | 0 | 0 | 0 | 0 | 0 |
| src/vm/lua_mesh.rs | 3 | 0 | 0 | 3 | 0 | 0 | 0 | 0 |

Note on `arithmetic_side_effects` (220 of 481): shipping builds use
`--config=release` (`compilation_mode=opt`) with no `-Coverflow-checks` or
`-Cdebug-assertions` flag (`.bazelrc`, `tools/bazel_runtime_build.py:147-150`),
so by rustc's defaults integer overflow wraps rather than panics there; only
integer division or remainder by zero (and `MIN / -1`) panics in release. The
lint does not separate these. Derived from flags and rustc defaults, not
checked on an artifact. A rough grep for integer `/` or `%` in shipping lines
finds 35 (pattern `[^/*]\s(/|%)\s*[a-z_(]`, includes float division; upper
bound).

### 3.3 Sites the lints do not see (grep)

Classifier (`classify.py`), heuristic: a line is non-shipping if its file is
reached only through a test- or `compiler`/`scriptnet`/`upstream-test-seams`/`tools`-gated
`mod` line (the `*_tests.rs`, `upstream_scripting_*.rs`, `facade/tests.rs`,
`gpu_canvas_ore/tests.rs`, `wave_c12_scalar_owner_tests.rs`,
`lua_rive_file.rs`, `lua_scriptnet.rs`), or lies inside an item whose `#[cfg(...)]`
mentions `test` or one of those features after removing `not(...)` arms
(`any(feature = "<shipping feature>", test)` counts as shipping), brace-matched.
Cross-check: it counts 72 shipping `.unwrap(`/`.expect(` sites versus 69 for the
Apple lint run and 73 for the wasm run, so it tracks the compiler's view
closely. Comments are stripped before matching.

Command: `python3 classify.py crates/nuxie-scripting hits.json`

| Pattern | Regex | Shipping | Non-shipping |
|---|---|---|---|
| `panic!`/`unreachable!`/`todo!`/`unimplemented!` | `\b(panic\|unreachable\|todo\|unimplemented)!` | 4 | 47 |
| `assert!`/`assert_eq!`/`assert_ne!` | `\b(assert\|assert_eq\|assert_ne)!` | 3 | 1779 |
| `debug_assert*!` | | 0 | 0 |
| `RefCell::borrow_mut` (and other `.borrow_mut()`) | `\.borrow_mut\(\)` | 222 | 80 |
| `.borrow()` | `\.borrow\(\)` | 173 | 154 |
| `copy_from_slice`/`clone_from_slice`/`copy_within` | | 12 | 15 |
| allocation calls | `with_capacity(`, `vec![..; n]`, `.resize(`, `.reserve(_exact)?(`, `.repeat(` | 54 | 23 |
| … whose size has an integer `as` cast | same plus `\bas (usize\|u32\|u64\|i32\|i64)\b` | 3 | 6 |

Shipping `panic`-family sites: `src/gpu_canvas.rs:1133`
(`unreachable!("native asset was resolved above")`),
`src/vm/listener_invocation.rs:112`, `src/vm/view_model.rs:643`,
`src/vm/view_model.rs:2966` (the lint sees these as `unreachable`).

Shipping `assert` sites: `src/vm.rs:672` (a `const _: () = assert!`, compile
time), `src/vm.rs:1073`, and `src/vm/lua_renderer.rs:132`
`assert_eq!(frame.token, self.token, "scoped renderer stack order")` **inside
`Drop for ScopedRendererAccess`**, next to `.expect("scoped renderer stack underflow")`
at `:131`; a second `expect` sits in `Drop for ActiveRendererBorrow` (`:150`).
A panic raised in `drop` while already unwinding aborts the process.

`RefCell` borrows: `try_borrow` appears 38 times in `src` (command
`grep -rn "try_borrow" crates/nuxie-scripting/src | wc -l`), against 395
shipping `borrow`/`borrow_mut` calls. Not every `.borrow()` is a `RefCell`
(some are the userdata `borrow::<T>()` of the facade, which returns `Result`);
the regex counts `.borrow()` with empty parentheses only, so typed userdata
borrows are excluded. Re-entrancy is the panic risk: host callbacks run inside
`lend(...)` (`facade/state.rs:916`) and scripts can call back into the host.

`copy_from_slice` shipping sites (each panics on length mismatch):
`src/gpu_canvas.rs:146`, `src/vm/lua_mesh.rs:43,44,123,189`,
`src/vm/lua_vec2d.rs:313,316`, `src/vm/luau_audio_samples.rs:205`,
`src/vm/luau_audio_shared.rs:791,810`, `src/vm/luau_audio_shared_web.rs:289,380`.

Allocation with `as` casts in shipping code:
`src/vm/lua_audio_samples.rs:518` (`vec![0.0; chunk_frames as usize * reader.channels() as usize]`),
`:523` (`Vec::with_capacity(chunk_bytes as usize)`),
`src/vm/lua_mesh.rs:198` (`this.resize(count as usize)`).

Allocation driven by script-supplied sizes (read from the 54 shipping sites):

| Site | Size source | Guard |
|---|---|---|
| `src/vm/lua_mesh.rs:197-198` → `:172` `staged.resize(count, ..)`, and `:165` `vec![..; count]` | script `ImageMeshInstances:resize(count: u32)` | **none**; up to 2^32 elements, outside the VM's 16 MiB ceiling; `Vec` capacity overflow panics, allocation failure aborts |
| `src/gpu_canvas.rs:2616-2617`, `:2683-2686` | script buffer sizes | `cpu_buffer_budget`/`gpu_buffer_budget.reserve(size)?` first |
| `src/vm/lua_audio_processors.rs:340-347` | stream descriptor `chunk_frames * channels * 4` | `try_reserve_exact`; the multiplication is unchecked `usize` arithmetic |
| `src/vm/lua_audio_samples.rs:518,523` | descriptor `chunk_frames`, `channels`, `chunk_bytes` | not traced further |
| `src/vm/luau_audio_shared_codec.rs:174-178` (decoding) | length word in the shared byte stream | `try_reserve_exact` |
| `src/vm/view_model.rs:2649` | script table length | `(2..=64).contains(&count)` first |
| `src/vm/bytecode.rs:195,201` | bytecode counts | `read_count` checks remaining bytes first (`bytecode.rs:548-557`) |
| `src/shader_asset.rs:653-1020` (7 sites) | shader asset entry counts | not traced; assets are authenticated (section 7) |
| `src/vm/luau_audio_web.rs:76,151` | host response length; JS caller `nuxie_audio_alloc(length)` | wasm kernel only |

Recursion with no depth bound. Command: `recur.py` (finds shipping functions
whose body calls themselves by name; it does **not** find mutual recursion).
12 found: `src/host_commands.rs:604 host_value_bytes` and `:622
host_value_nodes` walk a `HostValue` that was built under `max_value_depth`
(default 32, hard cap `MAX_HOST_VALUE_DEPTH = 64`, `host_commands.rs:18,49`;
`value_at_depth`/`table_value` at `:436-517` enforce it) — bounded.
**Unbounded**: `src/vm/lua_audio_processors.rs:656 to_shared` and `:733
from_shared` (script table → `SharedValue`; cycle check via `ancestors`, no depth
limit), `src/vm/luau_audio_processor_engine/safe.rs:601 push` and `:678 read`
(into and out of the processor VM), `src/vm/luau_audio_shared_codec.rs:96 write`
and `:159 value` (encode/decode of the same tree, the decoder reading a byte
stream). A script can post a deep non-cyclic table and overflow the native
stack, which no `catch_unwind` contains. Others found are not recursive on
data (`log_error`, `drop`, `observe_vm_error`, `sort` with a bound,
`from_render_raw_path` delegates). Mutual recursion elsewhere (facade value
conversion, view-model walks) was not measured.

## 4. `nuxie-scripting` `unsafe` sites

Command: `tools/ratchets/unsafe-count.sh --baseline <scratch> crates/nuxie-scripting vendor/luaur-common-0.1.8 crates/nuxie-luau-vm`
→ `crates/nuxie-scripting 52`, `vendor/luaur-common-0.1.8 174`,
`crates/nuxie-luau-vm 0` (also ast 1017, bytecode 232, compiler 744 in the same
output). Listing: `grep -rnE "unsafe (fn|\{|impl|extern)" crates/nuxie-scripting | grep "\.rs:"` → 52 lines.
`grep -rn "\bunsafe\b"` minus that pattern and minus comments finds no other
`unsafe` use (e.g. no `#[unsafe(no_mangle)]` in this crate). No
`forbid(unsafe_code)` in `nuxie-scripting` (only `nuxie-luau-vm/src/lib.rs:38`).

Not real: `src/vm/luau_audio_processor_engine/safe.rs:205` is a comment
("`unsafe impl Send`; the compiler checks it"). 51 real sites:

### 4.1 Shipping, Apple and/or Android (7)

| Site | What it does | Graph | Safe replacement needs |
|---|---|---|---|
| `src/shader_asset.rs:348` | calls `unsafe fn GpuCanvasVulkanSpirVShader::from_verified_parts` (`crates/nuxie-render-api/src/lib.rs`) with parts decoded from the authenticated ShaderAsset | Android (`decode_vulkan_spirv`, `cfg(any(feature = "android-authored-wgsl", test))`) | The callee's body is safe Rust (builds a struct after `provenance.authorizes_digest(..)`); `unsafe` is a trust marker for "native shader bytes are passed through to the driver". A sealed capability type instead of `unsafe fn` would remove the site; the actual risk is the GPU driver consuming unverified native code, which stays a trust boundary. |
| `src/shader_asset.rs:417` | same for `GpuCanvasAppleMetalShader::from_verified_parts` | Apple (`decode_apple_metal`, `cfg(any(feature = "apple-authored-msl", test))`) | same |
| `src/vm/lua_renderer.rs:112` | `mem::transmute` erasing the lifetime of `NonNull<dyn Renderer + '_>` to store it in a thread-local stack of active renderers | all | Scoped access without lifetime erasure: pass the `&mut dyn Renderer` down the call (a context argument), or keep it in a scoped slot the borrow checker sees (e.g. a closure-scoped `&mut` handed to the VM call), so script userdata holds only the token. |
| `src/vm/lua_renderer.rs:184` | `NonNull::as_mut` on that erased pointer while the token is marked borrowed | all | same design as `:112` |
| `src/gpu_canvas_ore/canvas.rs:63` | calls `unsafe fn wrapCanvasTextureInfo` on the ORE context (trait in `nuxie-ore-metal/.../ore_context_hpp.rs:411`; impls in `nuxie-renderer`) with a `CanvasTextureInfo` (raw native texture pointers) taken from a canvas the host allocated just above | Apple, Android | The raw-pointer seam lives in `nuxie-renderer`/`nuxie-ore-metal`; replacing `CanvasTextureInfo` (raw pointers) with an owned, typed handle at that trait would make the call safe. The backend wrap of a native texture is a true FFI boundary (Metal/Vulkan) in the renderer crates, contained there. |
| `src/gpu_canvas_ore/mod.rs:211` | `unsafe { ctx.wrapCanvasSampleView(...) }` (default trait method that calls `wrapRiveTexture` with a raw texture pointer) for `Image:view()` of a canvas on an immediate (non-recording) context | Apple, Android | same |
| `src/gpu_canvas_ore/mod.rs:222` | `unsafe { ctx.wrapImageSampleView(info) }` after a null check on `info.texture` | Apple, Android | same |

### 4.2 Shipping, wasm audio kernel only (12), file `src/vm/luau_audio_web.rs`

| Site | What it does | True FFI? |
|---|---|---|
| `:7` | `unsafe extern "C"` block importing `host_call`, `now_ms`, `shared_raw` from wasm module `nuxie_audio` (`cfg(feature = "browser-audio-host")`) | **yes**, the JS host import boundary; cannot be removed, only contained |
| `:28` | `unsafe fn host_call` stub when the host is absent | no; could be a safe fn |
| `:39` | `ptr::copy_nonoverlapping` in that stub | no; a slice copy would do |
| `:44` | `unsafe fn now_ms` stub | no |
| `:63`, `:77` | calls to the imported `host_call` with Rust-owned buffers | yes (call side of `:7`); contain in one safe wrapper taking `&[u8]`/`&mut [u8]` |
| `:103` | call to imported `shared_raw` | yes, same |
| `:132` | call to imported `now_ms` | yes, same |
| `:155`, `:156` | `pub unsafe fn free` / `Box::from_raw` of a buffer handed to JS by `alloc` | yes: JS owns the buffer between `nuxie_audio_alloc` and `nuxie_audio_free` (exports in `tools/luau-audio-web/src/lib.rs`, 9 `unsafe` there); containable to one module |
| `:164`, `:166` | `pub unsafe fn kernel_request` reading `(pointer, length)` from JS with `slice::from_raw_parts` | yes, same |

### 4.3 Not shipping (32)

| Sites | What | Gate |
|---|---|---|
| `build.rs:73,74,78` | `luau_compile` C ABI: `from_raw_parts` on the returned malloc buffer, `extern "C" free`, `free` | host build script. Replacement: a safe Rust API on `luaur-compiler` returning `Vec<u8>` (the facade's `compile` at `src/luau/facade/state.rs:1054-1075` already uses `compile_bytes(...).into_bytes()`, safe, except that it passes `&mut encoder as *mut dyn BytecodeEncoder`) |
| `src/gpu_canvas.rs:3602,3603,3607`; `src/vm/lua_path.rs:644,645,649`; `src/vm/lua_audio_processors.rs:862,863,866`; `src/vm/luau_audio_processor_engine.rs:670,671,674` | the same `luau_compile` + `free` pattern in test helpers | `cfg(test)` / `cfg(all(test, feature = "compiler"))` |
| `tests/async_shader_instantiation.rs:134`, `tests/promise_scenarios.rs:57,58,62`, `tests/shader_asset_resolution.rs:244`, `tests/scripted_interpolator.rs:19,20,24`, `tests/support/mod.rs:21,22,26` | same pattern | integration tests |
| `src/shader_asset.rs:1404` | `GpuCanvasShaderProvenance::for_verified_artifact_digest_unchecked` in a test helper | `cfg(test)` |
| `src/vm/upstream_scripting_gpu_features.rs:78,81`, `src/vm/script_call_gpu_scope_tests.rs:140,143` | test doubles implementing `unsafe fn wrapCanvasTexture`/`wrapRiveTexture` | test |
| `src/vm/lua_artboards.rs:551` | `pub unsafe fn upstream_test_scripted_artboard_render` (unsafe as a "trusted bytecode" marker) | `upstream-test-seams` |

Total: 7 + 12 + 32 = 51 real, + 1 comment = 52.

## 5. How a panic reaches a client today

### 5.1 Panic strategy per artifact

| Setting | Where | Value |
|---|---|---|
| Bazel, every build | `.bazelrc:3` `build --@rules_rust//rust/settings:extra_rustc_flag=-Cpanic=unwind` | unwind |
| Bazel wasm builds | `tools/luau-audio-web/pool.py:83` (kernel `build`) and `tools/bazel/runtime.py:435` (any `wasm32` target) append `--@rules_rust//rust/settings:extra_rustc_flag=-Cpanic=abort` | abort (rustc takes the last `-C panic`; the `.bazelrc` flag comes first on the command line) |
| Distribution builds | `tools/bazel_runtime_build.py:147-170`: `--config=release`, fat LTO, `codegen_units=1`, `-Cdebuginfo=0`; Apple adds `-Cstrip=none`; Android adds link args. No panic flag. It does not pass `--noworkspace_rc`, so `.bazelrc` applies | unwind |
| Cargo `[profile.release]` | `Cargo.toml:121-127` | `panic = "unwind"` (comment: keep host panic containment at protected/FFI boundaries) |
| Cargo `[profile.release-apple]` | `Cargo.toml:142-146` | `panic = "unwind"`, `strip = "debuginfo"` (comment: customer Apple binaries must contain panics at the C boundary) |
| Cargo `[profile.release-size]` | inherits release | unwind |
| Probe | `crates/nuxie-scripting/Cargo.toml:86-90`, example `safe_vm_abort_probe` | built with and without `-Cpanic=abort`, not shipped |

Shipped: Apple xcframework **unwind**; Android `.so` **unwind**; wasm audio
kernel **abort** (confirmed from the build flags; the produced `.wasm` was not
inspected). The kernel's `BUILD.bazel` target itself sets no panic flag, so a
kernel built through some other Bazel invocation without the wasm wrapper
flags would get unwind; on `wasm32-unknown-unknown` the standard library is
abort-only anyway.

### 5.2 Where panics are caught

Command: `grep -rn 'catch_unwind' <dir> | wc -l` (and `panic::set_hook|take_hook`).

| Crate (src) | `catch_unwind` | `set_hook`/`take_hook` |
|---|---|---|
| `crates/nux-capi` | 16 | 0 |
| `crates/nux-apple-product-extension` | 0 | 0 |
| `crates/nuxie-project-data-scripting` | 0 | 0 |
| `tools/luau-audio-web` (kernel) | 0 | 0 |
| `crates/nuxie-scripting` | 6 (3 shipping) | 0 |
| `crates/nuxie-runtime` | 38 | 0 |
| `crates/nuxie`, `crates/nuxie-luau-vm` | 0 | 0 |

No crate installs a panic hook, so the default hook prints the message to
stderr when a panic fires, even when it is caught.

**C boundary (`nux-capi`).** Guards in `crates/nux-capi/src/lib.rs`:
`ffi_guard` (`:212`, returns a caller-chosen default, usually
`NuxStatus::RuntimeError`), `ffi_guard_with_font_access` (`:220`),
`ffi_guard_with_result` (`:1843`: returns `NuxStatus::RuntimeError` = 4 and
publishes a `NuxCapiResult` with message "Rust panic contained at nux-capi
boundary"), `ffi_guard_with_handle_result` (`:1877`: also reclaims a
half-published handle), `ffi_guard_with_player_step_result` (`:3443`: "Rust
panic contained during atomic player step"). The guards also return
`ReentrantCall` (10) when a platform or font callback is active, before
running the body. Coverage scan: 191 exported `extern "C"` functions with
`no_mangle`/`export_name` across `nux-capi`, the product extension and
`nuxie-project-data-scripting`; 171 call a guard directly in their body; the
other 20 delegate (`nux_file_import_metal` → `import_metal_file_with_authority`
→ `ffi_guard`; `nux_renderer_resize` → `with_result` → `ffi_guard_with_result`;
`nux_player_step_begin`/`_end`/`set_row_collapsed` → `player_live_move_body` →
`ffi_guard_with_player_step_result`). 6 of the 20 were followed to a guard; the
other 14 were not each checked. The executor worker thread wraps its body in
`catch_unwind` (`crates/nux-capi/src/executor.rs:140`). What the host sees:
status 4 plus, where the API has an `out_result`, that message; state touched
by the panicking call is whatever it was (`AssertUnwindSafe` everywhere).

**Inside the Luau layer (shipping).**
`crates/nuxie-scripting/src/luau/facade/state.rs:905-930` `trampoline`: every
host function the facade registers runs under
`catch_unwind(AssertUnwindSafe(|| callback(&lua, args)))`; a panic becomes a
Luau error `"rust panic: <message>"` raised into the script, so the script (or
`pcall`) sees it and the host call continues.
`.../luau_audio_processor_engine/safe.rs:380`: each audio block's
`process_block` is caught; a panic zeroes the rest of the buffer and fails the
processor ("audio processor panicked"); the slicing `out[first..end]` that
builds the argument is outside the closure.
`.../luau_audio_processor_engine.rs:451`: processor construction on the
`luau-audio-init` thread is caught. The `luau-audio-init` thread loop itself
(`:632-639`) is not wrapped; a panic outside the inner catch ends that thread
and later submissions fail with "audio processor initialization worker
stopped".

**Product extension and kernel.** `nux_product_file_import_configured`
(`crates/nux-apple-product-extension/src/lib.rs:22`) delegates to
`nux_capi::nux_file_import_metal_with_trusted_native_shaders_and_program_adapter`;
whether that function's body guards was not checked. The wasm kernel exports
(`tools/luau-audio-web/src/lib.rs`) have no catch; under `-Cpanic=abort` a
panic is a wasm `unreachable` trap, which the AudioWorklet sees as a thrown
`RuntimeError` and the instance is dead afterwards. `kernel_request` also
`expect`s on JSON serialization (`luau_audio_web.rs:195`).

Not contained by any `catch_unwind`, regardless of strategy: stack overflow
(section 3.3), allocation failure (`handle_alloc_error` aborts), a panic during
unwinding (the `Drop` impls in `lua_renderer.rs:127-152`), and a panic crossing
an `extern "C"` callback that is not guarded (not found in the scripting path).

## 6. Forced panic-lint totals, Apple device graph (report only)

Same run as section 3.1 (Apple variants, macOS host, load 8.3 → 12.3).

| Crate (Bazel label) | Total | unwrap | expect | index | panic | unreach | arith | unwrap_in_result |
|---|---|---|---|---|---|---|---|---|
| nuxie-runtime__apple | 4985 | 624 | 1331 | 1291 | 31 | 49 | 1612 | 47 |
| nuxie-schema__apple | 3138 | 0 | 2 | 3128 | 7 | 0 | 1 | 0 |
| nuxie-renderer__apple | 2435 | 123 | 184 | 1020 | 46 | 20 | 1020 | 22 |
| nuxie-ore-metal__apple | 513 | 10 | 42 | 248 | 26 | 4 | 180 | 3 |
| nuxie-scripting__apple | 481 | 19 | 50 | 181 | 0 | 6 | 220 | 5 |
| nuxie-render-api__apple | 409 | 0 | 51 | 194 | 1 | 1 | 161 | 1 |
| nuxie-binary__apple | 291 | 7 | 41 | 153 | 0 | 4 | 62 | 24 |
| luaur-common__apple | 288 | 17 | 26 | 91 | 13 | 22 | 117 | 2 |
| nux-capi__apple | 76 | 22 | 4 | 28 | 0 | 5 | 16 | 1 |
| nuxie-audio__apple | 64 | 0 | 1 | 13 | 0 | 0 | 50 | 0 |
| nuxie-image-codec__apple | 20 | 0 | 0 | 16 | 0 | 1 | 3 | 0 |
| nuxie-project-data__apple | 8 | 0 | 0 | 1 | 0 | 0 | 7 | 0 |
| nuxie__apple | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 0 |
| nuxie-luau-vm__apple | 0 | 0 | 0 | 0 | 0 | 0 | 0 | 0 |
| nuxie-project-data-scripting__apple | 0 | | | | | | | |
| nuxie-script-signature__apple | 0 | | | | | | | |
| nux-apple-product-extension__apple | 0 | | | | | | | |
| **Total** | **12709** | | | | | | | |

Wasm kernel (wasm platform): nuxie-scripting__publisher 446,
luaur-common__publisher 341 (17/26/104/13/22/157/2), nuxie-luau-vm__publisher 0.
`luaur-common__apple` = 288 matches `tests/ratchets/panic-lints-baseline.txt`.
Vendored third-party crates in the Apple graph (`jpeg-decoder`, `skrifa`,
`symphonia-*`, `taffy`) were not linted.

## 7. Untrusted input reaching this code on a client

| Entry point | Where it enters | Validated before use? | Where |
|---|---|---|---|
| Script bytecode in a `.riv`/project file | `ScriptAsset::bytecode` (`crates/nuxie-runtime/src/mechanical_port/source/assets/script_asset.rs:505-530`) | **Signature**: signed content verified with libhydrogen (`nuxie-script-signature`); unsigned content is kept but marked unverified. `File::accepts_script` (`file.rs:1525-1531`) in a shipping build (no `tools` feature) accepts only verified scripts whether or not `require_signed_scripts` is set; host-authorized bytes come through `decode_with_host_authorization` (`script_asset.rs:495`, `unsafe fn` as a trust marker). **Structure**: `ScriptVm::load_bytecode` runs `validate_luau_bytecode` first (`crates/nuxie-scripting/src/vm.rs:2640-2646`), which rejects version 0, versions outside `nuxie_luau_vm::BYTECODE_VERSION_MIN..=MAX` (v12 and v13 with explicit notes, `bytecode.rs:53-89`), bad type versions, out-of-range counts (checked against remaining bytes), string and proto indices, opcodes `>= LOP__COUNT`, instruction overrun, and non-string userdata direct-access keys; then the safe VM validates again on load (facade `load_bytecode_as`, `state.rs:504-509`, "the safe VM validates the chunk (D7)"). |
| Audio processor modules | `ScriptVm::register_audio_processor_module` (`vm.rs:2737-2746`) | `validate_luau_bytecode`, then loaded into processor VMs through the facade (safe VM validation). |
| Host-authorized validation API | `validate_executable_luau_bytecode` (`vm.rs:3047`, public) | is the validator; side effect: `ScopedAllFlags` (section 2.1). |
| GPU canvas generator bytecode | `gpu_canvas.rs:2387` `vm.load_bytecode("gpu-canvas", bytecode)` | same `ScriptVm::load_bytecode` path. |
| Embedded helper chunks | `embedded_chunk.rs:54` | build-time output of our own compiler run (section 1.5); facade path (safe VM validation only). |
| Script-supplied values through host functions | facade conversion (`luau/facade/conversion.rs`, `value.rs`) on every host call; `trampoline` catches panics | Conversions return `Result` (type mismatch is an error); whether every conversion path is panic-free was not audited beyond the lint counts (`conversion.rs` does not appear in the top 30 files). No general size or depth limit at this layer. |
| Host commands (script → app) | `host_commands.rs:423-517` | depth limit (default 32, cap 64) and node/byte limits (`host_value_nodes`, `:316`) with error codes `script.resource.host_value_depth/_nodes`. |
| Audio messages (script → processor VM) | `lua_audio_processors.rs:656` `to_shared` | cycle check only; **no depth or size limit** (section 3.3). |
| Script-chosen sizes | `ImageMeshInstances:resize`, buffers, PCM readers (section 3.3 table) | mixed: budgets on GPU canvas buffers, `try_reserve` on some audio paths, **none** on mesh instances. The VM's own allocation is capped (`SCRIPT_VM_MEMORY_LIMIT_BYTES = 16 MiB`, `vm.rs:279`; `DEFAULT_SCRIPT_VM_MEMORY_BYTES = 64 MiB`, `vm.rs:854`; enforced in `resource_limits.rs`); host-side `Vec`s are outside it. |
| Shader assets | `shader_asset.rs` decoders (`decode_whole_module_bytes`, `decode_binding_map`, `decode_supplemental_reflection`, …) | authenticated (`GpuCanvasShaderProvenance`, digest checked by `from_verified_parts`) and cross-checked by the decoders before the `unsafe` constructors (section 4.1). Decoder panic sites: 20 lint sites in `shader_asset.rs` (16 indexing). |
| Image/audio file data reached from scripts | `lua_image_decode.rs`, `lua_audio_samples.rs`, `luau_audio_samples.rs` | not traced in this survey; decoding itself is in `nuxie-image-codec` and `symphonia-*` (section 6 / not linted). |
| wasm kernel requests from the AudioWorklet | `kernel_request(pointer, length)` (`luau_audio_web.rs:164`) | JSON parsed with `serde_json` (errors returned); the pointer/length pair is trusted. |
| `.riv` file structure | `nuxie-binary` / `nuxie-runtime` importers | outside the Luau code; counts in section 6. |

Separation for the design: hostile-input panics are the bytecode validator's
indexing (14 lint sites, after explicit bound checks in most cases), the
conversion and userdata paths (`buffer_ext.rs`, `lua_mat4.rs`, `lua_text.rs`,
audio codecs, mesh), and the unbounded recursion/allocation above. Internal
invariant panics are the `unreachable!`s, the renderer-scope `expect`/`assert`
in `Drop`, the `expect`s on JSON serialization and `allocation context`, and
`RefCell` re-entrancy.

## What could not be measured

- The exact `tools/ratchets/panic-lints.sh crates/nuxie-scripting` invocation
  was not run separately; its default-feature graph was linted through the
  same flags on `//crates/nuxie-scripting:nuxie-scripting` (481).
- The Android variant was linted for the macOS host, not an Android target.
- Integer overflow behaviour of the shipped artifacts was derived from flags
  and rustc defaults, not checked on a binary; same for the kernel's effective
  `panic=abort`.
- The grep counts in section 3.3 are heuristic (regex plus a brace-matching
  cfg classifier); mutual recursion was not searched; `.borrow()` counts mix
  `RefCell` with other `borrow` methods.
- 14 of 20 delegating C exports were not followed to their guard; the native
  audio output callback (who calls `AudioRenderer::render` on device) was not
  traced; image/audio file decoders reached from scripts were not traced.
- Vendored third-party crates in the device graph were not linted.
