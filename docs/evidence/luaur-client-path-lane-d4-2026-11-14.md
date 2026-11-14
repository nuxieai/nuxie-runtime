# Luau client path, lane D4 (GPU files), 2026-11-14

Design: `docs/luaur-client-path-safe.md` section 6 (lane D4) and section 5
items 2 and 3. Survey: `docs/evidence/luaur-client-path-survey-2026-11-14.md`.
Branch `claude/luaur-lane-d4-gpu-460b31`, based on `levi/luaur-client-safe`
(`9eb238d58f`). Commits: `4ec3a4bcb0` (shader proof, reverted by `1ee6f0021f`), `018e3fd3c7`, `3284eb7ce9`, `a21402f0d5`, `2062338bd1`, `d23de4ef38` (the `pass.rs` let-chain fix), and this evidence.

Files: `crates/nuxie-scripting/src/gpu_canvas.rs`, `src/gpu_canvas_ore/*`,
`src/shader_asset.rs`, and the two native shader constructors in
`crates/nuxie-render-api/src/lib.rs`.

## Summary

| Item | Result |
|---|---|
| C.2 verified shader constructors | built, then **reverted** after review: a digest-only proof is unsound (section 1); 2 shipped `unsafe` remain in `shader_asset.rs` |
| `for_verified_artifact_digest_unchecked` | left `unsafe`: its trust decision and shipped caller are in `crates/nuxie/src/script_import.rs` |
| C.3 canvas texture wrapping | stopped at the report: needs a change in every ORE backend (section 2); 3 shipped `unsafe` remain |
| Seven panic lints in these files | 61 / 61 / 62 (default / Android / Apple) to **0 / 0 / 0**; other files unchanged |
| `unsafe-count` | unchanged (`crates/nuxie-scripting` 52, `crates/nuxie-render-api` 13); shipped `unsafe` in these files: 5, as before |
| `nuxie-render-api` | net unchanged against the base |

## 1. Verified shader constructors (design 5.2): built, reverted

What was built (`4ec3a4bcb0`): a `GpuCanvasVerifiedShaderArtifact { provenance }`
proof with a private field, made only by a new
`GpuCanvasShaderProvenance::verify_artifact(self, size, &sha256)` (the
existing digest comparison), and both `from_verified_parts` made safe, taking
the proof. `shader_asset.rs` made the proof where it already checked the
digest. Behaviour and error text were unchanged; tests passed.

Why it was reverted (`1ee6f0021f`), from `codex review` round 1 (P1), which is
correct: the proof binds a provenance to an artifact *digest*, not the
decoded parts to the artifact *bytes*. Safe code that holds a provenance (it
is passed around freely; only minting is `unsafe`) and knows the digest can
make a proof and attach unrelated SPIR-V or MSL. That code then reaches
`ExactGpuCanvasShaderOccurrence::compile` and the backend's unchecked
`vkCreateShaderModule` / Metal library creation. The old `unsafe fn`'s safety
contract ("every part was decoded from these exact bytes") is exactly what
the proof cannot express, so making the constructor safe weakened a
soundness boundary.

What a sound version needs: a proof that covers the parts. Either the
decoder moves into `nuxie-render-api` and returns an opaque decoded artifact
(the constructors become private), or `nuxie-render-api` hashes the artifact
bytes itself and the decoder hands it borrowed sub-slices of those bytes whose
containment it can check. Both move the RSTB decoder (or its byte ownership)
out of `nuxie-scripting`, which is outside this lane, so per the stop rule I
report it instead. The two `unsafe` blocks in `shader_asset.rs`
(`decode_vulkan_spirv`, `decode_apple_metal`) remain.

`for_verified_artifact_digest_unchecked` follows the same pattern, but its
shipped caller is `crates/nuxie/src/script_import.rs:718`
(`mint_shader_provenance`), and the decision it records (the file's
native-shader capability admitted this artifact as trusted exporter output)
is made in `crates/nuxie`. A safe constructor in `nuxie-render-api` would let
any crate mint provenance. Left `unsafe`; its `nuxie-scripting` use is a
`cfg(test)` helper (`shader_asset.rs:1414`).

