# Luau client path, lane D2: value and renderer bindings panic-free (2026-11-14)

Design: `docs/luaur-client-path-safe.md` section 6 (lane D2), section 12.
Survey: `docs/evidence/luaur-client-path-survey-2026-11-14.md` (S).
Inventory: `docs/evidence/luaur-client-path-lane-e1-inventory-2026-11-14.md`
(E1). Branch `claude/luaur-lane-d2-values-8a750f`, from `d15e2cfbeb`
(= `levi/luaur-client-stage1c`, lanes A, B, C.1, D1, D4, E2a in).

Files (all `crates/nuxie-scripting/src/vm/`): `lua_mat4.rs`,
`lua_mat4_safe.rs`, `lua_mat2d.rs`, `lua_vec2d.rs`, `lua_canvas.rs`,
`lua_renderer.rs`, `lua_renderer_library.rs`, `lua_artboards.rs`,
`lua_transition.rs`, `lua_video.rs`, `lua_color.rs`, `lua_math.rs`,
`lua_image.rs`, `lua_blob.rs`, `lua_input.rs`. The design table's other D2
files (`lua_text.rs`, `lua_mesh.rs`, `lua_path.rs`) went to lane E2b and
`buffer_ext.rs` to lane E2a, so they are not in this lane.

## Result

Forced panic-lint sites (the seven lints), measured with
`tools/ratchets/panic-lints.sh --baseline <scratch copy of
tests/ratchets/panic-lints-baseline.txt> crates/nuxie-scripting
'crates/nuxie-scripting:nuxie-scripting__publisher@wasm'`, counted per file
from the two `*.clippy.diagnostics` files the run leaves under
`bazel-out/darwin_arm64-fastbuild/bin/crates/nuxie-scripting/` (primary
span of each diagnostic whose code is one of the seven).

| File (`crates/nuxie-scripting/src/vm/`) | native before | native after | wasm before | wasm after |
|---|---:|---:|---:|---:|
| `lua_mat4.rs` | 57 | 0 | 57 | 0 |
| `lua_mat4_safe.rs` | 5 | 0 | 5 | 0 |
| `lua_mat2d.rs` | 4 | 0 | 4 | 0 |
| `lua_vec2d.rs` | 5 | 0 | 5 | 0 |
| `lua_canvas.rs` | 3 | 0 | 3 | 0 |
| `lua_video.rs` | 3 | 0 | 3 | 0 |
| `lua_blob.rs` | 2 | 0 | 2 | 0 |
| `lua_artboards.rs` | 1 | 0 | 1 | 0 |
| `lua_renderer.rs`, `lua_renderer_library.rs`, `lua_transition.rs`, `lua_color.rs`, `lua_math.rs`, `lua_image.rs`, `lua_input.rs` | 0 | 0 | 0 | 0 |
| **These files** | **80** | **0** | **80** | **0** |
| Other files of the crate (unchanged, per file) | 132 | 132 | 131 | 131 |
| **Crate row** | **212** | **132** | **211** | **131** |

By lint, crate row after (native / wasm): `arithmetic_side_effects` 57 / 57
(was 92 / 92), `indexing_slicing` 56 / 56 (95 / 95), `expect_used` 10 / 9
(13 / 12), `unwrap_used` 3 / 3 (6 / 6), `unreachable` 5 / 5,
`unwrap_in_result` 1 / 1, `panic` 0 / 0. Ratchet rows to set at merge (not
edited here): `crates/nuxie-scripting` 132,
`crates/nuxie-scripting:nuxie-scripting__publisher@wasm` 131, per lint as
above. No new compiler or clippy warning (set difference of the non-lint
diagnostics before and after: empty).

Clippy does not flag a constant index into a fixed-size array (`m[5]` on a
`[f32; 16]`), which the compiler bounds-checks at build time. Those are left
as they are (`invert`, `invert_affine`, the constructors, `lookAt`); every
counted site was a computed index or integer arithmetic.

## `lua_mat4.rs` and `lua_mat4_safe.rs`

One module in two files, not two implementations. `lua_mat4.rs` holds the
matrix type (`ScriptedMat4`, `[f32; 16]` column-major) and the math
(`multiply`, `multiply_affine`, `invert`, `invert_affine`, `transpose`,
`transform_vec4`, the `mNN` name resolver), and includes
`lua_mat4_safe.rs` as its child module `safe` (`#[path]`), which is the
binding on the safe VM (tag metatable, direct fields, the `Mat4` table) and
calls the parent's math. The vendored-VM binding that used to sit beside it
was deleted in Phase 3. Neither supersedes the other; there is no dead code
to remove. Every math function is reached from the binding.

