# Luau on clients, lane H3: renderer charges, shader assets, canvas handles (2026-11-14)

Branch `claude/luaur-lane-h3-renderer-7fd816`, based on `levi/luaur-client-stage1f`
(`b495c08c29`). Owned files: `crates/nuxie-renderer`, `crates/nuxie-render-api`,
`crates/nuxie-ore-metal`, and in `crates/nuxie-scripting` the `gpu_canvas*` files,
`shader_asset.rs` and `vm/lua_canvas.rs`. Not pushed.

**Status: stopped early, with two open P1 review findings.** Free disk fell to
59 GB (the brief's stop line is 60 GB) during the final check battery. The
lane's own Bazel output base is 5.6 GB, so other work on the machine caused the
drop. The first `codex review` round then found two real defects, listed under
"Open findings". Neither is fixed, because fixing needs builds. The golden
compares, the `nuxie` suite, panic-lints and later review rounds were not run.

## Commits

| Commit | Piece | Concern |
| --- | --- | --- |
| `a4f167065a` | 1 | ore: a frame recording keeps opaque owners until it is reset |
| `ad59cadad6` | 1 | scripting: GPU charges live as long as the frame recording holds the memory |
| `fadc71519e` | 2 | render-api: native shaders are built only by decoding the artifact they come from |
| `25d23424e3` | 3 | ore: canvas infos carry handles the backend issued, and wrapping them is safe |

## Open findings (codex review round 1, both judged correct)

1. **[P1, piece 3] One handle type is shared by GL, Vulkan and WebGPU.**
   `ore_source_handle.rs` issues the same private `SourceTexture` and
   `SourceCanvas` types for every non-Metal backend. In a build with more than
   one of them (WebGPU already compiles the GL module), a GL image's handle
   downcasts in the Vulkan context. It then reaches Vulkan's `wrapRiveTexture`,
   which casts it to Vulkan's `Texture2D` with no check. Before this commit the
   same mix-up needed an `unsafe` call. Now a safe call reaches it, so the
   `SAFETY` claim in `wrap_source_texture` ("a source gpu::Texture is what every
   backend's raw wrap reads") is false for Vulkan and WebGPU. GL itself checks
   the execution domain.
   **Fix:** have the handle record which backend issued it. Either the producer
   stores its execution anchor (the `RiveRenderImageHandle` execution domain)
   and each context compares it to its own, or each backend gets its own handle
   types. The test: a handle from one backend's image is refused by another
   backend's context.
   **Alternative:** if that cannot be built or verified, revert `25d23424e3`.
   The base is then sound in the same way it was before (the obligation sits
   with the `unsafe` callers).

2. **[P1, piece 1] Inline passes drop the charges.** For contexts that do not
   record (immediate GL, Vulkan, WebGPU), `beginRecordedRenderPass` returns
   `InlineDeferredRenderPass` (`ore_cmd/ore_deferred_render_pass.rs:75`). That
   type does not override `RenderPassApi::keepWithRecording`, so it uses the
   default, which drops the owner. Commit `ad59cadad6` removed the pass's own
   `HeldCharges`. On these contexts, a bound buffer's charge is therefore
   refunded when its userdata is collected, while the inline command buffer
   still keeps the allocation in `keepAlive`. A loop that allocates, binds and
   collects buffers inside one open pass gets past the budget.
   **Fix:** in `InlineDeferredRenderPass`, forward `keepWithRecording` to
   `state.buffer.borrow_mut().keep(owner)`, so the inline buffer holds the
   owners until it resets. The test: the H3 loop test run on an immediate
   (non-recording) context.

## Piece 1: the frame recording holds the charges (G2 section 4)

How it works now:

- `OreCommandBuffer::keep(owner: Rc<dyn Any>)` stores the owner, keyed by its
  address so the same owner is kept once. `reset()` clears the owners;
  `keptOwners()` reports how many are held.
- `ContextApi::keepWithRecording` and `RenderPassApi::keepWithRecording` are
  trait defaults that drop the owner.
  - `DeferredOreContext` and `RenderPassRecording` forward it to the stream
    they record into.
  - `OwnedContext`, `ObservedContext` and `ObservedPass` forward it to the
    context or pass they wrap.
- The renderer never looks inside an owner and knows nothing about budgets.
- Scripting:
  - `keep_with_recording(ctx, &GpuCharge)`, and `Pass::hold` / `hold_group`,
    replace `HeldCharges`.
  - New GPU buffers and textures keep their charge.
  - So do the copies made by `GPUBuffer.new` with data, `Buffer:write` and
    `Texture:upload`, each charged at its length.
  - Canvas backings and pass attachments are kept the same way.

Tests:

- Renderer: `the_context_keeps_an_owner_until_reset_frame`,
  `a_pass_hands_its_owner_to_the_stream_it_records_into` and
  `keeping_the_same_owner_twice_keeps_it_once`, all passing.
- Scripting: `a_frame_that_loops_create_bind_drop_is_refused_at_the_budget`
  sets a 64 KiB budget and runs four loops: pass bind, `Buffer.new` with data,
  `write`, and `upload`. Each loop must be refused within 64 rounds, and
  `resetFrame` must return the charges to the starting level.
- With `keep` disabled, three scripting tests fail, so the new tests fail on the
  old code.

Limits that remain (besides open finding 2):

- The real allocation lives one frame past reset, until its destroy replays.
  The charge is refunded at reset.
- Shader-module blob copies (E1 row 8) are not charged.
- On immediate contexts the upload-copy charges last only for the call.
- `b:write(src)` with its default arguments raised "destination range exceeds
  buffer size" even though `b.size == 1024`. The test passes the arguments
  explicitly. The default-argument behaviour is not explained; it is a
  follow-up for the binding's owner.

## Piece 2: the RSTB decoder moved into render-api (D4)

How it works now:

- New `nuxie_render_api::shader_asset` holds:
  - the RSTB v4 constants;
  - `ShaderAssetError`;
  - `RstbIndex` (`parse`, `variant`, `texture_sampler_pairs`,
    `entry_bindings`, `supplemental_reflection`);
  - the decoders that moved from scripting.
- `GpuCanvasVulkanSpirVShader::decode` and `GpuCanvasAppleMetalShader::decode`
  build a native shader only from the artifact bytes plus a provenance that
  `authorizes_artifact` (the artifact's SHA-256). The decoded value is the
  proof.
- Removed: both `unsafe fn from_verified_parts`, `authorizes_digest`, and the
  scripting `unsafe` blocks with their `allow`s.
- `sha2` moved from optional scripting dependency to a render-api dependency.
  Both crate universes were repinned (the lockfile diffs are minimal).
- `GpuCanvasShaderProvenance::for_verified_artifact_digest_unchecked` stays
  `unsafe`. Its product caller is `crates/nuxie/src/script_import.rs:718`,
  outside the lane.

There are now two RSTB indexers, the runtime's and render-api's.
`the_render_api_index_selects_what_the_runtime_index_selects` checks that they
select the same things. Follow-up: the runtime `ShaderAsset` could use
render-api's index.

## Piece 3: CanvasTextureInfo carries opaque handles (D4)

How it works now:

- `CanvasTextureInfo.canvas` and `CanvasTextureInfo.texture` are
  `Option<Rc<dyn Any>>` handles issued by the producing backend. They used to
  be `*mut c_void`.
  - Metal issues its `MetalRenderCanvasBridge` and `MetalRiveTextureBridge`.
  - The source `RiveRenderImage` and `RenderCanvas` issue private
    `ore_source_handle` types.
- `wrapCanvasTextureInfo`, `wrapCanvasSampleView` and `wrapImageSampleView` are
  safe and default to `None`.
  - Metal downcasts the handle to its own bridge and resolves it through new
    safe cores, `mtlWrapTargetHost` and `wrapRiveTextureHost`.
  - GL, Vulkan and WebGPU resolve through `ore_source_handle::wrap_source_*`.
    Each has one `unsafe` call into the raw wrap, and open finding 1 applies
    here.
  - The deferred, owned and observed contexts forward the calls. The deferred
    context keeps its tripwire for an immediate sample wrap while recording.
- The replay's image path now wraps through `wrapImageSampleView` instead of the
  raw `wrapRiveTexture`.
- The raw `wrapCanvasTexture` and `wrapRiveTexture` stay on the trait. Removing
  them would also remove the test contexts' `unsafe fn` stubs, including the 4
  in scripting. That was not done.
- Nothing changes in the public C API or the replay format: neither referenced
  these pointers.

Tests:

- `a_metal_context_wraps_only_the_handles_metal_issued` (new): made-up handles,
  a handle holding the native texture's address, and a whole info nested as a
  handle are all refused. A texture bridge in the canvas slot is refused.
- The GL test `deferred_canvas_dispatch_stays_lazy_and_immediate_image_sampling_wraps_source`
  was extended: made-up handles and a canvas handle in the texture slot are
  refused.
- On the old code these cases cannot be written without `unsafe`, because the
  fields were addresses.

Where D4's estimate was off: it named about 17 files that mention
`CanvasTextureInfo`. The change touched 21, because the wrap methods' callers
and implementors (the backend contexts, the replay, the forwarding contexts and
the tests) also had to change.

## `unsafe` counts (`tools/ratchets/unsafe-count.sh`)

| Crate | Base `b495c08c29` | After |
| --- | --- | --- |
| nuxie-scripting (baseline row 47) | 46 | 41 |
| nuxie-render-api | 13 | 11 |
| nuxie-ore-metal | 260 | 252 |
| nuxie-renderer | 3960 | 3946 |
| tools/runtime-test-support | 9 | 5 |

No crate went up, and the ratchet passes. `tests/ratchets/*` was not edited (no
`--update`).

The brief's target was that only `vm::luau_audio_web::host` remains in
nuxie-scripting. That cannot hold as written. The remaining sites are:

- `luau_audio_web/host.rs` (8);
- test-only FFI compile helpers in `lua_path.rs`, `lua_audio_processors.rs` and
  `luau_audio_processor_engine.rs` (H1/H2 files), and in `gpu_canvas.rs`;
- the test `provenance()` helper in `shader_asset.rs`;
- the test contexts' raw-wrap `unsafe fn` stubs in
  `script_call_gpu_scope_tests.rs` and `upstream_scripting_gpu_features.rs`.
  These go only if the raw wraps leave `ContextApi`.

## Checks

Load averages are shown as 1/5/15 minutes.

| Check | Result | Load |
| --- | --- | --- |
| `runtime.py test -p nuxie-ore-metal` | pass | 12.9 / 16.8 / 18.2 |
| `runtime.py test -p nuxie-render-api` | pass | 15.3 / 17.1 / 18.2 |
| `runtime.py test -p nuxie-scripting` (after piece 3) | pass | 10.8 / 11.7 / 14.5 |
| `runtime.py test -p runtime-test-support` | pass | 12.1 / 11.9 / 14.5 |
| `runtime.py test -p nuxie-renderer --features renderer-metal` | 930 pass, 9 fail; the base has the same 9 (926 pass), so +4 new tests | 10.1 / 11.4 / 14.1 (base run: 37.2 / 19.9 / 16.7) |
| `runtime.py test -p nuxie-renderer --no-default-features --features renderer-webgl2` | 638 pass, 1 fail; the base has the same 1 (635 pass) | 14.1 / 12.2 / 14.2 (base run: 32.6 / 27.0 / 20.4) |
| `runtime.py check --locked -p nuxie-renderer --no-default-features --features renderer-vulkan / renderer-webgpu / renderer-webgl2` | pass (host build) | 34.0 / 21.1 / 18.1 at the end |
| `runtime.py check -p nuxie-renderer --no-default-features` | pass | — |
| `unsafe-count.sh` | PASS | — |

The renderer reds that also fail on the base:

- Metal: `gradients::{gradienttransform, gradienttypes}`,
  `stroke_position_shapes::{stroke_position_shapes, stroke_position_winding}`,
  `strokes::{stroke_position, stroke_position_clipped}`,
  `draw_cpp::…::source_path_draw_wang_vector_xform_preserves_pinned_segment_boundary`,
  `draw_shader::…::generated_namespace_inventory_has_the_pinned_namespaces`, and
  `native_metal::tests::source_render_context_drops_members_in_reverse_then_base`.
- GL: the same `draw_cpp` test.

The failure sets at base and after are identical (diffed).

Not run, because the disk stop came first:

- `runtime.py test -p nuxie --features scripting,test-support` (Bazel was
  stopped while starting);
- `make golden-compare` and `make scripted-golden-compare`;
- `tools/ratchets/panic-lints.sh` for `nuxie-scripting` and its `__apple` and
  `__android` variants;
- `generate.py --check` (it exited 2 after the piece 2 repin and was not
  looked into);
- `check -p nux-capi … android-authored-wgsl`;
- `codex review` rounds 2 to 4.

Not available on this machine: the Makefile's renderer golden gates,
`renderer-golden-same-runner` and `renderer-metal-cpp-parity`. They need the
Dawn live-reference replay and the upstream Metal archives, which do not exist
here and would take multi-GB bootstraps. Android Vulkan and browser (wasm)
builds were not run; the Vulkan, WebGPU and WebGL2 code was only compiled for
the host.

## Not verified

- Rendering output is unchanged: the golden compares were not run. Piece 3
  changes the wrap plumbing, and piece 2 changes where shaders are decoded.
- Piece 3 on Vulkan and WebGPU at run time: they were only compiled.
- Open findings 1 and 2.

## State left behind

- The Bazel output base (5.6 GB) and the copied fixtures
  (`crates/nuxie/tests/fixtures/`, untracked, not committed) are still in
  place, so the lane can resume after disk is freed.
- `bazel clean --expunge` and deleting the fixtures are still owed when the
  lane finishes.