## 2. Canvas texture wrapping (design 5.3): the report

The three shipped `unsafe` sites in `gpu_canvas_ore` (`canvas.rs:66`
`wrapCanvasTextureInfo`, `mod.rs:214` `wrapCanvasSampleView`, `mod.rs:225`
`wrapImageSampleView`) remain. Removing them needs a change in every ORE
backend, so per the brief I stopped here.

What the pointers are and who owns them. `CanvasTextureInfo`
(`crates/nuxie-ore-metal/.../ore/ore_context_hpp.rs:839`) is a plain struct
with public fields `canvas: *mut c_void`, `texture: *mut c_void`, `width`,
`height` and `owner: Option<Rc<dyn Any>>`. It is issued by the renderer
backend through `RenderCanvas::ore_texture_info` / `RenderImage::ore_texture_info`:

| Producer | `canvas` | `texture` | `owner` keeps alive |
|---|---|---|---|
| Metal canvas, `nuxie-renderer/src/native_metal/render_canvas.rs:295` | `&MetalRenderCanvasBridge` inside `owner` | image's texture | the bridge |
| Metal image, `native_metal/mod.rs:808` | null | `&MetalRiveTextureBridge` inside `owner` (retained `MTLTexture`) | the bridge and the image |
| exact-source canvas, `exact_source_adapter.rs:1117` | the source `RenderCanvas` | from its render image | image |
| `RiveRenderImage`, `rive_render_image_hpp.rs:427` | null | the image's `gpu::Texture` | the image |
| `nuxie-render-api/src/routing.rs:22` (canvas without a backend texture) | null | null | the canvas handle |

`routing::canvas_texture_info` wraps whatever the backend returned so that
`owner` also holds the typed `RenderCanvasHandle`. The pointers are only
dereferenced by the backend's `wrapCanvasTexture` / `wrapRiveTexture`: Metal
(`nuxie-ore-metal/.../ore_context_metal_mm.rs:2052,2059`) casts them to its
bridge types with no foreign-pointer check; WebGPU
(`webgpu/..._cpp__impl.rs:1109`) and Vulkan (`vulkan/..._cpp__impl.rs:1675`)
cast `canvas` to `RenderCanvas` and its render target to their own target
type; GL checks for a "non-GL, stale, or foreign" target. The trait's default
`wrapCanvasSampleView` / `wrapCanvasTextureInfo` / `wrapImageSampleView`
just forward `info.texture` / `info.canvas`; overrides live in
`deferred/ore/ore_deferred_context.rs`, `exact_source_adapter/ore_context_owner.rs`
and `tools/runtime-test-support/src/recording_gpu.rs`.

Why the narrow change is not sound. Making the three trait methods safe
requires that safe code cannot hand them a bad pointer. Today any safe code can
build a `CanvasTextureInfo` (public fields), and nothing ties an info to the
context it is passed to: an info issued by one backend and passed to a Metal
context is reinterpreted as a `MetalRiveTextureBridge`. So the info must
become an opaque, backend-issued handle, and each backend must recognise only
its own handles.

The shape that fits: drop the raw pointer fields; the handle carries only
`width`, `height` and `owner`, and each backend's wrap downcasts `owner` to
its own bridge type (`MetalRiveTextureBridge`, `MetalRenderCanvasBridge`, the
GL/WebGPU/Vulkan canvas types). A foreign or stale handle fails the downcast
and returns `None`, which the scripting callers already turn into "failed to
wrap canvas texture" / "Image:view() not supported on this backend". The
three trait methods become safe and `nuxie-scripting` loses all three sites.
The backend's remaining `unsafe` would be where it builds the bridge from the
native object (`native_metal/mod.rs:798-800`, `Retained::retain` and
`nativeHandle`) and inside its own wrap implementation.

