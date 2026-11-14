# Luau client path, lane G2 (GPU bindings), 2026-11-14

Branch `claude/luaur-lane-g2-gpu-30a6d2`, based on `levi/luaur-client-stage1c`
(`32db50aba8`). Files: `crates/nuxie-scripting/src/gpu_canvas.rs`,
`src/gpu_canvas_ore/*`, `src/vm/lua_canvas.rs`. Two small changes outside
them (section 6).

Commits (oldest first):

| Commit | What |
|---|---|
| `c0dbf2335b` | GPU canvases and textures are capped at 8,192 px a side and 16,777,216 px |
| `f9ad7f3f08` | GPU buffers, textures and canvas backings are charged to the VM's host memory budget |
| `94abaf14ac` | GPU descriptor reads pass on the draw call's renderer loan |
| `f3b93b1bca` | a cube texture's six faces count toward the pixel cap (codex round 1) |
| `fef57b5348` | a GPU render pass keeps the charges of what it binds (codex round 1) |
| `b6d96c84ba` | a pass keeps every buffer it bound and its attachments' charges (codex round 2) |
| `485d26ed18` | `depth24plusStencil8` is charged at Metal's 8 bytes a texel (codex round 3) |
| this file | evidence |

## Summary

| Item | Result |
|---|---|
| 1. Charge GPU allocations to `HostMemoryBudget` | **done** for buffers, textures and canvas backings. GPU and host memory share the one per-VM budget (64 MiB by default); no second budget. One gap is left: the deferred recording's lifetime (section 4) |
| 2. Size caps | **done**: 8,192 px a side, 16,777,216 px (w × h × layers) in total, ordinary Lua errors |
| 3. Renderer loan in descriptor metamethods | **done** in these files; no facade change was needed. `view_model.rs` (G3) still reads its canvas descriptors without the loan (section 6) |
| E1 rows 5, 6 | fixed |
| E1 rows 7, 8 | confirmed open, not fixed (section 5) |
| E1 row 9 | confirmed: the direct path has no product caller, only tests use it |
| Largest GPU charge any fixture reaches | **0 bytes** in the golden corpus (section 3) |
| Checks | all green, with the known red `stateful_component_image_bind` (section 7) |

## 1. Caps (item 2)

