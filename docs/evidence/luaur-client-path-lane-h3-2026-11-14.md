# Luau on clients, lane H3: renderer charges, shader assets, canvas handles (2026-11-14)

Branch `claude/luaur-lane-h3-renderer-7fd816`, based on `levi/luaur-client-stage1f`
(`b495c08c29`). Owned files: `crates/nuxie-renderer`, `crates/nuxie-render-api`,
`crates/nuxie-ore-metal`, and in `crates/nuxie-scripting` the `gpu_canvas*` files,
`shader_asset.rs` and `vm/lua_canvas.rs`. Not pushed.

**Status: done, with piece 3 reverted.** Lane H3 stopped at the 60 GB disk
line on 2026-11-14 with two open review findings. It resumed on 2026-11-15
when the coordinator freed disk and reported 137 GB free. Both findings are fixed, each with a
test that fails on the old code. Codex round 2 then found that the backend
type is not enough for the canvas handles (piece 3): a handle must also name
its device and keep that device alive. That needs a device identity in
Vulkan's wrap paths, which can only be compiled here, not run, so piece 3 is
reverted as the brief allows. Round 2's third finding, about pass charges,
is fixed. Round 3 found nothing. The full check battery passes on the final
tree.

## Commits

| Commit | Piece | Concern |
| --- | --- | --- |
| `a4f167065a` | 1 | ore: a frame recording keeps opaque owners until it is reset |
| `ad59cadad6` | 1 | scripting: GPU charges live as long as the frame recording holds the memory |
| `fadc71519e` | 2 | render-api: native shaders are built only by decoding the artifact they come from |
| `25d23424e3` | 3 | ore: canvas infos carry handles the backend issued, and wrapping them is safe (reverted) |
| `5625f6c47b` | 3 | renderer: a source handle names the backend whose context resolves it (round 1 finding 1; reverted) |
| `b6f298abed` | 1 | ore: an inline pass keeps what it is asked to keep with its recording (round 1 finding 2) |
| `e51696a804` | 1 | scripting: a pass keeps the charges of what it still holds itself (round 2 finding 3) |
| `7ba4f90f9d` | 3 | Revert of `5625f6c47b` |
| `b73c8c5d2e` | 3 | Revert of `25d23424e3`, with the reason in the message |

## Review findings and what happened to them

Round 1 (2026-11-14), both judged correct:

1. **[P1, piece 3] GL, Vulkan and WebGPU shared one handle type.** A GL
   image's handle downcast in the Vulkan context and reached Vulkan's
   unchecked `Texture2D` cast through a safe call.
   - Fixed in `5625f6c47b`. `SourceCanvas<I>` and `SourceTexture<I>` carry
     their issuer (`GlIssued`, `VulkanIssued`, `WebGpuIssued`), and each
     context resolves only its own.
     - An `ExactSourceBackend` names its issuer (`type SourceHandles`).
     - An image records its domain's texture issuer when it joins that
       domain. An unattached image, or one in Metal's domain, issues no
       source handle.
   - Test: the GL test `deferred_canvas_dispatch_stays_lazy_and_immediate_image_sampling_wraps_source`.
     - It builds a real GL context and, on macOS, a real Metal context in
       the same process.
     - Each refuses the other's handles.
     - The Vulkan and WebGPU resolvers refuse the GL handles, and the GL
       context refuses handles typed for them.
   - Fixed, then reverted with piece 3; see round 2.
2. **[P1, piece 1] `InlineDeferredRenderPass` dropped what it was asked to
   keep.**
   - Fixed in `b6f298abed`: it keeps the owner in its inline buffer, which
     lives until the pass drops after its replay.
   - Test: `an_inline_pass_on_an_immediate_context_keeps_owners_until_it_drops`,
     run on the immediate GL context through `beginRecordedRenderPass`. It
     fails on the old code ("the open inline pass keeps the owner").
   - Correction: the brief asked for the scripting loop test on an
     immediate context. That cannot run. `check_begin_pass`
     (`gpu_canvas_ore/canvas.rs`) refuses `beginRenderPass` unless the
     context records, so script passes always go to the deferred recorder
     and no script reached this gap. The fix guards the API for other
     callers. The test sits at the ORE level instead.