Scope of that change: `CanvasTextureInfo` is named in 17 files across
`nuxie-ore-metal` (trait, Metal backend, `ore_cmd` replay and recording),
`nuxie-renderer` (Metal, GL, WebGPU, Vulkan, deferred context, exact-source
adapter, GM helper, tests), `nuxie-render-api` (`routing.rs`, tests),
`tools/runtime-test-support` and `crates/nuxie/tests`. That is a renderer
change, not a scripting one.

## 3. The seven lints

Measured with `tools/ratchets/panic-lints.sh --baseline <scratch>
crates/nuxie-scripting crates/nuxie-scripting:nuxie-scripting__apple
crates/nuxie-scripting:nuxie-scripting__android` (one Bazel build of all
three labels), counted per file from the `.clippy.diagnostics` outputs.
Android was linted for the macOS host. Before: tip `9eb238d58f`, load average
8.29 at start, 25.06 at end. After (tip `1ee6f0021f`): load average 17.90 at
start, 22.38 at end. Re-run on `d23de4ef38` (the `pass.rs` let-chain fix,
load about 10 to 20): same counts, and the crate's other, non-ratcheted clippy
diagnostics back at 21 (the `Option`-returning `pass()` had added two
`collapsible_if`).

Before, per file (default / Android / Apple):

| File | default | Android | Apple | Lints (Apple) |
|---|---:|---:|---:|---|
| `gpu_canvas.rs` | 11 | 11 | 11 | arith 5, expect 1, index 4, unreachable 1 |
| `gpu_canvas_ore/canvas.rs` | 4 | 4 | 4 | unwrap 2, expect 1, index 1 |
| `gpu_canvas_ore/mod.rs` | 1 | 1 | 1 | expect 1 |
| `gpu_canvas_ore/pass.rs` | 3 | 3 | 3 | expect 2, arith 1 |
| `gpu_canvas_ore/pipeline.rs` | 6 | 6 | 6 | expect 2, arith 3, index 1 |
| `gpu_canvas_ore/resources.rs` | 10 | 10 | 10 | expect 6, arith 3, index 1 |
| `gpu_canvas_ore/shader.rs` | 3 | 3 | 3 | index 3 |
| `gpu_canvas_ore/target.rs` | 4 | 4 | 4 | unwrap 4 |
| `shader_asset.rs` | 19 | 19 | 20 | index 16, expect 2, arith 2 (one is `apple-authored-msl` only) |
| **lane total** | **61** | **61** | **62** | |
| other files | 415 | 414 | 414 | |
| crate total | 476 | 475 | 476 | |

After: every file above is at **0** in all three variants; other files 415 /
414 / 414 (unchanged); crate totals 415 / 414 / 414.

The survey counted 481/480; the baseline here is 476/475 because lane A's
commits on `levi/luaur-client-safe` had already removed some sites in other
files.

How each family went:

- **`shader_asset.rs` decoders.** `Cursor` now holds the unread tail
  (`rest: &[u8]`) instead of an offset and reads with `split_at_checked` and
  `split_first_chunk::<N>`; `read_u8/u16/u32/u64` and the 32-byte digests go
  through `read_array::<N>`. Binding-map rows are read through a cursor over
  the row instead of `row[0]..row[17]` (rows are at least 14 bytes, checked
  before, so the row cursor's truncation errors cannot fire). The section
  header skip `variants * 9` became nine-byte reads per variant.
  `entry_bindings()` returns `Result` ("missing native ShaderAsset", the
  message the sibling accessors already use) instead of `expect`, and the
  decode factory's `expect` became an error.
- **`gpu_canvas.rs`.** `GPUBuffer:write` uses `checked_sub` for the default
  length and checked `get`/`get_mut` for the copy (the `copy_from_slice` now
  only runs on two slices whose lengths were compared). The uniform range uses
  `bytes.get(start..end)` with the existing error. The `unreachable!` in
  `RegisteredGpuCanvasShaderAsset::resolve` is gone: `resolve` is one `match`
  on the asset state, so there is no impossible arm. The prepared-shader loop
  skips an asset with no handle like a failed resolve. Interrupt budget uses
  `checked_sub`; the attribute count uses `checked_add` with the limit;
  snapshot images compute `bytes_per_row` with `checked_mul` inside the
  existing size validation; the 1-based color-target number comes from
  `(1..).zip(..)`; the single render pass is walked with `iter_mut` instead of
  `[0]`.
- **`gpu_canvas_ore` texture views.** `textureViewBase()` returns `None` off
  the recording thread or for a non-view handle, so the `expect`/`unwrap`
  calls on it were reachable panics. One helper, `viewed_texture`, returns
  "GPU texture view is unavailable"; `TextureView::format` and
  `sample_count` return `Result` ("GPU texture is unavailable" when ORE has no
  format/sample count). The `GPUTarget` fields read through one
  `read_texture` helper. `image_texture_view` returns the view it stored
  instead of re-reading the cache with `expect`. `Canvas::replace_backing`
  returns an error instead of `expect("allocation context")`.
- **`pass.rs`.** `Pass::pass()` returns `Option`; every call site was already
  behind `validate()`, which returns `false` (no-op) when there is no pass, so
  `None` skips the call the same way. Dynamic offsets iterate `1..=len`.
- **`pipeline.rs` / `shader.rs`.** Attribute totals use `checked_add` and are
  added once per layout; the attribute table is borrowed with `if let` (it is
  `None` only when the count is zero); color targets and render-pass
  attachments are zipped with the fixed `[_; 4]` arrays (the counts are
  checked to be at most 4 before). `auto_layouts` marks seen groups with
  `get_mut` and builds the list with a running "used length" instead of
  `max_group`/`seen[group]`/`layouts[group]`. `BindGroup::follow` returns
  early if the remake inputs are gone instead of `expect("checked above")`.

Observable behaviour: no result or existing error text changed. New errors
appear only where the code used to panic.

## 4. What the lints miss, in these files

- **`copy_from_slice`.** One shipped site, `gpu_canvas.rs:153`
  (`GPUBuffer:write`): now runs only after both checked ranges are found and
  their lengths compared.
- **Integer division.** Two shipped sites, `gpu_canvas.rs:1661` (`offset %
  256`) and `:2645` (`offset % 4`): constant non-zero divisors, cannot panic.
- **`Drop`.** No `Drop` impl in these files.
- **`RefCell` borrows across a VM call.** Heuristic scan (a `let`-bound
  `.borrow()`/`.borrow_mut()` whose scope contains a Lua call) found six
  sites, all the ORE context `borrow_mut()` held across `lua.create_userdata`
  (`resources.rs` GPUTexture:view, GPUBuffer.new, GPUTexture.new,
  GPUSampler.new; `pipeline.rs` GPUPipeline.new; `shader.rs`
  GPUBindGroupLayout.new). A VM allocation can run a collection step that
  drops other GPU userdata; if a resource handle's drop needs the context, it
  would find it borrowed. Fixed (trivial): the guard is dropped before
  `create_userdata`. Not checked: whether any ORE resource drop actually
  borrows the context; borrows not bound by `let`; facade userdata borrows
  (`data.borrow_mut::<Self>()`, which return `Result`). For lane E.
- **Allocations sized by a script or file.**

  | Site | Size from | Bound |
  |---|---|---|
  | `gpu_canvas.rs:2635` `create_buffer_with_capacity(size)` | script | `cpu_buffer_budget.reserve(size)?` first |
  | `gpu_canvas.rs:2705` `vec![0; size]` / `to_vec` | script | `gpu_buffer_budget.reserve(size)?` first |
  | `shader_asset.rs` decoder `with_capacity` (7 sites) | asset counts | `u8` counts (at most 255), or the binding-map count after `remaining() >= required_bytes`; the asset is at most `MAX_RSTB_BYTES` (4 MiB) |
  | `pipeline.rs:387` `with_capacity(layouts.raw_len())` | script table | `raw_len() <= kMaxBindGroups` checked first |
  | `shader.rs:51` | decoded shader entries | at most 255 |
  | `GPUBuffer:write` `source.to_vec()` (`gpu_canvas.rs`, `resources.rs`) | a script buffer | a copy of a buffer that already lives inside the VM's memory limit; transient |
  | **`gpu_canvas.rs:557` `GPUTexture:upload` → `uploads.push`** | script buffer, repeated | **none**: each call copies the whole buffer into a host `Vec` the texture keeps, so repeated uploads grow host memory outside the VM limit. `gpu_canvas.rs:244` also clones the whole upload list each time a binding is made. For lane E (needs a limit chosen) |
  | `gpu_canvas.rs` `draws.push` (`:1810`, `:1870`) | draw calls | per-draw invocation limit; the number of draws per pass is bounded only by the interrupt budget per call. For lane E to confirm |

## 5. Checks

`RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3`; builds
local (no pool errors). Free disk stayed above 138 GB.

| Check | Result | Load average (start / end) |
|---|---|---|
| `runtime.py check --all-targets -p nuxie-render-api -p nuxie-scripting` | pass (before the revert); `check --all-targets -p nuxie-scripting` pass after it | 10.8 / 15.2 |
| `runtime.py test -p nuxie-scripting` | 39 / 39 targets pass (after the revert, and again after the `pass.rs` fix) | 22.4 / 11.3; 19.8 / 10.0 |
| `runtime.py test -p nuxie-render-api` | 6 / 6 pass (with the proof commit; net unchanged after the revert) | 15.2 / 14.0 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41 pass, 1 fails: `upstream_data_binding_images_direct::stateful_component_image_bind`, the known red on main | 14.0 / 9.1 |
| `make scripted-golden-compare` | exit 0, exact 348, diverges 16 | 11.3 / 22.8 |
| `make golden-compare` | exit 0, exact 356, diverges 8 | 22.8 / 15.7 |
| panic lints, three variants | 0 in these files; 415 / 414 / 414 elsewhere | 17.9 / 22.4 |
| `codex review --base levi/luaur-client-safe` | round 1: one P1 (the shader proof, accepted, reverted); round 2: no findings | |

`nuxie-ore-metal` and `nuxie-renderer` were not changed, so their tests were
not run. The `nuxie` tests and the golden compares ran on the tip before the
two-line `pass.rs` let-chain commit, which changes no behaviour.

## 6. Not verified

- No GPU on a device: the Metal and Vulkan paths were built and unit-tested
  on the macOS host only; the Vulkan SPIR-V decoder and the
  `android-authored-wgsl` code were compiled for the host (the `__android`
  lint build and the `cfg(test)` unit tests), not for an Android target.
- The texture-handle change (section 2) and a parts-binding shader proof
  (section 1) were not built.
- The `RefCell` scan is a heuristic over `let`-bound guards.

## 7. Where the design was wrong or incomplete

- 5.2 asked for a proof "produced where the digest is checked". A proof made
  there binds provenance to a digest, not the decoded parts to the bytes, so it
  cannot carry the old `unsafe` contract; building it made the constructors
  unsound and it was reverted (section 1). The site needs the decoder to own
  or return the parts inside `nuxie-render-api`. The same holds for
  `for_verified_artifact_digest_unchecked`, whose trust decision lives in
  `crates/nuxie`.
- 5.3 assumed the change fits "the trait and its implementations". The
  unsound part is the public-field `CanvasTextureInfo` that every backend
  issues and reinterprets, so the fix is in all four backends plus the
  deferred/replay layers (section 2).
- The lint counts understate the reachable panics in `gpu_canvas_ore`: the
  `textureViewBase().expect/unwrap` calls panic on a real runtime condition
  (wrong thread or non-view handle), not an internal invariant.