`gpu_canvas_ore/mod.rs`: `MAX_SCRIPT_GPU_DIMENSION = 8_192` (the image
decoder's `MAX_IMAGE_DIMENSION`) and `MAX_SCRIPT_GPU_PIXELS = 16_777_216` (the
decoder's 64 MiB `MAX_DECODED_IMAGE_BYTES` divided by 4). No fixture needed
more. `check_gpu_size(caller, w, h, layers)` runs before any charge or
allocation in:

- `context:gpuCanvas()` and `GPUCanvas:resize()` (`gpu_canvas_ore/canvas.rs`)
- `context:canvas()` and `Canvas:resize()` (`vm/lua_canvas.rs`)
- `GPUTexture.new`, with layers from `texture_layers`: a cube counts at least
  6 faces, anything else at least 1

Error texts are `"{caller}: width and height must be at most 8192 (got WxH)"`
and `"{caller}: WxHxL is more than 16777216 pixels"`.

The direct path (`gpu_canvas.rs`) keeps its own `MAX_GPU_CANVAS_DIMENSION`
of 2,048.

## 2. Charges (item 1)

Each charge is a `HostMemoryCharge` taken through `charge_gpu_bytes` **before
the backend is asked**, and held in an `Rc` (`GpuCharge`) by everything that
keeps the allocation alive. A refused charge raises the budget's ordinary
error, "script VM exceeded its N-byte host memory budget".

| Allocation | Charge | Held by |
|---|---|---|
| `GPUBuffer.new` | `size` | the `Buffer`, any `BindGroup` using it as a uniform buffer, any pass that bound it |
| `GPUTexture.new` | `texture_bytes(desc)`, below | the `Texture`, its views, bind groups using the views, passes that bound or attached them |
| `context:gpuCanvas()` backing (and each resize) | w × h × 4 | the `Canvas`, its `view` / `colorView`, the `ScriptedImage` made from it (`lua_image.rs`) |
| `context:canvas()` backing (and each resize) | w × h × 4 | the `ScriptedCanvas` and its image |

`texture_bytes` is an upper bound:

- Blocks: BC1 and ETC2 RGB8 are 4×4 blocks of 8 bytes. BC3, BC7, ETC2 RGBA8
  and ASTC 4×4 are 4×4 of 16. ASTC 6×6 and 8×8 are 6×6 and 8×8 of 16.
- Other formats are 1 texel at `textureFormatBytesPerTexel`, except
  `depth24plusStencil8`, which is 8 bytes (Metal on Apple GPUs stores it as
  `Depth32Float_Stencil8`).
- It sums every mip level; a level past 1×1 counts one block, without
  looping.
- It multiplies by layers (`texture_layers`) and by `sampleCount`.
- A 3D texture's depth is not halved per level.

A render pass's recording keeps everything it captured, including a rebound
slot's old resource. So the `Pass` holds each captured charge once: its
attachments, every vertex and index buffer, and every bind group it was ever
bound to, deduplicated by allocation.

Not charged, as constant-size backend objects: views (they share their
texture's charge), samplers, pipelines, bind group layouts and shader
modules.

**What a script now observes differently.** Each case is an ordinary Lua
error that `pcall` can catch:

- Sizes past either cap.
- GPU allocations that would take the VM's host memory past its budget. With
  the default 64 MiB budget, a 4,096 × 4,096 RGBA8 canvas or texture
  (64 MiB) leaves no room for anything else, so the budget, not the
  8,192 px cap, is the binding limit for RGBA8.
- Live GPU allocations now count against the same budget that images, fonts,
  paths and text use.

In-budget content behaves as before: the goldens are exact (section 7).

## 3. Largest charge a fixture reaches

I logged every successful `charge_gpu_bytes` call (bytes and budget used)
with a temporary patch that was not committed. Then I ran
`make scripted-golden-compare` (exact=348, diverges=16). The patch was
present in `target/debug/rust-golden-runner-scripted`, but **no charge was
recorded**.

Of the 355 corpus files, only `tests/unit_tests/assets/scope_probe.riv`
names the GPU API. The scripted runner gives it no GPU context, so none of
its calls reach a charge.

Not measured: `crates/nux-capi/tests/fixtures/android-gpu-canvas.riv`. It is
the only fixture that makes a real GPU canvas, sized from its scene; its test
runs on Android Vulkan only.

So the largest charge any fixture reaches is 0 bytes, and the charges above
are exercised by unit tests only.

## 4. Gap left: the deferred recording outlives its Lua handles (codex rounds 3 and 4, P1)

**Not fixed; needs a renderer change.** On the shipped deferred recorder
(`nuxie-renderer` `deferred/ore/ore_deferred_context.rs`), a pass records
into the context's shared stream (`render`, a `SharedOreCommandBuffer`). The
stream keeps two things until `resetFrame`:

- the real resource handles (`realResources`);
- the blob copies of buffer data and texture uploads (`recordMakeBuffer` and
  friends).

A script can therefore create a resource, bind it, finish the pass and let
everything be collected. The charge is then refunded while the frame still
holds the allocation. Repeated within one frame, this grows past the budget.
The only bound is the per-frame execution budget.

Scripting cannot see that stream:

- `RenderPassRecording` has no getter for its `cmd`.
- The deferred context is reached as `dyn ContextApi`, whose `pendingFrame()`
  is the base context's buffer, not `render`.

Either of these would close the gap:

1. **ORE (preferred):** let a recording keep an opaque owner token next to
   what it captures, for example `OreCommandBuffer::keep(Rc<dyn Any>)`,
   cleared in `reset`. The pass, and `GPUBuffer.new` / `Texture:upload` /
   `Buffer:write` on the recorder, would hand their `GpuCharge` to it. The
   charge then lives exactly as long as the copy.
2. **Smaller:** a public `RenderPassRecording::stream()` (or
   `ContextApi::recordingStream()`). Scripting could then keep the charges
   of collected passes in a per-VM list and drop entries once the stream's
   `keepAlive` (or `realResources`) no longer has their allocation identity.
   It would not cover blob copies.

Both changes are in `nuxie-ore-metal` / `nuxie-renderer`, outside this lane.

## 5. E1 inventory rows (section 5)

| Row | Inventory says | Found | Outcome |
|---|---|---|---|
| 5 | ORE GPU resources: no budget; up to 4 GiB a buffer | confirmed | **fixed** (sections 1–2) |
| 6 | GPU canvas: no dimension cap | confirmed, and also true of `context:canvas` in `lua_canvas.rs` | **fixed** |
| 7 | `Buffer:write`, `Texture:upload` bytes; "whether the recorder retains copies until replay was not verified" | **verified: it does.** `recordBufferUpdate` / `recordTextureUpload` (`ore_deferred_resource.rs`) copy into the stream until reset | open; same fix as section 4 option 1 |
| 8 | `context:shader(name)` compiles a new occurrence per call | confirmed: an occurrence per call from the prepared module, and a full compile when the factory does not match; no count cap | open |
| 9 | direct-path growth (`canvases`, `uploads`, `draws`) | confirmed, but the direct path has no product caller: `install_gpu_canvas_globals` installs the ORE path, and only `tests/gpu_canvas_inputs.rs` uses the direct one | not changed |

GPU notes:

- The inventory's "refunded on payload drop" is not enough on its own.
  Bind groups and passes keep resources alive past their userdata, so they
  hold the charges too (section 2), and the deferred recording does as well
  (section 4).
- The "GC pace follows the VM's tiny charge" problem is unchanged. The
  budget refuses new allocations; it does not make the collector run sooner.

## 6. Loan forwarding (item 3), and changes outside these files

All descriptor reads in `gpu_canvas_ore/*` go through `get_with(loan, key)`:

- the helpers `number`, `boolean`, `string`, `checked_string`,
  `optional_table`, `tables`, `stencil`, `resolve_stage_entry`,
  `dynamic_ubo_bindings`;
- every `X.new` constructor (`create_function_lent`);
- `Texture:view` / `:upload`, both `beginRenderPass` methods and
  `GPUSampler.new` (`add_method_lent`).

A descriptor whose `__index` calls `renderer:save()` during a draw now
reaches the draw's renderer (test
`gpu_descriptor_metamethods_reach_the_draw_renderer`). The direct path has
no renderer loan and passes `&mut None`.

**For G3** (`view_model.rs`, not changed): `context:canvas{width, height}`
and `context:gpuCanvas{...}` read their descriptor with plain `get`. They
need `add_method_lent` and `descriptor.get_with(reborrow_loan(&mut loan),
"width")`.

**Outside any lane, changed:**

- `vm/lua_image.rs`: `ScriptedImage` carries the canvas charge
  (`from_render_canvas(canvas, charge)`).
- `vm.rs`: `mod host_memory_budget` became `pub(crate)`.

**Codex findings in other lanes' files, not fixed, for their owners:**

| File (lane) | Finding |
|---|---|
| `lua_text.rs` (G4) | the text charge does not cover glyph outlines `RawText::update` appends (round 3, P1) |
| `lua_path.rs` (G4) | `p:measure()` and `contours()` storage is uncharged (rounds 1 and 3, P1) |
| `lua_path.rs` (G4) | `close()` on an empty or closed path raises a budget error under an exhausted budget (round 4, P2) |
| `lua_font.rs` (unowned) | a decoded font assigned to a view-model property loses its charge when the original userdata is collected (rounds 1 and 3, P2) |
| `lua_data_value.rs` (G3) | `checked_string` copies the string before the budget check (round 4, P2) |
| `lua_image_decode.rs` (unowned) | the decode charge is refunded before settlement callbacks; refund scheduling for cancelled decodes (round 2) |

## 7. Checks

All commands ran with `NUXIE_EXECUTION=local` and
`RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3`. The
full battery ran at `b6d96c84ba`. The last commit, `485d26ed18`, adds one
match arm and one assertion; the 18 GPU unit tests passed on it.

| Check | Result | Load average (1-minute, start → end) |
|---|---|---|
| `runtime.py test -p nuxie-scripting` | 39 / 39 targets pass | 13.8 → 11.7 |
| `runtime.py test -p nuxie --features scripting,test-support` | 41 / 42; the one red is the known `stateful_component_image_bind` (in `test_upstream_data_binding_images_direct`) | 11.7 → 7.1 |
| `make scripted-golden-compare` | exact=348, diverges=16, unsupported 0, not-yet 0 | 7.1 → 10.8 |
| `make scripted-golden-compare`, instrumented (section 3) | exact=348, diverges=16 | not recorded |
| `tools/ratchets/panic-lints.sh crates/nuxie-scripting 'crates/nuxie-scripting:nuxie-scripting__publisher@wasm'` | PASS, 0 forced sites on both rows | 13.8 |
| `tools/ratchets/unsafe-count.sh crates/nuxie-scripting` | PASS, 47 / 47 | 13.8 |
| `codex review --base levi/luaur-client-stage1c` | 4 rounds. Fixed in these files: cube cap (round 1), pass charges (rounds 1–2), depth-stencil size (round 3). Left: the deferred recording (rounds 3–4, section 4) and other lanes' files (section 6) | not recorded |

Tests that fail on the old code:

- A mutation run checked six of them. With the caps always passing, charges
  of 0 bytes and `reborrow_loan` returning `None`, these failed:
  - `texture_and_gpu_canvas_sizes_are_capped`
  - `canvas_sizes_are_capped_before_allocation`
  - `gpu_buffers_and_textures_are_charged_while_anything_holds_them`
  - `canvas_backings_are_charged_while_the_canvas_or_its_image_holds_them`
  - `gpu_allocations_past_the_budget_are_refused_before_the_backend`
  - `gpu_descriptor_metamethods_reach_the_draw_renderer`
- The later review-fix tests were not mutation-run; I checked them only by
  reasoning. On the old code:
  - the cube refusal in the caps test passes 2048 × 2048 × 6;
  - `a_pass_keeps_the_charges_of_what_it_binds` sees 1810 instead of 1830
    after rebinding, and 0 instead of 1024 for the attachment;
  - `gpu_texture_charges_cover_formats_layers_and_samples` sees 16 instead
    of 32 for `depth24plusStencil8`.

## 8. Not verified

- Real-device GPU memory against the charge formulas. They are upper bounds
  from the format tables, not measured backend allocations; Vulkan and Metal
  alignment padding could exceed them for small textures.
- `android-gpu-canvas.riv` (section 3).
- The full battery on `485d26ed18`.