Round 2 (after the fixes above, all P1):

1. **Same-backend devices share a handle type.** Two `NativeVulkanFactory`
   instances both issue `SourceTexture<VulkanIssued>`, so a texture from the
   wrong device resolves. Vulkan's wrap then records into the receiving
   context's command buffer.
2. **A handle does not keep its execution owner alive.** A caller can keep
   the handle after dropping the info, the image and the factory. On Vulkan
   the device is then destroyed while the source resource is still alive.
3. **A retained pass loses the charges of what it still holds.** A bind
   group set before any pipeline waits in the pass's deferred groups. If the
   script finishes and keeps the pass, the reset gives its charges back
   while the pass still holds the memory. This repeats every frame.

Findings 1 and 2 need each handle to carry and check its device identity
and keep its execution owner. GL and WebGPU check an execution stamp
natively, but Vulkan has no device identity on its textures or targets, and
its paths can be compiled here but not run. The coordinator's instruction
covers this case, so piece 3 is reverted (`7ba4f90f9d`, `b73c8c5d2e`). The
wraps are explicit `unsafe` calls again, with their documented invariants,
as on the base.

Finding 3 is fixed in `e51696a804`. The pass holds each charge itself,
once per allocation, as well as handing it to the recording, so a charge
lasts until both let go.
- Test: `a_retained_pass_keeps_the_charges_of_what_it_still_holds_after_reset`
  opens the pass through `canvas:beginRenderPass` on the recorder's frame.
  It fails when the pass's own set is disabled ("the retained pass still
  keeps the deferred group after the reset").

Round 3, on the final tree: "No actionable regressions were identified".

What a later attempt at piece 3 needs:
- a per-device identity on Vulkan textures and render targets, checked in
  `wrapRiveTexture`, `wrapCanvasTexture` and `wrapRenderTarget`, as GL's
  `belongs_to_owner_thread_execution` does;
- handles that retain the issuing execution owner and drop the source
  first;
- a run on Android to check it.

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
  - `keep_with_recording(ctx, &GpuCharge)` hands a charge to the recording.
    `Pass::hold` / `hold_group` hand it to the pass's recording and also
    keep it in the pass's own `HeldCharges` (`e51696a804`).
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

Limits that remain:

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

## Piece 3: CanvasTextureInfo carries opaque handles (D4) — reverted

`25d23424e3` replaced the `*mut c_void` fields with backend-issued
`Rc<dyn Any>` handles and made the wraps safe. `5625f6c47b` gave each backend
its own handle type. Both are reverted, for the reasons under "Review
findings". The branch is back to the base API, so the canvas wraps stay
`unsafe`. D4's estimate of about 17 touched files was low: the change
touched 21, because the wrap methods' callers and implementors also had to
change.

## `unsafe` counts

`tools/ratchets/unsafe-count.sh` tracks nuxie-scripting: 44 now against
baseline row 47 (down 3, from piece 2). It passes, and `tests/ratchets/*`
was not edited. For the renderer crates the table counts `unsafe` words in
`.rs` files, at the base and now:

| Crate | Base `b495c08c29` | Now |
| --- | --- | --- |
| nuxie-render-api | 16 | 12 |
| nuxie-ore-metal | 265 | 265 |
| nuxie-renderer | 3972 | 3972 |
| tools/runtime-test-support | 9 | 9 |

No crate went up. With piece 3 reverted, only piece 2's removals remain.

The brief's target was that only `vm::luau_audio_web::host` remains in
nuxie-scripting. That cannot hold as written. The remaining sites are:

- `luau_audio_web/host.rs` (8);
- test-only FFI compile helpers in `lua_path.rs`, `lua_audio_processors.rs` and
  `luau_audio_processor_engine.rs` (H1/H2 files), and in `gpu_canvas.rs`;
- the test `provenance()` helper in `shader_asset.rs`;
- the test contexts' raw-wrap `unsafe fn` stubs in
  `script_call_gpu_scope_tests.rs` and `upstream_scripting_gpu_features.rs`.
  These go only if the raw wraps leave `ContextApi`;
- with piece 3 reverted, the canvas and `Image:view()` wrap sites in
  `gpu_canvas_ore/canvas.rs` and `mod.rs`, as on the base.