## Shape changes

| Commit | Files | Change |
|---|---|---|
| `7cc70cc59e` | `lua_mat4.rs`, `lua_mat4_safe.rs` | A matrix is read as four destructured columns (`columns`/`from_columns`); `by_row(columns, f)` builds a column from each row's four entries. `multiply` is `columns(rhs).map(|[x, y, z, w]| by_row(lhs, |a, b, c, d| a * x + b * y + c * z + d * w))`, `multiply_affine` the same with three terms for columns 0 to 2 and `+ d` for column 3, `transform_vec4` one `by_row`, `transpose` a destructure. Every expression keeps its operands and left-to-right order, so results are bit-identical (rustc never contracts `a * b + c`). `mNN` names resolve by position in a 16-entry name table; the strtol fallback takes one leading ASCII whitespace or `+`, then 1 or 2 ASCII digits parsed with `str::parse`, then `checked_sub(1)` for values 1 to 16. `Mat4.values` numbers its arguments with `zip(1..)`. Tests: the indexed loops and the old resolver are kept in `#[cfg(test)]` as references; all 1- and 2-byte names and all `m` + 2-byte names resolve identically, and multiply, multiplyAffine, transpose and transformVec4 are bit-equal on 66 matrices (random exponents, signed zeros, infinities, NaN) pairwise. |
| `47fd1f5d5f` | `lua_mat2d.rs`, `lua_vec2d.rs` | `Mat2D` fields go through an `Element` enum (`Xx`..`Ty`) whose `slot` destructures `&mut [f32; 6]`; the name and Lua-index accessors are generated from one `Element::ALL` table (`(1i64..).zip`). `multiply_mat2d` destructures both operands; the `mul_add` calls are the same calls on the same operands. `Vector:writeToBuffer`/`writeVec4` collect the 12 or 16 native-order bytes by chaining the components (and `w`), so the two `copy_from_slice` sites and the `bytes[..byte_len]` slice are gone; the range check is unchanged and uses the collected length. Vector component lookup is a `Component` enum matched to `x()`/`y()`/`z()`. |
| `6836037750` | `lua_canvas.rs`, `lua_renderer.rs` | See "Canvas frames without a renderer" below. `replace_backing`'s `expect("present allocation device")` is gone: `new_backing(lua, bindings, context, ..)` takes the context it allocates from, `create` passes it from `if let Some(context)`, `satisfy_pending` from `self.render_context.insert(context)` (the context is still stored before the allocation, as before, so a failed allocation keeps it). `end_frame(userdata, frame)` takes the frame its caller took out of `self.frame` (`if let Some(frame) = canvas.frame.take()` in `close_orphan_canvas_frames`, `let Some(frame) = .. else { <existing endFrame error> }` in `endFrame`), so `take().expect("active Canvas frame")` is gone. |
| `6e75f17228` | `lua_video.rs`, `lua_blob.rs`, `lua_artboards.rs`, `lua_renderer_library.rs`, `lua_transition.rs` | `video:duration()`, `position()`, `state()`: `ScriptVideo::number(key)` returns the error "video property is unavailable" for `None` instead of `unwrap`. `None` is not reachable for these keys (`Values::get` finds every numeric key), but the type lives in `nuxie-runtime`. Blob scope ranking: `strip_prefix(scope).and_then(strip_prefix('/'))` instead of slicing at `scope.len() + 1`, and the rank is `2` or `0` instead of `u8 * 2` (same values). `artboard.children` numbers from `(1_usize..).zip`. `gpu_features` copies the `Features` value (it is `Copy`) out of the ORE context and ends the borrow before the 20 `Table::set` calls (E1 2(a) row 8). `ChildScope::drop` uses `try_borrow_mut` (below). |

No site was replaced by `x.get(i).ok_or(..)?` with a new error.

### Canvas frames without a renderer

`DeferredCanvasFrame::target`'s `expect("deferred host returned a null
renderer")` (E1 section 3) fired on the first renderer call after a
deferred host's `begin_canvas_content` returned `None`, as a caught
`rust panic: ...`.

Decision: the null is detected once, at `beginFrame`, where the host returns
it. The open frame is now an enum in `lua_canvas.rs`:

- `OpenFrame::Drawing(Rc<RefCell<Box<dyn RenderCanvasFrame>>>)`: as before.
  `DeferredCanvasFrame` now holds a `Box<dyn Renderer>`, not an `Option`, so
  it cannot exist without a renderer and `target()` is a plain borrow.
- `OpenFrame::WithoutRenderer { host, canvas }`: `beginFrame` still
  succeeds and still returns a `Renderer` userdata, but one created with no
  target (`ScriptedRenderer::create_canvas_userdata` now takes
  `Option<frame>`). Every call on it goes through `with_renderer_mut`, which
  answers a missing target with the binding's existing error
  `Renderer is no longer valid.`; `restore` with nothing saved keeps its
  existing "unbalanced" error first. `endFrame` (and the orphan sweep) still
  calls the host's `end_canvas_content`, exactly as
  `DeferredCanvasFrame::finish` did. Dropping the canvas with the frame open
  drops it without ending, as before.

What a script observes: only the formerly panicking calls change, from
`rust panic: deferred host returned a null renderer` to
`Renderer is no longer valid.`. Test:
`lua_canvas::tests::a_frame_the_deferred_host_gave_no_renderer_errors_on_every_call_and_still_ends`.
No shipped `DeferredCanvasHost` returns `None` today: `DeferredSession`
(`nuxie-renderer` `deferred_session.rs:696`), `SerializingCanvasHost` and
`SerializingFactory` (`nuxie-render-api` `serializing.rs`) always return a
renderer. Only test hosts and the `upstream_scripting_routing.rs` stub
return `None`. (The `begin_canvas_content` methods in `nux-capi`
`apple_metal.rs`/`android_vulkan/deferred.rs` and `deferred_replayer.rs`
that return `None` belong to the replay-side host trait, not this one.)
The trait allows it, so the binding handles it.

## What a script observes

Unchanged except where a panic was reachable:

- the null-renderer canvas calls above (new text: none; the existing
  `Renderer is no longer valid.`);
- `video:duration()`/`position()`/`state()` if `property_number` returned
  `None` (unreachable): new text `video property is unavailable`.

Found while testing: the old `mNN` resolver ended with
`(..).then_some(value - 1)`, which evaluates `value - 1` eagerly. For the
key `"0"` (`m["0"]`, `m[0]` through `__index` with a number coerced to a
string, or `m["+0"]`, `m[" 0"]`) `value` is 0 and the subtraction
underflowed: a panic in a debug or overflow-checked build, a discarded wrap
in release. Release results were "not a valid index" and still are; the new
code has no subtraction outside `checked_sub`.

## Lint-invisible sites in these files

- **Integer division**: none. Every `/` and `%` in these files is `f32`/`f64`
  (matrix math, `/ 255.0` in `lua_color.rs`). Shifts in `lua_color.rs` are
  by the constants 0, 8, 16, 24.
- **`copy_from_slice`**: the two in `lua_vec2d.rs` (S 3.3) are gone. None
  left in these files' shipped code.
- **`RefCell` guards across VM object creation or script calls**: E1 2(a)
  row 8 fixed (above). Read and clear: `lua_artboards.rs` `data` getter (the
  `if let` scrutinee guard ends before `create_scripted_view_model`; the
  write is a statement), the draw visitor's `failure`/`current` cells
  (statement borrows; `with_reborrowed_renderer` holds no guard across the
  visitor call), `VisitedDrawable::drawable`; `lua_transition.rs` (the child
  is cloned out before `child.draw`); `lua_canvas.rs` (host and canvas
  borrows call renderer and host code only; the ScriptedCanvas receiver is a
  facade `try_borrow`); `lua_renderer.rs` (all `try_borrow`, as lane C.1
  left it); `lua_blob.rs:83` keeps `native_file` borrowed across the asset
  scan, which runs runtime code (`with_file`, `with_downcast`, `rank`) and no
  Luau; `lua_image.rs` report sets are statement borrows. The factory used by
  `Artboard:draw` is a `PersistentFactoryContext` proxy that borrows per
  operation (`nuxie-render-api` `PersistentFactory`), so Luau run by the
  visitor can reach the factory.
- **`Drop`**: `ScriptedCanvas` (ends the renderer through `try_` paths,
  drops the open frame: renderer code), `InUse` (`Cell`), `RestoreTarget`
  (`try_borrow_mut`) cannot panic. `ChildScope::drop` did
  `borrow_mut().take()`; it is now `try_borrow_mut` (every other borrow of
  that cell is a statement temporary, so this never fails; if it did, the
  child would stay reachable instead of the drop panicking).
- **Recursion**: no recursive function over script or file data in these
  files (a self-name scan finds only `Renderer` delegations to another
  object and `RendererBindings` delegations to `PersistentRenderContext`).
- **Allocations a script sizes**: `Vector` (16 bytes) and `Mat4`
  writeToBuffer (64 bytes) are fixed. Reported below for E2b.

## Sites for other lanes