## Checks (final tree `b73c8c5d2e`, 2026-11-15)

Load averages are 1/5/15 minutes, at the end of each step. Free disk stayed
between 117 and 131 GB.

| Check | Result | Load |
| --- | --- | --- |
| `runtime.py test -p nuxie-renderer --features renderer-metal` | 929 pass, 9 fail: the same 9 as the base (diffed) | 23.8 / 17.5 / 16.0 |
| `runtime.py test -p nuxie-renderer --no-default-features --features renderer-webgl2` | 639 pass, 1 fail: the same 1 as the base; +1 is the new inline-pass test | 22.9 / 18.2 / 16.3 |
| `runtime.py test -p nuxie-scripting` | pass | 13.3 / 17.6 / 16.5 |
| `runtime.py test -p nuxie-ore-metal` | pass | 13.7 / 17.5 / 16.5 |
| `runtime.py test -p nuxie-render-api` | pass | 14.8 / 17.7 / 16.6 |
| `runtime.py test -p runtime-test-support` | pass | 16.3 / 17.9 / 16.7 |
| `runtime.py check --locked -p nuxie-renderer --no-default-features --features renderer-vulkan` / `renderer-webgpu` | pass (host build) | 15.8 / 17.7 / 16.6 |
| `runtime.py check -p nuxie-renderer --no-default-features` | pass | 16.4 / 17.8 / 16.7 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41 of 42 targets pass. `upstream_data_binding_images_direct::stateful_component_image_bind` fails the same way on the base `b495c08c29` (run separately) | 12.0 / 16.4 / 16.4 |
| `panic-lints.sh` for nuxie-scripting, `__apple`, `__android` | PASS, 0 forced sites each | 28.3 / 20.3 / 17.8 |
| `unsafe-count.sh` | PASS | 30.1 / 20.8 / 18.0 |
| `generate.py --check` | exit 0, nothing stale | 30.1 / 20.8 / 18.0 |
| `runtime.py check --locked -p nux-capi --no-default-features --features android-vulkan,scripting,android-authored-wgsl` | pass | 44.3 / 28.6 / 21.3 |
| `make golden-compare` | exact 356, diverges 8 | 29.1 / 27.6 / 21.8 |
| `make scripted-golden-compare` | exact 348, diverges 16 | 37.9 / 34.3 / 26.1 |
| `codex review --base levi/luaur-client-stage1f`, round 3 | no findings | — |

The `inline` and `HeldCharges` fixes passed the same battery before piece 3
was reverted. The handle fix `5625f6c47b` also passed it then, with the
renderer, golden and `nuxie` results above.

The renderer reds that also fail on the base:

- Metal: `gradients::{gradienttransform, gradienttypes}`,
  `stroke_position_shapes::{stroke_position_shapes, stroke_position_winding}`,
  `strokes::{stroke_position, stroke_position_clipped}`,
  `draw_cpp::…::source_path_draw_wang_vector_xform_preserves_pinned_segment_boundary`,
  `draw_shader::…::generated_namespace_inventory_has_the_pinned_namespaces`, and
  `native_metal::tests::source_render_context_drops_members_in_reverse_then_base`.
- GL: the same `draw_cpp` test.

`generate.py --check` "exit 2" was a misreading. In that earlier command,
`generate.py --check | tail -3` printed nothing, and the `2` came from the
`runtime.py test` run after it, which had three failing tests at that point.
`generate.py --check` exits 1 when files are stale and has never been
anything but clean on this branch.

Not available on this machine: the renderer golden gates
`renderer-golden-same-runner` and `renderer-metal-cpp-parity`. They need the
Dawn live-reference replay and the upstream Metal archives. Android Vulkan
and browser (wasm) builds were not run; Vulkan, WebGPU and WebGL2 were only
compiled for the host.

## Not verified

- Piece 1 on Vulkan and WebGPU at run time.
- The `stateful_component_image_bind` red. It is not this lane's: it fails
  identically on the base, and the test serializes a recording stream
  against a pinned silver without the renderer.

## State left behind

`bazel clean --expunge` was run in the worktree. The copied
`crates/nuxie/tests/fixtures/` and any `target/` were deleted. Nothing is
pushed.