For lane E2b (`HostMemoryBudget`); not budgeted here:

1. `lua_artboards.rs:188` `artboard:instance(viewModel)` (E1 section 5 row
   19): each call builds a full runtime artboard instance
   (`instance_with_factory`) held by the returned `ScriptedArtboard`.
   Charge needed: the instance's host size at creation, refunded when the
   `ScriptedArtboard` payload drops. `nuxie-runtime` has no byte-size
   accessor for an artboard instance; a fixed per-instance charge (or a
   count cap beside the byte budget) would need that number or a proxy (the
   source artboard's object count).
2. `lua_canvas.rs` `allocate_script_render_canvas`, from `context:canvas(w,
   h)` and `Canvas:resize(w, h)` (E1 row 6): `w * h * 4` bytes of backing
   per canvas (GPU or recorder), no dimension cap on this path (the direct
   GPU path checks). Charge needed: `w * h * 4` (checked) at allocation,
   refunded when the backing `RenderCanvasHandle` drops (the old backing on
   resize, and the `Image` userdata keeps it alive), plus the same dimension
   cap the direct path uses. This is also lane D4's GPU follow-up (the ORE
   `Canvas.create`/`resize` reach the same function).

For lane D3 / other files: none needed for this lane's fixes.

## Checks

Load averages are `uptime` 1-minute values at start and end of each run;
other sessions shared the machine throughout.

| Check | Result | Load (start, end) |
|---|---|---|
| panic-lints before (`d15e2cfbeb`) | 212 / 211, PASS against the committed baseline | 9.03, 15.53 |
| panic-lints after | 132 / 131, these files 0 in both rows, other files unchanged | 9.19, 8.98 (first run, pre-format); re-run on the committed tip: 22.21, 16.33, same counts |
| `python3 tools/bazel/runtime.py test -p nuxie-scripting` | 39 of 39 targets pass (includes `mat4_nontrivial_f32_results_are_bit_exact_to_cpp_rive`, the new shape tests and the canvas test) | 10.27, 10.38 |
| `python3 tools/bazel/runtime.py test -p nuxie --features scripting,test-support` | 41 of 42 targets pass; the failure is `stateful_component_image_bind` (known red on main), nothing else | 9.32, 9.48 |
| `make luau-vm-diff` | 409 runs, 0 disagreements, bytecode versions 7 and 11, corpus reaches every admitted opcode; exit 0 | 9.20, 8.91 |
| `make scripted-golden-compare` | 364 entries: 348 exact, 16 diverges, 0 unsupported, 0 not-yet; exit 0 | 8.91, 19.11 |
| `make golden-compare` | 364 entries: 356 exact, 8 diverges, 0 unsupported, 0 not-yet; exit 0 | 19.11, 20.50 |
| `codex review --base levi/luaur-client-stage1c` | round 1: no findings | – |

`NUXIE_EXECUTION=local`, `RIVE_RUNTIME_DIR=/Users/levi/dev/worktrees/rive-runtime-pin-cd5351c3`.

## Not verified

- The null-renderer path is exercised with a test host only; no shipped
  host returns `None`.
- `-Cpanic=abort` runs (design section 2 item 5) were not part of this
  lane.
- The `nuxie` crate and goldens exercise the canvas, matrix and vector
  bindings only as far as the fixtures use them; the Mat4 and Mat2D
  equivalence is proven by the bitwise shape tests and the C++ bit-exact
  test, not by fixture coverage.

## Where the design was wrong or thin

- Section 6's D2 file list is stale: `lua_text.rs`, `lua_mesh.rs` and
  `lua_path.rs` moved to E2b and `buffer_ext.rs` to E2a; this lane got the
  smaller binding files instead. The survey's "about 165 sites" for D2 is
  80 for this file set.
- "A matrix is `[f32; 16]` read by destructuring" needed no change of the
  stored type: `ScriptedMat4` already was `[f32; 16]`. Only the computed
  index loops were sites; constant indices are not lint sites and were left.
- "A row or column index from a script is validated once into an enum or a
  bounded newtype": the Mat4 element index stays a `usize` from the name
  resolver, and the binding reads it with `get`/`get_mut` it already used
  (a 16-way enum would add a 16-arm match per access for no safety gain;
  the `Option` from `get` is the existing "not a valid index" path). Mat2D,
  with six named fields, does use an enum.
- The null renderer could not be made "every renderer call is the ordinary
  error" inside `DeferredCanvasFrame` itself: `RenderCanvasFrame::renderer`
  and the `Renderer` methods return no `Result` (`nuxie-render-api`). It is
  handled one level up, by giving the script a `Renderer` with no target.
